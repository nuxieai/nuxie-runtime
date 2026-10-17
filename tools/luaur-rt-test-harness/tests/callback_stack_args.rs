use luaur_rt::{
    AnyUserData, Lua, MultiValue, Result, UserData, UserDataFields, UserDataMethods, Value,
    Variadic,
};

// Callbacks convert their arguments straight from the stack and return zero or
// one value without an intermediate MultiValue. Every conversion, error and
// result must match the value-list path they replace.

struct Point {
    x: f32,
    y: f32,
}

impl UserData for Point {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.x));
        fields.add_field_method_set("x", |_, this, x: f32| {
            this.x = x;
            Ok(())
        });
        fields.add_field_function_get("kind", |_, ud: AnyUserData| Ok(ud.is::<Point>()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("sum", |_, this, ()| Ok(this.x + this.y));
        methods.add_method("scaled", |_, this, (sx, sy): (f32, f32)| {
            Ok((this.x * sx, this.y * sy))
        });
        methods.add_method_mut("moveBy", |_, this, (dx, dy): (f32, Option<f32>)| {
            this.x += dx;
            this.y += dy.unwrap_or(0.0);
            Ok(())
        });
        methods.add_method("count", |_, _, rest: Variadic<Value>| Ok(rest.len()));
        methods.add_method("all", |_, _, rest: MultiValue| Ok(rest));
        methods.add_function("make", |lua, (x, y): (f32, f32)| {
            lua.create_userdata(Point { x, y })
        });
        methods.add_meta_method("__tostring", |_, this, ()| {
            Ok(format!("Point({}, {})", this.x, this.y))
        });
    }
}

fn lua_with_point() -> Result<Lua> {
    let lua = Lua::new();
    lua.globals()
        .set("p", lua.create_userdata(Point { x: 1.0, y: 2.0 })?)?;
    Ok(lua)
}

#[test]
fn methods_convert_arguments_and_results() -> Result<()> {
    let lua = lua_with_point()?;
    assert_eq!(lua.load("return p:sum()").eval::<f32>()?, 3.0);
    // Extra arguments are ignored, as with a value list.
    assert_eq!(lua.load("return p:sum(9, 9)").eval::<f32>()?, 3.0);
    let (x, y): (f32, f32) = lua.load("return p:scaled(2, 3, 'extra')").eval()?;
    assert_eq!((x, y), (2.0, 6.0));
    lua.load("p:moveBy(1)").exec()?;
    lua.load("p:moveBy(1, 5)").exec()?;
    assert_eq!(
        lua.load("return p.x, p:sum()").eval::<(f32, f32)>()?,
        (3.0, 10.0)
    );
    // Results: zero values, one value, and a value list.
    assert_eq!(
        lua.load("return select('#', p:moveBy(0))").eval::<i64>()?,
        0
    );
    assert_eq!(lua.load("return select('#', p:sum())").eval::<i64>()?, 1);
    assert_eq!(
        lua.load("return select('#', p:scaled(1, 1))")
            .eval::<i64>()?,
        2
    );
    assert_eq!(lua.load("return p:count(1, nil, 3)").eval::<i64>()?, 3);
    assert_eq!(
        lua.load("return select('#', p:all(1, 2, nil))")
            .eval::<i64>()?,
        3
    );
    assert_eq!(
        lua.load("return tostring(p)").eval::<String>()?,
        "Point(3, 7)"
    );
    Ok(())
}

#[test]
fn missing_and_mistyped_arguments_fail_as_before() -> Result<()> {
    let lua = lua_with_point()?;
    let missing = lua.load("return p:scaled(2)").eval::<Value>().unwrap_err();
    assert!(
        missing
            .to_string()
            .contains("error converting Lua nil to f64"),
        "{missing}"
    );
    let mistyped = lua
        .load("return p:scaled(2, {})")
        .eval::<Value>()
        .unwrap_err();
    assert!(
        mistyped
            .to_string()
            .contains("error converting Lua table to f64"),
        "{mistyped}"
    );
    // A method called with the wrong (or no) self is a type mismatch.
    for source in ["return p.sum(5)", "return p.sum()", "return p.sum({})"] {
        let error = lua.load(source).eval::<Value>().unwrap_err();
        assert!(
            error.to_string().contains("userdata type mismatch"),
            "{source}: {error}"
        );
    }
    Ok(())
}

#[test]
fn fields_and_functions_read_their_arguments() -> Result<()> {
    let lua = lua_with_point()?;
    lua.load("p.x = 4").exec()?;
    assert_eq!(lua.load("return p.x").eval::<f32>()?, 4.0);
    assert!(lua.load("return p.kind").eval::<bool>()?);
    let error = lua.load("p.x = 'wide'").exec().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("error converting Lua string to f64"),
        "{error}"
    );
    let made: AnyUserData = lua.load("return p.make(5, 6)").eval()?;
    assert_eq!(made.borrow::<Point>()?.y, 6.0);
    Ok(())
}

#[test]
fn functions_read_stack_arguments_and_return_values() -> Result<()> {
    let lua = Lua::new();
    let add = lua.create_function(|_, (a, b): (i64, i64)| Ok(a + b))?;
    let echo = lua.create_function(|_, values: MultiValue| Ok(values))?;
    let first = lua.create_function(|_, value: Value| Ok(value))?;
    let none = lua.create_function(|_, ()| Ok(()))?;
    lua.globals().set("add", add)?;
    lua.globals().set("echo", echo)?;
    lua.globals().set("first", first)?;
    lua.globals().set("none", none)?;
    assert_eq!(lua.load("return add(2, 3, 4)").eval::<i64>()?, 5);
    assert_eq!(
        lua.load("return select('#', echo(1, nil, 3, nil))")
            .eval::<i64>()?,
        4
    );
    assert_eq!(lua.load("return first('a', 'b')").eval::<String>()?, "a");
    assert!(lua.load("return first()").eval::<Value>()?.is_nil());
    assert_eq!(lua.load("return select('#', none(1, 2))").eval::<i64>()?, 0);
    let error = lua.load("return add(2)").eval::<Value>().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("error converting Lua nil to i64"),
        "{error}"
    );
    Ok(())
}
