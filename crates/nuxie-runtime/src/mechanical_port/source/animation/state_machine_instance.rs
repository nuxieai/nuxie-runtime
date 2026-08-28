use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::{
        animation_reset::AnimationReset,
        animation_reset_factory::AnimationResetFactory,
        animation_state::AnimationState,
        focus_listener_group::RuntimeFocusListenerGroupHandle,
        gamepad_listener_group::RuntimeGamepadListenerGroupHandle,
        keyboard_listener_group::RuntimeKeyboardListenerGroupHandle,
        linear_animation_instance::LinearAnimationInstance,
        listener_invocation::ListenerInvocation,
        listener_types::listener_input_type_semantic::ListenerInputTypeSemantic,
        listener_types::listener_input_type_viewmodel::ListenerInputTypeViewModel,
        semantic_listener_group::{RuntimeSemanticListenerGroupHandle, SemanticActionType},
        state_instance::RuntimeStateInstanceHandle,
        state_machine::StateMachine,
        state_machine_bool::StateMachineBool,
        state_machine_input_instance::{
            InputInstanceNotifier, SMIBool, SMIInput, SMINumber, SMITrigger,
        },
        state_machine_listener::StateMachineListener,
        state_machine_listener_single::StateMachineListenerSingle,
        state_machine_number::StateMachineNumber,
        state_machine_trigger::StateMachineTrigger,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    data_bind::{
        data_bind_container::DataBindContainer,
        data_context::{DataContext, RuntimeDataContextHandle},
    },
    dirtyable::Dirtyable,
    focus_data::FocusData,
    generated::{
        animation::{
            keyframe_bool_base::KeyFrameBoolBase, keyframe_color_base::KeyFrameColorBase,
            keyframe_double_base::KeyFrameDoubleBase, keyframe_string_base::KeyFrameStringBase,
            state_transition_base::StateTransitionBase,
        },
        data_bind::{
            bindable_property_boolean_base::BindablePropertyBooleanBase,
            bindable_property_color_base::BindablePropertyColorBase,
            bindable_property_number_base::BindablePropertyNumberBase,
            bindable_property_string_base::BindablePropertyStringBase,
        },
    },
    hit_result::HitResult,
    input::{
        focus_manager::{FocusManager, RuntimeFocusManagerHandle},
        focus_node::FocusNodeRef,
    },
    listener_group::{ListenerGroup, ListenerGroupProvider, RuntimeListenerGroupHandle},
    listener_type::ListenerType,
    math::{random::RandomProvider, vec2d::Vec2D},
    process_event_result::ProcessEventResult,
    semantic::{
        semantic_manager::{RuntimeSemanticManagerHandle, SemanticManager},
        semantic_node::SemanticNodeRef,
    },
    view_model_type::ViewModelType,
    viewmodel::{
        viewmodel::ViewModel,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
        viewmodel_instance_value::{ValueDependentHandle, ViewModelInstanceValue},
        viewmodel_value_dependent::ViewModelValueDependent,
    },
};
use std::{
    cell::{Cell, RefCell, RefMut},
    collections::HashMap,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeObjectHandle {
    slot: u32,
    generation: u32,
}

impl Default for RuntimeObjectHandle {
    fn default() -> Self {
        Self::NONE
    }
}

impl RuntimeObjectHandle {
    pub const NONE: Self = Self {
        slot: u32::MAX,
        generation: 0,
    };

    pub const fn new(slot: u32, generation: u32) -> Self {
        Self { slot, generation }
    }

    pub const fn parts(self) -> (u32, u32) {
        (self.slot, self.generation)
    }
}

impl PartialEq<i32> for RuntimeObjectHandle {
    fn eq(&self, other: &i32) -> bool {
        *other == 0 && *self == Self::NONE
    }
}

type Object = RuntimeObjectHandle;

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeComparisonValue {
    Number(f32),
    Boolean(bool),
    String(String),
    Color(i32),
    Uint(u32),
    ViewModel(CoreHandle),
}

const POINTER_HIT_LISTENER_TYPES: [ListenerType; 9] = [
    ListenerType::Enter,
    ListenerType::Exit,
    ListenerType::Down,
    ListenerType::Up,
    ListenerType::Move,
    ListenerType::Click,
    ListenerType::DragStart,
    ListenerType::DragEnd,
    ListenerType::Drag,
];

#[derive(Clone, Debug, Default)]
pub struct EventReport {
    pub event: Option<CoreHandle>,
    pub seconds_delay: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FocusState {
    pub has_focus: bool,
    pub expects_keyboard_input: bool,
}

#[derive(Clone)]
pub struct QueuedFocusEvent {
    pub group: RuntimeFocusListenerGroupHandle,
    pub is_focus: bool,
}

#[derive(Clone)]
pub struct QueuedSemanticEvent {
    pub group: RuntimeSemanticListenerGroupHandle,
    pub action_type: SemanticActionType,
}

pub enum InputInstance {
    Bool(Box<SMIBool>),
    Number(Box<SMINumber>),
    Trigger(Box<SMITrigger>),
}

impl InputInstance {
    fn from_definition(definition: &CoreHandle, notifier: InputInstanceNotifier) -> Option<Self> {
        if let Some(instance) = definition.with_downcast::<StateMachineBool, _>(|definition| {
            Self::Bool(Box::new(SMIBool::new(definition, notifier.clone())))
        }) {
            return Some(instance);
        }
        if let Some(instance) = definition.with_downcast::<StateMachineNumber, _>(|definition| {
            Self::Number(Box::new(SMINumber::new(definition, notifier.clone())))
        }) {
            return Some(instance);
        }
        definition.with_downcast::<StateMachineTrigger, _>(|definition| {
            Self::Trigger(Box::new(SMITrigger::new(definition, notifier)))
        })
    }

    fn advanced(&mut self) {
        if let Self::Trigger(trigger) = self {
            trigger.advanced();
        }
    }

    fn base(&self) -> &SMIInput {
        match self {
            Self::Bool(value) => &value.base,
            Self::Number(value) => &value.base,
            Self::Trigger(value) => &value.base,
        }
    }

    fn base_mut(&mut self) -> &mut SMIInput {
        match self {
            Self::Bool(value) => &mut value.base,
            Self::Number(value) => &mut value.base,
            Self::Trigger(value) => &mut value.base,
        }
    }
}

/// The services owned by the surrounding translated runtime. This interface
/// keeps all cross-owner object operations explicit while this owner retains
/// the exact state, ordering, and branches of the pinned implementation.
pub trait StateMachineInstanceRuntime {
    fn bindable_property_number_value(&self, property: &CoreHandle) -> Option<f32>;
    fn bindable_property_comparison_value(
        &self,
        property: &CoreHandle,
    ) -> Option<RuntimeComparisonValue>;
    fn component_comparison_value(
        &self,
        object_id: u32,
        property_key: u32,
    ) -> Option<RuntimeComparisonValue>;
    fn artboard_layout_dimensions(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
    ) -> Option<(f32, f32)>;
    fn bindable_source_changed_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) -> bool;
    fn use_bindable_property_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    );
    fn deterministic_mode(&self) -> bool;
    fn layer_any_state(&self, layer: &CoreHandle) -> CoreHandle;
    fn layer_entry_state(&self, layer: &CoreHandle) -> CoreHandle;
    fn make_state_instance(
        &mut self,
        state: CoreHandle,
        artboard: &RuntimeArtboardInstanceWeakHandle,
    ) -> RuntimeStateInstanceHandle;
    fn state_definition(&self, instance: &RuntimeStateInstanceHandle) -> CoreHandle;
    fn state_advance(
        &mut self,
        instance: &RuntimeStateInstanceHandle,
        seconds: f32,
        machine: &mut StateMachineInstance,
    );
    fn state_apply(
        &mut self,
        instance: &RuntimeStateInstanceHandle,
        artboard: &RuntimeArtboardInstanceWeakHandle,
        mix: f32,
    );
    fn state_keep_going(&self, instance: &RuntimeStateInstanceHandle) -> bool;
    fn state_clear_spilled_time(&mut self, instance: &RuntimeStateInstanceHandle);
    fn state_spilled_time(&self, instance: &RuntimeStateInstanceHandle) -> f32;
    fn state_animation(&self, instance: &RuntimeStateInstanceHandle) -> Option<CoreHandle>;
    fn state_animation_instance_time(&self, instance: &RuntimeStateInstanceHandle) -> f32;
    fn state_for_each_animation_instance(
        &mut self,
        machine: &mut StateMachineInstance,
        state: &RuntimeStateInstanceHandle,
        callback: &mut dyn FnMut(
            &mut dyn StateMachineInstanceRuntime,
            &mut StateMachineInstance,
            &mut LinearAnimationInstance,
        ),
    );
    fn state_transition_count(&self, state: &CoreHandle) -> usize;
    fn state_transition(&self, state: &CoreHandle, index: usize) -> CoreHandle;
    fn state_flags(&self, state: &CoreHandle) -> u32;
    fn state_events(&self, state: &CoreHandle) -> Vec<CoreHandle>;
    fn state_listener_actions(&self, state: &CoreHandle) -> Vec<CoreHandle>;
    fn transition_state_to(&self, transition: &CoreHandle) -> CoreHandle;
    fn transition_allowed(
        &mut self,
        transition: &CoreHandle,
        from: &RuntimeStateInstanceHandle,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> u8;
    fn transition_random_weight(&self, transition: &CoreHandle) -> u32;
    fn transition_evaluated_weight(&self, transition: &CoreHandle) -> u32;
    fn set_transition_evaluated_weight(&mut self, transition: &CoreHandle, value: u32);
    fn transition_use_layer(
        &mut self,
        transition: &CoreHandle,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    );
    fn transition_duration(&self, transition: &CoreHandle) -> u32;
    fn transition_duration_is_percentage(&self, transition: &CoreHandle) -> bool;
    fn transition_interpolator(&self, transition: &CoreHandle) -> Option<CoreHandle>;
    fn transition_enable_early_exit(&self, transition: &CoreHandle) -> bool;
    fn transition_pause_on_exit(&self, transition: &CoreHandle) -> bool;
    fn transition_apply_exit_condition(
        &mut self,
        transition: &CoreHandle,
        from: &RuntimeStateInstanceHandle,
    ) -> bool;
    fn transition_events(&self, transition: &CoreHandle) -> Vec<CoreHandle>;
    fn transition_listener_actions(&self, transition: &CoreHandle) -> Vec<CoreHandle>;
    fn transition_property_value(&self, property: &CoreHandle) -> f32;
    fn transition_property_instance(
        &self,
        machine: &StateMachineInstance,
        transition: &CoreHandle,
        property_key: u32,
    ) -> Option<CoreHandle>;
    fn fire_action_occurs(&self, action: &CoreHandle) -> u8;
    fn fire_action_perform(&mut self, action: &CoreHandle, machine: &mut StateMachineInstance);
    fn listener_action_matches(&self, action: &CoreHandle, occurrence: u8) -> bool;
    fn listener_action_perform(
        &mut self,
        action: &CoreHandle,
        machine: &mut StateMachineInstance,
        invocation: &ListenerInvocation,
    );
    fn animation_apply(
        &mut self,
        animation: &CoreHandle,
        artboard: &RuntimeArtboardInstanceWeakHandle,
        time: f32,
        mix: f32,
    );
    fn animation_duration_seconds(&self, animation: &CoreHandle) -> f32;
    fn interpolator_transform(&self, interpolator: &CoreHandle, value: f32) -> f32;
    fn artboard_frame_origin(&self, artboard: &RuntimeArtboardInstanceWeakHandle) -> bool;
    fn artboard_origin(&self, artboard: &RuntimeArtboardInstanceWeakHandle) -> Vec2D;
    fn artboard_layout_size(&self, artboard: &RuntimeArtboardInstanceWeakHandle) -> Vec2D;
    fn artboard_inverse_self_transform(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
        point: Vec2D,
    ) -> Option<Vec2D>;
    fn artboard_draw_order_change_counter(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
    ) -> u8;
    fn artboard_ordered_hit_components(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
        components: &[CoreHandle],
    ) -> Vec<usize>;
    fn artboard_has_component_dirt(&self, artboard: &RuntimeArtboardInstanceWeakHandle) -> bool;
    fn artboard_resolve(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
        id: u32,
    ) -> Option<CoreHandle>;
    fn object_is_event(&self, object: &CoreHandle) -> bool;
    fn artboard_file(&self, artboard: &RuntimeArtboardInstanceWeakHandle) -> Object;
    fn artboard_nested_artboards(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
    ) -> Vec<CoreHandle>;
    fn artboard_component_lists(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
    ) -> Vec<CoreHandle>;
    fn artboard_objects(&self, artboard: &RuntimeArtboardInstanceWeakHandle) -> Vec<CoreHandle>;
    fn artboard_text_inputs(&self, artboard: &RuntimeArtboardInstanceWeakHandle)
    -> Vec<CoreHandle>;
    fn artboard_source_data_binds(
        &self,
        artboard: &RuntimeArtboardInstanceWeakHandle,
    ) -> Vec<CoreHandle>;
    fn artboard_name(&self, artboard: &RuntimeArtboardInstanceWeakHandle) -> String;
    fn component_id(&self, component: &CoreHandle) -> u32;
    fn component_is_artboard(&self, component: &CoreHandle) -> bool;
    fn component_hit_test(
        &self,
        component: &CoreHandle,
        point: Vec2D,
        path: bool,
        clip: bool,
    ) -> bool;
    fn component_is_target_opaque(&self, component: &CoreHandle) -> bool;
    fn component_is_shape(&self, component: &CoreHandle) -> bool;
    fn component_is_text_run(&self, component: &CoreHandle) -> bool;
    fn component_is_container(&self, component: &CoreHandle) -> bool;
    fn component_is_layout(&self, component: &CoreHandle) -> bool;
    fn component_is_drawable_proxy(&self, component: &CoreHandle) -> bool;
    fn component_proxy(&self, component: &CoreHandle) -> Option<CoreHandle>;
    fn component_children(&self, component: &CoreHandle) -> Vec<CoreHandle>;
    fn component_mark_hit_path(&mut self, component: &CoreHandle);
    fn text_run_text_component(&self, component: &CoreHandle) -> Option<CoreHandle>;
    fn component_is_collapsed(&self, component: &CoreHandle) -> bool;
    fn component_is_paused(&self, component: &CoreHandle) -> bool;
    fn component_world_to_local(
        &self,
        component: &CoreHandle,
        point: Vec2D,
        index: Option<i32>,
    ) -> Option<Vec2D>;
    fn component_ordered_indices(&self, component: &CoreHandle) -> Vec<i32>;
    fn component_state_machine(
        &self,
        component: &CoreHandle,
        index: i32,
    ) -> Option<RuntimeStateMachineInstanceHandle>;
    fn nested_animations(&self, nested_artboard: &CoreHandle) -> Vec<CoreHandle>;
    fn nested_artboard_instance(
        &self,
        nested_artboard: &CoreHandle,
    ) -> RuntimeArtboardInstanceWeakHandle;
    fn nested_is_state_machine(&self, animation: &CoreHandle) -> bool;
    fn nested_state_machine_instance(
        &self,
        animation: &CoreHandle,
    ) -> Option<RuntimeStateMachineInstanceHandle>;
    fn nested_add_event_listener(
        &mut self,
        animation: &CoreHandle,
        nested_artboard: &CoreHandle,
        listener: RuntimeStateMachineInstanceWeakHandle,
    );
    fn nested_remove_event_listener(
        &mut self,
        animation: &CoreHandle,
        listener: RuntimeStateMachineInstanceWeakHandle,
    );
    fn make_text_input_listener_group(
        &mut self,
        text_input: &CoreHandle,
        machine: RuntimeStateMachineInstanceWeakHandle,
    ) -> RuntimeListenerGroupHandle;
    fn resolve_focus_data(&self, target: &CoreHandle) -> Option<CoreHandle>;
    fn resolve_semantic_data(&self, target: &CoreHandle) -> Option<CoreHandle>;
    fn provided_hit_components(
        &mut self,
        provider: &CoreHandle,
        machine: &mut StateMachineInstance,
    ) -> Vec<Box<dyn HitComponent>>;
    fn object_listener_provider(&self, object: &CoreHandle) -> Option<CoreHandle>;
    fn object_scripted(&self, object: &CoreHandle) -> Option<CoreHandle>;
    fn scripted_wants_keyboard(&self, object: &CoreHandle) -> bool;
    fn scripted_wants_text(&self, object: &CoreHandle) -> bool;
    fn scripted_wants_gamepad(&self, object: &CoreHandle) -> bool;
    fn listener_target_id(&self, listener: &CoreHandle) -> u32;
    fn listener_event_ids(&self, listener: &CoreHandle) -> Vec<u32>;
    fn listener_perform_changes(
        &mut self,
        listener: &CoreHandle,
        machine: &mut StateMachineInstance,
        invocation: &ListenerInvocation,
    );
    fn listener_invocation_none(&mut self) -> ListenerInvocation;
    fn listener_invocation_event(&mut self, event: &CoreHandle, delay: f32) -> ListenerInvocation;
    fn listener_invocation_view_model(
        &mut self,
        view_model: RuntimeListenerViewModelWeakHandle,
    ) -> ListenerInvocation;
    fn clone_data_bind(&mut self, bind: &CoreHandle) -> CoreHandle;
    fn data_bind_file(&self, bind: &CoreHandle) -> Object;
    fn data_bind_set_file(&mut self, bind: &CoreHandle, file: Object);
    fn data_bind_converter(&self, bind: &CoreHandle) -> Option<CoreHandle>;
    fn clone_data_converter(&mut self, converter: &CoreHandle) -> CoreHandle;
    fn data_bind_set_converter(&mut self, bind: &CoreHandle, converter: CoreHandle);
    fn data_bind_initialize(&mut self, bind: &CoreHandle);
    fn data_bind_target(&self, bind: &CoreHandle) -> Option<CoreHandle>;
    fn data_bind_flags(&self, bind: &CoreHandle) -> u32;
    fn data_bind_property_key(&self, bind: &CoreHandle) -> u32;
    fn data_bind_is_transition_target(&self, bind: &CoreHandle) -> bool;
    fn data_bind_is_keyframe_target(&self, bind: &CoreHandle) -> bool;
    fn data_bind_bindable_target(&self, bind: &CoreHandle) -> bool;
    fn clone_bindable_property(&mut self, property: &CoreHandle) -> CoreHandle;
    fn make_transition_property(&mut self) -> CoreHandle;
    fn configure_data_bind_target(
        &mut self,
        bind: &CoreHandle,
        target: CoreHandle,
        property_key: u32,
    );
    fn delete_owned_object(&mut self, object: CoreHandle);
    fn keyframe_type(&self, keyframe: &CoreHandle) -> u16;
    fn animation_keyframes(&self, animation_instance: &LinearAnimationInstance) -> Vec<CoreHandle>;
    fn make_keyframe_holder(&mut self, keyframe_type: u16) -> CoreHandle;
    fn add_keyframe_holder(
        &mut self,
        animation_instance: &mut LinearAnimationInstance,
        keyframe: &CoreHandle,
        holder: CoreHandle,
    );
    fn scripted_clone(
        &mut self,
        source: &CoreHandle,
        machine: &mut StateMachineInstance,
    ) -> CoreHandle;
    fn scripted_initialize(&mut self, object: &CoreHandle);
    fn scripted_hydrate_inputs(&mut self, object: &CoreHandle);
    fn scripted_delete(&mut self, object: CoreHandle);
    fn event_is_audio(&self, event: &CoreHandle) -> bool;
    fn event_play_audio(&mut self, event: &CoreHandle);
    fn nested_event_listeners(
        &self,
        machine: &StateMachineInstance,
    ) -> Vec<RuntimeStateMachineInstanceWeakHandle>;
    fn nested_artboard_context(&self, machine: &StateMachineInstance) -> Object;
    fn gamepad_submit_buffer(&mut self, machine: &mut StateMachineInstance, data: &[u8]) -> bool;
    fn gamepad_broadcast(
        &mut self,
        machine: &mut StateMachineInstance,
        invocation: &ListenerInvocation,
        skipped: Option<&CoreHandle>,
    ) -> HitResult;
}

pub type RuntimeServicesHandle = Rc<RefCell<Box<dyn StateMachineInstanceRuntime>>>;

pub struct StateMachineLayerInstance {
    runtime_services: Option<RuntimeServicesHandle>,
    occurrence: RuntimeStateMachineLayerInstanceWeakHandle,
    layer: Option<CoreHandle>,
    artboard_instance: RuntimeArtboardInstanceWeakHandle,
    any_state_instance: Option<RuntimeStateInstanceHandle>,
    current_state: Option<RuntimeStateInstanceHandle>,
    state_from: Option<RuntimeStateInstanceHandle>,
    transition: Option<CoreHandle>,
    transition_duration_property: Option<CoreHandle>,
    animation_reset: Option<AnimationReset>,
    transition_completed: bool,
    hold_animation_from: bool,
    mix: f32,
    mix_from: f32,
    state_machine_changed_on_advance: bool,
    waiting_for_exit: bool,
    hold_animation: Option<CoreHandle>,
    hold_time: f32,
}

#[derive(Clone)]
pub struct RuntimeStateMachineLayerInstanceHandle(Rc<RefCell<StateMachineLayerInstance>>);

#[derive(Clone, Default)]
pub struct RuntimeStateMachineLayerInstanceWeakHandle(Weak<RefCell<StateMachineLayerInstance>>);

impl RuntimeStateMachineLayerInstanceHandle {
    fn new(layer: StateMachineLayerInstance) -> Self {
        let handle = Self(Rc::new(RefCell::new(layer)));
        handle.0.borrow_mut().occurrence = handle.downgrade();
        handle
    }
    pub fn downgrade(&self) -> RuntimeStateMachineLayerInstanceWeakHandle {
        RuntimeStateMachineLayerInstanceWeakHandle(Rc::downgrade(&self.0))
    }
    pub fn with_layer<R>(&self, f: impl FnOnce(&StateMachineLayerInstance) -> R) -> R {
        f(&self.0.borrow())
    }
    pub fn with_layer_mut<R>(&self, f: impl FnOnce(&mut StateMachineLayerInstance) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

impl RuntimeStateMachineLayerInstanceWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeStateMachineLayerInstanceHandle> {
        self.0.upgrade().map(RuntimeStateMachineLayerInstanceHandle)
    }
    pub fn with_layer<R>(&self, f: impl FnOnce(&StateMachineLayerInstance) -> R) -> Option<R> {
        self.upgrade().map(|layer| layer.with_layer(f))
    }
    pub fn with_layer_mut<R>(
        &self,
        f: impl FnOnce(&mut StateMachineLayerInstance) -> R,
    ) -> Option<R> {
        self.upgrade().map(|layer| layer.with_layer_mut(f))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl Default for StateMachineLayerInstance {
    fn default() -> Self {
        Self {
            runtime_services: None,
            occurrence: RuntimeStateMachineLayerInstanceWeakHandle::default(),
            layer: None,
            artboard_instance: RuntimeArtboardInstanceWeakHandle::default(),
            any_state_instance: None,
            current_state: None,
            state_from: None,
            transition: None,
            transition_duration_property: None,
            animation_reset: None,
            transition_completed: false,
            hold_animation_from: false,
            mix: 1.0,
            mix_from: 1.0,
            state_machine_changed_on_advance: false,
            waiting_for_exit: false,
            hold_animation: None,
            hold_time: 0.0,
        }
    }
}

impl StateMachineLayerInstance {
    const MAX_ITERATIONS: usize = 100;

    fn runtime(&self) -> RefMut<'_, dyn StateMachineInstanceRuntime> {
        RefMut::map(
            self.runtime_services
                .as_ref()
                .expect("initialized layer retains runtime services")
                .borrow_mut(),
            |runtime| runtime.as_mut(),
        )
    }

    fn init(
        &mut self,
        state_machine_instance: &mut StateMachineInstance,
        runtime_services: RuntimeServicesHandle,
        layer: CoreHandle,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) {
        self.runtime_services = Some(runtime_services);
        self.artboard_instance = artboard.clone();
        let deterministic = self.runtime().deterministic_mode();
        let seed = RandomProvider::layer_seed(deterministic);
        RandomProvider::seed(seed);
        debug_assert!(self.layer.is_none());
        let any_state = self.runtime().layer_any_state(&layer);
        let any_state_instance = self.runtime().make_state_instance(any_state, &artboard);
        state_machine_instance.build_state_keyframe_binds(&any_state_instance);
        self.any_state_instance = Some(any_state_instance);
        let entry = self.runtime().layer_entry_state(&layer);
        self.layer = Some(layer);
        self.change_state(state_machine_instance, entry);
    }

    fn reset_state(&mut self, machine: &mut StateMachineInstance) {
        if let Some(state_from) = self.state_from.as_ref()
            && self
                .any_state_instance
                .as_ref()
                .is_none_or(|any| !state_from.ptr_eq(any))
            && self
                .current_state
                .as_ref()
                .is_none_or(|current| !state_from.ptr_eq(current))
        {
            machine.remove_state_keyframe_binds(state_from);
        }
        self.state_from = None;
        if let Some(current) = self.current_state.as_ref()
            && self
                .any_state_instance
                .as_ref()
                .is_none_or(|any| !current.ptr_eq(any))
        {
            machine.remove_state_keyframe_binds(current);
        }
        self.current_state = None;
        let entry = self
            .runtime()
            .layer_entry_state(self.layer.as_ref().expect("initialized layer"));
        self.change_state(machine, entry);
    }

    fn resolved_duration(&mut self) -> u32 {
        if let Some(property) = self.transition_duration_property.clone() {
            return self
                .runtime()
                .transition_property_value(&property)
                .round()
                .max(0.0) as u32;
        }
        self.transition
            .clone()
            .map(|transition| self.runtime().transition_duration(&transition))
            .unwrap_or(0)
    }

    fn resolved_mix_time(&mut self) -> f32 {
        let duration = self.resolved_duration();
        if duration == 0 {
            return 0.0;
        }
        let Some(transition) = self.transition.clone() else {
            return 0.0;
        };
        if self
            .runtime()
            .transition_duration_is_percentage(&transition)
        {
            let animation = self
                .state_from
                .as_ref()
                .and_then(|state| self.runtime().state_animation(state));
            let animation_duration = animation
                .as_ref()
                .map(|animation| self.runtime().animation_duration_seconds(animation))
                .unwrap_or(0.0);
            duration as f32 / 100.0 * animation_duration
        } else {
            duration as f32 / 1000.0
        }
    }

    fn update_mix(&mut self, machine: &mut StateMachineInstance, seconds: f32) {
        if self.transition.is_some() && self.state_from.is_some() && self.resolved_duration() != 0 {
            let mix_time = self.resolved_mix_time();
            self.mix = if mix_time == 0.0 {
                1.0
            } else {
                (self.mix + seconds / mix_time).clamp(0.0, 1.0)
            };
            if self.mix == 1.0 && !self.transition_completed {
                self.transition_completed = true;
                self.clear_animation_reset();
                let transition = self.transition.clone().expect("active transition");
                let events = self.runtime().transition_events(&transition);
                self.fire_events(machine, 1, &events);
                let actions = self.runtime().transition_listener_actions(&transition);
                self.perform_listener_actions(machine, 1, &actions);
            }
        } else {
            self.mix = 1.0;
        }
    }

    fn advance(
        &mut self,
        machine: &mut StateMachineInstance,
        seconds: f32,
        new_frame: bool,
    ) -> bool {
        if new_frame {
            self.state_machine_changed_on_advance = false;
        }
        if let Some(current) = self.current_state.clone() {
            self.runtime().state_advance(&current, seconds, machine);
        }
        self.update_mix(machine, seconds);
        if let Some(from) = self.state_from.clone()
            && self.mix < 1.0
            && !self.hold_animation_from
        {
            self.runtime().state_advance(&from, seconds, machine);
        }
        self.apply();
        let mut changed = false;
        for iteration in 0.. {
            if !self.update_state(machine) {
                break;
            }
            changed = true;
            self.apply();
            if iteration == Self::MAX_ITERATIONS {
                let machine_name = machine.name();
                let layer = self.layer.as_ref();
                let artboard = &self.artboard_instance;
                eprintln!(
                    "{} StateMachine exceeded max iterations in layer {} on artboard {}",
                    machine_name,
                    layer.is_some(),
                    artboard.upgrade().is_some()
                );
                return false;
            }
        }
        if let Some(current) = self.current_state.clone() {
            self.runtime().state_clear_spilled_time(&current);
        }
        changed
            || self.mix != 1.0
            || self.waiting_for_exit
            || self
                .current_state
                .as_ref()
                .is_some_and(|current| self.runtime().state_keep_going(current))
    }

    fn is_transitioning(&mut self) -> bool {
        self.transition.is_some()
            && self.state_from.is_some()
            && self.resolved_duration() != 0
            && self.mix < 1.0
    }

    fn update_state(&mut self, machine: &mut StateMachineInstance) -> bool {
        if self.is_transitioning()
            && !self
                .runtime()
                .transition_enable_early_exit(self.transition.as_ref().expect("active transition"))
        {
            return false;
        }
        self.waiting_for_exit = false;
        if self.try_change_state_from(machine, self.any_state_instance.clone()) {
            return true;
        }
        self.try_change_state_from(machine, self.current_state.clone())
    }

    fn fire_events(
        &mut self,
        machine: &mut StateMachineInstance,
        occurrence: u8,
        events: &[CoreHandle],
    ) {
        for event in events {
            if self.runtime().fire_action_occurs(event) == occurrence {
                self.runtime().fire_action_perform(event, machine);
            }
        }
    }

    fn perform_listener_actions(
        &mut self,
        machine: &mut StateMachineInstance,
        occurrence: u8,
        actions: &[CoreHandle],
    ) {
        for action in actions {
            if self.runtime().listener_action_matches(action, occurrence) {
                let invocation = self.runtime().listener_invocation_none();
                self.runtime()
                    .listener_action_perform(action, machine, &invocation);
            }
        }
    }

    fn can_change_state(&mut self, state_to: &CoreHandle) -> bool {
        self.current_state
            .as_ref()
            .is_none_or(|current| self.runtime().state_definition(current) != state_to.clone())
    }

    fn change_state(&mut self, machine: &mut StateMachineInstance, state_to: CoreHandle) {
        if self
            .current_state
            .as_ref()
            .is_some_and(|current| self.runtime().state_definition(current) == state_to)
        {
            return;
        }
        if let Some(current) = self.current_state.clone() {
            let state = self.runtime().state_definition(&current);
            let events = self.runtime().state_events(&state);
            self.fire_events(machine, 1, &events);
            let actions = self.runtime().state_listener_actions(&state);
            self.perform_listener_actions(machine, 1, &actions);
        }
        let current = self
            .runtime()
            .make_state_instance(state_to, &self.artboard_instance);
        machine.build_state_keyframe_binds(&current);
        let state = self.runtime().state_definition(&current);
        let events = self.runtime().state_events(&state);
        self.fire_events(machine, 0, &events);
        let actions = self.runtime().state_listener_actions(&state);
        self.perform_listener_actions(machine, 0, &actions);
        self.current_state = Some(current);
    }

    fn find_random_transition(
        &mut self,
        machine: &mut StateMachineInstance,
        from_instance: RuntimeStateInstanceHandle,
    ) -> Option<CoreHandle> {
        let state = self.runtime().state_definition(&from_instance);
        let mut total_weight = 0;
        for index in 0..self.runtime().state_transition_count(&state) {
            let transition = self.runtime().state_transition(&state, index);
            let state_to = self.runtime().transition_state_to(&transition);
            if self.can_change_state(&state_to) {
                let allowed = self.runtime().transition_allowed(
                    &transition,
                    &from_instance,
                    machine,
                    self.occurrence.clone(),
                );
                if allowed == 2 {
                    let weight = self.runtime().transition_random_weight(&transition);
                    self.runtime()
                        .set_transition_evaluated_weight(&transition, weight);
                    total_weight += weight;
                } else {
                    self.runtime()
                        .set_transition_evaluated_weight(&transition, 0);
                    if allowed == 1 {
                        self.waiting_for_exit = true;
                    }
                }
            } else {
                self.runtime()
                    .set_transition_evaluated_weight(&transition, 0);
            }
        }
        if total_weight == 0 {
            return None;
        }
        let random_weight = RandomProvider::generate_random_float() as f64 * total_weight as f64;
        let mut current_weight = 0.0;
        for index in 0..self.runtime().state_transition_count(&state) {
            let transition = self.runtime().state_transition(&state, index);
            let weight = self.runtime().transition_evaluated_weight(&transition) as f64;
            if current_weight + weight > random_weight {
                self.runtime()
                    .transition_use_layer(&transition, machine, self.occurrence.clone());
                return Some(transition);
            }
            current_weight += weight;
        }
        None
    }

    fn find_allowed_transition(
        &mut self,
        machine: &mut StateMachineInstance,
        from_instance: RuntimeStateInstanceHandle,
    ) -> Option<CoreHandle> {
        let state = self.runtime().state_definition(&from_instance);
        if self.runtime().state_flags(&state) & 1 != 0 {
            return self.find_random_transition(machine, from_instance);
        }
        for index in 0..self.runtime().state_transition_count(&state) {
            let transition = self.runtime().state_transition(&state, index);
            let state_to = self.runtime().transition_state_to(&transition);
            if !self.can_change_state(&state_to) {
                continue;
            }
            let allowed = self.runtime().transition_allowed(
                &transition,
                &from_instance,
                machine,
                self.occurrence.clone(),
            );
            if allowed == 2 {
                let weight = self.runtime().transition_random_weight(&transition);
                self.runtime()
                    .set_transition_evaluated_weight(&transition, weight);
                self.runtime()
                    .transition_use_layer(&transition, machine, self.occurrence.clone());
                return Some(transition);
            }
            self.runtime()
                .set_transition_evaluated_weight(&transition, 0);
            if allowed == 1 {
                self.waiting_for_exit = true;
            }
        }
        None
    }

    fn clear_animation_reset(&mut self) {
        if let Some(reset) = self.animation_reset.take() {
            AnimationResetFactory::release(reset);
        }
    }

    fn build_animation_reset_for_transition(&mut self) {
        let animations = [self.state_from.as_ref(), self.current_state.as_ref()]
            .into_iter()
            .flatten()
            .filter_map(|state| {
                state
                    .definition()
                    .with_downcast::<AnimationState, _>(AnimationState::animation)
                    .flatten()
            })
            .collect::<Vec<_>>();
        self.animation_reset = Some(
            self.artboard_instance
                .with_artboard(|artboard| {
                    AnimationResetFactory::from_animation_handles(&animations, artboard, false)
                })
                .expect("a state-machine layer retains its artboard instance"),
        );
    }

    fn try_change_state_from(
        &mut self,
        machine: &mut StateMachineInstance,
        from_instance: Option<RuntimeStateInstanceHandle>,
    ) -> bool {
        let Some(from_instance) = from_instance else {
            return false;
        };
        let out_state = self.current_state.clone();
        let Some(transition) = self.find_allowed_transition(machine, from_instance) else {
            return false;
        };
        self.clear_animation_reset();
        let state_to = self.runtime().transition_state_to(&transition);
        self.change_state(machine, state_to);
        self.state_machine_changed_on_advance = true;
        self.transition = Some(transition.clone());
        self.transition_duration_property = self.runtime().transition_property_instance(
            machine,
            &transition,
            StateTransitionBase::DURATION_PROPERTY_KEY as u32,
        );
        let events = self.runtime().transition_events(&transition);
        self.fire_events(machine, 0, &events);
        let actions = self.runtime().transition_listener_actions(&transition);
        self.perform_listener_actions(machine, 0, &actions);
        self.transition_completed = self.resolved_duration() == 0;
        if self.transition_completed {
            self.fire_events(machine, 1, &events);
            self.perform_listener_actions(machine, 1, &actions);
        }
        if let Some(state_from) = self.state_from.as_ref()
            && self
                .any_state_instance
                .as_ref()
                .is_none_or(|any| !state_from.ptr_eq(any))
        {
            machine.remove_state_keyframe_binds(state_from);
        }
        self.state_from = out_state;
        if !self.transition_completed {
            self.build_animation_reset_for_transition();
        }
        if let Some(out_state) = self.state_from.clone()
            && self
                .runtime()
                .transition_apply_exit_condition(&transition, &out_state)
        {
            self.hold_animation = self.runtime().state_animation(&out_state);
            self.hold_time = self.runtime().state_animation_instance_time(&out_state);
        }
        self.mix_from = self.mix;
        if self.mix != 0.0 {
            self.hold_animation_from = self.runtime().transition_pause_on_exit(&transition);
        }
        if let Some(current) = self.current_state.clone() {
            let advance_time = self
                .state_from
                .as_ref()
                .map(|from| self.runtime().state_spilled_time(from))
                .unwrap_or(0.0);
            self.runtime()
                .state_advance(&current, advance_time, machine);
        }
        self.mix = 0.0;
        self.update_mix(machine, 0.0);
        self.waiting_for_exit = false;
        true
    }

    fn apply(&mut self) {
        if let Some(animation_reset) = self.animation_reset.as_ref() {
            self.artboard_instance
                .with_artboard_mut(|artboard| animation_reset.apply(artboard))
                .expect("a state-machine layer retains its artboard instance");
        }
        if let Some(hold_animation) = self.hold_animation.take() {
            self.runtime().animation_apply(
                &hold_animation,
                &self.artboard_instance,
                self.hold_time,
                self.mix_from,
            );
        }
        let interpolator = self
            .transition
            .clone()
            .and_then(|transition| self.runtime().transition_interpolator(&transition));
        if let Some(state_from) = self.state_from.clone().filter(|_| self.mix < 1.0) {
            let mix = interpolator
                .as_ref()
                .map(|interpolator| {
                    self.runtime()
                        .interpolator_transform(interpolator, self.mix_from)
                })
                .unwrap_or(self.mix_from);
            self.runtime()
                .state_apply(&state_from, &self.artboard_instance, mix);
        }
        if let Some(current_state) = self.current_state.clone() {
            let mix = interpolator
                .as_ref()
                .map(|interpolator| {
                    self.runtime()
                        .interpolator_transform(interpolator, self.mix)
                })
                .unwrap_or(self.mix);
            self.runtime()
                .state_apply(&current_state, &self.artboard_instance, mix);
        }
    }

    fn current_state(&mut self) -> Option<CoreHandle> {
        let current = self.current_state.clone()?;
        Some(self.runtime().state_definition(&current))
    }

    fn current_animation(&mut self) -> Option<RuntimeStateInstanceHandle> {
        self.current_state
            .clone()
            .filter(|current| self.runtime().state_animation(current).is_some())
    }
}

pub trait HitComponent {
    fn component(&self) -> CoreHandle;
    fn as_hit_drawable_mut(&mut self) -> Option<&mut HitDrawable> {
        None
    }
    #[cfg(test)]
    fn early_out_count(&self) -> i32 {
        0
    }
    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        machine: &mut StateMachineInstance,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult;
    fn process_gamepad_invocation(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult;
    fn prepare_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        position: Vec2D,
        hit_type: ListenerType,
        pointer_id: i32,
    );
    fn hit_test(&self, runtime: &dyn StateMachineInstanceRuntime, position: Vec2D) -> bool;
    fn enable_pointer_events(
        &mut self,
        _runtime: &mut dyn StateMachineInstanceRuntime,
        _pointer_id: i32,
    ) {
    }
    fn disable_pointer_events(
        &mut self,
        _runtime: &mut dyn StateMachineInstanceRuntime,
        _pointer_id: i32,
    ) {
    }
}

pub struct HitDrawable {
    component: CoreHandle,
    drawable: CoreHandle,
    hit_radius: f32,
    is_hovered: bool,
    can_early_out: bool,
    has_down_listener: bool,
    has_up_listener: bool,
    is_opaque: bool,
    listeners: Vec<RuntimeListenerGroupHandle>,
    hit_path: bool,
    hit_clip: bool,
    #[cfg(test)]
    early_out_count: i32,
}

type HitExpandable = HitDrawable;
type HitTextRun = HitExpandable;
type HitLayout = HitDrawable;

impl HitDrawable {
    fn new(
        runtime: &dyn StateMachineInstanceRuntime,
        drawable: CoreHandle,
        component: CoreHandle,
        is_opaque: bool,
        hit_path: bool,
        hit_clip: bool,
    ) -> Self {
        Self {
            component,
            drawable,
            hit_radius: 2.0,
            is_hovered: false,
            can_early_out: !runtime.component_is_target_opaque(&drawable),
            has_down_listener: false,
            has_up_listener: false,
            is_opaque,
            listeners: Vec::new(),
            hit_path,
            hit_clip,
            #[cfg(test)]
            early_out_count: 0,
        }
    }

    fn add_listener(&mut self, group: RuntimeListenerGroupHandle) {
        let (can_early_out, needs_down, needs_up) = self
            .component
            .with(|component| {
                let component = component
                    .as_component()
                    .expect("a hit target retains a Component");
                group.with_group(|group| {
                    (
                        group.can_early_out(component),
                        group.needs_down_listener(component),
                        group.needs_up_listener(component),
                    )
                })
            })
            .expect("a hit target remains in its CoreArena");
        if !can_early_out {
            self.can_early_out = false;
        } else {
            if needs_down {
                self.has_down_listener = true;
            }
            if needs_up {
                self.has_up_listener = true;
            }
        }
        self.listeners.push(group);
    }
}

impl HitComponent for HitDrawable {
    fn component(&self) -> CoreHandle {
        self.component.clone()
    }

    fn as_hit_drawable_mut(&mut self) -> Option<&mut HitDrawable> {
        Some(self)
    }

    #[cfg(test)]
    fn early_out_count(&self) -> i32 {
        self.early_out_count
    }

    fn hit_test(&self, runtime: &dyn StateMachineInstanceRuntime, position: Vec2D) -> bool {
        runtime.component_hit_test(&self.component, position, self.hit_path, self.hit_clip)
    }

    fn prepare_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        position: Vec2D,
        hit_type: ListenerType,
        pointer_id: i32,
    ) {
        if self.can_early_out
            && (hit_type != ListenerType::Down || !self.has_down_listener)
            && (hit_type != ListenerType::Up || !self.has_up_listener)
        {
            #[cfg(test)]
            {
                self.early_out_count += 1;
            }
            return;
        }
        self.is_hovered = hit_type != ListenerType::Exit && self.hit_test(runtime, position);
        if self.is_hovered {
            for listener in &self.listeners {
                listener.with_group_mut(|listener| listener.hover(pointer_id));
            }
        }
    }

    fn process_gamepad_invocation(
        &mut self,
        _runtime: &mut dyn StateMachineInstanceRuntime,
        _invocation: &ListenerInvocation,
        _already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        HitResult::None
    }

    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        machine: &mut StateMachineInstance,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult {
        if self.can_early_out
            && (hit_type != ListenerType::Down || !self.has_down_listener)
            && (hit_type != ListenerType::Up || !self.has_up_listener)
        {
            return HitResult::None;
        }
        let mut blocking = false;
        for listener in &self.listeners {
            if listener.with_group(|listener| listener.is_consumed()) {
                continue;
            }
            let result = self
                .component
                .with_mut(|component| {
                    let component = component
                        .as_component_mut()
                        .expect("a hit target retains a Component");
                    listener.with_group_mut(|listener| {
                        listener.process_event(
                            component, position, pointer_id, hit_type, can_hit, timestamp, machine,
                        )
                    })
                })
                .expect("a hit target remains in its CoreArena");
            if result == ProcessEventResult::Scroll {
                blocking = true;
            }
        }
        if !self.is_hovered || !can_hit {
            HitResult::None
        } else if self.is_opaque || runtime.component_is_target_opaque(&self.drawable) || blocking {
            HitResult::HitOpaque
        } else {
            HitResult::Hit
        }
    }

    fn enable_pointer_events(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        pointer_id: i32,
    ) {
        for listener in &self.listeners {
            listener.with_group_mut(|listener| listener.enable(pointer_id));
        }
    }

    fn disable_pointer_events(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        pointer_id: i32,
    ) {
        for listener in &self.listeners {
            listener.with_group_mut(|listener| listener.disable(pointer_id));
        }
    }
}

struct HitNestedArtboard {
    component: CoreHandle,
}

impl HitComponent for HitNestedArtboard {
    fn component(&self) -> CoreHandle {
        self.component.clone()
    }

    fn hit_test(&self, runtime: &dyn StateMachineInstanceRuntime, position: Vec2D) -> bool {
        if runtime.component_is_collapsed(&self.component)
            || runtime.component_is_paused(&self.component)
        {
            return false;
        }
        let Some(local) = runtime.component_world_to_local(&self.component, position, None) else {
            return false;
        };
        runtime
            .nested_animations(&self.component)
            .into_iter()
            .filter(|animation| runtime.nested_is_state_machine(animation))
            .any(|animation| {
                runtime
                    .nested_state_machine_instance(&animation)
                    .is_some_and(|instance| instance.with_instance(|nested| nested.hit_test(local)))
            })
    }

    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        _machine: &mut StateMachineInstance,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult {
        if runtime.component_is_collapsed(&self.component)
            || runtime.component_is_paused(&self.component)
        {
            return HitResult::None;
        }
        let Some(local) = runtime.component_world_to_local(&self.component, position, None) else {
            return HitResult::None;
        };
        let mut result = HitResult::None;
        for animation in runtime.nested_animations(&self.component) {
            if !runtime.nested_is_state_machine(&animation) {
                continue;
            }
            let Some(instance) = runtime.nested_state_machine_instance(&animation) else {
                continue;
            };
            instance.with_instance_mut(|nested| {
                if can_hit {
                    result = match hit_type {
                        ListenerType::Down => nested.pointer_down(local, pointer_id),
                        ListenerType::Up => nested.pointer_up(local, pointer_id),
                        ListenerType::Move => nested.pointer_move(local, timestamp, pointer_id),
                        ListenerType::Exit => nested.pointer_exit(local, pointer_id),
                        ListenerType::DragStart => {
                            nested.drag_start(local, timestamp, true, pointer_id);
                            result
                        }
                        ListenerType::DragEnd => {
                            nested.drag_end(local, timestamp, pointer_id);
                            result
                        }
                        _ => result,
                    };
                } else if matches!(
                    hit_type,
                    ListenerType::Down | ListenerType::Up | ListenerType::Move | ListenerType::Exit
                ) {
                    nested.pointer_exit(local, pointer_id);
                }
            });
        }
        result
    }

    fn process_gamepad_invocation(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        for animation in runtime.nested_animations(&self.component) {
            if runtime.nested_is_state_machine(&animation) {
                if let Some(instance) = runtime.nested_state_machine_instance(&animation) {
                    instance.with_instance_mut(|nested| {
                        nested.broadcast_gamepad_to_scripted_drawables(
                            invocation,
                            already_dispatched,
                        );
                    });
                }
            }
        }
        HitResult::None
    }

    fn prepare_event(
        &mut self,
        _runtime: &mut dyn StateMachineInstanceRuntime,
        _position: Vec2D,
        _hit_type: ListenerType,
        _pointer_id: i32,
    ) {
    }
}

struct HitComponentList {
    component: CoreHandle,
}

impl HitComponent for HitComponentList {
    fn component(&self) -> CoreHandle {
        self.component.clone()
    }

    fn hit_test(&self, runtime: &dyn StateMachineInstanceRuntime, position: Vec2D) -> bool {
        if runtime.component_is_collapsed(&self.component) {
            return false;
        }
        for index in runtime
            .component_ordered_indices(&self.component)
            .into_iter()
            .rev()
        {
            let Some(local) =
                runtime.component_world_to_local(&self.component, position, Some(index))
            else {
                continue;
            };
            if runtime
                .component_state_machine(&self.component, index)
                .is_some_and(|machine| machine.with_instance(|nested| nested.hit_test(local)))
            {
                return true;
            }
        }
        false
    }

    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        _machine: &mut StateMachineInstance,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult {
        if runtime.component_is_collapsed(&self.component) {
            return HitResult::None;
        }
        let mut result = HitResult::None;
        let mut running_can_hit = can_hit;
        for index in runtime
            .component_ordered_indices(&self.component)
            .into_iter()
            .rev()
        {
            let Some(local) =
                runtime.component_world_to_local(&self.component, position, Some(index))
            else {
                continue;
            };
            let Some(machine) = runtime.component_state_machine(&self.component, index) else {
                continue;
            };
            let item = machine.with_instance_mut(|nested| {
                if running_can_hit {
                    match hit_type {
                        ListenerType::Down => nested.pointer_down(local, pointer_id),
                        ListenerType::Up => nested.pointer_up(local, pointer_id),
                        ListenerType::Move => nested.pointer_move(local, timestamp, pointer_id),
                        ListenerType::Exit => nested.pointer_exit(local, pointer_id),
                        ListenerType::DragStart => {
                            nested.drag_start(local, 0.0, true, pointer_id);
                            HitResult::None
                        }
                        ListenerType::DragEnd => {
                            nested.drag_end(local, 0.0, pointer_id);
                            HitResult::None
                        }
                        _ => HitResult::None,
                    }
                } else {
                    if matches!(
                        hit_type,
                        ListenerType::Down
                            | ListenerType::Up
                            | ListenerType::Move
                            | ListenerType::Exit
                    ) {
                        nested.pointer_exit(local, pointer_id);
                    }
                    HitResult::None
                }
            });
            if (result == HitResult::None && matches!(item, HitResult::Hit | HitResult::HitOpaque))
                || (result == HitResult::Hit && item == HitResult::HitOpaque)
            {
                result = item;
            }
            if result == HitResult::HitOpaque {
                running_can_hit = false;
            }
        }
        result
    }

    fn process_gamepad_invocation(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        if runtime.component_is_collapsed(&self.component) {
            return HitResult::None;
        }
        let mut result = HitResult::None;
        let mut running_can_hit = true;
        for index in runtime
            .component_ordered_indices(&self.component)
            .into_iter()
            .rev()
        {
            let Some(machine) = runtime.component_state_machine(&self.component, index) else {
                continue;
            };
            let item = if running_can_hit {
                machine.with_instance_mut(|nested| {
                    nested.broadcast_gamepad_to_scripted_drawables(invocation, already_dispatched)
                })
            } else {
                HitResult::None
            };
            if (result == HitResult::None && matches!(item, HitResult::Hit | HitResult::HitOpaque))
                || (result == HitResult::Hit && item == HitResult::HitOpaque)
            {
                result = item;
            }
            if result == HitResult::HitOpaque {
                running_can_hit = false;
            }
        }
        result
    }

    fn prepare_event(
        &mut self,
        _runtime: &mut dyn StateMachineInstanceRuntime,
        _position: Vec2D,
        _hit_type: ListenerType,
        _pointer_id: i32,
    ) {
    }
}

fn data_context_property(
    context: &RuntimeDataContextHandle,
    path_owner: &CoreHandle,
) -> Option<CoreHandle> {
    if let Some(value) = path_owner
        .with_downcast_mut::<StateMachineListenerSingle, _>(|owner| {
            owner
                .data_bind_path_referencer
                .with_data_bind_path_mut(|path| {
                    context.with_context(|context| context.get_property_from_path(path))
                })
                .flatten()
        })
        .flatten()
    {
        return Some(value);
    }
    path_owner
        .with_downcast_mut::<ListenerInputTypeViewModel, _>(|owner| {
            owner
                .data_bind_path_referencer
                .with_data_bind_path_mut(|path| {
                    context.with_context(|context| context.get_property_from_path(path))
                })
                .flatten()
        })
        .flatten()
}

fn trigger_value(value: &CoreHandle) -> Option<u32> {
    value.with_downcast::<ViewModelInstanceTrigger, _>(|trigger| trigger.base.property_value())
}

struct ListenerViewModelPropertyBinding {
    parent: RuntimeListenerViewModelWeakHandle,
    view_model_instance_value: Option<CoreHandle>,
    path_owner: CoreHandle,
    dependent_identity: Option<ValueDependentHandle>,
}

#[derive(Clone)]
pub struct RuntimeListenerViewModelPropertyBindingHandle(
    Rc<RefCell<ListenerViewModelPropertyBinding>>,
);

impl RuntimeListenerViewModelPropertyBindingHandle {
    fn new(
        parent: RuntimeListenerViewModelWeakHandle,
        value: CoreHandle,
        path_owner: CoreHandle,
    ) -> Self {
        let binding = Rc::new(RefCell::new(ListenerViewModelPropertyBinding {
            parent,
            view_model_instance_value: Some(value.clone()),
            path_owner,
            dependent_identity: None,
        }));
        let erased: Rc<RefCell<dyn ViewModelValueDependent>> = binding.clone();
        let identity = ValueDependentHandle::runtime(&erased);
        binding.borrow_mut().dependent_identity = Some(identity.clone());
        value.with_downcast_mut::<ViewModelInstanceValue, _>(|value| {
            value.add_dependent(identity);
        });
        let handle = Self(binding);
        handle
    }

    fn with_binding<R>(&self, f: impl FnOnce(&ListenerViewModelPropertyBinding) -> R) -> R {
        f(&self.0.borrow())
    }

    fn with_binding_mut<R>(&self, f: impl FnOnce(&mut ListenerViewModelPropertyBinding) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

impl ListenerViewModelPropertyBinding {
    fn clear_data_context(&mut self) {
        if let (Some(value), Some(identity)) = (
            self.view_model_instance_value.take(),
            self.dependent_identity.as_ref(),
        ) {
            value.with_downcast_mut::<ViewModelInstanceValue, _>(|value| {
                value.remove_dependent(identity);
            });
        }
    }

    fn relink_data_bind(&mut self) {
        let Some(context) = self
            .parent
            .with_listener(|parent| parent.data_context.clone())
            .flatten()
        else {
            return;
        };
        let value = data_context_property(&context, &self.path_owner);
        if value != self.view_model_instance_value {
            self.clear_data_context();
            if let Some(value) = value {
                if let Some(identity) = self.dependent_identity.clone() {
                    value.with_downcast_mut::<ViewModelInstanceValue, _>(|value| {
                        value.add_dependent(identity);
                    });
                }
                self.view_model_instance_value = Some(value);
            }
        }
    }

    fn add_dirt(&mut self) {
        if let Some(value) = self.view_model_instance_value.clone() {
            self.parent
                .with_listener_mut(|parent| parent.report_to_state_machine(value));
        }
    }
}

impl Drop for ListenerViewModelPropertyBinding {
    fn drop(&mut self) {
        self.clear_data_context();
    }
}

impl Dirtyable for ListenerViewModelPropertyBinding {
    fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        ListenerViewModelPropertyBinding::add_dirt(self);
    }
}

impl ViewModelValueDependent for ListenerViewModelPropertyBinding {
    fn relink_data_bind(&mut self) {
        ListenerViewModelPropertyBinding::relink_data_bind(self);
    }
}

struct ListenerViewModel {
    occurrence: RuntimeListenerViewModelWeakHandle,
    state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    listener: CoreHandle,
    data_context: Option<RuntimeDataContextHandle>,
    property_bindings: Vec<RuntimeListenerViewModelPropertyBindingHandle>,
}

#[derive(Clone)]
struct RuntimeListenerViewModelHandle(Rc<RefCell<ListenerViewModel>>);

#[derive(Clone, Default)]
pub struct RuntimeListenerViewModelWeakHandle(Weak<RefCell<ListenerViewModel>>);

impl RuntimeListenerViewModelHandle {
    fn new(listener: ListenerViewModel) -> Self {
        let handle = Self(Rc::new(RefCell::new(listener)));
        handle.0.borrow_mut().occurrence = handle.downgrade();
        handle
    }

    fn downgrade(&self) -> RuntimeListenerViewModelWeakHandle {
        RuntimeListenerViewModelWeakHandle(Rc::downgrade(&self.0))
    }

    fn with_listener<R>(&self, f: impl FnOnce(&ListenerViewModel) -> R) -> R {
        f(&self.0.borrow())
    }

    fn with_listener_mut<R>(&self, f: impl FnOnce(&mut ListenerViewModel) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

impl RuntimeListenerViewModelWeakHandle {
    fn with_listener<R>(&self, f: impl FnOnce(&ListenerViewModel) -> R) -> Option<R> {
        self.0.upgrade().map(|listener| f(&listener.borrow()))
    }

    fn with_listener_mut<R>(&self, f: impl FnOnce(&mut ListenerViewModel) -> R) -> Option<R> {
        self.0
            .upgrade()
            .map(|listener| f(&mut listener.borrow_mut()))
    }
}

impl ListenerViewModel {
    fn new(
        machine: RuntimeStateMachineInstanceWeakHandle,
        listener: CoreHandle,
    ) -> RuntimeListenerViewModelHandle {
        RuntimeListenerViewModelHandle::new(Self {
            occurrence: RuntimeListenerViewModelWeakHandle::default(),
            state_machine_instance: machine,
            listener,
            data_context: None,
            property_bindings: Vec::new(),
        })
    }

    fn clear_data_context(&mut self) {
        for binding in &self.property_bindings {
            binding.with_binding_mut(ListenerViewModelPropertyBinding::clear_data_context);
        }
        self.property_bindings.clear();
        self.data_context = None;
    }

    fn bind_from_context(&mut self, context: RuntimeDataContextHandle) {
        self.clear_data_context();
        self.data_context = Some(context.clone());
        if self
            .listener
            .with_downcast::<StateMachineListenerSingle, _>(|_| ())
            .is_some()
        {
            if let Some(value) = data_context_property(&context, &self.listener) {
                self.property_bindings
                    .push(RuntimeListenerViewModelPropertyBindingHandle::new(
                        self.occurrence.clone(),
                        value,
                        self.listener.clone(),
                    ));
            }
        } else {
            let inputs = self
                .listener
                .with_downcast::<StateMachineListener, _>(|listener| {
                    (0..listener.listener_input_type_count())
                        .filter_map(|index| listener.listener_input_type(index))
                        .filter(|input| {
                            input
                                .with_downcast::<ListenerInputTypeViewModel, _>(|_| ())
                                .is_some()
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for input in inputs {
                if let Some(value) = data_context_property(&context, &input) {
                    self.property_bindings.push(
                        RuntimeListenerViewModelPropertyBindingHandle::new(
                            self.occurrence.clone(),
                            value,
                            input,
                        ),
                    );
                }
            }
        }
        let pending: Vec<CoreHandle> = self
            .property_bindings
            .iter()
            .filter_map(|binding| {
                binding.with_binding(|binding| binding.view_model_instance_value.clone())
            })
            .filter(|value| trigger_value(value).is_some_and(|value| value != 0))
            .collect();
        for value in pending {
            self.report_to_state_machine(value);
        }
    }

    fn report_to_state_machine(&mut self, value: CoreHandle) {
        let occurrence = self.occurrence.clone();
        self.state_machine_instance.with_instance_mut(|machine| {
            let should_report = trigger_value(&value).is_none_or(|value| value != 0);
            if should_report {
                machine.report_listener_view_model(occurrence);
            }
        });
    }
}

#[derive(Clone)]
pub struct RuntimeStateMachineInstanceHandle(Rc<RefCell<StateMachineInstance>>);

#[derive(Clone, Default)]
pub struct RuntimeStateMachineInstanceWeakHandle(Weak<RefCell<StateMachineInstance>>);

impl RuntimeStateMachineInstanceHandle {
    fn new(instance: StateMachineInstance) -> Self {
        Self(Rc::new(RefCell::new(instance)))
    }

    pub fn downgrade(&self) -> RuntimeStateMachineInstanceWeakHandle {
        RuntimeStateMachineInstanceWeakHandle(Rc::downgrade(&self.0))
    }

    pub fn with_instance<R>(&self, f: impl FnOnce(&StateMachineInstance) -> R) -> R {
        f(&self.0.borrow())
    }

    pub fn with_instance_mut<R>(&self, f: impl FnOnce(&mut StateMachineInstance) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

impl RuntimeStateMachineInstanceWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        self.0.upgrade().map(RuntimeStateMachineInstanceHandle)
    }

    pub fn with_instance<R>(&self, f: impl FnOnce(&StateMachineInstance) -> R) -> Option<R> {
        self.upgrade().map(|instance| instance.with_instance(f))
    }

    pub fn with_instance_mut<R>(
        &self,
        f: impl FnOnce(&mut StateMachineInstance) -> R,
    ) -> Option<R> {
        self.upgrade().map(|instance| instance.with_instance_mut(f))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

pub struct StateMachineInstance {
    occurrence: RuntimeStateMachineInstanceWeakHandle,
    runtime: RuntimeServicesHandle,
    reported_events: Vec<EventReport>,
    reporting_events: Vec<EventReport>,
    events_applied_during_loop: Vec<EventReport>,
    machine: CoreHandle,
    artboard_instance: RuntimeArtboardInstanceWeakHandle,
    needs_advance: Rc<Cell<bool>>,
    input_instances: Vec<Option<InputInstance>>,
    layers: Vec<RuntimeStateMachineLayerInstanceHandle>,
    hit_components: Vec<Box<dyn HitComponent>>,
    listener_groups: Vec<RuntimeListenerGroupHandle>,
    parent_state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    parent_nested_artboard: Option<CoreHandle>,
    data_context_handle: Option<RuntimeDataContextHandle>,
    data_bind_container: DataBindContainer,
    listener_view_models: Vec<RuntimeListenerViewModelHandle>,
    reported_listener_view_models: Vec<RuntimeListenerViewModelWeakHandle>,
    reporting_listener_view_models: Vec<RuntimeListenerViewModelWeakHandle>,
    bindable_property_instances: HashMap<CoreHandle, CoreHandle>,
    scripted_objects_map: HashMap<CoreHandle, CoreHandle>,
    bindable_data_binds_to_target: HashMap<CoreHandle, CoreHandle>,
    bindable_data_binds_to_source: HashMap<CoreHandle, CoreHandle>,
    transition_property_instances: HashMap<CoreHandle, HashMap<u32, CoreHandle>>,
    state_keyframe_data_binds: HashMap<RuntimeStateInstanceHandle, Vec<CoreHandle>>,
    draw_order_change_counter: u8,
    focus_manager: RuntimeFocusManagerHandle,
    external_focus_manager: Option<RuntimeFocusManagerHandle>,
    focus_listener_groups: Vec<RuntimeFocusListenerGroupHandle>,
    keyboard_listener_groups: Vec<RuntimeKeyboardListenerGroupHandle>,
    gamepad_listener_groups: Vec<RuntimeGamepadListenerGroupHandle>,
    gamepad_scripted_drawables: Vec<CoreHandle>,
    embedder_gamepads: HashMap<i32, Object>,
    semantic_manager: Option<RuntimeSemanticManagerHandle>,
    external_semantic_manager: Option<RuntimeSemanticManagerHandle>,
    queued_focus_events: Vec<QueuedFocusEvent>,
    semantic_listener_groups: Vec<RuntimeSemanticListenerGroupHandle>,
    queued_semantic_events: Vec<QueuedSemanticEvent>,
    nested_event_listeners: Vec<RuntimeStateMachineInstanceWeakHandle>,
    nested_artboard: Option<CoreHandle>,
    #[cfg(feature = "tools")]
    input_changed_callback: Option<Box<dyn FnMut(RuntimeStateMachineInstanceWeakHandle, u64)>>,
}

impl StateMachineInstance {
    pub fn new(
        machine: CoreHandle,
        artboard_instance: RuntimeArtboardInstanceWeakHandle,
        runtime: RuntimeServicesHandle,
    ) -> RuntimeStateMachineInstanceHandle {
        let instance = Self {
            occurrence: RuntimeStateMachineInstanceWeakHandle::default(),
            runtime,
            reported_events: Vec::new(),
            reporting_events: Vec::new(),
            events_applied_during_loop: Vec::new(),
            machine,
            artboard_instance,
            needs_advance: Rc::new(Cell::new(false)),
            input_instances: Vec::new(),
            layers: Vec::new(),
            hit_components: Vec::new(),
            listener_groups: Vec::new(),
            parent_state_machine_instance: RuntimeStateMachineInstanceWeakHandle::default(),
            parent_nested_artboard: None,
            data_context_handle: None,
            data_bind_container: DataBindContainer::default(),
            listener_view_models: Vec::new(),
            reported_listener_view_models: Vec::new(),
            reporting_listener_view_models: Vec::new(),
            bindable_property_instances: HashMap::new(),
            scripted_objects_map: HashMap::new(),
            bindable_data_binds_to_target: HashMap::new(),
            bindable_data_binds_to_source: HashMap::new(),
            transition_property_instances: HashMap::new(),
            state_keyframe_data_binds: HashMap::new(),
            draw_order_change_counter: 0,
            focus_manager: RuntimeFocusManagerHandle::new(FocusManager::new()),
            external_focus_manager: None,
            focus_listener_groups: Vec::new(),
            keyboard_listener_groups: Vec::new(),
            gamepad_listener_groups: Vec::new(),
            gamepad_scripted_drawables: Vec::new(),
            embedder_gamepads: HashMap::new(),
            semantic_manager: None,
            external_semantic_manager: None,
            queued_focus_events: Vec::new(),
            semantic_listener_groups: Vec::new(),
            queued_semantic_events: Vec::new(),
            nested_event_listeners: Vec::new(),
            nested_artboard: None,
            #[cfg(feature = "tools")]
            input_changed_callback: None,
        };
        let handle = RuntimeStateMachineInstanceHandle::new(instance);
        handle.with_instance_mut(|instance| {
            instance.occurrence = handle.downgrade();
            instance
                .data_bind_container
                .set_state_machine_owner(handle.downgrade());
            let mut input_notifier = InputInstanceNotifier::new(Rc::clone(&instance.needs_advance));
            #[cfg(feature = "tools")]
            input_notifier.set_machine(handle.downgrade());

            let input_count = instance
                .machine
                .with_downcast::<StateMachine, _>(StateMachine::input_count)
                .unwrap_or(0);
            instance.input_instances.resize_with(input_count, || None);
            for index in 0..input_count {
                let Some(input) = instance
                    .machine
                    .with_downcast::<StateMachine, _>(|machine| machine.input(index))
                    .flatten()
                else {
                    continue;
                };
                instance.input_instances[index] =
                    InputInstance::from_definition(&input, input_notifier.clone());
                #[cfg(feature = "tools")]
                if let Some(input_instance) = instance.input_instances[index].as_mut() {
                    input_instance.base_mut().set_index(index as u64);
                }
            }

            let layer_count = instance
                .machine
                .with_downcast::<StateMachine, _>(StateMachine::layer_count)
                .unwrap_or(0);
            for index in 0..layer_count {
                let Some(layer) = instance
                    .machine
                    .with_downcast::<StateMachine, _>(|machine| machine.layer(index))
                    .flatten()
                else {
                    continue;
                };
                let layer_instance = RuntimeStateMachineLayerInstanceHandle::new(
                    StateMachineLayerInstance::default(),
                );
                let runtime_services = Rc::clone(&instance.runtime);
                layer_instance.with_layer_mut(|layer_instance| {
                    layer_instance.init(
                        instance,
                        runtime_services,
                        layer,
                        artboard_instance.clone(),
                    );
                });
                instance.layers.push(layer_instance);
            }

            instance.initialize_data_binds();
            let mut hit_lookup = HashMap::new();
            instance.initialize_listeners(&mut hit_lookup);
            instance.initialize_component_provided_listeners(&mut hit_lookup);
            instance.initialize_nested_hit_components();
            instance.initialize_text_inputs();
            instance.initialize_scripted_objects();
            instance.sort_hit_components();
            let manager = instance.focus_manager();
            let _ = artboard_instance.with_artboard_mut(|artboard| {
                artboard.build_focus_tree(Some(manager), None);
            });
        });
        handle
    }

    fn runtime_mut(&self) -> RefMut<'_, dyn StateMachineInstanceRuntime> {
        RefMut::map(self.runtime.borrow_mut(), |runtime| runtime.as_mut())
    }

    pub fn listener_has(&self, listener: &CoreHandle, listener_type: ListenerType) -> bool {
        listener
            .with_downcast::<StateMachineListener, _>(|listener| {
                listener.has_listener(listener_type)
            })
            .unwrap_or(false)
    }

    fn listener_has_any(&self, listener: &CoreHandle, listener_types: &[ListenerType]) -> bool {
        listener
            .with_downcast::<StateMachineListener, _>(|listener| {
                listener.has_listeners(listener_types)
            })
            .unwrap_or(false)
    }

    pub fn perform_listener_changes(
        &mut self,
        listener: &CoreHandle,
        invocation: ListenerInvocation,
    ) {
        let runtime = Rc::clone(&self.runtime);
        runtime
            .borrow_mut()
            .listener_perform_changes(listener, self, &invocation);
    }

    pub fn semantic_constraints_met(
        &self,
        listener: &CoreHandle,
        action: SemanticActionType,
    ) -> bool {
        listener
            .with_downcast::<StateMachineListener, _>(|listener| {
                ListenerInputTypeSemantic::semantic_listener_constraints_met(Some(listener), action)
            })
            .unwrap_or(false)
    }

    fn initialize_data_binds(&mut self) {
        let data_bind_count = self
            .machine
            .with_downcast::<StateMachine, _>(StateMachine::data_bind_count)
            .unwrap_or(0);
        for index in 0..data_bind_count {
            let Some(source) = self
                .machine
                .with_downcast::<StateMachine, _>(|machine| machine.data_bind(index))
                .flatten()
            else {
                continue;
            };
            let Some(original_target) = self.runtime.borrow_mut().data_bind_target(&source) else {
                continue;
            };
            let clone = self.runtime.borrow_mut().clone_data_bind(&source);
            let file = self.runtime.borrow_mut().data_bind_file(&source);
            self.runtime.borrow_mut().data_bind_set_file(&clone, file);
            if let Some(converter) = self.runtime.borrow_mut().data_bind_converter(&source) {
                let converter_clone = self.runtime.borrow_mut().clone_data_converter(&converter);
                self.runtime
                    .borrow_mut()
                    .data_bind_set_converter(&clone, converter_clone);
            }
            self.add_data_bind(clone.clone());
            if self.runtime.borrow_mut().data_bind_bindable_target(&source) {
                let property = self
                    .bindable_property_instances
                    .entry(original_target.clone())
                    .or_insert_with(|| {
                        self.runtime
                            .borrow_mut()
                            .clone_bindable_property(&original_target)
                    })
                    .clone();
                let property_key = self.runtime.borrow_mut().data_bind_property_key(&clone);
                self.runtime.borrow_mut().configure_data_bind_target(
                    &clone,
                    property.clone(),
                    property_key,
                );
                if self.runtime.borrow_mut().data_bind_flags(&clone) & 1 != 0 {
                    self.bindable_data_binds_to_source.insert(property, clone);
                } else {
                    self.bindable_data_binds_to_target.insert(property, clone);
                }
            } else if self
                .runtime
                .borrow_mut()
                .data_bind_is_transition_target(&source)
            {
                let property = self.runtime.borrow_mut().make_transition_property();
                self.transition_property_instances
                    .entry(original_target)
                    .or_default()
                    .insert(
                        self.runtime.borrow_mut().data_bind_property_key(&source),
                        property.clone(),
                    );
                self.runtime.borrow_mut().configure_data_bind_target(
                    &clone,
                    property,
                    BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY as u32,
                );
            }
        }
    }

    fn initialize_listeners(&mut self, hit_lookup: &mut HashMap<CoreHandle, usize>) {
        let machine = self.occurrence.clone();
        let listener_count = self
            .machine
            .with_downcast::<StateMachine, _>(StateMachine::listener_count)
            .unwrap_or(0);
        for index in 0..listener_count {
            let Some(listener) = self
                .machine
                .with_downcast::<StateMachine, _>(|machine| machine.listener(index))
                .flatten()
            else {
                continue;
            };
            if self.listener_has(&listener, ListenerType::Event) {
                continue;
            }
            if self.listener_has(&listener, ListenerType::ViewModel) {
                self.listener_view_models
                    .push(ListenerViewModel::new(machine.clone(), listener));
                continue;
            }
            let target = self.runtime.borrow_mut().artboard_resolve(
                &self.artboard_instance,
                self.runtime.borrow_mut().listener_target_id(&listener),
            );
            if self.listener_has(&listener, ListenerType::Focus)
                || self.listener_has(&listener, ListenerType::Blur)
            {
                if let Some(focus_data) = target
                    .as_ref()
                    .and_then(|target| self.runtime.borrow_mut().resolve_focus_data(target))
                {
                    let group = RuntimeFocusListenerGroupHandle::new(
                        focus_data,
                        listener.clone(),
                        machine.clone(),
                    );
                    self.focus_listener_groups.push(group);
                }
            }
            if self.listener_has(&listener, ListenerType::Keyboard)
                || self.listener_has(&listener, ListenerType::TextInput)
            {
                if let Some(focus_data) = target
                    .as_ref()
                    .and_then(|target| self.runtime.borrow_mut().resolve_focus_data(target))
                {
                    let group = RuntimeKeyboardListenerGroupHandle::new(
                        focus_data,
                        Some(listener.clone()),
                        machine.clone(),
                    );
                    self.keyboard_listener_groups.push(group);
                }
            }
            if self.listener_has(&listener, ListenerType::SemanticAction) {
                if let Some(semantic_data) = target
                    .as_ref()
                    .and_then(|target| self.runtime.borrow_mut().resolve_semantic_data(target))
                {
                    let group = RuntimeSemanticListenerGroupHandle::new(
                        semantic_data,
                        listener.clone(),
                        machine.clone(),
                    );
                    self.semantic_listener_groups.push(group);
                }
            }
            if self.listener_has_any(&listener, &POINTER_HIT_LISTENER_TYPES) {
                let group =
                    RuntimeListenerGroupHandle::new(Box::new(ListenerGroup::new(listener.clone())));
                if let Some(target) = target.as_ref() {
                    let is_layout = self.runtime.borrow_mut().component_is_layout(target);
                    let hit_target = if is_layout {
                        self.runtime.borrow_mut().component_proxy(target)
                    } else {
                        Some(target.clone())
                    };
                    if let Some(hit_target) = hit_target {
                        self.add_to_hit_lookup(hit_target, is_layout, hit_lookup, group, false);
                    }
                }
                self.listener_groups.push(group);
            }
            if self.listener_has(&listener, ListenerType::Gamepad) {
                if let Some(focus_data) = target
                    .as_ref()
                    .and_then(|target| self.runtime.borrow_mut().resolve_focus_data(target))
                {
                    let group = RuntimeGamepadListenerGroupHandle::new(
                        focus_data,
                        Some(listener.clone()),
                        machine.clone(),
                    );
                    self.gamepad_listener_groups.push(group);
                }
            }
        }
    }

    fn initialize_component_provided_listeners(
        &mut self,
        hit_lookup: &mut HashMap<CoreHandle, usize>,
    ) {
        let providers: Vec<CoreHandle> = self
            .runtime
            .borrow_mut()
            .artboard_objects(&self.artboard_instance)
            .into_iter()
            .filter_map(|object| self.runtime.borrow_mut().object_listener_provider(&object))
            .collect();
        for provider in providers {
            let groups = provider
                .with_mut(|provider| {
                    ListenerGroupProvider::from(provider)
                        .expect("a listener provider exposes ListenerGroupProvider")
                        .listener_groups()
                })
                .expect("a listener provider remains in its CoreArena");
            for group_with_targets in groups {
                let (group, targets) = group_with_targets.into_parts();
                for target in targets {
                    let target_handle = target.component();
                    let layout = self
                        .runtime
                        .borrow_mut()
                        .component_is_layout(&target_handle)
                        || self
                            .runtime
                            .borrow_mut()
                            .component_is_drawable_proxy(&target_handle);
                    self.add_to_hit_lookup(
                        target_handle,
                        layout,
                        hit_lookup,
                        group.clone(),
                        target.is_opaque(),
                    );
                }
                self.listener_groups.push(group);
            }
            let runtime = Rc::clone(&self.runtime);
            let hits = runtime
                .borrow_mut()
                .provided_hit_components(&provider, self);
            self.hit_components.extend(hits);
        }
    }

    fn initialize_nested_hit_components(&mut self) {
        for nested in self
            .runtime
            .borrow_mut()
            .artboard_nested_artboards(&self.artboard_instance)
        {
            self.hit_components
                .push(Box::new(HitNestedArtboard { component: nested }));
            for animation in self.runtime.borrow_mut().nested_animations(&nested) {
                if self
                    .runtime
                    .borrow_mut()
                    .nested_is_state_machine(&animation)
                {
                    let notifier = self
                        .runtime
                        .borrow_mut()
                        .nested_state_machine_instance(&animation);
                    if let Some(notifier) = notifier {
                        let listener = self.occurrence.clone();
                        notifier.with_instance_mut(|notifier| {
                            notifier.set_parent_nested_artboard(nested.clone());
                            notifier.set_nested_artboard(nested.clone());
                            notifier.add_nested_event_listener(listener);
                        });
                    }
                } else {
                    self.runtime.borrow_mut().nested_add_event_listener(
                        &animation,
                        &nested,
                        self.occurrence.clone(),
                    );
                }
            }
        }
        for list in self
            .runtime
            .borrow_mut()
            .artboard_component_lists(&self.artboard_instance)
        {
            self.hit_components
                .push(Box::new(HitComponentList { component: list }));
        }
    }

    fn initialize_text_inputs(&mut self) {
        let machine = self.occurrence.clone();
        for text_input in self
            .runtime
            .borrow_mut()
            .artboard_text_inputs(&self.artboard_instance)
        {
            let group = self
                .runtime
                .borrow_mut()
                .make_text_input_listener_group(&text_input, machine);
            let mut hit = HitDrawable::new(
                self.runtime.borrow_mut().as_ref(),
                text_input.clone(),
                text_input,
                true,
                true,
                true,
            );
            hit.add_listener(group.clone());
            self.hit_components.push(Box::new(hit));
            self.listener_groups.push(group);
        }
    }

    fn initialize_scripted_objects(&mut self) {
        let scripted_objects = self
            .machine
            .with_downcast::<StateMachine, _>(StateMachine::scripted_objects)
            .unwrap_or_default();
        for source in scripted_objects {
            let clone = self.runtime.borrow_mut().scripted_clone(&source, self);
            self.scripted_objects_map.insert(source, clone);
        }
        for object in self.scripted_objects_map.values() {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.set_data_context(self.data_context_handle.clone());
                }
            });
        }
        self.init_scripted_objects();
        for object in self
            .runtime
            .borrow_mut()
            .artboard_objects(&self.artboard_instance)
        {
            let Some(scripted) = self.runtime.borrow_mut().object_scripted(&object) else {
                continue;
            };
            if self.runtime.borrow_mut().scripted_wants_keyboard(&scripted)
                || self.runtime.borrow_mut().scripted_wants_text(&scripted)
            {
                if let Some(focus_data) = self.runtime.borrow_mut().resolve_focus_data(&object) {
                    let group = RuntimeKeyboardListenerGroupHandle::new(
                        focus_data,
                        None,
                        self.occurrence.clone(),
                    );
                    self.keyboard_listener_groups.push(group);
                }
            }
            if self.runtime.borrow_mut().scripted_wants_gamepad(&scripted) {
                self.gamepad_scripted_drawables.push(object);
            }
        }
    }

    fn add_to_hit_lookup(
        &mut self,
        target: CoreHandle,
        is_layout_component: bool,
        hit_lookup: &mut HashMap<CoreHandle, usize>,
        listener_group: RuntimeListenerGroupHandle,
        is_opaque: bool,
    ) {
        if is_layout_component {
            let index = if let Some(&index) = hit_lookup.get(&target) {
                index
            } else {
                let hit = HitDrawable::new(
                    self.runtime.borrow_mut().as_ref(),
                    target.clone(),
                    target.clone(),
                    is_opaque,
                    false,
                    true,
                );
                let index = self.hit_components.len();
                self.hit_components.push(Box::new(hit));
                hit_lookup.insert(target, index);
                index
            };
            let drawable = self.hit_components[index].as_mut().as_hit_drawable_mut();
            if let Some(drawable) = drawable {
                drawable.add_listener(listener_group);
                drawable.is_opaque |= is_opaque;
            }
            return;
        }
        if self.runtime.borrow_mut().component_is_shape(&target)
            || self.runtime.borrow_mut().component_is_text_run(&target)
        {
            let index = if let Some(&index) = hit_lookup.get(&target) {
                index
            } else {
                self.runtime.borrow_mut().component_mark_hit_path(&target);
                let drawable = if self.runtime.borrow_mut().component_is_text_run(&target) {
                    self.runtime
                        .borrow_mut()
                        .text_run_text_component(&target)
                        .unwrap_or_else(|| target.clone())
                } else {
                    target.clone()
                };
                let hit = HitDrawable::new(
                    self.runtime.borrow_mut().as_ref(),
                    drawable,
                    target.clone(),
                    false,
                    true,
                    true,
                );
                let index = self.hit_components.len();
                self.hit_components.push(Box::new(hit));
                hit_lookup.insert(target, index);
                index
            };
            if let Some(drawable) = self.hit_components[index].as_mut().as_hit_drawable_mut() {
                drawable.add_listener(listener_group);
            }
            return;
        }
        if self.runtime.borrow_mut().component_is_container(&target) {
            for child in self.runtime.borrow_mut().component_children(&target) {
                let is_layout = self.runtime.borrow_mut().component_is_layout(&child);
                self.add_to_hit_lookup(
                    child,
                    is_layout,
                    hit_lookup,
                    listener_group.clone(),
                    is_opaque,
                );
            }
        }
    }

    fn normalize_pointer_position(&self, mut position: Vec2D) -> Vec2D {
        if self
            .runtime
            .borrow_mut()
            .artboard_frame_origin(&self.artboard_instance)
        {
            let origin = self
                .runtime
                .borrow_mut()
                .artboard_origin(&self.artboard_instance);
            let size = self
                .runtime
                .borrow_mut()
                .artboard_layout_size(&self.artboard_instance);
            position = Vec2D::new(
                position.x - origin.x * size.x,
                position.y - origin.y * size.y,
            );
        }
        self.runtime
            .borrow_mut()
            .artboard_inverse_self_transform(&self.artboard_instance, position)
            .unwrap_or(position)
    }

    fn update_listeners(
        &mut self,
        position: Vec2D,
        hit_type: ListenerType,
        pointer_id: i32,
        timestamp: f32,
    ) -> HitResult {
        let position = self.normalize_pointer_position(position);
        for group in &self.listener_groups {
            group.with_group_mut(|group| group.reset(pointer_id));
        }
        let mut hit_components = std::mem::take(&mut self.hit_components);
        for component in &mut hit_components {
            component.prepare_event(
                self.runtime.borrow_mut().as_mut(),
                position,
                hit_type,
                pointer_id,
            );
        }
        let mut hit_something = false;
        let mut hit_opaque = false;
        for component in &mut hit_components {
            let runtime = Rc::clone(&self.runtime);
            let result = {
                let mut runtime = runtime.borrow_mut();
                component.process_event(
                    runtime.as_mut(),
                    self,
                    position,
                    hit_type,
                    !hit_opaque,
                    timestamp,
                    pointer_id,
                )
            };
            if result != HitResult::None {
                hit_something = true;
                hit_opaque |= result == HitResult::HitOpaque;
            }
        }
        self.hit_components = hit_components;
        if hit_type == ListenerType::Exit {
            for group in &self.listener_groups {
                group.with_group_mut(|group| group.release_event(pointer_id));
            }
        }
        if !hit_something {
            HitResult::None
        } else if hit_opaque {
            HitResult::HitOpaque
        } else {
            HitResult::Hit
        }
    }

    pub fn hit_test(&self, position: Vec2D) -> bool {
        let position = self.normalize_pointer_position(position);
        self.hit_components
            .iter()
            .any(|component| component.hit_test(self.runtime.borrow_mut().as_ref(), position))
    }

    pub fn pointer_move(&mut self, position: Vec2D, timestamp: f32, id: i32) -> HitResult {
        self.update_listeners(position, ListenerType::Move, id, timestamp)
    }

    pub fn pointer_down(&mut self, position: Vec2D, id: i32) -> HitResult {
        self.update_listeners(position, ListenerType::Down, id, 0.0)
    }

    pub fn pointer_up(&mut self, position: Vec2D, id: i32) -> HitResult {
        self.update_listeners(position, ListenerType::Up, id, 0.0)
    }

    pub fn pointer_exit(&mut self, position: Vec2D, id: i32) -> HitResult {
        self.update_listeners(position, ListenerType::Exit, id, 0.0)
    }

    pub fn drag_start(
        &mut self,
        position: Vec2D,
        _timestamp: f32,
        disable_pointer: bool,
        pointer_id: i32,
    ) -> HitResult {
        if disable_pointer {
            self.disable_pointer_events(pointer_id);
        }
        self.update_listeners(position, ListenerType::DragStart, pointer_id, 0.0)
    }

    pub fn drag_end(&mut self, position: Vec2D, timestamp: f32, pointer_id: i32) -> HitResult {
        self.enable_pointer_events(pointer_id);
        let hit = self.update_listeners(position, ListenerType::DragEnd, pointer_id, 0.0);
        self.pointer_move(position, timestamp, pointer_id);
        hit
    }

    fn sort_hit_components(&mut self) {
        let components: Vec<CoreHandle> = self
            .hit_components
            .iter()
            .map(|component| component.component())
            .collect();
        let order = self
            .runtime
            .borrow_mut()
            .artboard_ordered_hit_components(&self.artboard_instance, &components);
        let mut old: Vec<Option<Box<dyn HitComponent>>> =
            self.hit_components.drain(..).map(Some).collect();
        for index in order {
            if index < old.len() {
                if let Some(component) = old[index].take() {
                    self.hit_components.push(component);
                }
            }
        }
        self.hit_components
            .extend(old.into_iter().filter_map(|value| value));
    }

    pub fn try_change_state(&mut self) -> bool {
        self.data_bind_container.update_data_binds(false);
        let mut changed = false;
        let layers = self.layers.clone();
        for layer in layers {
            changed |= layer.with_layer_mut(|layer| layer.update_state(self));
        }
        changed
    }

    pub fn apply_events(&mut self) {
        self.events_applied_during_loop.clear();
        let mut iteration = 0;
        while (!self.reported_events.is_empty() || !self.reported_listener_view_models.is_empty())
            && iteration < 100
        {
            iteration += 1;
            self.data_bind_container.update_data_binds(false);
            self.reporting_events = std::mem::take(&mut self.reported_events);
            self.reporting_listener_view_models =
                std::mem::take(&mut self.reported_listener_view_models);
            if iteration > 1 {
                self.events_applied_during_loop
                    .extend(self.reporting_events.iter().cloned());
            }
            let events = self.reporting_events.clone();
            let view_models = self.reporting_listener_view_models.clone();
            self.notify_event_listeners(&events, None);
            self.notify_listener_view_models(&view_models);
        }
        if iteration >= 100 {
            eprintln!(
                "{} StateMachine exceeded max event iterations on artboard {}",
                self.name(),
                self.runtime
                    .borrow_mut()
                    .artboard_name(&self.artboard_instance)
            );
        }
    }

    pub fn set_external_focus_manager(&mut self, manager: Option<RuntimeFocusManagerHandle>) {
        let unchanged = match (&self.external_focus_manager, &manager) {
            (Some(current), Some(manager)) => current.ptr_eq(manager),
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }
        let _ = self.artboard_instance.with_artboard_mut(|artboard| {
            if artboard.focus_manager().is_some() {
                artboard.cleanup_focus_tree();
            }
        });
        self.external_focus_manager = manager;
        let focus_manager = self.focus_manager();
        let _ = self.artboard_instance.with_artboard_mut(|artboard| {
            artboard.build_focus_tree(Some(focus_manager), None);
        });
    }

    pub fn set_external_focus_manager_handle(&mut self, manager: RuntimeFocusManagerHandle) {
        self.set_external_focus_manager(Some(manager));
    }

    pub fn focus_manager(&self) -> RuntimeFocusManagerHandle {
        self.external_focus_manager
            .clone()
            .unwrap_or_else(|| self.focus_manager.clone())
    }

    pub fn internal_focus_manager(&self) -> RuntimeFocusManagerHandle {
        self.focus_manager.clone()
    }

    pub fn has_external_focus_manager(&self) -> bool {
        self.external_focus_manager.is_some()
    }

    pub fn enable_semantics(&mut self) {
        if self.semantic_manager().is_some() {
            return;
        }
        self.semantic_manager = Some(RuntimeSemanticManagerHandle::new(SemanticManager::new()));
        let manager = self.semantic_manager();
        let _ = self.artboard_instance.with_artboard_mut(|artboard| {
            artboard.build_semantic_tree(manager, None);
        });
    }

    pub fn semantic_manager(&self) -> Option<RuntimeSemanticManagerHandle> {
        self.external_semantic_manager
            .clone()
            .or_else(|| self.semantic_manager.clone())
    }

    pub fn set_external_semantic_manager(
        &mut self,
        manager: Option<RuntimeSemanticManagerHandle>,
        parent_node: Option<SemanticNodeRef>,
    ) {
        let unchanged = match (&self.external_semantic_manager, &manager) {
            (Some(current), Some(manager)) => current.ptr_eq(manager),
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }
        let _ = self.artboard_instance.with_artboard_mut(|artboard| {
            if artboard.semantic_manager().is_some() {
                artboard.cleanup_semantic_tree();
            }
        });
        self.external_semantic_manager = manager;
        let manager = self.semantic_manager();
        let _ = self.artboard_instance.with_artboard_mut(|artboard| {
            artboard.build_semantic_tree(manager, parent_node);
        });
    }

    pub fn set_external_semantic_manager_handle(
        &mut self,
        manager: RuntimeSemanticManagerHandle,
        parent_node: Option<SemanticNodeRef>,
    ) {
        self.set_external_semantic_manager(Some(manager), parent_node);
    }

    pub fn queue_focus_event(&mut self, group: RuntimeFocusListenerGroupHandle, is_focus: bool) {
        self.queued_focus_events
            .push(QueuedFocusEvent { group, is_focus });
        self.needs_advance.set(true);
    }

    pub fn set_focus(&mut self, focus_data: Option<CoreHandle>) {
        let manager = self.focus_manager();
        if let Some(node) = focus_data.and_then(|focus_data| {
            focus_data.with_downcast_mut::<FocusData, _>(FocusData::focus_node)
        }) {
            manager.with_focus_manager_mut(|manager| manager.set_focus(node));
        } else {
            manager.with_focus_manager_mut(FocusManager::clear_focus);
        }
    }

    pub fn focus_state(&self) -> FocusState {
        self.focus_manager().with_focus_manager(|manager| {
            let primary = manager.primary_focus();
            let expects_keyboard_input = primary
                .as_ref()
                .and_then(|node| node.borrow().focusable())
                .is_some_and(|focusable| focusable.borrow().accepts_keyboard_input());
            FocusState {
                has_focus: primary.is_some(),
                expects_keyboard_input,
            }
        })
    }

    fn process_focus_events(&mut self) {
        let events = std::mem::take(&mut self.queued_focus_events);
        for event in events {
            let (listener, listens) = event.group.with_group(|group| {
                (
                    group.listener(),
                    if event.is_focus {
                        group.is_focus_listener()
                    } else {
                        group.is_blur_listener()
                    },
                )
            });
            if !listens {
                continue;
            }
            let listener_index = self
                .focus_listener_groups
                .iter()
                .position(|group| group.ptr_eq(&event.group))
                .expect("a queued focus listener remains owned until dispatch");
            let invocation = ListenerInvocation::focus(listener_index, event.is_focus);
            self.perform_listener_changes(&listener, invocation);
        }
    }

    pub fn queue_semantic_event(
        &mut self,
        group: RuntimeSemanticListenerGroupHandle,
        action_type: SemanticActionType,
    ) {
        self.queued_semantic_events
            .push(QueuedSemanticEvent { group, action_type });
        self.needs_advance.set(true);
    }

    fn process_semantic_events(&mut self) {
        let events = std::mem::take(&mut self.queued_semantic_events);
        for event in events {
            let listener = event.group.with_group(|group| group.listener());
            let listener_index = self
                .semantic_listener_groups
                .iter()
                .position(|group| group.ptr_eq(&event.group))
                .expect("a queued semantic listener remains owned until dispatch");
            let invocation = ListenerInvocation::semantic(listener_index, event.action_type as u8);
            self.perform_listener_changes(&listener, invocation);
        }
    }

    pub fn fire_semantic_action(&mut self, node_id: u32, action_type: u8) {
        let Some(manager) = self.semantic_manager() else {
            return;
        };
        let semantic_data = manager
            .with_semantic_manager(|manager| manager.node_by_id(node_id))
            .and_then(|node| node.borrow().semantic_data.clone());
        let Some(semantic_data) = semantic_data else {
            return;
        };
        semantic_data.with_mut(|semantic_data| {
            let Some(semantic_data) = semantic_data.as_semantic_data_mut() else {
                return;
            };
            match SemanticActionType::from_raw(action_type as u32) {
                Some(SemanticActionType::Tap) => semantic_data.fire_semantic_tap(),
                Some(SemanticActionType::Increase) => semantic_data.fire_semantic_increase(),
                Some(SemanticActionType::Decrease) => semantic_data.fire_semantic_decrease(),
                None => {}
            }
        });
    }

    pub fn advance(&mut self, seconds: f32, new_frame: bool) -> bool {
        let counter = self
            .runtime
            .borrow_mut()
            .artboard_draw_order_change_counter(&self.artboard_instance);
        if self.draw_order_change_counter != counter {
            self.draw_order_change_counter = counter;
            self.sort_hit_components();
        }
        if new_frame {
            self.process_focus_events();
            self.process_semantic_events();
            self.apply_events();
            self.needs_advance.set(false);
        }
        self.data_bind_container.update_data_binds(false);
        let layers = self.layers.clone();
        for layer in layers {
            if layer.with_layer_mut(|layer| layer.advance(self, seconds, new_frame)) {
                self.needs_advance.set(true);
            }
        }
        if self.data_bind_container.advance_data_binds(seconds) {
            self.needs_advance.set(true);
        }
        for input in self.input_instances.iter_mut().flatten() {
            input.advanced();
        }
        self.needs_advance.get()
            || !self.reported_events.is_empty()
            || !self.reported_listener_view_models.is_empty()
    }

    pub fn advance_seconds(&mut self, seconds: f32) -> bool {
        self.advance(seconds, true)
    }

    pub fn advanced_data_context(&mut self) {
        if let Some(data_context) = self.data_context_handle.as_ref() {
            data_context.with_context(DataContext::advanced);
        }
    }

    pub fn reset(&mut self) {
        self.advanced_data_context();
        self.artboard_instance
            .with_artboard_mut(|artboard| artboard.base.reset());
    }

    pub fn advance_and_apply(&mut self, seconds: f32) -> bool {
        self.advance_and_apply_view_models(seconds, true)
    }

    pub fn advance_and_apply_view_models(
        &mut self,
        seconds: f32,
        advance_view_models: bool,
    ) -> bool {
        let root_flags = AdvanceFlags(
            AdvanceFlags::IS_ROOT.0
                | AdvanceFlags::ANIMATE.0
                | AdvanceFlags::ADVANCE_NESTED.0
                | AdvanceFlags::NEW_FRAME.0,
        );
        let loop_flags = AdvanceFlags(
            AdvanceFlags::IS_ROOT.0 | AdvanceFlags::ANIMATE.0 | AdvanceFlags::ADVANCE_NESTED.0,
        );
        let mut keep_going = self.advance(seconds, true) || seconds == 0.0;
        let manager = self.focus_manager();
        manager.with_focus_manager_mut(FocusManager::drop_focus_if_focus_target_hidden);
        if self
            .artboard_instance
            .with_artboard_mut(|artboard| artboard.base.advance_internal(seconds, root_flags))
            .unwrap_or(false)
        {
            keep_going = true;
        }
        for _ in 0..5 {
            if self
                .artboard_instance
                .with_artboard_mut(|artboard| artboard.base.update_pass(true))
                .unwrap_or(false)
            {
                keep_going = true;
            }
            if self.try_change_state() {
                self.advance(0.0, false);
                keep_going = true;
            }
            if self
                .artboard_instance
                .with_artboard_mut(|artboard| artboard.base.advance_internal(0.0, loop_flags))
                .unwrap_or(false)
            {
                keep_going = true;
            }
            if advance_view_models {
                self.reset();
            } else {
                self.artboard_instance
                    .with_artboard_mut(|artboard| artboard.base.reset());
            }
            if !self
                .runtime
                .borrow_mut()
                .artboard_has_component_dirt(&self.artboard_instance)
            {
                break;
            }
        }
        if advance_view_models {
            self.artboard_instance
                .with_artboard_mut(|artboard| artboard.base.advance_scripted_view_models());
        }
        keep_going
            || !self.reported_events.is_empty()
            || !self.reported_listener_view_models.is_empty()
    }

    pub fn mark_needs_advance(&mut self) {
        self.needs_advance.set(true);
    }

    pub fn needs_advance(&self) -> bool {
        self.needs_advance.get()
    }

    pub fn reset_state(&mut self) {
        let layers = self.layers.clone();
        for layer in layers {
            layer.with_layer_mut(|layer| layer.reset_state(self));
        }
    }

    pub fn name(&self) -> String {
        self.machine
            .with_downcast::<StateMachine, _>(|machine| machine.base.name().to_owned())
            .unwrap_or_default()
    }

    pub fn state_machine(&self) -> CoreHandle {
        self.machine.clone()
    }

    pub fn artboard(&self) -> RuntimeArtboardInstanceWeakHandle {
        self.artboard_instance.clone()
    }

    pub fn input_count(&self) -> usize {
        self.input_instances.len()
    }

    pub fn input(&self, index: usize) -> Option<&SMIInput> {
        self.input_instances
            .get(index)
            .and_then(Option::as_ref)
            .map(InputInstance::base)
    }

    pub fn number_input_value(&self, index: u32) -> Option<f32> {
        let InputInstance::Number(value) = self.input_instances.get(index as usize)?.as_ref()?
        else {
            return None;
        };
        Some(value.value())
    }

    pub fn bindable_property_number_value(&self, property: &CoreHandle) -> Option<f32> {
        self.runtime
            .borrow_mut()
            .bindable_property_number_value(property)
    }

    pub fn bindable_property_comparison_value(
        &self,
        property: &CoreHandle,
    ) -> Option<RuntimeComparisonValue> {
        self.runtime
            .borrow_mut()
            .bindable_property_comparison_value(property)
    }

    pub fn component_comparison_value(
        &self,
        object_id: u32,
        property_key: u32,
    ) -> Option<RuntimeComparisonValue> {
        self.runtime
            .borrow_mut()
            .component_comparison_value(object_id, property_key)
    }

    pub fn artboard_layout_size(&self) -> Option<(f32, f32)> {
        self.runtime
            .borrow_mut()
            .artboard_layout_dimensions(&self.artboard_instance)
    }

    pub fn bindable_source_changed_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) -> bool {
        self.runtime
            .borrow_mut()
            .bindable_source_changed_in_layer(property, layer)
    }

    pub fn use_bindable_property_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        self.runtime
            .borrow_mut()
            .use_bindable_property_in_layer(property, layer);
    }

    pub fn get_bool(&self, name: &str) -> Option<&SMIBool> {
        self.input_instances.iter().flatten().find_map(|instance| {
            let InputInstance::Bool(value) = instance else {
                return None;
            };
            (value.base.name() == name).then_some(value.as_ref())
        })
    }

    pub fn get_number(&self, name: &str) -> Option<&SMINumber> {
        self.input_instances.iter().flatten().find_map(|instance| {
            let InputInstance::Number(value) = instance else {
                return None;
            };
            (value.base.name() == name).then_some(value.as_ref())
        })
    }

    pub fn get_trigger(&self, name: &str) -> Option<&SMITrigger> {
        self.input_instances.iter().flatten().find_map(|instance| {
            let InputInstance::Trigger(value) = instance else {
                return None;
            };
            (value.base.name() == name).then_some(value.as_ref())
        })
    }

    pub fn get_bool_mut(&mut self, name: &str) -> Option<&mut SMIBool> {
        self.input_instances
            .iter_mut()
            .flatten()
            .find_map(|instance| {
                let InputInstance::Bool(value) = instance else {
                    return None;
                };
                (value.base.name() == name).then_some(value.as_mut())
            })
    }

    pub fn get_number_mut(&mut self, name: &str) -> Option<&mut SMINumber> {
        self.input_instances
            .iter_mut()
            .flatten()
            .find_map(|instance| {
                let InputInstance::Number(value) = instance else {
                    return None;
                };
                (value.base.name() == name).then_some(value.as_mut())
            })
    }

    pub fn get_trigger_mut(&mut self, name: &str) -> Option<&mut SMITrigger> {
        self.input_instances
            .iter_mut()
            .flatten()
            .find_map(|instance| {
                let InputInstance::Trigger(value) = instance else {
                    return None;
                };
                (value.base.name() == name).then_some(value.as_mut())
            })
    }

    pub fn set_parent_state_machine_instance(
        &mut self,
        instance: RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.parent_state_machine_instance = instance;
    }

    pub fn parent_state_machine_instance(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        self.parent_state_machine_instance.upgrade()
    }

    pub fn set_parent_nested_artboard(&mut self, artboard: CoreHandle) {
        self.parent_nested_artboard = Some(artboard);
    }

    pub fn parent_nested_artboard(&self) -> Option<CoreHandle> {
        self.parent_nested_artboard.clone()
    }

    pub fn add_nested_event_listener(&mut self, listener: RuntimeStateMachineInstanceWeakHandle) {
        if !self
            .nested_event_listeners
            .iter()
            .any(|candidate| candidate.ptr_eq(&listener))
        {
            self.nested_event_listeners.push(listener);
        }
    }

    pub fn remove_nested_event_listener(
        &mut self,
        listener: RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.nested_event_listeners
            .retain(|candidate| !candidate.ptr_eq(&listener));
    }

    pub fn set_nested_artboard(&mut self, artboard: CoreHandle) {
        self.nested_artboard = Some(artboard);
    }

    pub fn report_event(&mut self, event: CoreHandle, seconds_delay: f32) {
        self.reported_events.push(EventReport {
            event: Some(event),
            seconds_delay,
        });
    }

    fn report_listener_view_model(&mut self, listener: RuntimeListenerViewModelWeakHandle) {
        self.reported_listener_view_models.push(listener);
    }

    pub fn reported_event_count(&self) -> usize {
        self.events_applied_during_loop.len() + self.reported_events.len()
    }

    pub fn reported_event_at(&self, mut index: usize) -> EventReport {
        if index < self.events_applied_during_loop.len() {
            return self.events_applied_during_loop[index].clone();
        }
        index -= self.events_applied_during_loop.len();
        self.reported_events.get(index).cloned().unwrap_or_default()
    }

    pub fn notify(&mut self, events: &[EventReport], context: CoreHandle) {
        self.notify_event_listeners(events, Some(context));
        self.data_bind_container.update_data_binds(false);
    }

    pub fn notify_nested(&mut self, events: &[EventReport], context: Option<CoreHandle>) {
        self.notify_event_listeners(events, context);
        self.data_bind_container.update_data_binds(false);
    }

    fn notify_listener_view_models(&mut self, events: &[RuntimeListenerViewModelWeakHandle]) {
        for view_model in events {
            let Some(listener) = view_model.with_listener(|view_model| view_model.listener.clone())
            else {
                continue;
            };
            let invocation = self
                .runtime
                .borrow_mut()
                .listener_invocation_view_model(view_model.clone());
            let runtime = Rc::clone(&self.runtime);
            runtime
                .borrow_mut()
                .listener_perform_changes(&listener, self, &invocation);
        }
    }

    fn notify_event_listeners(&mut self, events: &[EventReport], source: Option<CoreHandle>) {
        if events.is_empty() {
            return;
        }
        let listener_count = self
            .machine
            .with_downcast::<StateMachine, _>(StateMachine::listener_count)
            .unwrap_or(0);
        for index in 0..listener_count {
            let Some(listener) = self
                .machine
                .with_downcast::<StateMachine, _>(|machine| machine.listener(index))
                .flatten()
            else {
                continue;
            };
            if !self.listener_has(&listener, ListenerType::Event) {
                continue;
            }
            let target = self.runtime.borrow_mut().artboard_resolve(
                &self.artboard_instance,
                self.runtime.borrow_mut().listener_target_id(&listener),
            );
            if source
                .as_ref()
                .is_some_and(|source| target.as_ref() != Some(source))
            {
                continue;
            }
            let source_artboard = if let Some(source) = source.as_ref() {
                self.runtime.borrow_mut().nested_artboard_instance(source)
            } else {
                self.artboard_instance.clone()
            };
            for report in events {
                if source.is_none() {
                    let resolved_target = self.runtime.borrow_mut().artboard_resolve(
                        &source_artboard,
                        self.runtime.borrow_mut().listener_target_id(&listener),
                    );
                    if resolved_target.as_ref().is_some_and(|resolved_target| {
                        !self
                            .runtime
                            .borrow_mut()
                            .component_is_artboard(resolved_target)
                            && !self.runtime.borrow_mut().object_is_event(resolved_target)
                    }) {
                        continue;
                    }
                }
                for event_id in self.runtime.borrow_mut().listener_event_ids(&listener) {
                    if self
                        .runtime
                        .borrow_mut()
                        .artboard_resolve(&source_artboard, event_id)
                        .as_ref()
                        == report.event.as_ref()
                    {
                        let Some(event) = report.event.as_ref() else {
                            continue;
                        };
                        let invocation = self
                            .runtime
                            .borrow_mut()
                            .listener_invocation_event(event, report.seconds_delay);
                        let runtime = Rc::clone(&self.runtime);
                        runtime
                            .borrow_mut()
                            .listener_perform_changes(&listener, self, &invocation);
                        break;
                    }
                }
            }
        }
        let listeners = self.nested_event_listeners.clone();
        if let Some(nested_artboard) = self.nested_artboard.clone() {
            for listener in listeners {
                listener
                    .with_instance_mut(|listener| listener.notify(events, nested_artboard.clone()));
            }
        }
        for report in events {
            let Some(event) = report.event.as_ref() else {
                continue;
            };
            if self.runtime.borrow_mut().event_is_audio(event) {
                self.runtime.borrow_mut().event_play_audio(event);
            }
        }
    }

    pub fn current_animation_count(&mut self) -> usize {
        self.layers
            .iter()
            .filter(|layer| {
                layer
                    .with_layer_mut(StateMachineLayerInstance::current_animation)
                    .is_some()
            })
            .count()
    }

    pub fn current_animation_by_index(
        &mut self,
        index: usize,
    ) -> Option<RuntimeStateInstanceHandle> {
        self.layers
            .iter()
            .filter_map(|layer| layer.with_layer_mut(StateMachineLayerInstance::current_animation))
            .nth(index)
    }

    pub fn state_changed_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| layer.with_layer(|layer| layer.state_machine_changed_on_advance))
            .count()
    }

    pub fn state_changed_by_index(&mut self, index: usize) -> Option<CoreHandle> {
        let mut count = 0;
        for layer in &self.layers {
            if layer.with_layer(|layer| layer.state_machine_changed_on_advance) {
                if count == index {
                    return layer.with_layer_mut(StateMachineLayerInstance::current_state);
                }
                count += 1;
            }
        }
        None
    }

    pub fn enable_pointer_events(&mut self, pointer_id: i32) {
        for component in &mut self.hit_components {
            component.enable_pointer_events(self.runtime.borrow_mut().as_mut(), pointer_id);
        }
    }

    pub fn disable_pointer_events(&mut self, pointer_id: i32) {
        for component in &mut self.hit_components {
            component.disable_pointer_events(self.runtime.borrow_mut().as_mut(), pointer_id);
        }
    }

    pub fn has_listeners(&self) -> bool {
        !self.hit_components.is_empty()
    }

    pub fn has_focus_nodes(&self) -> bool {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::has_focusable_content)
    }

    pub fn focus_next(&mut self) -> bool {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::focus_next)
    }

    pub fn focus_previous(&mut self) -> bool {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::focus_previous)
    }

    pub fn clear_focus(&mut self) {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::clear_focus);
    }

    pub fn submit_gamepads_from_buffer(&mut self, data: &[u8]) -> bool {
        let runtime = Rc::clone(&self.runtime);
        runtime.borrow_mut().gamepad_submit_buffer(self, data)
    }

    pub fn broadcast_gamepad_to_scripted_drawables(
        &mut self,
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        let runtime = Rc::clone(&self.runtime);
        runtime
            .borrow_mut()
            .gamepad_broadcast(self, invocation, already_dispatched)
    }

    pub fn duration_seconds(&self) -> f32 {
        -1.0
    }

    pub fn r#loop(&self) -> u8 {
        0
    }

    pub fn loop_value(&self) -> u8 {
        self.r#loop()
    }

    pub fn is_translucent(&self) -> bool {
        true
    }

    pub fn plays_audio(&self) -> bool {
        true
    }

    pub fn set_view_model_instance(&mut self, view_model_instance: CoreHandle) {
        if self.data_context_handle.is_none() {
            let data_context =
                RuntimeDataContextHandle::new(DataContext::new(Some(view_model_instance)));
            data_context.with_context_mut(|context| {
                context.add_state_machine_dependent_container(self.occurrence.clone());
            });
            self.data_context_handle = Some(data_context);
            return;
        }
        self.data_context_handle
            .as_ref()
            .unwrap()
            .with_context_mut(|context| {
                context.set_main_view_model_instance(Some(view_model_instance));
            });
    }

    pub fn set_global_view_model_instance(
        &mut self,
        name: &str,
        view_model_instance: impl Into<Option<CoreHandle>>,
    ) -> bool {
        let view_model_instance = view_model_instance.into();
        let Some(file) = self
            .artboard_instance
            .with_artboard(|artboard| artboard.base.file())
        else {
            return false;
        };
        let Some((slot_key, count, slot_view_model)) = file.with_file(|file| {
            let slot_key = file.view_model_id(name);
            (
                slot_key,
                file.view_model_count(),
                file.view_model(slot_key as usize),
            )
        }) else {
            return false;
        };
        if slot_key >= count as u32 {
            return false;
        }
        let Some(slot_view_model) = slot_view_model else {
            return false;
        };
        if slot_view_model
            .with_downcast::<ViewModel, _>(|view_model| {
                ViewModelType::from_u32(view_model.base.view_model_type())
            })
            .flatten()
            != Some(ViewModelType::Global)
        {
            return false;
        }
        if self.data_context_handle.is_none() {
            if view_model_instance.is_none() {
                return true;
            }
            let data_context = RuntimeDataContextHandle::new(DataContext::new(None));
            data_context.with_context_mut(|context| {
                context.add_state_machine_dependent_container(self.occurrence.clone());
            });
            self.data_context_handle = Some(data_context);
        }
        self.data_context_handle
            .as_ref()
            .unwrap()
            .with_context_mut(|context| {
                context.set_view_model_instance_for_slot(slot_key, view_model_instance);
            });
        true
    }

    pub fn bind(&mut self) {
        if self.data_context_handle.is_none() {
            let data_context = RuntimeDataContextHandle::new(DataContext::new(None));
            data_context.with_context_mut(|context| {
                context.add_state_machine_dependent_container(self.occurrence.clone());
            });
            self.data_context_handle = Some(data_context);
        }
        self.complete_view_model_instances();
        let data_context = self.data_context_handle.as_ref().unwrap().clone();
        self.artboard_instance.with_artboard_mut(|artboard| {
            artboard.base.internal_data_context(data_context.clone());
        });
        self.internal_data_context(data_context);
    }

    fn complete_view_model_instances(&mut self) {
        let Some(file) = self
            .artboard_instance
            .with_artboard(|artboard| artboard.base.file())
        else {
            return;
        };
        let data_context = self.data_context_handle.as_ref().unwrap().clone();
        if data_context
            .with_context(DataContext::main_view_model_instance)
            .is_none()
        {
            let artboard_source = self
                .artboard_instance
                .with_artboard(|artboard| artboard.base.artboard_source_handle())
                .flatten();
            let main = artboard_source
                .and_then(|artboard| {
                    file.with_file_mut(|file| {
                        file.create_default_view_model_instance_for_artboard(artboard)
                    })
                })
                .flatten();
            if let Some(main) = main {
                data_context.with_context_mut(|context| {
                    context.set_main_view_model_instance(Some(main));
                });
            }
        }
        let global_view_models = file
            .with_file(|file| file.global_view_models())
            .unwrap_or_default();
        for view_model in global_view_models {
            let Some(name) = view_model
                .with_downcast::<ViewModel, _>(|view_model| view_model.base.name().to_owned())
            else {
                continue;
            };
            let Some(slot_key) = file.with_file(|file| file.view_model_id(&name)) else {
                continue;
            };
            if data_context
                .with_context(|context| context.instance_for_slot(slot_key))
                .is_some()
            {
                continue;
            }
            let instance = file
                .with_file_mut(|file| file.create_default_view_model_instance(view_model))
                .flatten();
            if let Some(instance) = instance {
                data_context.with_context_mut(|context| {
                    context.set_view_model_instance_for_slot(slot_key, Some(instance));
                });
            }
        }
    }

    pub fn bind_view_model_instance(&mut self, view_model_instance: impl Into<Option<CoreHandle>>) {
        let Some(view_model_instance) = view_model_instance.into() else {
            self.clear_data_context();
            self.artboard_instance.with_artboard_mut(|artboard| {
                artboard.base.unbind();
            });
            return;
        };
        self.set_view_model_instance(view_model_instance);
        self.bind();
    }

    pub fn bind_view_model_instance_handle(&mut self, view_model_instance: CoreHandle) {
        self.bind_view_model_instance(view_model_instance);
    }

    pub fn global_view_model_instance(&self, name: &str) -> Option<CoreHandle> {
        let data_context = self.data_context_handle.as_ref()?;
        let file = self
            .artboard_instance
            .with_artboard(|artboard| artboard.base.file())?;
        let slot = file.with_file(|file| file.view_model_id(name))?;
        data_context.with_context(|context| context.instance_for_slot(slot))
    }

    pub fn bind_data_context(&mut self, data_context: RuntimeDataContextHandle) {
        self.clear_data_context();
        data_context.with_context_mut(|context| {
            context.add_state_machine_dependent_container(self.occurrence.clone());
        });
        self.artboard_instance.with_artboard_mut(|artboard| {
            artboard.base.clear_data_context();
            artboard.base.internal_data_context(data_context.clone());
        });
        self.internal_data_context(data_context);
    }

    pub fn bind_data_context_handle(&mut self, data_context: RuntimeDataContextHandle) {
        self.bind_data_context(data_context);
    }

    pub fn inherit_data_context(&mut self, data_context: RuntimeDataContextHandle) {
        data_context.with_context_mut(|context| {
            context.add_state_machine_dependent_container(self.occurrence.clone());
        });
        self.internal_data_context(data_context);
    }

    pub fn inherit_data_context_handle(&mut self, data_context: RuntimeDataContextHandle) {
        self.inherit_data_context(data_context);
    }

    pub fn set_data_context(&mut self, data_context: RuntimeDataContextHandle) {
        self.clear_data_context();
        self.internal_data_context(data_context);
    }

    pub fn set_data_context_handle(&mut self, data_context: RuntimeDataContextHandle) {
        self.set_data_context(data_context);
    }

    pub fn data_context(&self) -> Option<RuntimeDataContextHandle> {
        self.data_context_handle.clone()
    }

    pub fn data_context_handle(&self) -> Option<RuntimeDataContextHandle> {
        self.data_context_handle.clone()
    }

    pub fn internal_data_context_handle(&mut self, data_context: RuntimeDataContextHandle) {
        if self
            .data_context_handle
            .as_ref()
            .is_some_and(|current| current.ptr_eq(&data_context))
        {
            self.internal_data_context(data_context);
            return;
        }
        self.internal_data_context(data_context);
    }

    fn init_scripted_objects(&mut self) {
        for &object in self.scripted_objects_map.values() {
            self.runtime.borrow_mut().scripted_initialize(object);
            self.runtime.borrow_mut().scripted_hydrate_inputs(object);
        }
    }

    fn internal_data_context(&mut self, data_context: RuntimeDataContextHandle) {
        self.data_context_handle = Some(data_context.clone());
        self.data_bind_container
            .bind_data_binds_from_context(data_context.clone());
        for listener in &self.listener_view_models {
            listener.with_listener_mut(|listener| listener.bind_from_context(data_context.clone()));
        }
        for object in self.scripted_objects_map.values() {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.set_data_context(Some(data_context.clone()));
                }
            });
        }
        self.init_scripted_objects();
    }

    pub fn rebind(&mut self) {
        let Some(data_context) = self.data_context_handle.clone() else {
            return;
        };
        self.artboard_instance.with_artboard_mut(|artboard| {
            artboard.base.clear_data_context();
            artboard.base.internal_data_context(data_context.clone());
        });
        self.internal_data_context(data_context);
    }

    pub fn clear_data_context(&mut self) {
        if let Some(data_context) = self.data_context_handle.take() {
            data_context.with_context_mut(|context| {
                context.remove_state_machine_dependent_container(&self.occurrence);
            });
        }
        for listener in &self.listener_view_models {
            listener.with_listener_mut(ListenerViewModel::clear_data_context);
        }
    }

    pub fn relink_data_context(&mut self) {
        self.artboard_instance
            .with_artboard_mut(|artboard| artboard.base.relink_data_context());
    }

    pub fn rebuild_data_bind(&mut self, data_bind: CoreHandle) {
        if let Some(data_context) = self.data_context_handle.clone() {
            data_bind.with_mut(|data_bind| {
                if let Some(data_bind) = data_bind.as_data_bind_context_mut() {
                    data_bind.bind_from_context(Some(data_context));
                }
            });
        }
    }

    fn unbind(&mut self) {
        self.clear_data_context();
        self.data_bind_container.unbind_data_binds();
    }

    fn add_data_bind(&mut self, data_bind: CoreHandle) {
        self.data_bind_container.add_data_bind(data_bind);
    }

    pub fn add_dirty_data_bind(&mut self, data_bind: CoreHandle) {
        self.data_bind_container.add_dirty_data_bind(data_bind);
    }

    pub fn bindable_property_instance(&self, property: &CoreHandle) -> Option<CoreHandle> {
        self.bindable_property_instances.get(property).cloned()
    }

    pub fn bindable_data_bind_to_source(&self, property: &CoreHandle) -> Option<CoreHandle> {
        self.bindable_data_binds_to_source.get(property).cloned()
    }

    pub fn bindable_data_bind_to_target(&self, property: &CoreHandle) -> Option<CoreHandle> {
        self.bindable_data_binds_to_target.get(property).cloned()
    }

    pub fn find_transition_property_instance(
        &self,
        transition: &CoreHandle,
        property_key: u32,
    ) -> Option<CoreHandle> {
        self.transition_property_instances
            .get(transition)
            .and_then(|properties| properties.get(&property_key))
            .cloned()
    }

    fn keyframe_holder_property_key(keyframe_type: u16) -> u32 {
        match keyframe_type {
            KeyFrameDoubleBase::TYPE_KEY => {
                BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY as u32
            }
            KeyFrameColorBase::TYPE_KEY => {
                BindablePropertyColorBase::PROPERTY_VALUE_PROPERTY_KEY as u32
            }
            KeyFrameBoolBase::TYPE_KEY => {
                BindablePropertyBooleanBase::PROPERTY_VALUE_PROPERTY_KEY as u32
            }
            KeyFrameStringBase::TYPE_KEY => {
                BindablePropertyStringBase::PROPERTY_VALUE_PROPERTY_KEY as u32
            }
            _ => 0,
        }
    }

    pub fn build_state_keyframe_binds(&mut self, state_instance: &RuntimeStateInstanceHandle) {
        if self.artboard_instance.upgrade().is_none() {
            return;
        }
        let mut first_bind_by_target = HashMap::new();
        for data_bind in self
            .runtime
            .borrow_mut()
            .artboard_source_data_binds(&self.artboard_instance)
        {
            if let Some(target) = self.runtime.borrow_mut().data_bind_target(&data_bind)
                && self
                    .runtime
                    .borrow_mut()
                    .data_bind_is_keyframe_target(&data_bind)
            {
                first_bind_by_target.entry(target).or_insert(data_bind);
            }
        }
        if first_bind_by_target.is_empty() {
            return;
        }
        let runtime = Rc::clone(&self.runtime);
        runtime.borrow_mut().state_for_each_animation_instance(
            self,
            state_instance,
            &mut |runtime, machine, animation_instance| {
                for keyframe in runtime.animation_keyframes(animation_instance) {
                    let keyframe_type = runtime.keyframe_type(&keyframe);
                    let holder_property_key = Self::keyframe_holder_property_key(keyframe_type);
                    if holder_property_key == 0 {
                        continue;
                    }
                    let Some(source_bind) = first_bind_by_target.get(&keyframe) else {
                        continue;
                    };
                    let holder = runtime.make_keyframe_holder(keyframe_type);
                    runtime.add_keyframe_holder(animation_instance, &keyframe, holder.clone());
                    let clone = runtime.clone_data_bind(source_bind);
                    let file = runtime.data_bind_file(source_bind);
                    runtime.data_bind_set_file(&clone, file);
                    runtime.configure_data_bind_target(&clone, holder, holder_property_key);
                    runtime.data_bind_initialize(&clone);
                    if let Some(converter) = runtime.data_bind_converter(source_bind) {
                        let converter_clone = runtime.clone_data_converter(&converter);
                        runtime.data_bind_set_converter(&clone, converter_clone);
                    }
                    machine.add_data_bind(clone.clone());
                    machine
                        .state_keyframe_data_binds
                        .entry(state_instance.clone())
                        .or_default()
                        .push(clone);
                }
            },
        );
    }

    pub fn remove_state_keyframe_binds(&mut self, state_instance: &RuntimeStateInstanceHandle) {
        let Some(data_binds) = self.state_keyframe_data_binds.remove(state_instance) else {
            return;
        };
        for data_bind in data_binds {
            self.data_bind_container.remove_data_bind(data_bind.clone());
            data_bind.remove_occurrence();
        }
    }

    pub fn scripted_object(&self, source: &CoreHandle) -> Option<CoreHandle> {
        self.scripted_objects_map.get(source).cloned()
    }

    pub fn dispose(&mut self) {
        self.remove_event_listeners();
    }

    fn find_random_transition(
        &mut self,
        state_from: RuntimeStateInstanceHandle,
        layer_index: usize,
    ) -> Option<CoreHandle> {
        if layer_index >= self.layers.len() {
            return None;
        }
        let layer = self.layers[layer_index].clone();
        layer.with_layer_mut(|layer| layer.find_random_transition(self, state_from))
    }

    fn find_allowed_transition(
        &mut self,
        state_from: RuntimeStateInstanceHandle,
        layer_index: usize,
    ) -> Option<CoreHandle> {
        if layer_index >= self.layers.len() {
            return None;
        }
        let layer = self.layers[layer_index].clone();
        layer.with_layer_mut(|layer| layer.find_allowed_transition(self, state_from))
    }

    #[cfg(test)]
    pub fn hit_components_count(&self) -> usize {
        self.hit_components.len()
    }

    #[cfg(test)]
    pub fn hit_component(&self, index: usize) -> Option<&dyn HitComponent> {
        self.hit_components.get(index).map(Box::as_ref)
    }

    #[cfg(test)]
    pub fn layer_state(&mut self, index: usize) -> Option<CoreHandle> {
        self.layers
            .get(index)
            .map(|layer| layer.with_layer_mut(StateMachineLayerInstance::current_state))
            .flatten()
    }

    fn remove_event_listeners(&mut self) {
        for nested in self
            .runtime
            .borrow_mut()
            .artboard_nested_artboards(&self.artboard_instance)
        {
            for animation in self.runtime.borrow_mut().nested_animations(&nested) {
                self.runtime
                    .borrow_mut()
                    .nested_remove_event_listener(&animation, self.occurrence.clone());
            }
        }
    }

    #[cfg(feature = "tools")]
    pub fn on_input_changed(
        &mut self,
        callback: Option<Box<dyn FnMut(RuntimeStateMachineInstanceWeakHandle, u64)>>,
    ) {
        self.input_changed_callback = callback;
    }

    #[cfg(feature = "tools")]
    pub fn on_data_bind_changed(&mut self, callback: fn()) {
        for data_bind in self.data_bind_container.data_binds() {
            data_bind.with_downcast_mut::<crate::mechanical_port::source::data_bind::data_bind::DataBind, _>(|data_bind| {
                data_bind.set_changed_callback(callback);
            });
        }
    }
}

