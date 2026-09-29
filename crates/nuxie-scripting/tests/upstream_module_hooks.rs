//! c5faa1fa: ScriptingContext module hooks observe the actual module thread
//! after loading, or before consuming its top-level execution error.
#![cfg(feature = "luau")]

use std::{cell::RefCell, rc::Rc};

use luaur_rt::{Error, Function, Table, Thread, Value};
use nuxie_scripting::vm::{ModuleHooks, ScriptVm};

mod support;

#[derive(Default)]
struct Hooks {
    loaded: RefCell<Vec<(usize, String, Function)>>,
    errors: RefCell<Vec<(usize, String)>>,
}

impl ModuleHooks for Hooks {
    fn on_module_loaded(&self, thread: &Thread, name: &str) {
        let Value::Function(closure) = thread.stack_value(-1).unwrap() else {
            panic!("loaded module must still have its closure on top");
        };
        self.loaded
            .borrow_mut()
            .push((thread.to_pointer() as usize, name.to_owned(), closure));
    }

    fn on_module_error(&self, thread: &Thread) {
        let Value::String(error) = thread.stack_value(-1).unwrap() else {
            panic!("module error must be normalized before the hook");
        };
        // Inspect, but do not unwind or consume, the failed/yielded frames.
        let mut debug = std::mem::MaybeUninit::zeroed();
        let has_frame = unsafe {
            luaur_vm::functions::lua_getinfo::lua_getinfo(
                thread.state(),
                0,
                c"s".as_ptr(),
                debug.as_mut_ptr(),
            )
        }
        .unwrap();
        assert_eq!(has_frame, 1, "hook must precede frame cleanup");
        self.errors.borrow_mut().push((
            thread.to_pointer() as usize,
            error.to_str().unwrap().to_owned(),
        ));
    }
}

fn vm_with_hooks() -> (ScriptVm, Rc<Hooks>) {
    let vm = ScriptVm::new();
    let hooks = Rc::new(Hooks::default());
    vm.set_module_hooks(Some(hooks.clone()));
    (vm, hooks)
}

#[test]
fn loaded_hook_observes_sandboxed_closure_before_same_thread_executes() {
    let (vm, hooks) = vm_with_hooks();
    let bytecode = support::compile_source("moduleRan = true; return { ran = moduleRan }").unwrap();
    let thread = vm.load_module("loaded-module", &bytecode).unwrap();
    let identity = thread.to_pointer() as usize;
    let loaded = hooks.loaded.borrow();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].0, identity);
    assert_eq!(loaded[0].1, "loaded-module");
    let closure = loaded[0].2.clone();
    drop(loaded);
    let environment = closure.environment().unwrap();
    assert_ne!(environment.to_pointer(), vm.lua().globals().to_pointer());
    assert!(matches!(
        environment.get::<Value>("moduleRan").unwrap(),
        Value::Nil
    ));
    let Value::Function(on_stack) = thread.stack_value(-1).unwrap() else {
        panic!("hook inspection must not consume the closure");
    };
    assert_eq!(on_stack.to_pointer(), closure.to_pointer());
    let result: Table = vm.execute_module("loaded-module", thread).unwrap();
    assert!(result.get::<bool>("ran").unwrap());
    assert!(environment.get::<bool>("moduleRan").unwrap());
    assert!(matches!(
        vm.lua().globals().get::<Value>("moduleRan").unwrap(),
        Value::Nil
    ));
    assert_eq!(hooks.loaded.borrow().len(), 1);
    assert!(hooks.errors.borrow().is_empty());
}

#[test]
fn runtime_error_hook_keeps_same_thread_frames_and_error() {
    let (vm, hooks) = vm_with_hooks();
    let bytecode = support::compile_source(
        "local function fail() error('module boom') end; fail(); return {}",
    )
    .unwrap();
    let thread = vm.load_module("failing-module", &bytecode).unwrap();
    let identity = thread.to_pointer() as usize;
    let error = vm
        .execute_module::<Value>("failing-module", thread)
        .unwrap_err();
    assert!(error.to_string().contains("module boom"));
    let errors = hooks.errors.borrow();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, identity);
    assert_eq!(hooks.loaded.borrow()[0].0, identity);
    assert!(errors[0].1.contains("module boom"));
}

