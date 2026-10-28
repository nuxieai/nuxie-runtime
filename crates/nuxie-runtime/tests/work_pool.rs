//! Direct ports of the 19 pool cases in
//! pinned `tests/unit_tests/runtime/work_pool_test.cpp`.
//!
//! Native WorkTask owns task state and is boxed directly into WorkPool. Shared
//! test records observe only callbacks; barriers make worker ordering explicit
//! without a substitute task state machine. The chained threaded delivery case
//! retains upstream's deadlines and worker-queue delay.

#[cfg(feature = "threading")]
use nuxie_runtime::WorkStatus;
use nuxie_runtime::{WorkCallbacks, WorkPool, WorkTask};
#[cfg(feature = "threading")]
use std::sync::Condvar;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Default)]
struct TestTask {
    should_succeed: AtomicBool,
    executed: AtomicBool,
    completed: AtomicBool,
    errored: AtomicBool,
    cancelled: AtomicBool,
    error_message: Mutex<String>,
}

impl TestTask {
    fn new() -> Self {
        Self {
            should_succeed: AtomicBool::new(true),
            ..Self::default()
        }
    }
}

struct TestCallbacks(Arc<TestTask>);

impl WorkCallbacks for TestCallbacks {
    fn execute(&mut self, error_message: &mut String) -> bool {
        self.0.executed.store(true, Ordering::Release);
        if !self.0.should_succeed.load(Ordering::Acquire) {
            *error_message = "test failure".into();
            return false;
        }
        true
    }

    fn on_complete(&mut self) {
        self.0.completed.store(true, Ordering::Release);
    }

    fn on_error(&mut self, error: &str) {
        self.0.errored.store(true, Ordering::Release);
        *lock_unpoisoned(&self.0.error_message) = error.to_owned();
    }

    fn on_cancel(&mut self) {
        self.0.cancelled.store(true, Ordering::Release);
    }
}

fn poll_until(pool: &mut WorkPool, mut done: impl FnMut() -> bool) {
    while !done() {
        pool.poll_completed_work(16);
        std::thread::yield_now();
    }
}

#[cfg(feature = "threading")]
#[derive(Default)]
struct BlockingState {
    started: bool,
    unblock: bool,
}

#[cfg(feature = "threading")]
#[derive(Default)]
struct BlockingTask {
    gate: Mutex<BlockingState>,
    changed: Condvar,
    completed: AtomicBool,
}

