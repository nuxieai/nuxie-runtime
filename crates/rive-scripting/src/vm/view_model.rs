use std::collections::BTreeMap;

use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{
    Function, Lua, MultiValue, Table, UserData, UserDataFields, UserDataMethods, Value,
};
use rive_runtime::{ScriptViewModel, ScriptViewModelProperty};

/// Luau bindings ported from the ScriptedViewModel/ScriptedProperty trigger
/// slice of C++ `src/lua/lua_properties.cpp`.
struct ScriptedViewModelHandle {
    model: ScriptViewModel,
}

impl UserData for ScriptedViewModelHandle {}

pub(super) fn create_scripted_view_model(
    lua: &Lua,
    model: ScriptViewModel,
) -> luaur_rt::Result<Table> {
    let table = lua.create_table();
    table.set(
        "__rive_model",
        lua.create_userdata(ScriptedViewModelHandle {
            model: model.clone(),
        })?,
    )?;

    let get_number_model = model.clone();
    table.set(
        "getNumber",
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_number_model.property(&name) {
                Some(ScriptViewModelProperty::Number) => lua
                    .create_userdata(ScriptedPropertyNumber::new(get_number_model.clone(), name))
                    .map(Value::UserData),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_string_model = model.clone();
    table.set(
        "getString",
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_string_model.property(&name) {
                Some(ScriptViewModelProperty::String) => lua
                    .create_userdata(ScriptedPropertyString::new(get_string_model.clone(), name))
                    .map(Value::UserData),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let instance_model = model.clone();
    table.set(
        "instance",
        lua.create_function(move |lua, (_self, name): (Table, Option<String>)| {
            let model = instance_model
                .named_instance(name.as_deref())
                .or_else(|| instance_model.named_instance(None))
                .ok_or_else(|| luaur_rt::Error::runtime("view-model instance not found"))?;
            create_scripted_view_model(lua, model)
        })?,
    )?;
    let get_view_model = model.clone();
    table.set(
        "getViewModel",
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            Ok(match get_view_model.view_model(&name) {
                Some(model) => {
                    Value::UserData(lua.create_userdata(ScriptedPropertyViewModel::new(model))?)
                }
                None => Value::Nil,
            })
        })?,
    )?;

    for (name, kind) in model.properties() {
        let property = match kind {
            ScriptViewModelProperty::Number => {
                lua.create_userdata(ScriptedPropertyNumber::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::String => {
                lua.create_userdata(ScriptedPropertyString::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::Trigger => {
                lua.create_userdata(ScriptedPropertyTrigger::default())?
            }
            ScriptViewModelProperty::ViewModel => {
                let nested = model.view_model(name).ok_or_else(|| {
                    luaur_rt::Error::runtime(format!(
                        "view-model property '{name}' has no active instance"
                    ))
                })?;
                lua.create_userdata(ScriptedPropertyViewModel::new(nested))?
            }
        };
        table.set(name.as_str(), property)?;
    }
    Ok(table)
}

pub(super) fn model_from_table(table: &Table) -> luaur_rt::Result<ScriptViewModel> {
    let handle = table.get::<luaur_rt::AnyUserData>("__rive_model")?;
    Ok(handle.borrow::<ScriptedViewModelHandle>()?.model.clone())
}

struct ScriptedPropertyViewModel {
    model: ScriptViewModel,
}

impl ScriptedPropertyViewModel {
    fn new(model: ScriptViewModel) -> Self {
        Self { model }
    }
}

impl UserData for ScriptedPropertyViewModel {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            create_scripted_view_model(lua, this.model.clone()).map(Value::Table)
        });
    }
}

struct ScriptedPropertyNumber {
    model: ScriptViewModel,
    name: String,
}

impl ScriptedPropertyNumber {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self { model, name }
    }
}

impl UserData for ScriptedPropertyNumber {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(this.model.number(&this.name).unwrap_or_default())
        });
        fields.add_field_method_set("value", |_, this, value: f32| {
            this.model.set_number(&this.name, value);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getNumber", |_, this, ()| {
            Ok(this.model.number(&this.name).unwrap_or_default())
        });
    }
}

struct ScriptedPropertyString {
    model: ScriptViewModel,
    name: String,
}

impl ScriptedPropertyString {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self { model, name }
    }
}

impl UserData for ScriptedPropertyString {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(this.model.string(&this.name).unwrap_or_default())
        });
        fields.add_field_method_set("value", |_, this, value: String| {
            this.model.set_string(&this.name, &value);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getString", |_, this, ()| {
            Ok(this.model.string(&this.name).unwrap_or_default())
        });
    }
}

pub(super) struct ScriptedContext {
    model: Rc<RefCell<Option<ScriptViewModel>>>,
}

impl ScriptedContext {
    pub(super) fn new(model: Rc<RefCell<Option<ScriptViewModel>>>) -> Self {
        Self { model }
    }
}

impl UserData for ScriptedContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("viewModel", |lua, this, ()| {
            Ok(match this.model.borrow().clone() {
                Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                None => Value::Nil,
            })
        });
        methods.add_method("rootViewModel", |lua, this, ()| {
            Ok(match this.model.borrow().clone() {
                Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                None => Value::Nil,
            })
        });
    }
}

pub(super) fn install_data_global(
    lua: &Lua,
    models: &BTreeMap<String, ScriptViewModel>,
) -> luaur_rt::Result<()> {
    let data = lua.create_table();
    for (name, model) in models {
        let definition = lua.create_table();
        let model = model.clone();
        definition.set(
            "new",
            lua.create_function(move |lua, name: Option<String>| {
                let instance = model
                    .named_instance(name.as_deref())
                    .or_else(|| model.named_instance(None))
                    .ok_or_else(|| luaur_rt::Error::runtime("view-model instance not found"))?;
                create_scripted_view_model(lua, instance)
            })?,
        )?;
        data.set(name.as_str(), definition)?;
    }
    lua.globals().set("Data", data)
}

#[derive(Default)]
struct ScriptedPropertyTrigger {
    listeners: Vec<ScriptedListener>,
}

struct ScriptedListener {
    callback: Function,
    userdata: Option<Value>,
}

impl UserData for ScriptedPropertyTrigger {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |_, this, args: MultiValue| {
            let args = args.into_vec();
            let (userdata, callback) = match args.as_slice() {
                [Value::Function(callback)] => (None, callback.clone()),
                [userdata, Value::Function(callback)] => (Some(userdata.clone()), callback.clone()),
                _ => {
                    return Err(luaur_rt::Error::runtime(
                        "addListener expects a callback or userdata and callback",
                    ));
                }
            };
            this.listeners.push(ScriptedListener { callback, userdata });
            Ok(())
        });
        methods.add_method_mut("fire", |_, this, ()| {
            for listener in this.listeners.iter().rev() {
                listener
                    .callback
                    .call::<()>(listener.userdata.clone().unwrap_or(Value::Nil))?;
            }
            Ok(())
        });
    }
}
