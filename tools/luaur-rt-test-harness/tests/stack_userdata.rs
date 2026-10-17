use luaur_rt::{
    AnyUserData, Function, IntoLuaMulti, Lua, Result, Table, UserData, UserDataMethods, Value,
};

// A userdata a callback returns is created on the stack, and one a host
// holds for a call (Lua::with_stack_userdata) lives in a stack slot: neither
// takes a registry handle. Both must behave as handle-backed userdata do.

#[derive(Clone)]
struct Counter(i64);

impl UserData for Counter {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |_, this, ()| Ok(this.0));
        methods.add_method_mut("bump", |_, this, ()| {
            this.0 += 1;
            Ok(())
        });
        methods.add_method("next", |_, this, ()| Ok(Counter(this.0 + 1)));
    }
}

#[test]
fn returned_userdata_is_a_fresh_value() -> Result<()> {
    let lua = Lua::new();
    lua.globals().set(
        "make",
        lua.create_function(|_, start: i64| Ok(Counter(start)))?,
    )?;
    let (count, kind, value, distinct): (i64, String, i64, bool) = lua
        .load(
            "local a, b = make(3), make(3)\n\
             a:bump()\n\
             return select('#', make(1)), type(a), a:next():get(), not rawequal(a, b)",
        )
        .eval()?;
    assert_eq!(
        (count, kind.as_str(), value, distinct),
        (1, "userdata", 5, true)
    );
    // A returned userdata Lua keeps survives collection.
    lua.load("kept = { make(7), make(8) }").exec()?;
    lua.gc_collect()?;
    lua.gc_collect()?;
    assert_eq!(
        lua.load("return kept[1]:get() + kept[2]:get()")
            .eval::<i64>()?,
        15
    );
    let from_rust: AnyUserData = lua.load("return make(9)").eval()?;
    assert_eq!(from_rust.borrow::<Counter>()?.0, 9);
    Ok(())
}

#[test]
fn stack_userdata_is_an_argument_and_stays_borrowable() -> Result<()> {
    let lua = Lua::new();
    let script: Table = lua
        .load(
            "return { draw = function(self, counter)\n\
               counter:bump(); counter:bump()\n\
               self.seen = counter\n\
               return counter:get()\n\
             end, fail = function(self, counter) counter:bump(); error('boom') end }",
        )
        .eval()?;
    let (returned, after) = lua.with_stack_userdata(Counter(10), |counter| -> Result<_> {
        let returned = script
            .with_field_function("draw", |draw| {
                draw.call::<i64>((script.clone(), counter.arg()))
            })?
            .expect("draw is a function")?;
        Ok((returned, counter.borrow()?.0))
    })??;
    assert_eq!((returned, after), (12, 12));
    // Lua kept the same userdata; it outlives the slot.
    lua.gc_collect()?;
    let seen: AnyUserData = script.get("seen")?;
    assert_eq!(seen.borrow::<Counter>()?.0, 12);

    let (error, after) = lua.with_stack_userdata(Counter(0), |counter| -> Result<_> {
        let error = script
            .with_field_function("fail", |fail| {
                fail.call::<()>((script.clone(), counter.arg()))
            })?
            .expect("fail is a function")
            .unwrap_err()
            .to_string();
        Ok((error, counter.borrow()?.0))
    })??;
    assert!(error.contains("boom"), "{error}");
    assert_eq!(after, 1);
    Ok(())
}

#[test]
fn stack_arguments_cross_into_other_calls_as_values() -> Result<()> {
    let lua = Lua::new();
    let identity: Function = lua.load("return function(...) return ... end").eval()?;
    let echoed = lua.with_stack_userdata(Counter(4), |counter| -> Result<_> {
        let value: Value = identity.call(counter.arg())?;
        let multi: luaur_rt::MultiValue = identity.call((1, counter.arg(), 2))?;
        Ok((value, multi.len()))
    })??;
    let Value::UserData(ud) = echoed.0 else {
        panic!("expected userdata");
    };
    assert_eq!((ud.borrow::<Counter>()?.0, echoed.1), (4, 3));
    Ok(())
}

// Methods registered with add_method_with_args borrow userdata arguments in
// place and must fail exactly as AnyUserData arguments do.
struct Sink(i64);

