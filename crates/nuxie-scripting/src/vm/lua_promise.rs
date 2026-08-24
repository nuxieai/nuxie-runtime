//! Promise/A+ chaining, cancellation, and async/await interop.
//!
//! The public Promise value is userdata, matching Rive's C++ binding. The
//! mutable graph lives in Luau, however: keeping callbacks and result values in
//! Rust-owned `luaur_rt` handles would make the VM own handles back into itself
//! and prevent teardown. A weak-key state table gives Luau's collector
//! ownership of that graph, while terminal transitions eagerly sever the same
//! parent/consumer/callback links as `src/lua/lua_promise.cpp`.

use luaur_rt::{
    AnyUserData, Function, IntoLuaMulti, Lua, MultiValue, Result, Table, UserData, UserDataMethods,
    Value,
};

const PROMISE_ENGINE_REGISTRY_KEY: &str = "rive_scripting_promise_engine";

#[derive(Debug, Clone, Copy)]
struct ScriptedPromise;

unsafe fn create_async_thread(state: *mut luaur_rt::lua_State) -> core::ffi::c_int {
    luaur_vm::functions::lua_l_checktype::lua_l_checktype(
        state,
        1,
        luaur_vm::enums::lua_type::lua_Type::LUA_TFUNCTION as core::ffi::c_int,
    );

    // SAFETY: `state` is the live Luau state supplied to this C-function. The
    // new thread belongs to the same VM and is still suspended while its host
    // pointer and initial function are installed.
    unsafe {
        let thread = luaur_vm::functions::lua_newthread::lua_newthread(state);
        luaur_vm::functions::lua_setthreaddata::lua_setthreaddata(
            thread,
            luaur_vm::functions::lua_getthreaddata::lua_getthreaddata(state),
        );
        luaur_vm::functions::lua_xpush::lua_xpush(state, thread, 1);
    }
    1
}

fn dispatch(lua: &Lua, method: &str, args: MultiValue) -> Result<MultiValue> {
    let engine: Table = lua.named_registry_value(PROMISE_ENGINE_REGISTRY_KEY)?;
    let function: Function = engine.get(method)?;
    function.call(args)
}

fn call_engine<R: luaur_rt::FromLuaMulti>(
    lua: &Lua,
    method: &str,
    args: impl IntoLuaMulti,
) -> Result<R> {
    let engine: Table = lua.named_registry_value(PROMISE_ENGINE_REGISTRY_KEY)?;
    let function: Function = engine.get(method)?;
    function.call(args)
}

pub(super) fn new_pending(lua: &Lua) -> Result<AnyUserData> {
    call_engine(lua, "newPending", ())
}

pub(super) fn resolve(lua: &Lua, promise: AnyUserData, value: Value) -> Result<()> {
    call_engine(lua, "resolve", (promise, value))
}

pub(super) fn reject(lua: &Lua, promise: AnyUserData, reason: String) -> Result<()> {
    call_engine(lua, "reject", (promise, reason))
}

pub(super) fn set_on_cancel(lua: &Lua, promise: AnyUserData, callback: Function) -> Result<()> {
    call_engine(lua, "onCancel", (promise, callback))
}

impl UserData for ScriptedPromise {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("andThen", |lua, args: MultiValue| {
            dispatch(lua, "andThen", args)
        });
        methods.add_function("catch", |lua, args: MultiValue| {
            dispatch(lua, "catch", args)
        });
        methods.add_function("finally", |lua, args: MultiValue| {
            dispatch(lua, "finally", args)
        });
        methods.add_function("cancel", |lua, args: MultiValue| {
            dispatch(lua, "cancel", args)
        });
        methods.add_function("onCancel", |lua, args: MultiValue| {
            dispatch(lua, "onCancel", args)
        });
        methods.add_function("getStatus", |lua, args: MultiValue| {
            dispatch(lua, "getStatus", args)
        });
    }
}

