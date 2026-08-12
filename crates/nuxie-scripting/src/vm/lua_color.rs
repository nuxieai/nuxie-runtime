//! Rive's global Color helpers, mirroring `math/lua_color.cpp`.

use luaur_rt::{Error, Lua, MultiValue, Result, Value};
use nuxie_render_api::ColorInt;

pub(super) fn install_color_global(lua: &Lua) -> Result<()> {
    let table = lua.create_table();

    for (name, shift) in [("red", 16), ("green", 8), ("blue", 0), ("alpha", 24)] {
        table.set(
            name,
            lua.create_function(move |lua, args: MultiValue| {
                let color = required_unsigned(lua, args.front(), "color")?;
                let replacement = optional_unsigned(lua, args.get(1))?;
                Ok(match replacement {
                    Some(component) => replace_color_component(color, shift, component),
                    None => color_component(color, shift),
                })
            })?,
        )?;
    }

    table.set(
        "opacity",
        lua.create_function(|lua, args: MultiValue| {
            let color = required_unsigned(lua, args.front(), "color")?;
            Ok(match optional_number(lua, args.get(1))? {
                Some(opacity) => {
                    Value::Integer(
                        replace_color_component(color, 24, opacity_to_alpha(opacity)) as i64,
                    )
                }
                None => Value::Number((color_component(color, 24) as f32 / 255.0) as f64),
            })
        })?,
    )?;

    table.set(
        "lerp",
        lua.create_function(|lua, args: MultiValue| {
            let from = required_unsigned(lua, args.front(), "from color")?;
            let to = required_unsigned(lua, args.get(1), "to color")?;
            let mix = required_number(lua, args.get(2), "mix")?;
            Ok(color_lerp(from, to, mix) as f64)
        })?,
    )?;

    table.set(
        "rgb",
        lua.create_function(|lua, args: MultiValue| {
            let red = required_unsigned(lua, args.front(), "red")?;
            let green = required_unsigned(lua, args.get(1), "green")?;
            let blue = required_unsigned(lua, args.get(2), "blue")?;
            Ok(rgba(red, green, blue, 255))
        })?,
    )?;
    table.set(
        "rgba",
        lua.create_function(|lua, args: MultiValue| {
            let red = required_unsigned(lua, args.front(), "red")?;
            let green = required_unsigned(lua, args.get(1), "green")?;
            let blue = required_unsigned(lua, args.get(2), "blue")?;
            let alpha = required_unsigned(lua, args.get(3), "alpha")?;
            Ok(rgba(red, green, blue, alpha))
        })?,
    )?;

    table.set(
        "toFloat",
        lua.create_function(|lua, args: MultiValue| {
            let color = required_unsigned(lua, args.front(), "color")?;
            let components = lua.create_table();
            components.set(1, color_component(color, 16) as f64 / 255.0)?;
            components.set(2, color_component(color, 8) as f64 / 255.0)?;
            components.set(3, color_component(color, 0) as f64 / 255.0)?;
            components.set(4, color_component(color, 24) as f64 / 255.0)?;
            Ok(components)
        })?,
    )?;

    table.set_readonly(true);
    lua.globals().set("Color", table)?;
    Ok(())
}

fn rgba(red: u32, green: u32, blue: u32, alpha: u32) -> ColorInt {
    ((alpha & 0xff) << 24) | ((red & 0xff) << 16) | ((green & 0xff) << 8) | (blue & 0xff)
}

pub(super) fn required_unsigned(lua: &Lua, value: Option<&Value>, name: &str) -> Result<u32> {
    let value = value
        .cloned()
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))?;
    lua.coerce_number(value)?
        .map(|value| (value as i64) as u32)
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))
}

fn optional_unsigned(lua: &Lua, value: Option<&Value>) -> Result<Option<u32>> {
    value
        .cloned()
        .map(|value| {
            lua.coerce_number(value)
                .map(|value| value.map(|value| (value as i64) as u32))
        })
        .transpose()
        .map(Option::flatten)
}

fn required_number(lua: &Lua, value: Option<&Value>, name: &str) -> Result<f32> {
    let value = value
        .cloned()
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))?;
    lua.coerce_number(value)?
        .map(|value| value as f32)
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))
}

fn optional_number(lua: &Lua, value: Option<&Value>) -> Result<Option<f32>> {
    value
        .cloned()
        .map(|value| {
            lua.coerce_number(value)
                .map(|value| value.map(|value| value as f32))
        })
        .transpose()
        .map(Option::flatten)
}

fn color_component(color: ColorInt, shift: u32) -> u32 {
    (color >> shift) & 0xff
}

fn replace_color_component(color: ColorInt, shift: u32, component: u32) -> ColorInt {
    (color & !(0xff << shift)) | ((component & 0xff) << shift)
}

fn opacity_to_alpha(opacity: f32) -> u32 {
    // Keep the comparison order from C++ std::min/std::max, including its
    // behavior for NaN, before applying std::lround-equivalent rounding.
    let opacity = if opacity < 1.0 { opacity } else { 1.0 };
    let opacity = if 0.0 < opacity { opacity } else { 0.0 };
    (255.0 * opacity).round() as u32
}

