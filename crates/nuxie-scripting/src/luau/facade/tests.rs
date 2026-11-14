//! The facade's own contracts where they differ from a plain pass-through
//! to the safe VM.

use super::{CallbackCause, Error, Function, Lua};

fn destructed_callback(lua: &Lua) -> Function {
    lua.scope(|scope| {
        let callback = scope.create_function(|_, ()| Ok(()))?;
        lua.globals().set("destructed", callback)
    })
    .unwrap();
    lua.globals().get("destructed").unwrap()
}

/// The old binding crate raises the scope-destruction errors as a userdata error
/// object and recovers them at each protected boundary as `CallbackError`. The
/// old binding crate added one layer per boundary; here a relayed callback error
/// keeps its root cause and stays one layer deep (lane E2a), with the same text.
#[test]
fn scope_destruction_errors_stay_structured_across_boundaries() {
    let lua = Lua::new();
    let destructed = destructed_callback(&lua);

    let error = destructed.call::<()>(()).unwrap_err();
    let Error::CallbackError { cause, .. } = &error else {
        panic!("a direct call keeps the cause: {error}");
    };
    assert_eq!(*cause, CallbackCause::CallbackDestructed);
    let direct = error.to_string();

    // A script sees an error object, not text.
    let (ok, kind): (bool, String) = lua
        .load("local ok, e = pcall(destructed); return ok, type(e)")
        .eval()
        .unwrap();
    assert!(!ok);
    assert_eq!(kind, "userdata");

    // A Rust callback that passes the error on adds no layer.
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
    assert_eq!(*cause, CallbackCause::CallbackDestructed);
    assert_eq!(error.to_string(), direct);
}

/// Other callback errors stay text, raised as their `Display` (so a
/// `RuntimeError` gains a second `runtime error: ` prefix, as in the old binding
/// crate); a script's own non-string error object is still `<non-string error>`.
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

/// `set_memory_limit(0)` removes the cap, as in the old binding crate; the
/// previous cap comes back as `0` when there was none.
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

/// A scope that unwinds still revokes its callbacks: one that escaped before the
/// panic is `CallbackDestructed` afterwards, as in the old binding crate.
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
    assert_eq!(*cause, CallbackCause::CallbackDestructed);
}

/// A userdata made in a callback whose payload keeps a handle (as the canvases
/// keep theirs) does not keep its VM alive: the `Lua` a callback receives does
/// not own the VM, as the old binding crate's borrowed `LuaInner` does not.
/// Dropping the owning `Lua` drops the VM and the payload with it.
#[test]
fn handles_made_in_callbacks_do_not_keep_the_vm_alive() {
    use std::cell::Cell;
    use std::rc::Rc;

    struct Holder {
        _kept: super::AnyUserData,
        _table: super::Table,
        dropped: Rc<Cell<bool>>,
    }
    impl Drop for Holder {
        fn drop(&mut self) {
            self.dropped.set(true);
        }
    }
    impl super::UserData for Holder {}
    struct Leaf;
    impl super::UserData for Leaf {}

    let dropped = Rc::new(Cell::new(false));
    let lua = Lua::new();
    let flag = Rc::clone(&dropped);
    let make = lua
        .create_function(move |lua, ()| {
            let holder = Holder {
                _kept: lua.create_userdata(Leaf)?,
                _table: lua.create_table(),
                dropped: Rc::clone(&flag),
            };
            lua.create_userdata(holder)
        })
        .unwrap();
    lua.globals().set("make", make).unwrap();
    lua.load("held = make()").exec().unwrap();
    assert!(!dropped.get());

    let weak = lua.weak();
    drop(lua);
    assert!(dropped.get(), "the VM and the payload went with the owner");
    assert!(weak.try_upgrade().is_none());
}

/// A handle that outlives its VM reports the VM dropped instead of
/// reaching a VM that is gone; one made through the owning `Lua` keeps the
/// VM, as in the old binding crate.
#[test]
fn a_handle_outliving_its_vm_reports_it() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let lua = Lua::new();
    let owned = lua.create_table();
    let escaped: Rc<RefCell<Option<super::Table>>> = Rc::default();
    let slot = Rc::clone(&escaped);
    let keep = lua
        .create_function(move |lua, ()| {
            *slot.borrow_mut() = Some(lua.create_table());
            Ok(())
        })
        .unwrap();
    keep.call::<()>(()).unwrap();
    drop((lua, keep));

    owned.set("k", 1).unwrap();
    drop(owned);
    let escaped = escaped.borrow_mut().take().unwrap();
    let error = escaped.set("k", 1).unwrap_err();
    assert!(
        matches!(&error, Error::RuntimeError(m) if m == "the Lua VM was dropped"),
        "{error}"
    );
}

