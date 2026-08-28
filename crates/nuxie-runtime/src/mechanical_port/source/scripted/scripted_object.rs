use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::mechanical_port::source::{
    assets::file_asset_referencer::FileAssetReferencer, core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle, importers::import_stack::ImportStack,
    status_code::StatusCode,
};

use crate::scripting::{
    RuntimeScriptInstanceHandle, ScriptInstance as RuntimeScriptInstance,
    ScriptMethod as RuntimeScriptMethod, ScriptValue as RuntimeScriptValue,
};
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptProtocol {
    Utility = 0,
    Node = 1,
    Layout = 2,
    Converter = 3,
    PathEffect = 4,
    ListenerAction = 5,
    TransitionCondition = 6,
    Interpolator = 7,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptValue {
    Artboard(CoreHandle),
    Boolean(bool),
    Color(u32),
    Integer(i32),
    Number(f32),
    String(String),
    ViewModel(CoreHandle),
    Trigger,
}
pub const ADVANCES_BIT: u32 = 1 << 0;
pub const UPDATES_BIT: u32 = 1 << 1;
pub const MEASURES_BIT: u32 = 1 << 2;
pub const WANTS_POINTER_DOWN_BIT: u32 = 1 << 3;
pub const WANTS_POINTER_MOVE_BIT: u32 = 1 << 4;
pub const WANTS_POINTER_UP_BIT: u32 = 1 << 5;
pub const WANTS_POINTER_EXIT_BIT: u32 = 1 << 6;
pub const WANTS_POINTER_CANCEL_BIT: u32 = 1 << 7;
pub const DRAWS_BIT: u32 = 1 << 8;
pub const INITS_BIT: u32 = 1 << 9;
pub const DATA_CONVERTS_BIT: u32 = 1 << 10;
pub const DATA_REVERSE_CONVERTS_BIT: u32 = 1 << 11;
pub const RESIZES_BIT: u32 = 1 << 12;
pub const LISTENER_PERFORMS_BIT: u32 = 1 << 13;
pub const LISTENER_PERFORMS_ACTION_BIT: u32 = 1 << 14;
pub const DRAWS_CANVAS_BIT: u32 = 1 << 15;
pub const WANTS_KEYBOARD_INPUT_BIT: u32 = 1 << 16;
pub const WANTS_TEXT_INPUT_BIT: u32 = 1 << 17;
pub const WANTS_GAMEPAD_CONNECT_BIT: u32 = 1 << 18;
pub const WANTS_GAMEPAD_DISCONNECT_BIT: u32 = 1 << 19;
pub const WANTS_GAMEPAD_EVENT_BIT: u32 = 1 << 20;
pub const METHOD_MASK: u32 = (1 << 21) - 1;

/// One arena-owned scripted clone and the cloned bindings its runtime host
/// must retain in its `DataBindContainer`.
pub struct ScriptedObjectClone {
    pub owner: CoreHandle,
    pub data_binds: Vec<CoreHandle>,
}

/// Callback-local marker. Applying it after the VM returns avoids borrowing
/// the authored owner again while its callback is active.
#[derive(Default)]
pub struct ScriptUpdateRequestHost {
    requested: bool,
}

impl crate::scripting::ScriptHost for ScriptUpdateRequestHost {
    fn mark_script_update(&mut self) {
        self.requested = true;
    }
}

impl ScriptUpdateRequestHost {
    pub fn take_requested(&mut self) -> bool {
        std::mem::take(&mut self.requested)
    }
}

pub struct ScriptedObject {
    file_asset_referencer: FileAssetReferencer,
    self_ref: i32,
    context_ref: i32,
    runtime_instance: Option<RuntimeScriptInstanceHandle>,
    runtime_vm: Option<crate::mechanical_port::source::lua::scripting_vm::RuntimeScriptingVmHandle>,
    asset_id: u32,
    asset: Option<Rc<[u8]>>,
    inputs: HashMap<String, ScriptValue>,
    tracked_properties: Vec<usize>,
    data_context: Option<RuntimeDataContextHandle>,
    in_update_phase: bool,
    user_init_done: bool,
    disposed: bool,
    implemented_methods: u32,
    needs_update: bool,
    callback_update_requested: bool,
}
impl Default for ScriptedObject {
    fn default() -> Self {
        Self {
            file_asset_referencer: FileAssetReferencer::default(),
            self_ref: 0,
            context_ref: 0,
            runtime_instance: None,
            runtime_vm: None,
            asset_id: u32::MAX,
            asset: None,
            inputs: HashMap::new(),
            tracked_properties: Vec::new(),
            data_context: None,
            in_update_phase: false,
            user_init_done: false,
            disposed: false,
            implemented_methods: 0,
            needs_update: false,
            callback_update_requested: false,
        }
    }
}
impl ScriptedObject {
    /// The non-component scripted owners delete only ScriptInput-derived
    /// properties. Clear every input's backlink before removing any owner.
    pub fn dispose_owned_script_inputs(properties: &mut Vec<CoreHandle>) {
        let inputs: Vec<_> = properties
            .iter()
            .filter(|property| {
                property
                    .with_mut(|property| {
                        let Some(input) = property.as_bind_script_input_mut() else {
                            return false;
                        };
                        input.set_scripted_object(None);
                        true
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        properties.clear();
        for input in inputs {
            input.remove_occurrence();
        }
    }

    /// Pinned ScriptedObject::cloneProperties, with each generated clone
    /// installed in the same arena before its links are established.
    pub fn clone_properties(
        properties: &[CoreHandle],
        owner: &CoreHandle,
        container: &mut crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainer,
    ) -> Vec<CoreHandle> {
        Self::clone_properties_with(properties, owner, |bind| container.add_data_bind(bind))
    }

    /// Same upstream operation with a short-borrow container attachment, for
    /// artboards whose callbacks may access the containing authored arena.
    pub fn clone_properties_with(
        properties: &[CoreHandle],
        owner: &CoreHandle,
        mut add_data_bind: impl FnMut(CoreHandle),
    ) -> Vec<CoreHandle> {
        let mut clones = Vec::with_capacity(properties.len());
        for property in properties {
            let clone = property
                .clone_occurrence()
                .expect("a retained scripted property has a concrete clone");
            let added = owner.with_mut(|owner| owner.scripted_object_add_property(clone.clone()));
            assert_eq!(
                added,
                Some(true),
                "the scripted clone retains its property container"
            );
            let source_bind = property
                .with(|property| property.script_input_data_bind())
                .flatten();
            if let Some(source_bind) = source_bind {
                let bind = source_bind
                    .clone_occurrence()
                    .expect("a retained scripted input bind has a concrete clone");
                let (file, converter) = source_bind
                    .with(|bind| {
                        let bind = bind.as_data_bind().expect("the input retains a DataBind");
                        (bind.file(), bind.converter())
                    })
                    .expect("the input bind remains live");
                bind.with_mut(|bind| {
                    bind.as_data_bind_mut()
                        .expect("a cloned DataBind keeps its type")
                        .set_file(file);
                })
                .expect("the cloned input bind remains live");
                let converter = converter.map(|converter| {
                    converter
                        .clone_occurrence()
                        .expect("a retained scripted input converter has a concrete clone")
                });
                bind.with_mut(|bind| {
                    let bind = bind
                        .as_data_bind_mut()
                        .expect("a cloned DataBind keeps its type");
                    bind.set_converter(converter);
                    bind.set_target(Some(clone.clone()));
                })
                .expect("the cloned input bind remains live");
                add_data_bind(bind.clone());
                let attached = clone
                    .with_mut(|property| property.script_input_set_data_bind(Some(bind), false));
                assert_eq!(attached, Some(true), "a cloned script input keeps its type");
            }
            clones.push(clone);
        }
        clones
    }

    pub fn apply_update_request(owner: &CoreHandle) {
        use crate::mechanical_port::source::scripted::{
            scripted_drawable::ScriptedDrawable, scripted_layout::ScriptedLayout,
            scripted_path_effect::ScriptedPathEffect,
        };
        owner.with_mut(|owner| {
            if let Some(drawable) = owner.as_any_mut().downcast_mut::<ScriptedDrawable>() {
                drawable.mark_needs_update();
            } else if let Some(layout) = owner.as_any_mut().downcast_mut::<ScriptedLayout>() {
                layout.base.base.mark_needs_update();
            } else if let Some(effect) = owner.as_any_mut().downcast_mut::<ScriptedPathEffect>() {
                effect.mark_needs_update();
            }
            // Other scripted owners inherit ScriptedObject::markNeedsUpdate,
            // whose pinned implementation is intentionally empty.
        });
    }

    pub fn call_interpolator(
        &self,
        method: crate::scripting::ScriptInterpolatorMethod,
        args: &[f32],
    ) -> Option<f32> {
        use crate::scripting::ScriptOptionalNumberResult;
        if self.self_ref == 0 {
            return None;
        }
        let instance = self.runtime_instance.as_ref()?;
        // ScriptedInterpolator inherits the empty markNeedsUpdate; retain the
        // real callback request without inventing a component dirt effect.
        let mut host = ScriptUpdateRequestHost::default();
        match instance
            .borrow_mut()
            .call_interpolator(method, args, &mut host)
            .ok()?
        {
            ScriptOptionalNumberResult::Missing => None,
            ScriptOptionalNumberResult::Returned(value) => Some(value),
        }
    }

    pub fn initialize_occurrence(
        owner: &CoreHandle,
        properties: &[CoreHandle],
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> bool {
        use crate::mechanical_port::source::assets::script_asset::ScriptAsset;
        let Some(asset) = owner
            .with(|owner| owner.as_scripted_object().and_then(Self::script_asset))
            .flatten()
        else {
            return false;
        };
        let Some((vm, methods)) = asset
            .with_downcast_mut::<ScriptAsset, _>(|asset| {
                let vm = asset.scripting_vm()?;
                Some((vm, asset.prepare_implemented_methods()))
            })
            .flatten()
        else {
            return false;
        };
        // ScriptAsset updates these even when ensureScriptInitialized can
        // reuse the existing table in the same VM (e.g. editor live editing).
        owner.with_mut(|owner| {
            owner
                .as_scripted_object_mut()
                .expect("a scripted owner keeps its type")
                .set_implemented_methods(methods)
        });
        let live = owner
            .with(|owner| {
                let scripted = owner
                    .as_scripted_object()
                    .expect("a scripted owner keeps its type");
                scripted.self_ref != 0
                    && scripted
                        .runtime_vm
                        .as_ref()
                        .is_some_and(|current| current.ptr_eq(&vm))
                    && scripted
                        .runtime_instance
                        .as_ref()
                        .is_some_and(|instance| instance.borrow_mut().script_lifetime_valid())
            })
            .unwrap_or(false);
        if !live {
            owner.with_mut(|owner| {
                owner
                    .as_scripted_object_mut()
                    .expect("a scripted owner keeps its type")
                    .reinit();
            });
            for property in properties {
                if with_script_input(property, |input| input.validate_for_cold_script_init())
                    == Some(false)
                {
                    return false;
                }
            }
            let Some((instance, methods)) = ScriptAsset::instantiate_for_occurrence(&asset, host)
            else {
                return false;
            };
            owner
                .with_mut(|owner| {
                    let scripted = owner
                        .as_scripted_object_mut()
                        .expect("a scripted owner keeps its type");
                    scripted.install_script_instance(instance, vm);
                    scripted.set_implemented_methods(methods);
                })
                .expect("the stateful owner remains live");
        }
        true
    }

    /// Input hydration calls back into this same occurrence, so no owner
    /// borrow crosses validation or user initialization.
    pub fn hydrate_occurrence(
        owner: &CoreHandle,
        properties: &[CoreHandle],
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> bool {
        let live = owner
            .with(|owner| {
                owner.as_scripted_object().is_some_and(|scripted| {
                    scripted.self_ref != 0 && scripted.runtime_instance.is_some()
                })
            })
            .unwrap_or(false);
        if !live {
            return false;
        }
        for property in properties {
            if with_script_input(property, |input| input.validate_hydration_prerequisites())
                == Some(false)
            {
                return false;
            }
        }
        for property in properties {
            if with_script_input(property, |input| input.hydrate_script_input()) == Some(false) {
                return false;
            }
        }
        // The runtime instance is cloned out before invoking user code, so
        // callbacks may resolve the authored owner without a RefCell reborrow.
        let (instance, needs_init) = owner
            .with(|owner| {
                let scripted = owner
                    .as_scripted_object()
                    .expect("a scripted owner keeps its type");
                (
                    scripted.runtime_instance.clone(),
                    scripted.implemented_methods & INITS_BIT != 0 && !scripted.user_init_done,
                )
            })
            .expect("the reinitialized owner remains live");
        if needs_init {
            let Some(instance) = instance else {
                return false;
            };
            if !matches!(instance.borrow_mut().call_init(host), Ok(true)) {
                // A failed user init invalidates the self/context lifetime;
                // the next reinit must run the generator again.
                owner.with_mut(|owner| {
                    owner
                        .as_scripted_object_mut()
                        .expect("a scripted owner keeps its type")
                        .script_dispose();
                });
                return false;
            }
            owner.with_mut(|owner| {
                owner
                    .as_scripted_object_mut()
                    .expect("a scripted owner keeps its type")
                    .user_init_done = true;
            });
        }
        owner.with_mut(|owner| {
            use crate::mechanical_port::source::scripted::{
                scripted_data_converter::ScriptedDataConverter,
                scripted_drawable::ScriptedDrawable, scripted_layout::ScriptedLayout,
                scripted_path_effect::ScriptedPathEffect,
            };
            if let Some(value) = owner.as_any_mut().downcast_mut::<ScriptedDataConverter>() {
                value.did_hydrate_script_inputs();
            } else if let Some(value) = owner.as_any_mut().downcast_mut::<ScriptedLayout>() {
                value.did_hydrate_script_inputs();
            } else if let Some(value) = owner.as_any_mut().downcast_mut::<ScriptedDrawable>() {
                value.did_hydrate_script_inputs();
            } else if let Some(value) = owner.as_any_mut().downcast_mut::<ScriptedPathEffect>() {
                value.did_hydrate_script_inputs();
            }
        });
        true
    }

    pub fn reinit_occurrence(
        owner: &CoreHandle,
        properties: &[CoreHandle],
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> bool {
        let has_asset = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .is_some_and(|scripted| scripted.script_asset().is_some())
            })
            .unwrap_or(false);
        if !has_asset {
            return false;
        }
        Self::initialize_occurrence(owner, properties, host);
        Self::hydrate_occurrence(owner, properties, host)
    }

    pub fn custom_properties(owner: &CoreHandle) -> Vec<CoreHandle> {
        use crate::mechanical_port::source::{
            animation::{
                scripted_listener_action::ScriptedListenerAction,
                scripted_transition_condition::ScriptedTransitionCondition,
            },
            scripted::{
                scripted_data_converter::ScriptedDataConverter,
                scripted_drawable::ScriptedDrawable, scripted_interpolator::ScriptedInterpolator,
                scripted_layout::ScriptedLayout, scripted_path_effect::ScriptedPathEffect,
            },
        };
        owner
            .with(|owner| {
                if let Some(value) = owner.as_any().downcast_ref::<ScriptedListenerAction>() {
                    return value.properties.clone();
                }
                if let Some(value) = owner.as_any().downcast_ref::<ScriptedTransitionCondition>() {
                    return value.properties.clone();
                }
                if let Some(value) = owner.as_any().downcast_ref::<ScriptedDrawable>() {
                    return value.properties.clone();
                }
                if let Some(value) = owner.as_any().downcast_ref::<ScriptedLayout>() {
                    return value.base.base.properties.clone();
                }
                if let Some(value) = owner.as_any().downcast_ref::<ScriptedDataConverter>() {
                    return value.properties.clone();
                }
                if let Some(value) = owner.as_any().downcast_ref::<ScriptedInterpolator>() {
                    return value.properties.clone();
                }
                if let Some(value) = owner.as_any().downcast_ref::<ScriptedPathEffect>() {
                    return value.properties.clone();
                }
                panic!("a scripted occurrence must retain a concrete custom-property owner");
            })
            .expect("a retained scripted occurrence remains live")
    }

    pub fn perform_listener_action(
        owner: &CoreHandle,
        invocation: &crate::state_machine::ScriptListenerInvocation,
        host: &mut dyn crate::scripting::ScriptHost,
    ) {
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .and_then(|scripted| scripted.runtime_instance.clone())
            })
            .flatten();
        if let Some(instance) = instance {
            // A callback error does not cause a second legacy invocation.
            let _ = instance
                .borrow_mut()
                .call_preferred_listener_action(invocation, host);
        }
    }

    pub fn evaluate_condition(
        owner: &CoreHandle,
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> bool {
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .and_then(|scripted| scripted.runtime_instance.clone())
            })
            .flatten();
        instance.is_some_and(|instance| {
            matches!(
                instance
                    .borrow_mut()
                    .call_method(RuntimeScriptMethod::Evaluate, &[], host),
                Ok(RuntimeScriptValue::Bool(true))
            )
        })
    }

    pub fn perform_pointer(
        owner: &CoreHandle,
        method: RuntimeScriptMethod,
        pointer_id: i32,
        local_position: crate::mechanical_port::source::math::vec2d::Vec2D,
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> crate::scripting::ScriptedDrawablePointerResult {
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .and_then(|scripted| scripted.runtime_instance.clone())
            })
            .flatten();
        let Some(instance) = instance else {
            return crate::scripting::ScriptedDrawablePointerResult::default();
        };
        instance
            .borrow_mut()
            .call_scripted_drawable_pointer(
                method,
                pointer_id,
                local_position.x,
                local_position.y,
                host,
            )
            .unwrap_or_default()
    }

    pub(crate) fn file_asset_referencer_mut(&mut self) -> &mut FileAssetReferencer {
        &mut self.file_asset_referencer
    }

    pub fn runtime_instance(&self) -> Option<RuntimeScriptInstanceHandle> {
        self.runtime_instance.clone()
    }

    fn add_input_dirt(owner: &CoreHandle) {
        use crate::mechanical_port::source::{
            component::ComponentOccurrenceHandle, component_dirt::ComponentDirt,
            scripted::scripted_data_converter::ScriptedDataConverter,
        };
        let is_component = owner
            .with(|owner| owner.as_component().is_some())
            .expect("the scripted owner remains live during input assignment");
        if is_component {
            ComponentOccurrenceHandle::Authored(owner.clone())
                .add_dirt(ComponentDirt::SCRIPT_UPDATE, false);
        } else {
            owner.with_downcast_mut::<ScriptedDataConverter, _>(|converter| {
                converter.add_scripted_dirt(u32::from(ComponentDirt::SCRIPT_UPDATE.0), false);
            });
            // Actions, conditions, and interpolators intentionally add no dirt.
        }
    }

    pub fn set_primitive_input(owner: &CoreHandle, name: String, value: ScriptValue) {
        use crate::scripting::ScriptCoreString;
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .filter(|scripted| scripted.self_ref != 0)
                    .and_then(|scripted| scripted.runtime_instance())
            })
            .flatten();
        let Some(instance) = instance else {
            return;
        };
        let value =
            runtime_script_value(&value).expect("primitive input dispatch takes a primitive value");
        let assigned = instance
            .borrow_mut()
            .set_input_core(&ScriptCoreString::from_bytes(name.into_bytes()), value)
            .is_ok();
        if assigned {
            Self::add_input_dirt(owner);
        }
    }

    pub fn trigger_occurrence(owner: &CoreHandle, name: String) {
        use crate::scripting::ScriptCoreString;
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .filter(|scripted| scripted.self_ref != 0)
                    .and_then(|scripted| scripted.runtime_instance())
            })
            .flatten();
        let Some(instance) = instance else {
            return;
        };
        let mut host = ScriptUpdateRequestHost::default();
        // The native trigger adapter marks this host only after it finds and
        // attempts a function, including an ordinary protected-call failure.
        let _ = instance
            .borrow_mut()
            .call_input_trigger_core(&ScriptCoreString::from_bytes(name.into_bytes()), &mut host);
        if host.take_requested() {
            Self::add_input_dirt(owner);
        }
    }

    pub fn set_view_model_input_occurrence(owner: &CoreHandle, name: String, value: CoreHandle) {
        use crate::mechanical_port::source::viewmodel::viewmodel_instance_viewmodel::ViewModelInstanceViewModel;
        use crate::scripting::{ScriptCoreString, ScriptViewModel};
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .filter(|scripted| scripted.self_ref != 0)
                    .and_then(Self::runtime_instance)
            })
            .flatten();
        let Some(instance) = instance else {
            return;
        };
        let view_model = value
            .with_downcast::<ViewModelInstanceViewModel, _>(
                ViewModelInstanceViewModel::reference_view_model_instance,
            )
            .flatten();
        let Some(view_model) = view_model else {
            eprintln!(
                "riveLuaPushViewModelInstanceValue - passed in a ViewModelInstanceViewModel with no associated ViewModelInstance."
            );
            return;
        };
        let definition = view_model
            .with(|value| {
                value
                    .as_view_model_instance()
                    .and_then(|value| value.get_view_model())
            })
            .flatten()
            .expect("bound view-model instance has a definition");
        let file = definition
            .with(|model| model.as_view_model().unwrap().file())
            .and_then(|file| file.upgrade())
            .expect("bound view-model definition retains a live file");
        let facade = ScriptViewModel::from_native(view_model, file)
            .expect("resolved native view-model can be projected");
        let assigned = instance
            .borrow_mut()
            .set_view_model_input_core(&ScriptCoreString::from_bytes(name.into_bytes()), facade)
            .is_ok();
        if assigned {
            Self::add_input_dirt(owner);
        }
    }

    pub fn install_script_instance(
        &mut self,
        instance: Box<dyn RuntimeScriptInstance>,
        vm: crate::mechanical_port::source::lua::scripting_vm::RuntimeScriptingVmHandle,
    ) {
        self.script_dispose();
        self.runtime_instance = Some(RuntimeScriptInstanceHandle::new(instance));
        self.runtime_vm = Some(vm);
        self.self_ref = 1;
        self.context_ref = 1;
        self.user_init_done = false;
        self.disposed = false;
    }

    fn set(&mut self, name: String, value: ScriptValue) {
        if self.self_ref == 0 {
            return;
        }
        let Some(instance) = self.runtime_instance.clone() else {
            return;
        };
        let Some(converted) = runtime_script_value(&value) else {
            return;
        };
        if instance.borrow_mut().set_input(&name, converted).is_ok() {
            self.inputs.insert(name, value);
            self.mark_needs_update();
        }
    }
    pub fn set_artboard_input(&mut self, n: String, v: CoreHandle) {
        self.set(n, ScriptValue::Artboard(v))
    }
    pub fn set_boolean_input(&mut self, n: String, v: bool) {
        self.set(n, ScriptValue::Boolean(v))
    }
    pub fn set_integer_input(&mut self, n: String, v: i32) {
        self.set(n, ScriptValue::Integer(v))
    }
    pub fn set_number_input(&mut self, n: String, v: f32) {
        self.set(n, ScriptValue::Number(v))
    }
    pub fn set_string_input(&mut self, n: String, v: String) {
        self.set(n, ScriptValue::String(v))
    }
    pub fn set_view_model_input(&mut self, n: String, v: CoreHandle) {
        self.set(n, ScriptValue::ViewModel(v))
    }

    pub fn script_advance(&mut self, elapsed: f32) -> bool {
        if !self.advances() || self.self_ref == 0 {
            return false;
        }
        let Some(instance) = self.runtime_instance.clone() else {
            return false;
        };
        let mut host = ScriptUpdateRequestHost::default();
        let advanced = instance
            .borrow_mut()
            .call_advance_truthy(elapsed, &mut host)
            .unwrap_or(false);
        self.callback_update_requested |= host.take_requested();
        advanced
    }

    /// The most-derived owner applies markNeedsUpdate after releasing the VM
    /// borrow. This marker carries only callback effects, not authored dirt.
    pub fn take_update_request(&mut self) -> bool {
        std::mem::take(&mut self.callback_update_requested)
    }
    pub fn draw_canvas_occurrence(owner: &CoreHandle, factory: &mut dyn nuxie_render_api::Factory) {
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .filter(|scripted| scripted.draws_canvas() && scripted.self_ref != 0)
                    .and_then(|scripted| scripted.runtime_instance())
            })
            .flatten();
        if let Some(instance) = instance {
            let _ = instance.borrow_mut().call_draw_canvas(factory);
        }
    }

    pub fn script_update_occurrence(owner: &CoreHandle) {
        let instance = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .filter(|scripted| scripted.updates() && scripted.self_ref != 0)
                    .and_then(|scripted| scripted.runtime_instance())
            })
            .flatten();
        let Some(instance) = instance else {
            return;
        };
        owner.with_mut(|owner| {
            owner
                .as_scripted_object_mut()
                .unwrap()
                .set_in_update_phase(true)
        });
        let mut host = ScriptUpdateRequestHost::default();
        let _ =
            instance
                .borrow_mut()
                .call_optional_method(RuntimeScriptMethod::Update, &[], &mut host);
        // markNeedsUpdate requests made in the update phase are intentionally ignored.
        owner.with_mut(|owner| {
            owner
                .as_scripted_object_mut()
                .unwrap()
                .set_in_update_phase(false)
        });
    }
    pub fn script_dispose(&mut self) {
        if self.disposed {
            return;
        }
        if let Some(instance) = self.runtime_instance.take() {
            instance.borrow_mut().invalidate_for_init_retry();
        }
        self.self_ref = 0;
        self.context_ref = 0;
        self.inputs.clear();
        self.tracked_properties.clear();
        self.runtime_vm = None;
        self.user_init_done = false;
        self.callback_update_requested = false;
        self.disposed = true;
    }
    pub fn reinit(&mut self) {
        self.script_dispose();
        self.disposed = false;
        self.user_init_done = false
    }
    pub fn script_asset(&self) -> Option<CoreHandle> {
        self.file_asset_referencer.asset()
    }
    pub fn register_referencer(
        &mut self,
        owner: CoreHandle,
        import_stack: &mut ImportStack,
    ) -> StatusCode {
        self.file_asset_referencer
            .register_referencer(owner, import_stack)
    }
    pub fn set_asset(&mut self, owner: CoreHandle, asset: Option<CoreHandle>) {
        // The pinned override ignores null and non-ScriptAsset values and
        // does not dispose the current VM lifetime merely on assignment.
        if asset.as_ref().is_some_and(|asset| asset.is_type_of(
            crate::mechanical_port::source::generated::assets::script_asset_base::ScriptAssetBase::TYPE_KEY,
        )) {
            self.file_asset_referencer.set_asset(owner, asset);
        }
    }
    pub fn detach_asset(&mut self, owner: CoreHandle) {
        self.file_asset_referencer.detach(owner);
    }
    pub fn set_script_payload(&mut self, id: u32, a: Option<Rc<[u8]>>) {
        self.script_dispose();
        self.asset_id = id;
        self.asset = a;
        self.disposed = false
    }
    pub fn add_tracked_property(&mut self, p: usize) {
        if p != 0 {
            self.tracked_properties.push(p)
        }
    }
    pub fn remove_tracked_property(&mut self, p: usize) {
        self.tracked_properties.retain(|v| *v != p)
    }
    pub fn tracked_properties(&self) -> &[usize] {
        &self.tracked_properties
    }
    pub fn self_ref(&self) -> i32 {
        self.self_ref
    }
    pub fn data_context(&self) -> Option<RuntimeDataContextHandle> {
        self.data_context.clone()
    }
    pub fn set_data_context(&mut self, v: Option<RuntimeDataContextHandle>) {
        self.data_context = v
    }
    pub fn in_update_phase(&self) -> bool {
        self.in_update_phase
    }
    pub fn set_in_update_phase(&mut self, value: bool) {
        self.in_update_phase = value;
    }
    pub fn mark_needs_update(&mut self) {
        self.needs_update = true;
    }
    pub fn needs_update(&self) -> bool {
        self.needs_update
    }
    pub fn set_implemented_methods(&mut self, implemented: u32) {
        self.implemented_methods = implemented;
    }
    pub fn implemented_methods(&self) -> u32 {
        self.implemented_methods
    }
    pub fn advances(&self) -> bool {
        self.implemented_methods & ADVANCES_BIT != 0
    }
    pub fn updates(&self) -> bool {
        self.implemented_methods & UPDATES_BIT != 0
    }
    pub fn measures(&self) -> bool {
        self.implemented_methods & MEASURES_BIT != 0
    }
    pub fn resizes(&self) -> bool {
        self.implemented_methods & RESIZES_BIT != 0
    }
    pub fn draws(&self) -> bool {
        self.implemented_methods & DRAWS_BIT != 0
    }
    pub fn data_converts(&self) -> bool {
        self.implemented_methods & DATA_CONVERTS_BIT != 0
    }
    pub fn data_reverse_converts(&self) -> bool {
        self.implemented_methods & DATA_REVERSE_CONVERTS_BIT != 0
    }
    pub fn performs(&self) -> bool {
        self.implemented_methods & LISTENER_PERFORMS_BIT != 0
    }
    pub fn performs_action(&self) -> bool {
        self.implemented_methods & LISTENER_PERFORMS_ACTION_BIT != 0
    }
    pub fn draws_canvas(&self) -> bool {
        self.implemented_methods & DRAWS_CANVAS_BIT != 0
    }
    pub fn wants_keyboard_input(&self) -> bool {
        self.implemented_methods & WANTS_KEYBOARD_INPUT_BIT != 0
    }
    pub fn wants_text_input(&self) -> bool {
        self.implemented_methods & WANTS_TEXT_INPUT_BIT != 0
    }
    pub fn wants_pointer_down(&self) -> bool {
        self.implemented_methods & WANTS_POINTER_DOWN_BIT != 0
    }
    pub fn wants_pointer_move(&self) -> bool {
        self.implemented_methods & WANTS_POINTER_MOVE_BIT != 0
    }
    pub fn wants_pointer_up(&self) -> bool {
        self.implemented_methods & WANTS_POINTER_UP_BIT != 0
    }
    pub fn wants_pointer_exit(&self) -> bool {
        self.implemented_methods & WANTS_POINTER_EXIT_BIT != 0
    }
    pub fn wants_pointer_cancel(&self) -> bool {
        self.implemented_methods & WANTS_POINTER_CANCEL_BIT != 0
    }
    pub fn wants_gamepad_connect(&self) -> bool {
        self.implemented_methods & WANTS_GAMEPAD_CONNECT_BIT != 0
    }
    pub fn wants_gamepad_disconnect(&self) -> bool {
        self.implemented_methods & WANTS_GAMEPAD_DISCONNECT_BIT != 0
    }
    pub fn wants_gamepad_event(&self) -> bool {
        self.implemented_methods & WANTS_GAMEPAD_EVENT_BIT != 0
    }
    pub fn listens_to_pointer_events(&self) -> bool {
        self.implemented_methods
            & (WANTS_POINTER_DOWN_BIT
                | WANTS_POINTER_MOVE_BIT
                | WANTS_POINTER_UP_BIT
                | WANTS_POINTER_EXIT_BIT
                | WANTS_POINTER_CANCEL_BIT
                | WANTS_GAMEPAD_CONNECT_BIT
                | WANTS_GAMEPAD_DISCONNECT_BIT
                | WANTS_GAMEPAD_EVENT_BIT)
            != 0
    }
    pub fn clear_scripting_vm(&mut self) {
        self.runtime_instance = None;
        self.runtime_vm = None;
        self.self_ref = 0;
        self.context_ref = 0;
    }
    pub fn user_lua_init_done(&self) -> bool {
        self.user_init_done
    }
}

