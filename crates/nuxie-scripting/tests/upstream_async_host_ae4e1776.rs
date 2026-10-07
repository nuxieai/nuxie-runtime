//! Host-frame cases from upstream runtime/work_pool_test.cpp at ae4e1776.
#![cfg(all(feature = "luau", feature = "upstream-test-seams"))]

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeScriptingVmHandle, WorkCallbacks, WorkPool, WorkTask, get_global_work_pool,
    rive_has_pending_async_work, rive_poll_async_work,
    source::animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
};
use nuxie_scripting::vm::ScriptVm;

// The upstream cases share a process-global pool and run serially.
static HOST_TESTS: Mutex<()> = Mutex::new(());

struct TestTask(Arc<AtomicBool>);
impl WorkCallbacks for TestTask {
    fn execute(&mut self, _: &mut String) -> bool {
        true
    }
    fn on_complete(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn drain_global_async_work() {
    for _ in 0..100 {
        if !rive_has_pending_async_work() {
            break;
        }
        rive_poll_async_work(32);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!rive_has_pending_async_work());
}

fn read_file() -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let bytes =
        std::fs::read(std::path::PathBuf::from(root).join("tests/unit_tests/assets/ball_test.riv"))
            .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}

fn add_scripting_vm(file: &RuntimeFileHandle) -> u64 {
    let vm = ScriptVm::new();
    let owner = vm.upstream_test_async_owner_id();
    file.with_file_mut(|file| {
        file.set_scripting_vm(Some(RuntimeScriptingVmHandle::new(Box::new(vm))))
    });
    owner
}

fn settled_machine(
    file: &RuntimeFileHandle,
) -> (
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let artboard = file
        .with_file(|file| file.artboard_named("Artboard 2"))
        .unwrap();
    artboard.advance_default(0.0);
    let machine = artboard.state_machine_at(0).unwrap();
    machine.advance_and_apply(0.0);
    machine.advance_and_apply(0.9);
    machine.advance_and_apply(0.1);
    machine.advance_and_apply(0.1);
    assert!(!machine.advance_and_apply(0.1));
    (artboard, machine)
}

fn submit_tasks(owner: u64, count: usize) -> Vec<Arc<AtomicBool>> {
    (0..count)
        .map(|_| {
            let completed = Arc::new(AtomicBool::new(false));
            let mut task = WorkTask::new(TestTask(Arc::clone(&completed)));
            task.set_owner_id(owner);
            get_global_work_pool()
                .lock()
                .unwrap()
                .submit(Some(Box::new(task)));
            completed
        })
        .collect()
}

#[test]
fn state_machine_advance_and_apply_delivers_async_work() {
    let _serial = HOST_TESTS.lock().unwrap();
    drain_global_async_work();
    let file = read_file();
    let (_artboard, machine) = settled_machine(&file);
    let tasks = submit_tasks(0, 1);
    for _ in 0..1000 {
        if tasks[0].load(Ordering::SeqCst) {
            break;
        }
        machine.advance_and_apply(0.1);
        if !tasks[0].load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    assert!(tasks[0].load(Ordering::SeqCst));
}

#[test]
fn a_settled_machine_keeps_going_until_its_files_async_work_lands() {
    let _serial = HOST_TESTS.lock().unwrap();
    drain_global_async_work();
    let file = read_file();
    let owner = add_scripting_vm(&file);
    let (_artboard, machine) = settled_machine(&file);
    let tasks = submit_tasks(owner, 100);
    let all_delivered = || tasks.iter().all(|task| task.load(Ordering::SeqCst));
    let mut frames = 0;
    let mut settled_while_pending = 0;
    while frames < 1000 && !all_delivered() {
        let keep_going = machine.advance_and_apply(0.1);
        if !all_delivered() {
            settled_while_pending += usize::from(!keep_going);
            std::thread::sleep(Duration::from_millis(1));
        }
        frames += 1;
    }
    assert!(all_delivered());
    assert!(frames > 1);
    assert_eq!(settled_while_pending, 0);
    assert!(!machine.advance_and_apply(0.1));
}

#[test]
fn another_owners_async_work_does_not_hold_a_settled_machine() {
    let _serial = HOST_TESTS.lock().unwrap();
    drain_global_async_work();
    let file = read_file();
    add_scripting_vm(&file);
    let (_artboard, machine) = settled_machine(&file);
    let _tasks = submit_tasks(WorkPool::next_owner_id(), 100);
    let keep_going = machine.advance_and_apply(0.1);
    assert!(rive_has_pending_async_work());
    assert!(!keep_going);
    drain_global_async_work();
}
