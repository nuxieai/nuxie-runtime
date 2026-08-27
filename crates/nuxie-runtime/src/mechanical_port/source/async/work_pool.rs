use crate::mechanical_port::source::r#async::work_task::{DynWorkTask, WorkStatus};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(feature = "with_rive_threading")]
use std::{
    collections::HashMap,
    sync::{Condvar, atomic::AtomicI32},
    thread::{self, JoinHandle},
};

static NEXT_OWNER_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(not(feature = "with_rive_threading"))]
pub struct WorkPool {
    work_queue: VecDeque<Box<dyn DynWorkTask>>,
    next_handle: u64,
}

#[cfg(not(feature = "with_rive_threading"))]
impl Default for WorkPool {
    fn default() -> Self {
        Self {
            work_queue: VecDeque::new(),
            next_handle: 1,
        }
    }
}

#[cfg(not(feature = "with_rive_threading"))]
impl WorkPool {
    pub fn next_owner_id() -> u64 {
        NEXT_OWNER_ID.fetch_add(1, Ordering::Relaxed)
    }

    pub fn submit(&mut self, task: Option<Box<dyn DynWorkTask>>) -> u64 {
        let Some(mut task) = task else { return 0 };
        task.set_status(WorkStatus::Pending);
        self.work_queue.push_back(task);
        let handle = self.next_handle;
        self.next_handle += 1;
        handle
    }

    pub fn poll_completed_work(&mut self, max_callbacks: u32) -> u32 {
        let mut processed = 0;
        while processed < max_callbacks && !self.work_queue.is_empty() {
            let mut task = self.work_queue.pop_front().unwrap();
            if task.is_cancelled() {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel();
                processed += 1;
                continue;
            }
            task.set_status(WorkStatus::Running);
            let success = task.execute();
            if task.is_cancelled() {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel();
            } else if success {
                task.set_status(WorkStatus::Completed);
                task.on_complete();
            } else {
                task.set_status(WorkStatus::Failed);
                task.on_error();
            }
            processed += 1;
        }
        processed
    }

    pub fn has_pending_work(&self) -> bool {
        !self.work_queue.is_empty()
    }

    pub fn cancel_all_for_owner(&mut self, owner_id: u64) {
        // Polling delivers the single cancellation callback.
        for task in &self.work_queue {
            if task.owner_id() == owner_id {
                task.cancel();
            }
        }
    }
}

#[cfg(not(feature = "with_rive_threading"))]
impl Drop for WorkPool {
    fn drop(&mut self) {
        // Only tasks already marked cancelled receive destruction callbacks.
        for mut task in self.work_queue.drain(..) {
            if task.is_cancelled() {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel();
            }
        }
    }
}

#[cfg(feature = "with_rive_threading")]
struct QueueState {
    work_queue: VecDeque<Box<dyn DynWorkTask>>,
    cancelled_owners: HashMap<u64, u64>,
    cancel_generation: u64,
    shutdown: bool,
}

#[cfg(feature = "with_rive_threading")]
struct ThreadedState {
    queue: Mutex<QueueState>,
    completed_queue: Mutex<VecDeque<Box<dyn DynWorkTask>>>,
    have_work: Condvar,
    in_flight_count: AtomicI32,
}

#[cfg(feature = "with_rive_threading")]
pub struct WorkPool {
    state: Arc<ThreadedState>,
    threads: Vec<JoinHandle<()>>,
    next_handle: u64,
}

#[cfg(feature = "with_rive_threading")]
impl Default for WorkPool {
    fn default() -> Self {
        let state = Arc::new(ThreadedState {
            queue: Mutex::new(QueueState {
                work_queue: VecDeque::new(),
                cancelled_owners: HashMap::new(),
                cancel_generation: 0,
                shutdown: false,
            }),
            completed_queue: Mutex::new(VecDeque::new()),
            have_work: Condvar::new(),
            in_flight_count: AtomicI32::new(0),
        });
        let thread_count = thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .max(1)
            .min(4);
        let mut threads = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            let worker_state = Arc::clone(&state);
            threads.push(thread::spawn(move || worker_loop(worker_state)));
        }
        Self {
            state,
            threads,
            next_handle: 1,
        }
    }
}

