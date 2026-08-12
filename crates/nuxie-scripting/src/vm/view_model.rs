use std::collections::{BTreeMap, BTreeSet};

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use luaur_rt::{
    AnyUserData, Buffer, Function, Lua, MultiValue, Table, UserData, UserDataFields,
    UserDataMethods, Value,
};
use luaur_vm::functions::lua_getmetatable::lua_getmetatable;
use nuxie_runtime::view_model_cell::RuntimeCellDirtSink;
use nuxie_runtime::{
    RuntimeBlobAsset, RuntimeOwnedViewModelInstance, ScriptViewModel,
    ScriptViewModelChangeRegistration, ScriptViewModelProperty,
};

use super::lua_blob::{ScriptedBlob, ScriptedBlobAssets};
use super::lua_font::{ScriptedFont, create_asset_font};
use super::lua_image::{ScriptedImage, create_asset_image};

type ViewModelInstance = Rc<RefCell<RuntimeOwnedViewModelInstance>>;
type ViewModelInstanceWeak = Weak<RefCell<RuntimeOwnedViewModelInstance>>;
type ViewModelInstanceKey = usize;

const PROPERTY_METATABLE_PATCHER: &str = "rive_property_metatable_patcher";
const PROPERTY_LISTENER_FLUSH: &str = "rive_property_listener_flush";
const TRIGGER_MUTATING_METHODS: &[&str] = &["fire"];
const LIST_MUTATING_METHODS: &[&str] = &[
    "push",
    "pop",
    "swap",
    "shift",
    "clear",
    "insert",
    "remove",
    "removeAt",
    "removeAllOf",
];

fn instance_key(instance: &ViewModelInstance) -> ViewModelInstanceKey {
    Rc::as_ptr(instance) as usize
}

#[derive(Default)]
struct TrackedViewModels {
    instances: BTreeMap<ViewModelInstanceKey, TrackedViewModel>,
}

struct TrackedViewModel {
    instance: ViewModelInstanceWeak,
    strong_instance: Option<ViewModelInstance>,
    registrations: usize,
}

/// Per-VM equivalent of C++ `ScriptingContext`'s owner-counted detached VMI
/// registry. The runtime-owned instance topology decides which registered
/// instances are detached; this registry only owns registration lifetimes.
#[derive(Clone, Default)]
pub(crate) struct ScriptViewModelFrameContext {
    tracked: Rc<RefCell<TrackedViewModels>>,
    trigger_watches: Rc<RefCell<Vec<Rc<ScriptedTriggerWatch>>>>,
    blob_watches: Rc<RefCell<Vec<Rc<ScriptedBlobWatch>>>>,
    property_watches: Rc<RefCell<Vec<Rc<ScriptedPropertyWatch>>>>,
}

impl std::fmt::Debug for ScriptViewModelFrameContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptViewModelFrameContext")
            .field("tracked_instances", &self.tracked.borrow().instances.len())
            .field("trigger_watches", &self.trigger_watches.borrow().len())
            .field("blob_watches", &self.blob_watches.borrow().len())
            .field("property_watches", &self.property_watches.borrow().len())
            .finish()
    }
}

impl ScriptViewModelFrameContext {
    pub(crate) fn for_lua(lua: &Lua) -> Self {
        if let Some(context) = lua
            .app_data_ref::<ScriptViewModelFrameContext>()
            .map(|context| context.clone())
        {
            return context;
        }
        let context = Self::default();
        lua.set_app_data(context.clone());
        context
    }

    fn ensure_entry<'a>(
        tracked: &'a mut TrackedViewModels,
        instance: &ViewModelInstance,
    ) -> &'a mut TrackedViewModel {
        let key = instance_key(instance);
        let replace = tracked.instances.get(&key).is_some_and(|entry| {
            entry
                .instance
                .upgrade()
                .is_none_or(|current| !Rc::ptr_eq(&current, instance))
        });
        if replace {
            tracked.instances.remove(&key);
        }
        tracked
            .instances
            .entry(key)
            .or_insert_with(|| TrackedViewModel {
                instance: Rc::downgrade(instance),
                strong_instance: None,
                registrations: 0,
            })
    }

    pub(crate) fn register(&self, model: &ScriptViewModel) -> ScriptViewModelRegistration {
        let instance = model.owned_instance();
        let key = instance_key(&instance);
        {
            let mut tracked = self.tracked.borrow_mut();
            let entry = Self::ensure_entry(&mut tracked, &instance);
            entry.registrations = entry.registrations.saturating_add(1);
            entry.strong_instance = Some(Rc::clone(&instance));
        }
        ScriptViewModelRegistration {
            tracked: Rc::downgrade(&self.tracked),
            key,
        }
    }

    fn register_trigger_watch(&self, watch: &Rc<ScriptedTriggerWatch>) {
        let mut watches = self.trigger_watches.borrow_mut();
        if watches.iter().any(|candidate| Rc::ptr_eq(candidate, watch)) {
            return;
        }
        watches.push(Rc::clone(watch));
    }

    fn register_blob_watch(&self, watch: &Rc<ScriptedBlobWatch>) {
        let mut watches = self.blob_watches.borrow_mut();
        if watches.iter().any(|candidate| Rc::ptr_eq(candidate, watch)) {
            return;
        }
        watches.push(Rc::clone(watch));
    }

    fn register_property_watch(&self, watch: &Rc<ScriptedPropertyWatch>) {
        let mut watches = self.property_watches.borrow_mut();
        if watches.iter().any(|candidate| Rc::ptr_eq(candidate, watch)) {
            return;
        }
        watches.push(Rc::clone(watch));
    }

    fn dispatch_property_watches(&self) -> bool {
        let watches = self.property_watches.borrow().clone();
        let mut changed = false;
        for watch in watches {
            if watch.sink.take_dirt().is_empty() {
                continue;
            }
            changed = true;
            notify_property_listeners(&watch);
        }
        self.property_watches
            .borrow_mut()
            .retain(|watch| !watch.listeners.borrow().is_empty());
        changed
    }

    fn dispatch_pending_listeners(&self) {
        self.dispatch_trigger_watches();
        self.dispatch_blob_watches();
        self.dispatch_property_watches();
    }

    fn dispatch_trigger_watches(&self) -> bool {
        let watches = self.trigger_watches.borrow().clone();
        let mut changed = false;
        for watch in watches {
            if watch.sink.take_dirt().is_empty() {
                continue;
            }
            changed = true;
            let listeners = watch.listeners.borrow().clone();
            for listener in listeners.into_iter().rev() {
                let _ = listener
                    .callback
                    .call::<()>(listener.userdata.unwrap_or(Value::Nil));
            }
        }
        self.trigger_watches
            .borrow_mut()
            .retain(|watch| !watch.listeners.borrow().is_empty());
        changed
    }

    fn dispatch_blob_watches(&self) -> bool {
        let watches = self.blob_watches.borrow().clone();
        let mut changed = false;
        for watch in watches {
            if watch.sink.take_dirt().is_empty() {
                continue;
            }
            changed = true;
            let listeners = watch.listeners.borrow().clone();
            for listener in listeners.into_iter().rev() {
                let _ = listener
                    .callback
                    .call::<()>(listener.userdata.unwrap_or(Value::Nil));
            }
        }
        self.blob_watches
            .borrow_mut()
            .retain(|watch| !watch.listeners.borrow().is_empty());
        changed
    }

    fn clear_trigger_watch_dirt(&self) {
        let mut retained = self.trigger_watches.borrow_mut();
        retained.retain(|watch| {
            let _ = watch.sink.take_dirt();
            !watch.listeners.borrow().is_empty()
        });
    }

    pub(crate) fn advance_detached(&self) -> bool {
        let mut changed = self.dispatch_trigger_watches();
        changed |= self.dispatch_blob_watches();
        changed |= self.dispatch_property_watches();
        let roots = {
            let mut tracked = self.tracked.borrow_mut();
            tracked
                .instances
                .retain(|_, entry| entry.registrations > 0 && entry.instance.strong_count() > 0);
            tracked
                .instances
                .values()
                .filter_map(|entry| {
                    let instance = entry.instance.upgrade()?;
                    let is_detached = !instance.borrow().has_parents();
                    is_detached.then_some(instance)
                })
                .collect::<Vec<_>>()
        };
        changed |= ScriptViewModel::advance_owned_instances(&roots);
        // Trigger reset cascades ordinary dirt even though C++ suppresses its
        // delegate callback. Consume that reset dirt so it cannot replay the
        // Lua listener on the next host frame.
        self.clear_trigger_watch_dirt();
        changed
    }

    #[cfg(test)]
    fn registrations(&self, model: &ScriptViewModel) -> usize {
        self.tracked
            .borrow()
            .instances
            .get(&instance_key(&model.owned_instance()))
            .map(|entry| entry.registrations)
            .unwrap_or_default()
    }
}

pub(crate) struct ScriptViewModelRegistration {
    tracked: Weak<RefCell<TrackedViewModels>>,
    key: ViewModelInstanceKey,
}

impl Drop for ScriptViewModelRegistration {
    fn drop(&mut self) {
        let Some(tracked) = self.tracked.upgrade() else {
            return;
        };
        let mut tracked = tracked.borrow_mut();
        let Some(entry) = tracked.instances.get_mut(&self.key) else {
            return;
        };
        entry.registrations = entry.registrations.saturating_sub(1);
        if entry.registrations == 0 {
            entry.strong_instance = None;
        }
    }
}

/// Luau bindings ported from the ScriptedViewModel/ScriptedProperty trigger
/// slice of C++ `src/lua/lua_properties.cpp`.
struct ScriptedViewModelHandle {
    model: ScriptViewModel,
    _registration: ScriptViewModelRegistration,
}

impl UserData for ScriptedViewModelHandle {}

pub(super) fn create_scripted_view_model(
    lua: &Lua,
    model: ScriptViewModel,
) -> luaur_rt::Result<Table> {
    if lua
        .named_registry_value::<Function>(PROPERTY_METATABLE_PATCHER)
        .is_err()
    {
        install_property_binding_support(lua)?;
    }
    create_scripted_view_model_retained(lua, model)
}

pub(super) fn install_property_binding_support(lua: &Lua) -> luaur_rt::Result<()> {
    let flush = lua.create_function(|lua, ()| {
        ScriptViewModelFrameContext::for_lua(lua).dispatch_pending_listeners();
        Ok(())
    })?;
    lua.set_named_registry_value(PROPERTY_LISTENER_FLUSH, flush)?;
    // SAFETY: this bytecode is produced by the pinned build-time compiler from
    // the embedded source below.
    let chunk = unsafe {
        lua.load_bytecode(
            "rive_property_metatable",
            include_bytes!(concat!(
                env!("OUT_DIR"),
                "/property-metatable.luau-bytecode"
            )),
        )?
    };
    let patcher: Function = chunk.call(())?;
    lua.set_named_registry_value(PROPERTY_METATABLE_PATCHER, patcher)
}

#[allow(dead_code)]
const PROPERTY_METATABLE_PATCHER_SOURCE: &str = r#"
local getmetatable = getmetatable
local pack = table.pack
local type = type
local unpack = table.unpack
return function(property, writableValue, mutatingMethods, flush)
    local metatable = getmetatable(property)
    if metatable.__rivePropertyPatched then
        return property
    end
    local index = metatable.__index
    local newindex = metatable.__newindex
    metatable.__index = function(self, key)
        local result
        if type(index) == "function" then
            result = index(self, key)
        else
            result = index[key]
        end
        if type(result) == "function" and mutatingMethods[key] then
            return function(...)
                local values = pack(result(...))
                flush()
                return unpack(values, 1, values.n)
            end
        end
        return result
    end
    if writableValue then
        metatable.__newindex = function(self, key, value)
            if type(key) ~= "string" then
                error(`string expected, got {type(key)}`, 2)
            end
            if key == "value" then
                newindex(self, key, value)
                flush()
            end
        end
    end
    metatable.__rivePropertyPatched = true
    return property