#[cfg(feature = "threading")]
impl BlockingTask {
    fn wait_until_started(&self) {
        let mut gate = lock_unpoisoned(&self.gate);
        while !gate.started {
            gate = self
                .changed
                .wait(gate)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn unblock(&self) {
        lock_unpoisoned(&self.gate).unblock = true;
        self.changed.notify_all();
    }
}

#[cfg(feature = "threading")]
struct BlockingCallbacks(Arc<BlockingTask>);

#[cfg(feature = "threading")]
impl WorkCallbacks for BlockingCallbacks {
    fn execute(&mut self, _error_message: &mut String) -> bool {
        let mut gate = lock_unpoisoned(&self.0.gate);
        gate.started = true;
        self.0.changed.notify_all();
        while !gate.unblock {
            gate = self
                .0
                .changed
                .wait(gate)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        true
    }

    fn on_complete(&mut self) {
        self.0.completed.store(true, Ordering::Release);
    }
}

#[cfg(feature = "threading")]
fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
        .min(4)
}

#[cfg(feature = "threading")]
fn block_workers(pool: &mut WorkPool, count: usize) -> Vec<Arc<BlockingTask>> {
    let blockers = (0..count)
        .map(|_| Arc::new(BlockingTask::default()))
        .collect::<Vec<_>>();
    for blocker in &blockers {
        pool.submit(Some(Box::new(WorkTask::new(BlockingCallbacks(
            blocker.clone(),
        )))));
    }
    for blocker in &blockers {
        blocker.wait_until_started();
    }
    blockers
}

#[cfg(feature = "threading")]
fn unblock_all(blockers: &[Arc<BlockingTask>]) {
    for blocker in blockers {
        blocker.unblock();
    }
}

#[cfg(not(feature = "threading"))]
struct ChainTask {
    callbacks: TestCallbacks,
    pool: Weak<WorkPool>,
    next: Option<Box<WorkTask<TestCallbacks>>>,
}

#[cfg(not(feature = "threading"))]
impl WorkCallbacks for ChainTask {
    fn execute(&mut self, error_message: &mut String) -> bool {
        self.callbacks.execute(error_message)
    }

    fn on_complete(&mut self) {
        self.callbacks.on_complete();
        self.pool
            .upgrade()
            .expect("test pool remains alive during delivery")
            .submit(self.next.take().map(|task| task as Box<_>));
    }
}

#[cfg(not(feature = "threading"))]
struct NestedPollTask {
    callbacks: TestCallbacks,
    pool: Weak<WorkPool>,
    nested: Arc<AtomicUsize>,
}

#[cfg(not(feature = "threading"))]
impl WorkCallbacks for NestedPollTask {
    fn execute(&mut self, error_message: &mut String) -> bool {
        self.callbacks.execute(error_message)
    }

    fn on_complete(&mut self) {
        self.callbacks.on_complete();
        let nested = self
            .pool
            .upgrade()
            .expect("test pool remains alive during delivery")
            .poll_completed_work(16);
        self.nested.store(nested as usize, Ordering::Release);
    }
}

#[cfg(not(feature = "threading"))]
#[test]
fn work_submitted_during_delivery_waits_for_the_next_poll() {
    let pool = Arc::new(WorkPool::default());
    let second = Arc::new(TestTask::new());
    let first = Arc::new(TestTask::new());
    pool.submit(Some(Box::new(WorkTask::new(ChainTask {
        callbacks: TestCallbacks(first.clone()),
        pool: Arc::downgrade(&pool),
        next: Some(Box::new(WorkTask::new(TestCallbacks(second.clone())))),
    }))));

    assert_eq!(pool.poll_completed_work(16), 1);
    assert!(first.completed.load(Ordering::Acquire));
    assert!(!second.executed.load(Ordering::Acquire));
    assert!(pool.has_pending_work());

    assert_eq!(pool.poll_completed_work(16), 1);
    assert!(second.completed.load(Ordering::Acquire));
    assert!(!pool.has_pending_work());
}

#[cfg(not(feature = "threading"))]
#[test]
fn a_poll_nested_in_a_completion_leaves_the_outer_poll_consistent() {
    let pool = Arc::new(WorkPool::default());
    let first = Arc::new(TestTask::new());
    let nested = Arc::new(AtomicUsize::new(0));
    let second = Arc::new(TestTask::new());
    pool.submit(Some(Box::new(WorkTask::new(NestedPollTask {
        callbacks: TestCallbacks(first.clone()),
        pool: Arc::downgrade(&pool),
        nested: nested.clone(),
    }))));
    pool.submit(Some(Box::new(WorkTask::new(TestCallbacks(second.clone())))));

    // The nested poll delivers the second task; the outer one stops at the
    // emptied queue rather than reading past it.
    assert_eq!(pool.poll_completed_work(16), 1);
    assert_eq!(nested.load(Ordering::Acquire), 1);
    assert!(first.completed.load(Ordering::Acquire));
    assert!(second.completed.load(Ordering::Acquire));
    assert!(!pool.has_pending_work());
}

#[cfg(not(feature = "threading"))]
#[test]
fn work_submitted_during_a_nested_poll_waits_for_a_later_poll() {
    let pool = Arc::new(WorkPool::default());
    let first = Arc::new(TestTask::new());
    let second = Arc::new(TestTask::new());
    let third = Arc::new(TestTask::new());
    pool.submit(Some(Box::new(WorkTask::new(NestedPollTask {
        callbacks: TestCallbacks(first),
        pool: Arc::downgrade(&pool),
        nested: Arc::new(AtomicUsize::new(0)),
    }))));
    pool.submit(Some(Box::new(WorkTask::new(ChainTask {
        callbacks: TestCallbacks(second.clone()),
        pool: Arc::downgrade(&pool),
        next: Some(Box::new(WorkTask::new(TestCallbacks(third.clone())))),
    }))));

    // The nested poll delivers the second task, which submits the third;
    // neither poll that was running then may run it.
    assert_eq!(pool.poll_completed_work(16), 1);
    assert!(second.completed.load(Ordering::Acquire));
    assert!(!third.executed.load(Ordering::Acquire));
    assert_eq!(pool.poll_completed_work(16), 1);
    assert!(third.completed.load(Ordering::Acquire));
}

#[cfg(feature = "threading")]
struct ChainWaitTask {
    callbacks: TestCallbacks,
    pool: Weak<WorkPool>,
    next: Option<Box<WorkTask<TestCallbacks>>>,
}

#[cfg(feature = "threading")]
impl WorkCallbacks for ChainWaitTask {
    fn execute(&mut self, error_message: &mut String) -> bool {
        self.callbacks.execute(error_message)
    }

    fn on_complete(&mut self) {
        self.callbacks.on_complete();
        let next = self.next.take().expect("chain task completes once");
        let status = next.status_handle();
        self.pool
            .upgrade()
            .expect("test pool remains alive during delivery")
            .submit(Some(next));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while status.status() != WorkStatus::Completed && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        // The worker queues it right after setting its status.
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[cfg(feature = "threading")]
#[test]
fn work_completing_during_delivery_waits_for_the_next_poll() {
    let pool = Arc::new(WorkPool::default());
    let second = Arc::new(TestTask::new());
    let first = Arc::new(TestTask::new());
    pool.submit(Some(Box::new(WorkTask::new(ChainWaitTask {
        callbacks: TestCallbacks(first.clone()),
        pool: Arc::downgrade(&pool),
        next: Some(Box::new(WorkTask::new(TestCallbacks(second.clone())))),
    }))));

    let mut delivered = 0;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while delivered == 0 && std::time::Instant::now() < deadline {
        delivered = pool.poll_completed_work(16);
    }
    assert_eq!(delivered, 1);
    assert!(first.completed.load(Ordering::Acquire));
    assert!(second.executed.load(Ordering::Acquire));
    assert!(!second.completed.load(Ordering::Acquire));

    assert_eq!(pool.poll_completed_work(16), 1);
    assert!(second.completed.load(Ordering::Acquire));
}

#[test]
fn work_pool_executes_task_on_poll() {
    let mut pool = WorkPool::default();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&mut pool, worker_count());
    let task = Arc::new(TestTask::new());
    pool.submit(Some(Box::new(WorkTask::new(TestCallbacks(task.clone())))));

    assert!(!task.executed.load(Ordering::Acquire));
    assert!(!task.completed.load(Ordering::Acquire));
    assert!(pool.has_pending_work());
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&mut pool, || {
        task.completed.load(Ordering::Acquire) && {
            #[cfg(feature = "threading")]
            {
                blockers
                    .iter()
                    .all(|blocker| blocker.completed.load(Ordering::Acquire))
            }
            #[cfg(not(feature = "threading"))]
            {
                true
            }
        }
    });

    assert!(task.executed.load(Ordering::Acquire));
    assert!(task.completed.load(Ordering::Acquire));
    assert!(!task.errored.load(Ordering::Acquire));
    assert!(!task.cancelled.load(Ordering::Acquire));
    assert!(!pool.has_pending_work());
}

#[test]
fn work_pool_delivers_on_error_for_failed_tasks() {
    let mut pool = WorkPool::default();
    let task = Arc::new(TestTask::new());
    task.should_succeed.store(false, Ordering::Release);
    pool.submit(Some(Box::new(WorkTask::new(TestCallbacks(task.clone())))));
    poll_until(&mut pool, || task.errored.load(Ordering::Acquire));

    assert!(task.executed.load(Ordering::Acquire));
    assert!(!task.completed.load(Ordering::Acquire));
    assert!(task.errored.load(Ordering::Acquire));
    assert_eq!(&*lock_unpoisoned(&task.error_message), "test failure");
}

#[test]
fn work_pool_respects_max_callbacks() {
    let mut pool = WorkPool::default();
    #[cfg(feature = "threading")]
    let reserved_workers = block_workers(&mut pool, worker_count().saturating_sub(1));
    let tasks = (0..3)
        .map(|_| Arc::new(TestTask::new()))
        .collect::<Vec<_>>();
    for task in &tasks {
        pool.submit(Some(Box::new(WorkTask::new(TestCallbacks(task.clone())))));
    }

    // Keep all but one worker occupied, then put a blocking fence behind the
    // tasks. When that sole worker reaches the fence, all three completions
    // are queued in submission order and the bounded-poll assertions cannot
    // race worker scheduling.
    #[cfg(feature = "threading")]
    let completion_fence = {
        let fence = Arc::new(BlockingTask::default());
        pool.submit(Some(Box::new(WorkTask::new(BlockingCallbacks(
            fence.clone(),
        )))));
        fence.wait_until_started();
        fence
    };

    let processed = pool.poll_completed_work(2);
    assert_eq!(processed, 2);
    assert!(tasks[0].completed.load(Ordering::Acquire));
    assert!(tasks[1].completed.load(Ordering::Acquire));
    assert!(!tasks[2].completed.load(Ordering::Acquire));
    assert!(pool.has_pending_work());

    assert_eq!(pool.poll_completed_work(1), 1);
    assert!(tasks[2].completed.load(Ordering::Acquire));
    #[cfg(feature = "threading")]
    {
        completion_fence.unblock();
        unblock_all(&reserved_workers);
        poll_until(&mut pool, || {
            completion_fence.completed.load(Ordering::Acquire)
                && reserved_workers
                    .iter()
                    .all(|task| task.completed.load(Ordering::Acquire))
        });
    }
    assert!(!pool.has_pending_work());
}

#[test]
fn cancel_all_for_owner_marks_tasks_cancelled() {
    let mut pool = WorkPool::default();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&mut pool, worker_count());
    let t1 = Arc::new(TestTask::new());
    let t2 = Arc::new(TestTask::new());
    let mut t1_work = WorkTask::new(TestCallbacks(t1.clone()));
    t1_work.set_owner_id(42);
    let mut t2_work = WorkTask::new(TestCallbacks(t2.clone()));
    t2_work.set_owner_id(99);
    pool.submit(Some(Box::new(t1_work)));
    pool.submit(Some(Box::new(t2_work)));

    pool.cancel_all_for_owner(42);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&mut pool, || {
        t1.cancelled.load(Ordering::Acquire) && t2.completed.load(Ordering::Acquire)
    });

