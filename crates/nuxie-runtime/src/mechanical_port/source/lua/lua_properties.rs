use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::mechanical_port::source::lua::rive_lua_libs::*;

fn scripting_context(state: &mut LuaState) -> Option<&mut dyn ScriptingContext> {
    state.thread_data_optional::<dyn ScriptingContext>()
}

fn push_view_model_instance_value(state: &mut LuaState, property_value: CoreHandle) {
    let Some(core_type) = property_value.core_type() else {
        state.push_nil();
        return;
    };
    match core_type {
        ViewModelInstanceNumber::TYPE_KEY => {
            state.new_rive(ScriptedPropertyNumber::new(state, Some(property_value)))
        }
        ViewModelInstanceTrigger::TYPE_KEY => {
            state.new_rive(ScriptedPropertyTrigger::new(state, Some(property_value)))
        }
        ViewModelInstanceList::TYPE_KEY => {
            state.new_rive(ScriptedPropertyList::new(state, Some(property_value)))
        }
        ViewModelInstanceColor::TYPE_KEY => {
            state.new_rive(ScriptedPropertyColor::new(state, Some(property_value)))
        }
        ViewModelInstanceString::TYPE_KEY => {
            state.new_rive(ScriptedPropertyString::new(state, Some(property_value)))
        }
        ViewModelInstanceBoolean::TYPE_KEY => {
            state.new_rive(ScriptedPropertyBoolean::new(state, Some(property_value)))
        }
        ViewModelInstanceEnum::TYPE_KEY => {
            state.new_rive(ScriptedPropertyEnum::new(state, Some(property_value)))
        }
        ViewModelInstanceViewModel::TYPE_KEY => {
            let view_model = property_value
                .with(|value| {
                    value
                        .as_view_model_instance_view_model()
                        .and_then(ViewModelInstanceViewModel::reference_view_model_instance)
                })
                .flatten()
                .and_then(|instance| {
                    instance
                        .with(|instance| {
                            instance
                                .as_view_model_instance()
                                .and_then(ViewModelInstance::get_view_model)
                        })
                        .flatten()
                });
            state.new_rive(ScriptedPropertyViewModel::new(
                state,
                view_model,
                Some(property_value),
            ));
        }
        ViewModelInstanceSymbolListIndex::TYPE_KEY => {
            let value = property_value
                .with_downcast::<ViewModelInstanceSymbolListIndex, _>(|value| {
                    value.property_value()
                })
                .unwrap_or_default();
            state.push_integer(value as i64)
        }
        ViewModelInstanceAssetImage::TYPE_KEY => {
            state.new_rive(ScriptedPropertyImage::new(state, Some(property_value)))
        }
        ViewModelInstanceAssetFont::TYPE_KEY => {
            state.new_rive(ScriptedPropertyFont::new(state, Some(property_value)))
        }
        ViewModelInstanceAssetBlob::TYPE_KEY => {
            state.new_rive(ScriptedPropertyBlob::new(state, Some(property_value)))
        }
        _ => state.push_nil(),
    }
}

struct ScriptedPropertyDelegate {
    runtime: Rc<RefCell<ScriptedPropertyRuntime>>,
}

impl ViewModelInstanceValueDelegate for ScriptedPropertyDelegate {
    fn value_changed(&mut self) {
        ScriptedProperty::notify_runtime(&self.runtime);
    }
}

impl ScriptedProperty {
    pub fn new(state: &mut LuaState, instance_value: Option<CoreHandle>) -> Self {
        let runtime = Rc::new(RefCell::new(ScriptedPropertyRuntime {
            state,
            cached_value_ref: 0,
            listeners: Vec::new(),
        }));
        let delegate: ViewModelInstanceValueDelegateHandle =
            Rc::new(RefCell::new(ScriptedPropertyDelegate {
                runtime: runtime.clone(),
            }));
        let mut property = Self {
            runtime,
            delegate: Some(delegate.clone()),
            instance_value,
            owner: None,
            #[cfg(feature = "tools")]
            orphan_context: None,
            #[cfg(feature = "tools")]
            orphan_owner_tag: 0,
            disposed: false,
        };
        if let Some(value) = property.instance_value.as_ref() {
            value.with_mut(|value| {
                if let Some(value) = value.as_view_model_instance_value_mut() {
                    value.add_delegate(&delegate);
                }
            });
        }
        if let Some(context) = scripting_context(state) {
            property.owner = context.current_scripted_object();
            if let Some(owner) = property.owner.as_ref() {
                let identity = Rc::as_ptr(&property.runtime) as usize;
                owner.with_mut(|owner| owner.scripted_object_add_tracked_property(identity));
            } else {
                #[cfg(feature = "tools")]
                {
                    context.track_orphan_scripted_property(&mut property);
                    property.orphan_context = Some(context);
                    property.orphan_owner_tag = context.orphan_owner_tag();
                }
            }
        }
        property
    }

    pub fn clear_cached_value_ref(&mut self) {
        let mut runtime = self.runtime.borrow_mut();
        if runtime.cached_value_ref != 0 {
            unsafe { &mut *runtime.state }.unref(runtime.cached_value_ref);
            runtime.cached_value_ref = 0;
        }
    }

    pub fn clear_listeners(&mut self) {
        let mut runtime = self.runtime.borrow_mut();
        let state = unsafe { &mut *runtime.state };
        for listener in runtime.listeners.drain(..) {
            state.unref(listener.function);
            if listener.userdata != 0 {
                state.unref(listener.userdata);
            }
            state.unref(listener.property_self_ref);
        }
    }

    pub fn dispose(&mut self) {
        if self.disposed {
            return;
        }
        self.disposed = true;
        if let Some(owner) = self.owner.take() {
            let identity = Rc::as_ptr(&self.runtime) as usize;
            owner.with_mut(|owner| owner.scripted_object_remove_tracked_property(identity));
        }
        #[cfg(feature = "tools")]
        if let Some(context) = self.orphan_context.take() {
            unsafe { &mut *context }.untrack_orphan_scripted_property(self);
        }
        if let (Some(instance_value), Some(delegate)) =
            (self.instance_value.take(), self.delegate.take())
        {
            instance_value.with_mut(|value| {
                if let Some(value) = value.as_view_model_instance_value_mut() {
                    value.remove_delegate(&delegate);
                }
            });
        }
        self.clear_cached_value_ref();
        self.clear_listeners();
    }