pub(super) fn install_promise_globals(lua: &Lua) -> Result<()> {
    let new_promise = lua.create_function(|lua, ()| lua.create_userdata(ScriptedPromise))?;
    // SAFETY: `create_async_thread` follows the Luau C-function convention. It
    // validates its function argument, creates and returns one child thread,
    // and copies the invoking thread's host data before the child can resume.
    let new_async_thread = unsafe { lua.create_c_function(Some(create_async_thread))? };
    // SAFETY: this bytecode is produced by the pinned build-time compiler from
    // the embedded source below.
    let install = unsafe {
        lua.load_bytecode(
            "rive_promise",
            include_bytes!(concat!(env!("OUT_DIR"), "/promise-library.luau-bytecode")),
        )?
    };
    let install: Function = install.call(())?;
    let exports: Table = install.call((new_promise, new_async_thread))?;

    let engine: Table = exports.get("engine")?;
    lua.set_named_registry_value(PROMISE_ENGINE_REGISTRY_KEY, engine)?;
    lua.globals()
        .set("Promise", exports.get::<Table>("Promise")?)?;
    lua.globals()
        .set("await", exports.get::<Function>("await")?)?;
    lua.globals()
        .set("async", exports.get::<Function>("async")?)?;
    Ok(())
}

#[allow(dead_code)]
const PROMISE_LIBRARY: &str = r##"
return function(newPromise, newAsyncThread)
    local PENDING = "Pending"
    local FULFILLED = "Fulfilled"
    local REJECTED = "Rejected"
    local CANCELLED = "Cancelled"

    -- Promise userdata has no Rust-owned Lua handles. This ephemeron table is
    -- the sole userdata -> state association. Pending parent/consumer links
    -- intentionally retain their graph until cancellation or settlement;
    -- terminal cleanup then makes unreachable promises collectible by Luau.
    local states = setmetatable({}, { __mode = "k" })
    local engine = {}

    local function stateOf(promise)
        local state = states[promise]
        if state == nil then
            error("expected Promise", 3)
        end
        return state
    end

    local function isPromise(value)
        return states[value] ~= nil
    end

    local function newPending()
        local promise = newPromise()
        states[promise] = {
            status = PENDING,
            result = nil,
            thenCallbacks = {},
            finallyCallbacks = {},
            parent = nil,
            consumers = {},
            onCancel = nil,
        }
        return promise
    end

    local function cleanupLinks(state)
        state.parent = nil
        state.consumers = {}
        state.onCancel = nil
    end

    local resolve
    local reject
    local cancel
    local notifyCallbacks

    notifyCallbacks = function(promise)
        local state = stateOf(promise)
        if state.status == CANCELLED then
            return
        end

        local fulfilled = state.status == FULFILLED
        local result = state.result
        local thenCallbacks = state.thenCallbacks
        local finallyCallbacks = state.finallyCallbacks
        state.thenCallbacks = {}
        state.finallyCallbacks = {}

        for _, callback in ipairs(thenCallbacks) do
            local child = callback.child
            local childState = child ~= nil and states[child] or nil
            if childState ~= nil and childState.status == CANCELLED then
                child = nil
            end

            local handler = fulfilled and callback.success or callback.failure
            if handler ~= nil then
                local ok, value = pcall(handler, result)
                if child ~= nil then
                    if ok then
                        resolve(child, value)
                    else
                        reject(child, value)
                    end
                end
            elseif child ~= nil then
                if fulfilled then
                    resolve(child, result)
                else
                    reject(child, result)
                end
            end
        end

        for _, callback in ipairs(finallyCallbacks) do
            local child = callback.child
            local childState = child ~= nil and states[child] or nil
            if childState ~= nil and childState.status == CANCELLED then
                child = nil
            end

            if callback.handler ~= nil then
                local ok, err = pcall(callback.handler)
                if not ok then
                    if child ~= nil then
                        reject(child, err)
                    end
                    child = nil
                end
            end

            if child ~= nil then
                if fulfilled then
                    resolve(child, result)
                else
                    reject(child, result)
                end
            end
        end
    end

    resolve = function(promise, value)
        local state = stateOf(promise)
        if state.status ~= PENDING then
            return
        end

        if isPromise(value) and value ~= promise then
            local innerState = stateOf(value)
            if innerState.status == FULFILLED then
                resolve(promise, innerState.result)
                return
            elseif innerState.status == REJECTED then
                reject(promise, innerState.result)
                return
            elseif innerState.status == CANCELLED then
                cancel(promise)
                return
            end

            state.parent = value
            table.insert(innerState.consumers, promise)
            table.insert(innerState.thenCallbacks, { child = promise })
            return
        end

        state.status = FULFILLED
        state.result = value
        notifyCallbacks(promise)
        cleanupLinks(state)
    end

    reject = function(promise, reason)
        local state = stateOf(promise)
        if state.status ~= PENDING then
            return
        end
        state.status = REJECTED
        state.result = reason
        notifyCallbacks(promise)
        cleanupLinks(state)
    end

    cancel = function(promise)
        local state = stateOf(promise)
        if state.status ~= PENDING then
            return
        end

        state.status = CANCELLED

        if state.onCancel ~= nil then
            pcall(state.onCancel)
        end

        for _, consumer in ipairs(state.consumers) do
            local consumerState = states[consumer]
            if consumerState ~= nil and consumerState.status == PENDING then
                cancel(consumer)
            end
        end

        local parent = state.parent
        local parentState = parent ~= nil and states[parent] or nil
        if parentState ~= nil and parentState.status == PENDING then
            local allCancelled = true
            for _, consumer in ipairs(parentState.consumers) do
                local consumerState = states[consumer]
                if consumerState ~= nil and consumerState.status ~= CANCELLED then
                    allCancelled = false
                    break
                end
            end
            if allCancelled then
                cancel(parent)
            end
        end

        local thenCallbacks = state.thenCallbacks
        state.thenCallbacks = {}
        state.finallyCallbacks = {}
        for _, callback in ipairs(thenCallbacks) do
            if callback.cancel ~= nil then
                pcall(callback.cancel)
            end
        end

        cleanupLinks(state)
    end

    local function wire(source, child)
        local sourceState = stateOf(source)
        local childState = stateOf(child)
        table.insert(sourceState.consumers, child)
        childState.parent = source
    end

    engine.andThen = function(source, onFulfilled, onRejected)
        local sourceState = stateOf(source)
        local child = newPending()
        wire(source, child)
        table.insert(sourceState.thenCallbacks, {
            success = type(onFulfilled) == "function" and onFulfilled or nil,
            failure = type(onRejected) == "function" and onRejected or nil,
            child = child,
        })
        if sourceState.status ~= PENDING then
            notifyCallbacks(source)
        end
        return child
    end

    engine.catch = function(source, onRejected)
        return engine.andThen(source, nil, onRejected)
    end

    engine.finally = function(source, handler)
        local sourceState = stateOf(source)
        local child = newPending()
        wire(source, child)
        table.insert(sourceState.finallyCallbacks, {
            handler = type(handler) == "function" and handler or nil,
            child = child,
        })
        if sourceState.status ~= PENDING then
            notifyCallbacks(source)
        end
        return child
    end

    engine.cancel = function(promise)
        cancel(promise)
        return promise
    end

    engine.onCancel = function(promise, handler)
        local state = stateOf(promise)
        if type(handler) == "function" then
            state.onCancel = handler
        end
        return promise
    end

    engine.getStatus = function(promise)
        return stateOf(promise).status
    end

    -- Native producers use the same state machine without exposing these
    -- operations through the public Promise table.
    engine.newPending = newPending
    engine.resolve = resolve
    engine.reject = reject

    local statics = {}

    statics.resolve = function(...)
        local promise = newPending()
        local value = nil
        if select("#", ...) > 0 then
            value = select(1, ...)
        end
        resolve(promise, value)
        return promise
    end

    statics.reject = function(...)
        local promise = newPending()
        local reason = "rejected"
        if select("#", ...) > 0 then
            reason = select(1, ...)
        end
        reject(promise, reason)
        return promise
    end

    statics.new = function(executor)
        if type(executor) ~= "function" then
            error("Promise.new expects a function", 2)
        end

        local promise = newPending()
        local function resolveCallback(...)
            if stateOf(promise).status == PENDING then
                local value = nil
                if select("#", ...) > 0 then
                    value = select(1, ...)
                end
                resolve(promise, value)
            end
        end
        local function rejectCallback(...)
            if stateOf(promise).status == PENDING then
                local reason = "rejected"
                if select("#", ...) > 0 then
                    reason = select(1, ...)
                end
                reject(promise, reason)
            end
        end
        local function onCancelCallback(handler)
            if type(handler) == "function" then
                stateOf(promise).onCancel = handler
            end
        end

        local ok, err = pcall(executor, resolveCallback, rejectCallback, onCancelCallback)
        if not ok and stateOf(promise).status == PENDING then
            reject(promise, err)
        end
        return promise
    end

    statics.all = function(promises)
        if type(promises) ~= "table" then
            error("Promise.all expects a table", 2)
        end

        local resultPromise = newPending()
        local count = #promises
        if count == 0 then
            resolve(resultPromise, {})
            return resultPromise
        end

        local remaining = count
        local results = {}
        local done = false
        local chained = {}

        local function cancelChained()
            for _, child in ipairs(chained) do
                local childState = states[child]
                if childState ~= nil and childState.status == PENDING then
                    cancel(child)
                end
            end
        end
        stateOf(resultPromise).onCancel = cancelChained

        for index = 1, count do
            local input = promises[index]
            if not isPromise(input) then
                error("Promise.all: element " .. tostring(index) .. " is not a Promise", 2)
            end

            local inputState = stateOf(input)
            local child = newPending()
            table.insert(chained, child)
            wire(input, child)
            table.insert(inputState.thenCallbacks, {
                success = function(value)
                    if done then
                        return
                    end
                    results[index] = value
                    remaining = remaining - 1
                    if remaining == 0 then
                        done = true
                        resolve(resultPromise, results)
                    end
                end,
                failure = function(reason)
                    if done then
                        return
                    end
                    done = true
                    reject(resultPromise, reason)
                    cancelChained()
                end,
                child = child,
            })
            if inputState.status ~= PENDING then
                notifyCallbacks(input)
            end
        end

        return resultPromise
    end

    local Promise = {}
    setmetatable(Promise, {
        __index = function(_, key)
            local member = statics[key]
            if member == nil then
                error("'" .. tostring(key) .. "' is not a valid member of Promise", 2)
            end
            return member
        end,
    })

    local function awaitPromise(promise)
        local state = stateOf(promise)
        if not coroutine.isyieldable() then
            error("await() must be called inside async()", 2)
        end
        if state.status == FULFILLED then
            return true, state.result
        elseif state.status == REJECTED then
            return false, state.result
        elseif state.status == CANCELLED then
            return false, "Promise was cancelled"
        end
        return coroutine.yield(promise)
    end

    local function asyncPromise(body)
        if type(body) ~= "function" then
            error("async expects a function", 2)
        end

        local resultPromise = newPending()
        local thread = newAsyncThread(body)
        local resumeThread

        resumeThread = function(...)
            local resumed = table.pack(coroutine.resume(thread, ...))
            if not resumed[1] then
                reject(resultPromise, resumed[2])
                return
            end

            if coroutine.status(thread) == "dead" then
                local value = nil
                if resumed.n > 1 then
                    value = resumed[resumed.n]
                end
                resolve(resultPromise, value)
                return
            end

            if resumed.n < 2 then
                error("async: coroutine yielded without a promise", 2)
            end

            local awaited = resumed[2]
            if not isPromise(awaited) then
                error("async: await() argument is not a Promise", 2)
            end

            local awaitedState = stateOf(awaited)
            table.insert(awaitedState.thenCallbacks, {
                success = function(value)
                    resumeThread(true, value)
                end,
                failure = function(reason)
                    resumeThread(false, reason)
                end,
                cancel = function()
                    if coroutine.close ~= nil then
                        pcall(coroutine.close, thread)
                    end
                    if stateOf(resultPromise).status == PENDING then
                        reject(resultPromise, "Promise was cancelled")
                    end
                end,
            })
            if awaitedState.status ~= PENDING then
                notifyCallbacks(awaited)
            end
        end

        resumeThread()
        return resultPromise
    end

    return {
        Promise = Promise,
        await = awaitPromise,
        async = asyncPromise,
        engine = engine,
    }
