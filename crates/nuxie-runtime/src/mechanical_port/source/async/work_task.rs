use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub trait WorkCallbacks: Send {
    fn execute(&mut self, error_message: &mut String) -> bool;
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

/// Retains the task's cancellation field independently of worker-owned data.
/// Cancelling never waits for the executing task or invokes its callbacks.
#[derive(Clone, Default)]
pub struct WorkCancellationHandle(Arc<AtomicBool>);

impl WorkCancellationHandle {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Read-only access to the task's actual status, retained independently of the
/// worker-owned callbacks. The last status remains readable after task release.
#[derive(Clone)]
pub struct WorkStatusHandle(Arc<AtomicU8>);

impl Default for WorkStatusHandle {
    fn default() -> Self {
        Self(Arc::new(AtomicU8::new(WorkStatus::Pending as u8)))
    }
}

impl WorkStatusHandle {
    pub fn status(&self) -> WorkStatus {
        match self.0.load(Ordering::Acquire) {
            0 => WorkStatus::Pending,
            1 => WorkStatus::Running,
            2 => WorkStatus::Completed,
            3 => WorkStatus::Failed,
            4 => WorkStatus::Cancelled,
            // The atomic is private and is only written from a WorkStatus.
            _ => unreachable!("task status always contains a WorkStatus discriminant"),
        }
    }

    fn set(&self, value: WorkStatus) {
        self.0.store(value as u8, Ordering::Release);
    }
}

pub struct WorkTask<T: WorkCallbacks> {
    pub callbacks: T,
    pub error_message: String,
    status: WorkStatusHandle,
    cancelled: WorkCancellationHandle,
    owner_id: u64,
    submit_generation: u64,
}

impl<T: WorkCallbacks> DynWorkTask for WorkTask<T> {
    fn execute(&mut self) -> bool {
        self.callbacks.execute(&mut self.error_message)
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
        self.status.status()
    }
    fn set_status(&mut self, value: WorkStatus) {
        self.status.set(value);
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }
    fn cancel(&self) {
        self.cancelled.cancel();
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
            status: WorkStatusHandle::default(),
            cancelled: WorkCancellationHandle::default(),
            owner_id: 0,
            submit_generation: 0,
        }
    }
    pub fn status(&self) -> WorkStatus {
        self.status.status()
    }
    pub fn set_status(&mut self, value: WorkStatus) {
        self.status.set(value);
    }
    pub fn status_handle(&self) -> WorkStatusHandle {
        self.status.clone()
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }
    pub fn cancel(&self) {
        self.cancelled.cancel();
    }
    pub fn cancellation_handle(&self) -> WorkCancellationHandle {
        self.cancelled.clone()
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