    pub fn value_changed(&mut self) {
        Self::notify_runtime(&self.runtime);
    }

    fn notify_runtime(runtime: &Rc<RefCell<ScriptedPropertyRuntime>>) {
        let mut runtime = runtime.borrow_mut();
        if runtime.cached_value_ref != 0 {
            unsafe { &mut *runtime.state }.unref(runtime.cached_value_ref);
            runtime.cached_value_ref = 0;
        }
        if runtime.listeners.is_empty() {
            return;
        }
        let state = unsafe { &mut *runtime.state };
        if !state.check_stack((runtime.listeners.len() * 2 + LUA_MIN_STACK) as i32) {
            return;
        }
        for listener in runtime.listeners.iter().rev() {
            state.raw_get_i(LUA_REGISTRY_INDEX, listener.function);
            if listener.userdata != 0 {
                state.raw_get_i(LUA_REGISTRY_INDEX, listener.userdata);
            } else {
                state.push_nil();
            }
        }
        let calls = runtime.listeners.len();
        for _ in 0..calls {
            state.pcall(1, 0, 0);
        }
    }

    pub fn add_listener(&mut self) -> i32 {
        let mut runtime = self.runtime.borrow_mut();
        let state = unsafe { &mut *runtime.state };
        if state.is_function(2) {
            state.push_value(1);
            let property_self_ref = state.reference(-1);
            state.pop(1);
            let callback_ref = state.reference(2);
            runtime.listeners.push(ScriptedListener {
                function: callback_ref,
                userdata: 0,
                property_self_ref,
            });
            return 0;
        }
        if state.is_function(3) {
            state.push_value(1);
            let property_self_ref = state.reference(-1);
            state.pop(1);
            let userdata_ref = state.reference(2);
            let callback_ref = state.reference(3);
            runtime.listeners.push(ScriptedListener {
                function: callback_ref,
                userdata: userdata_ref,
                property_self_ref,
            });
            return 0;
        }
        state.type_error(2, state.type_name(LuaType::Function))
    }

    pub fn remove_listener(&mut self) -> i32 {
        let mut runtime = self.runtime.borrow_mut();
        let state = unsafe { &mut *runtime.state };
        let check_index = if state.is_function(2) {
            2
        } else if state.is_function(3) {
            3
        } else {
            state.type_error(2, state.type_name(LuaType::Function));
            2
        };
        let mut index = 0;
        while index < runtime.listeners.len() {
            let listener = &runtime.listeners[index];
            state.raw_get_i(LUA_REGISTRY_INDEX, listener.function);
            if state.raw_equal(-1, check_index) {
                let listener = runtime.listeners.remove(index);
                state.unref(listener.function);
                if listener.userdata != 0 {
                    state.unref(listener.userdata);
                }
                state.unref(listener.property_self_ref);
            } else {
                index += 1;
            }
            state.pop(1);
        }
        0
    }

    pub fn state(&self) -> *mut LuaState {
        self.runtime.borrow().state
    }

    pub fn owner(&self) -> Option<CoreHandle> {
        self.owner.clone()
    }
}

impl Drop for ScriptedProperty {
    fn drop(&mut self) {
        self.dispose();
    }
}

impl ScriptedPropertyViewModel {
    pub fn new(
        state: &mut LuaState,
        view_model: Option<CoreHandle>,
        value: Option<CoreHandle>,
    ) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
            view_model,
            value_ref: 0,
        }
    }

    pub fn set_value(&mut self, replacement: Option<CoreHandle>) {
        if let Some(value) = self.property.instance_value_mut() {
            let parent = value
                .with(|value| {
                    value
                        .as_view_model_instance_view_model()
                        .and_then(ViewModelInstanceViewModel::parent_view_model_instance)
                })
                .flatten();
            if let (Some(parent), Some(replacement)) = (parent, replacement) {
                parent.with_mut(|parent| {
                    if let Some(parent) = parent.as_view_model_instance_mut() {
                        parent.replace_view_model_property_handle(value, replacement);
                    }
                });
            }
        }
    }

    pub fn dispose(&mut self) {
        self.clear_ref();
        self.property.dispose();
    }

    pub fn clear_ref(&mut self) {
        if self.value_ref != 0 {
            unsafe { &mut *self.property.state() }.unref(self.value_ref);
            self.value_ref = 0;
        }
    }

    pub fn push_value(&mut self) -> i32 {
        let state = unsafe { &mut *self.property.state() };
        if self.value_ref != 0 {
            state.raw_get_i(LUA_REGISTRY_INDEX, self.value_ref);
            return 1;
        }
        if let Some(value) = self.property.instance_value_mut() {
            let reference = value
                .with(|value| {
                    value
                        .as_view_model_instance_view_model()
                        .and_then(ViewModelInstanceViewModel::reference_view_model_instance)
                })
                .flatten();
            let view_model = reference
                .as_ref()
                .and_then(|instance| {
                    instance
                        .with(|instance| {
                            instance
                                .as_view_model_instance()
                                .and_then(ViewModelInstance::get_view_model)
                        })
                        .flatten()
                })
                .or_else(|| self.view_model.clone());
            state.new_rive(ScriptedViewModel::new(state, view_model, reference));
        } else {
            state.new_rive(ScriptedViewModel::new(state, self.view_model.clone(), None));
        }
        self.value_ref = state.reference(-1);
        1
    }

    pub fn relink_data_bind(&mut self) {
        self.clear_ref();
    }
}

impl Drop for ScriptedPropertyViewModel {
    fn drop(&mut self) {
        self.dispose();
    }
}

impl ScriptedViewModel {
    pub fn new(
        state: &mut LuaState,
        view_model: Option<CoreHandle>,
        view_model_instance: Option<CoreHandle>,
    ) -> Self {
        let context = scripting_context(state).map(|context| context as *mut dyn ScriptingContext);
        if let (Some(context), Some(instance)) = (context, view_model_instance.as_ref()) {
            unsafe { &mut *context }.track_view_model_instance(instance.clone());
        }
        Self {
            state,
            view_model,
            view_model_instance,
            property_refs: HashMap::new(),
            scripting_context: context,
        }
    }

