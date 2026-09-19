//! The Rust-closure-as-Lua-function trampoline.
//!
//! ## Design (see also the crate-level docs)
//!
//! A user-supplied Rust closure is type-erased to
//! [`BoxedCallback`] = `Box<dyn Fn(&Lua, MultiValue) -> Result<MultiValue>>`.
//! That `Box` is stored inside a Lua userdata created with
//! [`lua_newuserdatadtor`] — the destructor reconstitutes and drops the `Box`,
//! so the closure's captured environment is freed exactly when the GC collects
//! the function. The userdata is then captured as **upvalue 1** of a single
//! C trampoline function ([`trampoline`]) pushed via [`lua_pushcclosurek`]
//! with `nup = 1`.
//!
//! When Lua calls the function:
//!  1. The trampoline fetches upvalue 1 with [`lua_upvalueindex`] and recovers
//!     `&BoxedCallback` from the userdata pointer.
//!  2. It pops all on-stack arguments into a [`MultiValue`].
//!  3. It runs the closure inside [`catch_unwind`] for host-panic isolation
//!     on unwind builds. This is not the guest-error mechanism and cannot
//!     recover Rust panics on abort builds.
//!  4. On success it pushes the results and returns the count.
//!  5. On a returned `Err`, or a caught panic, it pushes a message string and
//!     returns [`lua_error`]'s failure to the VM's protected-call boundary.
//!     Guest errors use returned values, including on abort builds.

use std::any::TypeId;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::error::{Error, Result};
use crate::function::Function;
use crate::multi::MultiValue;
use crate::state::Lua;
use crate::sys::*;
use luaur_vm::records::lua_exception::LuaResult;

// ---------------------------------------------------------------------------
// Structured error objects (for errors that must survive the Lua boundary)
// ---------------------------------------------------------------------------
//
// Most callback errors are raised as plain Lua strings via `lua_error`, and
// `Lua::pop_error` rebuilds a flat `Error::RuntimeError`. That keeps the simple,
// message-only error path that the rest of the crate (and its tests) rely on.
//
// A small set of errors, however, carry *structured* meaning that the caller
// must be able to pattern-match after the error has travelled up through a
// `lua_pcall` boundary — specifically `CallbackDestructed` and
// `UserDataDestructed`, raised when a scope-created callback/userdata is invoked
// after its `Lua::scope` has ended. For those we push a **userdata error
// object** holding a boxed `Error` (tagged with a magic `TypeId` header), call
// `lua_error`, and recover the structured error in `pop_error`, wrapping it in
// `Error::CallbackError { cause, .. }` exactly like mlua.

/// The fixed leading layout of a wrapped-error userdata. The `TypeId` lets us
/// recognise our own error object (vs. an arbitrary userdata a script may have
/// raised) before reading the boxed `Error`.
#[repr(C)]
pub(crate) struct WrappedErrorHeader {
    pub(crate) type_id: TypeId,
}

/// The wrapped-error userdata storage: a magic `TypeId` followed by the boxed
/// structured `Error`.
#[repr(C)]
struct WrappedError {
    type_id: TypeId,
    error: Box<Error>,
}

/// A private marker type whose `TypeId` tags our wrapped-error userdata.
struct WrappedErrorTag;

/// The magic tag identifying a luaur-rt wrapped-error userdata.
pub(crate) fn wrapped_error_tag() -> TypeId {
    TypeId::of::<WrappedErrorTag>()
}

/// Destructor for the [`WrappedError`] userdata: drops the boxed `Error`.
unsafe extern "C" fn wrapped_error_dtor(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { core::ptr::drop_in_place(ptr as *mut WrappedError) };
    }
}

/// Whether the given error should be raised as a *structured* userdata error
/// object (so the caller can match on it) rather than a flat string. Only the
/// scope-destruction errors qualify; everything else keeps the string path to
/// preserve backward-compatible `RuntimeError` behavior.
pub(crate) fn is_structured(err: &Error) -> bool {
    match err {
        Error::CallbackDestructed | Error::UserDataDestructed | Error::RecursiveMutCallback => true,
        // A `CallbackError` produced when a *nested* structured error crossed a
        // `pcall` boundary is itself re-raised structured, so each boundary adds
        // one `CallbackError` wrapper layer — mirroring mlua's nested
        // `CallbackError { cause: CallbackError { cause: .. } }`.
        Error::CallbackError { cause, .. } => is_structured(cause),
        _ => false,
    }
}