end
"#;

fn patch_property_userdata(
    lua: &Lua,
    property: AnyUserData,
    writable_value: bool,
    mutating_methods: &[&str],
) -> luaur_rt::Result<AnyUserData> {
    let mutating = lua.create_table();
    for method in mutating_methods {
        mutating.set(*method, true)?;
    }
    let patcher: Function = lua.named_registry_value(PROPERTY_METATABLE_PATCHER)?;
    let flush: Function = lua.named_registry_value(PROPERTY_LISTENER_FLUSH)?;
    patcher.call((property, writable_value, mutating, flush))
}

fn create_property_userdata<T: UserData + 'static>(
    lua: &Lua,
    property: T,
    writable_value: bool,
    mutating_methods: &[&str],
) -> luaur_rt::Result<AnyUserData> {
    let property = lua.create_userdata(property)?;
    patch_property_userdata(lua, property, writable_value, mutating_methods)
}

fn create_scripted_view_model_retained(
    lua: &Lua,
    model: ScriptViewModel,
) -> luaur_rt::Result<Table> {
    let frame_context = ScriptViewModelFrameContext::for_lua(lua);
    let registration = frame_context.register(&model);
    let table = lua.create_table();
    table.set(
        "__rive_model",
        lua.create_userdata(ScriptedViewModelHandle {
            model: model.clone(),
            _registration: registration,
        })?,
    )?;

    let get_number_model = model.clone();
    table.set(
        "getNumber",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_number_model.property(&name) {
                Some(ScriptViewModelProperty::Number) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_color_model = model.clone();
    table.set(
        "getColor",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_color_model.property(&name) {
                Some(ScriptViewModelProperty::Color) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_string_model = model.clone();
    table.set(
        "getString",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_string_model.property(&name) {
                Some(ScriptViewModelProperty::String) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_list_model = model.clone();
    table.set(
        "getList",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_list_model.property(&name) {
                Some(ScriptViewModelProperty::List) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_trigger_model = model.clone();
    table.set(
        "getTrigger",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_trigger_model.property(&name) {
                Some(ScriptViewModelProperty::Trigger) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_boolean_model = model.clone();
    table.set(
        "getBoolean",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_boolean_model.property(&name) {
                Some(ScriptViewModelProperty::Boolean) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_enum_model = model.clone();
    table.set(
        "getEnum",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_enum_model.property(&name) {
                Some(ScriptViewModelProperty::Enum) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_index_model = model.clone();
    table.set(
        "getIndex",
        lua.create_function(move |_, _self: Table| {
            Ok(get_index_model
                .component_list_item_index()
                .and_then(|index| i64::try_from(index).ok())
                .unwrap_or(-1))
        })?,
    )?;
    let get_image_model = model.clone();
    table.set(
        "getImage",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_image_model.property(&name) {
                Some(ScriptViewModelProperty::Image) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_blob_model = model.clone();
    table.set(
        "getBlob",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_blob_model.property(&name) {
                Some(ScriptViewModelProperty::Blob) => this.get(name),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_font_model = model.clone();
    table.set(
        "getFont",
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_font_model.property(&name) {
                Some(ScriptViewModelProperty::Font) => this.get(name),
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
        lua.create_function(move |_, (this, name): (Table, String)| {
            match get_view_model.view_model(&name) {
                Some(_) => this.get(name),
                None => Ok(Value::Nil),
            }
        })?,
    )?;

    let symbol_list_index_names = model
        .properties()
        .iter()
        .filter_map(|(name, kind)| {
            (*kind == ScriptViewModelProperty::SymbolListIndex).then(|| name.clone())
        })
        .collect::<BTreeSet<_>>();
    for (name, kind) in model.properties() {
        if *kind == ScriptViewModelProperty::SymbolListIndex {
            continue;
        }
        if *kind == ScriptViewModelProperty::List {
            table.set(
                name.as_str(),
                create_scripted_property_list(lua, model.clone(), name.clone())?,
            )?;
            continue;
        }
        let property = match kind {
            ScriptViewModelProperty::Number => create_property_userdata(
                lua,
                ScriptedPropertyNumber::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::Color => create_property_userdata(
                lua,
                ScriptedPropertyColor::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::String => create_property_userdata(
                lua,
                ScriptedPropertyString::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::Boolean => create_property_userdata(
                lua,
                ScriptedPropertyBoolean::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::Enum => create_property_userdata(
                lua,
                ScriptedPropertyEnum::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::Trigger => create_property_userdata(
                lua,
                ScriptedPropertyTrigger::new(model.clone(), name.clone()),
                false,
                TRIGGER_MUTATING_METHODS,
            )?,
            ScriptViewModelProperty::Image => create_property_userdata(
                lua,
                ScriptedPropertyImage::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::Blob => create_property_userdata(
                lua,
                ScriptedPropertyBlob::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::Font => create_property_userdata(
                lua,
                ScriptedPropertyFont::new(model.clone(), name.clone()),
                true,
                &[],
            )?,
            ScriptViewModelProperty::List => unreachable!("lists are installed before wrapping"),
            ScriptViewModelProperty::ViewModel => {
                model.view_model(name).ok_or_else(|| {
                    luaur_rt::Error::runtime(format!(
                        "view-model property '{name}' has no active instance"
                    ))
                })?;
                create_property_userdata(
                    lua,
                    ScriptedPropertyViewModel::new(model.clone(), name.clone()),
                    true,
                    &[],
                )?
            }
            ScriptViewModelProperty::SymbolListIndex => unreachable!(
                "symbol-list indices are exposed as scalar values before property wrapping"
            ),
        };
        table.set(name.as_str(), property)?;
    }
    if !symbol_list_index_names.is_empty() {
        let index_model = model.clone();
        let metatable = lua.create_table();
        metatable.set(
            "__index",
            lua.create_function(move |_, (_table, key): (Table, Value)| {
                let Value::String(key) = key else {
                    return Ok(Value::Nil);
                };
                if !symbol_list_index_names.contains(key.to_str()?.as_str()) {
                    return Ok(Value::Nil);
                }
                Ok(Value::Integer(
                    index_model
                        .component_list_item_index()
                        .and_then(|index| i64::try_from(index).ok())
                        .unwrap_or(-1),
                ))
            })?,
        )?;
        table.set_metatable(Some(metatable))?;
    }
    Ok(table)
}

pub(super) fn model_from_table(table: &Table) -> luaur_rt::Result<ScriptViewModel> {
    let handle = table.get::<luaur_rt::AnyUserData>("__rive_model")?;
    Ok(handle.borrow::<ScriptedViewModelHandle>()?.model.clone())
}

struct ScriptedPropertyWatch {
    sink: RuntimeCellDirtSink,
    listeners: Rc<RefCell<Vec<ScriptedListener>>>,
    _change_registration: RefCell<Option<ScriptViewModelChangeRegistration>>,
}

fn property_watch(model: &ScriptViewModel, name: &str) -> Rc<ScriptedPropertyWatch> {
    let listeners = Rc::new(RefCell::new(Vec::new()));
    let sink = model.property_dirt_sink(name).unwrap_or_default();
    let watch = Rc::new(ScriptedPropertyWatch {
        sink,
        listeners,
        _change_registration: RefCell::new(None),
    });
    let weak_watch = Rc::downgrade(&watch);
    let registration = model.add_property_change_callback(
        name,
        Rc::new(move || {
            let Some(watch) = weak_watch.upgrade() else {
                return;
            };
            if watch.sink.take_dirt().is_empty() {
                return;
            }
            notify_property_listeners(&watch);
        }),
    );
    *watch._change_registration.borrow_mut() = registration;
    watch
}

fn add_property_listener(
    lua: &Lua,
    watch: &Rc<ScriptedPropertyWatch>,
    args: MultiValue,
) -> luaur_rt::Result<()> {
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
    watch
        .listeners
        .borrow_mut()
        .push(ScriptedListener { callback, userdata });
    ScriptViewModelFrameContext::for_lua(lua).register_property_watch(watch);
    Ok(())
}

fn remove_property_listener(
    watch: &Rc<ScriptedPropertyWatch>,
    args: MultiValue,
) -> luaur_rt::Result<()> {
    let args = args.into_vec();
    let callback = match args.as_slice() {
        [Value::Function(callback)] | [_, Value::Function(callback)] => callback,
        _ => {
            return Err(luaur_rt::Error::runtime(
                "removeListener expects a callback or userdata and callback",
            ));
        }
    };
    let identity = callback.to_pointer();
    watch
        .listeners
        .borrow_mut()
        .retain(|listener| listener.callback.to_pointer() != identity);
    Ok(())
}

fn notify_property_listeners(watch: &ScriptedPropertyWatch) {
    call_property_listeners(&watch.listeners);
}

fn call_property_listeners(listeners: &RefCell<Vec<ScriptedListener>>) {
    let listeners = listeners.borrow().clone();
    for listener in listeners.into_iter().rev() {
        let _ = listener
            .callback
            .call::<()>(listener.userdata.unwrap_or(Value::Nil));
    }
}

struct ScriptedPropertyViewModel {
    parent: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedPropertyWatch>,
    cached_value: Rc<RefCell<Option<Table>>>,
    _change_sink: RuntimeCellDirtSink,
}

impl ScriptedPropertyViewModel {
    fn new(parent: ScriptViewModel, name: String) -> Self {
        let watch = property_watch(&parent, &name);
        let cached_value = Rc::new(RefCell::new(None));
        let weak_cached_value = Rc::downgrade(&cached_value);
        let change_sink = parent
            .property_change_sink(&name, move || {
                if let Some(cached_value) = weak_cached_value.upgrade() {
                    cached_value.borrow_mut().take();
                }
            })
            .unwrap_or_default();
        Self {
            parent,
            name,
            watch,
            cached_value,
            _change_sink: change_sink,
        }
    }
}

impl UserData for ScriptedPropertyViewModel {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            if let Some(cached) = this.cached_value.borrow().as_ref() {
                return Ok(Value::Table(cached.clone()));
            }
            let model = this.parent.view_model(&this.name).ok_or_else(|| {
                luaur_rt::Error::runtime(format!(
                    "view-model property '{}' has no active instance",
                    this.name
                ))
            })?;
            let value = create_scripted_view_model(lua, model)?;
            *this.cached_value.borrow_mut() = Some(value.clone());
            Ok(Value::Table(value))
        });
        fields.add_field_method_set("value", |_, this, value: Table| {
            let value = model_from_table(&value)?;
            this.parent
                .defer_property_change_callbacks(|| this.parent.set_view_model(&this.name, &value));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

struct ScriptedPropertyNumber {
    model: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedPropertyWatch>,
}

struct ScriptedPropertyColor {
    model: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedPropertyWatch>,
}

impl ScriptedPropertyColor {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let watch = property_watch(&model, &name);
        Self { model, name, watch }
    }
}

impl UserData for ScriptedPropertyColor {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(i64::from(this.model.color(&this.name).unwrap_or_default()))
        });
        fields.add_field_method_set("value", |lua, this, value: Value| {
            let value = super::lua_color::required_unsigned(lua, Some(&value), "color")?;
            this.model
                .defer_property_change_callbacks(|| this.model.set_color(&this.name, value));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

impl ScriptedPropertyNumber {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let watch = property_watch(&model, &name);
        Self { model, name, watch }
    }
}

impl UserData for ScriptedPropertyNumber {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(this.model.number(&this.name).unwrap_or_default())
        });
        fields.add_field_method_set("value", |lua, this, value: Value| {
            let value = super::lua_data_value::checked_number(lua, value)? as f32;
            this.model
                .defer_property_change_callbacks(|| this.model.set_number(&this.name, value));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

struct ScriptedPropertyString {
    model: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedPropertyWatch>,
}

struct ScriptedPropertyBoolean {
    model: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedPropertyWatch>,
}

impl ScriptedPropertyBoolean {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let watch = property_watch(&model, &name);
        Self { model, name, watch }
    }
}

impl UserData for ScriptedPropertyBoolean {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(this.model.boolean(&this.name).unwrap_or_default())
        });
        fields.add_field_method_set("value", |_, this, value: bool| {
            this.model
                .defer_property_change_callbacks(|| this.model.set_boolean(&this.name, value));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

impl ScriptedPropertyString {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let watch = property_watch(&model, &name);
        Self { model, name, watch }
    }
}

impl UserData for ScriptedPropertyString {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(this.model.string(&this.name).unwrap_or_default())
        });
        fields.add_field_method_set("value", |lua, this, value: Value| {
            let value = super::lua_data_value::checked_string(lua, value)?;
            this.model.defer_property_change_callbacks(|| {
                this.model
                    .set_string(&this.name, &String::from_utf8_lossy(&value))
            });
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

struct ScriptedPropertyEnum {
    model: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedPropertyWatch>,
}

impl ScriptedPropertyEnum {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let watch = property_watch(&model, &name);
        Self { model, name, watch }
    }
}

impl UserData for ScriptedPropertyEnum {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(this.model.enum_value(&this.name).unwrap_or_default())
        });
        fields.add_field_method_set("value", |lua, this, value: Value| {
            let value = super::lua_data_value::checked_string(lua, value)?;
            this.model.defer_property_change_callbacks(|| {
                this.model
                    .set_enum_value(&this.name, &String::from_utf8_lossy(&value))
            });
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("values", |lua, this, ()| {
            create_scripted_enum_values(lua, this.model.enum_values(&this.name).unwrap_or_default())
        });
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

struct ScriptedEnumValues {
    values: Vec<String>,
}

impl UserData for ScriptedEnumValues {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__len", |_, this, ()| Ok(this.values.len()));
    }
}

fn create_scripted_enum_values(lua: &Lua, values: Vec<String>) -> luaur_rt::Result<AnyUserData> {
    let values = lua.create_userdata(ScriptedEnumValues { values })?;
    let metatable: Table = unsafe {
        lua.exec_raw(Value::UserData(values.clone()), |state| {
            let has_metatable = lua_getmetatable(state, 1);
            debug_assert_ne!(has_metatable, 0);
        })?
    };
    metatable.set(
        "__index",
        lua.create_function(move |lua, (values, key): (AnyUserData, Value)| match key {
            Value::Integer(index) => Ok(usize::try_from(index)
                .ok()
                .and_then(|index| index.checked_sub(1))
                .and_then(|index| {
                    values
                        .borrow::<ScriptedEnumValues>()
                        .ok()?
                        .values
                        .get(index)
                        .cloned()
                })
                .map(|value| Value::String(lua.create_string(value)))
                .unwrap_or(Value::Nil)),
            Value::Number(index) if index.fract() == 0.0 => {
                let index = if index >= 1.0 && index <= usize::MAX as f64 {
                    Some(index as usize - 1)
                } else {
                    None
                };
                Ok(index
                    .and_then(|index| {
                        values
                            .borrow::<ScriptedEnumValues>()
                            .ok()?
                            .values
                            .get(index)
                            .cloned()
                    })
                    .map(|value| Value::String(lua.create_string(value)))
                    .unwrap_or(Value::Nil))
            }
            Value::Number(_) => Err(luaur_rt::Error::runtime("integer expected")),
            _ => Ok(Value::Nil),
        })?,
    )?;
    Ok(values)
}

struct ScriptedPropertyImage {
    model: ScriptViewModel,
    name: String,
    cached_value: Rc<RefCell<Option<AnyUserData>>>,
    _change_sink: RuntimeCellDirtSink,
    watch: Rc<ScriptedPropertyWatch>,
}

struct ScriptedBlobWatch {
    sink: RuntimeCellDirtSink,
    listeners: RefCell<Vec<ScriptedListener>>,
    _change_registration: RefCell<Option<ScriptViewModelChangeRegistration>>,
}

struct ScriptedPropertyBlob {
    model: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedBlobWatch>,
    cached_value: Rc<RefCell<Option<AnyUserData>>>,
    _change_sink: RuntimeCellDirtSink,
}

impl ScriptedPropertyBlob {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let sink = model.property_dirt_sink(&name).unwrap_or_default();
        let cached_value = Rc::new(RefCell::new(None));
        let weak_cached_value = Rc::downgrade(&cached_value);
        let change_sink = model
            .property_change_sink(&name, move || {
                if let Some(cached_value) = weak_cached_value.upgrade() {
                    cached_value.borrow_mut().take();
                }
            })
            .unwrap_or_default();
        let watch = Rc::new(ScriptedBlobWatch {
            sink,
            listeners: RefCell::new(Vec::new()),
            _change_registration: RefCell::new(None),
        });
        let weak_watch = Rc::downgrade(&watch);
        let registration = model.add_property_change_callback(
            &name,
            Rc::new(move || {
                let Some(watch) = weak_watch.upgrade() else {
                    return;
                };
                if watch.sink.take_dirt().is_empty() {
                    return;
                }
                call_property_listeners(&watch.listeners);
            }),
        );
        *watch._change_registration.borrow_mut() = registration;
        Self {
            model,
            name,
            watch,
            cached_value,
            _change_sink: change_sink,
        }
    }
}

impl UserData for ScriptedPropertyBlob {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            if let Some(cached) = this.cached_value.borrow().as_ref() {
                return Ok(Value::UserData(cached.clone()));
            }
            let Some(asset) = this.model.blob_asset(&this.name) else {
                return Ok(Value::Nil);
            };
            let value = lua.create_userdata(ScriptedBlob::from_asset(asset))?;
            *this.cached_value.borrow_mut() = Some(value.clone());
            Ok(Value::UserData(value))
        });
        fields.add_field_method_set("value", |_, this, value: Value| {
            let asset = match value {
                Value::Nil => None,
                Value::String(value) => Some(Arc::new(RuntimeBlobAsset::new(
                    "",
                    Arc::<[u8]>::from(value.as_bytes()),
                ))),
                Value::Buffer(value) => Some(Arc::new(RuntimeBlobAsset::new(
                    "",
                    Arc::<[u8]>::from(value.to_vec()),
                ))),
                Value::UserData(value) => {
                    let blob = value.borrow::<ScriptedBlob>()?;
                    Some(blob.asset())
                }
                _ => {
                    return Err(luaur_rt::Error::runtime(
                        "expected Blob, string, buffer, or nil",
                    ));
                }
            };
            this.model
                .defer_property_change_callbacks(|| this.model.set_blob_asset(&this.name, asset));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
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
            this.watch
                .listeners
                .borrow_mut()
                .push(ScriptedListener { callback, userdata });
            ScriptViewModelFrameContext::for_lua(lua).register_blob_watch(&this.watch);
            Ok(())
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            let args = args.into_vec();
            let callback = match args.as_slice() {
                [Value::Function(callback)] | [_, Value::Function(callback)] => callback,
                _ => {
                    return Err(luaur_rt::Error::runtime(
                        "removeListener expects a callback or userdata and callback",
                    ));
                }
            };
            let identity = callback.to_pointer();
            this.watch
                .listeners
                .borrow_mut()
                .retain(|listener| listener.callback.to_pointer() != identity);
            Ok(())
        });
    }
}

impl ScriptedPropertyImage {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let cached_value = Rc::new(RefCell::new(None));
        let weak_cached_value = Rc::downgrade(&cached_value);
        let change_sink = model
            .property_change_sink(&name, move || {
                if let Some(cached_value) = weak_cached_value.upgrade() {
                    cached_value.borrow_mut().take();
                }
            })
            .unwrap_or_default();
        let watch = property_watch(&model, &name);
        Self {
            model,
            name,
            cached_value,
            _change_sink: change_sink,
            watch,
        }
    }
}

impl UserData for ScriptedPropertyImage {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            if let Some(cached) = this.cached_value.borrow().as_ref() {
                return Ok(Value::UserData(cached.clone()));
            }
            let value = if let Some(image) = this.model.render_image(&this.name) {
                Some(lua.create_userdata(ScriptedImage::from_render_image_rc(image))?)
            } else {
                this.model
                    .image(&this.name)
                    .map(|image| create_asset_image(lua, image))
                    .transpose()?
                    .flatten()
            };
            let Some(value) = value else {
                return Ok(Value::Nil);
            };
            *this.cached_value.borrow_mut() = Some(value.clone());
            Ok(Value::UserData(value))
        });
        fields.add_field_method_set("value", |_, this, value: Value| {
            match value {
                Value::Nil => {
                    this.model.defer_property_change_callbacks(|| {
                        this.model.set_render_image(&this.name, None)
                    });
                }
                Value::UserData(image) => {
                    let image = image.borrow::<ScriptedImage>()?;
                    let image = image.render_image()?;
                    this.model.defer_property_change_callbacks(|| {
                        this.model.set_render_image(&this.name, Some(image))
                    });
                }
                _ => return Err(luaur_rt::Error::runtime("expected Image userdata or nil")),
            }
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

struct ScriptedPropertyFont {
    model: ScriptViewModel,
    name: String,
    cached_value: Rc<RefCell<Option<AnyUserData>>>,
    _change_sink: RuntimeCellDirtSink,
    watch: Rc<ScriptedPropertyWatch>,
}

impl ScriptedPropertyFont {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let cached_value = Rc::new(RefCell::new(None));
        let weak_cached_value = Rc::downgrade(&cached_value);
        let change_sink = model
            .property_change_sink(&name, move || {
                if let Some(cached_value) = weak_cached_value.upgrade() {
                    cached_value.borrow_mut().take();
                }
            })
            .unwrap_or_default();
        let watch = property_watch(&model, &name);
        Self {
            model,
            name,
            cached_value,
            _change_sink: change_sink,
            watch,
        }
    }
}

impl UserData for ScriptedPropertyFont {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            if let Some(cached) = this.cached_value.borrow().as_ref() {
                return Ok(Value::UserData(cached.clone()));
            }
            let value = this
                .model
                .font(&this.name)
                .map(|font| create_asset_font(lua, font))
                .transpose()?
                .flatten();
            let Some(value) = value else {
                return Ok(Value::Nil);
            };
            *this.cached_value.borrow_mut() = Some(value.clone());
            Ok(Value::UserData(value))
        });
        fields.add_field_method_set("value", |_, this, value: Value| {
            let font_bytes = match value {
                Value::Nil => None,
                Value::UserData(font) => Some(font.borrow::<ScriptedFont>()?.font_bytes()),
                _ => return Err(luaur_rt::Error::runtime("expected Font userdata or nil")),
            };
            this.model.defer_property_change_callbacks(|| {
                this.model.set_font_bytes(&this.name, font_bytes)
            });
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

struct ScriptedPropertyList {
    model: ScriptViewModel,
    name: String,
    item_refs: BTreeMap<ViewModelInstanceKey, Table>,
    watch: Rc<ScriptedPropertyWatch>,
}

impl ScriptedPropertyList {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let watch = property_watch(&model, &name);
        Self {
            model,
            name,
            item_refs: BTreeMap::new(),
            watch,
        }
    }

    fn retain_current_item_refs(&mut self) {
        let current = (0..self.model.list_len(&self.name).unwrap_or_default())
            .filter_map(|index| self.model.list_item(&self.name, index))
            .map(|item| instance_key(&item.owned_instance()))
            .collect::<BTreeSet<_>>();
        self.item_refs.retain(|key, _| current.contains(key));
    }

    fn item_value(&mut self, lua: &Lua, index: usize) -> luaur_rt::Result<Value> {
        self.retain_current_item_refs();
        let item = self.model.list_item(&self.name, index);
        match item {
            // Registration of the parent table synchronizes all current list
            // edges. Do not add an explicit edge here: removing the item must
            // make a retained wrapper detached immediately.
            Some(item) => {
                let key = instance_key(&item.owned_instance());
                if let Some(table) = self.item_refs.get(&key) {
                    return Ok(Value::Table(table.clone()));
                }
                let table = create_scripted_view_model(lua, item)?;
                self.item_refs.insert(key, table.clone());
                Ok(Value::Table(table))
            }
            None => Ok(Value::Nil),
        }
    }
}

fn create_scripted_property_list(
    lua: &Lua,
    model: ScriptViewModel,
    name: String,
) -> luaur_rt::Result<AnyUserData> {
    let property = lua.create_userdata(ScriptedPropertyList::new(model, name))?;

    // luaur-rt synthesizes an `__index` dispatcher for registered fields and
    // methods, overwriting a UserData-provided `__index` metamethod. Preserve
    // that dispatcher for `length` and method lookup, then layer C++'s numeric
    // list indexing in front of it on this userdata instance.
    let metatable: Table = unsafe {
        lua.exec_raw(Value::UserData(property.clone()), |state| {
            let has_metatable = lua_getmetatable(state, 1);
            debug_assert_ne!(has_metatable, 0);
        })?
    };
    if !metatable
        .get::<bool>("__riveListIndexPatched")
        .unwrap_or(false)
    {
        let fallback: Function = metatable.get("__index")?;
        let index = lua.create_function(
            move |lua, (property, key): (AnyUserData, Value)| match key {
                Value::Integer(index) => usize::try_from(index)
                    .ok()
                    .and_then(|index| index.checked_sub(1))
                    .map_or(Ok(Value::Nil), |index| {
                        property
                            .borrow_mut::<ScriptedPropertyList>()?
                            .item_value(lua, index)
                    }),
                Value::Number(index) if index.fract() == 0.0 => {
                    if index < 1.0 || index > usize::MAX as f64 {
                        return Ok(Value::Nil);
                    }
                    property
                        .borrow_mut::<ScriptedPropertyList>()?
                        .item_value(lua, index as usize - 1)
                }
                Value::Number(_) => Err(luaur_rt::Error::runtime("integer expected")),
                key => fallback.call((Value::UserData(property), key)),
            },
        )?;
        metatable.set("__index", index)?;
        metatable.set("__riveListIndexPatched", true)?;
    }
    patch_property_userdata(lua, property, false, LIST_MUTATING_METHODS)
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
            this.model
                .defer_property_change_callbacks(|| this.model.push_list_item(&this.name, &item));
            Ok(())
        });
        methods.add_method("insert", |_, this, (item, index): (Table, usize)| {
            let item = model_from_table(&item)?;
            this.model.defer_property_change_callbacks(|| {
                this.model
                    .insert_list_item(&this.name, index.saturating_sub(1), &item)
            });
            Ok(())
        });
        methods.add_method("pop", |lua, this, ()| {
            let item = this
                .model
                .defer_property_change_callbacks(|| this.model.pop_list_item(&this.name));
            match item {
                Some(item) => create_scripted_view_model(lua, item).map(Value::Table),
                None => Ok(Value::Nil),
            }
        });
        methods.add_method("shift", |lua, this, ()| {
            let item = this
                .model
                .defer_property_change_callbacks(|| this.model.shift_list_item(&this.name));
            match item {
                Some(item) => create_scripted_view_model(lua, item).map(Value::Table),
                None => Ok(Value::Nil),
            }
        });
        methods.add_method("swap", |_, this, (first, second): (usize, usize)| {
            this.model.defer_property_change_callbacks(|| {
                this.model.swap_list_items(
                    &this.name,
                    first.saturating_sub(1),
                    second.saturating_sub(1),
                )
            });
            Ok(())
        });
        methods.add_method("clear", |_, this, ()| {
            this.model
                .defer_property_change_callbacks(|| this.model.clear_list_items(&this.name));
            Ok(())
        });
        methods.add_method("remove", |_, this, item: Value| {
            let Value::Table(item) = item else {
                return Ok(());
            };
            let Ok(item) = model_from_table(&item) else {
                return Ok(());
            };
            this.model.defer_property_change_callbacks(|| {
                this.model.remove_list_item(&this.name, &item, false)
            });
            Ok(())
        });
        methods.add_method("removeAt", |_, this, index: usize| {
            let Some(index) = index.checked_sub(1) else {
                return Err(luaur_rt::Error::runtime("removeAt index out of range"));
            };
            if !this.model.defer_property_change_callbacks(|| {
                this.model.remove_list_item_at(&this.name, index)
            }) {
                return Err(luaur_rt::Error::runtime("removeAt index out of range"));
            }
            Ok(())
        });
        methods.add_method("removeAllOf", |_, this, item: Value| {
            let Value::Table(item) = item else {
                return Ok(());
            };
            let Ok(item) = model_from_table(&item) else {
                return Ok(());
            };
            this.model.defer_property_change_callbacks(|| {
                this.model.remove_list_item(&this.name, &item, true)
            });
            Ok(())
        });
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
            add_property_listener(lua, &this.watch, args)
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            remove_property_listener(&this.watch, args)
        });
    }
}

pub(super) struct ScriptedContext {
    model: Rc<RefCell<Option<ScriptViewModel>>>,
    context_present: Rc<Cell<bool>>,
    parents: Vec<Option<ScriptViewModel>>,
    missing_requested_data: Rc<Cell<bool>>,
    gpu_canvas: Option<crate::gpu_canvas::GpuCanvasContextBindings>,
    alive: Rc<Cell<bool>>,
}

impl ScriptedContext {
    pub(super) fn new(
        model: Rc<RefCell<Option<ScriptViewModel>>>,
        parents: Vec<Option<ScriptViewModel>>,
        missing_requested_data: Rc<Cell<bool>>,
        gpu_canvas: Option<crate::gpu_canvas::GpuCanvasContextBindings>,
    ) -> Self {
        let context_present = Rc::new(Cell::new(model.borrow().is_some()));
        Self::new_with_lifetime(
            model,
            context_present,
            parents,
            missing_requested_data,
            gpu_canvas,
            Rc::new(Cell::new(true)),
        )
    }

    pub(super) fn new_with_lifetime(
        model: Rc<RefCell<Option<ScriptViewModel>>>,
        context_present: Rc<Cell<bool>>,
        parents: Vec<Option<ScriptViewModel>>,
        missing_requested_data: Rc<Cell<bool>>,
        gpu_canvas: Option<crate::gpu_canvas::GpuCanvasContextBindings>,
        alive: Rc<Cell<bool>>,
    ) -> Self {
        Self {
            model,
            context_present,
            parents,
            missing_requested_data,
            gpu_canvas,
            alive,
        }
    }

    pub(super) fn set_parents(&mut self, parents: Vec<Option<ScriptViewModel>>) {
        self.parents = parents;
    }

    fn require_live(&self, method: &str) -> luaur_rt::Result<()> {
        if self.alive.get() {
            Ok(())
        } else {
            Err(luaur_rt::Error::runtime(format!(
                "context:{method}() called on a disposed context — the context passed to init() must not be used after init() returns",
            )))
        }
    }
}

impl UserData for ScriptedContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("viewModel", |lua, this, ()| {
            this.require_live("viewModel")?;
            Ok(match this.model.borrow().clone() {
                Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                None => {
                    this.missing_requested_data.set(true);
                    Value::Nil
                }
            })
        });
        methods.add_method("rootViewModel", |lua, this, ()| {
            this.require_live("rootViewModel")?;
            let root = this
                .parents
                .last()
                .cloned()
                .unwrap_or_else(|| this.model.borrow().clone());
            Ok(match root {
                Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                None => {
                    this.missing_requested_data.set(true);
                    Value::Nil
                }
            })
        });
        methods.add_method("dataContext", |lua, this, ()| {
            this.require_live("dataContext")?;
            if !this.context_present.get() {
                this.missing_requested_data.set(true);
                return Ok(Value::Nil);
            }
            lua.create_userdata(ScriptedDataContext {
                model: this.model.borrow().clone(),
                parents: this.parents.clone(),
            })
            .map(Value::UserData)
        });
        methods.add_method("markNeedsUpdate", |_, this, ()| {
            this.require_live("markNeedsUpdate")?;
            // Base ScriptedObject deliberately owns no component dirt target.
            // Listener actions and data converters therefore accept this API
            // as a live no-op; component-derived scripted owners override it
            // on the C++ side (`lua_scripted_context.cpp:188-210`;
            // `scripted_object.cpp:556`).
            Ok(())
        });
        methods.add_method("image", |lua, this, name: String| {
            this.require_live("image")?;
            let Some(model) = this.model.borrow().clone() else {
                this.missing_requested_data.set(true);
                return Ok(Value::Nil);
            };
            Ok(match model.image_asset_named(&name) {
                Some(image) => create_asset_image(lua, image)?
                    .map(Value::UserData)
                    .unwrap_or(Value::Nil),
                None => Value::Nil,
            })
        });
        methods.add_method("blob", |lua, this, name: String| {
            this.require_live("blob")?;
            ScriptedBlobAssets::lookup(lua, &name)
        });
        methods.add_method("decodeImage", |lua, this, encoded: Buffer| {
            this.require_live("decodeImage")?;
            super::lua_image_decode::start(lua, encoded)
        });
        methods.add_method("audio", |lua, this, name: String| {
            this.require_live("audio")?;
            super::lua_audio::ScriptedAudioAssets::lookup(lua, &name)
        });
        methods.add_method("canvas", |_, this, _: MultiValue| {
            this.require_live("canvas")?;
            Err::<Value, _>(luaur_rt::Error::runtime(
                "unsupported: scripted-context-canvas binding is unavailable",
            ))
        });
        methods.add_method("gpuCanvas", |lua, this, descriptor: Option<Table>| {
            this.require_live("gpuCanvas")?;
            let gpu_canvas = this
                .gpu_canvas
                .as_ref()
                .ok_or_else(|| luaur_rt::Error::runtime("GPU-canvas context is unavailable"))?;
            let (width, height) = match descriptor {
                Some(descriptor) => (
                    descriptor.get::<Option<u32>>("width")?.unwrap_or(0),
                    descriptor.get::<Option<u32>>("height")?.unwrap_or(0),
                ),
                None => (0, 0),
            };
            gpu_canvas.canvas_userdata_with_size(lua, width, height)
        });
        methods.add_method("features", |lua, this, ()| {
            this.require_live("features")?;
            // The Rust renderer seam deliberately exposes the same
            // conservative no-render-context defaults used by pinned C++.
            // `lua_scripted_context.cpp:84-122`.
            let features = lua.create_table();
            for name in [
                "bc",
                "etc2",
                "astc",
                "anisotropicFiltering",
                "texture3D",
                "textureArrays",
                "colorBufferFloat",
                "colorBufferHalfFloat",
                "perTargetBlend",
                "perTargetWriteMask",
                "drawBaseInstance",
                "depthBiasClamp",
            ] {
                features.set(name, false)?;
            }
            for (name, value) in [
                ("maxTextureSize2D", 4096_u32),
                ("maxTextureSizeCube", 4096),
                ("maxTextureSize3D", 256),
                ("maxColorAttachments", 4),
                ("maxUniformBufferSize", 16_384),
                ("maxSamplers", 16),
                ("maxSamples", 4),
            ] {
                features.set(name, value)?;
            }
            features.set_readonly(true);
            Ok(features)
        });
        methods.add_method("shader", |lua, this, name: String| {
            this.require_live("shader")?;
            let gpu_canvas = this
                .gpu_canvas
                .as_ref()
                .ok_or_else(|| luaur_rt::Error::runtime("GPU-canvas context is unavailable"))?;
            gpu_canvas.shader_userdata(lua, name)
        });
    }
}

struct ScriptedDataContext {
    model: Option<ScriptViewModel>,
    parents: Vec<Option<ScriptViewModel>>,
}

impl UserData for ScriptedDataContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("viewModel", |lua, this, ()| match this.model.clone() {
            Some(model) => create_scripted_view_model(lua, model).map(Value::Table),
            None => Ok(Value::Nil),
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
            lua.create_function(move |lua, args: MultiValue| {
                let name = if args.len() == 1 {
                    match args.front() {
                        Some(Value::Nil) => None,
                        Some(value @ (Value::String(_) | Value::Integer(_) | Value::Number(_))) => {
                            let value: luaur_rt::LuaString = lua.unpack(value.clone())?;
                            Some(value.to_string_lossy())
                        }
                        Some(_) => return Ok(Value::Nil),
                        None => unreachable!("one argument disappeared"),
                    }
                } else {
                    None
                };
                let instance = model
                    .named_instance(name.as_deref())
                    .or_else(|| model.named_instance(None))
                    .ok_or_else(|| luaur_rt::Error::runtime("view-model instance not found"))?;
                create_scripted_view_model(lua, instance).map(Value::Table)
            })?,
        )?;
        data.set(name.as_str(), definition)?;
    }
    lua.globals().set("Data", data)
}

struct ScriptedTriggerWatch {
    sink: RuntimeCellDirtSink,
    listeners: RefCell<Vec<ScriptedListener>>,
    _change_registration: RefCell<Option<ScriptViewModelChangeRegistration>>,
}

struct ScriptedPropertyTrigger {
    model: ScriptViewModel,
    name: String,
    watch: Rc<ScriptedTriggerWatch>,
}

impl ScriptedPropertyTrigger {
    fn new(model: ScriptViewModel, name: String) -> Self {
        let sink = model.property_dirt_sink(&name).unwrap_or_default();
        let watch = Rc::new(ScriptedTriggerWatch {
            sink,
            listeners: RefCell::new(Vec::new()),
            _change_registration: RefCell::new(None),
        });
        let weak_watch = Rc::downgrade(&watch);
        let registration = model.add_property_change_callback(
            &name,
            Rc::new(move || {
                let Some(watch) = weak_watch.upgrade() else {
                    return;
                };
                if watch.sink.take_dirt().is_empty() {
                    return;
                }
                call_property_listeners(&watch.listeners);
            }),
        );
        *watch._change_registration.borrow_mut() = registration;
        Self { model, name, watch }
    }
}

#[derive(Clone)]
struct ScriptedListener {
    callback: Function,
    userdata: Option<Value>,
}

impl UserData for ScriptedPropertyTrigger {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("addListener", |lua, this, args: MultiValue| {
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
            this.watch
                .listeners
                .borrow_mut()
                .push(ScriptedListener { callback, userdata });
            ScriptViewModelFrameContext::for_lua(lua).register_trigger_watch(&this.watch);
            Ok(())
        });
        methods.add_method_mut("removeListener", |_, this, args: MultiValue| {
            let args = args.into_vec();
            let callback = match args.as_slice() {
                [Value::Function(callback)] | [_, Value::Function(callback)] => callback,
                _ => {
                    return Err(luaur_rt::Error::runtime(
                        "removeListener expects a callback or userdata and callback",
                    ));
                }
            };
            let identity = callback.to_pointer();
            this.watch
                .listeners
                .borrow_mut()
                .retain(|listener| listener.callback.to_pointer() != identity);
            Ok(())
        });
        methods.add_method_mut("fire", |_, this, ()| {
            this.model
                .defer_property_change_callbacks(|| this.model.fire_trigger(&this.name));
            Ok(())
        });
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::super::{ScriptProgram, ScriptVm};
    use super::*;

    #[test]
    fn absent_context_values_mark_requested_data_missing() {
        let lua = Lua::new();
        let missing_requested_data = Rc::new(Cell::new(false));
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::clone(&missing_requested_data),
                None,
            ))
            .expect("scripted context");
        lua.globals()
            .set("context", context)
            .expect("context global");

        let values: Table = lua
            .load(
                r#"
                return {
                    context:viewModel(),
                    context:rootViewModel(),
                    context:dataContext(),
                }
                "#,
            )
            .eval()
            .expect("missing context values evaluate");

        assert_eq!(values.raw_len(), 0);
        assert!(missing_requested_data.get());
    }

    #[test]
    fn parent_data_context_without_a_view_model_remains_in_the_chain() {
        let lua = Lua::new();
        let parent = fixture_models()
            .into_values()
            .next()
            .expect("fixture parent view model");
        let missing_requested_data = Rc::new(Cell::new(false));
        let context = lua
            .create_userdata(ScriptedContext::new_with_lifetime(
                Rc::new(RefCell::new(None)),
                Rc::new(Cell::new(true)),
                vec![None, Some(parent)],
                Rc::clone(&missing_requested_data),
                None,
                Rc::new(Cell::new(true)),
            ))
            .expect("scripted context");
        lua.globals()
            .set("context", context)
            .expect("context global");

        let (
            has_context,
            has_local_model,
            has_parent,
            has_parent_model,
            has_grandparent,
            has_grandparent_model,
        ): (bool, bool, bool, bool, bool, bool) = lua
            .load(
                r#"
                local dataContext = context:dataContext()
                local parent = dataContext:parent()
                local grandparent = parent:parent()
                context:markNeedsUpdate()
                return dataContext ~= nil,
                    dataContext:viewModel() ~= nil,
                    parent ~= nil,
                    parent:viewModel() ~= nil,
                    grandparent ~= nil,
                    grandparent:viewModel() ~= nil
                "#,
            )
            .eval()
            .expect("attached empty context evaluates");

        assert!(has_context);
        assert!(!has_local_model);
        assert!(has_parent);
        assert!(!has_parent_model);
        assert!(has_grandparent);
        assert!(has_grandparent_model);
        assert!(
            !missing_requested_data.get(),
            "requesting the retained DataContext object itself is not missing data"
        );
    }

    #[test]
    fn mark_needs_update_is_a_live_noop_and_rejects_a_disposed_context() {
        let lua = Lua::new();
        let alive = Rc::new(Cell::new(true));
        let context = lua
            .create_userdata(ScriptedContext::new_with_lifetime(
                Rc::new(RefCell::new(None)),
                Rc::new(Cell::new(true)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
                Rc::clone(&alive),
            ))
            .expect("scripted context");
        lua.globals()
            .set("context", context)
            .expect("context global");
        lua.load("context:markNeedsUpdate()")
            .exec()
            .expect("base C++ ScriptedObject accepts the live no-op");
        alive.set(false);
        let error = lua
            .load("context:markNeedsUpdate()")
            .exec()
            .expect_err("escaped disposed Context must reject every method");
        assert!(error.to_string().contains("disposed context"));
    }

    #[test]
    fn context_features_match_the_pinned_headless_surface_and_are_readonly() {
        let lua = Lua::new();
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .expect("scripted context");
        lua.globals().set("context", context).unwrap();

        let values: Table = lua
            .load(
                "local f = context:features()\n\
                 assert(table.isfrozen(f))\n\
                 return { f.bc, f.texture3D, f.maxTextureSize2D, f.maxTextureSize3D, f.maxUniformBufferSize, f.maxSamples }",
            )
            .eval()
            .expect("headless feature table");
        assert!(!values.get::<bool>(1).unwrap());
        assert!(!values.get::<bool>(2).unwrap());
        assert_eq!(values.get::<u32>(3).unwrap(), 4096);
        assert_eq!(values.get::<u32>(4).unwrap(), 256);
        assert_eq!(values.get::<u32>(5).unwrap(), 16_384);
        assert_eq!(values.get::<u32>(6).unwrap(), 4);
    }

    #[test]
    fn context_gpu_canvas_accepts_the_upstream_descriptor_shape() {
        let lua = Lua::new();
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                Some(crate::gpu_canvas::GpuCanvasContextBindings::for_test()),
            ))
            .expect("scripted context");
        lua.globals().set("context", context).unwrap();

        let (width, height, deferred_width, retained_width): (u32, u32, u32, u32) = lua
            .load(
                "local sized = context:gpuCanvas({ width = 320, height = 180 })\n\
                 local width, height = sized.width, sized.height\n\
                 local deferred = context:gpuCanvas()\n\
                 return width, height, deferred.width, sized.width",
            )
            .eval()
            .expect("GPU-canvas descriptor");
        assert_eq!(
            (width, height, deferred_width, retained_width),
            (320, 180, 0, 320)
        );
    }

    #[test]
    fn context_canvas_names_the_missing_binding() {
        let lua = Lua::new();
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .expect("scripted context");
        lua.globals().set("context", context).unwrap();

        let error = lua
            .load("context:canvas({ width = 16, height = 16 })")
            .exec()
            .expect_err("2D canvas has no Rust backing surface");
        assert!(
            error
                .to_string()
                .contains("unsupported: scripted-context-canvas binding is unavailable"),
            "got: {error}"
        );
    }

    fn fixture_models_from(asset: &str) -> BTreeMap<String, ScriptViewModel> {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets")
            .join(asset);
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
        nuxie_runtime::script_view_models(&file)
    }

    fn fixture_models() -> BTreeMap<String, ScriptViewModel> {
        fixture_models_from("script_create_viewmodel_instance.riv")
    }

    fn model_with_property(kind: ScriptViewModelProperty) -> (ScriptViewModel, String) {
        model_with_property_from("script_create_viewmodel_instance.riv", kind)
    }

    fn model_with_property_from(
        asset: &str,
        kind: ScriptViewModelProperty,
    ) -> (ScriptViewModel, String) {
        fixture_models_from(asset)
            .into_values()
            .find_map(|model| {
                let name = model
                    .properties()
                    .iter()
                    .find_map(|(name, candidate)| (*candidate == kind).then(|| name.clone()))?;
                Some((model.named_instance(None)?, name))
            })
            .unwrap_or_else(|| panic!("fixture has no {kind:?} property"))
    }

    #[test]
    fn property_listeners_survive_userdata_gc_while_subscribed() {
        let (model, property_name) = model_with_property(ScriptViewModelProperty::Number);
        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).unwrap();
        lua.globals()
            .set("propertyName", property_name.clone())
            .unwrap();
        lua.load(
            "calls = 0\n\
                 observed = 0\n\
                 local property = model:getNumber(propertyName)\n\
                 property:addListener(property, function(value)\n\
                     calls += 1; observed = value.value\n\
                 end)\n\
                 property = nil",
        )
        .exec()
        .expect("listener subscribes without an external wrapper owner");
        lua.gc_collect().expect("userdata collection");
        assert!(model.set_number(&property_name, 99.0));
        let calls: i64 = lua
            .load("return calls")
            .eval()
            .expect("listener survives wrapper collection");
        assert_eq!(calls, 1);
        assert_eq!(lua.globals().get::<f32>("observed").unwrap(), 99.0);

        lua.load(
            "removedCalls = 0\n\
             local property = model:getNumber(propertyName)\n\
             local function removed() removedCalls += 1 end\n\
             property:addListener(removed)\n\
             property:removeListener(removed)\n\
             property = nil",
        )
        .exec()
        .expect("listener removes its userdata anchor");
        lua.gc_collect().expect("removed listener collection");
        assert!(model.set_number(&property_name, 100.0));
        assert_eq!(lua.globals().get::<i64>("removedCalls").unwrap(), 0);
    }

    #[test]
    fn scripted_string_boolean_and_enum_properties_match_upstream_luau_access() {
        let (number_model, number_name) = model_with_property(ScriptViewModelProperty::Number);
        let lua = Lua::new();
        let table =
            create_scripted_view_model(&lua, number_model.clone()).expect("scripted number model");
        lua.globals().set("model", table).unwrap();
        lua.globals()
            .set("propertyName", number_name.clone())
            .unwrap();
        let result: Table = lua
            .load(
                "local property = model:getNumber(propertyName)\n\
                 local calls = 0\n\
                 local observed = 0\n\
                 local removedCalls = 0\n\
                 local function removed() removedCalls += 1 end\n\
                 assert(property == model[propertyName])\n\
                 model[propertyName]:addListener(removed)\n\
                 model:getNumber(propertyName):removeListener(removed)\n\
                 property:addListener(property, function(value)\n\
                     calls += 1; observed = value.value\n\
                 end)\n\
                 property.unknown = 'ignored'\n\
                 assert(not pcall(function() property[{}] = 'rejected' end))\n\
                 property.value = '12.5'\n\
                 return { calls, observed, removedCalls }",
            )
            .eval()
            .expect("number property surface");
        assert_eq!(result.get::<i64>(1).unwrap(), 1);
        assert_eq!(result.get::<f32>(2).unwrap(), 12.5);
        assert_eq!(result.get::<i64>(3).unwrap(), 0);
        assert_eq!(number_model.number(&number_name), Some(12.5));

        let scenarios = [
            (
                "scripted_string.riv",
                ScriptViewModelProperty::String,
                "'Hello World'",
            ),
            (
                "scripted_boolean.riv",
                ScriptViewModelProperty::Boolean,
                "true",
            ),
        ];
        for (asset, kind, assigned) in scenarios {
            let (model, property_name) = model_with_property_from(asset, kind);
            let lua = Lua::new();
            let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
            lua.globals().set("model", table).unwrap();
            lua.globals()
                .set("propertyName", property_name.clone())
                .unwrap();
            let source = format!(
                "local property = model[propertyName]\n\
                 calls = 0\n\
                 observed = nil\n\
                 local function changed(value)\n\
                     assert(value == property and value.value == property.value)\n\
                     calls += 1; observed = value.value\n\
                 end\n\
                 property:addListener(property, changed)\n\
                 property.value = {assigned}\n\
                 return calls"
            );
            assert_eq!(lua.load(&source).eval::<i64>().unwrap(), 1, "{asset}");
            match kind {
                ScriptViewModelProperty::String => {
                    assert_eq!(model.string(&property_name).as_deref(), Some("Hello World"));
                    assert!(model.set_string(&property_name, "yoo"));
                    assert_eq!(lua.globals().get::<String>("observed").unwrap(), "yoo");
                }
                ScriptViewModelProperty::Boolean => {
                    assert_eq!(model.boolean(&property_name), Some(true));
                    assert!(model.set_boolean(&property_name, false));
                    assert!(!lua.globals().get::<bool>("observed").unwrap());
                }
                _ => unreachable!(),
            }
            assert_eq!(lua.globals().get::<i64>("calls").unwrap(), 2, "{asset}");
        }

        let (model, property_name) =
            model_with_property_from("scripted_enum.riv", ScriptViewModelProperty::Enum);
        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted enum model");
        lua.globals().set("model", table).unwrap();
        lua.globals()
            .set("propertyName", property_name.clone())
            .unwrap();
        let result: Table = lua
            .load(
                "local property = model:getEnum(propertyName)\n\
                 local values = property:values()\n\
                 local calls = 0\n\
                 property:addListener(property, function(value)\n\
                     calls += 1\n\
                 end)\n\
                 property.value = 'blue'\n\
                 property.value = 'orange'\n\
                 property.value = 'red'\n\
                 return { property.value, values[1], #values, calls }",
            )
            .eval()
            .expect("enum property surface");
        assert_eq!(result.get::<String>(1).unwrap(), "red");
        assert!(!result.get::<String>(2).unwrap().is_empty());
        assert!(result.get::<i64>(3).unwrap() > 0);
        assert_eq!(result.get::<i64>(4).unwrap(), 3);
        assert_eq!(model.enum_value(&property_name).as_deref(), Some("red"));
    }

    #[test]
    fn scripted_color_property_supports_direct_and_named_access() {
        let (model, color) = fixture_models_from("scripting_root_viewmodel.riv")
            .into_values()
            .find_map(|model| {
                let name = model.properties().iter().find_map(|(name, candidate)| {
                    (*candidate == ScriptViewModelProperty::Color).then(|| name.clone())
                })?;
                Some((model.named_instance(None)?, name))
            })
            .expect("fixture has a color property");
        let expected = model.color(&color).expect("authored color");
        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).expect("model global");
        lua.globals()
            .set("colorName", color.clone())
            .expect("color name global");

        let values: Table = lua
            .load(
                r#"
                return {
                    model[colorName].value,
                    model:getColor(colorName).value,
                }
                "#,
            )
            .eval()
            .expect("color reads");
        assert_eq!(values.get::<i64>(1).unwrap(), i64::from(expected));
        assert_eq!(values.get::<i64>(2).unwrap(), i64::from(expected));

        let calls: i64 = lua
            .load(
                "local property = model[colorName]\n\
                 colorCalls = 0\n\
                 observedColor = 0\n\
                 property:addListener(property, function(value)\n\
                     observedColor = value.value; colorCalls += 1\n\
                 end)\n\
                 property.value = -1\n\
                 return colorCalls",
            )
            .eval()
            .expect("color write and listener");
        assert_eq!(calls, 1);
        assert_eq!(model.color(&color), Some(0xffff_ffff));
        assert!(model.set_color(&color, 0xff10_1567));
        assert_eq!(lua.globals().get::<i64>("colorCalls").unwrap(), 2);
        assert_eq!(
            lua.globals().get::<i64>("observedColor").unwrap(),
            0xff10_1567
        );
    }

    #[test]
    fn data_constructor_matches_pinned_argument_count_and_type_dispatch() {
        let (model_name, model) = fixture_models()
            .into_iter()
            .next()
            .expect("fixture contains a view-model definition");
        let lua = Lua::new();
        install_data_global(&lua, &BTreeMap::from([(model_name.clone(), model)]))
            .expect("Data global installs");
        lua.globals()
            .set("modelName", model_name)
            .expect("model name global");

        let result: Table = lua
            .load(
                r#"
                local constructor = Data[modelName].new
                return {
                    zero = constructor() ~= nil,
                    nilArg = constructor(nil) ~= nil,
                    booleanArg = constructor(true) == nil,
                    tableArg = constructor({}) == nil,
                    numericName = constructor(123) ~= nil,
                    extraArgs = constructor("missing", true) ~= nil,
                }
                "#,
            )
            .eval()
            .expect("Data constructor scenario runs");

        for field in [
            "zero",
            "nilArg",
            "booleanArg",
            "tableArg",
            "numericName",
            "extraArgs",
        ] {
            assert!(result.get::<bool>(field).unwrap(), "{field}");
        }
    }

    #[test]
    fn unported_context_binding_reports_the_script_and_binding_names() {
        let vm = ScriptVm::new();
        let chunk = vm
            .load(
                "lt2-unported-animation.luau",
                r#"
                return function(context)
                    context:animation("missing")
                    return {}
                end
                "#,
            )
            .expect("spot-check script compiles");
        let generator: Function = chunk.call(()).expect("script returns a generator");
        let program = ScriptProgram { generator };

        let error = match vm.instantiate_registered_script_with_context(&program, None, Vec::new())
        {
            Ok(_) => panic!("an unported Context binding must fail loudly"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("lt2-unported-animation.luau"), "{error}");
        assert!(error.contains("animation"), "{error}");
    }

    #[test]
    fn registrations_retain_until_the_last_owner_and_then_stop_advancing() {
        let (model, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        let context = ScriptViewModelFrameContext::default();
        let first = context.register(&model);
        let second = context.register(&model);
        assert_eq!(context.registrations(&model), 2);

        assert!(model.fire_trigger(&trigger));
        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));

        drop(first);
        assert_eq!(context.registrations(&model), 1);
        assert!(model.fire_trigger(&trigger));
        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));

        drop(second);
        assert_eq!(context.registrations(&model), 0);
        assert!(model.fire_trigger(&trigger));
        assert!(!context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(1));
    }

    #[test]
    fn only_parentless_roots_advance_and_registered_roots_recurse_to_children() {
        let (parent, list) = model_with_property(ScriptViewModelProperty::List);
        let (child, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        assert!(parent.push_list_item(&list, &child));
        let context = ScriptViewModelFrameContext::default();
        let _child_registration = context.register(&child);

        assert!(child.fire_trigger(&trigger));
        assert!(!context.advance_detached());
        assert_eq!(child.trigger(&trigger), Some(1));

        let _parent_registration = context.register(&parent);
        assert!(context.advance_detached());
        assert_eq!(child.trigger(&trigger), Some(0));
    }

    #[test]
    fn view_model_property_assignment_replaces_the_frame_parent_edge() {
        let (parent, property_name) = model_with_property(ScriptViewModelProperty::ViewModel);
        let previous = parent
            .view_model(&property_name)
            .expect("authored child occurrence");
        assert!(previous.has_parents());
        let replacement = previous
            .named_instance(None)
            .expect("fresh replacement occurrence");
        assert!(!replacement.has_parents());
        let lua = Lua::new();
        let parent_table =
            create_scripted_view_model(&lua, parent.clone()).expect("scripted parent");
        let replacement_table =
            create_scripted_view_model(&lua, replacement.clone()).expect("replacement child");
        lua.globals()
            .set("parent", parent_table)
            .expect("parent global");
        lua.globals()
            .set("propertyName", property_name.clone())
            .expect("property name global");
        lua.globals()
            .set("replacement", replacement_table)
            .expect("replacement global");

        lua.load(
            "local property = parent[propertyName]\n\
             oldChild = property.value\n\
             assert(oldChild == property.value)\n\
             property.value = replacement\n\
             assert(property.value ~= oldChild and property.value == property.value)",
        )
        .exec()
        .expect("replace child through ScriptedPropertyViewModel");

        let old_table = lua.globals().get::<Table>("oldChild").expect("old child");
        let old = model_from_table(&old_table).expect("old child model");
        assert!(!old.has_parents());
        assert!(replacement.has_parents());

        let current = parent
            .view_model(&property_name)
            .expect("replacement remains reachable from parent");
        assert!(Rc::ptr_eq(
            &current.owned_instance(),
            &replacement.owned_instance()
        ));
        assert!(!Rc::ptr_eq(
            &old.owned_instance(),
            &replacement.owned_instance()
        ));
    }

    #[test]
    fn nested_view_model_property_mints_the_referenced_type() {
        let (parent, property_name) = model_with_property(ScriptViewModelProperty::ViewModel);
        let referenced = parent
            .view_model(&property_name)
            .expect("authored referenced child occurrence");
        assert_ne!(
            parent.properties(),
            referenced.properties(),
            "fixture must distinguish the owner and referenced schemas"
        );

        let lua = Lua::new();
        let parent_table =
            create_scripted_view_model(&lua, parent.clone()).expect("scripted parent");
        lua.globals().set("parent", parent_table).unwrap();
        lua.globals().set("propertyName", property_name).unwrap();
        let minted_table: Table = lua
            .load(
                "local property = parent:getViewModel(propertyName)\n\
                 local referenced = property.value\n\
                 return referenced:instance()",
            )
            .eval()
            .expect("nested instance creation");
        let minted = model_from_table(&minted_table).expect("minted nested model");

        assert_eq!(minted.properties(), referenced.properties());
        assert_ne!(minted.properties(), parent.properties());
    }

    #[test]
    fn detached_root_recurses_through_shared_list_instances() {
        let (parent, list) = model_with_property(ScriptViewModelProperty::List);
        let (child, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        assert!(parent.push_list_item(&list, &child));

        let context = ScriptViewModelFrameContext::default();
        let _parent_registration = context.register(&parent);
        assert!(child.fire_trigger(&trigger));
        assert!(context.advance_detached());
        assert_eq!(child.trigger(&trigger), Some(0));
    }

    #[test]
    fn runtime_list_parent_edges_change_without_a_scripting_rescan() {
        let (parent, list) = model_with_property(ScriptViewModelProperty::List);
        let (child, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        assert!(parent.push_list_item(&list, &child));

        let context = ScriptViewModelFrameContext::default();
        let _child_registration = context.register(&child);
        assert!(child.fire_trigger(&trigger));
        assert!(!context.advance_detached());
        assert_eq!(child.trigger(&trigger), Some(1));

        assert!(parent.remove_list_item(&list, &child, false));
        assert!(context.advance_detached());
        assert_eq!(child.trigger(&trigger), Some(0));
    }

    #[test]
    fn list_remove_ignores_nil_like_cpp() {
        let (model, list) = model_with_property(ScriptViewModelProperty::List);
        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model).expect("scripted model");
        lua.globals().set("model", table).expect("model global");
        lua.globals()
            .set("listName", list)
            .expect("list name global");

        lua.load(
            "local list = model:getList(listName)\n\
             list:remove(nil)\n\
             list:removeAllOf(nil)",
        )
        .exec()
        .expect("nil removals are no-ops");
    }

    #[test]
    fn list_numeric_index_returns_a_stable_removable_view_model() {
        let (model, list) = model_with_property(ScriptViewModelProperty::List);
        let (child, _) = model_with_property(ScriptViewModelProperty::String);
        assert!(model.push_list_item(&list, &child));

        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).expect("model global");
        lua.globals()
            .set("listName", list.clone())
            .expect("list name global");

        let stable: bool = lua
            .load(
                "local list = model:getList(listName)\n\
                 local first = list[1]\n\
                 local same = first ~= nil and list == model[listName] and first == list[1] and model[listName][1] ~= nil\n\
                 assert(not pcall(function() list.unknown = 1 end))\n\
                 list:remove(first)\n\
                 return same",
            )
            .eval()
            .expect("numeric list access");

        assert!(stable);
        assert_eq!(model.list_len(&list), Some(0));
    }

    #[test]
    fn scripted_trigger_fire_mutates_backing_model_and_reset_skips_listeners() {
        let (model, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        let lua = Lua::new();
        let context = ScriptViewModelFrameContext::for_lua(&lua);
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).expect("model global");
        lua.globals()
            .set("triggerName", trigger.clone())
            .expect("trigger name global");

        lua.load(
            r#"
            listenerCalls = 0
            local property = model:getTrigger(triggerName)
            assert(property ~= nil and property == model[triggerName])
            assert(not pcall(function() property.unknown = 1 end))
            property:addListener(function()
                listenerCalls += 100
                error("ignored listener failure")
            end)
            property:addListener(function()
                listenerCalls += 1
            end)
            property:fire()
            "#,
        )
        .exec()
        .expect("trigger script runs");

        assert_eq!(model.trigger(&trigger), Some(1));
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 101);
        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 101);

        lua.globals().set("model", Value::Nil).unwrap();
        lua.gc_collect().expect("collect scripted model wrapper");
        assert_eq!(context.registrations(&model), 0);
        assert!(model.fire_trigger(&trigger));
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 202);
        assert!(!context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(1));
    }

    #[test]
    fn host_trigger_mutation_notifies_lua_listener_before_frame_reset() {
        let (model, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        let lua = Lua::new();
        let context = ScriptViewModelFrameContext::for_lua(&lua);
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).expect("model global");
        lua.globals()
            .set("triggerName", trigger.clone())
            .expect("trigger name global");
        lua.load(
            "listenerCalls = 0\n\
             model:getTrigger(triggerName):addListener(function()\n\
                 listenerCalls += 1\n\
             end)",
        )
        .exec()
        .expect("listener registration");

        assert!(model.fire_trigger(&trigger));
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 1);
        assert!(context.advance_detached());
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 1);
        assert_eq!(model.trigger(&trigger), Some(0));
        assert!(!context.advance_detached());
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 1);
    }

    #[test]
    fn scripted_view_model_exposes_the_component_list_index() {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/list_index_script_access.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
        let model = nuxie_runtime::script_view_models(&file)
            .into_values()
            .find(|model| model.component_list_item_index().is_some())
            .expect("fixture has an item-index model");
        let expected = model.component_list_item_index().unwrap() as i64;
        let index_name = model
            .properties()
            .iter()
            .find_map(|(name, kind)| {
                (*kind == ScriptViewModelProperty::SymbolListIndex).then(|| name.clone())
            })
            .expect("fixture has a named item-index property");
        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).expect("model global");
        lua.globals()
            .set("indexName", index_name)
            .expect("index name global");

        let actual: Table = lua
            .load("return { model:getIndex(), model[indexName] }")
            .eval()
            .expect("index reads run");

        assert_eq!(actual.get::<i64>(1).unwrap(), expected);
        assert_eq!(actual.get::<i64>(2).unwrap(), expected);

        let next = expected + 1;
        assert!(
            model
                .owned_instance()
                .borrow_mut()
                .set_symbol_list_index_by_property_name(
                    lua.globals().get::<String>("indexName").unwrap().as_str(),
                    next as u64,
                )
        );
        let updated: Table = lua
            .load("return { model:getIndex(), model[indexName] }")
            .eval()
            .expect("updated index reads run");
        assert_eq!(updated.get::<i64>(1).unwrap(), next);
        assert_eq!(updated.get::<i64>(2).unwrap(), next);
    }

    #[test]
    fn scripted_images_round_trip_between_context_and_view_model_properties() {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/image_scripting_property_value.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
        let (view_model_name, instance_name) = file
            .view_models()
            .into_iter()
            .find_map(|view_model| {
                if !view_model
                    .properties
                    .iter()
                    .any(|property| property.type_name == "ViewModelPropertyAssetImage")
                {
                    return None;
                }
                Some((
                    view_model.object.string_property("name")?.to_owned(),
                    view_model
                        .instances
                        .first()?
                        .object
                        .string_property("name")?
                        .to_owned(),
                ))
            })
            .expect("fixture has an authored image view model");
        let definition = nuxie_runtime::script_view_models(&file)
            .remove(&view_model_name)
            .expect("script view model is registered");
        let model = definition
            .named_instance(Some(&instance_name))
            .expect("authored instance is selectable");
        let property_name = model
            .properties()
            .iter()
            .find_map(|(name, kind)| {
                (*kind == ScriptViewModelProperty::Image && model.image(name).is_some())
                    .then(|| name.clone())
            })
            .expect("authored instance has an image property");
        let current = model.image(&property_name).expect("property has an image");
        let (asset_name, expected, expected_global_id) = file
            .file_assets()
            .into_iter()
            .enumerate()
            .find_map(|(index, asset)| {
                let index = u64::try_from(index).ok()?;
                (asset.type_name == "ImageAsset" && index != current.file_asset_index()).then(
                    || {
                        (
                            asset.string_property("name").unwrap().to_owned(),
                            index,
                            asset.id,
                        )
                    },
                )
            })
            .expect("fixture has a replacement image");

        let mut factory = nuxie_render_api::RecordingFactory::new();
        let mut loader = |_: &nuxie_runtime::RuntimeFileAsset,
                          _: &[u8],
                          _: &mut dyn nuxie_render_api::Factory| false;
        let owners = nuxie_runtime::RuntimeFileAssetOwners::import_with_loader(
            &file,
            None,
            &mut factory,
            &mut loader,
        );
        let lua = Lua::new();
        crate::vm::lua_image::set_image_asset_owners(&lua, owners.image_assets());
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        let missing_requested_data = Rc::new(Cell::new(false));
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(Some(model.clone()))),
                Vec::new(),
                Rc::clone(&missing_requested_data),
                None,
            ))
            .expect("scripted context");
        lua.globals().set("model", table).unwrap();
        lua.globals().set("context", context).unwrap();
        lua.globals()
            .set("propertyName", property_name.clone())
            .unwrap();
        lua.globals().set("assetName", asset_name).unwrap();

        lua.load(
            "local property = model:getImage(propertyName)\n\
             assert(property ~= nil)\n\
             property.value = context:image(assetName)\n\
             local first = property.value\n\
             assert(first ~= nil and first == property.value)\n\
             imageProperty = property\n\
             savedImage = first",
        )
        .exec()
        .expect("image property script runs");

        assert!(model.image(&property_name).is_none());
        let retained = model
            .render_image(&property_name)
            .expect("asset-backed assignment retains the decoded image identity");
        let expected_image = owners
            .image_assets()
            .get(expected_global_id)
            .expect("replacement image was decoded");
        assert!(Rc::ptr_eq(&retained, &expected_image));
        assert_ne!(expected, current.file_asset_index());

        let mut png_header = vec![0; 24];
        png_header[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        png_header[12..16].copy_from_slice(b"IHDR");
        png_header[16..20].copy_from_slice(&3_u32.to_be_bytes());
        png_header[20..24].copy_from_slice(&5_u32.to_be_bytes());
        let factory_image: Rc<dyn nuxie_render_api::RenderImage> = Rc::from(
            nuxie_render_api::Factory::decode_image(&mut factory, &png_header)
                .expect("recording factory image"),
        );
        assert!(model.set_render_image(&property_name, Some(Rc::clone(&factory_image))));
        lua.globals()
            .set(
                "factoryImage",
                lua.create_userdata(ScriptedImage::from_render_image_rc(Rc::clone(
                    &factory_image,
                )))
                .unwrap(),
            )
            .unwrap();
        let dimensions: Table = lua
            .load(
                "local changed = imageProperty.value; \
                 local stableAfterHostWrite = changed == imageProperty.value; \
                 imageProperty.value = factoryImage; \
                 return { changed.width, changed.height, \
                          changed ~= savedImage, stableAfterHostWrite, \
                          changed == imageProperty.value }",
            )
            .eval()
            .expect("factory-backed image round trip");
        assert_eq!(dimensions.get::<u32>(1).unwrap(), 3);
        assert_eq!(dimensions.get::<u32>(2).unwrap(), 5);
        assert!(dimensions.get::<bool>(3).unwrap());
        assert!(dimensions.get::<bool>(4).unwrap());
        assert!(dimensions.get::<bool>(5).unwrap());
        let retained = model
            .render_image(&property_name)
            .expect("runtime view-model state retains the factory image");
        assert_eq!((retained.width(), retained.height()), (3, 5));
        assert!(!missing_requested_data.get());
    }

    #[test]
    fn root_view_model_preserves_a_terminal_context_without_a_model() {
        let lua = Lua::new();
        let local = fixture_models()
            .into_values()
            .next()
            .expect("fixture local view model");
        let nearer_parent = local
            .named_instance(None)
            .expect("fixture nearer parent view model");
        let missing_requested_data = Rc::new(Cell::new(false));
        let context = lua
            .create_userdata(ScriptedContext::new_with_lifetime(
                Rc::new(RefCell::new(Some(local))),
                Rc::new(Cell::new(true)),
                vec![Some(nearer_parent), None],
                Rc::clone(&missing_requested_data),
                None,
                Rc::new(Cell::new(true)),
            ))
            .expect("scripted context");
        lua.globals()
            .set("context", context)
            .expect("context global");

        let root_is_nil: bool = lua
            .load("return context:rootViewModel() == nil")
            .eval()
            .expect("root view model lookup");

        assert!(root_is_nil);
        assert!(missing_requested_data.get());
    }

    #[test]
    fn context_image_returns_nil_until_the_asset_has_a_render_resource() {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/image_scripting_property_value.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
        let model = nuxie_runtime::script_view_models(&file)
            .into_values()
            .next()
            .and_then(|definition| definition.named_instance(None))
            .expect("fixture has a script view model");
        let property_name = model
            .properties()
            .iter()
            .find_map(|(name, kind)| {
                (*kind == ScriptViewModelProperty::Image).then(|| name.clone())
            })
            .expect("fixture has an image property");
        let asset_name = file
            .file_assets()
            .into_iter()
            .find(|asset| asset.type_name == "ImageAsset")
            .and_then(|asset| asset.string_property("name"))
            .expect("fixture has an image")
            .to_owned();
        let lua = Lua::new();
        crate::vm::lua_image::set_image_asset_owners(
            &lua,
            std::sync::Arc::new(nuxie_runtime::RuntimeImageAssetOwners::default()),
        );
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(Some(model))),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .unwrap();
        lua.globals().set("model", table).unwrap();
        lua.globals().set("context", context).unwrap();
        lua.globals().set("propertyName", property_name).unwrap();
        lua.globals().set("assetName", asset_name).unwrap();

        assert!(
            lua.load(
                "return context:image(assetName) == nil \
                 and model:getImage(propertyName).value == nil",
            )
            .eval::<bool>()
            .unwrap()
        );
    }

    #[test]
    fn scripted_blob_property_reads_and_writes_bytes() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/sync/data_bind_blob_test.riv");
        let bytes = std::fs::read(path).expect("vendored blob fixture");
        let file = nuxie_binary::read_runtime_file(&bytes).expect("blob fixture parses");
        let model = nuxie_runtime::script_view_models(&file)
            .into_values()
            .find_map(|definition| {
                definition
                    .properties()
                    .values()
                    .any(|kind| *kind == ScriptViewModelProperty::Blob)
                    .then(|| definition.named_instance(None))
                    .flatten()
            })
            .expect("fixture exposes blob property");
        let property_name = model
            .properties()
            .iter()
            .find_map(|(name, kind)| (*kind == ScriptViewModelProperty::Blob).then(|| name.clone()))
            .expect("blob property name");
        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).unwrap();
        lua.globals()
            .set("propertyName", property_name.clone())
            .unwrap();

        let values: Table = lua
            .load(
                "local property = model:getBlob(propertyName)\n\
                 assert(property ~= nil)\n\
                 property.value = 'abcd'\n\
                 local firstValue = property.value\n\
                 local size = firstValue.size\n\
                 local first = buffer.readu8(firstValue.data, 0)\n\
                 local stable = firstValue == property.value\n\
                 property.value = ''\n\
                 local empty = property.value\n\
                 return { size, first, stable, firstValue ~= empty, empty ~= nil, empty.size, empty.data == nil }",
            )
            .eval()
            .expect("blob property script");

        assert_eq!(values.get::<usize>(1).unwrap(), 4);
        assert_eq!(values.get::<u8>(2).unwrap(), b'a');
        assert!(values.get::<bool>(3).unwrap());
        assert!(values.get::<bool>(4).unwrap());
        assert!(values.get::<bool>(5).unwrap());
        assert_eq!(values.get::<usize>(6).unwrap(), 0);
        assert!(values.get::<bool>(7).unwrap());
        assert_eq!(model.blob(&property_name).as_deref(), Some(&b""[..]));
    }

    #[test]
    fn scripted_blob_property_fires_listeners_only_when_identity_changes() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/sync/data_bind_blob_test.riv");
        let bytes = std::fs::read(path).expect("vendored blob fixture");
        let file = nuxie_binary::read_runtime_file(&bytes).expect("blob fixture parses");
        let model = nuxie_runtime::script_view_models(&file)
            .into_values()
            .find_map(|definition| {
                definition
                    .properties()
                    .values()
                    .any(|kind| *kind == ScriptViewModelProperty::Blob)
                    .then(|| definition.named_instance(None))
                    .flatten()
            })
            .expect("fixture exposes blob property");
        let property_name = model
            .properties()
            .iter()
            .find_map(|(name, kind)| (*kind == ScriptViewModelProperty::Blob).then(|| name.clone()))
            .expect("blob property name");
        let lua = Lua::new();
        let assets = ScriptedBlobAssets::install(&lua);
        assets
            .register("payload.bin", "payload.bin", b"abcd")
            .expect("blob registration");
        let source = ScriptedBlobAssets::lookup(&lua, "payload.bin").expect("blob lookup");
        let table = create_scripted_view_model(&lua, model).expect("scripted model");
        lua.globals().set("model", table).unwrap();
        lua.globals().set("propertyName", property_name).unwrap();
        lua.globals().set("source", source).unwrap();

        let values: Table = lua
            .load(
                "local property = model:getBlob(propertyName)\n\
                 local count = 0\n\
                 local function changed(_) count += 1 end\n\
                 local function throwing(_) error('ignored listener failure') end\n\
                 property:addListener(changed)\n\
                 property:addListener(throwing)\n\
                 property.value = source\n\
                 local same = property.value\n\
                 property.value = same\n\
                 property:removeListener(changed)\n\
                 property:removeListener(throwing)\n\
                 property.value = 'efgh'\n\
                 return { count, same.name }",
            )
            .eval()
            .expect("blob listener script");

        assert_eq!(values.get::<i64>(1).unwrap(), 1);
        assert_eq!(values.get::<String>(2).unwrap(), "payload.bin");
    }

    #[test]
    fn font_properties_retain_the_exact_font_owner_across_lua_assignment() {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/data_bind_font_test.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
        let (model, property_name) = nuxie_runtime::script_view_models(&file)
            .into_iter()
            .find_map(|(view_model_name, definition)| {
                let instance_names = file
                    .view_models()
                    .into_iter()
                    .find(|view_model| {
                        view_model.object.string_property("name") == Some(&view_model_name)
                    })?
                    .instances
                    .iter()
                    .filter_map(|instance| instance.object.string_property("name"))
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                instance_names.into_iter().find_map(|instance_name| {
                    let model = definition.named_instance(Some(&instance_name))?;
                    let property_name = model.properties().iter().find_map(|(name, kind)| {
                        (*kind == ScriptViewModelProperty::Font && model.font(name).is_some())
                            .then(|| name.clone())
                    })?;
                    Some((model, property_name))
                })
            })
            .expect("fixture has a file-backed font property");
        let font = model.font(&property_name).expect("font identity");
        let asset_global_id = font.asset_global_id().expect("file font asset identity");
        let owners =
            std::sync::Arc::new(nuxie_runtime::RuntimeFontAssetOwners::from_runtime(&file));
        let expected = owners
            .get(asset_global_id)
            .expect("fixture font was decoded");
        let lua = Lua::new();
        crate::vm::lua_font::set_font_asset_owners(&lua, owners);
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        lua.globals().set("model", table).unwrap();
        lua.globals()
            .set("propertyName", property_name.clone())
            .unwrap();
        lua.load(
            "local property = model:getFont(propertyName)\n\
             assert(property ~= nil and property.value ~= nil)\n\
             assert(model[propertyName].value ~= nil)\n\
             local first = property.value\n\
             assert(first == property.value)\n\
             fontProperty = property\n\
             savedFont = first",
        )
        .exec()
        .expect("getFont and direct font properties resolve");

        let host_font: std::sync::Arc<[u8]> = std::sync::Arc::from(expected.as_ref());
        assert!(model.set_font_bytes(&property_name, Some(host_font.clone())));
        lua.load(
            "local changed = fontProperty.value\n\
             assert(changed ~= savedFont and changed == fontProperty.value)\n\
             savedFont = changed",
        )
        .exec()
        .expect("host font replacement invalidates the cached wrapper");

        crate::vm::lua_font::set_font_asset_owners(
            &lua,
            std::sync::Arc::new(nuxie_runtime::RuntimeFontAssetOwners::default()),
        );
        lua.load(
            "fontProperty.value = nil\n\
             assert(fontProperty.value == nil)\n\
             fontProperty.value = savedFont\n\
             local changed = fontProperty.value\n\
             assert(changed ~= nil and changed ~= savedFont)\n\
             assert(changed == fontProperty.value)",
        )
        .exec()
        .expect("retained Font userdata remains assignable after registry replacement");

        let retained = model
            .owned_instance()
            .borrow()
            .font_asset_value_by_property_name(&property_name)
            .and_then(|value| value.live_font_bytes_arc().cloned())
            .expect("Lua assignment installed a live font owner");
        assert!(std::sync::Arc::ptr_eq(&retained, &host_font));
    }

    fn positive_blob_lookup_surface() -> String {
        let lua = Lua::new();
        let assets = ScriptedBlobAssets::install(&lua);
        assets
            .register("payload.bin", "payload.bin", &[])
            .expect("empty duplicate registration");
        assets
            .register("payload.bin", "payload.bin", &[0, 1, 2, 0xff])
            .expect("blob registration");
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .expect("scripted context");
        lua.globals().set("context", context).unwrap();

        let result: Table = lua
            .load(
                r#"
                local blob = context:blob("payload.bin")
                assert(blob ~= nil)
                local first = blob.data
                buffer.writeu8(first, 0, 99)
                local second = blob.data
                return {
                    blob.name,
                    blob.size,
                    buffer.len(first),
                    buffer.readu8(second, 0),
                    buffer.readu8(second, 3),
                    context:blob("missing") == nil,
                }
                "#,
            )
            .eval()
            .expect("positive Blob lookup");

        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            result.get::<String>(1).unwrap(),
            result.get::<usize>(2).unwrap(),
            result.get::<usize>(3).unwrap(),
            result.get::<u8>(4).unwrap(),
            result.get::<u8>(5).unwrap(),
            result.get::<bool>(6).unwrap(),
        )
    }

    #[test]
    fn context_blob_positive_lookup_matches_pinned_copy_surface() {
        assert_eq!(
            positive_blob_lookup_surface(),
            "payload.bin\t4\t4\t0\t255\ttrue\n"
        );
    }

    #[test]
    fn context_blob_prefers_caller_scope_and_exposes_the_authored_short_name() {
        let lua = Lua::new();
        let assets = ScriptedBlobAssets::install(&lua);
        assets.register("payload.bin", "payload.bin", &[1]).unwrap();
        assets
            .register("Effects#7@1/payload.bin", "payload.bin", &[2])
            .unwrap();
        assets
            .register("Effects#7@2/payload.bin", "payload.bin", &[3])
            .unwrap();
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .unwrap();
        lua.globals().set("context", context).unwrap();

        let result: Table = lua
            .load(
                "local blob = context:blob('payload.bin')\n\
                 return { blob.name, buffer.readu8(blob.data, 0) }",
            )
            .set_name("Effects#7@2/probe")
            .eval()
            .expect("caller-scoped blob lookup");

        assert_eq!(result.get::<String>(1).unwrap(), "payload.bin");
        assert_eq!(result.get::<u8>(2).unwrap(), 3);
    }

    #[test]
    #[ignore = "requires pinned C++ libraries; run `make blob-differential`"]
    fn context_blob_positive_lookup_matches_live_cpp_oracle() {
        let oracle = std::path::PathBuf::from(
            std::env::var_os("NUXIE_CPP_BLOB_ORACLE")
                .expect("NUXIE_CPP_BLOB_ORACLE is unset; run `make blob-differential`"),
        );
        assert!(
            oracle.is_file(),
            "C++ Blob oracle does not exist at {}",
            oracle.display()
        );
        let output = std::process::Command::new(&oracle)
            .arg("--blob-lookup-positive")
            .output()
            .expect("start C++ Blob oracle");
        assert!(
            output.status.success(),
            "C++ Blob oracle failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let cpp = String::from_utf8(output.stdout).expect("C++ Blob oracle UTF-8");
        assert_eq!(positive_blob_lookup_surface(), cpp);
    }
}
