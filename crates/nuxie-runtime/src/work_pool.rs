//! Main-thread completion queue for runtime work.
//!
//! This ports Rive's `WorkTask` state and `WorkPool` lifecycle from
//! `include/rive/async/{work_task,work_pool}.hpp` and
//! `src/async/work_pool.cpp` at `d788e8ec`.

use std::collections::VecDeque;
#[cfg(feature = "threading")]
use std::sync::Condvar;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
#[cfg(feature = "threading")]
use std::thread::JoinHandle;

/// Lifecycle state of a [`WorkTask`](crate::WorkTask).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum WorkStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// State shared by a task implementation and its pool.
#[derive(Debug, Default)]
pub struct WorkTaskState {
    status: AtomicU8,
    cancelled: AtomicBool,
    owner_id: AtomicU64,
    submit_generation: AtomicU64,
    error_message: Mutex<String>,
}

impl WorkTaskState {
    pub fn status(&self) -> WorkStatus {
        match self.status.load(Ordering::Acquire) {
            1 => WorkStatus::Running,
            2 => WorkStatus::Completed,
            3 => WorkStatus::Failed,
            4 => WorkStatus::Cancelled,
            _ => WorkStatus::Pending,
        }
    }

    pub fn set_status(&self, status: WorkStatus) {
        self.status.store(status as u8, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn owner_id(&self) -> u64 {
        self.owner_id.load(Ordering::Acquire)
    }

    pub fn set_owner_id(&self, owner_id: u64) {
        self.owner_id.store(owner_id, Ordering::Release);
    }

    pub fn submit_generation(&self) -> u64 {
        self.submit_generation.load(Ordering::Acquire)
    }

    pub fn set_submit_generation(&self, generation: u64) {
        self.submit_generation.store(generation, Ordering::Release);
    }

    /// Returns a synchronized snapshot of the task's error text.
    ///
    /// C++ exposes a retained string reference. Rust snapshots under the
    /// mutex so worker publication and polling-thread callback delivery cannot
    /// race; the pool borrows that snapshot only for the callback invocation.
    pub fn error_message(&self) -> String {
        lock_unpoisoned(&self.error_message).clone()
    }

    pub fn set_error_message(&self, message: impl Into<String>) {
        *lock_unpoisoned(&self.error_message) = message.into();
    }
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A unit of background work with callbacks delivered by
/// [`WorkPool::poll_completed_work`].
///
/// `execute` runs on a worker with the `threading` feature and on the polling
/// thread without it. The three callbacks always run on the polling thread.
pub trait WorkTask: Send + Sync + 'static {
    fn state(&self) -> &WorkTaskState;
    fn execute(&self) -> bool;
    fn on_complete(&self) {}
    fn on_error(&self, _error: &str) {}
    fn on_cancel(&self) {}
}

pub type WorkTaskRef<T = dyn WorkTask> = Arc<T>;

static NEXT_OWNER_ID: AtomicU64 = AtomicU64::new(1);

/// Returns a process-unique owner generation ID.
pub fn next_work_owner_id() -> u64 {
    NEXT_OWNER_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(not(feature = "threading"))]
pub struct WorkPool {
    work_queue: Mutex<VecDeque<WorkTaskRef>>,
    next_handle: AtomicU64,
}

#[cfg(not(feature = "threading"))]
impl WorkPool {
    pub fn new() -> Self {
        Self {
            work_queue: Mutex::new(VecDeque::new()),
            next_handle: AtomicU64::new(1),
        }
    }

    pub fn next_owner_id() -> u64 {
        next_work_owner_id()
    }

    pub fn submit(&self, task: Option<WorkTaskRef>) -> u64 {
        let Some(task) = task else {
            return 0;
        };
        task.state().set_status(WorkStatus::Pending);
        lock_unpoisoned(&self.work_queue).push_back(task);
        self.next_handle.fetch_add(1, Ordering::Relaxed)
    }

    pub fn poll_completed_work(&self, max_callbacks: u32) -> u32 {
        let mut processed = 0;
        while processed < max_callbacks {
            let Some(task) = lock_unpoisoned(&self.work_queue).pop_front() else {
                break;
            };
            let state = task.state();
            if state.is_cancelled() {
                state.set_status(WorkStatus::Cancelled);
                task.on_cancel();
                processed = processed.saturating_add(1);
                continue;
            }

            state.set_status(WorkStatus::Running);
            let success = task.execute();
            if state.is_cancelled() {
                state.set_status(WorkStatus::Cancelled);
                task.on_cancel();
            } else if success {
                state.set_status(WorkStatus::Completed);
                task.on_complete();
            } else {
                state.set_status(WorkStatus::Failed);
                task.on_error(&state.error_message());
            }
            processed = processed.saturating_add(1);
        }
        processed
    }

    pub fn has_pending_work(&self) -> bool {
        !lock_unpoisoned(&self.work_queue).is_empty()
    }

    pub fn cancel_all_for_owner(&self, owner_id: u64) {
        for task in lock_unpoisoned(&self.work_queue).iter() {
            if task.state().owner_id() == owner_id {
                task.state().cancel();
            }
        }
    }
}

#[cfg(not(feature = "threading"))]
impl Default for WorkPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "threading"))]
impl Drop for WorkPool {
    fn drop(&mut self) {
        let work_queue = self
            .work_queue
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for task in work_queue.drain(..) {
            if task.state().is_cancelled() {
                task.state().set_status(WorkStatus::Cancelled);
                task.on_cancel();
            }
        }
    }
}

