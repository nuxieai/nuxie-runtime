//! The facade's own contracts where they differ from a plain pass-through
//! to the safe VM.

use super::{Error, Function, Lua};

fn destructed_callback(lua: &Lua) -> Function {
    lua.scope(|scope| {
        let callback = scope.create_function(|_, ()| Ok(()))?;
        lua.globals().set("destructed", callback)
    })
    .unwrap();
    lua.globals().get("destructed").unwrap()
}

/// luaur-rt raises the scope-destruction errors as a userdata error object
/// and recovers them at each protected boundary as `CallbackError`, one
/// layer per boundary (vendor/luaur-rt-0.1.8/src/callback.rs).
#[test]
fn scope_destruction_errors_stay_structured_across_boundaries() {
    let lua = Lua::new();
    let destructed = destructed_callback(&lua);

    let error = destructed.call::<()>(()).unwrap_err();
    let Error::CallbackError { cause, .. } = &error else {
        panic!("a direct call keeps the cause: {error}");
    };
    assert!(matches!(cause.as_ref(), Error::CallbackDestructed));

    // A script sees an error object, not text.
    let (ok, kind): (bool, String) = lua
        .load("local ok, e = pcall(destructed); return ok, type(e)")
        .eval()
        .unwrap();
    assert!(!ok);
    assert_eq!(kind, "userdata");

    // A Rust callback that passes the error on adds a layer.
    let relay = lua
        .create_function(|lua, ()| {
            let destructed: Function = lua.globals().get("destructed")?;
            destructed.call::<()>(())
        })
        .unwrap();
    let error = relay.call::<()>(()).unwrap_err();
    let Error::CallbackError { cause, .. } = &error else {
        panic!("the relay keeps the cause: {error}");
    };
    let Error::CallbackError { cause: inner, .. } = cause.as_ref() else {
        panic!("one layer per boundary: {cause}");
    };
    assert!(matches!(inner.as_ref(), Error::CallbackDestructed));
}

/// Other callback errors stay text, raised as their `Display` (so a
/// `RuntimeError` gains a second `runtime error: ` prefix, as in luaur-rt);
/// a script's own non-string error object is still `<non-string error>`.
#[test]
fn other_errors_stay_flat() {
    let lua = Lua::new();
    let failing = lua
        .create_function(|_, ()| -> super::Result<()> { Err(Error::runtime("plain")) })
        .unwrap();
    let error = failing.call::<()>(()).unwrap_err();
    assert!(matches!(&error, Error::RuntimeError(m) if m == "runtime error: plain"), "{error}");

    let table_error = lua.load("error({})").exec().unwrap_err();
    assert!(
        matches!(&table_error, Error::RuntimeError(m) if m == "<non-string error>"),
        "{table_error}"
    );
}

/// A string error that reads like the safe VM's rendering of an error
/// object stays the string it is; only a real object becomes
/// `<non-string error>`.
#[test]
fn error_objects_are_told_from_strings_that_look_like_them() {
    let lua = Lua::new();
    let lookalike = lua
        .load("error('(error object is a table value)', 0)")
        .exec()
        .unwrap_err();
    assert!(
        matches!(&lookalike, Error::RuntimeError(m) if m == "(error object is a table value)"),
        "{lookalike}"
    );
    let boolean = lua.load("error(true)").exec().unwrap_err();
    assert!(
        matches!(&boolean, Error::RuntimeError(m) if m == "<non-string error>"),
        "{boolean}"
    );
}

/// `set_memory_limit(0)` removes the cap, as in luaur-rt; the previous cap
/// comes back as `0` when there was none.
#[test]
fn a_zero_memory_limit_means_no_limit() {
    let lua = Lua::new();
    assert_eq!(lua.set_memory_limit(0).unwrap(), 0);
    let grow = "local t = {} for i = 1, 10000 do t[i] = tostring(i) end";
    lua.load(grow).exec().unwrap();
    let cap = lua.used_memory().saturating_mul(8);
    assert_eq!(lua.set_memory_limit(cap).unwrap(), 0);
    assert_eq!(lua.set_memory_limit(0).unwrap(), cap);
    lua.load(grow).exec().unwrap();
}

/// Handles compare by identity within their VM; strings by content.
#[test]
fn handle_equality_is_per_vm_identity_and_string_content() {
    let (a, b) = (Lua::new(), Lua::new());
    let (ta, tb) = (a.create_table(), b.create_table());
    assert_ne!(ta, tb, "tables of two VMs are never equal");
    assert_eq!(ta, ta.clone());
    assert_ne!(ta, a.create_table());

    let (x, y) = (a.create_string("x"), b.create_string("x"));
    assert_eq!(x, y, "strings compare by content");
    assert_ne!(x, a.create_string("y"));
}

/// A scope that unwinds still revokes its callbacks: one that escaped
/// before the panic is `CallbackDestructed` afterwards, as in luaur-rt.
#[test]
fn scoped_callbacks_are_revoked_when_the_scope_unwinds() {
    let lua = Lua::new();
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lua.scope(|scope| -> super::Result<()> {
            let callback = scope.create_function(|_, ()| Ok(()))?;
            lua.globals().set("escaped", callback)?;
            panic!("the scope body fails after publishing a callback");
        })
    }));
    assert!(unwound.is_err());

    let escaped: Function = lua.globals().get("escaped").unwrap();
    let error = escaped.call::<()>(()).unwrap_err();
    let Error::CallbackError { cause, .. } = &error else {
        panic!("a revoked callback reports why: {error}");
    };
    assert!(matches!(cause.as_ref(), Error::CallbackDestructed));
}