    assert!(t1.cancelled.load(Ordering::Acquire));
    assert!(!t1.completed.load(Ordering::Acquire));
    assert!(!t1.executed.load(Ordering::Acquire));
    assert!(!t2.cancelled.load(Ordering::Acquire));
    assert!(t2.completed.load(Ordering::Acquire));
    assert!(t2.executed.load(Ordering::Acquire));
}

#[test]
fn on_cancel_is_delivered_exactly_once() {
    struct CountingTask {
        cancel_count: Arc<AtomicUsize>,
    }

    impl WorkCallbacks for CountingTask {
        fn execute(&mut self, _error_message: &mut String) -> bool {
            true
        }
        fn on_cancel(&mut self) {
            self.cancel_count.fetch_add(1, Ordering::AcqRel);
        }
    }

    let mut pool = WorkPool::default();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&mut pool, worker_count());
    let cancel_count = Arc::new(AtomicUsize::new(0));
    let mut task = WorkTask::new(CountingTask {
        cancel_count: cancel_count.clone(),
    });
    task.set_owner_id(1);
    pool.submit(Some(Box::new(task)));
    pool.cancel_all_for_owner(1);
    assert_eq!(cancel_count.load(Ordering::Acquire), 0);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&mut pool, || cancel_count.load(Ordering::Acquire) == 1);

    assert_eq!(cancel_count.load(Ordering::Acquire), 1);
}