#[cfg(feature = "tools")]
impl StateMachineInstance {
    pub(crate) fn input_changed(&mut self, index: u64) {
        if let Some(callback) = self.input_changed_callback.as_mut() {
            callback(self.occurrence.clone(), index);
        }
    }
}

impl Drop for StateMachineInstance {
    fn drop(&mut self) {
        if self.external_focus_manager.is_none() {
            let _ = self
                .artboard_instance
                .with_artboard_mut(|artboard| artboard.cleanup_focus_tree());
        }
        if self.external_semantic_manager.is_none() && self.semantic_manager.is_some() {
            let _ = self
                .artboard_instance
                .with_artboard_mut(|artboard| artboard.cleanup_semantic_tree());
        }
        self.embedder_gamepads.clear();
        self.unbind();
        self.input_instances.clear();
        self.listener_groups.clear();
        let data_binds = self.data_bind_container.data_binds();
        self.data_bind_container.delete_data_binds();
        for data_bind in data_binds {
            data_bind.remove_occurrence();
        }
        self.state_keyframe_data_binds.clear();
        self.layers.clear();
        for (_, property) in self.bindable_property_instances.drain() {
            self.runtime.borrow_mut().delete_owned_object(property);
        }
        for (_, properties) in self.transition_property_instances.drain() {
            for (_, property) in properties {
                self.runtime.borrow_mut().delete_owned_object(property);
            }
        }
        self.listener_view_models.clear();
        for (_, object) in self.scripted_objects_map.drain() {
            self.runtime.borrow_mut().scripted_delete(object);
        }
    }
}