    pub fn instance(&mut self, state: &mut LuaState) -> i32 {
        if let Some(view_model) = self.view_model.as_ref() {
            let instance = if state.top() == 2 && !state.is_none_or_nil(-1) && state.is_string(-1) {
                let name = state.to_string(-1).unwrap();
                view_model
                    .with_downcast_mut::<ViewModel, _>(|view_model| {
                        view_model
                            .create_from_instance(&name)
                            .or_else(|| view_model.create_instance())
                    })
                    .flatten()
            } else {
                view_model
                    .with_downcast_mut::<ViewModel, _>(ViewModel::create_instance)
                    .flatten()
            };
            state.new_rive(ScriptedViewModel::new(
                state,
                Some(view_model.clone()),
                instance,
            ));
        } else {
            state.push_nil();
        }
        1
    }

    pub fn push_value(&mut self, name: &str, core_type: u16) -> i32 {
        let state = unsafe { &mut *self.state };
        if let Some(reference) = self.property_refs.get(name) {
            state.raw_get_i(LUA_REGISTRY_INDEX, *reference);
            return 1;
        }
        if let Some(instance) = self.view_model_instance.as_ref() {
            if let Some(property_value) = instance
                .with(|instance| {
                    instance
                        .as_view_model_instance()
                        .and_then(|instance| instance.property_value_named(name))
                })
                .flatten()
            {
                if core_type == 0 || property_value.core_type() == Some(core_type) {
                    push_view_model_instance_value(state, property_value);
                } else {
                    state.push_nil();
                }
            } else {
                state.push_nil();
            }
        } else if core_type != 0 || self.view_model.is_none() {
            state.push_nil();
        } else if let Some(property) = self
            .view_model
            .as_ref()
            .and_then(|model| {
                model.with_downcast::<ViewModel, _>(|model| model.property_named(name))
            })
            .flatten()
        {
            match property.core_type().unwrap_or_default() {
                ViewModelPropertyNumber::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyNumber::new(state, None))
                }
                ViewModelPropertyTrigger::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyTrigger::new(state, None))
                }
                ViewModelPropertyList::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyList::new(state, None))
                }
                ViewModelPropertyColor::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyColor::new(state, None))
                }
                ViewModelPropertyString::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyString::new(state, None))
                }
                ViewModelPropertyBoolean::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyBoolean::new(state, None))
                }
                ViewModelPropertyEnum::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyEnum::new(state, None))
                }
                ViewModelPropertyViewModel::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyViewModel::new(state, None, None))
                }
                ViewModelPropertyAssetImage::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyImage::new(state, None))
                }
                ViewModelPropertyAssetFont::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyFont::new(state, None))
                }
                ViewModelPropertyAssetBlob::TYPE_KEY => {
                    state.new_rive(ScriptedPropertyBlob::new(state, None))
                }
                ViewModelPropertySymbolListIndex::TYPE_KEY => state.push_integer(1),
                _ => state.push_nil(),
            }
        } else {
            state.push_nil();
        }
        self.property_refs
            .insert(name.to_owned(), state.reference(-1));
        1
    }

    pub fn push_index(&mut self) -> i32 {
        let state = unsafe { &mut *self.state };
        let index = self
            .view_model_instance
            .as_ref()
            .and_then(|instance| {
                instance
                    .with(|instance| {
                        instance.as_view_model_instance().and_then(|instance| {
                            instance.property_value_for_symbol(SymbolType::ITEM_INDEX)
                        })
                    })
                    .flatten()
            })
            .and_then(|value| {
                value.with_downcast::<ViewModelInstanceSymbolListIndex, _>(
                    ViewModelInstanceSymbolListIndex::property_value,
                )
            })
            .unwrap_or(-1);
        state.push_integer(index as i64);
        1
    }

    pub fn state(&self) -> *mut LuaState {
        self.state
    }

    pub fn view_model_instance(&self) -> Option<CoreHandle> {
        self.view_model_instance.clone()
    }

    pub fn mutable_view_model_instance(&mut self) -> Option<CoreHandle> {
        self.view_model_instance.clone()
    }
}

impl Drop for ScriptedViewModel {
    fn drop(&mut self) {
        if let (Some(context), Some(instance)) =
            (self.scripting_context, self.view_model_instance.as_ref())
        {
            unsafe { &mut *context }.untrack_view_model_instance(instance);
        }
        let state = unsafe { &mut *self.state };
        for (_, reference) in self.property_refs.drain() {
            state.unref(reference);
        }
    }
}

macro_rules! scalar_property {
    ($name:ident, $value:ty, $as_value:ident, $default:expr, $push:ident) => {
        impl $name {
            pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
                Self {
                    property: ScriptedProperty::new(state, value),
                }
            }

            pub fn push_value(&mut self) -> i32 {
                let value = self
                    .property
                    .instance_value()
                    .and_then(|value| {
                        value.with_downcast::<$value, _>(|value| value.property_value())
                    })
                    .unwrap_or($default);
                unsafe { &mut *self.property.state() }.$push(value.into());
                1
            }
        }
    };
}

scalar_property!(
    ScriptedPropertyNumber,
    ViewModelInstanceNumber,
    as_number,
    0.0_f32,
    push_number
);
scalar_property!(
    ScriptedPropertyColor,
    ViewModelInstanceColor,
    as_color,
    0_u32,
    push_unsigned
);
scalar_property!(
    ScriptedPropertyBoolean,
    ViewModelInstanceBoolean,
    as_boolean,
    false,
    push_boolean
);

impl ScriptedPropertyNumber {
    pub fn set_value(&mut self, value: f32) {
        if let Some(property) = self.property.instance_value_mut() {
            property.with_downcast_mut::<ViewModelInstanceNumber, _>(|property| {
                property.set_property_value(value)
            });
        }
    }
}

impl ScriptedPropertyColor {
    pub fn set_value(&mut self, value: u32) {
        if let Some(property) = self.property.instance_value_mut() {
            property.with_downcast_mut::<ViewModelInstanceColor, _>(|property| {
                property.set_property_value(value as i32)
            });
        }
    }
}

