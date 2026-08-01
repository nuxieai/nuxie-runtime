#![cfg(feature = "luau")]

use luaur_compiler::functions::luau_compile::luau_compile;
use luaur_rt::{Function, Table, Value};
use nuxie_scripting::vm::ScriptVm;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Clone, Copy)]
enum Expected {
    Exact(&'static str),
    Contains(&'static str),
}

struct Scenario {
    name: &'static str,
    source: &'static str,
    expected: Expected,
}

fn normalize(value: Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.to_string_lossy(),
        other => panic!("Promise oracle scenario returned unsupported {other:?}"),
    }
}

fn assert_scenario(scenario: &Scenario, actual: &str) {
    match scenario.expected {
        Expected::Exact(expected) => assert_eq!(actual, expected, "{}", scenario.name),
        Expected::Contains(expected) => assert!(
            actual.contains(expected),
            "{}: expected {actual:?} to contain {expected:?}",
            scenario.name
        ),
    }
}

fn compile_luau(source: &str) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null(), "pinned Luau compiler returned null");
    // SAFETY: luau_compile returns a malloc allocation of output_size bytes.
    let bytecode = unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec();
    unsafe extern "C" {
        fn free(pointer: *mut std::ffi::c_void);
    }
    // SAFETY: output is the allocation returned by luau_compile above.
    unsafe { free(output.cast()) };
    bytecode
}

fn run_rust_scenario(vm: &ScriptVm, scenario: &Scenario) -> String {
    let function = vm
        .load("promise_scenario", scenario.source)
        .unwrap_or_else(|error| panic!("{}: {error}", scenario.name));
    normalize(
        function
            .call::<Value>(())
            .unwrap_or_else(|error| panic!("{}: {error}", scenario.name)),
    )
}

fn cpp_oracle_path() -> PathBuf {
    let configured = PathBuf::from(
        std::env::var_os("NUXIE_CPP_PROMISE_ORACLE")
            .expect("NUXIE_CPP_PROMISE_ORACLE is unset; run `make promise-differential`"),
    );
    assert!(
        configured.is_file(),
        "C++ Promise oracle does not exist at {}",
        configured.display()
    );
    configured
}

fn run_cpp_oracle(oracle: &PathBuf, scenario: &Scenario) -> String {
    let mut child = Command::new(oracle)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("{}: could not start C++ oracle: {error}", scenario.name));
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&compile_luau(scenario.source))
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}: C++ oracle failed: {}",
        scenario.name,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn promise_scenarios_match_checked_in_pinned_oracle_baseline() {
    assert_eq!(SCENARIOS.len(), 47, "the pinned C++ oracle has 47 cases");

    for scenario in SCENARIOS {
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let actual = run_rust_scenario(&vm, scenario);
        assert_scenario(scenario, &actual);
    }

    for scenario in COROUTINE_ERROR_SCENARIOS {
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let actual = run_rust_scenario(&vm, scenario);
        assert_scenario(scenario, &actual);
    }
}

#[test]
#[ignore = "requires pinned C++ libraries; run `make promise-differential`"]
fn promise_scenarios_match_live_cpp_oracle() {
    assert_eq!(SCENARIOS.len(), 47, "the pinned C++ oracle has 47 cases");
    let cpp_oracle = cpp_oracle_path();

    for scenario in SCENARIOS.iter().chain(COROUTINE_ERROR_SCENARIOS) {
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let actual = run_rust_scenario(&vm, scenario);
        let cpp_actual = run_cpp_oracle(&cpp_oracle, scenario);

        assert_scenario(scenario, &actual);
        assert_scenario(scenario, &cpp_actual);
        assert_eq!(actual, cpp_actual, "{}", scenario.name);
    }
}

