//! All six cases from pinned `tests/unit_tests/runtime/work_task_test.cpp`
//! against the translated WorkTask owner and its virtual callback boundary.

use nuxie_runtime::{DynWorkTask, WorkCallbacks, WorkStatus, WorkTask};

#[derive(Default)]
struct TestWorkTask {
    execute_result: bool,
}

impl TestWorkTask {
    fn succeeding() -> WorkTask<Self> {
        WorkTask::new(Self {
            execute_result: true,
        })
    }
}

impl WorkCallbacks for TestWorkTask {
    fn execute(&mut self, error_message: &mut String) -> bool {
        if !self.execute_result {
            *error_message = "test failure".into();
        }
        self.execute_result
    }
}

#[test]
fn work_task_default_state() {
    let task = TestWorkTask::succeeding();

    assert_eq!(task.status(), WorkStatus::Pending);
    assert!(!task.is_cancelled());
    assert_eq!(task.owner_id(), 0);
    assert_eq!(task.submit_generation(), 0);
    assert!(task.error_message().is_empty());
}

#[test]
fn work_task_status_transitions() {
    let mut task = TestWorkTask::succeeding();
    let retained_status = task.status_handle();

    task.set_status(WorkStatus::Running);
    assert_eq!(task.status(), WorkStatus::Running);
    assert_eq!(retained_status.status(), WorkStatus::Running);
    task.set_status(WorkStatus::Completed);
    assert_eq!(task.status(), WorkStatus::Completed);
    task.set_status(WorkStatus::Failed);
    assert_eq!(task.status(), WorkStatus::Failed);
    task.set_status(WorkStatus::Cancelled);
    assert_eq!(task.status(), WorkStatus::Cancelled);
    drop(task);
    assert_eq!(retained_status.status(), WorkStatus::Cancelled);
}

#[test]
fn work_task_cancel() {
    let task = TestWorkTask::succeeding();

    assert!(!task.is_cancelled());
    task.cancel();
    assert!(task.is_cancelled());
}

#[test]
fn work_task_owner_and_generation() {
    let mut task = TestWorkTask::succeeding();

    task.set_owner_id(42);
    assert_eq!(task.owner_id(), 42);
    task.set_submit_generation(7);
    assert_eq!(task.submit_generation(), 7);
}

#[test]
fn work_task_execute_success() {
    let mut task = TestWorkTask::succeeding();

    assert!(task.execute());
    assert!(task.error_message().is_empty());
}

#[test]
fn work_task_execute_failure() {
    let mut task = TestWorkTask::succeeding();
    task.callbacks.execute_result = false;

    assert!(!task.execute());
    assert_eq!(task.error_message(), "test failure");
}
