//! Public scripting_module_bound_test.cpp cases at upstream 6ec0b04b.
//!
//! Rust retains its existing 100,000-safepoint ceiling. A spin can hit that
//! ceiling before the new wall-clock deadline; the separate expired-deadline
//! test exercises the upstream timeout error and flag without removing it.
use super::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const WATCHDOG: Duration = Duration::from_millis(8000);
const WATCHDOG_MESSAGE: &str = "test watchdog fired";

// Construct the non-Send VM on the worker. The independent receiver bounds the
// test even if both production interrupt limits regress; no callback is replaced.
fn spin(timeout: Option<i32>) -> (String, bool, i32, Duration) {
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        let vm = ScriptVm::new();
        if let Some(timeout) = timeout {
            vm.set_timeout_ms(timeout);
            assert_eq!(vm.timeout_ms(), timeout);
        }
        let start = Instant::now();
        let result = vm.register_source_module("spin", "while true do end");
        let elapsed = start.elapsed();
        let error = result
            .expect_err("an endless module must not register")
            .to_string();
        send.send((error, vm.budget_exceeded(), vm.timeout_ms(), elapsed))
            .ok();
    });
    receive.recv_timeout(WATCHDOG).expect(WATCHDOG_MESSAGE)
}

fn assert_execution_bound(error: &str, exceeded: bool) {
    if error.contains("execution exceeded") {
        assert!(exceeded, "wall-clock exhaustion must set budgetExceeded");
    } else {
        // Explicit approved Rust adaptation, not a replacement error accepted
        // indiscriminately: retain the stronger existing safepoint restriction.
        assert!(
            error.contains("script cycle exceeds 100000 script safepoints"),
            "{error}"
        );
        assert!(!exceeded, "a safepoint failure is not a wall-clock failure");
    }
    assert!(!error.contains(WATCHDOG_MESSAGE));
}

#[test]
fn a_module_top_level_that_never_terminates_is_stopped() {
    let (error, exceeded, _, elapsed) = spin(None);
    assert_execution_bound(&error, exceeded);
    assert!(elapsed < WATCHDOG, "{elapsed:?}: {error}");
    assert!(
        elapsed < Duration::from_millis(3 * MODULE_EXECUTION_TIMEOUT_MS as u64),
        "{elapsed:?}: {error}"
    );
}

#[test]
fn a_module_top_level_that_does_real_work_still_registers() {
    let vm = ScriptVm::new();
    let source = r#"
local entries = {}
for i = 1, 20000 do
    entries[i] = i
end
local sum = 0
for i = 1, #entries do
    sum += entries[i]
end
return { sum = sum }
"#;
    vm.register_source_module("summed", source)
        .expect("real work registers without error");
    let sum: f64 = vm
        .lua
        .load("return require(\"summed\").sum")
        .eval()
        .unwrap();
    assert_eq!(sum, 200010000.0);
}

#[test]
fn module_execution_raises_a_host_timeout_to_the_module_floor() {
    let vm = ScriptVm::new();
    let floor = MODULE_EXECUTION_TIMEOUT_MS;
    let below = floor / 2;
    let above = floor * 2;
    assert!(below > 0);
    for timeout in [0, below, above] {
        vm.set_timeout_ms(timeout);
        assert_eq!(vm.timeout_ms(), timeout);
        let deadline_before = vm.module_execution_timing.deadline.get();
        assert!(deadline_before.is_none());
        let execution = vm.start_module_execution().unwrap();
        assert_eq!(vm.timeout_ms(), floor.max(timeout));
        assert!(vm.module_execution_timing.deadline.get().is_some());
        drop(execution);
        assert_eq!(vm.timeout_ms(), timeout);
        assert_eq!(vm.module_execution_timing.deadline.get(), deadline_before);
    }
    // The scoped module interrupt preserves resource checks while temporarily
    // replacing the host callback; guard drop restores that callback and deadline.
}

#[test]
fn a_failed_module_top_level_restores_the_host_timeout() {
    let timeout = MODULE_EXECUTION_TIMEOUT_MS / 4;
    assert!(timeout > 0);
    let (error, exceeded, restored, _) = spin(Some(timeout));
    assert_execution_bound(&error, exceeded);
    assert_eq!(restored, timeout);
}

#[test]
fn expired_module_deadline_sets_the_upstream_error_and_budget_flag() {
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        let vm = ScriptVm::new();
        vm.set_timeout_ms(0);
        let execution = vm.start_module_execution().unwrap();
        assert_eq!(vm.timeout_ms(), MODULE_EXECUTION_TIMEOUT_MS);
        assert!(!vm.budget_exceeded());
        // Drive the real interrupt through an already-expired deadline. Do not
        // replace that interrupt or raise/remove the existing safepoint ceiling.
        vm.module_execution_timing.deadline.set(Some(
            nuxie_runtime::source::engine_now_micros().saturating_sub(2000),
        ));
        let error = vm
            .lua
            .load("while true do end")
            .exec()
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("execution exceeded 1.0 second timeout"),
            "{error}"
        );
        assert!(vm.budget_exceeded());
        assert!(!error.contains(WATCHDOG_MESSAGE));
        drop(execution);
        assert_eq!(vm.timeout_ms(), 0);
        assert!(vm.module_execution_timing.deadline.get().is_none());
        send.send(()).unwrap();
    });
    receive.recv_timeout(WATCHDOG).expect(WATCHDOG_MESSAGE);
}

