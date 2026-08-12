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
    let install = lua.load_bytecode(
        "rive_promise",
        include_bytes!(concat!(env!("OUT_DIR"), "/promise-library.luau-bytecode")),
    )?;
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
