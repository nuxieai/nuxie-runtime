use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::{
        animation_reset::AnimationReset,
        animation_reset_factory::AnimationResetFactory,
        animation_state::AnimationState,
        blend_animation_1d::BlendAnimation1D,
        blend_animation_direct::BlendAnimationDirect,
        blend_state_instance::BlendAnimationDefinition,
        focus_listener_group::RuntimeFocusListenerGroupHandle,
        gamepad_listener_group::RuntimeGamepadListenerGroupHandle,
        keyboard_listener_group::RuntimeKeyboardListenerGroupHandle,
        layer_state::LayerState,
        linear_animation::LinearAnimation,
        linear_animation_instance::LinearAnimationInstance,
        listener_invocation::{ListenerInvocation, ListenerInvocationKind},
        listener_types::listener_input_type_semantic::ListenerInputTypeSemantic,
        listener_types::listener_input_type_viewmodel::ListenerInputTypeViewModel,
        nested_bool::NestedBool,
        nested_number::NestedNumber,
        nested_trigger::NestedTrigger,
        semantic_listener_group::{RuntimeSemanticListenerGroupHandle, SemanticActionType},
        state_instance::RuntimeStateInstanceHandle,
        state_machine::StateMachine,
        state_machine_bool::StateMachineBool,
        state_machine_input_instance::{
            InputInstanceNotifier, SMIBool, SMIInput, SMINumber, SMITrigger,
        },
        state_machine_layer::StateMachineLayer,
        state_machine_listener::StateMachineListener,
        state_machine_listener_single::StateMachineListenerSingle,
        state_machine_number::StateMachineNumber,
        state_machine_trigger::StateMachineTrigger,
        state_transition::{AllowTransition, TransitionRuntime},
    },
    artboard::{Artboard, RuntimeArtboardInstanceWeakHandle},
    artboard_component_list::ArtboardComponentList,
    audio_event::AudioEvent,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    data_bind::{
        bindable_property_artboard::BindablePropertyArtboard,
        bindable_property_asset::BindablePropertyAsset,
        bindable_property_boolean::BindablePropertyBoolean,
        bindable_property_color::BindablePropertyColor,
        bindable_property_enum::BindablePropertyEnum,
        bindable_property_integer::BindablePropertyInteger,
        bindable_property_number::BindablePropertyNumber,
        bindable_property_string::BindablePropertyString,
        bindable_property_trigger::BindablePropertyTrigger,
        bindable_property_viewmodel::BindablePropertyViewModel,
        data_bind::DataBind,
        data_bind_container::DataBindContainer,
        data_context::{DataContext, RuntimeDataContextHandle},
    },
    dirtyable::Dirtyable,
    drawable::RuntimeDrawableOccurrence,
    file::DETERMINISTIC_MODE,
    focus_data::FocusData,
    generated::{
        animation::{
            keyframe_base::KeyFrameBase, keyframe_bool_base::KeyFrameBoolBase,
            keyframe_color_base::KeyFrameColorBase, keyframe_double_base::KeyFrameDoubleBase,
            keyframe_string_base::KeyFrameStringBase, state_transition_base::StateTransitionBase,
        },
        core_registry::CoreRegistry,
        data_bind::{
            bindable_property_base::BindablePropertyBase,
            bindable_property_boolean_base::BindablePropertyBooleanBase,
            bindable_property_color_base::BindablePropertyColorBase,
            bindable_property_number_base::BindablePropertyNumberBase,
            bindable_property_string_base::BindablePropertyStringBase,
        },
        event_base::EventBase,
    },
    hit_result::HitResult,
    input::{
        focus_manager::{FocusManager, RuntimeFocusManagerHandle},
        focus_node::FocusNodeRef,
        gamepad_batch::{GamepadBatchState, GamepadDispatcher, GamepadInvocation},
    },
    listener_group::{ListenerGroup, ListenerGroupProvider, RuntimeListenerGroupHandle},
    listener_type::ListenerType,
    math::{random::RandomProvider, vec2d::Vec2D},
    process_event_result::ProcessEventResult,
    scripted::scripted_object::{ScriptUpdateRequestHost, ScriptedObject},
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
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
    sync::atomic::Ordering,
};

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

pub struct StateMachineLayerInstance {
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

    fn init(
        &mut self,
        state_machine_instance: &mut StateMachineInstance,
        layer: CoreHandle,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) {
        self.artboard_instance = artboard.clone();
        let deterministic = DETERMINISTIC_MODE.load(Ordering::Relaxed);
        let seed = RandomProvider::layer_seed(deterministic);
        RandomProvider::seed(seed);
        debug_assert!(self.layer.is_none());
        let any_state = layer
            .with_downcast::<StateMachineLayer, _>(StateMachineLayer::any_state)
            .flatten()
            .expect("an imported state-machine layer has AnyState");
        let any_state_instance = Self::make_state_instance(any_state, &artboard);
        state_machine_instance.build_state_keyframe_binds(&any_state_instance);
        self.any_state_instance = Some(any_state_instance);
        let entry = layer
            .with_downcast::<StateMachineLayer, _>(StateMachineLayer::entry_state)
            .flatten()
            .expect("an imported state-machine layer has EntryState");
        self.layer = Some(layer);
        self.change_state(state_machine_instance, Some(entry));
    }

    fn make_state_instance(
        state: CoreHandle,
        artboard: &RuntimeArtboardInstanceWeakHandle,
    ) -> RuntimeStateInstanceHandle {
        let behavior = state
            .with(|state| state.layer_state_make_instance(artboard.clone()))
            .flatten()
            .expect("an imported LayerState must provide makeInstance");
        RuntimeStateInstanceHandle::new(state, behavior)
    }

    fn layer_component_events(component: &CoreHandle) -> Vec<CoreHandle> {
        component
            .with(|component| component.state_machine_layer_component_events())
            .flatten()
            .expect("an authored state/transition must expose fire events")
    }

    fn layer_component_listener_actions(component: &CoreHandle) -> Vec<CoreHandle> {
        component
            .with(|component| component.state_machine_layer_component_listener_actions())
            .flatten()
            .expect("an authored state/transition must expose listener actions")
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
            .layer
            .as_ref()
            .expect("initialized layer")
            .with_downcast::<StateMachineLayer, _>(StateMachineLayer::entry_state)
            .flatten()
            .expect("an imported state-machine layer has EntryState");
        self.change_state(machine, Some(entry));
    }

    fn resolved_duration(&mut self) -> u32 {
        if let Some(property) = self.transition_duration_property.clone() {
            return self
                .transition_duration_property_value(&property)
                .round()
                .max(0.0) as u32;
        }
        self.transition
            .as_ref()
            .and_then(|transition| {
                transition
                    .with(|transition| transition.state_transition_duration())
                    .flatten()
            })
            .unwrap_or(0)
    }

    fn transition_duration_property_value(&self, property: &CoreHandle) -> f32 {
        property
            .with_downcast::<BindablePropertyNumber, _>(|property| property.base.property_value())
            .unwrap_or(0.0)
    }

    fn resolved_mix_time(&mut self) -> f32 {
        let duration = self.resolved_duration();
        if duration == 0 {
            return 0.0;
        }
        let Some(transition) = self.transition.clone() else {
            return 0.0;
        };
        let duration_is_percentage = transition
            .with(|transition| transition.state_transition_duration_is_percentage())
            .flatten()
            .expect("an authored StateTransition must expose percentage duration");
        if duration_is_percentage {
            let animation = self.state_from.as_ref().and_then(|state| {
                state
                    .definition()
                    .with_downcast::<AnimationState, _>(AnimationState::animation)
                    .flatten()
            });
            let animation_duration = animation
                .as_ref()
                .and_then(|animation| {
                    animation.with_downcast::<LinearAnimation, _>(LinearAnimation::duration_seconds)
                })
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
                let events = Self::layer_component_events(&transition);
                self.fire_events(machine, 1, &events);
                let actions = Self::layer_component_listener_actions(&transition);
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
            current.with_state_mut(|state| state.advance(seconds, machine));
        }
        self.update_mix(machine, seconds);
        if let Some(from) = self.state_from.clone()
            && self.mix < 1.0
            && !self.hold_animation_from
        {
            from.with_state_mut(|state| state.advance(seconds, machine));
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
            current.with_state_mut(|state| state.clear_spilled_time());
        }
        changed
            || self.mix != 1.0
            || self.waiting_for_exit
            || self
                .current_state
                .as_ref()
                .is_some_and(|current| current.with_state(|state| state.keep_going()))
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
                .transition
                .as_ref()
                .expect("active transition")
                .with(|transition| transition.state_transition_enable_early_exit())
                .flatten()
                .expect("an authored StateTransition must expose early-exit enablement")
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
            let scheduled = event
                .with(|event| event.state_machine_fire_action_occurs())
                .flatten()
                .expect("an authored fire action must expose occurrence");
            if scheduled.0 == occurrence as i32 {
                let performed = event
                    .with_mut(|event| event.state_machine_fire_action_perform(machine))
                    .unwrap_or(false);
                assert!(performed, "an authored fire action must expose perform");
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
            let scheduled = crate::mechanical_port::source::animation::state_machine_fire_action::StateMachineFireOccurance(occurrence as i32);
            let matches = action
                .with(|action| action.listener_action_matches(scheduled))
                .flatten()
                .expect("an authored listener action must expose scheduled occurrence");
            if matches {
                let invocation = ListenerInvocation::none();
                let performed = action
                    .with_mut(|action| action.listener_action_perform(machine, &invocation))
                    .unwrap_or(false);
                assert!(performed, "an authored listener action must expose perform");
            }
        }
    }

