//! Direct retained owner for scripted Font values.

use std::sync::Arc;

use luaur_rt::{
    AnyUserData, Buffer, Error, Function, Lua, MultiValue, Result, Table, UserData, UserDataFields,
    UserDataMethods, Value,
};
use nuxie_runtime::source::{
    renderer::to_render_raw_path,
    text::{font_hb::HbFont, raw_text::RawText},
    text_engine::{Coord, Feature, FontRef},
};
use nuxie_runtime::{RuntimeFontAssetOwners, ScriptFont};

#[derive(Clone, Default)]
struct ScriptedFontAssetOwners(Option<Arc<RuntimeFontAssetOwners>>);

impl ScriptedFontAssetOwners {
    fn set(lua: &Lua, owners: Arc<RuntimeFontAssetOwners>) {
        lua.set_app_data(Self(Some(owners)));
    }

    fn for_lua(lua: &Lua) -> Option<Arc<RuntimeFontAssetOwners>> {
        lua.app_data_ref::<Self>()
            .and_then(|owners| owners.0.clone())
    }
}

/// Lua's opaque Font extern type. The Arc is the Rust counterpart of the
/// upstream `rcp<Font>` member and runs its destructor with the userdata.
pub(super) struct ScriptedFont {
    font_bytes: Arc<[u8]>,
    font: ScriptFont,
}

impl ScriptedFont {
    fn from_asset(lua: &Lua, font: ScriptFont) -> Option<Self> {
        let font = if let Some(native) = font
            .asset_global_id()
            .and_then(|id| ScriptedFontAssetOwners::for_lua(lua)?.native_font(id))
        {
            font.with_native_font(native)
        } else {
            font
        };
        let font_bytes = font
            .live_font_bytes_arc()
            .cloned()
            .or_else(|| {
                let asset_global_id = font.asset_global_id()?;
                ScriptedFontAssetOwners::for_lua(lua)?.get(asset_global_id)
            })
            .or_else(|| font.native_font().map(|_| Arc::from([])))?;
        let font = font.with_resolved_font_bytes(font_bytes.clone());
        let font = if font.native_font().is_none() {
            font.with_native_font(HbFont::decode(&font_bytes)?)
        } else {
            font
        };
        Some(Self { font_bytes, font })
    }

    pub(super) fn font_bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.font_bytes)
    }

    pub(super) fn font(&self) -> ScriptFont {
        self.font.clone()
    }

    pub(super) fn native(&self) -> Result<FontRef> {
        self.font
            .native_font()
            .ok_or_else(|| Error::runtime("Font is empty."))
    }
    pub(super) fn from_native(font: FontRef) -> Self {
        Self {
            font_bytes: Arc::from([]),
            font: ScriptFont::from_native_font(font),
        }
    }
}