impl ScriptedPropertyBoolean {
    pub fn set_value(&mut self, value: bool) {
        if let Some(property) = self.property.instance_value_mut() {
            property.with_downcast_mut::<ViewModelInstanceBoolean, _>(|property| {
                property.set_property_value(value)
            });
        }
    }
}

impl ScriptedPropertyTrigger {
    pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
        }
    }
}

impl ScriptedPropertyString {
    pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
        }
    }

    pub fn set_value(&mut self, value: String) {
        if let Some(property) = self.property.instance_value_mut() {
            property.with_downcast_mut::<ViewModelInstanceString, _>(|property| {
                property.set_property_value(value)
            });
        }
    }

    pub fn push_value(&mut self) -> i32 {
        let value = self
            .property
            .instance_value()
            .and_then(|value| {
                value.with_downcast::<ViewModelInstanceString, _>(|value| {
                    value.property_value().to_owned()
                })
            })
            .unwrap_or_default();
        unsafe { &mut *self.property.state() }.push_string(&value);
        1
    }
}

impl ScriptedPropertyEnum {
    pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
        }
    }

    pub fn set_value(&mut self, value: String) {
        if let Some(property) = self.property.instance_value_mut() {
            property.with_downcast_mut::<ViewModelInstanceEnum, _>(|property| {
                property.set_value_named(&value)
            });
        }
    }

    pub fn push_value(&mut self) -> i32 {
        let state = unsafe { &mut *self.property.state() };
        let key = self.property.instance_value().and_then(|value| {
            value.with_downcast::<ViewModelInstanceEnum, _>(|value| {
                let data_enum = value
                    .view_model_property_enum()
                    .and_then(|property| property.data_enum());
                data_enum
                    .and_then(|data_enum| {
                        data_enum.with(|data_enum| {
                            let data_enum = data_enum.as_data_enum().expect("authored enum");
                            data_enum.key_at(value.property_value()).to_owned()
                        })
                    })
                    .unwrap_or_default()
            })
        });
        if let Some(key) = key.filter(|key| !key.is_empty()) {
            state.push_string(&key);
            return 1;
        }
        state.push_string("");
        1
    }
}

impl ScriptedPropertyList {
    pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
            changed: false,
            property_refs: HashMap::new(),
        }
    }

    pub fn value_changed(&mut self) {
        self.changed = true;
        self.property.value_changed();
    }

    pub fn push_length(&mut self) -> i32 {
        let length = self
            .property
            .instance_value()
            .and_then(|list| {
                list.with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
            })
            .unwrap_or(0);
        unsafe { &mut *self.property.state() }.push_integer(length as i64);
        1
    }

    pub fn push_value(&mut self, index: usize) -> i32 {
        let state = unsafe { &mut *self.property.state() };
        let Some(list) = self.property.instance_value() else {
            state.push_nil();
            return 1;
        };
        let items = list
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
            .unwrap_or_default();
        if self.changed {
            let mut references = HashMap::new();
            for item in &items {
                if let Some(instance) = item
                    .with(|item| {
                        item.as_view_model_instance_list_item()
                            .and_then(ViewModelInstanceListItem::view_model_instance)
                    })
                    .flatten()
                {
                    let key = instance.clone();
                    if let Some(reference) = self.property_refs.remove(&key) {
                        references.insert(key, reference);
                    }
                }
            }
            for (_, reference) in self.property_refs.drain() {
                state.unref(reference);
            }
            self.property_refs = references;
            self.changed = false;
        }
        if let Some(instance) = items.get(index).and_then(|item| {
            item.with(|item| {
                item.as_view_model_instance_list_item()
                    .and_then(ViewModelInstanceListItem::view_model_instance)
            })
            .flatten()
        }) {
            let key = instance.clone();
            if let Some(reference) = self.property_refs.get(&key) {
                state.raw_get_i(LUA_REGISTRY_INDEX, *reference);
            } else {
                let view_model = instance
                    .with(|instance| {
                        instance
                            .as_view_model_instance()
                            .and_then(ViewModelInstance::get_view_model)
                    })
                    .flatten();
                state.new_rive(ScriptedViewModel::new(
                    state,
                    view_model,
                    Some(instance.clone()),
                ));
                self.property_refs.insert(key, state.reference(-1));
            }
        } else {
            state.push_nil();
        }
        1
    }
}

impl Drop for ScriptedPropertyList {
    fn drop(&mut self) {
        let state = unsafe { &mut *self.property.state() };
        for (_, reference) in self.property_refs.drain() {
            state.unref(reference);
        }
    }
}

fn file_asset_for_property(property: &ScriptedProperty, asset_id: u32) -> Option<CoreHandle> {
    if let Some(owner) = property.owner() {
        if let Some(file) = owner.with(|owner| owner.scripted_object_file()).flatten() {
            return file
                .with_file(|file| file.asset(asset_id as usize))
                .flatten();
        }
    }
    #[cfg(feature = "tools")]
    if property.owner().is_none() {
        let file = property.instance_value().and_then(|value| {
            value
                .with(|value| {
                    value
                        .as_view_model_instance_value()
                        .and_then(ViewModelInstanceValue::view_model_instance)
                })
                .flatten()
                .and_then(|instance| {
                    instance
                        .with(|instance| {
                            instance
                                .as_view_model_instance()
                                .and_then(ViewModelInstance::get_view_model)
                        })
                        .flatten()
                })
                .and_then(|view_model| view_model.with_downcast::<ViewModel, _>(ViewModel::file))
        });
        if let Some(file) = file {
            return file
                .with_file(|file| file.asset(asset_id as usize))
                .flatten();
        }
    }
    None
}