#[cfg(feature = "threading")]
#[derive(Default)]
struct WorkQueue {
    tasks: VecDeque<WorkTaskRef>,
    cancelled_owners: std::collections::HashMap<u64, u64>,
    cancel_generation: u64,
    next_handle: u64,
    shutdown: bool,
}

#[cfg(feature = "threading")]
struct ThreadedPool {
    work: Mutex<WorkQueue>,
    completed: Mutex<VecDeque<WorkTaskRef>>,
    have_work: Condvar,
    in_flight_count: std::sync::atomic::AtomicUsize,
}

#[cfg(feature = "threading")]
pub struct WorkPool {
    pool: Arc<ThreadedPool>,
    threads: Vec<JoinHandle<()>>,
}

#[cfg(feature = "threading")]
impl WorkPool {
    pub fn new() -> Self {
        let pool = Arc::new(ThreadedPool {
            work: Mutex::new(WorkQueue {
                next_handle: 1,
                ..WorkQueue::default()
            }),
            completed: Mutex::new(VecDeque::new()),
            have_work: Condvar::new(),
            in_flight_count: std::sync::atomic::AtomicUsize::new(0),
        });
        let worker_count = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1)
            .min(4);
        let mut threads = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let worker_pool = Arc::clone(&pool);
            threads.push(std::thread::spawn(move || worker_loop(worker_pool)));
        }
        Self { pool, threads }
    }

    pub fn next_owner_id() -> u64 {
        next_work_owner_id()
    }

    pub fn submit(&self, task: Option<WorkTaskRef>) -> u64 {
        let Some(task) = task else {
            return 0;
        };
        let handle = {
            let mut work = lock_unpoisoned(&self.pool.work);
            task.state().set_status(WorkStatus::Pending);
            task.state().set_submit_generation(work.cancel_generation);
            let handle = work.next_handle;
            work.next_handle = handle.wrapping_add(1);
            work.tasks.push_back(task);
            handle
        };
        self.pool.have_work.notify_one();
        handle
    }

    pub fn poll_completed_work(&self, max_callbacks: u32) -> u32 {
        let mut processed = 0;
        while processed < max_callbacks {
            let task = lock_unpoisoned(&self.pool.completed).pop_front();
            let Some(task) = task else {
                break;
            };
            let owner_cancelled = {
                let work = lock_unpoisoned(&self.pool.work);
                work.cancelled_owners
                    .get(&task.state().owner_id())
                    .is_some_and(|generation| task.state().submit_generation() < *generation)
            };

            if !task.state().is_cancelled() && !owner_cancelled {
                match task.state().status() {
                    WorkStatus::Completed => task.on_complete(),
                    WorkStatus::Failed => task.on_error(&task.state().error_message()),
                    _ => {}
                }
            } else {
                task.state().set_status(WorkStatus::Cancelled);
                task.on_cancel();
            }
            processed = processed.saturating_add(1);
        }
        processed
    }

    pub fn has_pending_work(&self) -> bool {
        self.pool.in_flight_count.load(Ordering::Relaxed) > 0
            || !lock_unpoisoned(&self.pool.work).tasks.is_empty()
            || !lock_unpoisoned(&self.pool.completed).is_empty()
    }

    pub fn cancel_all_for_owner(&self, owner_id: u64) {
        {
            let mut work = lock_unpoisoned(&self.pool.work);
            work.cancel_generation = work.cancel_generation.wrapping_add(1);
            let generation = work.cancel_generation;
            work.cancelled_owners.insert(owner_id, generation);
            for task in &work.tasks {
                if task.state().owner_id() == owner_id {
                    task.state().cancel();
                }
            }
        }
        for task in lock_unpoisoned(&self.pool.completed).iter() {
            if task.state().owner_id() == owner_id {
                task.state().cancel();
            }
        }
    }
}

