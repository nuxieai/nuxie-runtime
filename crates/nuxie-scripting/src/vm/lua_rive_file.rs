//! Live Lua bindings for `lua_rive_file.cpp`.
pub(super) use super::lua_main_ref::MainThreadRef;
use super::{
    ScriptVm, lua_promise, lua_renderer_library::RendererBindings,
    view_model::create_scripted_view_model,
};
use luaur_rt::{
    AnyUserData, Buffer, Lua, MetaMethod, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value,
};
use nuxie_runtime::source::{
    bindable_artboard::RuntimeBindableArtboardHandle,
    scripted::decoded_file::{decoded_bindable, import_decoded_file},
};
use nuxie_runtime::{
    CoreHandle, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle, RuntimeScriptingVmHandle,
    ScriptViewModel,
};
use std::cell::RefCell;

// These new bindings use byte-exact atom matching, but C-string diagnostics.
// Keep the older Font adapter's behavior untouched.
pub(super) fn c_prefix_index(
    lua: &Lua,
    value: &AnyUserData,
    name: &'static str,
    fields: &'static [&'static str],
) -> Result<()> {
    let metatable: Table = unsafe {
        lua.exec_raw(value.clone(), |state| {
            luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
            Ok(())
        })?
    };
    metatable.set_readonly(false);
    metatable.set(
        "__index",
        lua.create_function(
            move |lua, (value, key): (AnyUserData, luaur_rt::LuaString)| {
                let bytes = key.as_bytes();
                if !fields.iter().any(|field| field.as_bytes() == bytes) {
                    let prefix = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
                    return Err(luaur_rt::Error::runtime(format!(
                        "'{}' is not a valid index of {name}",
                        String::from_utf8_lossy(prefix)
                    )));
                }
                let metatable: Table = unsafe {
                    lua.exec_raw(value.clone(), |state| {
                        luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
                        Ok(())
                    })?
                };
                metatable
                    .raw_get::<luaur_rt::Function>("__rive_source_index")?
                    .call::<Value>((value, key))
            },
        )?,
    )?;
    metatable.set_readonly(true);
    Ok(())
}

