// Adapted from mlua (https://github.com/mlua-rs/mlua), MIT License,
// © 2019 Aleksandr Orlenko / mlua authors. See tests/ATTRIBUTION.md.
//
// Ports mlua's `tests/scope.rs`. luaur-rt's `Lua::scope` supports:
//   - `Scope::create_function` / `create_function_mut` (non-`'static` closures),
//   - `Scope::create_userdata` (non-`'static`, `UserData`-driven userdata),
//   - `Scope::add_destructor`,
// with the scope-exit invalidation that surfaces post-scope use as
// `Error::CallbackError { cause: CallbackDestructed }` /
// `Error::CallbackError { cause: UserDataDestructed }`.
//
// DEVIATION (API shape): mlua's newer tests register userdata via a single
// `register(reg: &mut UserDataRegistry<Self>)` method. luaur-rt uses the
// established split `add_fields`/`add_methods` shape (as the rest of this test
// suite does); the registrations are otherwise identical.
//
// DEFERRED (features luaur-rt has not implemented; not part of `Scope` itself):
//   - test_scope_userdata_values   — needs `AnyUserData::call_method`,
//     `set_user_value`/`user_value`.
//   - test_scope_userdata_mismatch — needs the structured `Error::BadArgument`
//     error from userdata-method `self` conversion (luaur-rt method dispatch
//     reports `UserDataTypeMismatch` directly, not wrapped in `BadArgument`).
//   - test_scope_userdata_ref / _ref_mut — need `Scope::create_userdata_ref`
//     `_mut` and `AnyUserData::borrow_scoped`/`borrow_mut_scoped`. luaur-rt's
//     scoped userdata carries no `TypeId`, so the borrowing read-back variants
//     are not yet expressible soundly; deferred.
//   - test_scope_any_userdata / _ref / _ref_mut — need `create_any_userdata`,
//     `Scope::create_any_userdata*`, and `register_userdata_type`.
//   - test_scope_destructors — needs `lua.create_any_userdata`,
//     `AnyUserData::destroy`, and `borrow_scoped` (only `add_destructor`, which
//     this crate does implement, is exercised indirectly by the kept tests).

use std::cell::Cell;
use std::rc::Rc;

use luaur_rt::{
    Error, Function, Lua, MetaMethod, Result, UserData, UserDataFields, UserDataMethods,
};

#[test]
fn test_scope_func() -> Result<()> {
    let lua = Lua::new();

    let rc = Rc::new(Cell::new(0));
    lua.scope(|scope| {
        let rc2 = rc.clone();
        let f = scope.create_function(move |_, ()| {
            rc2.set(42);
            Ok(())
        })?;
        lua.globals().set("f", &f)?;
        f.call::<()>(())?;
        assert_eq!(Rc::strong_count(&rc), 2);
        Ok(())
    })?;
    assert_eq!(rc.get(), 42);
    assert_eq!(Rc::strong_count(&rc), 1);

    match lua.globals().get::<Function>("f")?.call::<()>(()) {
        Err(Error::CallbackError { ref cause, .. }) => match *cause.as_ref() {
            Error::CallbackDestructed => {}
            ref err => panic!("wrong error type {:?}", err),
        },
        r => panic!("improper return for destructed function: {:?}", r),
    };

    Ok(())
}

#[test]
fn test_scope_capture() -> Result<()> {
    let lua = Lua::new();

    let mut i = 0;
    lua.scope(|scope| {
        scope
            .create_function_mut(|_, ()| {
                i = 42;
                Ok(())
            })?
            .call::<()>(())
    })?;
    assert_eq!(i, 42);

    Ok(())
}

#[test]
fn test_scope_outer_lua_access() -> Result<()> {
    let lua = Lua::new();

    // DEVIATION: luaur-rt's `create_table` is infallible (no `?`); the
    // `Result`-returning `create_table_result` exists for signature parity.
    let table = lua.create_table();
    lua.scope(|scope| {
        scope
            .create_function(|_, ()| table.set("a", "b"))?
            .call::<()>(())
    })?;
    assert_eq!(table.get::<String>("a")?, "b");

    Ok(())
}

