use std::collections::{BTreeMap, BTreeSet};

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use luaur_rt::{
    AnyUserData, Buffer, Function, Lua, MultiValue, Table, UserData, UserDataFields,
    UserDataMethods, Value,
};
use luaur_vm::functions::lua_getmetatable::lua_getmetatable;
use nuxie_runtime::{RuntimeOwnedViewModelInstance, ScriptViewModel, ScriptViewModelProperty};

use super::lua_blob::ScriptedBlobAssets;
use super::lua_image::{ScriptedImage, create_asset_image};

type ViewModelInstance = Rc<RefCell<RuntimeOwnedViewModelInstance>>;
type ViewModelInstanceWeak = Weak<RefCell<RuntimeOwnedViewModelInstance>>;
type ViewModelInstanceKey = usize;

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
    parents: BTreeMap<ViewModelInstanceKey, ParentRelationship>,
}

struct ParentRelationship {
    instance: ViewModelInstanceWeak,
    explicit: bool,
    list: bool,
}

/// Per-VM equivalent of C++ `ScriptingContext`'s owner-counted detached VMI
/// registry. Relationships are weak; registrations alone retain instances.
#[derive(Clone, Default)]
pub(crate) struct ScriptViewModelFrameContext {
    tracked: Rc<RefCell<TrackedViewModels>>,
}