pub(super) struct ScriptedBindableArtboard {
    pub(super) artboard: RuntimeBindableArtboardHandle,
    pub(super) view_model: Option<CoreHandle>,
    data: RefCell<Option<MainThreadRef>>,
}
impl ScriptedBindableArtboard {
    pub(super) fn create(
        lua: &Lua,
        artboard: RuntimeBindableArtboardHandle,
        view_model: Option<CoreHandle>,
    ) -> Result<AnyUserData> {
        let value = lua.create_userdata(Self::new(artboard, view_model))?;
        let metatable: Table = unsafe {
            lua.exec_raw(value.clone(), |state| {
                luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
                Ok(())
            })?
        };
        if !metatable.is_readonly() {
            let index: luaur_rt::Function = metatable.get("__index")?;
            metatable.raw_set("__rive_source_index", index)?;
            metatable.set(
                "__index",
                lua.create_function(|lua, (value, key): (AnyUserData, luaur_rt::LuaString)| {
                    value.borrow::<ScriptedBindableArtboard>()?;
                    let bytes = key.as_bytes();
                    if bytes != b"name" && bytes != b"data" {
                        let prefix = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
                        return Err(luaur_rt::Error::runtime(format!(
                            "'{}' is not a valid index of BindableArtboard",
                            String::from_utf8_lossy(prefix)
                        )));
                    }
                    let metatable: Table = unsafe {
                        lua.exec_raw(value.clone(), |state| {
                            luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
                            Ok(())
                        })?
                    };
                    metatable
                        .raw_get::<luaur_rt::Function>("__rive_source_index")?
                        .call::<Value>((value, key))
                })?,
            )?;
            metatable.set_readonly(true);
        }
        Ok(value)
    }
    pub(super) fn new(
        artboard: RuntimeBindableArtboardHandle,
        view_model: Option<CoreHandle>,
    ) -> Self {
        Self {
            artboard,
            view_model,
            data: RefCell::new(None),
        }
    }
}
impl UserData for ScriptedBindableArtboard {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| {
            Ok(this
                .artboard
                .with_artboard(|a| a.base.base.name().to_owned()))
        });
        fields.add_field_method_get("data", |lua, this| {
            if let Some(data) = this.data.borrow().clone() {
                return data.get::<Table>(lua).map(Some);
            }
            let model = this
                .view_model
                .clone()
                .zip(this.artboard.file().and_then(|f| f.upgrade()))
                .and_then(|(vm, file)| ScriptViewModel::from_native(vm, file));
            let data = model
                .map(|model| create_scripted_view_model(lua, model))
                .transpose()?;
            *this.data.borrow_mut() = data
                .as_ref()
                .map(|data| MainThreadRef::new(lua, data.clone()))
                .transpose()?;
            Ok(data)
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Eq, |_, this, other: AnyUserData| {
            Ok(other
                .borrow::<Self>()
                .is_ok_and(|other| this.artboard.ptr_eq(&other.artboard)))
        });
    }
}
struct ScriptedRiveFile(RuntimeFileHandle);
impl UserData for ScriptedRiveFile {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("artboardNames", |lua, this| {
            let names = this.0.with_file(|file| {
                (0..file.artboard_count())
                    .map(|i| file.artboard_name_at(i))
                    .collect::<Vec<_>>()
            });
            lua.create_sequence_from(names)
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "bindableArtboard",
            |lua, this, name: Option<luaur_rt::LuaString>| {
                let name = name.map(|name| name.as_bytes());
                let canonical = name.as_ref().and_then(|name| {
                    let prefix = name.split(|byte| *byte == 0).next().unwrap_or_default();
                    this.0.with_file(|file| {
                        (0..file.artboard_count())
                            .map(|i| file.artboard_name_at(i))
                            .find(|name| name.as_bytes() == prefix)
                    })
                });
                if name.is_some() && canonical.is_none() {
                    return Ok(None);
                }
                let bindable = this
                    .0
                    .with_file(|file| decoded_bindable(file, canonical.as_deref()));
                bindable
                    .artboard
                    .map(|artboard| {
                        ScriptedBindableArtboard::create(lua, artboard, bindable.view_model)
                    })
                    .transpose()
            },
        );
    }
}
pub(super) fn decode_file(lua: &Lua, encoded: Buffer) -> Result<AnyUserData> {
    let bytes = encoded.to_vec();
    if bytes.is_empty() {
        return Err(luaur_rt::Error::runtime("decodeFile: empty buffer"));
    }
    let factory = RendererBindings::for_lua(lua)
        .and_then(|bindings| {
            bindings
                .with_factory(|factory| Ok(RuntimeFactoryHandle::from_factory(factory)))
                .ok()
                .flatten()
        })
        .ok_or_else(|| {
            luaur_rt::Error::runtime("decodeFile: this Lua state has no factory to decode with")
        })?;
    let promise = lua_promise::new_pending(lua)?;
    let mut result = ImportResult::Success;
    let mut make_vm = |factory: RuntimeFactoryHandle| {
        let vm = ScriptVm::new();
        factory
            .with_factory_mut(|factory| vm.install_render_factory(factory))
            .expect("a fresh decoded VM accepts its retained factory");
        RuntimeScriptingVmHandle::new(Box::new(vm))
    };
    let file = import_decoded_file(&bytes, factory, Some(&mut result), Some(&mut make_vm));
    if let Some(file) = file {
        lua_promise::resolve(
            lua,
            promise.clone(),
            Value::UserData({
                let value = lua.create_userdata(ScriptedRiveFile(file))?;
                super::lua_font::source_metatable(
                    lua,
                    &value,
                    "RiveFile",
                    &["artboardNames"],
                    &["bindableArtboard"],
                    None,
                    |value| value.borrow::<ScriptedRiveFile>().map(|_| ()),
                )?;
                c_prefix_index(lua, &value, "RiveFile", &["artboardNames"])?;
                value
            }),
        )?;
    } else {
        lua_promise::reject(
            lua,
            promise.clone(),
            if result == ImportResult::UnsupportedVersion {
                "unsupportedVersion: not a Rive file this runtime can read"
            } else {
                "malformed: not a Rive file"
            }
            .into(),
        )?;
    }
    Ok(promise)
}
