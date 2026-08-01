//! Direct ports of the 13 cases in
//! `tests/unit_tests/runtime/work_pool_test.cpp` at `d788e8ec`.
//!
//! The C++ probe ABI does not expose WorkPool. Concurrency-sensitive cases
//! therefore use barriers against the public Rust interface instead of
//! timing, sleeps, or fixture-backed differentials.

#[cfg(feature = "threading")]
use nuxie_runtime::WorkStatus;
use nuxie_runtime::{WorkPool, WorkTask, WorkTaskRef, WorkTaskState};
#[cfg(feature = "threading")]
use std::sync::Condvar;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

fn task_ref<T: WorkTask>(task: T) -> WorkTaskRef<T> {
    std::sync::Arc::new(task)
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Default)]
struct TestTask {
    state: WorkTaskState,
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

impl WorkTask for TestTask {
    fn state(&self) -> &WorkTaskState {
        &self.state
    }

    fn execute(&self) -> bool {
        self.executed.store(true, Ordering::Release);
        if !self.should_succeed.load(Ordering::Acquire) {
            self.state.set_error_message("test failure");
            return false;
        }
        true
    }

    fn on_complete(&self) {
        self.completed.store(true, Ordering::Release);
    }

    fn on_error(&self, error: &str) {
        self.errored.store(true, Ordering::Release);
        *lock_unpoisoned(&self.error_message) = error.to_owned();
    }