fn color_lerp(from: ColorInt, to: ColorInt, mix: f32) -> ColorInt {
    fn lerp_component(from: u32, to: u32, mix: f32) -> u32 {
        let value = from as f32 * (1.0 - mix) + to as f32 * mix;
        let value = if value < 255.0 { value } else { 255.0 };
        let value = if 0.0 < value { value } else { 0.0 };
        value.round() as u32
    }

    rgba(
        lerp_component(color_component(from, 16), color_component(to, 16), mix),
        lerp_component(color_component(from, 8), color_component(to, 8), mix),
        lerp_component(color_component(from, 0), color_component(to, 0), mix),
        lerp_component(color_component(from, 24), color_component(to, 24), mix),
    )
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use luaur_rt::Table;

    use super::*;

    fn color_lua() -> Lua {
        let lua = Lua::new();
        install_color_global(&lua).expect("Color global installs");
        lua
    }

    #[test]
    fn color_construction_and_component_overloads_match_cpp() {
        let lua = color_lua();
        let result: Table = lua
            .load(
                r#"
                local original = Color.rgba(225, 48, 108, 255)
                local red = Color.red(original, 129)
                local green = Color.green(original, 129)
                local blue = Color.blue(original, 129)
                local alpha = Color.alpha(original, 129)
                local wrapped = Color.red(original, -1)
                local truncated = Color.green(original, 129.9)
                return {
                    white = Color.rgba(255, 255, 255, 255),
                    yellow = Color.rgba(255, 255, 0, 255),
                    opaqueRed = Color.rgb(255, 0, 0),
                    original = original,
                    red = Color.red(original),
                    redSet = Color.red(red),
                    green = Color.green(original, nil),
                    greenSet = Color.green(green),
                    blue = Color.blue(original, false),
                    blueSet = Color.blue(blue),
                    alpha = Color.alpha(original),
                    alphaSet = Color.alpha(alpha),
                    wrapped = Color.red(wrapped),
                    truncated = Color.green(truncated),
                }
                "#,
            )
            .eval()
            .expect("Color component script runs");

        assert_eq!(result.get::<u32>("white").unwrap(), 0xffff_ffff);
        assert_eq!(result.get::<u32>("yellow").unwrap(), 0xffff_ff00);
        assert_eq!(result.get::<u32>("opaqueRed").unwrap(), 0xffff_0000);
        assert_eq!(result.get::<u32>("original").unwrap(), 0xffe1_306c);
        assert_eq!(result.get::<u32>("red").unwrap(), 225);
        assert_eq!(result.get::<u32>("redSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("green").unwrap(), 48);
        assert_eq!(result.get::<u32>("greenSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("blue").unwrap(), 108);
        assert_eq!(result.get::<u32>("blueSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("alpha").unwrap(), 255);
        assert_eq!(result.get::<u32>("alphaSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("wrapped").unwrap(), 255);
        assert_eq!(result.get::<u32>("truncated").unwrap(), 129);
    }

    #[test]
    fn color_opacity_lerp_and_float_conversion_match_cpp() {
        let lua = color_lua();
        let result: Table = lua
            .load(
                r#"
                local color = Color.rgba(225, 48, 108, 255)
                local sixtyPercent = Color.opacity(color, 0.6)
                local floats = Color.toFloat(Color.rgba(255, 128, 0, 64))
                return {
                    opaque = Color.opacity(color),
                    sixtyPercent = Color.opacity(sixtyPercent),
                    sixtyPercentAlpha = Color.alpha(sixtyPercent),
                    clampedLow = Color.alpha(Color.opacity(color, -1)),
                    clampedHigh = Color.alpha(Color.opacity(color, 2)),
                    halfway = Color.lerp(Color.rgb(0, 0, 0), Color.rgb(255, 255, 255), 0.5),
                    extrapolatedLow = Color.lerp(Color.rgb(64, 64, 64), Color.rgb(255, 255, 255), -1),
                    extrapolatedHigh = Color.lerp(Color.rgb(0, 0, 0), Color.rgb(192, 192, 192), 2),
                    floats = floats,
                }
                "#,
            )
            .eval()
            .expect("Color opacity/lerp script runs");

        assert_eq!(result.get::<f64>("opaque").unwrap(), 1.0);
        assert!((result.get::<f64>("sixtyPercent").unwrap() - 0.6).abs() < 1e-6);
        assert_eq!(result.get::<u32>("sixtyPercentAlpha").unwrap(), 153);
        assert_eq!(result.get::<u32>("clampedLow").unwrap(), 0);
        assert_eq!(result.get::<u32>("clampedHigh").unwrap(), 255);
        assert_eq!(result.get::<u32>("halfway").unwrap(), 0xff80_8080);
        assert_eq!(result.get::<u32>("extrapolatedLow").unwrap(), 0xff00_0000);
        assert_eq!(result.get::<u32>("extrapolatedHigh").unwrap(), 0xffff_ffff);

        let floats = result.get::<Table>("floats").unwrap();
        assert_eq!(floats.get::<f64>(1).unwrap(), 1.0);
        assert_eq!(floats.get::<f64>(2).unwrap(), 128.0 / 255.0);
        assert_eq!(floats.get::<f64>(3).unwrap(), 0.0);
        assert_eq!(floats.get::<f64>(4).unwrap(), 64.0 / 255.0);
    }
}
