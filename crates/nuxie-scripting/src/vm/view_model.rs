use std::collections::{BTreeMap, BTreeSet};

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use luaur_rt::{
    Function, Lua, MultiValue, Table, UserData, UserDataFields, UserDataMethods, Value,
};
use nuxie_runtime::{
    RuntimeOwnedViewModelInstance, ScriptImage, ScriptViewModel, ScriptViewModelProperty,
};

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
                Some(ScriptViewModelProperty::List) => lua
                    .create_userdata(ScriptedPropertyList::new(get_list_model.clone(), name))
                    .map(Value::UserData),
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
                Some(model) => Value::UserData(lua.create_userdata(
                    ScriptedPropertyViewModel::new(model, get_view_model.clone()),
                )?),
                None => Value::Nil,
            })
        })?,
    )?;

    for (name, kind) in model.properties() {
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
            ScriptViewModelProperty::List => {
                lua.create_userdata(ScriptedPropertyList::new(model.clone(), name.clone()))?
            }
            ScriptViewModelProperty::ViewModel => {
                let nested = model.view_model(name).ok_or_else(|| {
                    luaur_rt::Error::runtime(format!(
                        "view-model property '{name}' has no active instance"
                    ))
                })?;
                lua.create_userdata(ScriptedPropertyViewModel::new(nested, model.clone()))?
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
    parent: ScriptViewModel,
}

impl ScriptedPropertyViewModel {
    fn new(model: ScriptViewModel, parent: ScriptViewModel) -> Self {
        Self { model, parent }
    }
}

impl UserData for ScriptedPropertyViewModel {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            create_scripted_view_model_with_parent(lua, this.model.clone(), Some(&this.parent))
                .map(Value::Table)
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

struct ScriptedImage(ScriptImage);

impl UserData for ScriptedImage {}

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
            Ok(match this.model.image(&this.name) {
                Some(image) => Value::UserData(lua.create_userdata(ScriptedImage(image))?),
                None => Value::Nil,
            })
        });
        fields.add_field_method_set("value", |_, this, value: Value| {
            let image = match value {
                Value::Nil => None,
                Value::UserData(image) => Some(image.borrow::<ScriptedImage>()?.0),
                _ => return Err(luaur_rt::Error::runtime("expected Image userdata or nil")),
            };
            this.model.set_image(&this.name, image);
            Ok(())
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
            // Registration of the parent table synchronizes all current list
            // edges. Do not add an explicit edge here: removing the item must
            // make a retained wrapper detached immediately.
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
    missing_requested_data: Rc<Cell<bool>>,
}

impl ScriptedContext {
    pub(super) fn new(
        model: Rc<RefCell<Option<ScriptViewModel>>>,
        parents: Vec<ScriptViewModel>,
        missing_requested_data: Rc<Cell<bool>>,
    ) -> Self {
        Self {
            model,
            parents,
            missing_requested_data,
        }
    }
}

impl UserData for ScriptedContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("viewModel", |lua, this, ()| {
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
            let Some(model) = this.model.borrow().clone() else {
                this.missing_requested_data.set(true);
                return Ok(Value::Nil);
            };
            lua.create_userdata(ScriptedDataContext {
                model,
                parents: this.parents.clone(),
            })
            .map(Value::UserData)
        });
        methods.add_method("image", |lua, this, name: String| {
            let Some(model) = this.model.borrow().clone() else {
                this.missing_requested_data.set(true);
                return Ok(Value::Nil);
            };
            Ok(match model.image_asset_named(&name) {
                Some(image) => Value::UserData(lua.create_userdata(ScriptedImage(image))?),
                None => Value::Nil,
            })
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
            create_scripted_view_model_with_parent(lua, this.model.clone(), this.parents.first())
                .map(Value::Table)
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
        lua.globals().set("listName", list).expect("list name global");

        lua.load(
            "local list = model:getList(listName)\n\
             list:remove(nil)\n\
             list:removeAllOf(nil)",
        )
        .exec()
        .expect("nil removals are no-ops");
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
        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model).expect("scripted model");
        lua.globals().set("model", table).expect("model global");

        let actual: i64 = lua
            .load("return model:getIndex()")
            .eval()
            .expect("getIndex runs");

        assert_eq!(actual, expected);
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
        let (asset_name, expected) = file
            .file_assets()
            .into_iter()
            .enumerate()
            .find_map(|(index, asset)| {
                let index = u64::try_from(index).ok()?;
                (asset.type_name == "ImageAsset" && index != current.file_asset_index()).then(|| {
                    (
                        asset.string_property("name").unwrap().to_owned(),
                        index,
                    )
                })
            })
            .expect("fixture has a replacement image");

        let lua = Lua::new();
        let table = create_scripted_view_model(&lua, model.clone()).expect("scripted model");
        let missing_requested_data = Rc::new(Cell::new(false));
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(Some(model.clone()))),
                Vec::new(),
                Rc::clone(&missing_requested_data),
            ))
            .expect("scripted context");
        lua.globals().set("model", table).unwrap();
        lua.globals().set("context", context).unwrap();
        lua.globals().set("propertyName", property_name.clone()).unwrap();
        lua.globals().set("assetName", asset_name).unwrap();

        lua.load(
            "local property = model:getImage(propertyName)\n\
             assert(property ~= nil)\n\
             property.value = context:image(assetName)\n\
             assert(property.value ~= nil)",
        )
        .exec()
        .expect("image property script runs");

        assert_eq!(
            model.image(&property_name).unwrap().file_asset_index(),
            expected
        );
        assert!(!missing_requested_data.get());
    }
}