fn runtime_script_value(value: &ScriptValue) -> Option<RuntimeScriptValue> {
    match value {
        ScriptValue::Boolean(value) => Some(RuntimeScriptValue::Bool(*value)),
        ScriptValue::Color(value) => Some(RuntimeScriptValue::Color(*value)),
        ScriptValue::Integer(value) => Some(RuntimeScriptValue::Number(f64::from(*value as u32))),
        ScriptValue::Number(value) => Some(RuntimeScriptValue::Number(f64::from(*value))),
        ScriptValue::String(value) => Some(RuntimeScriptValue::CoreString(
            crate::scripting::ScriptCoreString::from_bytes(value.as_bytes().to_vec()),
        )),
        ScriptValue::Artboard(_) | ScriptValue::ViewModel(_) | ScriptValue::Trigger => None,
    }
}
impl Drop for ScriptedObject {
    fn drop(&mut self) {
        self.script_dispose()
    }
}

fn with_script_input<R>(
    property: &CoreHandle,
    use_input: impl FnOnce(
        &mut dyn crate::mechanical_port::source::assets::script_asset::ScriptInputBehavior,
    ) -> R,
) -> Option<R> {
    property.with_mut(|property| {
        if let Some(input) = property.as_any_mut().downcast_mut::<crate::mechanical_port::source::script_input_artboard::ScriptInputArtboard>() {
            return Some(use_input(input));
        }
        if let Some(input) = property.as_any_mut().downcast_mut::<crate::mechanical_port::source::script_input_boolean::ScriptInputBoolean>() {
            return Some(use_input(input));
        }
        if let Some(input) = property.as_any_mut().downcast_mut::<crate::mechanical_port::source::script_input_color::ScriptInputColor>() {
            return Some(use_input(input));
        }
        if let Some(input) = property.as_any_mut().downcast_mut::<crate::mechanical_port::source::script_input_number::ScriptInputNumber>() {
            return Some(use_input(input));
        }
        if let Some(input) = property.as_any_mut().downcast_mut::<crate::mechanical_port::source::script_input_string::ScriptInputString>() {
            return Some(use_input(input));
        }
        if let Some(input) = property.as_any_mut().downcast_mut::<crate::mechanical_port::source::script_input_trigger::ScriptInputTrigger>() {
            return Some(use_input(input));
        }
        if let Some(input) = property.as_any_mut().downcast_mut::<crate::mechanical_port::source::script_input_viewmodel_property::ScriptInputViewModelProperty>() {
            return Some(use_input(input));
        }
        None
    }).flatten()
}
