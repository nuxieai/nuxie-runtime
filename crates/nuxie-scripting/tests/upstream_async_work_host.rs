//! Host-frame cases from upstream runtime/async_work_host_test.cpp at 8092a195.
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
    source::viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
    },
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

fn read_file_named(name: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(name),
    )
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

fn read_file() -> RuntimeFileHandle {
    read_file_named("ball_test.riv")
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

fn settled_while_pending(mut advance: impl FnMut(f32) -> bool, tasks: &[Arc<AtomicBool>]) -> usize {
    let all_delivered = || tasks.iter().all(|task| task.load(Ordering::SeqCst));
    let mut frames = 0;
    let mut settled = 0;
    while frames < 1000 && !all_delivered() {
        let keep_going = advance(0.1);
        if !all_delivered() {
            settled += usize::from(!keep_going);
            std::thread::sleep(Duration::from_millis(1));
        }
        frames += 1;
    }
    assert!(all_delivered());
    assert!(frames > 1);
    settled
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
    assert_eq!(
        settled_while_pending(|seconds| machine.advance_and_apply(seconds), &tasks),
        0
    );
    assert!(!machine.advance_and_apply(0.1));
}

#[test]
fn a_finished_animation_keeps_going_until_its_files_async_work_lands() {
    let _serial = HOST_TESTS.lock().unwrap();
    drain_global_async_work();
    let file = read_file();
    let owner = add_scripting_vm(&file);
    let artboard = file
        .with_file(|file| file.artboard_named("Nested Artboard 2"))
        .unwrap();
    let mut animation = artboard.animation_named("Bounce").unwrap();
    animation.advance_and_apply(0.0);
    assert!(!animation.advance_and_apply(100.0));
    let tasks = submit_tasks(owner, 100);
    assert_eq!(
        settled_while_pending(|seconds| animation.advance_and_apply(seconds), &tasks),
        0
    );
    assert!(!animation.advance_and_apply(0.1));
}

#[test]
fn a_settled_machine_keeps_going_until_a_bound_files_async_work_lands() {
    let _serial = HOST_TESTS.lock().unwrap();
    drain_global_async_work();
    let file = read_file_named("swappable_artboards_focus.riv");
    let other_file = read_file_named("swappable_artboards_focus.riv");
    add_scripting_vm(&file);
    let other_owner = add_scripting_vm(&other_file);
    let artboard = file.with_file(|file| file.artboard_named("Main")).unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    let vmi = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    let property = vmi
        .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("artboardProp"))
        .flatten()
        .unwrap();
    let bindable = other_file
        .with_file(|file| file.bindable_artboard_named("Swappable1"))
        .unwrap();
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
            property.set_asset(Some(bindable))
        })
        .unwrap();
    let mut settled = false;
    for _ in 0..100 {
        if settled {
            break;
        }
        settled = !machine.advance_and_apply(0.1);
    }
    assert!(settled);
    let tasks = submit_tasks(other_owner, 100);
    assert_eq!(
        settled_while_pending(|seconds| machine.advance_and_apply(seconds), &tasks),
        0
    );
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

// Rust integration: pool callbacks hand decoded results to a VM-local queue.
// The upstream pool callbacks settle Lua promises directly, including those
// belonging to a foreign-bound file. Plain HostTask completion cannot test it.
#[test]
#[cfg(feature = "compiler")]
fn host_poll_delivers_foreign_and_root_decode_promises_in_pool_order() {
    let _serial = HOST_TESTS.lock().unwrap();
    drain_global_async_work();
    let file = read_file_named("swappable_artboards_focus.riv");
    let other_file = read_file_named("swappable_artboards_focus.riv");
    let root_vm = std::rc::Rc::new(ScriptVm::new());
    root_vm.install_rive_globals().unwrap();
    file.with_file_mut(|file| {
        file.set_scripting_vm(Some(RuntimeScriptingVmHandle::new(Box::new(
            std::rc::Rc::clone(&root_vm),
        ))))
    });
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let lua = vm.lua().clone();
    let vm = std::rc::Rc::new(vm);
    other_file.with_file_mut(|file| {
        file.set_scripting_vm(Some(RuntimeScriptingVmHandle::new(Box::new(
            std::rc::Rc::clone(&vm),
        ))))
    });
    let artboard = file.with_file(|file| file.artboard_named("Main")).unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    let vmi = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    let property = vmi
        .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("artboardProp"))
        .flatten()
        .unwrap();
    let bindable = other_file
        .with_file(|file| file.bindable_artboard_named("Swappable1"))
        .unwrap();
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
            property.set_asset(Some(bindable))
        })
        .unwrap();
    let mut settled = false;
    for _ in 0..100 {
        if settled {
            break;
        }
        settled = !machine.advance_and_apply(0.1);
    }
    assert!(settled);
    let order = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    for (vm, name) in [(&vm, "foreign"), (&root_vm, "root")] {
        let order = std::rc::Rc::clone(&order);
        let callback = vm
            .lua()
            .create_function(move |_, ()| {
                order.borrow_mut().push(name);
                Ok(())
            })
            .unwrap();
        vm.lua().globals().set("recordDecode", callback).unwrap();
    }
    let promise = vm.upstream_test_decode_image(&[0, 0, 0, 0]).unwrap();
    lua.globals().set("decodePromise", promise).unwrap();
    lua.load("decodePromise:catch(function() recordDecode() end)")
        .exec()
        .unwrap();
    // Force this completion to enter the handoff queue first, even on workers.
    // Poll only the pool, not the host/VM promise delivery boundary.
    for pending_vm in [&vm, &root_vm] {
        if std::rc::Rc::ptr_eq(pending_vm, &root_vm) {
            let promise = root_vm.upstream_test_decode_image(&[0, 0, 0, 0]).unwrap();
            root_vm
                .lua()
                .globals()
                .set("decodePromise", promise)
                .unwrap();
            root_vm
                .lua()
                .load("decodePromise:catch(function() recordDecode() end)")
                .exec()
                .unwrap();
        }
        for _ in 0..1000 {
            rive_poll_async_work(32);
            if nuxie_runtime::ScriptingVm::pending_async_work_sequence(pending_vm.as_ref())
                .is_some()
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            nuxie_runtime::ScriptingVm::pending_async_work_sequence(pending_vm.as_ref()).is_some()
        );
        assert_eq!(
            pending_vm
                .lua()
                .load("return decodePromise:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Pending"
        );
    }
    assert_eq!(
        lua.load("return decodePromise:getStatus()")
            .eval::<String>()
            .unwrap(),
        "Pending"
    );
    for _ in 0..1000 {
        let keep_going = machine.advance_and_apply(0.1);
        if lua
            .load("return decodePromise:getStatus()")
            .eval::<String>()
            .unwrap()
            != "Pending"
        {
            break;
        }
        assert!(keep_going, "foreign decode must keep its host ticking");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        lua.load("return decodePromise:getStatus()")
            .eval::<String>()
            .unwrap(),
        "Rejected"
    );
    assert_eq!(*order.borrow(), ["foreign", "root"]);
    assert_eq!(
        root_vm
            .lua()
            .load("return decodePromise:getStatus()")
            .eval::<String>()
            .unwrap(),
        "Rejected"
    );
    assert!(!machine.advance_and_apply(0.1));
}
