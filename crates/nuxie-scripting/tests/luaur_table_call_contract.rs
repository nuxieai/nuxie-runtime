use luaur_rt::{Function, Lua, Table};

#[test]
fn table_call_function_unit_combines_lookup_and_fixed_result_call() {
    let lua = Lua::new();
    let table: Table = lua
        .load(
            "local methods = { draw = function(self, a, b) \
                 self.sum = a + b \
             end }; \
             return setmetatable({}, { __index = methods })",
        )
        .eval()
        .unwrap();

    assert!(
        table
            .call_function_unit("draw", (table.clone(), 20, 22))
            .unwrap()
    );
    assert_eq!(table.get::<i64>("sum").unwrap(), 42);
    assert!(!table.call_function_unit("missing", ()).unwrap());

    table
        .set(
            "throwing",
            lua.load("return function() error('unit boom') end")
                .eval::<Function>()
                .unwrap(),
        )
        .unwrap();
    let error = table.call_function_unit("throwing", ()).unwrap_err();
    assert!(error.to_string().contains("unit boom"), "{error}");
}

#[test]
fn table_call_function_truthy_combines_lookup_call_and_result_coercion() {
    let lua = Lua::new();
    let table: Table = lua
        .load(
            "return { \
                 nil_value = function() return nil end, \
                 false_value = function() return false end, \
                 truthy_value = function(self, value) return value end, \
             }",
        )
        .eval()
        .unwrap();

    assert!(!table.call_function_truthy("nil_value", ()).unwrap());
    assert!(!table.call_function_truthy("false_value", ()).unwrap());
    assert!(
        table
            .call_function_truthy("truthy_value", (table.clone(), 42))
            .unwrap()
    );
    assert!(!table.call_function_truthy("missing", ()).unwrap());
}