    fn can_change_state(&mut self, state_to: &Option<CoreHandle>) -> bool {
        self.current_state
            .as_ref()
            .map(RuntimeStateInstanceHandle::definition)
            .as_ref()
            != state_to.as_ref()
    }

    fn change_state(&mut self, machine: &mut StateMachineInstance, state_to: Option<CoreHandle>) {
        if self
            .current_state
            .as_ref()
            .map(RuntimeStateInstanceHandle::definition)
            .as_ref()
            == state_to.as_ref()
        {
            return;
        }
        if let Some(current) = self.current_state.clone() {
            let state = current.definition();
            let events = Self::layer_component_events(&state);
            self.fire_events(machine, 1, &events);
            let actions = Self::layer_component_listener_actions(&state);
            self.perform_listener_actions(machine, 1, &actions);
        }
        let Some(state_to) = state_to else {
            self.current_state = None;
            return;
        };
        let current = Self::make_state_instance(state_to, &self.artboard_instance);
        machine.build_state_keyframe_binds(&current);
        let state = current.definition();
        let events = Self::layer_component_events(&state);
        self.fire_events(machine, 0, &events);
        let actions = Self::layer_component_listener_actions(&state);
        self.perform_listener_actions(machine, 0, &actions);
        self.current_state = Some(current);
    }