impl ScriptedPropertyImage {
    pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
        }
    }

    pub fn set_value(&mut self, image: Option<RenderImageRef>) {
        if let Some(property) = self.property.instance_value_mut() {
            property.with_downcast_mut::<ViewModelInstanceAssetImage, _>(|property| {
                property.set_value(image)
            });
        }
    }

    pub fn push_value(&mut self) -> i32 {
        let state = unsafe { &mut *self.property.state() };
        if self.property.cached_value_ref() != 0 {
            state.raw_get_i(LUA_REGISTRY_INDEX, self.property.cached_value_ref());
            return 1;
        }
        let render_image = self.property.instance_value().and_then(|value| {
            value.with_downcast::<ViewModelInstanceAssetImage, _>(|value| {
                value.asset().render_image().or_else(|| {
                    file_asset_for_property(&self.property, value.base.property_value())
                        .and_then(|asset| {
                            asset.with_downcast::<ImageAsset, _>(|asset| {
                                asset.render_image().cloned()
                            })
                        })
                        .flatten()
                })
            })
        });
        if let Some(render_image) = render_image {
            let scripted_image = ScriptedImage::lua_new(state);
            scripted_image.image = Some(render_image);
            self.property.set_cached_value_ref(state.reference(-1));
        } else {
            state.push_nil();
        }
        1
    }
}

impl ScriptedPropertyFont {
    pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
        }
    }

    pub fn set_value(&mut self, font: Option<FontRef>) {
        if let Some(property) = self.property.instance_value_mut() {
            property.with_downcast_mut::<ViewModelInstanceAssetFont, _>(|property| {
                property.set_value(font)
            });
        }
    }

    pub fn push_value(&mut self) -> i32 {
        let state = unsafe { &mut *self.property.state() };
        if self.property.cached_value_ref() != 0 {
            state.raw_get_i(LUA_REGISTRY_INDEX, self.property.cached_value_ref());
            return 1;
        }
        let font = self.property.instance_value().and_then(|value| {
            value.with_downcast::<ViewModelInstanceAssetFont, _>(|value| {
                value.asset().font().or_else(|| {
                    file_asset_for_property(&self.property, value.base.property_value())
                        .and_then(|asset| asset.with_downcast::<FontAsset, _>(FontAsset::font))
                        .flatten()
                })
            })
        });
        if let Some(font) = font {
            state.new_rive(ScriptedFont { font: Some(font) });
            self.property.set_cached_value_ref(state.reference(-1));
        } else {
            state.push_nil();
        }
        1
    }
}

impl ScriptedPropertyBlob {
    pub fn new(state: &mut LuaState, value: Option<CoreHandle>) -> Self {
        Self {
            property: ScriptedProperty::new(state, value),
        }
    }

    pub fn set_value(&mut self, blob: Option<CoreHandle>) {
        if let Some(property) = self.property.instance_value_mut() {
            let asset_id = blob
                .as_ref()
                .and_then(|asset| {
                    asset.with_downcast::<BlobAsset, _>(|asset| asset.base.asset_id())
                })
                .unwrap_or(u32::MAX);
            property.with_downcast_mut::<ViewModelInstanceAssetBlob, _>(|property| {
                property.set_value(None);
                property.base.set_property_value(asset_id);
            });
        }
    }

    pub fn push_value(&mut self) -> i32 {
        let state = unsafe { &mut *self.property.state() };
        if self.property.cached_value_ref() != 0 {
            state.raw_get_i(LUA_REGISTRY_INDEX, self.property.cached_value_ref());
            return 1;
        }
        let file_asset = self.property.instance_value().and_then(|value| {
            value
                .with_downcast::<ViewModelInstanceAssetBlob, _>(|value| {
                    file_asset_for_property(&self.property, value.base.property_value())
                })
                .flatten()
        });
        if let Some(asset) = file_asset {
            state.new_rive(ScriptedBlob { asset: Some(asset) });
            self.property.set_cached_value_ref(state.reference(-1));
        } else {
            state.push_nil();
        }
        1
    }
}

impl ScriptedEnumValues {
    pub fn new(state: &mut LuaState, data_enum: Option<CoreHandle>) -> Self {
        Self { state, data_enum }
    }

    pub fn push_value(&self, index: i32) -> i32 {
        let state = unsafe { &mut *self.state };
        if index >= 0 {
            if let Some(key) = self
                .data_enum
                .as_ref()
                .and_then(|data_enum| {
                    data_enum.with(|data_enum| {
                        let data_enum = data_enum.as_data_enum().expect("authored enum");
                        data_enum
                            .values()
                            .get(index as usize)
                            .map(|_| data_enum.key_at(index as u32))
                    })
                })
                .flatten()
            {
                state.push_string(&key);
                return 1;
            }
        }
        state.push_nil();
        1
    }

    pub fn push_length(&self) -> i32 {
        let length = self
            .data_enum
            .as_ref()
            .and_then(|data_enum| {
                data_enum.with(|data_enum| {
                    data_enum
                        .as_data_enum()
                        .expect("authored enum")
                        .values()
                        .len()
                })
            })
            .unwrap_or(0);
        unsafe { &mut *self.state }.push_integer(length as i64);
        1
    }
}

fn property_vm_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyViewModel>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_vm_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let replacement = state.to_rive::<ScriptedViewModel>(3).view_model_instance();
        state
            .to_rive_mut::<ScriptedPropertyViewModel>(1)
            .set_value(replacement);
    }
    0
}

fn view_model_index(state: &mut LuaState) -> i32 {
    let name = state.check_string(2);
    let view_model = state.to_rive_mut::<ScriptedViewModel>(1);
    assert!(std::ptr::eq(view_model.state(), state));
    view_model.push_value(&name, 0)
}

fn push_list_item(state: &mut LuaState, item: Option<CoreHandle>) -> i32 {
    if let Some(instance) = item.and_then(|item| {
        item.with(|item| {
            item.as_view_model_instance_list_item()
                .and_then(ViewModelInstanceListItem::view_model_instance)
        })
        .flatten()
    }) {
        let view_model = instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .and_then(ViewModelInstance::get_view_model)
            })
            .flatten();
        state.new_rive(ScriptedViewModel::new(state, view_model, Some(instance)));
        1
    } else {
        0
    }
}

fn insert_list_item(list: &CoreHandle, instance: Option<CoreHandle>) -> Option<CoreHandle> {
    let mut item = ViewModelInstanceListItem::default();
    item.set_view_model_instance(instance);
    list.insert_sibling(item)
}