#[test]
fn test_scope_capture_scope() -> Result<()> {
    let lua = Lua::new();

    let i = Cell::new(0);
    lua.scope(|scope| {
        let f = scope.create_function(|_, ()| {
            scope.create_function(|_, n: u32| {
                i.set(i.get() + n);
                Ok(())
            })
        })?;
        f.call::<Function>(())?.call::<()>(10)?;
        Ok(())
    })?;

    assert_eq!(i.get(), 10);

    Ok(())
}

#[test]
fn test_scope_userdata_fields() -> Result<()> {
    struct MyUserData<'a>(&'a Cell<i64>);

    impl UserData for MyUserData<'_> {
        // DEVIATION: split `add_fields` instead of mlua's unified `register`.
        fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
            fields.add_field("field", "hello");
            fields.add_field_method_get("val", |_, data| Ok(data.0.get()));
            fields.add_field_method_set("val", |_, data, val| {
                data.0.set(val);
                Ok(())
            });
        }
    }

    let lua = Lua::new();

    let i = Cell::new(42);
    let f: Function = lua
        .load(
            r#"
            function(u)
                assert(u.field == "hello")
                assert(u.val == 42)
                u.val = 44
            end
        "#,
        )
        .eval()?;

    lua.scope(|scope| f.call::<()>(scope.create_userdata(MyUserData(&i))?))?;

    assert_eq!(i.get(), 44);

    Ok(())
}

#[test]
fn test_scope_userdata_methods() -> Result<()> {
    struct MyUserData<'a>(&'a Cell<i64>);

    impl UserData for MyUserData<'_> {
        fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
            methods.add_method("inc", |_, data, ()| {
                data.0.set(data.0.get() + 1);
                Ok(())
            });

            methods.add_method("dec", |_, data, ()| {
                data.0.set(data.0.get() - 1);
                Ok(())
            });
        }
    }

    let lua = Lua::new();

    let i = Cell::new(42);
    let f: Function = lua
        .load(
            r#"
            function(u)
                u:inc()
                u:inc()
                u:inc()
                u:dec()
            end
        "#,
        )
        .eval()?;

    lua.scope(|scope| f.call::<()>(scope.create_userdata(MyUserData(&i))?))?;

    assert_eq!(i.get(), 44);

    Ok(())
}

#[test]
fn test_scope_userdata_ops() -> Result<()> {
    struct MyUserData<'a>(&'a i64);

    impl UserData for MyUserData<'_> {
        fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
            methods.add_meta_method(MetaMethod::Add, |lua, this, ()| {
                let globals = lua.globals();
                globals.set("i", globals.get::<i64>("i")? + this.0)?;
                Ok(())
            });
            methods.add_meta_method(MetaMethod::Sub, |lua, this, ()| {
                let globals = lua.globals();
                globals.set("i", globals.get::<i64>("i")? + this.0)?;
                Ok(())
            });
        }
    }

    let lua = Lua::new();

    let dummy = 1;
    let f = lua
        .load(
            r#"
            i = 0
            return function(u)
                _ = u + u
                _ = u - 1
                _ = u + 1
            end
        "#,
        )
        .eval::<Function>()?;

    lua.scope(|scope| f.call::<()>(scope.create_userdata(MyUserData(&dummy))?))?;

    assert_eq!(lua.globals().get::<i64>("i")?, 3);

    Ok(())
}

#[test]
fn test_scope_userdata_drop() -> Result<()> {
    let lua = Lua::new();

    struct MyUserData<'a>(&'a Cell<i64>, #[allow(unused)] Rc<()>);

    impl UserData for MyUserData<'_> {
        fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
            methods.add_method("inc", |_, data, ()| {
                data.0.set(data.0.get() + 1);
                Ok(())
            });
        }
    }

    let (i, rc) = (Cell::new(1), Rc::new(()));
    lua.scope(|scope| {
        let ud = scope.create_userdata(MyUserData(&i, rc.clone()))?;
        lua.globals().set("ud", ud)?;
        lua.load("ud:inc()").exec()?;
        assert_eq!(Rc::strong_count(&rc), 2);
        Ok(())
    })?;
    assert_eq!(Rc::strong_count(&rc), 1);
    assert_eq!(i.get(), 2);

    match lua.load("ud:inc()").exec() {
        Err(Error::CallbackError { ref cause, .. }) => match cause.as_ref() {
            Error::UserDataDestructed => {}
            err => panic!("expected UserDataDestructed, got {err:?}"),
        },
        r => panic!("improper return for destructed userdata: {r:?}"),
    };

    Ok(())
}