    fn find_random_transition(
        &mut self,
        machine: &mut StateMachineInstance,
        from_instance: RuntimeStateInstanceHandle,
    ) -> Option<CoreHandle> {
        let state = from_instance.definition();
        let mut total_weight = 0;
        let transition_count = state
            .with(|state| state.layer_state_transition_count())
            .flatten()
            .expect("an authored LayerState must expose transition count");
        for index in 0..transition_count {
            let Some(transition) = state
                .with(|state| state.layer_state_transition(index))
                .flatten()
            else {
                continue;
            };
            let state_to = transition
                .with(|transition| transition.state_transition_state_to())
                .flatten();
            if self.can_change_state(&state_to) {
                let allowed = transition
                    .with_mut(|transition| {
                        transition.state_transition_allowed(
                            &from_instance,
                            machine,
                            self.occurrence.clone(),
                        )
                    })
                    .flatten()
                    .unwrap_or(AllowTransition::No);
                if allowed == AllowTransition::Yes {
                    let weight = transition
                        .with(|transition| transition.state_transition_random_weight())
                        .flatten()
                        .expect("an authored StateTransition must expose random weight");
                    transition.with_mut(|transition| {
                        transition.state_transition_set_evaluated_random_weight(weight);
                    });
                    total_weight += weight;
                } else {
                    transition.with_mut(|transition| {
                        transition.state_transition_set_evaluated_random_weight(0);
                    });
                    if allowed == AllowTransition::WaitingForExit {
                        self.waiting_for_exit = true;
                    }
                }
            } else {
                transition.with_mut(|transition| {
                    transition.state_transition_set_evaluated_random_weight(0);
                });
            }
        }
        if total_weight == 0 {
            return None;
        }
        let random_weight = RandomProvider::generate_random_float() as f64 * total_weight as f64;
        let mut current_weight = 0.0;
        for index in 0..transition_count {
            let Some(transition) = state
                .with(|state| state.layer_state_transition(index))
                .flatten()
            else {
                continue;
            };
            let weight = transition
                .with(|transition| transition.state_transition_evaluated_random_weight())
                .flatten()
                .expect("an authored StateTransition must expose evaluated random weight")
                as f64;
            if current_weight + weight > random_weight {
                transition.with_mut(|transition| {
                    transition.state_transition_use_layer(machine, self.occurrence.clone());
                });
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
        let state = from_instance.definition();
        let flags = state
            .with(|state| state.layer_state_flags())
            .flatten()
            .expect("an imported LayerState must expose flags");
        if flags & 1 != 0 {
            return self.find_random_transition(machine, from_instance);
        }
        let transition_count = state
            .with(|state| state.layer_state_transition_count())
            .flatten()
            .expect("an authored LayerState must expose transition count");
        for index in 0..transition_count {
            let Some(transition) = state
                .with(|state| state.layer_state_transition(index))
                .flatten()
            else {
                continue;
            };
            let state_to = transition
                .with(|transition| transition.state_transition_state_to())
                .flatten();
            if !self.can_change_state(&state_to) {
                continue;
            }
            let allowed = transition
                .with_mut(|transition| {
                    transition.state_transition_allowed(
                        &from_instance,
                        machine,
                        self.occurrence.clone(),
                    )
                })
                .flatten()
                .unwrap_or(AllowTransition::No);
            if allowed == AllowTransition::Yes {
                let weight = transition
                    .with(|transition| transition.state_transition_random_weight())
                    .flatten()
                    .expect("an authored StateTransition must expose random weight");
                transition.with_mut(|transition| {
                    transition.state_transition_set_evaluated_random_weight(weight);
                });
                transition.with_mut(|transition| {
                    transition.state_transition_use_layer(machine, self.occurrence.clone());
                });
                return Some(transition);
            }
            transition.with_mut(|transition| {
                transition.state_transition_set_evaluated_random_weight(0);
            });
            if allowed == AllowTransition::WaitingForExit {
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
        let state_to = transition
            .with(|transition| transition.state_transition_state_to())
            .flatten();
        self.change_state(machine, state_to);
        self.state_machine_changed_on_advance = true;
        self.transition = Some(transition.clone());
        self.transition_duration_property = machine.find_transition_property_instance(
            &transition,
            StateTransitionBase::DURATION_PROPERTY_KEY as u32,
        );
        let events = Self::layer_component_events(&transition);
        self.fire_events(machine, 0, &events);
        let actions = Self::layer_component_listener_actions(&transition);
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
        if let Some(out_state) = self.state_from.clone() {
            let hold_animation = out_state
                .definition()
                .with_downcast::<AnimationState, _>(AnimationState::animation)
                .flatten();
            let use_exit = transition
                .with(|transition| transition.state_transition_enable_exit_time())
                .flatten()
                .expect("an authored StateTransition must expose exit-time enablement")
                && hold_animation.is_some();
            let pause_on_exit = transition
                .with(|transition| transition.state_transition_pause_on_exit())
                .flatten()
                .expect("an authored StateTransition must expose pause-on-exit");
            let applied = if pause_on_exit && use_exit {
                let flags = transition
                    .with(|transition| transition.state_transition_flags())
                    .flatten()
                    .expect("an authored StateTransition must expose flags");
                let authored_exit_time = transition
                    .with(|transition| transition.state_transition_exit_time())
                    .flatten()
                    .expect("an authored StateTransition must expose exit time");
                let exit_time = out_state
                    .first_animation(|animation| {
                        if flags & 32 != 0 {
                            animation.start_time()
                                + authored_exit_time as f32 / 100.0 * animation.duration_seconds()
                        } else {
                            authored_exit_time as f32 / 1000.0
                        }
                    })
                    .unwrap_or(0.0);
                out_state.first_animation(|animation| animation.set_time(exit_time));
                true
            } else {
                use_exit
            };
            if applied {
                self.hold_animation = hold_animation;
                self.hold_time = out_state
                    .first_animation(LinearAnimationInstance::time)
                    .unwrap_or(0.0);
            }
        }
        self.mix_from = self.mix;
        if self.mix != 0.0 {
            self.hold_animation_from = transition
                .with(|transition| transition.state_transition_pause_on_exit())
                .flatten()
                .expect("an authored StateTransition must expose pause-on-exit");
        }
        if let Some(current) = self.current_state.clone() {
            let advance_time = self
                .state_from
                .as_ref()
                .and_then(|from| from.first_animation(LinearAnimationInstance::spilled_time))
                .unwrap_or(0.0);
            current.with_state_mut(|state| state.advance(advance_time, machine));
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
            self.artboard_instance.with_artboard_mut(|artboard| {
                hold_animation.with_downcast_mut::<LinearAnimation, _>(|animation| {
                    animation.apply(artboard, self.hold_time, self.mix_from, None)
                });
            });
        }
        let interpolator = self.transition.as_ref().and_then(|transition| {
            transition
                .with(|transition| transition.state_transition_interpolator())
                .flatten()
        });
        if let Some(state_from) = self.state_from.clone().filter(|_| self.mix < 1.0) {
            let mix = interpolator
                .as_ref()
                .and_then(|interpolator| {
                    interpolator
                        .with_mut(|interpolator| {
                            interpolator.keyframe_interpolator_transform(self.mix_from)
                        })
                        .flatten()
                })
                .unwrap_or(self.mix_from);
            state_from.with_state_mut(|state| state.apply(&self.artboard_instance, mix));
        }
        if let Some(current_state) = self.current_state.clone() {
            let mix = interpolator
                .as_ref()
                .and_then(|interpolator| {
                    interpolator
                        .with_mut(|interpolator| {
                            interpolator.keyframe_interpolator_transform(self.mix)
                        })
                        .flatten()
                })
                .unwrap_or(self.mix);
            current_state.with_state_mut(|state| state.apply(&self.artboard_instance, mix));
        }
    }

    fn current_state(&mut self) -> Option<CoreHandle> {
        let current = self.current_state.clone()?;
        Some(current.definition())
    }

    fn current_animation(&mut self) -> Option<RuntimeStateInstanceHandle> {
        self.current_state.clone().filter(|current| {
            current
                .definition()
                .with_downcast::<AnimationState, _>(AnimationState::animation)
                .flatten()
                .is_some()
        })
    }
}

pub(crate) struct DirectTransitionRuntime {
    exit_blend_animation: Option<CoreHandle>,
}

impl DirectTransitionRuntime {
    pub(crate) fn plain() -> Self {
        Self {
            exit_blend_animation: None,
        }
    }

    pub(crate) fn blend(exit_blend_animation: Option<CoreHandle>) -> Self {
        Self {
            exit_blend_animation,
        }
    }

    fn selected_blend_animation(&self) -> Option<CoreHandle> {
        let blend = self.exit_blend_animation.as_ref()?;
        blend
            .with_downcast::<BlendAnimation1D, _>(BlendAnimationDefinition::animation)
            .or_else(|| {
                blend.with_downcast::<BlendAnimationDirect, _>(BlendAnimationDefinition::animation)
            })
            .flatten()
    }
}

impl TransitionRuntime for DirectTransitionRuntime {
    fn evaluate_condition(
        &self,
        condition: &CoreHandle,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        condition
            .with_mut(|condition| condition.transition_condition_allowed(machine, layer))
            .flatten()
            .expect("an authored TransitionCondition must expose allowed")
    }

    fn use_condition_in_layer(
        &self,
        condition: &CoreHandle,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) {
        let used = condition
            .with_mut(|condition| condition.transition_condition_use_layer(machine, layer))
            .unwrap_or(false);
        assert!(
            used,
            "an authored TransitionCondition must expose useInLayer"
        );
    }

    fn animation_duration(&self, state: &LayerState) -> Option<f32> {
        let animation = if self.exit_blend_animation.is_some() {
            self.selected_blend_animation()?
        } else {
            let state = state.base.base.base.base.handle()?;
            state
                .with_downcast::<AnimationState, _>(AnimationState::animation)
                .flatten()?
        };
        animation.with_downcast::<LinearAnimation, _>(LinearAnimation::duration_seconds)
    }

    fn exit_animation(&self, state: &LayerState) -> Option<(f32, f32)> {
        let animation = if self.exit_blend_animation.is_some() {
            self.selected_blend_animation()?
        } else {
            let state = state.base.base.base.base.handle()?;
            state
                .with_downcast::<AnimationState, _>(AnimationState::animation)
                .flatten()?
        };
        animation.with_downcast::<LinearAnimation, _>(|animation| {
            (animation.start_time(), animation.duration_seconds())
        })
    }

    fn exit_instance_times(
        &self,
        from: &RuntimeStateInstanceHandle,
    ) -> Option<(f32, f32, f32, i32)> {
        let use_animation = |animation: &mut LinearAnimationInstance| {
            (
                animation.last_total_time(),
                animation.total_time(),
                animation.duration_seconds(),
                animation.loop_value(),
            )
        };
        if let Some(blend) = self.exit_blend_animation.as_ref() {
            from.animation_for_blend(blend, use_animation)
        } else if from
            .definition()
            .with_downcast::<AnimationState, _>(|_| ())
            .is_some()
        {
            from.first_animation(use_animation)
        } else {
            None
        }
    }

    fn set_exit_instance_time(&self, from: &RuntimeStateInstanceHandle, time: f32) {
        if let Some(blend) = self.exit_blend_animation.as_ref() {
            from.animation_for_blend(blend, |animation| animation.set_time(time));
        } else if from
            .definition()
            .with_downcast::<AnimationState, _>(|_| ())
            .is_some()
        {
            from.first_animation(|animation| animation.set_time(time));
        }
    }
}

pub trait HitComponent {
    fn component(&self) -> RuntimeDrawableOccurrence;
    fn as_hit_drawable_mut(&mut self) -> Option<&mut HitDrawable> {
        None
    }
    #[cfg(test)]
    fn early_out_count(&self) -> i32 {
        0
    }
    fn process_event(
        &mut self,
        machine: &mut StateMachineInstance,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult;
    fn process_gamepad_invocation(
        &mut self,
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult;
    fn prepare_event(&mut self, position: Vec2D, hit_type: ListenerType, pointer_id: i32);
    fn hit_test(&self, position: Vec2D) -> bool;
    fn enable_pointer_events(&mut self, _pointer_id: i32) {}
    fn disable_pointer_events(&mut self, _pointer_id: i32) {}
}

fn component_is_collapsed(component: &CoreHandle) -> bool {
    component
        .with(|component| {
            component
                .as_component()
                .map(|component| component.is_collapsed())
        })
        .flatten()
        .expect("a hit target is a live Component")
}

fn nested_is_paused(component: &CoreHandle) -> bool {
    component
        .with(|component| {
            component
                .as_nested_artboard()
                .map(|nested| nested.base.is_paused())
        })
        .flatten()
        .expect("a nested hit target is a NestedArtboard")
}

fn nested_world_to_local(component: &CoreHandle, position: Vec2D) -> Option<Vec2D> {
    component
        .with(|component| {
            let nested = component.as_nested_artboard()?;
            let mut local = Vec2D::new(0.0, 0.0);
            nested.world_to_local(position, &mut local).then_some(local)
        })
        .flatten()
}

fn nested_animations(component: &CoreHandle) -> Vec<CoreHandle> {
    component
        .with(|component| {
            component
                .as_nested_artboard()
                .map(|nested| nested.nested_animations().to_vec())
        })
        .flatten()
        .expect("a nested hit target is a NestedArtboard")
}

fn nested_state_machine(animation: &CoreHandle) -> Option<RuntimeStateMachineInstanceHandle> {
    animation.with_downcast::<crate::mechanical_port::source::animation::nested_state_machine::NestedStateMachine, _>(|nested| nested.state_machine_instance())
        .flatten()
}

fn component_list_indices(component: &CoreHandle) -> Vec<i32> {
    component
        .with_downcast_mut::<ArtboardComponentList, _>(|list| list.ordered_list_indices().to_vec())
        .expect("a component-list hit target is an ArtboardComponentList")
}

fn component_list_world_to_local(
    component: &CoreHandle,
    position: Vec2D,
    index: i32,
) -> Option<Vec2D> {
    component
        .with_downcast_mut::<ArtboardComponentList, _>(|list| {
            let mut local = Vec2D::new(0.0, 0.0);
            list.world_to_local(position, &mut local, index)
                .then_some(local)
        })
        .flatten()
}

fn component_list_state_machine(
    component: &CoreHandle,
    index: i32,
) -> Option<RuntimeStateMachineInstanceHandle> {
    component
        .with_downcast::<ArtboardComponentList, _>(|list| list.state_machine_instance(index))
        .flatten()
}

pub struct HitDrawable {
    component: RuntimeDrawableOccurrence,
    drawable: RuntimeDrawableOccurrence,
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
        drawable: RuntimeDrawableOccurrence,
        component: RuntimeDrawableOccurrence,
        is_opaque: bool,
        hit_path: bool,
        hit_clip: bool,
    ) -> Self {
        let can_early_out = !drawable.is_target_opaque();
        Self {
            component,
            drawable,
            hit_radius: 2.0,
            is_hovered: false,
            can_early_out,
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
            .with_component(|component| {
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
    fn component(&self) -> RuntimeDrawableOccurrence {
        self.component.clone()
    }

    fn as_hit_drawable_mut(&mut self) -> Option<&mut HitDrawable> {
        Some(self)
    }

    #[cfg(test)]
    fn early_out_count(&self) -> i32 {
        self.early_out_count
    }

    fn hit_test(&self, position: Vec2D) -> bool {
        self.component
            .hit_test_point(&position, self.hit_path, self.hit_clip)
    }

    fn prepare_event(&mut self, position: Vec2D, hit_type: ListenerType, pointer_id: i32) {
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
        self.is_hovered = hit_type != ListenerType::Exit && self.hit_test(position);
        if self.is_hovered {
            for listener in &self.listeners {
                listener.with_group_mut(|listener| listener.hover(pointer_id));
            }
        }
    }

    fn process_gamepad_invocation(
        &mut self,
        _invocation: &ListenerInvocation,
        _already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        HitResult::None
    }

    fn process_event(
        &mut self,
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
                .with_component_mut(|component| {
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
        } else if self.is_opaque || self.drawable.is_target_opaque() || blocking {
            HitResult::HitOpaque
        } else {
            HitResult::Hit
        }
    }

    fn enable_pointer_events(&mut self, pointer_id: i32) {
        for listener in &self.listeners {
            listener.with_group_mut(|listener| listener.enable(pointer_id));
        }
    }

    fn disable_pointer_events(&mut self, pointer_id: i32) {
        for listener in &self.listeners {
            listener.with_group_mut(|listener| listener.disable(pointer_id));
        }
    }
}

struct HitNestedArtboard {
    component: CoreHandle,
}

impl HitComponent for HitNestedArtboard {
    fn component(&self) -> RuntimeDrawableOccurrence {
        RuntimeDrawableOccurrence::Authored(self.component.clone())
    }

    fn hit_test(&self, position: Vec2D) -> bool {
        if component_is_collapsed(&self.component) || nested_is_paused(&self.component) {
            return false;
        }
        let Some(local) = nested_world_to_local(&self.component, position) else {
            return false;
        };
        nested_animations(&self.component)
            .into_iter()
            .filter(|animation| animation.is_type_of(crate::mechanical_port::source::generated::animation::nested_state_machine_base::NestedStateMachineBase::TYPE_KEY))
            .any(|animation| {
                nested_state_machine(&animation)
                    .is_some_and(|instance| instance.with_instance(|nested| nested.hit_test(local)))
            })
    }

    fn process_event(
        &mut self,
        _machine: &mut StateMachineInstance,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult {
        if component_is_collapsed(&self.component) || nested_is_paused(&self.component) {
            return HitResult::None;
        }
        let Some(local) = nested_world_to_local(&self.component, position) else {
            return HitResult::None;
        };
        let mut result = HitResult::None;
        for animation in nested_animations(&self.component) {
            if !animation.is_type_of(crate::mechanical_port::source::generated::animation::nested_state_machine_base::NestedStateMachineBase::TYPE_KEY) {
                continue;
            }
            let Some(instance) = nested_state_machine(&animation) else {
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
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        for animation in nested_animations(&self.component) {
            if animation.is_type_of(crate::mechanical_port::source::generated::animation::nested_state_machine_base::NestedStateMachineBase::TYPE_KEY) {
                if let Some(instance) = nested_state_machine(&animation) {
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

    fn prepare_event(&mut self, _position: Vec2D, _hit_type: ListenerType, _pointer_id: i32) {}
}

struct HitComponentList {
    component: CoreHandle,
}

impl HitComponent for HitComponentList {
    fn component(&self) -> RuntimeDrawableOccurrence {
        RuntimeDrawableOccurrence::Authored(self.component.clone())
    }

    fn hit_test(&self, position: Vec2D) -> bool {
        if component_is_collapsed(&self.component) {
            return false;
        }
        for index in component_list_indices(&self.component).into_iter().rev() {
            let Some(local) = component_list_world_to_local(&self.component, position, index)
            else {
                continue;
            };
            if component_list_state_machine(&self.component, index)
                .is_some_and(|machine| machine.with_instance(|nested| nested.hit_test(local)))
            {
                return true;
            }
        }
        false
    }

    fn process_event(
        &mut self,
        _machine: &mut StateMachineInstance,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult {
        if component_is_collapsed(&self.component) {
            return HitResult::None;
        }
        let mut result = HitResult::None;
        let mut running_can_hit = can_hit;
        for index in component_list_indices(&self.component).into_iter().rev() {
            let Some(local) = component_list_world_to_local(&self.component, position, index)
            else {
                continue;
            };
            let Some(machine) = component_list_state_machine(&self.component, index) else {
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
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        if component_is_collapsed(&self.component) {
            return HitResult::None;
        }
        let mut result = HitResult::None;
        let mut running_can_hit = true;
        for index in component_list_indices(&self.component).into_iter().rev() {
            let Some(machine) = component_list_state_machine(&self.component, index) else {
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

    fn prepare_event(&mut self, _position: Vec2D, _hit_type: ListenerType, _pointer_id: i32) {}
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
    fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }

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
    embedder_gamepads: GamepadBatchState,
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

impl GamepadDispatcher for StateMachineInstance {
    fn dispatch(&mut self, invocation: GamepadInvocation) {
        let invocation = match invocation {
            GamepadInvocation::Connected(snapshot) => ListenerInvocation::gamepad_connected(&snapshot),
            GamepadInvocation::Disconnected(id) => ListenerInvocation::gamepad_disconnected(id),
            GamepadInvocation::Event(event) => ListenerInvocation::gamepad_event(
                crate::mechanical_port::source::animation::listener_invocation::GamepadEventInvocation {
                    full_state: event.full_state,
                    change: event.change,
                    standard_button: event.standard_button,
                    standard_axis: event.standard_axis,
                }),
        };
        let mut dispatched = None;
        self.focus_manager().with_focus_manager_mut(|manager| {
            manager.gamepad_dispatch(&invocation, Some(&mut dispatched));
        });
        self.broadcast_gamepad_to_scripted_drawables(&invocation, dispatched.as_ref());
    }
}

impl StateMachineInstance {
    pub fn new(
        machine: CoreHandle,
        artboard_instance: RuntimeArtboardInstanceWeakHandle,
    ) -> RuntimeStateMachineInstanceHandle {
        let instance = Self {
            occurrence: RuntimeStateMachineInstanceWeakHandle::default(),
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
            embedder_gamepads: GamepadBatchState::default(),
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
                layer_instance.with_layer_mut(|layer_instance| {
                    layer_instance.init(instance, layer, artboard_instance.clone());
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

    pub fn listener_has(&self, listener: &CoreHandle, listener_type: ListenerType) -> bool {
        listener
            .with(|listener| listener.state_machine_listener_has(listener_type))
            .flatten()
            .unwrap_or(false)
    }

    fn listener_has_any(&self, listener: &CoreHandle, listener_types: &[ListenerType]) -> bool {
        listener_types
            .iter()
            .copied()
            .any(|kind| self.listener_has(listener, kind))
    }

    fn listener_target_id(listener: &CoreHandle) -> u32 {
        CoreRegistry::get_uint_handle(listener,
            crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBase::TARGET_ID_PROPERTY_KEY as i32)
            .expect("a StateMachineListener retains targetId")
    }

    fn focus_data_child(target: &CoreHandle) -> Option<CoreHandle> {
        target
            .with(|target| {
                target.as_node()?;
                Some(target.as_container_component()?.children().to_vec())
            })
            .flatten()?
            .into_iter()
            .find(|child| child.with_downcast::<FocusData, _>(|_| ()).is_some())
    }

    fn semantic_data_child(target: &CoreHandle) -> Option<CoreHandle> {
        target
            .with(|target| {
                target.as_node()?;
                Some(target.as_container_component()?.children().to_vec())
            })
            .flatten()?
            .into_iter()
            .find(|child| {
                child
                    .with(|child| child.as_semantic_data().is_some())
                    .unwrap_or(false)
            })
    }

    pub fn perform_listener_changes(
        &mut self,
        listener: &CoreHandle,
        invocation: ListenerInvocation,
    ) {
        let actions = listener
            .with(|listener| listener.state_machine_listener_actions())
            .flatten()
            .expect("a listener retains its authored action list");
        for action in actions {
            action.with_mut(|action| {
                assert!(
                    action.listener_action_perform(self, &invocation),
                    "an authored listener action has concrete dispatch"
                );
            });
        }
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
            let Some(original_target) = source
                .with(|source| source.as_data_bind().and_then(DataBind::target))
                .flatten()
            else {
                continue;
            };
            let clone = source
                .clone_occurrence()
                .expect("a state-machine DataBind must be cloneable in its authored arena");
            let (file, converter) = source
                .with(|source| {
                    let source = source.as_data_bind()?;
                    Some((source.file(), source.converter()))
                })
                .flatten()
                .unwrap_or_default();
            let converter = converter.and_then(|converter| converter.clone_occurrence());
            clone.with_mut(|clone| {
                if let Some(clone) = clone.as_data_bind_mut() {
                    clone.set_file(file);
                    clone.set_converter(converter);
                }
            });
            self.add_data_bind(clone.clone());
            if original_target.is_type_of(BindablePropertyBase::TYPE_KEY) {
                let property = if let Some(property) = self
                    .bindable_property_instances
                    .get(&original_target)
                    .cloned()
                {
                    property
                } else {
                    let property = original_target.clone_occurrence().expect(
                        "a state-machine BindableProperty must be cloneable in its authored arena",
                    );
                    self.bindable_property_instances
                        .insert(original_target.clone(), property.clone());
                    property
                };
                clone.with_mut(|clone| {
                    if let Some(clone) = clone.as_data_bind_mut() {
                        clone.set_target(Some(property.clone()));
                    }
                });
                let to_source = clone
                    .with(|clone| {
                        clone
                            .as_data_bind()
                            .is_some_and(|clone| clone.base.flags() & 1 != 0)
                    })
                    .unwrap_or(false);
                if to_source {
                    self.bindable_data_binds_to_source.insert(property, clone);
                } else {
                    self.bindable_data_binds_to_target.insert(property, clone);
                }
            } else {
                clone.with_mut(|clone| {
                    if let Some(clone) = clone.as_data_bind_mut() {
                        clone.set_target(Some(original_target.clone()));
                    }
                });
                if original_target.is_type_of(StateTransitionBase::TYPE_KEY) {
                    let property = original_target
                        .insert_sibling(BindablePropertyNumber::default())
                        .expect("a transition data bind must retain its authored arena");
                    let property_key = source
                        .with(|source| {
                            source
                                .as_data_bind()
                                .map(|source| source.base.property_key())
                        })
                        .flatten()
                        .unwrap_or_default();
                    self.transition_property_instances
                        .entry(original_target)
                        .or_default()
                        .insert(property_key, property.clone());
                    clone.with_mut(|clone| {
                        if let Some(clone) = clone.as_data_bind_mut() {
                            clone.configure_target(
                                property,
                                BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY as u32,
                            );
                        }
                    });
                }
            }
        }
    }

    fn initialize_listeners(&mut self, hit_lookup: &mut HashMap<RuntimeDrawableOccurrence, usize>) {
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
            let target = self.resolve_artboard_object(Self::listener_target_id(&listener));
            if self.listener_has(&listener, ListenerType::Focus)
                || self.listener_has(&listener, ListenerType::Blur)
            {
                if let Some(focus_data) = target.as_ref().and_then(Self::focus_data_child) {
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
                if let Some(focus_data) = target.as_ref().and_then(Self::focus_data_child) {
                    let group = RuntimeKeyboardListenerGroupHandle::new(
                        focus_data,
                        Some(listener.clone()),
                        machine.clone(),
                    );
                    self.keyboard_listener_groups.push(group);
                }
            }
            if self.listener_has(&listener, ListenerType::SemanticAction) {
                if let Some(semantic_data) = target.as_ref().and_then(Self::semantic_data_child) {
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
                    let is_layout = target
                        .with(|target| target.as_layout_component().is_some())
                        .unwrap_or(false);
                    let hit_target = if is_layout {
                        target
                            .with_mut(|target| {
                                target
                                    .as_layout_component_mut()
                                    .and_then(|layout| layout.proxy())
                            })
                            .flatten()
                    } else {
                        Some(RuntimeDrawableOccurrence::Authored(target.clone()))
                    };
                    if let Some(hit_target) = hit_target {
                        self.add_to_hit_lookup(hit_target, is_layout, hit_lookup, group, false);
                    }
                }
                self.listener_groups.push(group);
            }
            if self.listener_has(&listener, ListenerType::Gamepad) {
                if let Some(focus_data) = target.as_ref().and_then(Self::focus_data_child) {
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
        hit_lookup: &mut HashMap<RuntimeDrawableOccurrence, usize>,
    ) {
        let providers: Vec<ListenerGroupProvider> = self
            .artboard_instance
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .expect("a state machine retains its ArtboardInstance")
            .into_iter()
            .filter_map(|object| ListenerGroupProvider::from(&object))
            .collect();
        for provider in providers {
            let groups = provider.listener_groups();
            for group_with_targets in groups {
                let (group, targets) = group_with_targets.into_parts();
                for target in targets {
                    let target_handle = target.component();
                    let layout = target_handle
                        .with(|target| {
                            target.as_layout_component().is_some()
                                || target
                                    .as_drawable()
                                    .is_some_and(|drawable| drawable.is_proxy())
                        })
                        .unwrap_or(false);
                    self.add_to_hit_lookup(
                        RuntimeDrawableOccurrence::Authored(target_handle),
                        layout,
                        hit_lookup,
                        group.clone(),
                        target.is_opaque(),
                    );
                }
                self.listener_groups.push(group);
            }
            let hits = provider.hit_components();
            self.hit_components.extend(hits);
        }
    }

    fn initialize_nested_hit_components(&mut self) {
        for nested in self
            .artboard_instance
            .with_artboard(|artboard| artboard.nested_artboards())
            .expect("a state machine retains its ArtboardInstance")
        {
            self.hit_components.push(Box::new(HitNestedArtboard {
                component: nested.clone(),
            }));
            for animation in nested_animations(&nested) {
                if animation.is_type_of(crate::mechanical_port::source::generated::animation::nested_state_machine_base::NestedStateMachineBase::TYPE_KEY)
                {
                    let notifier = nested_state_machine(&animation);
                    if let Some(notifier) = notifier {
                        let listener = self.occurrence.clone();
                        notifier.with_instance_mut(|notifier| {
                            notifier.set_nested_artboard(nested.clone());
                            notifier.add_nested_event_listener(listener);
                        });
                    }
                } else {
                    animation.with_mut(|animation| {
                        if let Some(notifier) = animation.as_nested_linear_animation_mut()
                            .and_then(|animation| animation.animation_instance_mut()) {
                            notifier.set_nested_artboard(nested.clone());
                            notifier.add_nested_event_listener(self.occurrence.clone());
                        }
                    });
                }
            }
        }
        for list in self
            .artboard_instance
            .with_artboard(|artboard| artboard.artboard_component_lists())
            .expect("a state machine retains its ArtboardInstance")
        {
            self.hit_components
                .push(Box::new(HitComponentList { component: list }));
        }
    }

    fn initialize_text_inputs(&mut self) {
        for text_input in self
            .artboard_instance
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .filter(|object| {
                        object
                            .with(|object| object.as_text_input().is_some())
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .expect("a state machine retains its ArtboardInstance")
        {
            let group = RuntimeListenerGroupHandle::new(Box::new(crate::mechanical_port::source::animation::text_input_listener_group::TextInputListenerGroup::new(text_input.clone())));
            let mut hit = HitDrawable::new(
                RuntimeDrawableOccurrence::Authored(text_input.clone()),
                RuntimeDrawableOccurrence::Authored(text_input),
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
            let mut host = ScriptUpdateRequestHost::default();
            let clone = if source.is_type_of(crate::mechanical_port::source::generated::animation::scripted_listener_action_base::ScriptedListenerActionBase::TYPE_KEY) {
                crate::mechanical_port::source::animation::scripted_listener_action::ScriptedListenerAction::clone_scripted_occurrence(&source, &mut self.data_bind_container, &mut host)
            } else {
                crate::mechanical_port::source::animation::scripted_transition_condition::ScriptedTransitionCondition::clone_scripted_occurrence(&source, &mut self.data_bind_container, &mut host)
            }.expect("a state-machine scripted definition has a concrete clone owner");
            if host.take_requested() {
                ScriptedObject::apply_update_request(&clone);
            }
            self.scripted_objects_map.insert(source, clone);
        }
        let context = self
            .artboard_instance
            .with_artboard(|artboard| artboard.data_context())
            .flatten();
        for object in self.scripted_objects_map.values() {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.set_data_context(context.clone());
                }
            });
        }
        self.init_scripted_objects();
        for object in self
            .artboard_instance
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .expect("a state machine retains its ArtboardInstance")
        {
            let Some((wants_keyboard, wants_gamepad)) = object
                .with(|object| {
                    object.as_container_component()?;
                    let scripted = object.as_scripted_object()?;
                    Some((
                        scripted.wants_keyboard_input() || scripted.wants_text_input(),
                        object.as_scripted_drawable().is_some()
                            && (scripted.wants_gamepad_connect()
                                || scripted.wants_gamepad_disconnect()
                                || scripted.wants_gamepad_event()),
                    ))
                })
                .flatten()
            else {
                continue;
            };
            if wants_keyboard {
                if let Some(focus_data) = Self::focus_data_child(&object) {
                    let group = RuntimeKeyboardListenerGroupHandle::new(
                        focus_data,
                        None,
                        self.occurrence.clone(),
                    );
                    self.keyboard_listener_groups.push(group);
                }
            }
            if wants_gamepad {
                self.gamepad_scripted_drawables.push(object);
            }
        }
    }

    fn add_to_hit_lookup(
        &mut self,
        target: RuntimeDrawableOccurrence,
        is_layout_component: bool,
        hit_lookup: &mut HashMap<RuntimeDrawableOccurrence, usize>,
        listener_group: RuntimeListenerGroupHandle,
        is_opaque: bool,
    ) {
        if is_layout_component {
            let index = if let Some(&index) = hit_lookup.get(&target) {
                index
            } else {
                let hit = HitDrawable::new(target.clone(), target.clone(), is_opaque, false, true);
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
        let Some(authored_target) = target.authored_handle() else {
            return;
        };
        let (is_shape, is_text_run) = authored_target
            .with(|target| {
                (
                    target.as_shape().is_some(),
                    target.as_text_value_run().is_some(),
                )
            })
            .expect("a hit target retains its authored owner");
        if is_shape || is_text_run {
            let index = if let Some(&index) = hit_lookup.get(&target) {
                index
            } else {
                let drawable = if is_text_run {
                    let text = authored_target
                        .with_mut(|target| {
                            let run = target.as_text_value_run_mut()?;
                            run.set_is_hit_target(true);
                            run.text_component()
                        })
                        .flatten()
                        .expect("a text run hit target retains its Text");
                    text.with_mut(|text| text.component_add_dirt(ComponentDirt::PATH, true));
                    RuntimeDrawableOccurrence::Authored(text)
                } else {
                    authored_target.with_mut(|target| {
                        target.as_shape_mut().expect("a shape hit target retains its Shape")
                            .add_flags(crate::mechanical_port::source::shapes::path_flags::PathFlags::NEVER_DEFER_UPDATE);
                        target.component_add_dirt(ComponentDirt::PATH, true);
                    });
                    target.clone()
                };
                let hit = HitDrawable::new(drawable, target.clone(), false, true, true);
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
        if let Some(children) = authored_target
            .with(|target| {
                target
                    .as_container_component()
                    .map(|container| container.children().to_vec())
            })
            .flatten()
        {
            for child in children {
                let is_layout = child
                    .with(|child| child.as_layout_component().is_some())
                    .unwrap_or(false);
                self.add_to_hit_lookup(
                    RuntimeDrawableOccurrence::Authored(child),
                    is_layout,
                    hit_lookup,
                    listener_group.clone(),
                    is_opaque,
                );
            }
        }
    }

    fn normalize_pointer_position(&self, mut position: Vec2D) -> Vec2D {
        self.artboard_instance
            .with_artboard(|artboard| {
                if artboard.frame_origin() {
                    position = Vec2D::new(
                        position.x - artboard.origin_x() * artboard.layout_width(),
                        position.y - artboard.origin_y() * artboard.layout_height(),
                    );
                }
                if artboard.has_self_transform() {
                    position = artboard.self_transform().invert_or_identity() * position;
                }
                position
            })
            .expect("a state machine retains its ArtboardInstance")
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
            component.prepare_event(position, hit_type, pointer_id);
        }
        let mut hit_something = false;
        let mut hit_opaque = false;
        for component in &mut hit_components {
            let result = component.process_event(
                self,
                position,
                hit_type,
                !hit_opaque,
                timestamp,
                pointer_id,
            );
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
            .any(|component| component.hit_test(position))
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
        let count = self.hit_components.len();
        let mut sorted = 0;
        for index in 0..count {
            if self.hit_components[index].component().authored_handle().is_some_and(|handle|
                handle.is_type_of(crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY)) {
                self.hit_components.swap(sorted, index);
                sorted += 1;
            }
        }
        let mut last = self
            .artboard_instance
            .with_artboard(|artboard| artboard.first_drawable())
            .flatten();
        while let Some(previous) = last.as_ref().and_then(|last| {
            last.with(|drawable| {
                drawable
                    .prev
                    .as_ref()
                    .and_then(|previous| previous.upgrade())
            })
            .flatten()
        }) {
            last = Some(previous);
        }
        while let Some(drawable) = last {
            for index in sorted..count {
                if self.hit_components[index].component().ptr_eq(&drawable) {
                    self.hit_components.swap(sorted, index);
                    sorted += 1;
                }
            }
            if sorted == count {
                break;
            }
            last = drawable
                .with(|drawable| drawable.next.as_ref().and_then(|next| next.upgrade()))
                .flatten();
        }
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
                self.artboard_instance
                    .with_artboard(|artboard| artboard.name().to_owned())
                    .expect("a state machine retains its ArtboardInstance")
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
            .artboard_instance
            .with_artboard(|artboard| artboard.draw_order_change_counter())
            .expect("a state machine retains its ArtboardInstance");
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
                .artboard_instance
                .with_artboard(|artboard| artboard.base.has_component_dirt())
                .unwrap_or(false)
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

    pub fn bool_input(&self, index: u32) -> Option<&SMIBool> {
        let InputInstance::Bool(value) = self.input_instances.get(index as usize)?.as_ref()? else {
            return None;
        };
        Some(value)
    }

    pub fn number_input(&self, index: u32) -> Option<&SMINumber> {
        let InputInstance::Number(value) = self.input_instances.get(index as usize)?.as_ref()?
        else {
            return None;
        };
        Some(value)
    }

    pub fn trigger_input(&self, index: u32) -> Option<&SMITrigger> {
        let InputInstance::Trigger(value) = self.input_instances.get(index as usize)?.as_ref()?
        else {
            return None;
        };
        Some(value)
    }

    pub fn bool_input_mut(&mut self, index: u32) -> Option<&mut SMIBool> {
        let InputInstance::Bool(value) = self.input_instances.get_mut(index as usize)?.as_mut()?
        else {
            return None;
        };
        Some(value)
    }

    pub fn number_input_mut(&mut self, index: u32) -> Option<&mut SMINumber> {
        let InputInstance::Number(value) =
            self.input_instances.get_mut(index as usize)?.as_mut()?
        else {
            return None;
        };
        Some(value)
    }

    pub fn trigger_input_mut(&mut self, index: u32) -> Option<&mut SMITrigger> {
        let InputInstance::Trigger(value) =
            self.input_instances.get_mut(index as usize)?.as_mut()?
        else {
            return None;
        };
        Some(value)
    }

    pub fn resolve_artboard_object(&self, id: u32) -> Option<CoreHandle> {
        self.artboard_instance
            .with_artboard(|artboard| artboard.base.resolve_handle(id))
            .flatten()
    }

    pub fn resolve_event(&self, id: u32) -> Option<CoreHandle> {
        self.resolve_artboard_object(id)
            .filter(|event| event.is_type_of(EventBase::TYPE_KEY))
    }

    pub fn view_model_property(&self, path: &[u32]) -> Option<CoreHandle> {
        self.data_context_handle
            .as_ref()?
            .with_context(|context| context.get_view_model_property(path))
    }

    pub fn nested_bool(&self, id: u32) -> Option<bool> {
        self.resolve_artboard_object(id)
            .and_then(|input| input.with_downcast::<NestedBool, _>(NestedBool::nested_value))
    }

    pub fn set_nested_bool(&mut self, id: u32, value: bool) {
        if let Some(input) = self.resolve_artboard_object(id) {
            input.with_downcast_mut::<NestedBool, _>(|input| input.set_nested_value(value));
        }
    }

    pub fn set_nested_number(&mut self, id: u32, value: f32) -> bool {
        self.resolve_artboard_object(id)
            .and_then(|input| {
                input.with_downcast_mut::<NestedNumber, _>(|input| input.set_nested_value(value))
            })
            .is_some()
    }

    pub fn fire_nested_trigger(&mut self, id: u32) -> bool {
        self.resolve_artboard_object(id)
            .and_then(|input| {
                input.with_downcast_mut::<NestedTrigger, _>(NestedTrigger::apply_value)
            })
            .is_some()
    }

    pub fn number_input_value(&self, index: u32) -> Option<f32> {
        let InputInstance::Number(value) = self.input_instances.get(index as usize)?.as_ref()?
        else {
            return None;
        };
        Some(value.value())
    }

    pub fn bindable_property_number_value(&self, property: &CoreHandle) -> Option<f32> {
        self.bindable_property_instance(property)?
            .with_downcast::<BindablePropertyNumber, _>(|property| property.base.property_value())
    }

    pub fn bindable_property_comparison_value(
        &self,
        property: &CoreHandle,
    ) -> Option<RuntimeComparisonValue> {
        let property = self.bindable_property_instance(property)?;
        property
            .with_downcast::<BindablePropertyNumber, _>(|property| {
                RuntimeComparisonValue::Number(property.base.property_value())
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyInteger, _>(|property| {
                    RuntimeComparisonValue::Uint(property.base.property_value())
                })
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyBoolean, _>(|property| {
                    RuntimeComparisonValue::Boolean(property.base.property_value())
                })
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyString, _>(|property| {
                    RuntimeComparisonValue::String(property.base.property_value().to_owned())
                })
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyColor, _>(|property| {
                    RuntimeComparisonValue::Color(property.base.property_value())
                })
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyEnum, _>(|property| {
                    RuntimeComparisonValue::Uint(property.base.property_value())
                })
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyTrigger, _>(|property| {
                    RuntimeComparisonValue::Uint(property.base.base.property_value())
                })
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyAsset, _>(|property| {
                    RuntimeComparisonValue::Uint(property.base.property_value())
                })
            })
            .or_else(|| {
                property.with_downcast::<BindablePropertyArtboard, _>(|property| {
                    RuntimeComparisonValue::Uint(property.base.property_value())
                })
            })
            .or_else(|| {
                property
                    .with_downcast::<BindablePropertyViewModel, _>(|property| {
                        property
                            .view_model_instance_value()
                            .map(RuntimeComparisonValue::ViewModel)
                    })
                    .flatten()
            })
    }

    pub fn component_comparison_value(
        &self,
        object_id: u32,
        property_key: u32,
    ) -> Option<RuntimeComparisonValue> {
        let object = self.resolve_artboard_object(object_id)?;
        match CoreRegistry::property_field_id(property_key as i32) as u16 {
            CoreRegistry::CORE_DOUBLE_TYPE_ID => {
                CoreRegistry::get_double_handle(&object, property_key as i32)
                    .map(RuntimeComparisonValue::Number)
            }
            CoreRegistry::CORE_BOOL_TYPE_ID => {
                CoreRegistry::get_bool_handle(&object, property_key as i32)
                    .map(RuntimeComparisonValue::Boolean)
            }
            CoreRegistry::CORE_STRING_TYPE_ID => {
                CoreRegistry::get_string_handle(&object, property_key as i32)
                    .map(RuntimeComparisonValue::String)
            }
            CoreRegistry::CORE_COLOR_TYPE_ID => {
                CoreRegistry::get_color_handle(&object, property_key as i32)
                    .map(RuntimeComparisonValue::Color)
            }
            CoreRegistry::CORE_UINT_TYPE_ID => {
                CoreRegistry::get_uint_handle(&object, property_key as i32)
                    .map(RuntimeComparisonValue::Uint)
            }
            _ => None,
        }
    }

    pub fn artboard_layout_size(&self) -> Option<(f32, f32)> {
        self.artboard_instance
            .with_artboard(|artboard| (artboard.base.layout_width(), artboard.base.layout_height()))
    }

    pub fn bindable_source_changed_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) -> bool {
        let Some(property) = self.bindable_property_instance(property) else {
            return false;
        };
        let Some(data_bind) = self.bindable_data_bind_to_target(&property) else {
            return false;
        };
        let source = data_bind
            .with(|data_bind| data_bind.as_data_bind().and_then(DataBind::source))
            .flatten();
        let Some(source) = source else {
            return false;
        };
        source
            .with(|source| {
                source.as_view_model_instance_value().is_some_and(|source| {
                    source.has_changed()
                        && layer
                            .as_ref()
                            .is_none_or(|layer| !source.is_used_in_layer(layer))
                })
            })
            .unwrap_or(false)
    }

    pub fn use_bindable_property_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        let Some(layer) = layer else {
            return;
        };
        let Some(property) = self.bindable_property_instance(property) else {
            return;
        };
        let Some(data_bind) = self.bindable_data_bind_to_target(&property) else {
            return;
        };
        let source = data_bind
            .with(|data_bind| data_bind.as_data_bind().and_then(DataBind::source))
            .flatten();
        if let Some(source) = source {
            source.with_mut(|source| {
                if let Some(source) = source.as_view_model_instance_value_mut() {
                    source.use_in_layer(layer);
                }
            });
        }
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
            let index = self
                .listener_view_models
                .iter()
                .position(|candidate| candidate.downgrade().ptr_eq(view_model))
                .expect("a reported view-model listener remains owned until dispatch");
            self.perform_listener_changes(&listener, ListenerInvocation::view_model_change(index));
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
            let target = self.resolve_artboard_object(Self::listener_target_id(&listener));
            if source
                .as_ref()
                .is_some_and(|source| target.as_ref() != Some(source))
            {
                continue;
            }
            let source_artboard = if let Some(source) = source.as_ref() {
                source
                    .with(|source| {
                        source
                            .as_nested_artboard()
                            .and_then(|nested| nested.artboard_instance_default())
                    })
                    .flatten()
                    .map(|artboard| artboard.downgrade())
                    .expect("an event source retains its nested ArtboardInstance")
            } else {
                self.artboard_instance.clone()
            };
            for report in events {
                if source.is_none() {
                    let resolved_target = source_artboard
                        .with_artboard(|artboard| {
                            artboard.resolve_handle(Self::listener_target_id(&listener))
                        })
                        .flatten();
                    if resolved_target.as_ref().is_some_and(|resolved_target| {
                        !resolved_target.is_type_of(crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY)
                            && !resolved_target.is_type_of(EventBase::TYPE_KEY)
                    }) {
                        continue;
                    }
                }
                let single_event =
                    listener.with_downcast::<StateMachineListenerSingle, _>(|listener| {
                        listener.base.event_id()
                    });
                let event_ids = if let Some(event_id) = single_event {
                    vec![event_id]
                } else {
                    listener.with(|listener| listener.state_machine_listener_input_types()).flatten()
                        .expect("a listener retains its input types").into_iter()
                        .filter_map(|input| input.with_downcast::<crate::mechanical_port::source::animation::listener_types::listener_input_type_event::ListenerInputTypeEvent, _>(|input| input.base.event_id()))
                        .collect()
                };
                let mut matched = false;
                for event_id in event_ids {
                    if source_artboard
                        .with_artboard(|artboard| artboard.resolve_handle(event_id))
                        .flatten()
                        .as_ref()
                        == report.event.as_ref()
                    {
                        let Some(event) = report.event.as_ref() else {
                            continue;
                        };
                        self.perform_listener_changes(
                            &listener,
                            ListenerInvocation::reported_event(event.clone(), report.seconds_delay),
                        );
                        matched = true;
                        break;
                    }
                }
                if matched && single_event.is_some() {
                    break;
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
            event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
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
            component.enable_pointer_events(pointer_id);
        }
    }

    pub fn disable_pointer_events(&mut self, pointer_id: i32) {
        for component in &mut self.hit_components {
            component.disable_pointer_events(pointer_id);
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

    pub fn focus_up(&mut self) -> bool {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::focus_up)
    }

    pub fn focus_down(&mut self) -> bool {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::focus_down)
    }

    pub fn focus_left(&mut self) -> bool {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::focus_left)
    }

    pub fn focus_right(&mut self) -> bool {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::focus_right)
    }

    pub fn clear_focus(&mut self) {
        self.focus_manager()
            .with_focus_manager_mut(FocusManager::clear_focus);
    }

    pub fn submit_gamepads_from_buffer(&mut self, data: &[u8]) -> bool {
        let mut gamepads = std::mem::take(&mut self.embedder_gamepads);
        let result = gamepads.submit(Some(data), self);
        self.embedder_gamepads = gamepads;
        result
    }

    pub fn broadcast_gamepad_to_scripted_drawables(
        &mut self,
        invocation: &ListenerInvocation,
        already_dispatched: Option<&CoreHandle>,
    ) -> HitResult {
        let mut hit_something = false;
        let mut hit_opaque = false;
        for component in &mut self.hit_components {
            let result = component.process_gamepad_invocation(invocation, already_dispatched);
            hit_something |= result != HitResult::None;
            hit_opaque |= result == HitResult::HitOpaque;
        }
        for drawable in self.gamepad_scripted_drawables.clone() {
            if Some(&drawable) == already_dispatched {
                continue;
            }
            hit_something |= drawable
                .with_mut(|drawable| {
                    let Some(drawable) = drawable.as_scripted_drawable_mut() else {
                        return false;
                    };
                    let accepts = match invocation.kind() {
                        ListenerInvocationKind::GamepadConnected => {
                            drawable.scripted.wants_gamepad_connect()
                        }
                        ListenerInvocationKind::GamepadEvent => {
                            drawable.scripted.wants_gamepad_event()
                        }
                        ListenerInvocationKind::GamepadDisconnected => {
                            drawable.scripted.wants_gamepad_disconnect()
                        }
                        _ => false,
                    };
                    accepts && drawable.gamepad_dispatch(invocation)
                })
                .unwrap_or(false);
        }
        if !hit_something {
            HitResult::None
        } else if hit_opaque {
            HitResult::HitOpaque
        } else {
            HitResult::Hit
        }
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
        for object in self
            .scripted_objects_map
            .values()
            .cloned()
            .collect::<Vec<_>>()
        {
            let properties = object.with_downcast::<crate::mechanical_port::source::animation::scripted_listener_action::ScriptedListenerAction, _>(|object| object.properties.clone())
                .or_else(|| object.with_downcast::<crate::mechanical_port::source::animation::scripted_transition_condition::ScriptedTransitionCondition, _>(|object| object.properties.clone()))
                .expect("a state-machine scripted occurrence retains its concrete owner");
            let mut host = ScriptUpdateRequestHost::default();
            ScriptedObject::reinit_occurrence(&object, &properties, &mut host);
            if host.take_requested() {
                ScriptedObject::apply_update_request(&object);
            }
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
        let source_artboard = self
            .artboard_instance
            .with_artboard(|artboard| artboard.base.artboard_source_handle())
            .flatten();
        let Some(source_artboard) = source_artboard else {
            return;
        };
        let source_data_binds = source_artboard
            .with_downcast::<Artboard, _>(Artboard::data_bind_handles)
            .unwrap_or_default();
        for data_bind in source_data_binds {
            let target = data_bind
                .with(|data_bind| data_bind.as_data_bind().and_then(DataBind::target))
                .flatten();
            if let Some(target) = target
                && target.is_type_of(KeyFrameBase::TYPE_KEY)
            {
                first_bind_by_target.entry(target).or_insert(data_bind);
            }
        }
        if first_bind_by_target.is_empty() {
            return;
        }
        state_instance.with_state_mut(|state| {
            state.for_each_animation_instance(&mut |animation_instance| {
                for keyframe in animation_instance.keyframes() {
                    let keyframe_type = keyframe.core_type().unwrap_or_default();
                    let holder_property_key = Self::keyframe_holder_property_key(keyframe_type);
                    if holder_property_key == 0 {
                        continue;
                    }
                    let Some(source_bind) = first_bind_by_target.get(&keyframe) else {
                        continue;
                    };
                    let holder = match keyframe_type {
                        KeyFrameDoubleBase::TYPE_KEY => {
                            keyframe.insert_sibling(BindablePropertyNumber::default())
                        }
                        KeyFrameColorBase::TYPE_KEY => {
                            keyframe.insert_sibling(BindablePropertyColor::default())
                        }
                        KeyFrameBoolBase::TYPE_KEY => {
                            keyframe.insert_sibling(BindablePropertyBoolean::default())
                        }
                        KeyFrameStringBase::TYPE_KEY => {
                            keyframe.insert_sibling(BindablePropertyString::default())
                        }
                        _ => None,
                    }
                    .expect("a supported KeyFrame retains its authored arena");
                    animation_instance.add_keyframe_value_holder(keyframe.clone(), holder.clone());
                    let clone = source_bind
                        .clone_occurrence()
                        .expect("an artboard DataBind must be cloneable in its authored arena");
                    let (file, converter) = source_bind
                        .with(|source_bind| {
                            let source_bind = source_bind.as_data_bind()?;
                            Some((source_bind.file(), source_bind.converter()))
                        })
                        .flatten()
                        .unwrap_or_default();
                    let converter = converter.and_then(|converter| converter.clone_occurrence());
                    clone.with_mut(|clone| {
                        if let Some(clone) = clone.as_data_bind_mut() {
                            clone.set_file(file);
                            clone.configure_target(holder, holder_property_key);
                            clone.initialize();
                            clone.set_converter(converter);
                        }
                    });
                    self.add_data_bind(clone.clone());
                    self.state_keyframe_data_binds
                        .entry(state_instance.clone())
                        .or_default()
                        .push(clone);
                }
            });
        });
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

    pub fn perform_scripted_listener(
        &mut self,
        owner: &CoreHandle,
        invocation: &ListenerInvocation,
    ) {
        let mut host = ScriptUpdateRequestHost::default();
        crate::mechanical_port::source::animation::scripted_listener_action::ScriptedListenerAction::perform_stateful(
            owner, &invocation.to_script_invocation(), &mut host,
        );
        if host.take_requested() {
            ScriptedObject::apply_update_request(owner);
        }
    }

    pub fn evaluate_scripted_condition(&self, owner: &CoreHandle) -> bool {
        let mut host = ScriptUpdateRequestHost::default();
        let result = crate::mechanical_port::source::animation::scripted_transition_condition::ScriptedTransitionCondition::evaluate_stateful(owner, &mut host);
        if host.take_requested() {
            ScriptedObject::apply_update_request(owner);
        }
        result
    }

    pub fn perform_scripted_pointer(
        &mut self,
        owner: &CoreHandle,
        hit_type: ListenerType,
        can_hit: bool,
        position: Vec2D,
        pointer_id: i32,
    ) -> HitResult {
        use crate::scripting::{ScriptMethod, ScriptedDrawablePointerHit};
        let Some((method, local)) = owner
            .with(|owner| {
                let scripted = owner.as_scripted_object()?;
                scripted.script_asset()?;
                if scripted.self_ref() == 0 {
                    return None;
                }
                let (accepts, method) = if can_hit {
                    match hit_type {
                        ListenerType::Down => {
                            (scripted.wants_pointer_down(), ScriptMethod::PointerDown)
                        }
                        ListenerType::Up => (scripted.wants_pointer_up(), ScriptMethod::PointerUp),
                        ListenerType::DragStart | ListenerType::DragEnd => return None,
                        _ => (scripted.wants_pointer_move(), ScriptMethod::PointerMove),
                    }
                } else {
                    (scripted.wants_pointer_exit(), ScriptMethod::PointerExit)
                };
                if !accepts {
                    return None;
                }
                let transform = owner.as_world_transform_component()?.world_transform();
                let mut inverse = crate::mechanical_port::source::math::mat2d::Mat2D::IDENTITY;
                if !transform.invert(&mut inverse) {
                    return None;
                }
                Some((method, inverse * position))
            })
            .flatten()
        else {
            return HitResult::None;
        };
        let mut host = ScriptUpdateRequestHost::default();
        let result = ScriptedObject::perform_pointer(owner, method, pointer_id, local, &mut host);
        if host.take_requested() {
            ScriptedObject::apply_update_request(owner);
        }
        if result.invoked {
            owner.with_mut(|owner| {
                owner
                    .as_scripted_drawable_mut()
                    .expect("a pointer script retains its ScriptedDrawable")
                    .wake_advance()
            });
        }
        match result.hit {
            ScriptedDrawablePointerHit::None => HitResult::None,
            ScriptedDrawablePointerHit::Hit => HitResult::Hit,
            ScriptedDrawablePointerHit::HitOpaque => HitResult::HitOpaque,
        }
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
            .artboard_instance
            .with_artboard(|artboard| artboard.nested_artboards())
            .unwrap_or_default()
        {
            for animation in nested_animations(&nested) {
                if let Some(machine) = nested_state_machine(&animation) {
                    machine.with_instance_mut(|machine| {
                        machine.remove_nested_event_listener(self.occurrence.clone())
                    });
                } else {
                    animation.with_mut(|animation| {
                        if let Some(notifier) = animation
                            .as_nested_linear_animation_mut()
                            .and_then(|animation| animation.animation_instance_mut())
                        {
                            notifier.remove_nested_event_listener(self.occurrence.clone());
                        }
                    });
                }
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
            property.remove_occurrence();
        }
        for (_, properties) in self.transition_property_instances.drain() {
            for (_, property) in properties {
                property.remove_occurrence();
            }
        }
        self.listener_view_models.clear();
        for (_, object) in self.scripted_objects_map.drain() {
            object.remove_occurrence();
        }
    }
}