#[test]
fn cancelled_owner_does_not_interfere_with_other_owners() {
    let mut pool = WorkPool::default();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&mut pool, worker_count());
    let owner_a1 = Arc::new(TestTask::new());
    let owner_a2 = Arc::new(TestTask::new());
    let owner_b1 = Arc::new(TestTask::new());
    let mut owner_a1_work = WorkTask::new(TestCallbacks(owner_a1.clone()));
    owner_a1_work.set_owner_id(10);
    let mut owner_a2_work = WorkTask::new(TestCallbacks(owner_a2.clone()));
    owner_a2_work.set_owner_id(10);
    let mut owner_b1_work = WorkTask::new(TestCallbacks(owner_b1.clone()));
    owner_b1_work.set_owner_id(20);
    pool.submit(Some(Box::new(owner_a1_work)));
    pool.submit(Some(Box::new(owner_b1_work)));
    pool.submit(Some(Box::new(owner_a2_work)));

    pool.cancel_all_for_owner(10);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&mut pool, || {
        owner_a1.cancelled.load(Ordering::Acquire)
            && owner_a2.cancelled.load(Ordering::Acquire)
            && owner_b1.completed.load(Ordering::Acquire)
    });

    assert!(owner_a1.cancelled.load(Ordering::Acquire));
    assert!(!owner_a1.completed.load(Ordering::Acquire));
    assert!(owner_a2.cancelled.load(Ordering::Acquire));
    assert!(!owner_a2.completed.load(Ordering::Acquire));
    assert!(!owner_b1.cancelled.load(Ordering::Acquire));
    assert!(owner_b1.completed.load(Ordering::Acquire));
    assert!(owner_b1.executed.load(Ordering::Acquire));
}

