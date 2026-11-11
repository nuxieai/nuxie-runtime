//! Rive's Luau `Vector` helpers, mirroring `math/lua_vec2d.cpp`.

use crate::luau::{
    Buffer as LuaBuffer, Error, Function, Lua, Result, Table, Value, Vector as LuaVector,
};

use super::embedded_chunk::EmbeddedChunk;

const VECTOR_METHODS_KEY: &str = "nuxie.vector.methods";

/// `Vector` functions reachable with instance syntax (`value:length()`).
const VECTOR_INSTANCE_METHODS: [&str; 9] = [
    "length",
    "lengthSquared",
    "normalized",
    "distance",
    "distanceSquared",
    "dot",
    "lerp",
    "writeToBuffer",
    "writeVec4",
];

// Coarsely translated from nuxie-runtime/src/lua/math/lua_vec2d.cpp. The C++
// API is a static-method table; __call retains the constructor shape exposed
// by the initial Rust scripting seam.
pub(super) fn install_vector_global(lua: &Lua) -> Result<()> {
    let vector = lua.create_table();
    vector.set(
        "distance",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            let delta = LuaVector::new(rhs.x() - lhs.x(), rhs.y() - lhs.y(), rhs.z() - lhs.z());
            Ok(vector_dot3(delta, delta).sqrt())
        })?,
    )?;
    vector.set(
        "distanceSquared",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            let delta = LuaVector::new(rhs.x() - lhs.x(), rhs.y() - lhs.y(), rhs.z() - lhs.z());
            Ok(vector_dot3(delta, delta))
        })?,
    )?;
    vector.set(
        "dot",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| Ok(vector_dot3(lhs, rhs)))?,
    )?;
    vector.set(
        "cross",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            Ok(lhs.x().mul_add(rhs.y(), -(lhs.y() * rhs.x())))
        })?,
    )?;
    vector.set(
        "cross3",
        lua.create_function(|_, (a, b): (LuaVector, LuaVector)| {
            Ok(LuaVector::new(
                a.y().mul_add(b.z(), -(a.z() * b.y())),
                a.z().mul_add(b.x(), -(a.x() * b.z())),
                a.x().mul_add(b.y(), -(a.y() * b.x())),
            ))
        })?,
    )?;
    vector.set(
        "scaleAndAdd",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                b.x().mul_add(scale, a.x()),
                b.y().mul_add(scale, a.y()),
                b.z().mul_add(scale, a.z()),
            ))
        })?,
    )?;
    vector.set(
        "scaleAndSub",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                b.x().mul_add(-scale, a.x()),
                b.y().mul_add(-scale, a.y()),
                b.z().mul_add(-scale, a.z()),
            ))
        })?,
    )?;
    vector.set(
        "lerp",
        lua.create_function(|_, (lhs, rhs, factor): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                vector_lerp_component(lhs.x(), rhs.x(), factor),
                vector_lerp_component(lhs.y(), rhs.y(), factor),
                vector_lerp_component(lhs.z(), rhs.z(), factor),
            ))
        })?,
    )?;
    vector.set(
        "xy",
        lua.create_function(|_, (x, y): (Option<f32>, Option<f32>)| {
            Ok(LuaVector::new(x.unwrap_or(0.0), y.unwrap_or(0.0), 0.0))
        })?,
    )?;
    vector.set(
        "xyz",
        lua.create_function(|_, (x, y, z): (Option<f32>, Option<f32>, Option<f32>)| {
            Ok(LuaVector::new(
                x.unwrap_or(0.0),
                y.unwrap_or(0.0),
                z.unwrap_or(0.0),
            ))
        })?,
    )?;
    vector.set(
        "origin",
        lua.create_function(|_, ()| Ok(LuaVector::zero()))?,
    )?;
    vector.set(
        "length",
        lua.create_function(|_, value: LuaVector| Ok(vector_dot3(value, value).sqrt()))?,
    )?;
    vector.set(
        "lengthSquared",
        lua.create_function(|_, value: LuaVector| Ok(vector_dot3(value, value)))?,
    )?;
    vector.set(
        "normalized",
        lua.create_function(|_, value: LuaVector| {
            let length_squared = vector_dot3(value, value);
            let scale = if length_squared > 0.0 {
                1.0 / length_squared.sqrt()
            } else {
                1.0
            };
            Ok(LuaVector::new(
                value.x() * scale,
                value.y() * scale,
                value.z() * scale,
            ))
        })?,
    )?;
    vector.set(
        "writeToBuffer",
        lua.create_function(|_, (value, buffer, offset): (LuaVector, LuaBuffer, i64)| {
            write_vector_buffer(value, &buffer, offset, None, "writeToBuffer")
        })?,
    )?;
    vector.set(
        "writeVec4",
        lua.create_function(
            |_, (value, buffer, offset, w): (LuaVector, LuaBuffer, i64, f32)| {
                write_vector_buffer(value, &buffer, offset, Some(w), "writeVec4")
            },
        )?,
    )?;

    let metatable = lua.create_table();
    metatable.set(
        "__call",
        lua.create_function(
            |_, (_table, x, y, z): (Table, Option<f32>, Option<f32>, Option<f32>)| {
                Ok(LuaVector::new(
                    x.unwrap_or(0.0),
                    y.unwrap_or(0.0),
                    z.unwrap_or(0.0),
                ))
            },
        )?,
    )?;
    install_vm_constructors(lua, &vector, &metatable)?;
    vector.set_metatable(Some(metatable))?;

    // Rive replaces Luau's built-in vector metatable so instance syntax
    // (`value:length()`) reaches the same bindings as `Vector.length(value)`.
    // An __index callback also preserves axis and numeric component access.
    // Lua owns its method table; a Rust closure retaining a Table would keep
    // the entire VM alive through its own vector metatable.
    lua.set_named_registry_value(VECTOR_METHODS_KEY, &vector)?;
    let value_metatable = lua.create_table();
    let fallback = lua.create_function(|lua, (value, key): (LuaVector, Value)| {
            if let Some(component) = vector_component(value, &key)? {
                return Ok(Value::Number(component as f64));
            }

            if let Value::String(name) = &key {
                let name = name.to_str()?;
                if VECTOR_INSTANCE_METHODS.contains(&name.as_str()) {
                    let methods: Table = lua.named_registry_value(VECTOR_METHODS_KEY)?;
                    let method: Value = methods.get(name.as_str())?;
                    if !matches!(method, Value::Nil) {
                        return Ok(method);
                    }
                }
            }

            Err(Error::runtime(format!(
                "'{}' is not a valid index of Vector",
                vector_index_name(&key)
            )))
        })?;
    // Instance methods resolve in the VM. No method name is also a component
    // key, so trying methods first answers every key as the fallback would.
    let instance_methods = lua.create_table();
    for name in VECTOR_INSTANCE_METHODS {
        instance_methods.raw_set(name, vector.get::<Value>(name)?)?;
    }
    value_metatable.set(
        "__index",
        super::host_dispatch::methods_first_index(lua, &instance_methods, fallback)?,
    )?;
    value_metatable.set_readonly(true);
    lua.set_type_metatable::<LuaVector>(Some(value_metatable))?;

    vector.set_readonly(true);
    lua.globals().set("Vector", vector)
}