impl UserData for Sink {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("take", |_, this, (a, b): (AnyUserData, AnyUserData)| {
            let mut a = a.borrow_mut::<Counter>()?;
            let b = b.borrow::<Counter>()?;
            a.0 += b.0 + this.0;
            Ok(a.0)
        });
        methods.add_method_with_args("takeInPlace", |_, this, args| {
            let (a, b) = (args.userdata(0)?, args.userdata(1)?);
            let mut a = a.borrow_mut::<Counter>()?;
            let b = b.borrow::<Counter>()?;
            a.0 += b.0 + this.0;
            Ok(a.0)
        });
        methods.add_method_with_args("count", |_, _, args| Ok(args.len()));
    }
}

#[test]
fn in_place_userdata_arguments_match_handle_arguments() -> Result<()> {
    let lua = Lua::new();
    lua.globals().set("sink", Sink(100))?;
    lua.globals().set(
        "make",
        lua.create_function(|_, start: i64| Ok(Counter(start)))?,
    )?;
    // One call site for both methods, so their messages must be identical.
    let call: Function = lua
        .load("return function(method, ...) return sink[method](sink, ...) end")
        .eval()?;
    let outcome = |method: &str, case: usize| -> Result<std::result::Result<i64, String>> {
        // Fresh arguments for each call: the success case mutates them.
        let a: AnyUserData = lua.load("return make(1)").eval()?;
        let b: AnyUserData = lua.load("return make(2)").eval()?;
        let args = match case {
            0 => (a.clone(), b).into_lua_multi(&lua)?,
            1 => (5, b).into_lua_multi(&lua)?,
            2 => (a.clone(),).into_lua_multi(&lua)?,
            3 => (a.clone(), lua.create_table()).into_lua_multi(&lua)?,
            4 => (a.clone(), lua.globals().get::<Value>("sink")?).into_lua_multi(&lua)?,
            5 => (a.clone(), a.clone()).into_lua_multi(&lua)?,
            _ => ().into_lua_multi(&lua)?,
        };
        Ok(call
            .call::<i64>((method, args))
            .map_err(|error| error.to_string()))
    };
    for case in 0..7 {
        let handle = outcome("take", case)?;
        let in_place = outcome("takeInPlace", case)?;
        assert_eq!(handle, in_place, "case {case}");
        assert_eq!(handle.is_ok(), case == 0, "case {case}: {handle:?}");
    }
    assert_eq!(lua.load("return sink:count(1, nil, 3)").eval::<i64>()?, 3);
    assert_eq!(lua.load("return sink:count()").eval::<i64>()?, 0);
    Ok(())
}

// A stack-held value names a slot of the frame it was taken in. Used from a
// nested frame (a scoped callback Lua calls), the same index names another
// value there, so the handle must refuse instead of reading it.
#[test]
fn stack_values_refuse_use_from_another_frame() -> Result<()> {
    let lua = Lua::new();
    lua.globals().set(
        "make",
        lua.create_function(|_, start: i64| Ok(Counter(start)))?,
    )?;
    let script: Table = lua
        .load("return { get = function() return 1 end }")
        .eval()?;
    let outcomes = lua.with_stack_userdata(Counter(3), |counter| -> Result<_> {
        assert_eq!(counter.borrow()?.0, 3);
        lua.scope(|scope| {
            let read = scope.create_function(|_, _: luaur_rt::MultiValue| {
                Ok(match counter.borrow() {
                    Ok(counter) => format!("read {}", counter.0),
                    Err(error) => error.to_string(),
                })
            })?;
            lua.globals().set("read", read)?;
            // With and without a same-typed userdata in the nested frame's
            // slots: neither may be mistaken for the held one.
            let plain: String = lua.load("return read()").eval()?;
            let decoy: String = lua.load("return read(make(9), make(9))").eval()?;
            let called = script.with_field_function("get", |get| {
                lua.scope(|inner| {
                    let call = inner.create_function(|_, ()| {
                        Ok(match get.call::<i64>(()) {
                            Ok(value) => format!("called {value}"),
                            Err(error) => error.to_string(),
                        })
                    })?;
                    lua.globals().set("callGet", call)?;
                    lua.load("return callGet()").eval::<String>()
                })
            })?;
            Ok((plain, decoy, called))
        })
    })??;
    let (plain, decoy, called) = outcomes;
    for outcome in [&plain, &decoy] {
        assert!(outcome.contains("outside the call frame"), "{outcome}");
    }
    let called = called.expect("get is a function")?;
    assert!(called.contains("outside the call frame"), "{called}");
    Ok(())
}