#[cfg(feature = "threading")]
impl Default for WorkPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "threading")]
impl Drop for WorkPool {
    fn drop(&mut self) {
        {
            lock_unpoisoned(&self.pool.work).shutdown = true;
        }
        self.pool.have_work.notify_all();
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }

        let mut abandoned = Vec::new();
        abandoned.extend(lock_unpoisoned(&self.pool.completed).drain(..));
        abandoned.extend(lock_unpoisoned(&self.pool.work).tasks.drain(..));
        for task in abandoned {
            if task.state().is_cancelled() {
                task.state().set_status(WorkStatus::Cancelled);
                task.on_cancel();
            }
        }
    }
}

#[cfg(feature = "threading")]
fn worker_loop(pool: Arc<ThreadedPool>) {
    loop {
        let task = {
            let mut work = lock_unpoisoned(&pool.work);
            while !work.shutdown && work.tasks.is_empty() {
                work = pool
                    .have_work
                    .wait(work)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
            if work.shutdown && work.tasks.is_empty() {
                return;
            }
            work.tasks.pop_front()
        };
        let Some(task) = task else {
            continue;
        };
        pool.in_flight_count.fetch_add(1, Ordering::Relaxed);

        if task.state().is_cancelled() {
            task.state().set_status(WorkStatus::Cancelled);
            lock_unpoisoned(&pool.completed).push_back(task);
            pool.in_flight_count.fetch_sub(1, Ordering::Relaxed);
            continue;
        }

        task.state().set_status(WorkStatus::Running);
        let success = task.execute();
        if task.state().is_cancelled() {
            task.state().set_status(WorkStatus::Cancelled);
        } else if success {
            task.state().set_status(WorkStatus::Completed);
        } else {
            task.state().set_status(WorkStatus::Failed);
        }
        lock_unpoisoned(&pool.completed).push_back(task);
        pool.in_flight_count.fetch_sub(1, Ordering::Relaxed);
    }
}

static GLOBAL_WORK_POOL: OnceLock<WorkPool> = OnceLock::new();

/// Calls `callback` with the lazily-created process-global pool.
pub fn with_global_work_pool<R>(callback: impl FnOnce(&WorkPool) -> R) -> R {
    callback(GLOBAL_WORK_POOL.get_or_init(WorkPool::new))
}

pub fn with_global_work_pool_if_exists<R>(callback: impl FnOnce(&WorkPool) -> R) -> Option<R> {
    GLOBAL_WORK_POOL.get().map(callback)
}

pub fn global_work_pool_exists() -> bool {
    GLOBAL_WORK_POOL.get().is_some()
}

/// Drains up to 16 callbacks from the existing global pool without creating
/// one. This is called at the root artboard advance seam.
pub fn poll_async_work() {
    let _ = with_global_work_pool_if_exists(|pool| {
        if pool.has_pending_work() {
            pool.poll_completed_work(16);
        }
    });
}