/// A VM operation that unwinds puts the VM back, outside a callback and
/// inside one (where the trampoline catches the panic).
#[test]
fn a_panicking_vm_operation_puts_the_vm_back() {
    let lua = Lua::new();
    lua.globals().set("kept", 99).unwrap();

    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lua.with_vm(|_| -> super::Result<()> { panic!("an operation fails mid-way") })
    }));
    assert!(unwound.is_err());
    assert_eq!(lua.globals().get::<i64>("kept").unwrap(), 99);

    let panicking = lua
        .create_function(|lua, ()| {
            lua.with_vm(|_| -> super::Result<()> { panic!("a callback's operation fails") })
        })
        .unwrap();
    assert!(panicking.call::<()>(()).is_err());
    assert_eq!(lua.globals().get::<i64>("kept").unwrap(), 99);
}

/// A module is told to have raised an error object by the value it died
/// with: a string that reads like the facade's rendering stays the string.
#[test]
fn a_module_error_is_classified_by_its_value() {
    use std::cell::RefCell;

    let lua = Lua::new();
    let run = |source: &str| -> (Error, String) {
        let body = lua.load(source).into_function().unwrap();
        let thread = lua.create_thread(body).unwrap();
        let top = RefCell::new(String::new());
        let error = thread
            .resume_module::<super::Value, ()>("mod", || Ok(()), |thread| {
                if let Ok(super::Value::String(s)) = thread.stack_value(-1) {
                    *top.borrow_mut() = s.to_string_lossy();
                }
            })
            .unwrap_err();
        (error, top.into_inner())
    };

    let (error, top) = run("error('<non-string error>', 0)");
    assert!(
        matches!(&error, Error::RuntimeError(m) if m == "<non-string error>"),
        "{error}"
    );
    assert_eq!(top, "<non-string error>");

    let (error, top) = run("error({})");
    let unknown = "mod:1: unknown error while running module";
    assert!(matches!(&error, Error::RuntimeError(m) if m == unknown), "{error}");
    assert_eq!(top, unknown);
}

/// An error object a host handled inside `with_vm` does not classify a
/// later string error that reads like its rendering.
#[test]
fn a_handled_error_object_does_not_classify_a_later_string() {
    use super::vm;

    let lua = Lua::new();
    lua.with_vm(|raw| {
        let object = raw.create_table().map_err(Error::from_vm)?;
        let raise = raw
            .create_function("raise", move |_, (): ()| -> Result<(), vm::HostError> {
                Err(vm::HostError::from_value(vm::Value::Table(object.clone())))
            })
            .map_err(Error::from_vm)?;
        assert!(raise.call::<_, ()>(raw, ()).is_err(), "handled here");
        Ok(())
    })
    .unwrap();

    let lookalike = lua
        .load("error('(error object is a table value)', 0)")
        .exec()
        .unwrap_err();
    assert!(
        matches!(&lookalike, Error::RuntimeError(m) if m == "(error object is a table value)"),
        "{lookalike}"
    );
}

/// An ordinary failed resume takes the thread's error off its stack, as
/// the old binding crate's does; only a module error hook sees it there.
#[test]
fn an_ordinary_failed_resume_consumes_the_thread_error() {
    let lua = Lua::new();
    let body = lua.load("error('boom', 0)").into_function().unwrap();
    let thread = lua.create_thread(body).unwrap();
    assert!(thread.resume::<()>(()).is_err());
    assert_eq!(thread.status(), super::ThreadStatus::Error);
    for index in [-1, 1] {
        assert!(matches!(thread.stack_value(index), Ok(super::Value::Nil)));
    }
}

/// `try_app_data_ref` reads absent data as `None`, present data by type, and
/// refuses while the store is mutably borrowed instead of reading `None`.
#[test]
fn try_app_data_ref_reports_a_mutable_borrow() {
    let lua = Lua::new();
    assert!(lua.try_app_data_ref::<u32>().unwrap().is_none());
    lua.set_app_data(7_u32);
    assert_eq!(lua.try_app_data_ref::<u32>().unwrap().as_deref(), Some(&7));
    assert!(lua.try_app_data_ref::<u64>().unwrap().is_none());

    let held = lua.app_data_mut::<u32>().unwrap();
    let error = lua.try_app_data_ref::<u32>().unwrap_err();
    assert!(error.to_string().contains("mutably borrowed"), "{error}");
    drop(held);
    assert!(lua.try_app_data_ref::<u32>().unwrap().is_some());
}

/// A string's length is read in place: the same as its bytes, NULs included.
#[test]
fn string_length_matches_its_bytes() {
    let lua = Lua::new();
    for bytes in [&b""[..], b"abc", b"a\0b", &[0xff; 70_000]] {
        let string = lua.create_string(bytes);
        assert_eq!(string.len(), bytes.len());
        assert_eq!(string.len(), string.as_bytes().len());
        assert_eq!(string.is_empty(), bytes.is_empty());
    }
}
