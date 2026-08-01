//! Rive's Luau `Vector` helpers, mirroring `math/lua_vec2d.cpp`.

use luaur_rt::{Buffer as LuaBuffer, Error, Lua, Result, Table, Value, Vector as LuaVector};

// Coarsely translated from nuxie-runtime/src/lua/math/lua_vec2d.cpp. The C++
// API is a static-method table; __call retains the constructor shape exposed
// by the initial Rust scripting seam.
pub(super) fn install_vector_global(lua: &Lua) -> Result<()> {
    let vector = lua.create_table();
    vector.set(
        "distance",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            let x = lhs.x() - rhs.x();
            let y = lhs.y() - rhs.y();
            let z = lhs.z() - rhs.z();
            Ok((x * x + y * y + z * z).sqrt())
        })?,
    )?;
    vector.set(
        "distanceSquared",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            let x = lhs.x() - rhs.x();
            let y = lhs.y() - rhs.y();
            let z = lhs.z() - rhs.z();
            Ok(x * x + y * y + z * z)
        })?,
    )?;
    vector.set(
        "dot",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| Ok(vector_dot3(lhs, rhs)))?,
    )?;
    vector.set(
        "cross",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            Ok(lhs.x() * rhs.y() - lhs.y() * rhs.x())
        })?,
    )?;
    vector.set(
        "cross3",
        lua.create_function(|_, (a, b): (LuaVector, LuaVector)| {
            Ok(LuaVector::new(
                a.y() * b.z() - a.z() * b.y(),
                a.z() * b.x() - a.x() * b.z(),
                a.x() * b.y() - a.y() * b.x(),
            ))
        })?,
    )?;
    vector.set(
        "scaleAndAdd",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                a.x() + b.x() * scale,
                a.y() + b.y() * scale,
                a.z() + b.z() * scale,
            ))
        })?,
    )?;
    vector.set(
        "scaleAndSub",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                a.x() - b.x() * scale,
                a.y() - b.y() * scale,
                a.z() - b.z() * scale,
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
    vector.set_metatable(Some(metatable))?;

    // Rive replaces Luau's built-in vector metatable so instance syntax
    // (`value:length()`) reaches the same bindings as `Vector.length(value)`.
    // An __index callback also preserves axis and numeric component access.
    let methods = vector.clone();
    let value_metatable = lua.create_table();
    value_metatable.set(
        "__index",
        lua.create_function(move |_, (value, key): (LuaVector, Value)| {
            if let Some(component) = vector_component(value, &key)? {
                return Ok(Value::Number(component as f64));
            }

            if let Value::String(name) = &key {
                let name = name.to_str()?;
                if matches!(
                    name.as_str(),
                    "length"
                        | "lengthSquared"
                        | "normalized"
                        | "distance"
                        | "distanceSquared"
                        | "dot"
                        | "lerp"
                        | "writeToBuffer"
                        | "writeVec4"
                ) {
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
        })?,
    )?;
    value_metatable.set_readonly(true);
    lua.set_type_metatable::<LuaVector>(Some(value_metatable));

    vector.set_readonly(true);
    lua.globals().set("Vector", vector)
}

#[inline]
fn vector_dot3(lhs: LuaVector, rhs: LuaVector) -> f32 {
    lhs.x() * rhs.x() + lhs.y() * rhs.y() + lhs.z() * rhs.z()
}

#[inline]
fn vector_lerp_component(a: f32, b: f32, factor: f32) -> f32 {
    if factor == 1.0 {
        b
    } else {
        a + (b - a) * factor
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