    fn on_cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

fn poll_until(pool: &WorkPool, mut done: impl FnMut() -> bool) {
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
    state: WorkTaskState,
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
impl WorkTask for BlockingTask {
    fn state(&self) -> &WorkTaskState {
        &self.state
    }

    fn execute(&self) -> bool {
        let mut gate = lock_unpoisoned(&self.gate);
        gate.started = true;
        self.changed.notify_all();
        while !gate.unblock {
            gate = self
                .changed
                .wait(gate)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        true
    }

    fn on_complete(&self) {
        self.completed.store(true, Ordering::Release);
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
fn block_workers(pool: &WorkPool, count: usize) -> Vec<WorkTaskRef<BlockingTask>> {
    let blockers = (0..count)
        .map(|_| task_ref(BlockingTask::default()))
        .collect::<Vec<_>>();
    for blocker in &blockers {
        pool.submit(Some(blocker.clone()));
    }
    for blocker in &blockers {
        blocker.wait_until_started();
    }
    blockers
}

#[cfg(feature = "threading")]
fn unblock_all(blockers: &[WorkTaskRef<BlockingTask>]) {
    for blocker in blockers {
        blocker.unblock();
    }
}

#[test]
fn work_pool_executes_task_on_poll() {
    let pool = WorkPool::new();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&pool, worker_count());
    let task = task_ref(TestTask::new());
    pool.submit(Some(task.clone()));

    assert!(!task.executed.load(Ordering::Acquire));
    assert!(!task.completed.load(Ordering::Acquire));
    assert!(pool.has_pending_work());
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&pool, || {
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
    let pool = WorkPool::new();
    let task = task_ref(TestTask::new());
    task.should_succeed.store(false, Ordering::Release);
    pool.submit(Some(task.clone()));
    poll_until(&pool, || task.errored.load(Ordering::Acquire));

    assert!(task.executed.load(Ordering::Acquire));
    assert!(!task.completed.load(Ordering::Acquire));
    assert!(task.errored.load(Ordering::Acquire));
    assert_eq!(&*lock_unpoisoned(&task.error_message), "test failure");
}

#[test]
fn work_pool_respects_max_callbacks() {
    let pool = WorkPool::new();
    #[cfg(feature = "threading")]
    let reserved_workers = block_workers(&pool, worker_count().saturating_sub(1));
    let tasks = (0..3)
        .map(|_| task_ref(TestTask::new()))
        .collect::<Vec<_>>();
    for task in &tasks {
        pool.submit(Some(task.clone()));
    }

    // Keep all but one worker occupied, then put a blocking fence behind the
    // tasks. When that sole worker reaches the fence, all three completions
    // are queued in submission order and the bounded-poll assertions cannot
    // race worker scheduling.
    #[cfg(feature = "threading")]
    let completion_fence = {
        let fence = task_ref(BlockingTask::default());
        pool.submit(Some(fence.clone()));
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
        poll_until(&pool, || {
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
    let pool = WorkPool::new();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&pool, worker_count());
    let t1 = task_ref(TestTask::new());
    let t2 = task_ref(TestTask::new());
    t1.state.set_owner_id(42);
    t2.state.set_owner_id(99);
    pool.submit(Some(t1.clone()));
    pool.submit(Some(t2.clone()));

    pool.cancel_all_for_owner(42);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&pool, || {
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
        state: WorkTaskState,
        cancel_count: AtomicUsize,
    }

    impl WorkTask for CountingTask {
        fn state(&self) -> &WorkTaskState {
            &self.state
        }
        fn execute(&self) -> bool {
            true
        }
        fn on_cancel(&self) {
            self.cancel_count.fetch_add(1, Ordering::AcqRel);
        }
    }

    let pool = WorkPool::new();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&pool, worker_count());
    let task = task_ref(CountingTask {
        state: WorkTaskState::default(),
        cancel_count: AtomicUsize::new(0),
    });
    task.state.set_owner_id(1);
    pool.submit(Some(task.clone()));
    pool.cancel_all_for_owner(1);
    assert_eq!(task.cancel_count.load(Ordering::Acquire), 0);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&pool, || task.cancel_count.load(Ordering::Acquire) == 1);

    assert_eq!(task.cancel_count.load(Ordering::Acquire), 1);
}

#[test]
fn cancelled_owner_does_not_interfere_with_other_owners() {
    let pool = WorkPool::new();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&pool, worker_count());
    let owner_a1 = task_ref(TestTask::new());
    let owner_a2 = task_ref(TestTask::new());
    let owner_b1 = task_ref(TestTask::new());
    owner_a1.state.set_owner_id(10);
    owner_a2.state.set_owner_id(10);
    owner_b1.state.set_owner_id(20);
    pool.submit(Some(owner_a1.clone()));
    pool.submit(Some(owner_b1.clone()));
    pool.submit(Some(owner_a2.clone()));

    pool.cancel_all_for_owner(10);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&pool, || {
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
    let pool = WorkPool::new();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&pool, worker_count());
    let t1 = task_ref(TestTask::new());
    t1.state.set_owner_id(5);
    pool.submit(Some(t1.clone()));
    pool.cancel_all_for_owner(5);
    let t2 = task_ref(TestTask::new());
    t2.state.set_owner_id(5);
    pool.submit(Some(t2.clone()));
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&pool, || {
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
    let pool = WorkPool::new();
    let task = task_ref(BlockingTask::default());
    pool.submit(Some(task.clone()));
    task.wait_until_started();

    assert!(pool.has_pending_work());
    task.unblock();
    poll_until(&pool, || task.completed.load(Ordering::Acquire));
    assert!(task.completed.load(Ordering::Acquire));
}

#[cfg(feature = "threading")]
#[test]
fn owner_cancel_of_in_flight_task_sets_status_to_cancelled() {
    let pool = WorkPool::new();
    let task = task_ref(BlockingTask::default());
    task.state.set_owner_id(77);
    pool.submit(Some(task.clone()));
    task.wait_until_started();

    pool.cancel_all_for_owner(77);
    task.unblock();
    poll_until(&pool, || task.state.status() == WorkStatus::Cancelled);

    assert_eq!(task.state.status(), WorkStatus::Cancelled);
    assert!(!task.completed.load(Ordering::Acquire));
}

#[test]
fn destructor_delivers_on_cancel_for_pre_cancelled_tasks() {
    let t1 = task_ref(TestTask::new());
    t1.state.set_owner_id(42);
    let t2 = task_ref(TestTask::new());
    {
        let pool = WorkPool::new();
        #[cfg(feature = "threading")]
        let blockers = block_workers(&pool, worker_count());
        pool.submit(Some(t1.clone()));
        pool.submit(Some(t2.clone()));
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
    let pool = WorkPool::new();
    #[cfg(feature = "threading")]
    let blockers = block_workers(&pool, worker_count());
    let t1 = task_ref(TestTask::new());
    t1.state.set_owner_id(5);
    pool.submit(Some(t1.clone()));
    pool.cancel_all_for_owner(5);
    #[cfg(feature = "threading")]
    unblock_all(&blockers);
    poll_until(&pool, || t1.cancelled.load(Ordering::Acquire));
    assert!(t1.cancelled.load(Ordering::Acquire));

    let t2 = task_ref(TestTask::new());
    t2.state.set_owner_id(5);
    pool.submit(Some(t2.clone()));
    poll_until(&pool, || t2.completed.load(Ordering::Acquire));
    assert!(t2.completed.load(Ordering::Acquire));
    assert!(!t2.cancelled.load(Ordering::Acquire));
}

#[test]
fn empty_pool_has_no_pending_work() {
    let pool = WorkPool::new();

    assert_eq!(pool.submit(None), 0);
    assert!(!pool.has_pending_work());
    assert_eq!(pool.poll_completed_work(16), 0);
}
