//! Direct ports of `tests/unit_tests/runtime/work_task_test.cpp` at
//! `d788e8ec`. The pinned probe ABI has no WorkTask surface, so these six
//! API-contract cases run against the public Rust port directly.

use nuxie_runtime::{WorkStatus, WorkTask, WorkTaskState};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
struct TestWorkTask {
    state: WorkTaskState,
    execute_result: AtomicBool,
}

impl TestWorkTask {
    fn succeeding() -> Self {
        Self {
            state: WorkTaskState::default(),
            execute_result: AtomicBool::new(true),
        }
    }
}

impl WorkTask for TestWorkTask {
    fn state(&self) -> &WorkTaskState {
        &self.state
    }

    fn execute(&self) -> bool {
        let result = self.execute_result.load(Ordering::Acquire);
        if !result {
            self.state.set_error_message("test failure");
        }
        result
    }
}

#[test]
fn work_task_default_state() {
    let task = TestWorkTask::succeeding();

    assert_eq!(task.state.status(), WorkStatus::Pending);
    assert!(!task.state.is_cancelled());
    assert_eq!(task.state.owner_id(), 0);
    assert_eq!(task.state.submit_generation(), 0);
    assert!(task.state.error_message().is_empty());
}

#[test]
fn work_task_status_transitions() {
    let task = TestWorkTask::succeeding();

    task.state.set_status(WorkStatus::Running);
    assert_eq!(task.state.status(), WorkStatus::Running);
    task.state.set_status(WorkStatus::Completed);
    assert_eq!(task.state.status(), WorkStatus::Completed);
    task.state.set_status(WorkStatus::Failed);
    assert_eq!(task.state.status(), WorkStatus::Failed);
    task.state.set_status(WorkStatus::Cancelled);
    assert_eq!(task.state.status(), WorkStatus::Cancelled);
}

#[test]
fn work_task_cancel() {
    let task = TestWorkTask::succeeding();

    assert!(!task.state.is_cancelled());
    task.state.cancel();
    assert!(task.state.is_cancelled());
}

#[test]
fn work_task_owner_and_generation() {
    let task = TestWorkTask::succeeding();

    task.state.set_owner_id(42);
    assert_eq!(task.state.owner_id(), 42);
    task.state.set_submit_generation(7);
    assert_eq!(task.state.submit_generation(), 7);
}

#[test]
fn work_task_execute_success() {
    let task = TestWorkTask::succeeding();

    assert!(task.execute());
    assert!(task.state.error_message().is_empty());
}

#[test]
fn work_task_execute_failure() {
    let task = TestWorkTask::succeeding();
    task.execute_result.store(false, Ordering::Release);

    assert!(!task.execute());
    assert_eq!(task.state.error_message(), "test failure");
}