fn property_namecall_atom(
    state: &mut LuaState,
    property: &mut ScriptedProperty,
    atom: LuaAtoms,
) -> Option<i32> {
    match atom {
        LuaAtoms::AddListener => Some(property.add_listener()),
        LuaAtoms::RemoveListener => Some(property.remove_listener()),
        LuaAtoms::Fire => {
            if let Some(value) = property.instance_value_mut()
                && value
                    .with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| trigger.trigger())
                    .is_some()
            {
                return Some(0);
            }
            let view_model = state.to_rive::<ScriptedViewModel>(2);
            let instance = view_model.view_model_instance();
            let list = property.instance_value_mut().unwrap();
            if let Some(item) = insert_list_item(&list, instance) {
                list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item));
            }
            Some(0)
        }
        LuaAtoms::Push => {
            let view_model = state.to_rive::<ScriptedViewModel>(2);
            let instance = view_model.view_model_instance();
            let list = property.instance_value_mut().unwrap();
            if let Some(item) = insert_list_item(&list, instance) {
                list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item));
            }
            Some(0)
        }
        LuaAtoms::Pop => {
            let list = property.instance_value_mut().unwrap();
            let item = list
                .with_downcast_mut::<ViewModelInstanceList, _>(ViewModelInstanceList::pop)
                .flatten();
            Some(push_list_item(state, item))
        }
        LuaAtoms::Swap => {
            let first = state.to_unsigned(2) - 1;
            let second = state.to_unsigned(3) - 1;
            property
                .instance_value_mut()
                .unwrap()
                .with_downcast_mut::<ViewModelInstanceList, _>(|list| list.swap(first, second));
            Some(0)
        }
        LuaAtoms::Shift => {
            let list = property.instance_value_mut().unwrap();
            let item = list
                .with_downcast_mut::<ViewModelInstanceList, _>(ViewModelInstanceList::shift)
                .flatten();
            Some(push_list_item(state, item))
        }
        LuaAtoms::Clear => {
            property
                .instance_value_mut()
                .unwrap()
                .with_downcast_mut::<ViewModelInstanceList, _>(
                    ViewModelInstanceList::remove_all_items,
                );
            Some(0)
        }
        LuaAtoms::Insert => {
            let view_model = state.to_rive::<ScriptedViewModel>(2);
            let index = state.to_unsigned(3) - 1;
            let list = property.instance_value_mut().unwrap();
            if let Some(item) = insert_list_item(&list, view_model.view_model_instance()) {
                list.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                    list.add_item_at(item, index as i32)
                });
            }
            Some(0)
        }
        LuaAtoms::Remove => {
            if let Some(view_model) = state.to_rive_optional::<ScriptedViewModel>(2, true) {
                if let Some(instance) = view_model.view_model_instance() {
                    let list = property.instance_value_mut().unwrap();
                    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                        let item = list
                            .list_items()
                            .iter()
                            .find(|item| {
                                item.with(|item| {
                                    item.as_view_model_instance_list_item()
                                        .and_then(ViewModelInstanceListItem::view_model_instance)
                                })
                                .flatten()
                                .as_ref()
                                    == Some(&instance)
                            })
                            .cloned();
                        if let Some(item) = item {
                            list.remove_item(&item);
                        }
                    });
                }
            }
            Some(0)
        }
        LuaAtoms::RemoveAt => {
            let lua_index = state.check_integer(2);
            let list = property.instance_value_mut().unwrap();
            let length = list
                .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
                .unwrap_or_default();
            if lua_index < 1 || lua_index as usize > length {
                return Some(state.error("removeAt index out of range"));
            }
            list.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                list.remove_item_at((lua_index - 1) as i32)
            });
            Some(0)
        }
        LuaAtoms::RemoveAllOf => {
            if let Some(view_model) = state.to_rive_optional::<ScriptedViewModel>(2, true) {
                if let Some(instance) = view_model.view_model_instance() {
                    property
                        .instance_value_mut()
                        .unwrap()
                        .with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                            list.remove_all_items_with_view_model_instance(Some(instance))
                        });
                }
            }
            Some(0)
        }
        LuaAtoms::Values => {
            let data_enum = property
                .instance_value()
                .and_then(|value| {
                    value
                        .with(|value| {
                            value
                                .as_view_model_instance_value()
                                .and_then(ViewModelInstanceValue::view_model_property)
                        })
                        .flatten()
                })
                .and_then(|property| {
                    property
                        .with(|property| {
                            property
                                .as_view_model_property_enum()
                                .and_then(ViewModelPropertyEnum::data_enum)
                        })
                        .flatten()
                });
            state.new_rive(ScriptedEnumValues::new(state, data_enum));
            Some(1)
        }
        _ => None,
    }
}

fn view_model_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    let core_type = match atom {
        LuaAtoms::GetNumber => Some(ViewModelInstanceNumber::TYPE_KEY),
        LuaAtoms::GetTrigger => Some(ViewModelInstanceTrigger::TYPE_KEY),
        LuaAtoms::GetString => Some(ViewModelInstanceString::TYPE_KEY),
        LuaAtoms::GetBoolean => Some(ViewModelInstanceBoolean::TYPE_KEY),
        LuaAtoms::GetColor => Some(ViewModelInstanceColor::TYPE_KEY),
        LuaAtoms::GetList => Some(ViewModelInstanceList::TYPE_KEY),
        LuaAtoms::GetViewModel => Some(ViewModelInstanceViewModel::TYPE_KEY),
        LuaAtoms::GetEnum => Some(ViewModelInstanceEnum::TYPE_KEY),
        LuaAtoms::GetImage => Some(ViewModelInstanceAssetImage::TYPE_KEY),
        LuaAtoms::GetFont => Some(ViewModelInstanceAssetFont::TYPE_KEY),
        LuaAtoms::GetBlob => Some(ViewModelInstanceAssetBlob::TYPE_KEY),
        _ => None,
    };
    if let Some(core_type) = core_type {
        let property_name = state.check_string(2);
        return state
            .to_rive_mut::<ScriptedViewModel>(1)
            .push_value(&property_name, core_type);
    }
    match atom {
        LuaAtoms::GetIndex => state.to_rive_mut::<ScriptedViewModel>(1).push_index(),
        LuaAtoms::Instance | LuaAtoms::New => {
            state.to_rive_mut::<ScriptedViewModel>(1).instance(state)
        }
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedPropertyViewModel::LUA_NAME
        )),
    }
}

