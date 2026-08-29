use crate::mechanical_port::source::{
    lua::rive_lua_libs::{LuaReg, LuaState},
    shapes::paint::color::*,
};
fn rgb(s: &mut LuaState) -> i32 {
    let (r, g, b) = (
        s.check_unsigned(1),
        s.check_unsigned(2),
        s.check_unsigned(3),
    );
    s.push_unsigned(color_argb(255, r, g, b));
    1
}
fn rgba(s: &mut LuaState) -> i32 {
    let (r, g, b, a) = (
        s.check_unsigned(1),
        s.check_unsigned(2),
        s.check_unsigned(3),
        s.check_unsigned(4),
    );
    s.push_unsigned(color_argb(a, r, g, b));
    1
}
fn channel(s: &mut LuaState, get: fn(u32) -> u32, set: fn(u32, u32) -> u32) -> i32 {
    let color = s.check_unsigned(1);
    if let Some(value) = s.to_unsigned(2) {
        s.push_unsigned(set(color, value));
    } else {
        s.push_unsigned(get(color));
    }
    1
}
fn red(s: &mut LuaState) -> i32 {
    channel(s, color_red, |c, v| {
        color_argb(color_alpha(c), v, color_green(c), color_blue(c))
    })
}
fn green(s: &mut LuaState) -> i32 {
    channel(s, color_green, |c, v| {
        color_argb(color_alpha(c), color_red(c), v, color_blue(c))
    })
}
fn blue(s: &mut LuaState) -> i32 {
    channel(s, color_blue, |c, v| {
        color_argb(color_alpha(c), color_red(c), color_green(c), v)
    })
}
fn alpha(s: &mut LuaState) -> i32 {
    channel(s, color_alpha, |c, v| {
        color_argb(v, color_red(c), color_green(c), color_blue(c))
    })
}
fn opacity(s: &mut LuaState) -> i32 {
    let color = s.check_unsigned(1);
    if let Some(value) = s.to_number(2) {
        s.push_unsigned(color_argb(
            opacity_to_alpha(value as f32),
            color_red(color),
            color_green(color),
            color_blue(color),
        ));
    } else {
        s.push_number(color_opacity(color) as f64);
    }
    1
}
fn lerp(s: &mut LuaState) -> i32 {
    let value = color_lerp(
        s.check_unsigned(1),
        s.check_unsigned(2),
        s.check_number(3) as f32,
    );
    s.push_number(value as f64);
    1
}
fn to_float(s: &mut LuaState) -> i32 {
    let color = s.check_unsigned(1);
    s.create_table(4, 0);
    for (index, value) in [
        color_red(color),
        color_green(color),
        color_blue(color),
        color_alpha(color),
    ]
    .into_iter()
    .enumerate()
    {
        s.push_number(value as f64 / 255.0);
        s.raw_set_i(-2, index + 1);
    }
    1
}
const METHODS: &[LuaReg] = &[
    LuaReg::new("red", red),
    LuaReg::new("green", green),
    LuaReg::new("blue", blue),
    LuaReg::new("alpha", alpha),
    LuaReg::new("opacity", opacity),
    LuaReg::new("lerp", lerp),
    LuaReg::new("rgb", rgb),
    LuaReg::new("rgba", rgba),
    LuaReg::new("toFloat", to_float),
    LuaReg::END,
];
pub fn luaopen_rive_color(state: &mut LuaState) -> i32 {
    state.register("Color", METHODS);
    1
}
