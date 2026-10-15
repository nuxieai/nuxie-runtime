use luaur_rt::{Lua, Result, Table, Value};

// Table::get reads a table without a metatable raw; one with a metatable
// keeps the protected lua_gettable path. get_field must match get exactly.

#[test]
fn plain_table_reads_raw_values_and_missing_fields_as_nil() -> Result<()> {
    let lua = Lua::new();
    let table: Table = lua.load("return { advance = 7, [1] = 'one' }").eval()?;
    assert_eq!(table.get::<i64>("advance")?, 7);
    assert_eq!(table.get_field::<i64>("advance")?, 7);
    assert_eq!(table.get::<String>(1)?, "one");
    assert!(matches!(table.get_field::<Value>("draw")?, Value::Nil));
    assert!(matches!(table.get::<Value>("draw")?, Value::Nil));
    Ok(())
}

#[test]
fn index_metamethods_still_resolve_missing_fields() -> Result<()> {
    let lua = Lua::new();
    let table: Table = lua
        .load(
            "local class = { draw = 3 }\n\
             local reads = 0\n\
             local object = setmetatable({ own = 1 }, { __index = function(_, key)\n\
               reads = reads + 1\n\
               return class[key] or reads\n\
             end })\n\
             return object",
        )
        .eval()?;
    assert_eq!(table.get_field::<i64>("own")?, 1);
    assert_eq!(table.get_field::<i64>("draw")?, 3);
    assert_eq!(table.get::<i64>("draw")?, 3);
    // A computed fallback runs once per read.
    assert_eq!(table.get_field::<i64>("missing")?, 3);
    assert_eq!(table.get::<i64>("missing")?, 4);
    Ok(())
}

#[test]
fn a_raising_index_metamethod_is_an_error_not_a_panic() -> Result<()> {
    let lua = Lua::new();
    let table: Table = lua
        .load("return setmetatable({}, { __index = function() error('no field') end })")
        .eval()?;
    let error = table.get_field::<Value>("advance").unwrap_err().to_string();
    assert!(error.contains("no field"), "{error}");
    let error = table.get::<Value>("advance").unwrap_err().to_string();
    assert!(error.contains("no field"), "{error}");
    // The stack is balanced after the error: the table still reads normally.
    assert!(matches!(table.raw_get::<Value>("advance")?, Value::Nil));
    Ok(())
}