fn property_vm_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    let property = state.to_rive_mut::<ScriptedPropertyViewModel>(1);
    if let Some(result) = property_namecall_atom(state, &mut property.property, atom) {
        result
    } else {
        state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedPropertyViewModel::LUA_NAME
        ))
    }
}

fn property_namecall(state: &mut LuaState) -> i32 {
    let (method_name, atom) = state.namecall_atom();
    let tag = state.userdata_tag(1);
    let type_name = match tag {
        ScriptedPropertyNumber::LUA_TAG => ScriptedPropertyNumber::LUA_NAME,
        ScriptedPropertyTrigger::LUA_TAG => ScriptedPropertyTrigger::LUA_NAME,
        ScriptedPropertyColor::LUA_TAG => ScriptedPropertyColor::LUA_NAME,
        ScriptedPropertyString::LUA_TAG => ScriptedPropertyString::LUA_NAME,
        ScriptedPropertyBoolean::LUA_TAG => ScriptedPropertyBoolean::LUA_NAME,
        ScriptedPropertyEnum::LUA_TAG => ScriptedPropertyEnum::LUA_NAME,
        ScriptedPropertyList::LUA_TAG => ScriptedPropertyList::LUA_NAME,
        ScriptedPropertyImage::LUA_TAG => ScriptedPropertyImage::LUA_NAME,
        ScriptedPropertyFont::LUA_TAG => ScriptedPropertyFont::LUA_NAME,
        ScriptedPropertyBlob::LUA_TAG => ScriptedPropertyBlob::LUA_NAME,
        _ => return state.type_error(1, "Property"),
    };
    let result = match tag {
        ScriptedPropertyNumber::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyNumber>(1).property,
            atom,
        ),
        ScriptedPropertyTrigger::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyTrigger>(1).property,
            atom,
        ),
        ScriptedPropertyColor::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyColor>(1).property,
            atom,
        ),
        ScriptedPropertyString::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyString>(1).property,
            atom,
        ),
        ScriptedPropertyBoolean::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyBoolean>(1).property,
            atom,
        ),
        ScriptedPropertyEnum::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyEnum>(1).property,
            atom,
        ),
        ScriptedPropertyList::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyList>(1).property,
            atom,
        ),
        ScriptedPropertyImage::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyImage>(1).property,
            atom,
        ),
        ScriptedPropertyFont::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyFont>(1).property,
            atom,
        ),
        ScriptedPropertyBlob::LUA_TAG => property_namecall_atom(
            state,
            &mut state.to_rive_mut::<ScriptedPropertyBlob>(1).property,
            atom,
        ),
        _ => None,
    };
    if let Some(result) = result {
        result
    } else {
        state.error(format!(
            "{} is not a valid method of {}",
            method_name.unwrap_or_default(),
            type_name
        ))
    }
}

fn property_number_direct_value(userdata: *mut (), result: *mut LuaDirectFieldResult) {
    // SAFETY: Luau invokes this direct-field callback with userdata carrying
    // `ScriptedPropertyNumber::LUA_TAG` and a live out-result for this call.
    let property = unsafe { &mut *(userdata as *mut ScriptedPropertyNumber) };
    let value = property
        .property
        .instance_value()
        .and_then(ViewModelInstanceValue::as_number)
        .map(ViewModelInstanceNumber::property_value)
        .unwrap_or(0.0);
    unsafe { &mut *result }.set_number(value as f64);
}

fn property_color_direct_value(userdata: *mut (), result: *mut LuaDirectFieldResult) {
    // SAFETY: Luau's registered color-property tag establishes both pointer
    // types and keeps the userdata/result live for the callback duration.
    let property = unsafe { &mut *(userdata as *mut ScriptedPropertyColor) };
    let value = property
        .property
        .instance_value()
        .and_then(ViewModelInstanceValue::as_color)
        .map(ViewModelInstanceColor::property_value)
        .unwrap_or(0);
    unsafe { &mut *result }.set_number(value as u32 as f64);
}

fn property_boolean_direct_value(userdata: *mut (), result: *mut LuaDirectFieldResult) {
    // SAFETY: the callback is registered only for boolean-property userdata;
    // Luau supplies a valid same-call result out pointer.
    let property = unsafe { &mut *(userdata as *mut ScriptedPropertyBoolean) };
    let value = property
        .property
        .instance_value()
        .and_then(ViewModelInstanceValue::as_boolean)
        .map(ViewModelInstanceBoolean::property_value)
        .unwrap_or(false);
    unsafe { &mut *result }.set_boolean(value);
}

fn property_list_direct_length(userdata: *mut (), result: *mut LuaDirectFieldResult) {
    // SAFETY: the callback is registered only for list-property userdata and
    // both ABI pointers remain live and uniquely borrowed for this call.
    let property = unsafe { &mut *(userdata as *mut ScriptedPropertyList) };
    let length = property
        .property
        .instance_value()
        .and_then(ViewModelInstanceValue::as_list)
        .map(|list| list.list_items().len())
        .unwrap_or(0);
    unsafe { &mut *result }.set_number(length as f64);
}

fn property_list_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    let property = state.to_rive_mut::<ScriptedPropertyList>(1);
    if key.is_none() {
        return property.push_value((state.check_integer(2) - 1) as usize);
    }
    if atom == LuaAtoms::Length {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_length()
    } else {
        0
    }
}

fn enum_value_length(state: &mut LuaState) -> i32 {
    let values = state.to_rive::<ScriptedEnumValues>(1);
    assert!(std::ptr::eq(values.state, state));
    values.push_length()
}

fn enum_value_index(state: &mut LuaState) -> i32 {
    let (key, _) = state.to_string_atom(2);
    let values = state.to_rive::<ScriptedEnumValues>(1);
    if key.is_none() {
        values.push_value((state.check_integer(2) - 1) as i32)
    } else {
        0
    }
}

fn property_number_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyNumber>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_number_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let value = state.check_number(3) as f32;
        state
            .to_rive_mut::<ScriptedPropertyNumber>(1)
            .set_value(value);
    }
    0
}

fn property_color_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyColor>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_color_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let value = state.check_unsigned(3);
        state
            .to_rive_mut::<ScriptedPropertyColor>(1)
            .set_value(value);
    }
    0
}