#[test]
fn yield_hook_observes_normalized_module_error_and_live_frames() {
    let (vm, hooks) = vm_with_hooks();
    let bytecode = support::compile_source("coroutine.yield('not an error'); return {}").unwrap();
    let thread = vm.load_module("yielding-module", &bytecode).unwrap();
    let identity = thread.to_pointer() as usize;
    let error = vm
        .execute_module::<Value>("display-name", thread)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("display-name:1: module can not yield")
    );
    let errors = hooks.errors.borrow();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, identity);
    assert_eq!(hooks.loaded.borrow()[0].0, identity);
    assert_eq!(errors[0].1, "display-name:1: module can not yield");
}

#[test]
fn rejected_bytecode_does_not_fire_module_hooks() {
    let (vm, hooks) = vm_with_hooks();
    assert!(vm.load_module("invalid", &[0xff, 0, 0x13, 0x37]).is_err());
    assert!(vm.load_module("empty", &[]).is_err());
    assert!(hooks.loaded.borrow().is_empty());
    assert!(hooks.errors.borrow().is_empty());
}

#[test]
fn successful_resume_with_invalid_result_does_not_fire_error_hook() {
    for (source, message) in [
        ("return", "display-name:1: module must return a value"),
        (
            "return 42",
            "display-name:1: module must return a table or function",
        ),
    ] {
        let (vm, hooks) = vm_with_hooks();
        let bytecode = support::compile_source(source).unwrap();
        let thread = vm.load_module("result-validation", &bytecode).unwrap();
        let error = vm
            .execute_module::<Value>("display-name", thread)
            .unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
        assert_eq!(hooks.loaded.borrow().len(), 1);
        // Upstream dispatches onModuleError only for nonzero resume status,
        // not for errors synthesized while validating a successful return.
        assert!(hooks.errors.borrow().is_empty());
    }
}

#[test]
fn successful_module_selects_last_result_without_error_hook() {
    for (source, expect_function) in [
        ("return {}, function() end", true),
        ("return function() end, {}", false),
    ] {
        let (vm, hooks) = vm_with_hooks();
        let bytecode = support::compile_source(source).unwrap();
        let thread = vm.load_module("multiple-results", &bytecode).unwrap();
        let result: Value = vm.execute_module("multiple-results", thread).unwrap();
        if expect_function {
            assert!(matches!(result, Value::Function(_)));
        } else {
            assert!(matches!(result, Value::Table(_)));
        }
        assert_eq!(hooks.loaded.borrow().len(), 1);
        assert!(hooks.errors.borrow().is_empty());
    }
}

#[test]
fn host_structured_error_retains_cause_and_is_visible_to_module_hook() {
    let (vm, hooks) = vm_with_hooks();
    vm.lua()
        .scope(|scope| {
            let callback = scope.create_function(|_, ()| Ok(()))?;
            vm.lua().globals().set("hostFailure", callback)
        })
        .unwrap();
    let bytecode = support::compile_source("hostFailure(); return {}").unwrap();
    let thread = vm.load_module("host-error", &bytecode).unwrap();
    let identity = thread.to_pointer() as usize;
    let error = vm
        .execute_module::<Value>("host-error", thread)
        .unwrap_err();
    let Error::CallbackError { cause, .. } = error else {
        panic!("module execution must retain the structured host cause: {error}");
    };
    assert!(matches!(cause.as_ref(), Error::CallbackDestructed));
    let errors = hooks.errors.borrow();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, identity);
    assert_eq!(hooks.loaded.borrow()[0].0, identity);
    assert_eq!(
        errors[0].1,
        "host-error:1: unknown error while running module"
    );
}

#[test]
fn arbitrary_non_string_lua_error_is_normalized_for_module_hook() {
    let (vm, hooks) = vm_with_hooks();
    let bytecode = support::compile_source("error({}); return {}").unwrap();
    let thread = vm.load_module("lua-error", &bytecode).unwrap();
    let identity = thread.to_pointer() as usize;
    let error = vm
        .execute_module::<Value>("display-name", thread)
        .unwrap_err();
    let expected = "display-name:1: unknown error while running module";
    assert!(matches!(&error, Error::RuntimeError(message) if message == expected));
    let errors = hooks.errors.borrow();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, identity);
    assert_eq!(hooks.loaded.borrow()[0].0, identity);
    assert_eq!(errors[0].1, expected);
}
