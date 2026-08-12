//! `DataValue` userdata and constructors corresponding to
//! `src/lua/lua_data_value.cpp`.

use luaur_rt::{
    AnyUserData, Error, Function, Lua, LuaString, Result, UserData, UserDataFields,
    UserDataMethods, Value,
};
use nuxie_runtime::{ScriptCoreString, ScriptValue};

use super::script_value_to_lua;

const DATA_VALUE_METATABLE_PATCHER: &str = "rive_data_value_metatable_patcher";

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

    fn color_channel(&self, name: &str, shift: u32) -> Result<u32> {
        let ScriptValue::Color(value) = self.value else {
            return Err(Error::runtime(format!(
                "'{name}' is not a valid index of DataValue"
            )));
        };
        Ok((value >> shift) & 0xff)
    }

    fn set_color_channel(&mut self, shift: u32, channel: i32) {
        if let ScriptValue::Color(value) = &mut self.value {
            let mask = !(0xff_u32 << shift);
            *value = (*value & mask) | (channel.wrapping_shl(shift) as u32);
        }
    }
}

pub(super) fn checked_number(lua: &Lua, value: Value) -> Result<f64> {
    lua.coerce_number(value)?
        .ok_or_else(|| Error::runtime("expected number"))
}

fn checked_integer(lua: &Lua, value: Value) -> Result<i32> {
    checked_number(lua, value).map(|value| value as i32)
}

pub(super) fn checked_string(lua: &Lua, value: Value) -> Result<Vec<u8>> {
    let value: LuaString = lua.unpack(value)?;
    let mut bytes = value.as_bytes();
    bytes.truncate(
        bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len()),
    );
    Ok(bytes)
}

fn set_value(lua: &Lua, current: &mut ScriptValue, value: Value) -> Result<()> {
    *current = match current {
        ScriptValue::Number(_) => ScriptValue::Number(checked_number(lua, value)? as f32 as f64),
        ScriptValue::String(_) => {
            let bytes = checked_string(lua, value)?;
            match String::from_utf8(bytes.clone()) {
                Ok(value) => ScriptValue::String(value),
                Err(_) => ScriptValue::CoreString(ScriptCoreString::from_bytes(bytes)),
            }
        }
        ScriptValue::CoreString(_) => {
            ScriptValue::CoreString(ScriptCoreString::from_bytes(checked_string(lua, value)?))
        }
        ScriptValue::Bool(_) => match value {
            Value::Boolean(value) => ScriptValue::Bool(value),
            value => {
                return Err(Error::runtime(format!(
                    "expected boolean, got {}",
                    value.type_name()
                )));
            }
        },
        ScriptValue::Color(_) => ScriptValue::Color(super::lua_color::required_unsigned(
            lua,
            Some(&value),
            "color",
        )?),
        expected => {
            return Err(Error::runtime(format!(
                "cannot assign Lua {} to scripted data value {expected:?}",
                value.type_name()
            )));
        }
    };
    Ok(())
}

impl UserData for ScriptedDataValue {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            Ok(script_value_to_lua(lua, &this.value))
        });
        fields.add_field_method_set("value", |lua, this, value: Value| {
            set_value(lua, &mut this.value, value)
        });
        for (name, shift) in [("red", 16), ("green", 8), ("blue", 0), ("alpha", 24)] {
            fields.add_field_method_get(name, move |_, this| this.color_channel(name, shift));
            fields.add_field_method_set(name, move |lua, this, value: Value| {
                if !matches!(this.value, ScriptValue::Color(_)) {
                    return Ok(());
                }
                this.set_color_channel(shift, checked_integer(lua, value)?);
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

pub(super) fn create_data_value(lua: &Lua, value: ScriptValue) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(ScriptedDataValue::new(value))?;
    let patcher = ensure_metatable_patcher(lua)?;
    patcher.call(userdata)
}

/// Data values are also created on states that never ran the full VM boot
/// (e.g. a bare `LuaScriptInstance`), so the patcher installs on first use.
fn ensure_metatable_patcher(lua: &Lua) -> Result<Function> {
    if let Some(patcher) =
        lua.named_registry_value::<Option<Function>>(DATA_VALUE_METATABLE_PATCHER)?
    {
        return Ok(patcher);
    }
    // SAFETY: this bytecode is produced by the pinned build-time compiler from
    // the embedded source below.
    let chunk = unsafe {
        lua.load_bytecode(
            "rive_data_value_metatable",
            include_bytes!(concat!(
                env!("OUT_DIR"),
                "/data-value-metatable.luau-bytecode"
            )),
        )?
    };
    let patcher: Function = chunk.call(())?;
    lua.set_named_registry_value(DATA_VALUE_METATABLE_PATCHER, &patcher)?;
    Ok(patcher)
}

#[allow(dead_code)]
const DATA_VALUE_METATABLE_PATCHER_SOURCE: &str = r#"
local getmetatable = getmetatable
return function(value)
    local metatable = getmetatable(value)
    if metatable.__riveDataValuePatched then
        return value
    end
    local index = metatable.__index
    local newindex = metatable.__newindex
    metatable.__index = function(self, key)
        local result = index(self, key)
        if result ~= nil then
            return result
        end
        error(`'{tostring(key)}' is not a valid index of DataValue`, 2)
    end
    metatable.__newindex = function(self, key, newValue)
        if key == "value" or key == "red" or key == "green"
            or key == "blue" or key == "alpha" then
            return newindex(self, key, newValue)
        end
    end
    metatable.__riveDataValuePatched = true
    return value
end
"#;

pub(super) fn install_data_value_global(lua: &Lua) -> Result<()> {
    ensure_metatable_patcher(lua)?;

    let data_value = lua.create_table();
    data_value.set(
        "number",
        lua.create_function(|lua, ()| create_data_value(lua, ScriptValue::Number(0.0)))?,
    )?;
    data_value.set(
        "string",
        lua.create_function(|lua, ()| create_data_value(lua, ScriptValue::String(String::new())))?,
    )?;
    data_value.set(
        "boolean",
        lua.create_function(|lua, ()| create_data_value(lua, ScriptValue::Bool(false)))?,
    )?;
    data_value.set(
        "color",
        lua.create_function(|lua, ()| create_data_value(lua, ScriptValue::Color(0)))?,
    )?;
    data_value.set_readonly(true);
    lua.globals().set("DataValue", data_value)
}
