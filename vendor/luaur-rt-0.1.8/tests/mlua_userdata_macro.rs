// Adapted from mlua (https://github.com/mlua-rs/mlua), MIT License,
// © 2019 Aleksandr Orlenko / mlua authors. See tests/ATTRIBUTION.md.
//
// Ported from mlua's `tests/userdata_macro.rs`, exercising the `macros`
// feature: the `#[derive(UserData)]` and `#[derive(FromLua)]` procedural
// derives (`crates/luaur-rt-derive`).
//
// SCOPE / DEFERRED.
//   mlua's `userdata_macro.rs` is built almost entirely on the
//   `#[mlua::userdata_impl]` *attribute* macro, which turns a whole `impl`
//   block into method/meta/field registrations through an `inventory`-based
//   `UserDataRegistry` plus `Lua::create_proxy`. luaur-rt has neither the
//   registry, the inventory collection, nor `create_proxy`, and its `UserData`
//   trait uses the `add_fields` / `add_methods` shape (mirroring mlua 0.9). So
//   only the part of mlua's derive *system* that luaur-rt-derive actually
//   implements is ported here: the `#[derive(UserData)]` **field** surface
//   (getters/setters from struct fields, with the `#[lua(skip|get|set|name)]`
//   attributes) and `#[derive(FromLua)]`. The following are intentionally NOT
//   ported (no equivalent luaur-rt API surface to swap to):
//     * `#[mlua::userdata_impl]` methods / meta-methods / static fields
//       (`area`, `scale`, `__tostring`, `__call`, `__add`, `description`,
//       `default_size`, the consuming/`&mut`-param methods, …);
//     * `Lua::create_proxy::<T>()` (the `Rectangle.new(...)` constructor proxy);
//     * `#[lua(meta, field)]` meta fields and `add_meta_field`;
//     * the `async` userdata block (the `async` feature is deferred).
//   These are noted, not silently skipped, per the porting rule.
//
// What is proven here: `#[derive(UserData)]` produces a `UserData` impl whose
// field getters/setters behave exactly like mlua's derived ones — default
// get+set, `#[lua(skip)]` hides a field, `#[lua(get, name = "...")]` makes a
// renamed read-only field — and `#[derive(FromLua)]` recovers a `Clone` value
// out of its userdata.

#![cfg(feature = "macros")]

use luaur_rt::{FromLua as _, IntoLua as _, Lua, Result, UserData};

// `#[derive(UserData)]` over named fields. Mirrors the *field* portion of
// mlua's `Rectangle`: `length`/`width` are default get+set, `version` is a
// renamed read-only getter, and `_internal` is skipped.
#[derive(Default, Clone, Debug, UserData)]
struct Rectangle {
    length: u32,

    // bare `#[lua]` == default (get + set), same as no attribute.
    #[lua]
    width: u32,

    // read-only, renamed field.
    #[lua(get, name = "version")]
    version_ro: u32,

    // not exposed to Lua at all.
    #[lua(skip)]
    _internal: u64,
}

#[test]
fn test_rectangle_fields() -> Result<()> {
    let lua = Lua::new();

    let rect = Rectangle {
        length: 5,
        width: 10,
        version_ro: 1,
        _internal: 99,
    };
    lua.globals().set("rect", lua.create_userdata(rect)?)?;

    lua.load(
        r#"
        -- default get + set fields
        assert(rect.length == 5, "length should be 5")
        assert(rect.width == 10, "width should be 10")

        -- read-only renamed field
        assert(rect.version == 1, "version should be 1")
        local ok = pcall(function() rect.version = 2 end)
        assert(not ok, "version should be read-only")

        -- skipped field is invisible
        assert(rect._internal == nil, "_internal should be nil")
        assert(rect.version_ro == nil, "the original ident is not exposed; only the renamed 'version' is")

        -- setters work and are observable
        rect.length = 15
        rect.width = 20
        assert(rect.length == 15, "length should be updated to 15")
        assert(rect.width == 20, "width should be updated to 20")
    "#,
    )
    .exec()?;

    // Rust-side: the setters mutated the stored value.
    let ud = lua.globals().get::<luaur_rt::AnyUserData>("rect")?;
    let stored = ud.borrow::<Rectangle>()?;
    assert_eq!(stored.length, 15);
    assert_eq!(stored.width, 20);
    // The skipped field keeps its original value (never touched by Lua).
    assert_eq!(stored._internal, 99);

    Ok(())
}

// A tuple struct / enum still gets a valid (no-field) `UserData` impl from the
// derive — exactly like mlua, where field exposure only applies to named-field
// structs. (mlua's `Point(i32, i32)` / `enum Color` derive their `UserData`
// here too; their *methods* come from `#[userdata_impl]`, which is deferred.)
#[derive(Clone, Debug, UserData)]
struct Point(#[allow(dead_code)] i32, #[allow(dead_code)] i32);

#[derive(Clone, Debug, UserData)]
#[allow(dead_code)]
enum Color {
    Red,
    Green,
    Blue,
}

#[test]
fn test_fieldless_userdata_derive() -> Result<()> {
    let lua = Lua::new();
    // A no-field userdata is still constructible and usable as a value; it
    // simply exposes no fields/methods.
    lua.globals().set("p", lua.create_userdata(Point(3, 4))?)?;
    lua.globals().set("c", lua.create_userdata(Color::Green)?)?;
    lua.load(
        r#"
        assert(type(p) == "userdata", "Point should be userdata")
        assert(type(c) == "userdata", "Color should be userdata")
        assert(p.x == nil, "Point exposes no fields")
    "#,
    )
    .exec()?;
    Ok(())
}

// `#[derive(FromLua)]` — faithful port of mlua's `from_lua` derive: a `Clone`
// userdata type can be recovered from a Lua `Value::UserData` of that type.
// (Mirrors the `#[derive(mlua::FromLua)]` usage in mlua's `tests/userdata.rs`.)
#[derive(Clone, Copy, Debug, PartialEq, luaur_rt::FromLua)]
struct MyValue(i32);

impl UserData for MyValue {}

#[test]
fn test_from_lua_derive() -> Result<()> {
    let lua = Lua::new();

    // Round-trip a value through Lua via userdata and recover it by FromLua.
    let ud = lua.create_userdata(MyValue(123))?;
    lua.globals().set("v", &ud)?;

    // A Rust function that takes `MyValue` by value relies on the derived
    // `FromLua` to extract it from the userdata argument.
    let f = lua.create_function(|_, v: MyValue| Ok(v.0 * 2))?;
    lua.globals().set("double", f)?;
    let doubled: i32 = lua.load("return double(v)").eval()?;
    assert_eq!(doubled, 246);

    // The derived `FromLua` also works directly off a `Value`.
    let value: luaur_rt::Value = ud.into_lua(&lua)?;
    let recovered = MyValue::from_lua(value, &lua)?;
    assert_eq!(recovered, MyValue(123));

    // Wrong-type conversion produces a `FromLuaConversionError`.
    let err = MyValue::from_lua(luaur_rt::Value::Integer(7), &lua).unwrap_err();
    assert!(
        matches!(&err, luaur_rt::Error::FromLuaConversionError { to, .. } if to == "MyValue"),
        "expected FromLuaConversionError to MyValue, got {err:?}"
    );

    Ok(())
}
