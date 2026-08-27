#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::{
    animation::listener_type::ListenerType, hit_info::HitResult, lua::rive_lua_libs::*,
    math::vec2d::Vec2D,
};
fn listener_name(raw: i32) -> &'static str {
    match ListenerType::from(raw) {
        ListenerType::Enter => "pointerEnter",
        ListenerType::Exit => "pointerExit",
        ListenerType::Down => "pointerDown",
        ListenerType::Up => "pointerUp",
        ListenerType::Move => "pointerMove",
        ListenerType::Click => "click",
        ListenerType::Drag => "pointerDrag",
        _ => "unknown",
    }
}
fn direct_id(v: &ScriptedPointerEvent, r: &mut DirectFieldResult) {
    r.set_number(v.id as f64)
}
fn direct_position(v: &ScriptedPointerEvent, r: &mut DirectFieldResult) {
    r.set_vector(v.position.x, v.position.y, 0.0, 0.0)
}
fn direct_previous(v: &ScriptedPointerEvent, r: &mut DirectFieldResult) {
    r.set_vector(v.previous_position.x, v.previous_position.y, 0.0, 0.0)
}
fn direct_time(v: &ScriptedPointerEvent, r: &mut DirectFieldResult) {
    r.set_number(v.time_stamp)
}
fn index(s: &mut LuaState) -> i32 {
    let (key, atom) = s.to_string_atom(2);
    if key.is_none() {
        return s.type_error(2, s.type_name(LuaType::String));
    }
    let event = s.to_rive::<ScriptedPointerEvent>(1);
    match atom {
        LuaAtoms::Id => s.push_unsigned(event.id as u32),
        LuaAtoms::Position => s.push_vector2(event.position.x, event.position.y),
        LuaAtoms::PreviousPosition => {
            s.push_vector2(event.previous_position.x, event.previous_position.y)
        }
        LuaAtoms::Type => s.push_string(listener_name(event.hit_listener_type)),
        LuaAtoms::TimeStamp => s.push_number(event.time_stamp),
        _ => {
            return s.error(format!(
                "{} is not a valid field of {}",
                s.check_string(1),
                ScriptedPointerEvent::LUA_NAME
            ));
        }
    }
    1
}
fn hit(s: &mut LuaState) -> i32 {
    let result = if s.is_boolean(2) && s.to_boolean(2) {
        HitResult::Hit
    } else {
        HitResult::HitOpaque
    };
    s.to_rive_mut::<ScriptedPointerEvent>(1).hit_result = result;
    0
}
fn namecall(s: &mut LuaState) -> i32 {
    let (_, atom) = s.namecall_atom();
    if atom == LuaAtoms::Hit {
        return hit(s);
    }
    s.error(format!(
        "{} is not a valid method of {}",
        s.check_string(1),
        ScriptedPointerEvent::LUA_NAME
    ))
}
fn new_event(s: &mut LuaState) -> i32 {
    let id = s.check_integer(1) as i32;
    let position = *s.check_vec2d(2);
    s.new_rive(ScriptedPointerEvent::new(
        id,
        Vec2D::new(position.x, position.y),
    ));
    1
}
pub fn luaopen_rive_input(s: &mut LuaState) -> i32 {
    s.register(
        ScriptedPointerEvent::LUA_NAME,
        &[LuaReg::new("new", new_event), LuaReg::END],
    );
    s.register_rive::<ScriptedPointerEvent>();
    s.push_function(index);
    s.set_field(-2, "__index");
    s.push_function(namecall);
    s.set_field(-2, "__namecall");
    s.set_readonly(-1, true);
    s.pop(1);
    s.register_userdata_direct_field_get::<ScriptedPointerEvent>("id", direct_id);
    s.register_userdata_direct_field_get::<ScriptedPointerEvent>("position", direct_position);
    s.register_userdata_direct_field_get::<ScriptedPointerEvent>(
        "previousPosition",
        direct_previous,
    );
    s.register_userdata_direct_field_get::<ScriptedPointerEvent>("timeStamp", direct_time);
    rive_lua_register_listener_invocation_types(s);
    1
}