/// Push a structured [`Error`] as a wrapped-error userdata error object and
/// return [`lua_error`]'s explicit VM failure.
///
/// # Safety
/// `state` must be a valid `lua_State` with at least one free stack slot.
pub(crate) unsafe fn raise_structured_error(state: *mut lua_State, err: Error) -> LuaResult<c_int> {
    unsafe {
        let storage = lua_newuserdatadtor(
            state,
            core::mem::size_of::<WrappedError>(),
            Some(wrapped_error_dtor),
        )?;
        if storage.is_null() {
            // Fall back to a string error if we cannot allocate the userdata.
            return raise_lua_error(state, &err.to_string());
        }
        core::ptr::write(
            storage as *mut WrappedError,
            WrappedError {
                type_id: wrapped_error_tag(),
                error: Box::new(err),
            },
        );
        lua_error(state)
    }
}

/// If the value at stack index `idx` is a luaur-rt wrapped-error userdata,
/// return a clone of the contained [`Error`]. Does not pop.
///
/// # Safety
/// `state` must be valid and `idx` a valid (absolute) stack index.
pub(crate) unsafe fn recover_wrapped_error(state: *mut lua_State, idx: c_int) -> Option<Error> {
    unsafe {
        if lua_type(state, idx) != ttype::USERDATA {
            return None;
        }
        let ptr = lua_touserdata(state, idx);
        if ptr.is_null() {
            return None;
        }
        // Read the header's TypeId; only our wrapped errors carry the magic tag.
        let header = &*(ptr as *const WrappedErrorHeader);
        if header.type_id != wrapped_error_tag() {
            return None;
        }
        let wrapped = &*(ptr as *const WrappedError);
        Some((*wrapped.error).clone())
    }
}

/// The type-erased boxed callback stored in the trampoline's upvalue userdata.
///
/// Under the `send` feature the boxed closure is additionally `Send` (so a
/// callback may capture `Send` data moved in from another thread and the whole
/// VM can be moved across threads). Without the feature the `+ Send` bound is
/// absent and this is byte-identical to before. See [`crate::sync::MaybeSend`].
#[cfg(feature = "send")]
pub(crate) type BoxedCallback = Box<dyn Fn(&Lua, MultiValue) -> Result<MultiValue> + Send>;

/// See the `send`-gated variant above.
#[cfg(not(feature = "send"))]
pub(crate) type BoxedCallback = Box<dyn Fn(&Lua, MultiValue) -> Result<MultiValue>>;

/// The destructor installed on the callback userdata: reconstruct the `Box`
/// inside the userdata storage and drop it (calling `Drop` on captures).
///
/// `lua_newuserdatadtor` stores the data inline; `lua_touserdata` returns a
/// pointer to that storage, which is exactly where we wrote the
/// `BoxedCallback`. We drop it in place.
unsafe extern "C" fn callback_dtor(ptr: *mut c_void) {
    if !ptr.is_null() {
        let bc = ptr as *mut BoxedCallback;
        unsafe { core::ptr::drop_in_place(bc) };
    }
}