end
"##;

#[cfg(all(test, feature = "compiler"))]
mod upstream_promise_tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn promise_lua() -> Lua {
        let lua = Lua::new();
        install_promise_globals(&lua).expect("install Promise/async/await globals");
        lua
    }

    macro_rules! number_case {
        ($name:ident, $source:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let lua = promise_lua();
                let actual: f64 = lua
                    .load($source)
                    .eval()
                    .expect("evaluate upstream Promise case");
                assert_eq!(actual, $expected);
            }
        };
    }

    macro_rules! string_case {
        ($name:ident, $source:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let lua = promise_lua();
                let actual: String = lua
                    .load($source)
                    .eval()
                    .expect("evaluate upstream Promise case");
                assert_eq!(actual, $expected);
            }
        };
    }

    macro_rules! string_contains_case {
        ($name:ident, $source:expr, [$($expected:expr),+ $(,)?]) => {
            #[test]
            fn $name() {
                let lua = promise_lua();
                let actual: String = lua.load($source).eval().expect("evaluate upstream Promise case");
                $(assert!(actual.contains($expected), "{actual:?} does not contain {:?}", $expected);)+
            }
        };
    }

    macro_rules! bool_case {
        ($name:ident, $source:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let lua = promise_lua();
                let actual: bool = lua
                    .load($source)
                    .eval()
                    .expect("evaluate upstream Promise case");
                assert_eq!(actual, $expected);
            }
        };
    }

    number_case!(
        promise_resolve_creates_a_fulfilled_promise,
        r#"
            local p = Promise.resolve(42)
            local result = 0
            p:andThen(function(v) result = v end)
            return result
        "#,
        42.0
    );
    string_case!(
        promise_reject_creates_a_rejected_promise,
        r#"
            local p = Promise.reject("oops")
            local caught = ""
            p:catch(function(err) caught = err end)
            return caught
        "#,
        "oops"
    );
    string_case!(
        promise_reject_with_no_args_uses_default_reason,
        r#"
            local p = Promise.reject()
            local caught = ""
            p:catch(function(err) caught = err end)
            return caught
        "#,
        "rejected"
    );
    number_case!(
        and_then_chains_propagate_values,
        r#"
            local result = 0
            Promise.resolve(10)
                :andThen(function(v) return v * 2 end)
                :andThen(function(v) result = v end)
            return result
        "#,
        20.0
    );
    string_case!(
        and_then_propagates_to_catch_on_rejection,
        r#"
            local caught = ""
            Promise.reject("fail")
                :andThen(function(v) return "should not happen" end)
                :catch(function(err) caught = err end)
            return caught
        "#,
        "fail"
    );
    bool_case!(
        finally_runs_on_fulfillment,
        r#"
            local ran = false
            Promise.resolve(1):finally(function() ran = true end)
            return ran
        "#,
        true
    );
    bool_case!(
        finally_runs_on_rejection,
        r#"
            local ran = false
            Promise.reject("err"):finally(function() ran = true end)
            return ran
        "#,
        true
    );
    number_case!(
        promise_all_resolves_when_all_resolve,
        r#"
            local result = 0
            Promise.all({Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)})
                :andThen(function(values) result = values[1] + values[2] + values[3] end)
            return result
        "#,
        6.0
    );
    string_case!(
        promise_all_rejects_if_any_rejects,
        r#"
            local caught = ""
            Promise.all({Promise.resolve(1), Promise.reject("boom"), Promise.resolve(3)})
                :catch(function(err) caught = err end)
            return caught
        "#,
        "boom"
    );
    number_case!(
        promise_all_with_empty_array_resolves_immediately,
        r#"
            local result = nil
            Promise.all({}):andThen(function(values) result = #values end)
            return result
        "#,
        0.0
    );
    number_case!(
        async_await_with_already_resolved_promise,
        r#"
            local result = 0
            async(function()
                local ok, v = await(Promise.resolve(99))
                result = v
            end)
            return result
        "#,
        99.0
    );

    #[test]
    fn async_coroutine_inherits_thread_data_print_works() {
        let lua = promise_lua();
        let console = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&console);
        let print = lua
            .create_function(move |_, value: String| {
                output.borrow_mut().push(value);
                Ok(())
            })
            .expect("create print capture");
        lua.globals()
            .set("print", print)
            .expect("install print capture");
        let actual: bool = lua
            .load(
                r#"
                    async(function()
                        print("first resume")
                        await(Promise.resolve(1))
                        print("post-await resume")
                    end)
                    return true
                "#,
            )
            .eval()
            .expect("evaluate upstream Promise case");
        assert!(actual);
        assert_eq!(
            console.borrow().as_slice(),
            ["first resume", "post-await resume"]
        );
    }

    number_case!(
        async_await_chains_multiple_awaits,
        r#"
            local result = 0
            async(function()
                local ok1, a = await(Promise.resolve(10))
                local ok2, b = await(Promise.resolve(20))
                result = a + b
            end)
            return result
        "#,
        30.0
    );
    string_contains_case!(
        await_returns_false_error_on_rejection_no_throw,
        r#"
            local gotOk = nil
            local gotErr = ""
            async(function()
                local ok, err = await(Promise.reject("async_error"))
                gotOk = ok
                gotErr = err
            end)
            return tostring(gotOk) .. "," .. gotErr
        "#,
        ["false", "async_error"]
    );
    string_contains_case!(
        await_rejection_resumes_coroutine_so_cleanup_code_runs,
        r#"
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
        ["true", "deferred_fail"]
    );
    string_contains_case!(
        await_on_cancelled_promise_returns_false_message,
        r#"
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
        ["false", "cancelled"]
    );
    number_case!(
        async_returns_a_promise_that_resolves_with_return_value,
        r#"
            local result = 0
            local p = async(function() return 42 end)
            p:andThen(function(v) result = v end)
            return result
        "#,
        42.0
    );
    string_contains_case!(
        and_then_handler_error_rejects_chained_promise,
        r#"
            local caught = ""
            Promise.resolve(1)
                :andThen(function(v) error("handler_error") end)
                :catch(function(err) caught = tostring(err) end)
            return caught
        "#,
        ["handler_error"]
    );
    string_contains_case!(
        finally_error_rejects_chained_promise,
        r#"
            local caught = ""
            Promise.resolve(1)
                :finally(function() error("finally_boom") end)
                :catch(function(err) caught = tostring(err) end)
            return caught
        "#,
        ["finally_boom"]
    );
    string_case!(
        cancelled_promise_does_not_fire_then_catch_finally_handlers,
        r#"
            local thenRan = false
            local catchRan = false
            local finallyRan = false
            local p = Promise.new(function(resolve, reject, onCancel)
                onCancel(function() end)
            end)
            p:andThen(function(v) thenRan = true end):catch(function(e) catchRan = true end)
            p:finally(function() finallyRan = true end)
            p:cancel()
            return tostring(thenRan) .. "," .. tostring(catchRan) .. "," .. tostring(finallyRan)
        "#,
        "false,false,false"
    );
    string_case!(
        promise_new_creates_a_pending_promise,
        r#"local p = Promise.new(function(resolve, reject, onCancel) end); return p:getStatus()"#,
        "Pending"
    );
    number_case!(
        promise_new_resolve_settles_the_promise,
        r#"
            local result = 0
            Promise.new(function(resolve, reject, onCancel) resolve(42) end)
                :andThen(function(v) result = v end)
            return result
        "#,
        42.0
    );
    string_case!(
        promise_new_reject_settles_the_promise,
        r#"
            local caught = ""
            Promise.new(function(resolve, reject, onCancel) reject("oops") end)
                :catch(function(err) caught = err end)
            return caught
        "#,
        "oops"
    );
    string_contains_case!(
        promise_new_executor_error_rejects_the_promise,
        r#"
            local caught = ""
            Promise.new(function(resolve, reject, onCancel) error("executor_boom") end)
                :catch(function(err) caught = tostring(err) end)
            return caught
        "#,
        ["executor_boom"]
    );
    string_case!(
        cancel_sets_state_to_cancelled,
        r#"local p = Promise.new(function() end); p:cancel(); return p:getStatus()"#,
        "Cancelled"
    );
    bool_case!(
        cancel_fires_on_cancel_hook,
        r#"
            local hookFired = false
            local p = Promise.new(function(resolve, reject, onCancel)
                onCancel(function() hookFired = true end)
            end)
            p:cancel()
            return hookFired
        "#,
        true
    );
    string_case!(
        cancel_propagates_down_to_consumers,
        r#"
            local p = Promise.new(function() end)
            local child = p:andThen(function() end)
            p:cancel()
            return child:getStatus()
        "#,
        "Cancelled"
    );
    string_case!(
        cancel_propagates_up_when_all_consumers_cancelled,
        r#"
            local p = Promise.new(function() end)
            local c1 = p:andThen(function() end)
            local c2 = p:andThen(function() end)
            c1:cancel()
            local afterFirst = p:getStatus()
            c2:cancel()
            return afterFirst .. "," .. p:getStatus()
        "#,
        "Pending,Cancelled"
    );
    string_case!(
        cancel_is_noop_on_already_settled_promise,
        r#"local p = Promise.resolve(1); p:cancel(); return p:getStatus()"#,
        "Fulfilled"
    );
    bool_case!(
        cancelled_promise_does_not_fire_and_then_callbacks,
        r#"
            local called = false
            local p = Promise.new(function() end)
            p:andThen(function() called = true end)
            p:cancel()
            return called
        "#,
        false
    );
    string_case!(
        get_status_returns_correct_strings,
        r#"
            local pending = Promise.new(function() end):getStatus()
            local fulfilled = Promise.resolve(1):getStatus()
            local rejected = Promise.reject("e"):getStatus()
            local cancelled = Promise.new(function() end)
            cancelled:cancel()
            return pending .. "," .. fulfilled .. "," .. rejected .. "," .. cancelled:getStatus()
        "#,
        "Pending,Fulfilled,Rejected,Cancelled"
    );
    string_contains_case!(
        await_on_already_rejected_promise_returns_false_error,
        r#"
            local gotOk = nil
            local gotErr = ""
            async(function()
                local ok, err = await(Promise.reject("already_rejected"))
                gotOk = ok
                gotErr = err
            end)
            return tostring(gotOk) .. "," .. gotErr
        "#,
        ["false", "already_rejected"]
    );
    string_case!(
        promise_all_cancels_remaining_on_rejection,
        r#"
            local hookFired = false
            local p1 = Promise.new(function(resolve, reject, onCancel)
                onCancel(function() hookFired = true end)
            end)
            local p2 = Promise.reject("boom")
            local caught = ""
            Promise.all({p1, p2}):catch(function(err) caught = err end)
            return caught .. "," .. tostring(hookFired)
        "#,
        "boom,true"
    );
    string_case!(
        await_success_returns_true_value_explicitly,
        r#"
            local gotOk = nil
            local gotVal = nil
            async(function()
                local ok, val = await(Promise.resolve(42))
                gotOk = ok
                gotVal = val
            end)
            return tostring(gotOk) .. "," .. tostring(gotVal)
        "#,
        "true,42"
    );
    string_case!(
        await_mixed_success_and_failure_in_sequence,
        r#"
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
        "true:good,false:bad,true:recovered"
    );
    number_case!(
        async_function_handles_rejection_and_returns_recovery_value,
        r#"
            local final = 0
            local p = async(function()
                local ok, err = await(Promise.reject("oops"))
                if not ok then return 999 end
                return 0
            end)
            p:andThen(function(v) final = v end)
            return final
        "#,
        999.0
    );
    string_case!(
        await_in_retry_loop_pattern,
        r#"
            local attempts = 0
            local finalVal = nil
            async(function()
                for i = 1, 3 do
                    attempts = i
                    local p
                    if i < 3 then p = Promise.reject("retry") else p = Promise.resolve("done") end
                    local ok, val = await(p)
                    if ok then finalVal = val; break end
                end
            end)
            return tostring(attempts) .. "," .. tostring(finalVal)
        "#,
        "3,done"
    );
    bool_case!(
        on_cancel_via_instance_method,
        r#"
            local hookFired = false
            local p = Promise.new(function() end)
            p:onCancel(function() hookFired = true end)
            p:cancel()
            return hookFired
        "#,
        true
    );
    number_case!(
        promise_resolve_flattens_a_fulfilled_promise,
        r#"
            local result = 0
            local inner = Promise.resolve(42)
            Promise.resolve(inner):andThen(function(v) result = v end)
            return result
        "#,
        42.0
    );
    string_case!(
        promise_resolve_flattens_a_rejected_promise,
        r#"
            local caught = ""
            local inner = Promise.reject("boom")
            Promise.resolve(inner):catch(function(err) caught = err end)
            return caught
        "#,
        "boom"
    );
    number_case!(
        and_then_flattens_a_returned_promise,
        r#"
            local result = 0
            Promise.resolve(10)
                :andThen(function(v) return Promise.resolve(v * 3) end)
                :andThen(function(v) result = v end)
            return result
        "#,
        30.0
    );
    string_case!(
        and_then_flattens_a_returned_rejected_promise,
        r#"
            local caught = ""
            Promise.resolve(1)
                :andThen(function(v) return Promise.reject("handler-fail") end)
                :catch(function(err) caught = err end)
            return caught
        "#,
        "handler-fail"
    );
    number_case!(
        recursive_flattening_through_multiple_promise_layers,
        r#"
            local result = 0
            local p = Promise.resolve(Promise.resolve(Promise.resolve(99)))
            p:andThen(function(v) result = v end)
            return result
        "#,
        99.0
    );
    number_case!(
        promise_new_resolve_callback_flattens_a_promise,
        r#"
            local result = 0
            local p = Promise.new(function(resolve) resolve(Promise.resolve(77)) end)
            p:andThen(function(v) result = v end)
            return result
        "#,
        77.0
    );
    number_case!(
        catch_handler_returning_a_promise_flattens_for_recovery,
        r#"
            local result = 0
            Promise.reject("err")
                :catch(function(e) return Promise.resolve(55) end)
                :andThen(function(v) result = v end)
            return result
        "#,
        55.0
    );
    number_case!(
        flattening_a_pending_promise_adopts_its_eventual_value,
        r#"
            local result = 0
            local innerResolve
            local inner = Promise.new(function(resolve) innerResolve = resolve end)
            local outer = Promise.new(function(resolve) resolve(inner) end)
            outer:andThen(function(v) result = v end)
            innerResolve(123)
            return result
        "#,
        123.0
    );
    string_case!(
        flattening_a_pending_promise_that_rejects,
        r#"
            local caught = ""
            local innerReject
            local inner = Promise.new(function(resolve, reject) innerReject = reject end)
            local outer = Promise.new(function(resolve) resolve(inner) end)
            outer:catch(function(err) caught = err end)
            innerReject("deferred-fail")
            return caught
        "#,
        "deferred-fail"
    );
    bool_case!(
        cancelling_adopted_promise_propagates_to_inner,
        r#"
            local innerCancelled = false
            local inner = Promise.new(function() end)
            inner:onCancel(function() innerCancelled = true end)
            local outer = Promise.new(function(resolve) resolve(inner) end)
            outer:cancel()
            return innerCancelled
        "#,
        true
    );
}