#[test]
fn cancel_only_affects_tasks_present_at_time_of_call() {
    let mut pool = WorkPool::default();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&mut pool, worker_count());
    let t1 = Arc::new(TestTask::new());
    let mut t1_work = WorkTask::new(TestCallbacks(t1.clone()));
    t1_work.set_owner_id(5);
    pool.submit(Some(Box::new(t1_work)));
    pool.cancel_all_for_owner(5);
    let t2 = Arc::new(TestTask::new());
    let mut t2_work = WorkTask::new(TestCallbacks(t2.clone()));
    t2_work.set_owner_id(5);
    pool.submit(Some(Box::new(t2_work)));
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&mut pool, || {
        t1.cancelled.load(Ordering::Acquire) && t2.completed.load(Ordering::Acquire)
    });

    assert!(t1.cancelled.load(Ordering::Acquire));
    assert!(t2.executed.load(Ordering::Acquire));
    assert!(t2.completed.load(Ordering::Acquire));
}

#[test]
fn next_owner_id_generates_unique_ids() {
    let a = WorkPool::next_owner_id();
    let b = WorkPool::next_owner_id();

    assert_ne!(a, b);
    assert_eq!(b, a + 1);
}

#[cfg(feature = "threading")]
#[test]
fn has_pending_work_is_true_while_task_is_in_flight() {
    let mut pool = WorkPool::default();
    let task = Arc::new(BlockingTask::default());
    pool.submit(Some(Box::new(WorkTask::new(BlockingCallbacks(
        task.clone(),
    )))));
    task.wait_until_started();

    assert!(pool.has_pending_work());
    task.unblock();
    poll_until(&mut pool, || task.completed.load(Ordering::Acquire));
    assert!(task.completed.load(Ordering::Acquire));
}

#[cfg(feature = "threading")]
#[test]
fn owner_cancel_of_in_flight_task_sets_status_to_cancelled() {
    let mut pool = WorkPool::default();
    let task = Arc::new(BlockingTask::default());
    let mut native_task = WorkTask::new(BlockingCallbacks(task.clone()));
    native_task.set_owner_id(77);
    let status = native_task.status_handle();
    pool.submit(Some(Box::new(native_task)));
    task.wait_until_started();

    pool.cancel_all_for_owner(77);
    task.unblock();
    poll_until(&mut pool, || status.status() == WorkStatus::Cancelled);

    assert_eq!(status.status(), WorkStatus::Cancelled);
    assert!(!task.completed.load(Ordering::Acquire));
}

