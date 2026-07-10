use std::collections::BTreeMap;

use luaur_rt::{AnyUserData, Function, MultiValue, UserData, UserDataMethods, Value};
use rive_runtime::{ScriptViewModel, ScriptViewModelProperty};

/// Luau bindings ported from the ScriptedViewModel/ScriptedProperty trigger
/// slice of C++ `src/lua/lua_properties.cpp`.
pub(super) struct ScriptedViewModel {
    model: ScriptViewModel,
    property_cache: BTreeMap<String, AnyUserData>,
}

impl ScriptedViewModel {
    pub(super) fn new(model: ScriptViewModel) -> Self {
        Self {
            model,
            property_cache: BTreeMap::new(),
        }
    }

    pub(super) fn dispose(&mut self) {
        for property in self.property_cache.values() {
            if let Ok(mut trigger) = property.borrow_mut::<ScriptedPropertyTrigger>() {
                trigger.listeners.clear();
            }
        }
        self.property_cache.clear();
    }
}

impl UserData for ScriptedViewModel {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method_mut("__index", |lua, this, name: String| {
            if let Some(property) = this.property_cache.get(&name) {
                return Ok(Value::UserData(property.clone()));
            }
            let property = match this.model.property(&name) {
                Some(ScriptViewModelProperty::Trigger) => {
                    lua.create_userdata(ScriptedPropertyTrigger::default())?
                }
                None => return Ok(Value::Nil),
            };
            this.property_cache.insert(name, property.clone());
            Ok(Value::UserData(property))
        });
    }
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
