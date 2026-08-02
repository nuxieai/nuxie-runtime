//! `DataValue` userdata and constructors corresponding to
//! `src/lua/lua_data_value.cpp`.

use luaur_rt::{Lua, Result, UserData, UserDataFields, UserDataMethods, Value};
use nuxie_runtime::{ScriptCoreString, ScriptValue};

use super::script_value_to_lua;

#[derive(Debug, Clone)]
pub(super) struct ScriptedDataValue {
    value: ScriptValue,
}

impl ScriptedDataValue {
    pub(super) fn new(value: ScriptValue) -> Self {
        Self { value }
    }

    pub(super) fn value(&self) -> &ScriptValue {
        &self.value
    }

    fn color_channel(&self, shift: u32) -> Option<u32> {
        let ScriptValue::Color(value) = self.value else {
            return None;
        };
        Some((value >> shift) & 0xff)
    }

    fn set_color_channel(&mut self, shift: u32, channel: u32) {
        if let ScriptValue::Color(value) = &mut self.value {
            let mask = !(0xff << shift);
            *value = (*value & mask) | ((channel & 0xff) << shift);
        }
    }
}

impl UserData for ScriptedDataValue {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            Ok(script_value_to_lua(lua, &this.value))
        });
        fields.add_field_method_set("value", |_, this, value: Value| {
            this.value = match (&this.value, value) {
                (ScriptValue::Number(_), Value::Integer(value)) => {
                    ScriptValue::Number(value as f64)
                }
                (ScriptValue::Number(_), Value::Number(value)) => ScriptValue::Number(value),
                (ScriptValue::String(_), Value::String(value)) => match value.to_str() {
                    Ok(value) => ScriptValue::String(value),
                    Err(_) => ScriptValue::CoreString(ScriptCoreString::from_bytes(
                        value.as_bytes().to_vec(),
                    )),
                },
                (ScriptValue::CoreString(_), Value::String(value)) => {
                    ScriptValue::CoreString(ScriptCoreString::from_bytes(
                        ScriptCoreString::from_bytes(value.as_bytes().to_vec())
                            .as_c_str_bytes()
                            .to_vec(),
                    ))
                }
                (ScriptValue::Bool(_), Value::Boolean(value)) => ScriptValue::Bool(value),
                (ScriptValue::Color(_), Value::Integer(value)) => ScriptValue::Color(value as u32),
                (ScriptValue::Color(_), Value::Number(value)) => ScriptValue::Color(value as u32),
                (expected, value) => {
                    return Err(luaur_rt::Error::runtime(format!(
                        "cannot assign Lua {} to scripted data value {expected:?}",
                        value.type_name()
                    )));
                }
            };
            Ok(())
        });
        for (name, shift) in [("red", 16), ("green", 8), ("blue", 0), ("alpha", 24)] {
            fields.add_field_method_get(name, move |_, this| Ok(this.color_channel(shift)));
            fields.add_field_method_set(name, move |_, this, value: u32| {
                this.set_color_channel(shift, value);
                Ok(())
            });
        }
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("isNumber", |_, this, ()| {
            Ok(matches!(this.value, ScriptValue::Number(_)))
        });
        methods.add_method("isString", |_, this, ()| {
            Ok(matches!(
                this.value,
                ScriptValue::String(_) | ScriptValue::CoreString(_)
            ))
        });
        methods.add_method("isBoolean", |_, this, ()| {
            Ok(matches!(this.value, ScriptValue::Bool(_)))
        });
        methods.add_method("isColor", |_, this, ()| {
            Ok(matches!(this.value, ScriptValue::Color(_)))
        });
    }
}

pub(super) fn install_data_value_global(lua: &Lua) -> Result<()> {
    let data_value = lua.create_table();
    data_value.set(
        "number",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::Number(0.0)))
        })?,
    )?;
    data_value.set(
        "string",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::String(String::new())))
        })?,
    )?;
    data_value.set(
        "boolean",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::Bool(false)))
        })?,
    )?;
    data_value.set(
        "color",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::Color(0)))
        })?,
    )?;
    data_value.set_readonly(true);
    lua.globals().set("DataValue", data_value)
}