#[test]
fn promise_public_contract_and_registry_lifetime() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let (kind, status, weak): (String, String, Table) = vm
        .eval(
            r#"
            local fulfilled = Promise.resolve(1)
            local pending = Promise.new(function() end)
            local child = pending:andThen(function() end)
            local weak = setmetatable({ fulfilled, pending, child }, { __mode = "v" })
            fulfilled = nil
            pending = nil
            child = nil
            return type(Promise.resolve()),
                Promise.resolve():getStatus(),
                weak
            "#,
        )
        .unwrap();

    vm.lua().gc_collect().unwrap();
    vm.lua().gc_collect().unwrap();

    assert_eq!(kind, "userdata");
    assert_eq!(status, "Fulfilled");
    assert!(matches!(weak.get::<Value>(1).unwrap(), Value::Nil));
    assert!(matches!(weak.get::<Value>(2).unwrap(), Value::UserData(_)));
    assert!(matches!(weak.get::<Value>(3).unwrap(), Value::UserData(_)));

    let cancel_retained_chain: Function = vm
        .lua()
        .load("return function(values) values[2]:cancel() end")
        .eval()
        .unwrap();
    cancel_retained_chain.call::<()>(weak.clone()).unwrap();
    vm.lua().gc_collect().unwrap();
    vm.lua().gc_collect().unwrap();
    assert!(matches!(weak.get::<Value>(2).unwrap(), Value::Nil));
    assert!(matches!(weak.get::<Value>(3).unwrap(), Value::Nil));
}