/// Replace `Vector.xy`, `Vector.xyz`, `Vector.origin` and `Vector(...)` with
/// Luau functions that build the value with the VM's native vector
/// constructor, so constructing a vector makes no host call. Any argument
/// that is not a plain number goes to the Rust version, so conversions and
/// errors are unchanged.
fn install_vm_constructors(lua: &Lua, vector: &Table, metatable: &Table) -> Result<()> {
    let Value::Table(native) = lua.globals().get::<Value>("vector")? else {
        return Ok(());
    };
    if !matches!(native.get::<Value>("create")?, Value::Function(_)) {
        return Ok(());
    }
    let make: Function = EmbeddedChunk::VectorConstructors.load(lua)?.call(())?;
    let (xy, xyz, origin, call): (Function, Function, Function, Function) = make.call((
        vector.get::<Function>("xy")?,
        vector.get::<Function>("xyz")?,
        metatable.get::<Function>("__call")?,
    ))?;
    vector.set("xy", xy)?;
    vector.set("xyz", xyz)?;
    vector.set("origin", origin)?;
    metatable.set("__call", call)
}

#[allow(dead_code)]
const VECTOR_CONSTRUCTORS_SOURCE: &str = r#"
-- The Rust constructors read integral numbers through i64, which turns -0
-- into 0. `n == 0` holds only for zeros, so this keeps every other value,
-- NaN included, exactly as the native constructor would read it.
return function(xy, xyz, call)
    local function fastXy(x, y)
        if type(x) == "number" and type(y) == "number" then
            if x == 0 then x = 0 end
            if y == 0 then y = 0 end
            return vector.create(x, y, 0)
        end
        return xy(x, y)
    end
    local function fastXyz(x, y, z)
        if type(x) == "number" and type(y) == "number" and type(z) == "number" then
            if x == 0 then x = 0 end
            if y == 0 then y = 0 end
            if z == 0 then z = 0 end
            return vector.create(x, y, z)
        end
        return xyz(x, y, z)
    end
    local function origin()
        return vector.create(0, 0, 0)
    end
    local function fastCall(self, x, y, z)
        if type(x) == "number" and type(y) == "number" and type(z) == "number" then
            if x == 0 then x = 0 end
            if y == 0 then y = 0 end
            if z == 0 then z = 0 end
            return vector.create(x, y, z)
        end
        return call(self, x, y, z)
    end
    return fastXy, fastXyz, origin, fastCall
end
"#;

