use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub trait WorkCallbacks: Send {
    fn execute(&mut self) -> bool;
    fn on_complete(&mut self) {}
    fn on_error(&mut self, _error: &str) {}
    fn on_cancel(&mut self) {}
}

pub trait DynWorkTask: Send {
    fn execute(&mut self) -> bool;
    fn on_complete(&mut self);
    fn on_error(&mut self);
    fn on_cancel(&mut self);
    fn status(&self) -> WorkStatus;
    fn set_status(&mut self, value: WorkStatus);
    fn is_cancelled(&self) -> bool;
    fn cancel(&self);
    fn owner_id(&self) -> u64;
    fn submit_generation(&self) -> u64;
    fn set_submit_generation(&mut self, value: u64);
}

pub struct WorkTask<T: WorkCallbacks> {
    pub callbacks: T,
    pub error_message: String,
    status: WorkStatus,
    cancelled: AtomicBool,
    owner_id: u64,
    submit_generation: u64,
}

impl<T: WorkCallbacks> DynWorkTask for WorkTask<T> {
    fn execute(&mut self) -> bool {
        self.callbacks.execute()
    }
    fn on_complete(&mut self) {
        self.callbacks.on_complete()
    }
    fn on_error(&mut self) {
        let error = self.error_message.clone();
        self.callbacks.on_error(&error)
    }
    fn on_cancel(&mut self) {
        self.callbacks.on_cancel()
    }
    fn status(&self) -> WorkStatus {
        self.status
    }
    fn set_status(&mut self, value: WorkStatus) {
        self.status = value;
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    fn owner_id(&self) -> u64 {
        self.owner_id
    }
    fn submit_generation(&self) -> u64 {
        self.submit_generation
    }
    fn set_submit_generation(&mut self, value: u64) {
        self.submit_generation = value;
    }
}
impl<T: WorkCallbacks> WorkTask<T> {
    pub fn new(callbacks: T) -> Self {
        Self {
            callbacks,
            error_message: String::new(),
            status: WorkStatus::Pending,
            cancelled: AtomicBool::new(false),
            owner_id: 0,
            submit_generation: 0,
        }
    }
    pub fn status(&self) -> WorkStatus {
        self.status
    }
    pub fn set_status(&mut self, value: WorkStatus) {
        self.status = value;
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    pub fn error_message(&self) -> &str {
        &self.error_message
    }
    pub fn owner_id(&self) -> u64 {
        self.owner_id
    }
    pub fn set_owner_id(&mut self, value: u64) {
        self.owner_id = value;
    }
    pub fn submit_generation(&self) -> u64 {
        self.submit_generation
    }
    pub fn set_submit_generation(&mut self, value: u64) {
        self.submit_generation = value;
    }
}