fn tag(value: luaur_rt::LuaString) -> Result<u32> {
    let bytes = value.as_bytes();
    let bytes: [u8; 4] = bytes.as_slice().try_into().map_err(|_| {
        Error::runtime(format!(
            "'{}' is not a four letter tag",
            value.to_string_lossy()
        ))
    })?;
    Ok(u32::from_be_bytes(bytes))
}
fn tag_table(lua: &Lua, table: Option<Table>) -> Result<Vec<(u32, f64)>> {
    let mut values = Vec::new();
    if let Some(table) = table {
        for pair in table.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let Value::String(key) = key else {
                return Err(Error::runtime("font tags are four letter strings"));
            };
            let tag = tag(key)?;
            let value = lua
                .coerce_number(value)?
                .ok_or_else(|| Error::runtime("expected number"))?;
            values.push((tag, value));
        }
    }
    Ok(values)
}
impl UserData for ScriptedFont {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("ascent", |_, this| Ok(this.native()?.line_metrics().ascent));
        fields.add_field_method_get("descent", |_, this| {
            Ok(this.native()?.line_metrics().descent)
        });
        fields.add_field_method_get("capHeight", |_, this| {
            Ok(this.native()?.line_metrics().cap_height)
        });
        fields.add_field_method_get("xHeight", |_, this| {
            Ok(this.native()?.line_metrics().x_height)
        });
        fields.add_field_method_get("weight", |_, this| Ok(this.native()?.get_weight()));
        fields.add_field_method_get("isItalic", |_, this| Ok(this.native()?.is_italic()));
        fields.add_field_method_get("axes", |lua, this| {
            let font = this.native()?;
            let axes = lua.create_table();
            for i in 0..font.get_axis_count() {
                let axis = font.get_axis(i);
                let row = lua.create_table();
                row.set("tag", lua.create_string(axis.tag.to_be_bytes()))?;
                row.set("min", axis.min)?;
                row.set("default", axis.def)?;
                row.set("max", axis.max)?;
                axes.raw_set(i + 1, row)?;
            }
            Ok(axes)
        });
        fields.add_field_method_get("features", |lua, this| {
            let table = lua.create_table();
            for (i, feature) in this.native()?.features().into_iter().enumerate() {
                table.raw_set(i + 1, lua.create_string(feature.to_be_bytes()))?;
            }
            Ok(table)
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("lineHeight", |_, this, size: f32| {
            let font = this.native()?;
            Ok(font.descent(size) - font.ascent(size))
        });
        methods.add_method("hasGlyph", |lua, this, value: Value| {
            Ok(this
                .native()?
                .has_glyph(super::lua_color::required_unsigned(
                    lua,
                    Some(&value),
                    "codepoint",
                )?))
        });
        methods.add_method("axisValue", |_, this, value: luaur_rt::LuaString| {
            Ok(this.native()?.get_axis_value(tag(value)?))
        });
        methods.add_method(
            "withOptions",
            |lua, this, (coords, features): (Option<Table>, Option<Table>)| {
                let coords: Vec<_> = tag_table(lua, coords)?
                    .into_iter()
                    .map(|(axis, value)| Coord {
                        axis,
                        value: value as f32,
                    })
                    .collect();
                let features: Vec<_> = tag_table(lua, features)?
                    .into_iter()
                    .map(|(tag, value)| Feature {
                        tag,
                        value: (value as i64) as u32,
                    })
                    .collect();
                push_font(lua, this.native()?.with_options(&coords, &features))
            },
        );
        methods.add_method("glyphPath", |lua, this, glyph: Value| {
            let glyph = super::lua_color::required_unsigned(lua, Some(&glyph), "glyph")?;
            let path = RawText::glyph_path(this.native()?.as_ref(), glyph as u16);
            lua.create_userdata(super::lua_path::ScriptedPath::from_render_raw_path(
                to_render_raw_path(&path),
            ))
        });
    }
}

pub(super) fn register(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "decode",
        lua.create_function(|lua, buffer: Buffer| {
            HbFont::decode(&buffer.to_vec())
                .map(|font| push_font(lua, font))
                .transpose()
        })?,
    )?;
    lua.globals().set("Font", table)
}

pub(super) fn set_font_asset_owners(lua: &Lua, owners: Arc<RuntimeFontAssetOwners>) {
    ScriptedFontAssetOwners::set(lua, owners);
}

pub(super) fn create_asset_font(lua: &Lua, font: ScriptFont) -> Result<Option<AnyUserData>> {
    ScriptedFont::from_asset(lua, font)
        .map(|font| {
            let userdata = lua.create_userdata(font)?;
            prepare_font(lua, &userdata)?;
            Ok(userdata)
        })
        .transpose()
}