#[inline]
fn vector_dot3(lhs: LuaVector, rhs: LuaVector) -> f32 {
    let xy = lhs.x().mul_add(rhs.x(), lhs.y() * rhs.y());
    lhs.z().mul_add(rhs.z(), xy)
}

#[inline]
fn vector_lerp_component(a: f32, b: f32, factor: f32) -> f32 {
    if factor == 1.0 {
        b
    } else {
        (b - a).mul_add(factor, a)
    }
}

fn write_vector_buffer(
    value: LuaVector,
    buffer: &LuaBuffer,
    offset: i64,
    w: Option<f32>,
    method: &'static str,
) -> Result<()> {
    let byte_len = if w.is_some() { 16 } else { 12 };
    let error = || Error::runtime(format!("Vector:{method} offset out of range"));
    let offset = usize::try_from(offset).map_err(|_| error())?;
    if offset > buffer.len().saturating_sub(byte_len) || buffer.len() < byte_len {
        return Err(error());
    }

    let mut bytes = [0_u8; 16];
    for (index, component) in [value.x(), value.y(), value.z()].into_iter().enumerate() {
        let start = index * 4;
        bytes[start..start + 4].copy_from_slice(&component.to_ne_bytes());
    }
    if let Some(w) = w {
        bytes[12..16].copy_from_slice(&w.to_ne_bytes());
    }
    buffer.write_bytes(offset, &bytes[..byte_len]);
    Ok(())
}

fn vector_component(value: LuaVector, key: &Value) -> Result<Option<f32>> {
    let index = match key {
        Value::String(name) => match name.to_str()?.as_str() {
            "x" | "1" => Some(0),
            "y" | "2" => Some(1),
            "z" | "3" => Some(2),
            _ => None,
        },
        Value::Integer(1) => Some(0),
        Value::Integer(2) => Some(1),
        Value::Integer(3) => Some(2),
        Value::Number(number) if *number == 1.0 => Some(0),
        Value::Number(number) if *number == 2.0 => Some(1),
        Value::Number(number) if *number == 3.0 => Some(2),
        _ => None,
    };
    Ok(index.map(|index| [value.x(), value.y(), value.z()][index]))
}

fn vector_index_name(key: &Value) -> String {
    match key {
        Value::String(value) => value
            .to_str()
            .unwrap_or_else(|_| "<non-utf8 string>".to_owned()),
        other => format!("{other:?}"),
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use crate::luau::{Lua, Value};

    use super::install_vector_global;

    /// Each call's result, or its error, rendered for comparison.
    fn observe(lua: &Lua) -> Vec<String> {
        [
            "Vector.xy(1.1, 2.5)",
            "Vector.xy(-0, 1e40)",
            "Vector.xyz(-0, -0.5, -0)",
            "Vector(-0, 2^63, -2^70)",
            "Vector.xy(0/0, 3)",
            "Vector.xy(1)",
            "Vector.xy()",
            "Vector.xy('3', 4)",
            "Vector.xy(1, 2, 3)",
            "Vector.xy(true, 1)",
            "Vector.xy({}, 1)",
            "Vector.xyz(1, 2, 3.3)",
            "Vector.xyz(1, 2)",
            "Vector.xyz(1, 2, 'x')",
            "Vector.origin()",
            "Vector.origin(5)",
            "Vector(1, 2, 3)",
            "Vector(1, 2)",
            "Vector(1, false, 3)",
            "Vector.xy(1.1, 2.5) == vector.create(1.1, 2.5, 0)",
            "type(Vector.xy(1, 2))",
            "select('#', Vector.xy(1, 2))",
            "Vector.xy(3, 4):length()",
        ]
        .iter()
        .map(|expression| {
            let source = format!(
                "local ok, a, b = pcall(function() return {expression} end) \
                 return tostring(ok) .. ' ' .. tostring(a) .. ' ' .. tostring(b)"
            );
            lua.load(source).eval::<String>().expect(expression)
        })
        .collect()
    }

    #[test]
    fn vm_constructors_match_the_rust_constructors() {
        let native = Lua::new();
        install_vector_global(&native).expect("Vector installs");

        // Without the native `vector` library the Rust constructors stay.
        let rust_only = Lua::new();
        let library = rust_only.globals().get::<Value>("vector").expect("vector library");
        rust_only.globals().set("vector", Value::Nil).expect("hide vector library");
        install_vector_global(&rust_only).expect("Vector installs");
        rust_only.globals().set("vector", library).expect("restore vector library");

        let rust_xy = rust_only.load("return Vector.xy").eval::<Value>().expect("xy");
        let native_xy = native.load("return Vector.xy").eval::<Value>().expect("xy");
        assert!(matches!(&rust_xy, Value::Function(f) if f.info().what == "C"), "{rust_xy:?}");
        assert!(matches!(&native_xy, Value::Function(f) if f.info().what == "Lua"), "{native_xy:?}");

        assert_eq!(observe(&native), observe(&rust_only));
    }
}