#[test]
fn destructor_delivers_on_cancel_for_pre_cancelled_tasks() {
    let t1 = Arc::new(TestTask::new());
    let mut t1_work = WorkTask::new(TestCallbacks(t1.clone()));
    t1_work.set_owner_id(42);
    let t2 = Arc::new(TestTask::new());
    let t2_work = WorkTask::new(TestCallbacks(t2.clone()));
    {
        let mut pool = WorkPool::default();
        #[cfg(feature = "threading")]
        let blockers = block_workers(&mut pool, worker_count());
        pool.submit(Some(Box::new(t1_work)));
        pool.submit(Some(Box::new(t2_work)));
        pool.cancel_all_for_owner(42);
        #[cfg(feature = "threading")]
        unblock_all(&blockers);
    }

    assert!(t1.cancelled.load(Ordering::Acquire));
    assert!(!t1.completed.load(Ordering::Acquire));
    assert!(!t2.cancelled.load(Ordering::Acquire));
    assert!(!t2.completed.load(Ordering::Acquire));
}

#[test]
fn cancelled_owner_does_not_block_future_tasks_with_same_owner() {
    let mut pool = WorkPool::default();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&mut pool, worker_count());
    let t1 = Arc::new(TestTask::new());
    let mut t1_work = WorkTask::new(TestCallbacks(t1.clone()));
    t1_work.set_owner_id(5);
    pool.submit(Some(Box::new(t1_work)));
    pool.cancel_all_for_owner(5);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&mut pool, || t1.cancelled.load(Ordering::Acquire));
    assert!(t1.cancelled.load(Ordering::Acquire));

    let t2 = Arc::new(TestTask::new());
    let mut t2_work = WorkTask::new(TestCallbacks(t2.clone()));
    t2_work.set_owner_id(5);
    pool.submit(Some(Box::new(t2_work)));
    poll_until(&mut pool, || t2.completed.load(Ordering::Acquire));
    assert!(t2.completed.load(Ordering::Acquire));
    assert!(!t2.cancelled.load(Ordering::Acquire));
}

#[test]
fn empty_pool_has_no_pending_work() {
    let mut pool = WorkPool::default();

    assert_eq!(pool.submit(None), 0);
    assert!(!pool.has_pending_work());
    assert_eq!(pool.poll_completed_work(16), 0);
}

#[test]
fn has_pending_work_for_owner_sees_only_that_owners_tasks() {
    let mut pool = WorkPool::default();
    let owner = WorkPool::next_owner_id();
    let other = WorkPool::next_owner_id();
    let task = Arc::new(TestTask::new());
    let mut work = WorkTask::new(TestCallbacks(task.clone()));
    work.set_owner_id(owner);
    pool.submit(Some(Box::new(work)));

    assert!(pool.has_pending_work_for_owner(owner));
    assert!(!pool.has_pending_work_for_owner(other));

    while pool.has_pending_work() {
        pool.poll_completed_work(16);
    }

    assert!(task.completed.load(Ordering::Acquire));
    assert!(!pool.has_pending_work_for_owner(owner));
}

#[test]
fn has_pending_work_for_owner_holds_a_cancelled_task_until_it_is_delivered() {
    let mut pool = WorkPool::default();
    let owner = WorkPool::next_owner_id();
    let task = Arc::new(TestTask::new());
    let mut work = WorkTask::new(TestCallbacks(task.clone()));
    work.set_owner_id(owner);
    pool.submit(Some(Box::new(work)));
    pool.cancel_all_for_owner(owner);

    assert!(pool.has_pending_work_for_owner(owner));

    while pool.has_pending_work() {
        pool.poll_completed_work(16);
    }

    assert!(task.cancelled.load(Ordering::Acquire));
    assert!(!pool.has_pending_work_for_owner(owner));
}

#[test]
fn upstream_offline_join_delivers_only_selected_task() {
    let pool = WorkPool::default();
    let first = Arc::new(TestTask::new());
    let second = Arc::new(TestTask::new());
    pool.submit(Some(Box::new(WorkTask::new(TestCallbacks(first.clone())))));
    let selected = WorkTask::new(TestCallbacks(second.clone()));
    let status = selected.status_handle();
    pool.submit(Some(Box::new(selected)));
    assert!(pool.join_task(&status, std::time::Duration::from_secs(10)));
    assert!(second.completed.load(Ordering::Acquire));
    assert!(status.is_delivered());
    assert!(!first.completed.load(Ordering::Acquire));
    #[cfg(not(feature = "threading"))]
    assert!(!first.executed.load(Ordering::Acquire));
    assert!(pool.join_task(&status, std::time::Duration::ZERO));
    pool.poll_completed_work(16);
}