pub(super) fn push_font(lua: &Lua, font: FontRef) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(ScriptedFont::from_native(font))?;
    prepare_font(lua, &userdata)?;
    Ok(userdata)
}
fn prepare_font(lua: &Lua, userdata: &AnyUserData) -> Result<()> {
    source_metatable(
        lua,
        userdata,
        "Font",
        &[
            "ascent",
            "descent",
            "capHeight",
            "xHeight",
            "weight",
            "isItalic",
            "axes",
            "features",
        ],
        &[
            "lineHeight",
            "hasGlyph",
            "axisValue",
            "withOptions",
            "glyphPath",
        ],
        None,
        |userdata| {
            userdata.borrow::<ScriptedFont>()?.native()?;
            Ok(())
        },
    )
}

// Keep typed luaur storage while matching upstream's strict index and
// namecall-only method surface. The original dispatcher is retained privately.
fn source_dispatcher(lua: &Lua, userdata: &AnyUserData, key: &str) -> Result<Function> {
    let metatable: Table = unsafe {
        lua.exec_raw(userdata.clone(), |state| {
            luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
            Ok(())
        })?
    };
    metatable.raw_get(key)
}

pub(super) fn source_metatable(
    lua: &Lua,
    userdata: &AnyUserData,
    name: &'static str,
    fields: &'static [&'static str],
    methods: &'static [&'static str],
    setters: Option<&'static [&'static str]>,
    validate: fn(&AnyUserData) -> Result<()>,
) -> Result<()> {
    let metatable: Table = unsafe {
        lua.exec_raw(userdata.clone(), |state| {
            luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
            Ok(())
        })?
    };
    if metatable.is_readonly() {
        return Ok(());
    }
    let index: Function = metatable.get("__index")?;
    // Lua owns these references. Capturing a strong luaur Function in a Rust
    // callback would retain the VM (and the originating coroutine) in a cycle.
    metatable.raw_set("__rive_source_index", index)?;
    metatable.set(
        "__index",
        lua.create_function(move |lua, (userdata, key): (AnyUserData, Value)| {
            let key: luaur_rt::LuaString = lua.unpack(key)?;
            validate(&userdata)?;
            let key = key.to_string_lossy();
            if !fields.contains(&key.as_str()) {
                return Err(Error::runtime(format!(
                    "'{key}' is not a valid index of {name}"
                )));
            }
            source_dispatcher(lua, &userdata, "__rive_source_index")?.call::<Value>((userdata, key))
        })?,
    )?;
    metatable.set(
        "__namecall",
        lua.create_function(move |lua, mut args: MultiValue| {
            let method = unsafe {
                let mut atom = 0;
                let ptr = luaur_vm::functions::lua_namecallatom::lua_namecallatom(
                    lua.current_thread().state(),
                    &mut atom,
                );
                if ptr.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            };
            let userdata: AnyUserData = lua.unpack(args.front().cloned().unwrap_or(Value::Nil))?;
            validate(&userdata)?;
            if !methods.contains(&method.as_str()) {
                return Err(Error::runtime(format!(
                    "{method} is not a valid method of {name}"
                )));
            }
            let function: Function = source_dispatcher(lua, &userdata, "__rive_source_index")?
                .call((userdata, method))?;
            function.call::<MultiValue>(std::mem::take(&mut args))
        })?,
    )?;
    if let Some(setters) = setters {
        let setter: Function = metatable.get("__newindex")?;
        metatable.raw_set("__rive_source_newindex", setter)?;
        metatable.set(
            "__newindex",
            lua.create_function(
                move |lua, (userdata, key, value): (AnyUserData, Value, Value)| {
                    let key: luaur_rt::LuaString = lua.unpack(key)?;
                    let key = key.to_string_lossy();
                    validate(&userdata)?;
                    if !setters.contains(&key.as_str()) {
                        return Err(Error::runtime(format!(
                            "'{key}' is not a valid index of {name}"
                        )));
                    }
                    source_dispatcher(lua, &userdata, "__rive_source_newindex")?
                        .call::<()>((userdata, key, value))
                },
            )?,
        )?;
    }
    metatable.set_readonly(true);
    Ok(())
}