fn property_string_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyString>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_string_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let value = state.check_string(3);
        state
            .to_rive_mut::<ScriptedPropertyString>(1)
            .set_value(value);
    }
    0
}

fn property_boolean_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyBoolean>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_boolean_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let value = state.check_boolean(3);
        state
            .to_rive_mut::<ScriptedPropertyBoolean>(1)
            .set_value(value);
    }
    0
}

fn property_enum_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyEnum>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_enum_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let value = state.check_string(3);
        state
            .to_rive_mut::<ScriptedPropertyEnum>(1)
            .set_value(value);
    }
    0
}

fn property_image_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyImage>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_image_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let image = state
            .to_rive_optional::<ScriptedImage>(3, true)
            .and_then(|image| image.image.clone());
        state
            .to_rive_mut::<ScriptedPropertyImage>(1)
            .set_value(image);
    }
    0
}

fn property_font_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyFont>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_font_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Value {
        let font = state
            .to_rive_optional::<ScriptedFont>(3, true)
            .and_then(|font| font.font.clone());
        state.to_rive_mut::<ScriptedPropertyFont>(1).set_value(font);
    }
    0
}

fn property_blob_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let property = state.to_rive_mut::<ScriptedPropertyBlob>(1);
    if atom == LuaAtoms::Value {
        assert!(std::ptr::eq(property.property.state(), state));
        property.push_value()
    } else {
        0
    }
}

fn property_blob_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    if atom != LuaAtoms::Value {
        return 0;
    }
    if state.is_nil(3) {
        state.to_rive_mut::<ScriptedPropertyBlob>(1).set_value(None);
        return 0;
    }
    if state.is_buffer(3) || state.type_of(3) == LuaType::String {
        let bytes = if state.is_buffer(3) {
            state.to_buffer(3).to_vec()
        } else {
            state.to_bytes(3).to_vec()
        };
        let mut blob = BlobAsset::default_rc();
        blob.decode(&bytes, None);
        state
            .to_rive_mut::<ScriptedPropertyBlob>(1)
            .set_value(Some(blob));
        return 0;
    }
    let blob = state
        .to_rive::<ScriptedBlob>(3)
        .asset
        .as_ref()
        .and_then(|asset| asset.as_blob_asset_rc());
    state.to_rive_mut::<ScriptedPropertyBlob>(1).set_value(blob);
    0
}

fn view_model_eq(state: &mut LuaState) -> i32 {
    let left = state.to_rive::<ScriptedViewModel>(1).view_model_instance();
    let right = state.to_rive::<ScriptedViewModel>(2).view_model_instance();
    state.push_boolean(left == right);
    1
}

fn register_property<T: LuaRive>(
    state: &mut LuaState,
    index: Option<LuaFunction>,
    new_index: Option<LuaFunction>,
    namecall: Option<LuaFunction>,
) {
    state.register_rive::<T>();
    if let Some(index) = index {
        state.push_function(index);
        state.set_field(-2, "__index");
    }
    if let Some(new_index) = new_index {
        state.push_function(new_index);
        state.set_field(-2, "__newindex");
    }
    if let Some(namecall) = namecall {
        state.push_function(namecall);
        state.set_field(-2, "__namecall");
    }
    state.set_readonly(-1, true);
    state.pop(1);
}

pub fn luaopen_rive_properties(state: &mut LuaState) -> i32 {
    state.register_rive::<ScriptedViewModel>();
    state.push_function(view_model_index);
    state.set_field(-2, "__index");
    state.push_function(view_model_namecall);
    state.set_field(-2, "__namecall");
    state.push_function(view_model_eq);
    state.set_field(-2, "__eq");
    state.set_readonly(-1, true);
    state.pop(1);

    register_property::<ScriptedPropertyViewModel>(
        state,
        Some(property_vm_index),
        Some(property_vm_newindex),
        Some(property_vm_namecall),
    );
    register_property::<ScriptedPropertyNumber>(
        state,
        Some(property_number_index),
        Some(property_number_newindex),
        Some(property_namecall),
    );
    state.register_userdata_direct_field_get(
        ScriptedPropertyNumber::LUA_TAG,
        "value",
        property_number_direct_value,
    );
    register_property::<ScriptedPropertyColor>(
        state,
        Some(property_color_index),
        Some(property_color_newindex),
        Some(property_namecall),
    );
    state.register_userdata_direct_field_get(
        ScriptedPropertyColor::LUA_TAG,
        "value",
        property_color_direct_value,
    );
    register_property::<ScriptedPropertyString>(
        state,
        Some(property_string_index),
        Some(property_string_newindex),
        Some(property_namecall),
    );
    register_property::<ScriptedPropertyBoolean>(
        state,
        Some(property_boolean_index),
        Some(property_boolean_newindex),
        Some(property_namecall),
    );
    state.register_userdata_direct_field_get(
        ScriptedPropertyBoolean::LUA_TAG,
        "value",
        property_boolean_direct_value,
    );
    register_property::<ScriptedPropertyEnum>(
        state,
        Some(property_enum_index),
        Some(property_enum_newindex),
        Some(property_namecall),
    );
    register_property::<ScriptedPropertyTrigger>(state, None, None, Some(property_namecall));
    register_property::<ScriptedPropertyList>(
        state,
        Some(property_list_index),
        None,
        Some(property_namecall),
    );
    state.register_userdata_direct_field_get(
        ScriptedPropertyList::LUA_TAG,
        "length",
        property_list_direct_length,
    );
    register_property::<ScriptedPropertyImage>(
        state,
        Some(property_image_index),
        Some(property_image_newindex),
        Some(property_namecall),
    );
    state.register_rive::<ScriptedFont>();
    register_property::<ScriptedPropertyFont>(
        state,
        Some(property_font_index),
        Some(property_font_newindex),
        Some(property_namecall),
    );
    register_property::<ScriptedPropertyBlob>(
        state,
        Some(property_blob_index),
        Some(property_blob_newindex),
        Some(property_namecall),
    );
    state.register_rive::<ScriptedEnumValues>();
    state.push_function(enum_value_index);
    state.set_field(-2, "__index");
    state.push_function(enum_value_length);
    state.set_field(-2, "__len");
    state.set_readonly(-1, true);
    state.pop(1);
    0
}
