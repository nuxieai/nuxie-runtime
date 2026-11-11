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
