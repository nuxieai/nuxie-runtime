use crate::mechanical_port::source::r#async::work_task::{DynWorkTask, WorkStatus};
use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT_OWNER_ID: AtomicU64 = AtomicU64::new(1);
pub struct WorkPool {
    work_queue: VecDeque<Box<dyn DynWorkTask>>,
    completed_queue: VecDeque<Box<dyn DynWorkTask>>,
    next_handle: u64,
    cancel_generation: u64,
    cancelled_owners: HashMap<u64, u64>,
    in_flight_count: u32,
}
impl Default for WorkPool {
    fn default() -> Self {
        Self {
            work_queue: VecDeque::new(),
            completed_queue: VecDeque::new(),
            next_handle: 1,
            cancel_generation: 0,
            cancelled_owners: HashMap::new(),
            in_flight_count: 0,
        }
    }
}
impl WorkPool {
    pub fn next_owner_id() -> u64 {
        NEXT_OWNER_ID.fetch_add(1, Ordering::Relaxed)
    }
    pub fn submit(&mut self, task: Option<Box<dyn DynWorkTask>>) -> u64 {
        let Some(mut task) = task else { return 0 };
        task.set_status(WorkStatus::Pending);
        task.set_submit_generation(self.cancel_generation);
        let h = self.next_handle;
        self.next_handle += 1;
        self.work_queue.push_back(task);
        h
    }
    fn execute_one(&mut self) {
        let Some(mut task) = self.work_queue.pop_front() else {
            return;
        };
        self.in_flight_count += 1;
        if task.is_cancelled() {
            task.set_status(WorkStatus::Cancelled)
        } else {
            task.set_status(WorkStatus::Running);
            let ok = task.execute();
            task.set_status(if task.is_cancelled() {
                WorkStatus::Cancelled
            } else if ok {
                WorkStatus::Completed
            } else {
                WorkStatus::Failed
            });
        }
        self.completed_queue.push_back(task);
        self.in_flight_count -= 1;
    }
    pub fn poll_completed_work(&mut self, max_callbacks: u32) -> u32 {
        let mut done = 0;
        while done < max_callbacks {
            if self.completed_queue.is_empty() {
                self.execute_one()
            }
            let Some(mut task) = self.completed_queue.pop_front() else {
                break;
            };
            let owner_cancelled = self
                .cancelled_owners
                .get(&task.owner_id())
                .is_some_and(|generation| task.submit_generation() < *generation);
            if task.is_cancelled() || owner_cancelled {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel()
            } else {
                match task.status() {
                    WorkStatus::Completed => task.on_complete(),
                    WorkStatus::Failed => task.on_error(),
                    _ => {}
                }
            }
            done += 1;
        }
        done
    }
    pub fn has_pending_work(&self) -> bool {
        self.in_flight_count > 0 || !self.work_queue.is_empty() || !self.completed_queue.is_empty()
    }
    pub fn cancel_all_for_owner(&mut self, owner: u64) {
        self.cancel_generation += 1;
        self.cancelled_owners.insert(owner, self.cancel_generation);
        for task in self.work_queue.iter().chain(self.completed_queue.iter()) {
            if task.owner_id() == owner {
                task.cancel()
            }
        }
    }
}
impl Drop for WorkPool {
    fn drop(&mut self) {
        for mut task in self
            .completed_queue
            .drain(..)
            .chain(self.work_queue.drain(..))
        {
            if task.is_cancelled() {
                task.set_status(WorkStatus::Cancelled);
                task.on_cancel()
            }
        }
    }
}