#[cfg(feature = "with_rive_threading")]
fn worker_loop(state: Arc<ThreadedState>) {
    loop {
        let mut task = {
            let mut queue = state.queue.lock().unwrap();
            queue = state
                .have_work
                .wait_while(queue, |queue| {
                    !queue.shutdown && queue.work_queue.is_empty()
                })
                .unwrap();
            if queue.shutdown && queue.work_queue.is_empty() {
                return;
            }
            queue.work_queue.pop_front().unwrap()
        };

        state.in_flight_count.fetch_add(1, Ordering::Relaxed);
        if task.is_cancelled() {
            task.set_status(WorkStatus::Cancelled);
            {
                let mut completed = state.completed_queue.lock().unwrap();
                completed.push_back(task);
                state.in_flight_count.fetch_sub(1, Ordering::Relaxed);
            }
            continue;
        }
        task.set_status(WorkStatus::Running);
        let success = task.execute();
        if task.is_cancelled() {
            task.set_status(WorkStatus::Cancelled);
        } else if success {
            task.set_status(WorkStatus::Completed);
        } else {
            task.set_status(WorkStatus::Failed);
        }
        state.completed_queue.lock().unwrap().push_back(task);
        state.in_flight_count.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "with_rive_threading")]
impl WorkPool {
    pub fn next_owner_id() -> u64 {
        NEXT_OWNER_ID.fetch_add(1, Ordering::Relaxed)
    }

    pub fn submit(&mut self, task: Option<Box<dyn DynWorkTask>>) -> u64 {
        let Some(mut task) = task else { return 0 };
        let handle;
        {
            let mut queue = self.state.queue.lock().unwrap();
            task.set_status(WorkStatus::Pending);
            task.set_submit_generation(queue.cancel_generation);
            handle = self.next_handle;
            self.next_handle += 1;
            queue.work_queue.push_back(task);
        }
        self.state.have_work.notify_one();
        handle
    }

    pub fn poll_completed_work(&mut self, max_callbacks: u32) -> u32 {
        let mut processed = 0;
        while processed < max_callbacks {
            let Some(mut task) = self.state.completed_queue.lock().unwrap().pop_front() else {
                break;
            };
            let owner_cancelled = {
                let queue = self.state.queue.lock().unwrap();
                queue
                    .cancelled_owners
                    .get(&task.owner_id())
                    .is_some_and(|generation| task.submit_generation() < *generation)
            };
            if !task.is_cancelled() && !owner_cancelled {
                if task.status() == WorkStatus::Completed {
                    task.on_complete();
                } else if task.status() == WorkStatus::Failed {
                    task.on_error();
                }
            } else {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel();
            }
            processed += 1;
        }
        processed
    }

    pub fn has_pending_work(&self) -> bool {
        if self.state.in_flight_count.load(Ordering::Relaxed) > 0 {
            return true;
        }
        if !self.state.queue.lock().unwrap().work_queue.is_empty() {
            return true;
        }
        if !self.state.completed_queue.lock().unwrap().is_empty() {
            return true;
        }
        false
    }

    pub fn cancel_all_for_owner(&mut self, owner_id: u64) {
        {
            let mut queue = self.state.queue.lock().unwrap();
            queue.cancel_generation += 1;
            let generation = queue.cancel_generation;
            queue.cancelled_owners.insert(owner_id, generation);
            for task in &queue.work_queue {
                if task.owner_id() == owner_id {
                    task.cancel();
                }
            }
        }
        {
            let completed = self.state.completed_queue.lock().unwrap();
            for task in completed.iter() {
                if task.owner_id() == owner_id {
                    task.cancel();
                }
            }
        }
    }
}

#[cfg(feature = "with_rive_threading")]
impl Drop for WorkPool {
    fn drop(&mut self) {
        {
            let mut queue = self.state.queue.lock().unwrap();
            queue.shutdown = true;
        }
        self.state.have_work.notify_all();
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
        for mut task in self.state.completed_queue.lock().unwrap().drain(..) {
            if task.is_cancelled() {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel();
            }
        }
        for mut task in self.state.queue.lock().unwrap().work_queue.drain(..) {
            if task.is_cancelled() {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel();
            }
        }
    }
}

static GLOBAL_WORK_POOL: OnceLock<Arc<Mutex<WorkPool>>> = OnceLock::new();

pub fn get_global_work_pool() -> &'static Arc<Mutex<WorkPool>> {
    GLOBAL_WORK_POOL.get_or_init(|| Arc::new(Mutex::new(WorkPool::default())))
}

pub fn get_global_work_pool_if_exists() -> Option<&'static Arc<Mutex<WorkPool>>> {
    GLOBAL_WORK_POOL.get()
}

pub fn rive_poll_async_work() {
    if let Some(pool) = get_global_work_pool_if_exists() {
        let mut pool = pool.lock().unwrap();
        if pool.has_pending_work() {
            pool.poll_completed_work(16);
        }
    }
}