#[test]
fn pending_await_resume_and_cancel_ownership_contract() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let resumed: String = vm
        .eval(
            r#"
            local resolveLater
            local awaited = Promise.new(function(resolve) resolveLater = resolve end)
            local continued = false
            local value = 0
            local result = async(function()
                local ok, resolved = await(awaited)
                continued = true
                return ok and resolved * 2 or -1
            end)
            local before = result:getStatus()
            resolveLater(21)
            result:andThen(function(resolved) value = resolved end)
            return before .. "," .. result:getStatus() .. "," .. tostring(continued) .. "," .. tostring(value)
            "#,
        )
        .unwrap();
    assert_eq!(resumed, "Pending,Fulfilled,true,42");

    let cancelled: String = vm
        .eval(
            r#"
            local awaited = Promise.new(function() end)
            local continued = false
            local caught = ""
            local result = async(function()
                await(awaited)
                continued = true
            end)
            result:catch(function(reason) caught = reason end)
            awaited:cancel()
            return result:getStatus() .. "," .. tostring(continued) .. "," .. caught
            "#,
        )
        .unwrap();
    assert_eq!(cancelled, "Rejected,false,Promise was cancelled");
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "Promise.resolve creates a fulfilled promise",
        source: r#"
            local p = Promise.resolve(42)
            local result = 0
            p:andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("42"),
    },
    Scenario {
        name: "Promise.reject creates a rejected promise",
        source: r#"
            local caught = ""
            Promise.reject("oops"):catch(function(err) caught = err end)
            return caught
        "#,
        expected: Expected::Exact("oops"),
    },
    Scenario {
        name: "Promise.reject with no args uses default reason",
        source: r#"
            local caught = ""
            Promise.reject():catch(function(err) caught = err end)
            return caught
        "#,
        expected: Expected::Exact("rejected"),
    },
    Scenario {
        name: "andThen chains propagate values",
        source: r#"
            local result = 0
            Promise.resolve(10)
                :andThen(function(v) return v * 2 end)
                :andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("20"),
    },
    Scenario {
        name: "andThen propagates to catch on rejection",
        source: r#"
            local caught = ""
            Promise.reject("fail")
                :andThen(function() return "should not happen" end)
                :catch(function(err) caught = err end)
            return caught
        "#,
        expected: Expected::Exact("fail"),
    },
    Scenario {
        name: "finally runs on fulfillment",
        source: r#"
            local ran = false
            Promise.resolve(1):finally(function() ran = true end)
            return ran
        "#,
        expected: Expected::Exact("true"),
    },
    Scenario {
        name: "finally runs on rejection",
        source: r#"
            local ran = false
            Promise.reject("err"):finally(function() ran = true end)
            return ran
        "#,
        expected: Expected::Exact("true"),
    },
    Scenario {
        name: "Promise.all resolves when all resolve",
        source: r#"
            local result = 0
            Promise.all({ Promise.resolve(1), Promise.resolve(2), Promise.resolve(3) })
                :andThen(function(values) result = values[1] + values[2] + values[3] end)
            return result
        "#,
        expected: Expected::Exact("6"),
    },
    Scenario {
        name: "Promise.all rejects if any rejects",
        source: r#"
            local caught = ""
            Promise.all({ Promise.resolve(1), Promise.reject("boom"), Promise.resolve(3) })
                :catch(function(err) caught = err end)
            return caught
        "#,
        expected: Expected::Exact("boom"),
    },
    Scenario {
        name: "Promise.all with empty array resolves immediately",
        source: r#"
            local result = nil
            Promise.all({}):andThen(function(values) result = #values end)
            return result
        "#,
        expected: Expected::Exact("0"),
    },
    Scenario {
        name: "async/await with already-resolved promise",
        source: r#"
            local result = 0
            async(function()
                local ok, value = await(Promise.resolve(99))
                result = value
            end)
            return result
        "#,
        expected: Expected::Exact("99"),
    },
    Scenario {
        name: "async/await chains multiple awaits",
        source: r#"
            local result = 0
            async(function()
                local ok1, a = await(Promise.resolve(10))
                local ok2, b = await(Promise.resolve(20))
                result = a + b
            end)
            return result
        "#,
        expected: Expected::Exact("30"),
    },
    Scenario {
        name: "await returns false and error on rejection",
        source: r#"
            local gotOk = nil
            local gotErr = ""
            async(function()
                local ok, err = await(Promise.reject("async_error"))
                gotOk = ok
                gotErr = err
            end)
            return tostring(gotOk) .. "," .. gotErr
        "#,
        expected: Expected::Exact("false,async_error"),
    },
    Scenario {
        name: "await rejection resumes coroutine so cleanup code runs",
        source: r#"
            local cleanupRan = false
            local caughtErr = ""
            local p = Promise.new(function(resolve, reject) reject("deferred_fail") end)
            async(function()
                local ok, err = await(p)
                caughtErr = tostring(err)
                cleanupRan = true
            end)
            return tostring(cleanupRan) .. "," .. caughtErr
        "#,
        expected: Expected::Exact("true,deferred_fail"),
    },
    Scenario {
        name: "await on cancelled promise returns false and message",
        source: r#"
            local gotOk = nil
            local gotMsg = ""
            local p = Promise.new(function() end)
            p:cancel()
            async(function()
                local ok, msg = await(p)
                gotOk = ok
                gotMsg = msg
            end)
            return tostring(gotOk) .. "," .. gotMsg
        "#,
        expected: Expected::Exact("false,Promise was cancelled"),
    },
    Scenario {
        name: "async returns a promise that resolves with return value",
        source: r#"
            local result = 0
            async(function() return 42 end):andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("42"),
    },
    Scenario {
        name: "andThen handler error rejects chained promise",
        source: r#"
            local caught = ""
            Promise.resolve(1)
                :andThen(function() error("handler_error") end)
                :catch(function(err) caught = tostring(err) end)
            return caught
        "#,
        expected: Expected::Contains("handler_error"),
    },
    Scenario {
        name: "finally error rejects chained promise",
        source: r#"
            local caught = ""
            Promise.resolve(1)
                :finally(function() error("finally_boom") end)
                :catch(function(err) caught = tostring(err) end)
            return caught
        "#,
        expected: Expected::Contains("finally_boom"),
    },
    Scenario {
        name: "cancelled promise does not fire handlers",
        source: r#"
            local thenRan, catchRan, finallyRan = false, false, false
            local p = Promise.new(function(resolve, reject, onCancel) onCancel(function() end) end)
            p:andThen(function() thenRan = true end):catch(function() catchRan = true end)
            p:finally(function() finallyRan = true end)
            p:cancel()
            return tostring(thenRan) .. "," .. tostring(catchRan) .. "," .. tostring(finallyRan)
        "#,
        expected: Expected::Exact("false,false,false"),
    },
    Scenario {
        name: "Promise.new creates a pending promise",
        source: "return Promise.new(function() end):getStatus()",
        expected: Expected::Exact("Pending"),
    },
    Scenario {
        name: "Promise.new resolve settles the promise",
        source: r#"
            local result = 0
            Promise.new(function(resolve) resolve(42) end):andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("42"),
    },
    Scenario {
        name: "Promise.new reject settles the promise",
        source: r#"
            local caught = ""
            Promise.new(function(resolve, reject) reject("oops") end):catch(function(err) caught = err end)
            return caught
        "#,
        expected: Expected::Exact("oops"),
    },
    Scenario {
        name: "Promise.new executor error rejects the promise",
        source: r#"
            local caught = ""
            Promise.new(function() error("executor_boom") end):catch(function(err) caught = tostring(err) end)
            return caught
        "#,
        expected: Expected::Contains("executor_boom"),
    },
    Scenario {
        name: "cancel sets state to Cancelled",
        source: r#"
            local p = Promise.new(function() end)
            p:cancel()
            return p:getStatus()
        "#,
        expected: Expected::Exact("Cancelled"),
    },
    Scenario {
        name: "cancel fires onCancel hook",
        source: r#"
            local hookFired = false
            local p = Promise.new(function(resolve, reject, onCancel)
                onCancel(function() hookFired = true end)
            end)
            p:cancel()
            return hookFired
        "#,
        expected: Expected::Exact("true"),
    },
    Scenario {
        name: "cancel propagates down to consumers",
        source: r#"
            local p = Promise.new(function() end)
            local child = p:andThen(function() end)
            p:cancel()
            return child:getStatus()
        "#,
        expected: Expected::Exact("Cancelled"),
    },
    Scenario {
        name: "cancel propagates up when all consumers cancelled",
        source: r#"
            local p = Promise.new(function() end)
            local c1 = p:andThen(function() end)
            local c2 = p:andThen(function() end)
            c1:cancel()
            local afterFirst = p:getStatus()
            c2:cancel()
            return afterFirst .. "," .. p:getStatus()
        "#,
        expected: Expected::Exact("Pending,Cancelled"),
    },
    Scenario {
        name: "cancel is no-op on already settled promise",
        source: r#"
            local p = Promise.resolve(1)
            p:cancel()
            return p:getStatus()
        "#,
        expected: Expected::Exact("Fulfilled"),
    },
    Scenario {
        name: "cancelled promise does not fire andThen callbacks",
        source: r#"
            local called = false
            local p = Promise.new(function() end)
            p:andThen(function() called = true end)
            p:cancel()
            return called
        "#,
        expected: Expected::Exact("false"),
    },
    Scenario {
        name: "getStatus returns correct strings",
        source: r#"
            local pending = Promise.new(function() end):getStatus()
            local fulfilled = Promise.resolve(1):getStatus()
            local rejected = Promise.reject("e"):getStatus()
            local cancelled = Promise.new(function() end)
            cancelled:cancel()
            return pending .. "," .. fulfilled .. "," .. rejected .. "," .. cancelled:getStatus()
        "#,
        expected: Expected::Exact("Pending,Fulfilled,Rejected,Cancelled"),
    },
    Scenario {
        name: "await on already-rejected promise returns false and error",
        source: r#"
            local gotOk = nil
            local gotErr = ""
            async(function()
                local ok, err = await(Promise.reject("already_rejected"))
                gotOk, gotErr = ok, err
            end)
            return tostring(gotOk) .. "," .. gotErr
        "#,
        expected: Expected::Exact("false,already_rejected"),
    },
    Scenario {
        name: "Promise.all cancels remaining on rejection",
        source: r#"
            local hookFired = false
            local p1 = Promise.new(function(resolve, reject, onCancel)
                onCancel(function() hookFired = true end)
            end)
            local caught = ""
            Promise.all({ p1, Promise.reject("boom") }):catch(function(err) caught = err end)
            return caught .. "," .. tostring(hookFired)
        "#,
        expected: Expected::Exact("boom,true"),
    },
    Scenario {
        name: "await success returns true and value explicitly",
        source: r#"
            local gotOk, gotVal
            async(function()
                local ok, value = await(Promise.resolve(42))
                gotOk, gotVal = ok, value
            end)
            return tostring(gotOk) .. "," .. tostring(gotVal)
        "#,
        expected: Expected::Exact("true,42"),
    },
    Scenario {
        name: "await mixed success and failure in sequence",
        source: r#"
            local results = {}
            async(function()
                local ok1, v1 = await(Promise.resolve("good"))
                table.insert(results, tostring(ok1) .. ":" .. tostring(v1))
                local ok2, v2 = await(Promise.reject("bad"))
                table.insert(results, tostring(ok2) .. ":" .. tostring(v2))
                local ok3, v3 = await(Promise.resolve("recovered"))
                table.insert(results, tostring(ok3) .. ":" .. tostring(v3))
            end)
            return table.concat(results, ",")
        "#,
        expected: Expected::Exact("true:good,false:bad,true:recovered"),
    },
    Scenario {
        name: "async function handles rejection and returns recovery value",
        source: r#"
            local final = 0
            async(function()
                local ok, err = await(Promise.reject("oops"))
                if not ok then return 999 end
                return 0
            end):andThen(function(v) final = v end)
            return final
        "#,
        expected: Expected::Exact("999"),
    },
    Scenario {
        name: "await in retry loop pattern",
        source: r#"
            local attempts = 0
            local finalVal = nil
            async(function()
                for i = 1, 3 do
                    attempts = i
                    local p = i < 3 and Promise.reject("retry") or Promise.resolve("done")
                    local ok, value = await(p)
                    if ok then finalVal = value break end
                end
            end)
            return tostring(attempts) .. "," .. tostring(finalVal)
        "#,
        expected: Expected::Exact("3,done"),
    },
    Scenario {
        name: "onCancel via instance method",
        source: r#"
            local hookFired = false
            local p = Promise.new(function() end)
            p:onCancel(function() hookFired = true end)
            p:cancel()
            return hookFired
        "#,
        expected: Expected::Exact("true"),
    },
    Scenario {
        name: "Promise.resolve flattens a fulfilled promise",
        source: r#"
            local result = 0
            Promise.resolve(Promise.resolve(42)):andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("42"),
    },
    Scenario {
        name: "Promise.resolve flattens a rejected promise",
        source: r#"
            local caught = ""
            Promise.resolve(Promise.reject("boom")):catch(function(err) caught = err end)
            return caught
        "#,
        expected: Expected::Exact("boom"),
    },
    Scenario {
        name: "andThen flattens a returned promise",
        source: r#"
            local result = 0
            Promise.resolve(10)
                :andThen(function(v) return Promise.resolve(v * 3) end)
                :andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("30"),
    },
    Scenario {
        name: "andThen flattens a returned rejected promise",
        source: r#"
            local caught = ""
            Promise.resolve(1)
                :andThen(function() return Promise.reject("handler-fail") end)
                :catch(function(err) caught = err end)
            return caught
        "#,
        expected: Expected::Exact("handler-fail"),
    },
    Scenario {
        name: "recursive flattening through multiple promise layers",
        source: r#"
            local result = 0
            Promise.resolve(Promise.resolve(Promise.resolve(99)))
                :andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("99"),
    },
    Scenario {
        name: "Promise.new resolve callback flattens a promise",
        source: r#"
            local result = 0
            Promise.new(function(resolve) resolve(Promise.resolve(77)) end)
                :andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("77"),
    },
    Scenario {
        name: "catch handler returning a promise flattens for recovery",
        source: r#"
            local result = 0
            Promise.reject("err")
                :catch(function() return Promise.resolve(55) end)
                :andThen(function(v) result = v end)
            return result
        "#,
        expected: Expected::Exact("55"),
    },
    Scenario {
        name: "flattening a pending promise adopts its eventual value",
        source: r#"
            local result = 0
            local innerResolve
            local inner = Promise.new(function(resolve) innerResolve = resolve end)
            local outer = Promise.new(function(resolve) resolve(inner) end)
            outer:andThen(function(v) result = v end)
            innerResolve(123)
            return result
        "#,
        expected: Expected::Exact("123"),
    },
    Scenario {
        name: "flattening a pending promise that rejects",
        source: r#"
            local caught = ""
            local innerReject
            local inner = Promise.new(function(resolve, reject) innerReject = reject end)
            Promise.new(function(resolve) resolve(inner) end)
                :catch(function(err) caught = err end)
            innerReject("deferred-fail")
            return caught
        "#,
        expected: Expected::Exact("deferred-fail"),
    },
    Scenario {
        name: "cancelling adopted promise propagates to inner",
        source: r#"
            local innerCancelled = false
            local inner = Promise.new(function() end)
            inner:onCancel(function() innerCancelled = true end)
            local outer = Promise.new(function(resolve) resolve(inner) end)
            outer:cancel()
            return innerCancelled
        "#,
        expected: Expected::Exact("true"),
    },
];

// These contract differentials supplement the 47 upstream TEST_CASEs. They
// exercise the two invalid coroutine-yield paths distinguished by the pinned
// C++ implementation's handleCoroutineCompletion helper.
const COROUTINE_ERROR_SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "async rejects a coroutine yield without values",
        source: r#"
            local ok, reason = pcall(function()
                async(function() coroutine.yield() end)
            end)
            local message = tostring(reason):match("async: coroutine yielded without a promise")
            return tostring(ok) .. "," .. tostring(message)
        "#,
        expected: Expected::Exact("false,async: coroutine yielded without a promise"),
    },
    Scenario {
        name: "async rejects a coroutine yield with a non-Promise",
        source: r#"
            local ok, reason = pcall(function()
                async(function() coroutine.yield("not a promise") end)
            end)
            local message = tostring(reason):match("async: await%(%) argument is not a Promise")
            return tostring(ok) .. "," .. tostring(message)
        "#,
        expected: Expected::Exact("false,async: await() argument is not a Promise"),
    },
];
