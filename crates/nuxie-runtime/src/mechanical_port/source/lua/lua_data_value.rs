use crate::mechanical_port::source::{
    data_bind::data_values::{
        DataValue, DataValueBoolean, DataValueColor, DataValueNumber, DataValueString,
    },
    lua::rive_lua_libs::*,
};
impl Drop for ScriptedDataValue {
    fn drop(&mut self) {
        self.data_value.take();
    }
}
fn push_field(s: &mut LuaState, value: &ScriptedDataValue, atom: LuaAtoms) -> bool {
    match (atom, value.data_value.as_ref().unwrap()) {
        (LuaAtoms::Value, DataValue::Number(v)) => s.push_number(v.value() as f64),
        (LuaAtoms::Value, DataValue::String(v)) => s.push_string(v.value()),
        (LuaAtoms::Value, DataValue::Boolean(v)) => s.push_boolean(v.value()),
        (LuaAtoms::Value, DataValue::Color(v)) => s.push_integer(v.value() as i64),
        (LuaAtoms::Red, DataValue::Color(v)) => s.push_integer(v.red() as i64),
        (LuaAtoms::Green, DataValue::Color(v)) => s.push_integer(v.green() as i64),
        (LuaAtoms::Blue, DataValue::Color(v)) => s.push_integer(v.blue() as i64),
        (LuaAtoms::Alpha, DataValue::Color(v)) => s.push_integer(v.alpha() as i64),
        _ => return false,
    };
    true
}
fn index(s: &mut LuaState) -> i32 {
    let (name, atom) = s.to_string_atom(2);
    let value = s.to_rive::<ScriptedDataValue>(1);
    if push_field(s, value, atom) {
        1
    } else {
        s.error(format!(
            "'{}' is not a valid index of DataValue",
            name.unwrap_or_default()
        ))
    }
}
fn newindex(s: &mut LuaState) -> i32 {
    let (key, atom) = s.to_string_atom(2);
    if key.is_none() {
        return s.type_error(2, s.type_name(LuaType::String));
    }
    let tag = s.userdata_tag(1);
    let value = s.to_rive_mut::<ScriptedDataValue>(1);
    match (atom, value.data_value.as_mut().unwrap()) {
        (LuaAtoms::Value, DataValue::Number(v)) => v.set_value(s.check_number(3) as f32),
        (LuaAtoms::Value, DataValue::String(v)) => v.set_value(s.check_string(3)),
        (LuaAtoms::Value, DataValue::Boolean(v)) => v.set_value(s.check_boolean(3)),
        (LuaAtoms::Value, DataValue::Color(v)) => v.set_value(s.check_unsigned(3)),
        (LuaAtoms::Red, DataValue::Color(v)) => v.set_red(s.check_integer(3) as i32),
        (LuaAtoms::Green, DataValue::Color(v)) => v.set_green(s.check_integer(3) as i32),
        (LuaAtoms::Blue, DataValue::Color(v)) => v.set_blue(s.check_integer(3) as i32),
        (LuaAtoms::Alpha, DataValue::Color(v)) => v.set_alpha(s.check_integer(3) as i32),
        (LuaAtoms::Red | LuaAtoms::Green | LuaAtoms::Blue | LuaAtoms::Alpha, _) => {}
        _ => return 0,
    }
    1
}
fn number(s: &mut LuaState) -> i32 {
    s.new_rive(ScriptedDataValueNumber::new(s, 0.0));
    1
}
fn string(s: &mut LuaState) -> i32 {
    s.new_rive(ScriptedDataValueString::new(s, ""));
    1
}
fn boolean(s: &mut LuaState) -> i32 {
    s.new_rive(ScriptedDataValueBoolean::new(s, false));
    1
}
fn color(s: &mut LuaState) -> i32 {
    s.new_rive(ScriptedDataValueColor::new(s, 0));
    1
}
fn namecall(s: &mut LuaState) -> i32 {
    let (name, atom) = s.namecall_atom();
    let v = s.to_rive::<ScriptedDataValue>(1);
    let result = match atom {
        LuaAtoms::IsNumber => v.is_number(),
        LuaAtoms::IsString => v.is_string(),
        LuaAtoms::IsBoolean => v.is_boolean(),
        LuaAtoms::IsColor => v.is_color(),
        _ => {
            return s.error(format!(
                "{} is not a valid method of {}",
                name.unwrap_or_default(),
                ScriptedPropertyViewModel::LUA_NAME
            ));
        }
    };
    s.push_boolean(result);
    1
}
fn register<T: LuaRiveDataValue>(s: &mut LuaState) {
    s.register_rive::<T>();
    for (name, f) in [
        ("__index", index as LuaFunction),
        ("__newindex", newindex),
        ("__namecall", namecall),
    ] {
        s.push_function(f);
        s.set_field(-2, name);
    }
    s.set_readonly(-1, true);
    s.pop(1);
}
pub fn luaopen_rive_data_values(s: &mut LuaState) -> i32 {
    s.register(
        ScriptedDataValue::LUA_NAME,
        &[
            LuaReg::new("number", number),
            LuaReg::new("string", string),
            LuaReg::new("boolean", boolean),
            LuaReg::new("color", color),
            LuaReg::END,
        ],
    );
    register::<ScriptedDataValueNumber>(s);
    s.register_data_value_direct_fields::<ScriptedDataValueNumber>(&["value"]);
    register::<ScriptedDataValueString>(s);
    register::<ScriptedDataValueBoolean>(s);
    s.register_data_value_direct_fields::<ScriptedDataValueBoolean>(&["value"]);
    register::<ScriptedDataValueColor>(s);
    s.register_data_value_direct_fields::<ScriptedDataValueColor>(&[
        "value", "red", "green", "blue", "alpha",
    ]);
    1
}