/// The one C trampoline shared by every `create_function` closure.
unsafe fn trampoline(state: *mut lua_State) -> LuaResult<c_int> {
    unsafe {
        // 1. Recover the boxed callback from upvalue 1.
        let ud = lua_touserdata(state, lua_upvalueindex(1));
        if ud.is_null() {
            // Should be impossible; fail loudly but safely via lua_error.
            return raise_lua_error(state, "luaur-rt: missing callback upvalue");
        }
        let callback = &*(ud as *const BoxedCallback);

        // 2. Build a borrowed Lua handle for the calling thread (must NOT close it).
        let lua = Lua::from_borrowed(state);

        // 3. Pull the arguments off the stack into a MultiValue. They occupy
        //    stack indices 1..=nargs.
        let nargs = lua_gettop(state);
        let args = match collect_args(&lua, nargs) {
            Ok(a) => a,
            Err(e) => return raise_lua_error(state, &e.to_string()),
        };

        // 4. Run the user closure inside catch_unwind so a user `panic!` never
        //    becomes a nested panic when we then call lua_error.
        let outcome: std::thread::Result<Result<MultiValue>> =
            catch_unwind(AssertUnwindSafe(|| callback(&lua, args)));

        match outcome {
            Ok(Ok(results)) => {
                // 5a. Push every result and return its count. Reserve stack space
                //     first: an unchecked push of a very large result list would
                //     overflow the Lua stack and trip a fatal VM assertion
                //     (SIGTRAP) instead of erroring. Guard it and raise a
                //     catchable error if the results cannot fit (mirroring mlua).
                let n = results.len() as c_int;
                if lua_checkstack(state, n.max(1)) == 0 {
                    return raise_lua_error(state, "too many results to return to Lua");
                }
                for v in results.iter() {
                    if let Err(e) = lua.push_value(v) {
                        return raise_lua_error(state, &e.to_string());
                    }
                }
                Ok(n)
            }
            Ok(Err(err)) => {
                // 5b. The closure returned Err -> raise it as a Lua error.
                //     A translated C callback can explicitly retain
                //     `luaL_error`'s caller attribution; this path also uses
                //     the raw message rather than nesting Error's display
                //     prefix inside the Lua runtime error.
                if let Error::LuaLRuntimeError(message) = &err {
                    return raise_lua_l_runtime_error(state, message);
                }
                //     Structured errors (scope destruction) travel as a userdata
                //     error object so the caller can pattern-match on them; all
                //     others keep the flat string path.
                if is_structured(&err) {
                    raise_structured_error(state, err)
                } else {
                    raise_lua_error(state, &err.to_string())
                }
            }
            Err(panic_payload) => {
                // 5c. The closure panicked -> turn it into a catchable Lua error.
                let msg = panic_message(&panic_payload);
                raise_lua_error(state, &format!("rust panic: {msg}"))
            }
        }
    }
}

/// The actual `lua_CFunction` pointer (an `Option<unsafe fn(...)>`).
fn trampoline_ptr() -> lua_CFunction {
    Some(trampoline)
}

/// Collect stack arguments `1..=nargs` into a [`MultiValue`].
unsafe fn collect_args(lua: &Lua, nargs: c_int) -> Result<MultiValue> {
    let mut m = MultiValue::with_capacity(nargs.max(0) as usize);
    for i in 1..=nargs {
        m.push_back(lua.value_from_stack(i)?);
    }
    Ok(m)
}

/// Push `msg` as the error object and return the VM failure to the caller.
unsafe fn raise_lua_error(state: *mut lua_State, msg: &str) -> LuaResult<c_int> {
    unsafe {
        lua_pushlstring(state, msg.as_ptr() as *const c_char, msg.len())?;
        lua_error(state)
    }
}

/// Raise `msg` with the Lua caller location, matching `luaL_error(L, "%s",
/// msg)` from a translated C callback.
unsafe fn raise_lua_l_runtime_error(state: *mut lua_State, msg: &str) -> LuaResult<c_int> {
    unsafe {
        luaur_vm::functions::lua_l_error_l::lua_l_error_l(
            state,
            c"%s".as_ptr(),
            format_args!("{msg}"),
        )
    }
}

/// Best-effort extraction of a panic payload's message.
fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Build a [`Function`] from a type-erased boxed callback. Used by
/// [`Lua::create_function`] and the userdata method machinery.
pub(crate) fn create_callback_function(lua: &Lua, callback: BoxedCallback) -> Result<Function> {
    let state = lua.state();
    unsafe {
        let _stack = crate::stack_guard::StackGuard::new(state);
        // Allocate userdata sized for a BoxedCallback, with our dtor.
        let storage = lua_newuserdatadtor(
            state,
            core::mem::size_of::<BoxedCallback>(),
            Some(callback_dtor),
        )
        .map_err(|error| Error::from_vm(error))?;
        if storage.is_null() {
            return Err(Error::runtime(
                "luaur-rt: failed to allocate callback userdata",
            ));
        }
        // Move the box into the userdata storage (do NOT run its drop here).
        core::ptr::write(storage as *mut BoxedCallback, callback);

        // The userdata is now on top of the stack; capture it as upvalue 1 of
        // the trampoline closure.
        lua_pushcclosurek(
            state,
            trampoline_ptr(),
            c"luaur-rt-callback".as_ptr(),
            1, // nup: consumes the userdata above as upvalue 1
            None,
        )
        .map_err(|error| Error::from_vm(error))?;
        // The closure is now on top; take a registry ref.
        Ok(Function::from_ref(lua.try_pop_ref()?))
    }
}
