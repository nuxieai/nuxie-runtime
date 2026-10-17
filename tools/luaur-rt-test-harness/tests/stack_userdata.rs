use luaur_rt::{AnyUserData, Function, Lua, Result, Table, UserData, UserDataMethods, Value};

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
