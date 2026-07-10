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
    let get_list_model = model.clone();
    table.set(
        "getList",
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_list_model.property(&name) {
                Some(ScriptViewModelProperty::List) => lua
                    .create_userdata(ScriptedPropertyList::new(get_list_model.clone(), name))
                    .map(Value::UserData),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_boolean_model = model.clone();
    table.set(
        "getBoolean",
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_boolean_model.property(&name) {
                Some(ScriptViewModelProperty::Boolean) => lua
                    .create_userdata(ScriptedPropertyBoolean::new(
                        get_boolean_model.clone(),
                        name,
                    ))
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
            ScriptViewModelProperty::Boolean => {
                lua.create_userdata(ScriptedPropertyBoolean::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::Trigger => {
                lua.create_userdata(ScriptedPropertyTrigger::default())?
            }
            ScriptViewModelProperty::List => {
                lua.create_userdata(ScriptedPropertyList::new(model.clone(), name.clone()))?
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

struct ScriptedPropertyBoolean {
    model: ScriptViewModel,
    name: String,
}

impl ScriptedPropertyBoolean {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self { model, name }
    }
}

impl UserData for ScriptedPropertyBoolean {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(this.model.boolean(&this.name).unwrap_or_default())
        });
        fields.add_field_method_set("value", |_, this, value: bool| {
            this.model.set_boolean(&this.name, value);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getBoolean", |_, this, ()| {
            Ok(this.model.boolean(&this.name).unwrap_or_default())
        });
    }
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

struct ScriptedPropertyList {
    model: ScriptViewModel,
    name: String,
}

impl ScriptedPropertyList {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self { model, name }
    }

    fn item_value(&self, lua: &Lua, index: usize) -> luaur_rt::Result<Value> {
        match self.model.list_item(&self.name, index) {
            Some(item) => create_scripted_view_model(lua, item).map(Value::Table),
            None => Ok(Value::Nil),
        }
    }
}

impl UserData for ScriptedPropertyList {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("length", |_, this| {
            Ok(this.model.list_len(&this.name).unwrap_or_default())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("push", |_, this, item: Table| {
            let item = model_from_table(&item)?;
            this.model.push_list_item(&this.name, &item);
            Ok(())
        });
        methods.add_method("insert", |_, this, (item, index): (Table, usize)| {
            let item = model_from_table(&item)?;
            this.model
                .insert_list_item(&this.name, index.saturating_sub(1), &item);
            Ok(())
        });
        methods.add_method("pop", |lua, this, ()| {
            match this.model.pop_list_item(&this.name) {
                Some(item) => create_scripted_view_model(lua, item).map(Value::Table),
                None => Ok(Value::Nil),
            }
        });
        methods.add_method("shift", |lua, this, ()| {
            match this.model.shift_list_item(&this.name) {
                Some(item) => create_scripted_view_model(lua, item).map(Value::Table),
                None => Ok(Value::Nil),
            }
        });
        methods.add_method("swap", |_, this, (first, second): (usize, usize)| {
            this.model.swap_list_items(
                &this.name,
                first.saturating_sub(1),
                second.saturating_sub(1),
            );
            Ok(())
        });
        methods.add_method("clear", |_, this, ()| {
            this.model.clear_list_items(&this.name);
            Ok(())
        });
        methods.add_method("remove", |_, this, item: Table| {
            let item = model_from_table(&item)?;
            this.model.remove_list_item(&this.name, &item, false);
            Ok(())
        });
        methods.add_method("removeAt", |_, this, index: usize| {
            let Some(index) = index.checked_sub(1) else {
                return Err(luaur_rt::Error::runtime("removeAt index out of range"));
            };
            if !this.model.remove_list_item_at(&this.name, index) {
                return Err(luaur_rt::Error::runtime("removeAt index out of range"));
            }
            Ok(())
        });
        methods.add_method("removeAllOf", |_, this, item: Table| {
            let item = model_from_table(&item)?;
            this.model.remove_list_item(&this.name, &item, true);
            Ok(())
        });
        methods.add_meta_method("__index", |lua, this, key: Value| match key {
            Value::Integer(index) => usize::try_from(index)
                .ok()
                .and_then(|index| index.checked_sub(1))
                .map_or(Ok(Value::Nil), |index| this.item_value(lua, index)),
            Value::Number(index) if index >= 1.0 && index.fract() == 0.0 => {
                this.item_value(lua, index as usize - 1)
            }
            _ => Ok(Value::Nil),
        });
    }
}

pub(super) struct ScriptedContext {
    model: Rc<RefCell<Option<ScriptViewModel>>>,
    parents: Vec<ScriptViewModel>,
}

impl ScriptedContext {
    pub(super) fn new(
        model: Rc<RefCell<Option<ScriptViewModel>>>,
        parents: Vec<ScriptViewModel>,
    ) -> Self {
        Self { model, parents }
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
            Ok(
                match this
                    .parents
                    .last()
                    .cloned()
                    .or_else(|| this.model.borrow().clone())
                {
                    Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                    None => Value::Nil,
                },
            )
        });
        methods.add_method("dataContext", |lua, this, ()| {
            let Some(model) = this.model.borrow().clone() else {
                return Ok(Value::Nil);
            };
            lua.create_userdata(ScriptedDataContext {
                model,
                parents: this.parents.clone(),
            })
            .map(Value::UserData)
        });
    }
}

struct ScriptedDataContext {
    model: ScriptViewModel,
    parents: Vec<ScriptViewModel>,
}

impl UserData for ScriptedDataContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("viewModel", |lua, this, ()| {
            create_scripted_view_model(lua, this.model.clone()).map(Value::Table)
        });
        methods.add_method("parent", |lua, this, ()| {
            let Some((parent, remaining)) = this.parents.split_first() else {
                return Ok(Value::Nil);
            };
            lua.create_userdata(ScriptedDataContext {
                model: parent.clone(),
                parents: remaining.to_vec(),
            })
            .map(Value::UserData)
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