impl std::fmt::Debug for ScriptViewModelFrameContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptViewModelFrameContext")
            .field("tracked_instances", &self.tracked.borrow().instances.len())
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
                parents: BTreeMap::new(),
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
        self.sync_list_parents(model);
        ScriptViewModelRegistration {
            tracked: Rc::downgrade(&self.tracked),
            key,
        }
    }

    pub(crate) fn link_parent(&self, child: &ScriptViewModel, parent: &ScriptViewModel) {
        let child_instance = child.owned_instance();
        let parent_instance = parent.owned_instance();
        let parent_key = instance_key(&parent_instance);
        let mut tracked = self.tracked.borrow_mut();
        Self::ensure_entry(&mut tracked, &parent_instance);
        let child_entry = Self::ensure_entry(&mut tracked, &child_instance);
        child_entry
            .parents
            .entry(parent_key)
            .and_modify(|relationship| {
                relationship.instance = Rc::downgrade(&parent_instance);
                relationship.explicit = true;
            })
            .or_insert_with(|| ParentRelationship {
                instance: Rc::downgrade(&parent_instance),
                explicit: true,
                list: false,
            });
    }

    /// Mirror `ViewModelInstanceViewModel::referenceViewModelInstance` for
    /// the scripting registry's detached-root projection.
    ///
    /// The retained runtime owner performs the real remove/store/add sequence.
    /// This companion edge update keeps C++'s `hasParents()` classification
    /// visible to `advanceDetachedViewModels`: the replaced child becomes a
    /// detached root and the replacement becomes attached immediately.
    pub(crate) fn replace_explicit_parent(
        &self,
        previous: Option<&ScriptViewModel>,
        replacement: &ScriptViewModel,
        parent: &ScriptViewModel,
    ) {
        let parent_instance = parent.owned_instance();
        let parent_key = instance_key(&parent_instance);
        let mut tracked = self.tracked.borrow_mut();
        Self::ensure_entry(&mut tracked, &parent_instance);

        if let Some(previous) = previous {
            let previous_instance = previous.owned_instance();
            let previous_entry = Self::ensure_entry(&mut tracked, &previous_instance);
            let remove = previous_entry
                .parents
                .get_mut(&parent_key)
                .is_some_and(|relationship| {
                    relationship.explicit = false;
                    !relationship.list
                });
            if remove {
                previous_entry.parents.remove(&parent_key);
            }
        }

        let replacement_instance = replacement.owned_instance();
        let replacement_entry = Self::ensure_entry(&mut tracked, &replacement_instance);
        replacement_entry
            .parents
            .entry(parent_key)
            .and_modify(|relationship| {
                relationship.instance = Rc::downgrade(&parent_instance);
                relationship.explicit = true;
            })
            .or_insert_with(|| ParentRelationship {
                instance: Rc::downgrade(&parent_instance),
                explicit: true,
                list: false,
            });
    }

    pub(crate) fn sync_list_parents(&self, parent: &ScriptViewModel) {
        let parent_instance = parent.owned_instance();
        self.sync_list_parent_instance(&parent_instance);
    }

    fn sync_list_parent_instance(&self, parent_instance: &ViewModelInstance) {
        let parent_key = instance_key(parent_instance);
        let list_children = ScriptViewModel::owned_list_children(parent_instance)
            .into_iter()
            .map(|instance| (instance_key(&instance), instance))
            .collect::<BTreeMap<_, _>>();

        let mut tracked = self.tracked.borrow_mut();
        Self::ensure_entry(&mut tracked, parent_instance);
        for child in list_children.values() {
            Self::ensure_entry(&mut tracked, child);
        }
        for (child_key, child_entry) in &mut tracked.instances {
            let Some(child_instance) = list_children.get(child_key) else {
                let remove = child_entry
                    .parents
                    .get_mut(&parent_key)
                    .is_some_and(|relationship| {
                        relationship.list = false;
                        !relationship.explicit
                    });
                if remove {
                    child_entry.parents.remove(&parent_key);
                }
                continue;
            };
            child_entry
                .parents
                .entry(parent_key)
                .and_modify(|relationship| {
                    relationship.instance = Rc::downgrade(parent_instance);
                    relationship.list = true;
                })
                .or_insert_with(|| ParentRelationship {
                    instance: Rc::downgrade(parent_instance),
                    explicit: false,
                    list: true,
                });
            debug_assert!(Rc::ptr_eq(
                &child_entry.instance.upgrade().expect("live list child"),
                child_instance
            ));
        }
    }

    pub(crate) fn advance_detached(&self) -> bool {
        // Lists can also change through data binding or host APIs. Refresh all
        // live parent edges here, not only in Lua list methods, before deciding
        // which registered instances are detached roots.
        let live_instances = self
            .tracked
            .borrow()
            .instances
            .values()
            .filter_map(|entry| entry.instance.upgrade())
            .collect::<Vec<_>>();
        for instance in live_instances {
            self.sync_list_parent_instance(&instance);
        }

        let (instances, roots, children) = {
            let mut tracked = self.tracked.borrow_mut();
            tracked
                .instances
                .retain(|_, entry| entry.registrations > 0 || entry.instance.strong_count() > 0);
            for entry in tracked.instances.values_mut() {
                entry
                    .parents
                    .retain(|_, parent| parent.instance.strong_count() > 0);
            }

            let instances = tracked
                .instances
                .iter()
                .filter_map(|(key, entry)| {
                    entry.instance.upgrade().map(|instance| (*key, instance))
                })
                .collect::<BTreeMap<_, _>>();
            let roots = tracked
                .instances
                .iter()
                .filter_map(|(key, entry)| {
                    (entry.registrations > 0
                        && entry
                            .parents
                            .values()
                            .all(|parent| parent.instance.strong_count() == 0))
                    .then_some(*key)
                })
                .collect::<Vec<_>>();
            let mut children = BTreeMap::<ViewModelInstanceKey, Vec<ViewModelInstanceKey>>::new();
            for (child_key, entry) in &tracked.instances {
                for (parent_key, parent) in &entry.parents {
                    if parent.instance.strong_count() > 0 {
                        children.entry(*parent_key).or_default().push(*child_key);
                    }
                }
            }
            (instances, roots, children)
        };

        fn collect_registered(
            key: ViewModelInstanceKey,
            instances: &BTreeMap<ViewModelInstanceKey, ViewModelInstance>,
            children: &BTreeMap<ViewModelInstanceKey, Vec<ViewModelInstanceKey>>,
            visited: &mut BTreeSet<ViewModelInstanceKey>,
            ordered: &mut Vec<ViewModelInstance>,
        ) {
            if !visited.insert(key) {
                return;
            }
            if let Some(instance) = instances.get(&key) {
                ordered.push(Rc::clone(instance));
            }
            if let Some(child_keys) = children.get(&key) {
                for child_key in child_keys {
                    collect_registered(*child_key, instances, children, visited, ordered);
                }
            }
        }

        let mut visited = BTreeSet::new();
        let mut ordered = Vec::new();
        for root in roots {
            collect_registered(root, &instances, &children, &mut visited, &mut ordered);
        }
        ScriptViewModel::advance_owned_instances(&ordered)
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

    #[cfg(test)]
    fn has_explicit_parent(&self, child: &ScriptViewModel, parent: &ScriptViewModel) -> bool {
        let parent_key = instance_key(&parent.owned_instance());
        self.tracked
            .borrow()
            .instances
            .get(&instance_key(&child.owned_instance()))
            .and_then(|entry| entry.parents.get(&parent_key))
            .is_some_and(|relationship| relationship.explicit)
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
    create_scripted_view_model_with_parent(lua, model, None)
}

fn create_scripted_view_model_with_parent(
    lua: &Lua,
    model: ScriptViewModel,
    parent: Option<&ScriptViewModel>,
) -> luaur_rt::Result<Table> {
    let frame_context = ScriptViewModelFrameContext::for_lua(lua);
    if let Some(parent) = parent {
        frame_context.link_parent(&model, parent);
    }
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
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_number_model.property(&name) {
                Some(ScriptViewModelProperty::Number) => lua
                    .create_userdata(ScriptedPropertyNumber::new(get_number_model.clone(), name))
                    .map(Value::UserData),
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_color_model = model.clone();
    table.set(
        "getColor",
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_color_model.property(&name) {
                Some(ScriptViewModelProperty::Color) => lua
                    .create_userdata(ScriptedPropertyColor::new(get_color_model.clone(), name))
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
                Some(ScriptViewModelProperty::List) => {
                    create_scripted_property_list(lua, get_list_model.clone(), name)
                        .map(Value::UserData)
                }
                _ => Ok(Value::Nil),
            }
        })?,
    )?;
    let get_trigger_model = model.clone();
    table.set(
        "getTrigger",
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_trigger_model.property(&name) {
                Some(ScriptViewModelProperty::Trigger) => lua
                    .create_userdata(ScriptedPropertyTrigger::new(
                        get_trigger_model.clone(),
                        name,
                    ))
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
        lua.create_function(move |lua, (_self, name): (Table, String)| {
            match get_image_model.property(&name) {
                Some(ScriptViewModelProperty::Image) => lua
                    .create_userdata(ScriptedPropertyImage::new(get_image_model.clone(), name))
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
                Some(_) => Value::UserData(lua.create_userdata(ScriptedPropertyViewModel::new(
                    get_view_model.clone(),
                    name,
                ))?),
                None => Value::Nil,
            })
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
            ScriptViewModelProperty::Number => {
                lua.create_userdata(ScriptedPropertyNumber::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::Color => {
                lua.create_userdata(ScriptedPropertyColor::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::String => {
                lua.create_userdata(ScriptedPropertyString::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::Boolean => {
                lua.create_userdata(ScriptedPropertyBoolean::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::Trigger => {
                lua.create_userdata(ScriptedPropertyTrigger::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::Image => {
                lua.create_userdata(ScriptedPropertyImage::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::List => unreachable!("lists are installed before wrapping"),
            ScriptViewModelProperty::ViewModel => {
                model.view_model(name).ok_or_else(|| {
                    luaur_rt::Error::runtime(format!(
                        "view-model property '{name}' has no active instance"
                    ))
                })?;
                lua.create_userdata(ScriptedPropertyViewModel::new(model.clone(), name.clone()))?
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

struct ScriptedPropertyViewModel {
    parent: ScriptViewModel,
    name: String,
}

impl ScriptedPropertyViewModel {
    fn new(parent: ScriptViewModel, name: String) -> Self {
        Self { parent, name }
    }
}

impl UserData for ScriptedPropertyViewModel {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            let model = this.parent.view_model(&this.name).ok_or_else(|| {
                luaur_rt::Error::runtime(format!(
                    "view-model property '{}' has no active instance",
                    this.name
                ))
            })?;
            create_scripted_view_model_with_parent(lua, model, Some(&this.parent)).map(Value::Table)
        });
        fields.add_field_method_set("value", |lua, this, value: Table| {
            let value = model_from_table(&value)?;
            let previous = this.parent.view_model(&this.name);
            if this.parent.set_view_model(&this.name, &value) {
                ScriptViewModelFrameContext::for_lua(lua).replace_explicit_parent(
                    previous.as_ref(),
                    &value,
                    &this.parent,
                );
            }
            Ok(())
        });
    }
}

struct ScriptedPropertyNumber {
    model: ScriptViewModel,
    name: String,
}

struct ScriptedPropertyColor {
    model: ScriptViewModel,
    name: String,
}

impl ScriptedPropertyColor {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self { model, name }
    }
}

impl UserData for ScriptedPropertyColor {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| {
            Ok(i64::from(this.model.color(&this.name).unwrap_or_default()))
        });
        fields.add_field_method_set("value", |_, this, value: u32| {
            this.model.set_color(&this.name, value);
            Ok(())
        });
    }
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

struct ScriptedPropertyImage {
    model: ScriptViewModel,
    name: String,
}

impl ScriptedPropertyImage {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self { model, name }
    }
}

impl UserData for ScriptedPropertyImage {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            if let Some(image) = this.model.render_image(&this.name) {
                return lua
                    .create_userdata(ScriptedImage::from_render_image_rc(image))
                    .map(Value::UserData);
            }
            let asset = this.model.image(&this.name);
            Ok(asset
                .map(|image| create_asset_image(lua, image))
                .transpose()?
                .flatten()
                .map(Value::UserData)
                .unwrap_or(Value::Nil))
        });
        fields.add_field_method_set("value", |_, this, value: Value| {
            match value {
                Value::Nil => {
                    this.model.set_render_image(&this.name, None);
                }
                Value::UserData(image) => {
                    let image = image.borrow::<ScriptedImage>()?;
                    this.model
                        .set_render_image(&this.name, Some(image.render_image()?));
                }
                _ => return Err(luaur_rt::Error::runtime("expected Image userdata or nil")),
            }
            Ok(())
        });
    }
}

struct ScriptedPropertyList {
    model: ScriptViewModel,
    name: String,
    item_refs: BTreeMap<ViewModelInstanceKey, Table>,
}

impl ScriptedPropertyList {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self {
            model,
            name,
            item_refs: BTreeMap::new(),
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
    Ok(property)
}

impl UserData for ScriptedPropertyList {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("length", |_, this| {
            Ok(this.model.list_len(&this.name).unwrap_or_default())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("push", |lua, this, item: Table| {
            let item = model_from_table(&item)?;
            this.model.push_list_item(&this.name, &item);
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            Ok(())
        });
        methods.add_method("insert", |lua, this, (item, index): (Table, usize)| {
            let item = model_from_table(&item)?;
            this.model
                .insert_list_item(&this.name, index.saturating_sub(1), &item);
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            Ok(())
        });
        methods.add_method("pop", |lua, this, ()| {
            let item = this.model.pop_list_item(&this.name);
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            match item {
                Some(item) => create_scripted_view_model(lua, item).map(Value::Table),
                None => Ok(Value::Nil),
            }
        });
        methods.add_method("shift", |lua, this, ()| {
            let item = this.model.shift_list_item(&this.name);
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            match item {
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
        methods.add_method("clear", |lua, this, ()| {
            this.model.clear_list_items(&this.name);
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            Ok(())
        });
        methods.add_method("remove", |lua, this, item: Value| {
            let Value::Table(item) = item else {
                return Ok(());
            };
            let Ok(item) = model_from_table(&item) else {
                return Ok(());
            };
            this.model.remove_list_item(&this.name, &item, false);
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            Ok(())
        });
        methods.add_method("removeAt", |lua, this, index: usize| {
            let Some(index) = index.checked_sub(1) else {
                return Err(luaur_rt::Error::runtime("removeAt index out of range"));
            };
            if !this.model.remove_list_item_at(&this.name, index) {
                return Err(luaur_rt::Error::runtime("removeAt index out of range"));
            }
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            Ok(())
        });
        methods.add_method("removeAllOf", |lua, this, item: Value| {
            let Value::Table(item) = item else {
                return Ok(());
            };
            let Ok(item) = model_from_table(&item) else {
                return Ok(());
            };
            this.model.remove_list_item(&this.name, &item, true);
            ScriptViewModelFrameContext::for_lua(lua).sync_list_parents(&this.model);
            Ok(())
        });
    }
}

pub(super) struct ScriptedContext {
    model: Rc<RefCell<Option<ScriptViewModel>>>,
    context_present: Rc<Cell<bool>>,
    parents: Vec<ScriptViewModel>,
    missing_requested_data: Rc<Cell<bool>>,
    gpu_canvas: Option<crate::gpu_canvas::GpuCanvasContextBindings>,
    alive: Rc<Cell<bool>>,
}

impl ScriptedContext {
    pub(super) fn new(
        model: Rc<RefCell<Option<ScriptViewModel>>>,
        parents: Vec<ScriptViewModel>,
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
        parents: Vec<ScriptViewModel>,
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

    pub(super) fn set_parents(&mut self, parents: Vec<ScriptViewModel>) {
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
                Some(model) => Value::Table(create_scripted_view_model_with_parent(
                    lua,
                    model,
                    this.parents.first(),
                )?),
                None => {
                    this.missing_requested_data.set(true);
                    Value::Nil
                }
            })
        });
        methods.add_method("rootViewModel", |lua, this, ()| {
            this.require_live("rootViewModel")?;
            Ok(
                match this
                    .parents
                    .last()
                    .cloned()
                    .or_else(|| this.model.borrow().clone())
                {
                    Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                    None => {
                        this.missing_requested_data.set(true);
                        Value::Nil
                    }
                },
            )
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
        methods.add_method("gpuCanvas", |lua, this, ()| {
            this.require_live("gpuCanvas")?;
            let gpu_canvas = this
                .gpu_canvas
                .as_ref()
                .ok_or_else(|| luaur_rt::Error::runtime("GPU-canvas context is unavailable"))?;
            gpu_canvas.canvas_userdata(lua)
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
    parents: Vec<ScriptViewModel>,
}

impl UserData for ScriptedDataContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("viewModel", |lua, this, ()| match this.model.clone() {
            Some(model) => create_scripted_view_model_with_parent(lua, model, this.parents.first())
                .map(Value::Table),
            None => Ok(Value::Nil),
        });
        methods.add_method("parent", |lua, this, ()| {
            let Some((parent, remaining)) = this.parents.split_first() else {
                return Ok(Value::Nil);
            };
            lua.create_userdata(ScriptedDataContext {
                model: Some(parent.clone()),
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

struct ScriptedPropertyTrigger {
    model: ScriptViewModel,
    name: String,
    listeners: Vec<ScriptedListener>,
}

impl ScriptedPropertyTrigger {
    fn new(model: ScriptViewModel, name: String) -> Self {
        Self {
            model,
            name,
            listeners: Vec::new(),
        }
    }
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
            // C++ fires the backing ViewModelInstanceTrigger first; its
            // delegates then notify listeners synchronously. Keeping this
            // ordering means a callback observes the incremented counter.
            this.model.fire_trigger(&this.name);
            for listener in this.listeners.iter().rev() {
                listener
                    .callback
                    .call::<()>(listener.userdata.clone().unwrap_or(Value::Nil))?;
            }
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
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
    fn attached_empty_data_context_remains_non_nil_and_keeps_its_parent() {
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
                vec![parent],
                Rc::clone(&missing_requested_data),
                None,
                Rc::new(Cell::new(true)),
            ))
            .expect("scripted context");
        lua.globals()
            .set("context", context)
            .expect("context global");

        let (has_context, has_local_model, has_parent, has_parent_model): (bool, bool, bool, bool) =
            lua.load(
                r#"
                local dataContext = context:dataContext()
                local parent = dataContext:parent()
                context:markNeedsUpdate()
                return dataContext ~= nil,
                    dataContext:viewModel() ~= nil,
                    parent ~= nil,
                    parent:viewModel() ~= nil
                "#,
            )
            .eval()
            .expect("attached empty context evaluates");

        assert!(has_context);
        assert!(!has_local_model);
        assert!(has_parent);
        assert!(has_parent_model);
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
        fixture_models()
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

        lua.load("model[colorName].value = 0x10203040")
            .exec()
            .expect("color write");
        assert_eq!(model.color(&color), Some(0x1020_3040));
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
        let (parent, _) = model_with_property(ScriptViewModelProperty::Trigger);
        let (child, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        let context = ScriptViewModelFrameContext::default();
        context.link_parent(&child, &parent);
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
        let replacement = previous
            .named_instance(None)
            .expect("fresh replacement occurrence");
        let lua = Lua::new();
        let context = ScriptViewModelFrameContext::for_lua(&lua);
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
            "oldChild = parent[propertyName].value\n\
             parent[propertyName].value = replacement",
        )
        .exec()
        .expect("replace child through ScriptedPropertyViewModel");

        let old_table = lua.globals().get::<Table>("oldChild").expect("old child");
        let old = model_from_table(&old_table).expect("old child model");
        assert!(!context.has_explicit_parent(&old, &parent));
        assert!(context.has_explicit_parent(&replacement, &parent));

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
    fn frame_end_refreshes_list_parent_edges_changed_outside_lua() {
        let (parent, list) = model_with_property(ScriptViewModelProperty::List);
        let (child, trigger) = model_with_property(ScriptViewModelProperty::Trigger);
        assert!(parent.push_list_item(&list, &child));

        let context = ScriptViewModelFrameContext::default();
        context.sync_list_parents(&parent);
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
                 local same = first ~= nil and first == list[1] and model[listName][1] ~= nil\n\
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
            assert(property ~= nil)
            property:addListener(function()
                listenerCalls += 1
            end)
            property:fire()
            "#,
        )
        .exec()
        .expect("trigger script runs");

        assert_eq!(model.trigger(&trigger), Some(1));
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 1);
        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));
        assert_eq!(lua.globals().get::<i64>("listenerCalls").unwrap(), 1);

        lua.globals().set("model", Value::Nil).unwrap();
        lua.gc_collect().expect("collect scripted model wrapper");
        assert_eq!(context.registrations(&model), 0);
        assert!(model.fire_trigger(&trigger));
        assert!(!context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(1));
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
             assert(property.value ~= nil)",
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
        let factory_image = nuxie_render_api::Factory::decode_image(&mut factory, &png_header)
            .expect("recording factory image");
        lua.globals()
            .set(
                "factoryImage",
                lua.create_userdata(ScriptedImage::from_render_image(factory_image))
                    .unwrap(),
            )
            .unwrap();
        let dimensions: Table = lua
            .load(
                "local property = model:getImage(propertyName); \
                 property.value = factoryImage; \
                 return { property.value.width, property.value.height }",
            )
            .eval()
            .expect("factory-backed image round trip");
        assert_eq!(dimensions.get::<u32>(1).unwrap(), 3);
        assert_eq!(dimensions.get::<u32>(2).unwrap(), 5);
        let retained = model
            .render_image(&property_name)
            .expect("runtime view-model state retains the factory image");
        assert_eq!((retained.width(), retained.height()), (3, 5));
        assert!(!missing_requested_data.get());
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

    fn positive_blob_lookup_surface() -> String {
        let lua = Lua::new();
        let assets = ScriptedBlobAssets::install(&lua);
        assets
            .register("payload.bin", &[])
            .expect("empty duplicate registration");
        assets
            .register("payload.bin", &[0, 1, 2, 0xff])
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