#[cfg(feature = "threading")]
#[test]
fn upstream_offline_join_running_task_obeys_timeout() {
    struct Blocking {
        started: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
    }
    impl WorkCallbacks for Blocking {
        fn execute(&mut self, _: &mut String) -> bool {
            self.started.send(()).unwrap();
            self.release.recv().unwrap();
            true
        }
    }
    let pool = WorkPool::default();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let task = WorkTask::new(Blocking {
        started: started_tx,
        release: release_rx,
    });
    let status = task.status_handle();
    pool.submit(Some(Box::new(task)));
    started_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let premature = pool.join_task(&status, std::time::Duration::ZERO);
    release_tx.send(()).unwrap();
    assert!(!premature);
    assert!(pool.join_task(&status, std::time::Duration::from_secs(10)));
    assert!(status.is_delivered());
}

#[test]
fn joined_task_waits_for_concurrent_completion_callback() {
    struct Callback {
        entered: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
        visible: Arc<AtomicBool>,
    }
    impl WorkCallbacks for Callback {
        fn execute(&mut self, _: &mut String) -> bool {
            true
        }
        fn on_complete(&mut self) {
            self.entered.send(()).unwrap();
            self.release.recv().unwrap();
            self.visible.store(true, Ordering::Release);
        }
    }
    let pool = Arc::new(WorkPool::default());
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let visible = Arc::new(AtomicBool::new(false));
    let task = WorkTask::new(Callback {
        entered: entered_tx,
        release: release_rx,
        visible: visible.clone(),
    });
    let status = task.status_handle();
    pool.submit(Some(Box::new(task)));
    let poll_pool = pool.clone();
    let poller = std::thread::spawn(move || {
        while poll_pool.poll_completed_work(1) == 0 {
            std::thread::yield_now();
        }
    });
    entered_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let premature = pool.join_task(&status, std::time::Duration::ZERO);
    let delivered_while_blocked = status.is_delivered();
    release_tx.send(()).unwrap();
    let finished = pool.join_task(&status, std::time::Duration::from_secs(10));
    poller.join().unwrap();
    assert!(!premature);
    assert!(!delivered_while_blocked);
    assert!(finished);
    assert!(visible.load(Ordering::Acquire));
}

#[test]
fn completion_callback_can_join_its_own_task_without_publishing_early() {
    struct Callback {
        pool: Weak<WorkPool>,
        status: Arc<Mutex<Option<nuxie_runtime::WorkStatusHandle>>>,
        checked: Arc<AtomicBool>,
    }
    impl WorkCallbacks for Callback {
        fn execute(&mut self, _: &mut String) -> bool {
            true
        }
        fn on_complete(&mut self) {
            let status = self.status.lock().unwrap().as_ref().unwrap().clone();
            assert!(!status.is_delivered());
            assert!(
                self.pool
                    .upgrade()
                    .unwrap()
                    .join_task(&status, std::time::Duration::ZERO)
            );
            assert!(!status.is_delivered());
            self.checked.store(true, Ordering::Release);
        }
    }
    let pool = Arc::new(WorkPool::default());
    let status_slot = Arc::new(Mutex::new(None));
    let checked = Arc::new(AtomicBool::new(false));
    let task = WorkTask::new(Callback {
        pool: Arc::downgrade(&pool),
        status: status_slot.clone(),
        checked: checked.clone(),
    });
    let status = task.status_handle();
    *status_slot.lock().unwrap() = Some(status.clone());
    pool.submit(Some(Box::new(task)));
    assert!(pool.join_task(&status, std::time::Duration::from_secs(10)));
    assert!(status.is_delivered());
    assert!(checked.load(Ordering::Acquire));
}
