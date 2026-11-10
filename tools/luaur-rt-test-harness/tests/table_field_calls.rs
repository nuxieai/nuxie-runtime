use luaur_rt::{Function, Lua, MultiValue, Result, Table, Value};

// Table::with_field reads a field exactly as Table::get_field does but keeps
// it on the stack; Function::call converts its results from the stack. Both
// must match the handle-and-value-list paths they replace.

#[test]
fn function_calls_convert_every_result() -> Result<()> {
    let lua = Lua::new();
    let three: Function = lua
        .load("return function(...) return 1, 'two', {3} end")
        .eval()?;
    let (one, two, table): (i64, String, Table) = three.call(())?;
    assert_eq!((one, two.as_str(), table.get::<i64>(1)?), (1, "two", 3));
    assert_eq!(three.call::<i64>(())?, 1);
    assert_eq!(three.call::<MultiValue>(())?.len(), 3);
    three.call::<()>(())?;
    let count: Function = lua
        .load("return function(...) return select('#', ...) end")
        .eval()?;
    assert_eq!(count.call::<i64>((1, Value::Nil, (2, 3)))?, 4);
    Ok(())
}

#[test]
fn field_functions_run_in_place() -> Result<()> {
    let lua = Lua::new();
    let object: Table = lua
        .load("return { scale = 2, double = function(self, x) return x * self.scale, 'extra' end }")
        .eval()?;
    let doubled = object.with_field_function("double", |double| {
        double.call::<(i64, String)>((object.clone(), 21))
    })?;
    assert_eq!(doubled.transpose()?, Some((42, "extra".to_string())));
    // A field that is not a function never reaches the callback.
    assert!(object.with_field_function("scale", |_| ())?.is_none());
    assert!(object.with_field_function("missing", |_| ())?.is_none());
    let scale = object.with_field("scale", |field| {
        assert!(field.as_function()?.is_none());
        field.to_value()
    })??;
    assert!(matches!(scale, Value::Integer(2)));
    Ok(())
}

#[test]
fn field_lookups_honor_index_metamethods_once() -> Result<()> {
    let lua = Lua::new();
    let object: Table = lua
        .load(
            "reads = 0\n\
             local class = { greet = function() return 'hi' end }\n\
             return setmetatable({}, { __index = function(_, key)\n\
               reads = reads + 1\n\
               return class[key]\n\
             end })",
        )
        .eval()?;
    let greeting = object.with_field_function("greet", |greet| greet.call::<String>(()))?;
    assert_eq!(greeting.transpose()?.as_deref(), Some("hi"));
    assert!(object.with_field_function("missing", |_| ())?.is_none());
    assert_eq!(lua.globals().get::<i64>("reads")?, 2);
    let raising: Table = lua
        .load("return setmetatable({}, { __index = function() error('no field') end })")
        .eval()?;
    let error = raising.with_field_function("draw", |_| ()).unwrap_err();
    assert!(error.to_string().contains("no field"), "{error}");
    // The stack is balanced after the error.
    assert!(matches!(raising.raw_get::<Value>("draw")?, Value::Nil));
    Ok(())
}
