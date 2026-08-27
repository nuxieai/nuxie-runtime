#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::{
    lua::rive_lua_libs::{LuaReg, LuaState, ScriptedGradient, ScriptingContext},
    shapes::paint::color::ColorInt,
};
fn fill_stops(state: &mut LuaState) -> (Vec<f32>, Vec<ColorInt>) {
    state.check_type(3, LuaType::Table);
    let (mut stops, mut colors) = (Vec::new(), Vec::new());
    let mut index = 1;
    loop {
        if state.raw_get_i(3, index) != LuaType::Table {
            state.pop(1);
            break;
        }
        index += 1;
        state.raw_get_field(-1, "position");
        stops.push(state.check_number(-1) as f32);
        state.pop(1);
        state.raw_get_field(-1, "color");
        colors.push(state.check_unsigned(-1));
        state.pop(1);
    }
    (stops, colors)
}
fn linear(state: &mut LuaState) -> i32 {
    let from = *state.check_vec2d(1);
    let to = *state.check_vec2d(2);
    let (stops, colors) = fill_stops(state);
    let shader = state
        .thread_data::<dyn ScriptingContext>()
        .factory()
        .make_linear_gradient(from.x, from.y, to.x, to.y, &colors, &stops, stops.len());
    state.new_rive(ScriptedGradient { shader });
    1
}
fn radial(state: &mut LuaState) -> i32 {
    let from = *state.check_vec2d(1);
    let radius = state.check_number(2) as f32;
    let (stops, colors) = fill_stops(state);
    let shader = state
        .thread_data::<dyn ScriptingContext>()
        .factory()
        .make_radial_gradient(from.x, from.y, radius, &colors, &stops, stops.len());
    state.new_rive(ScriptedGradient { shader });
    1
}
const METHODS: &[LuaReg] = &[
    LuaReg::new("linear", linear),
    LuaReg::new("radial", radial),
    LuaReg::END,
];
pub fn luaopen_rive_gradient(state: &mut LuaState) -> i32 {
    state.register(ScriptedGradient::LUA_NAME, METHODS);
    state.register_rive::<ScriptedGradient>();
    1
}