// Rust integration: public Lua host interrupt controls must not disable the
// scoped upstream module deadline, and the caller's callback must be restored.
#[test]
fn module_deadline_survives_host_interrupt_replacement_or_removal() {
    for remove in [false, true] {
        let (send, receive) = mpsc::channel();
        std::thread::spawn(move || {
            let vm = ScriptVm::new();
            let calls = Rc::new(Cell::new(0usize));
            let count = calls.clone();
            vm.lua().set_interrupt(move |_| {
                count.set(count.get() + 1);
                Ok(luaur_rt::VmState::Continue)
            });
            if remove {
                vm.lua().remove_interrupt();
            }
            let execution = vm.start_module_execution().unwrap();
            vm.module_execution_timing.deadline.set(Some(
                nuxie_runtime::source::engine_now_micros().saturating_sub(2000),
            ));
            let error = vm
                .lua()
                .load("while true do end")
                .exec()
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("execution exceeded 1.0 second timeout"),
                "{error}"
            );
            assert!(vm.budget_exceeded());
            drop(execution);
            let calls_before = calls.get();
            let value: i32 = vm
                .lua()
                .load("local n = 0; for i = 1, 100 do n += 1 end; return n")
                .eval()
                .unwrap();
            assert_eq!(value, 100);
            if remove {
                assert_eq!(calls.get(), calls_before);
            } else {
                assert!(calls.get() > calls_before, "host callback was not restored");
            }
            send.send(()).unwrap();
        });
        receive.recv_timeout(WATCHDOG).expect(WATCHDOG_MESSAGE);
    }
}

#[test]
fn module_error_hook_observes_restored_timeout_and_can_run_lua() {
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        struct Hook {
            vm: std::rc::Weak<ScriptVm>,
            called: Cell<bool>,
        }
        impl ModuleHooks for Hook {
            fn on_module_error(&self, _: &Thread) {
                let vm = self.vm.upgrade().unwrap();
                assert_eq!(vm.timeout_ms(), 250);
                assert!(vm.module_execution_timing.deadline.get().is_none());
                assert!(vm.budget_exceeded());
                let value: i32 = vm
                    .lua()
                    .load("local n = 0; for i = 1, 100 do n += 1 end; return n")
                    .eval()
                    .unwrap();
                assert_eq!(value, 100);
                self.called.set(true);
            }
        }
        let vm = Rc::new(ScriptVm::new());
        vm.set_timeout_ms(250);
        let timing = vm.module_execution_timing.clone();
        vm.lua()
            .globals()
            .set(
                "expireDeadline",
                vm.lua()
                    .create_function(move |_, (): ()| {
                        timing.deadline.set(Some(
                            nuxie_runtime::source::engine_now_micros().saturating_sub(2000),
                        ));
                        Ok(())
                    })
                    .unwrap(),
            )
            .unwrap();
        let hook = Rc::new(Hook {
            vm: Rc::downgrade(&vm),
            called: Cell::new(false),
        });
        vm.set_module_hooks(Some(hook.clone()));
        let error = vm
            .register_source_module("expired", "expireDeadline(); while true do end")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("execution exceeded 1.0 second timeout"),
            "{error}"
        );
        assert!(hook.called.get());
        assert_eq!(vm.timeout_ms(), 250);
        assert!(vm.module_execution_timing.deadline.get().is_none());
        send.send(()).unwrap();
    });
    receive.recv_timeout(WATCHDOG).expect(WATCHDOG_MESSAGE);
}

#[test]
fn dropping_scoped_interrupt_allows_captured_destructor_to_reenter_interrupt_registry() {
    struct RemoveOtherInterrupt {
        lua: Lua,
        dropped: Rc<Cell<bool>>,
    }
    impl RemoveOtherInterrupt {
        fn is_dropped(&self) -> bool {
            self.dropped.get()
        }
    }
    impl Drop for RemoveOtherInterrupt {
        fn drop(&mut self) {
            self.lua.remove_interrupt();
            self.dropped.set(true);
        }
    }
    let vm = ScriptVm::new();
    let other = ScriptVm::new();
    let dropped = Rc::new(Cell::new(false));
    let captured = RemoveOtherInterrupt {
        lua: other.lua().clone(),
        dropped: dropped.clone(),
    };
    let guard = vm.lua().scoped_interrupt(move |_| {
        assert!(!captured.is_dropped());
        Ok(luaur_rt::VmState::Continue)
    });
    assert!(!dropped.get());
    // The callback destructor mutates the same thread-local registry for a
    // different VM. Drop must release the registry borrow before destroying it.
    drop(guard);
    assert!(dropped.get());
    let result: i32 = vm.lua().load("return 42").eval().unwrap();
    assert_eq!(result, 42);
}
