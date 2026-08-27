use crate::mechanical_port::source::{
    animation::{
        linear_animation_instance::LinearAnimationInstance,
        state_machine_input_instance::{
            InputInstanceMachine, SMIBool, SMIInput, SMINumber, SMITrigger,
        },
    },
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
    listener_type::ListenerType,
    math::vec2d::Vec2D,
    process_event_result::ProcessEventResult,
};
use std::collections::HashMap;

type Object = usize;

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

#[derive(Clone, Copy, Debug, Default)]
pub struct EventReport {
    pub event: Object,
    pub seconds_delay: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FocusState {
    pub has_focus: bool,
    pub expects_keyboard_input: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct QueuedFocusEvent {
    pub group: Object,
    pub is_focus: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct QueuedSemanticEvent {
    pub group: Object,
    pub action_type: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum InputInstance {
    Bool(*mut SMIBool),
    Number(*mut SMINumber),
    Trigger(*mut SMITrigger),
}

impl InputInstance {
    fn base(self) -> *mut SMIInput {
        match self {
            Self::Bool(value) => unsafe { &mut (*value).base },
            Self::Number(value) => unsafe { &mut (*value).base },
            Self::Trigger(value) => unsafe { &mut (*value).base },
        }
    }
}

/// The services owned by the surrounding translated runtime. This interface
/// keeps all cross-owner object operations explicit while this owner retains
/// the exact state, ordering, and branches of the pinned implementation.
pub trait StateMachineInstanceRuntime {
    fn deterministic_mode(&self) -> bool;
    fn seed_random(&mut self, seed: u32);
    fn random_value(&mut self) -> f64;
    fn machine_name(&self, machine: Object) -> String;
    fn machine_input_count(&self, machine: Object) -> usize;
    fn machine_input(&self, machine: Object, index: usize) -> Object;
    fn machine_layer_count(&self, machine: Object) -> usize;
    fn machine_layer(&self, machine: Object, index: usize) -> Object;
    fn machine_listener_count(&self, machine: Object) -> usize;
    fn machine_listener(&self, machine: Object, index: usize) -> Object;
    fn machine_data_bind_count(&self, machine: Object) -> usize;
    fn machine_data_bind(&self, machine: Object, index: usize) -> Object;
    fn machine_scripted_objects(&self, machine: Object) -> Vec<Object>;
    fn input_core_type(&self, input: Object) -> u16;
    fn make_input_instance(&mut self, input: Object, machine: *mut ()) -> Option<InputInstance>;
    fn input_name(&self, input: Object) -> &str;
    fn input_advanced(&mut self, input: InputInstance);
    fn layer_any_state(&self, layer: Object) -> Object;
    fn layer_entry_state(&self, layer: Object) -> Object;
    fn make_state_instance(&mut self, state: Object, artboard: Object) -> Object;
    fn delete_state_instance(&mut self, instance: Object);
    fn state_definition(&self, instance: Object) -> Object;
    fn state_advance(&mut self, instance: Object, seconds: f32, machine: *mut ());
    fn state_apply(&mut self, instance: Object, artboard: Object, mix: f32);
    fn state_keep_going(&self, instance: Object) -> bool;
    fn state_clear_spilled_time(&mut self, instance: Object);
    fn state_spilled_time(&self, instance: Object) -> f32;
    fn state_animation(&self, instance: Object) -> Object;
    fn state_animation_instance(&self, instance: Object) -> Object;
    fn state_for_each_animation_instance(
        &mut self,
        state: Object,
        callback: &mut dyn FnMut(&mut dyn StateMachineInstanceRuntime, Object),
    );
    fn state_transition_count(&self, state: Object) -> usize;
    fn state_transition(&self, state: Object, index: usize) -> Object;
    fn state_flags(&self, state: Object) -> u32;
    fn state_events(&self, state: Object) -> Vec<Object>;
    fn state_listener_actions(&self, state: Object) -> Vec<Object>;
    fn transition_state_to(&self, transition: Object) -> Object;
    fn transition_allowed(
        &mut self,
        transition: Object,
        from: Object,
        machine: *mut (),
        layer: *mut (),
    ) -> u8;
    fn transition_random_weight(&self, transition: Object) -> u32;
    fn transition_evaluated_weight(&self, transition: Object) -> u32;
    fn set_transition_evaluated_weight(&mut self, transition: Object, value: u32);
    fn transition_use_layer(&mut self, transition: Object, machine: *mut (), layer: *mut ());
    fn transition_duration(&self, transition: Object) -> u32;
    fn transition_duration_is_percentage(&self, transition: Object) -> bool;
    fn transition_interpolator(&self, transition: Object) -> Object;
    fn transition_enable_early_exit(&self, transition: Object) -> bool;
    fn transition_pause_on_exit(&self, transition: Object) -> bool;
    fn transition_apply_exit_condition(&mut self, transition: Object, from: Object) -> bool;
    fn transition_events(&self, transition: Object) -> Vec<Object>;
    fn transition_listener_actions(&self, transition: Object) -> Vec<Object>;
    fn transition_property_value(&self, property: Object) -> f32;
    fn transition_property_instance(
        &self,
        machine: &StateMachineInstance,
        transition: Object,
        property_key: u32,
    ) -> Object;
    fn fire_action_occurs(&self, action: Object) -> u8;
    fn fire_action_perform(&mut self, action: Object, machine: *mut ());
    fn listener_action_matches(&self, action: Object, occurrence: u8) -> bool;
    fn listener_action_perform(&mut self, action: Object, machine: *mut (), invocation: Object);
    fn make_animation_reset(&mut self, from: Object, to: Object, artboard: Object) -> Object;
    fn release_animation_reset(&mut self, reset: Object);
    fn apply_animation_reset(&mut self, reset: Object, artboard: Object);
    fn animation_apply(&mut self, animation: Object, artboard: Object, time: f32, mix: f32);
    fn animation_duration_seconds(&self, animation: Object) -> f32;
    fn animation_instance_time(&self, instance: Object) -> f32;
    fn interpolator_transform(&self, interpolator: Object, value: f32) -> f32;
    fn artboard_frame_origin(&self, artboard: Object) -> bool;
    fn artboard_origin(&self, artboard: Object) -> Vec2D;
    fn artboard_layout_size(&self, artboard: Object) -> Vec2D;
    fn artboard_inverse_self_transform(&self, artboard: Object, point: Vec2D) -> Option<Vec2D>;
    fn artboard_draw_order_change_counter(&self, artboard: Object) -> u8;
    fn artboard_ordered_hit_components(
        &self,
        artboard: Object,
        components: &[Object],
    ) -> Vec<usize>;
    fn artboard_update_data_binds(&mut self, machine: *mut (), force: bool);
    fn artboard_advance_data_binds(&mut self, machine: *mut (), seconds: f32) -> bool;
    fn artboard_advance_internal(&mut self, artboard: Object, seconds: f32, flags: u32) -> bool;
    fn artboard_update_pass(&mut self, artboard: Object, is_root: bool) -> bool;
    fn artboard_has_component_dirt(&self, artboard: Object) -> bool;
    fn artboard_reset(&mut self, artboard: Object);
    fn artboard_advance_scripted_view_models(&mut self, artboard: Object);
    fn artboard_resolve(&self, artboard: Object, id: u32) -> Object;
    fn artboard_file(&self, artboard: Object) -> Object;
    fn artboard_nested_artboards(&self, artboard: Object) -> Vec<Object>;
    fn artboard_component_lists(&self, artboard: Object) -> Vec<Object>;
    fn artboard_objects(&self, artboard: Object) -> Vec<Object>;
    fn artboard_text_inputs(&self, artboard: Object) -> Vec<Object>;
    fn artboard_source_data_binds(&self, artboard: Object) -> Vec<Object>;
    fn artboard_name(&self, artboard: Object) -> String;
    fn artboard_cleanup_focus_tree(&mut self, artboard: Object);
    fn artboard_build_focus_tree(&mut self, artboard: Object, manager: Object, parent: Object);
    fn artboard_focus_manager(&self, artboard: Object) -> Object;
    fn artboard_cleanup_semantic_tree(&mut self, artboard: Object);
    fn artboard_build_semantic_tree(&mut self, artboard: Object, manager: Object, parent: Object);
    fn artboard_semantic_manager(&self, artboard: Object) -> Object;
    fn component_id(&self, component: Object) -> Object;
    fn component_is_artboard(&self, component: Object) -> bool;
    fn component_hit_test(&self, component: Object, point: Vec2D, path: bool, clip: bool) -> bool;
    fn component_is_target_opaque(&self, component: Object) -> bool;
    fn component_is_shape(&self, component: Object) -> bool;
    fn component_is_text_run(&self, component: Object) -> bool;
    fn component_is_container(&self, component: Object) -> bool;
    fn component_is_layout(&self, component: Object) -> bool;
    fn component_is_drawable_proxy(&self, component: Object) -> bool;
    fn component_proxy(&self, component: Object) -> Object;
    fn component_children(&self, component: Object) -> Vec<Object>;
    fn component_mark_hit_path(&mut self, component: Object);
    fn text_run_text_component(&self, component: Object) -> Object;
    fn component_is_collapsed(&self, component: Object) -> bool;
    fn component_is_paused(&self, component: Object) -> bool;
    fn component_world_to_local(
        &self,
        component: Object,
        point: Vec2D,
        index: Option<i32>,
    ) -> Option<Vec2D>;
    fn component_ordered_indices(&self, component: Object) -> Vec<i32>;
    fn component_state_machine(&self, component: Object, index: i32) -> *mut StateMachineInstance;
    fn nested_animations(&self, nested_artboard: Object) -> Vec<Object>;
    fn nested_is_state_machine(&self, animation: Object) -> bool;
    fn nested_state_machine_instance(&self, animation: Object) -> *mut StateMachineInstance;
    fn nested_add_event_listener(&mut self, animation: Object, listener: *mut ());
    fn nested_remove_event_listener(&mut self, animation: Object, listener: *mut ());
    fn listener_group_reset(&mut self, group: Object, pointer: i32);
    fn listener_group_release(&mut self, group: Object, pointer: i32);
    fn listener_group_hover(&mut self, group: Object, pointer: i32);
    fn listener_group_consumed(&self, group: Object) -> bool;
    fn listener_group_process(
        &mut self,
        group: Object,
        component: Object,
        position: Vec2D,
        pointer: i32,
        kind: ListenerType,
        can_hit: bool,
        timestamp: f32,
        machine: *mut (),
    ) -> ProcessEventResult;
    fn listener_group_can_early_out(&self, group: Object, component: Object) -> bool;
    fn listener_group_needs_down(&self, group: Object, component: Object) -> bool;
    fn listener_group_needs_up(&self, group: Object, component: Object) -> bool;
    fn listener_group_enable(&mut self, group: Object, pointer: i32);
    fn listener_group_disable(&mut self, group: Object, pointer: i32);
    fn make_listener_group(&mut self, listener: Object) -> Object;
    fn make_text_input_listener_group(&mut self, text_input: Object, machine: *mut ()) -> Object;
    fn make_focus_listener_group(
        &mut self,
        focus_data: Object,
        listener: Object,
        machine: *mut (),
    ) -> Object;
    fn make_keyboard_listener_group(
        &mut self,
        focus_data: Object,
        listener: Object,
        machine: *mut (),
    ) -> Object;
    fn make_gamepad_listener_group(
        &mut self,
        focus_data: Object,
        listener: Object,
        machine: *mut (),
    ) -> Object;
    fn make_semantic_listener_group(
        &mut self,
        semantic_data: Object,
        listener: Object,
        machine: *mut (),
    ) -> Object;
    fn resolve_focus_data(&self, target: Object) -> Object;
    fn resolve_semantic_data(&self, target: Object) -> Object;
    fn listener_groups_from_provider(
        &mut self,
        provider: Object,
    ) -> Vec<(Object, Vec<(Object, bool)>)>;
    fn provided_hit_components(
        &mut self,
        provider: Object,
        machine: *mut (),
    ) -> Vec<Box<dyn HitComponent>>;
    fn object_listener_provider(&self, object: Object) -> Object;
    fn object_scripted(&self, object: Object) -> Object;
    fn scripted_wants_keyboard(&self, object: Object) -> bool;
    fn scripted_wants_text(&self, object: Object) -> bool;
    fn scripted_wants_gamepad(&self, object: Object) -> bool;
    fn listener_has(&self, listener: Object, kind: ListenerType) -> bool;
    fn listener_has_any(&self, listener: Object, kinds: &[ListenerType]) -> bool;
    fn listener_target_id(&self, listener: Object) -> u32;
    fn listener_event_ids(&self, listener: Object) -> Vec<u32>;
    fn listener_perform_changes(&mut self, listener: Object, machine: *mut (), invocation: Object);
    fn listener_invocation_none(&mut self) -> Object;
    fn listener_invocation_focus(&mut self, group: Object, focused: bool) -> Object;
    fn listener_invocation_semantic(&mut self, group: Object, action: u8) -> Object;
    fn listener_invocation_event(&mut self, event: Object, delay: f32) -> Object;
    fn listener_invocation_view_model(&mut self, view_model: Object) -> Object;
    fn listener_for_focus_group(&self, group: Object) -> Object;
    fn listener_for_semantic_group(&self, group: Object) -> Object;
    fn focus_manager_new(&mut self) -> Object;
    fn focus_manager_set_focus(&mut self, manager: Object, focus_data: Object);
    fn focus_manager_clear(&mut self, manager: Object);
    fn focus_manager_state(&self, manager: Object) -> FocusState;
    fn focus_manager_drop_hidden(&mut self, manager: Object);
    fn focus_manager_has_content(&self, manager: Object) -> bool;
    fn focus_manager_next(&mut self, manager: Object) -> bool;
    fn focus_manager_previous(&mut self, manager: Object) -> bool;
    fn semantic_manager_new(&mut self) -> Object;
    fn semantic_fire_action(&mut self, manager: Object, node: u32, action: u8);
    fn data_context_advanced(&mut self, context: Object);
    fn data_context_add_container(&mut self, context: Object, container: *mut ());
    fn data_context_remove_container(&mut self, context: Object, container: *mut ());
    fn data_context_main(&self, context: Object) -> Object;
    fn data_context_set_main(&mut self, context: Object, instance: Object);
    fn data_context_slot(&self, context: Object, slot: u32) -> Object;
    fn data_context_set_slot(&mut self, context: Object, slot: u32, instance: Object);
    fn data_context_new(&mut self, main: Object) -> Object;
    fn data_context_property(&self, context: Object, path_owner: Object) -> Object;
    fn listener_is_single(&self, listener: Object) -> bool;
    fn listener_view_model_inputs(&self, listener: Object) -> Vec<Object>;
    fn view_model_value_is_trigger(&self, value: Object) -> bool;
    fn view_model_trigger_value(&self, value: Object) -> u32;
    fn view_model_add_dependent(&mut self, value: Object, dependent: Object);
    fn view_model_remove_dependent(&mut self, value: Object, dependent: Object);
    fn complete_default_main(&mut self, artboard: Object) -> Object;
    fn global_view_models(&self, file: Object) -> Vec<Object>;
    fn view_model_name(&self, view_model: Object) -> String;
    fn view_model_slot(&self, file: Object, name: &str) -> Option<u32>;
    fn view_model_is_global(&self, file: Object, slot: u32) -> bool;
    fn create_default_view_model(&mut self, file: Object, view_model: Object) -> Object;
    fn artboard_set_data_context(&mut self, artboard: Object, context: Object);
    fn artboard_clear_data_context(&mut self, artboard: Object);
    fn artboard_relink_data_context(&mut self, artboard: Object);
    fn bind_data_binds_from_context(&mut self, machine: *mut (), context: Object);
    fn unbind_data_binds(&mut self, machine: *mut ());
    fn clone_data_bind(&mut self, bind: Object) -> Object;
    fn data_bind_target(&self, bind: Object) -> Object;
    fn data_bind_flags(&self, bind: Object) -> u32;
    fn data_bind_property_key(&self, bind: Object) -> u32;
    fn data_bind_is_transition_target(&self, bind: Object) -> bool;
    fn data_bind_is_keyframe_target(&self, bind: Object) -> bool;
    fn data_bind_bindable_target(&self, bind: Object) -> bool;
    fn clone_bindable_property(&mut self, property: Object) -> Object;
    fn make_transition_property(&mut self) -> Object;
    fn configure_data_bind_target(&mut self, bind: Object, target: Object, property_key: u32);
    fn add_data_bind(&mut self, machine: *mut (), bind: Object);
    fn remove_data_bind(&mut self, machine: *mut (), bind: Object);
    fn delete_data_bind(&mut self, bind: Object);
    fn delete_all_data_binds(&mut self, machine: *mut ());
    fn data_bind_on_changed(&mut self, bind: Object, callback: fn());
    fn delete_owned_object(&mut self, object: Object);
    fn keyframe_type(&self, keyframe: Object) -> u16;
    fn animation_keyframes(&self, animation_instance: Object) -> Vec<Object>;
    fn make_keyframe_holder(&mut self, keyframe_type: u16) -> Object;
    fn add_keyframe_holder(&mut self, animation_instance: Object, keyframe: Object, holder: Object);
    fn scripted_clone(&mut self, source: Object, machine: *mut ()) -> Object;
    fn scripted_set_data_context(&mut self, object: Object, context: Object);
    fn scripted_initialize(&mut self, object: Object);
    fn scripted_hydrate_inputs(&mut self, object: Object);
    fn scripted_delete(&mut self, object: Object);
    fn event_is_audio(&self, event: Object) -> bool;
    fn event_play_audio(&mut self, event: Object);
    fn nested_event_listeners(&self, machine: *mut ()) -> Vec<*mut StateMachineInstance>;
    fn nested_artboard_context(&self, machine: *mut ()) -> Object;
    fn gamepad_submit_buffer(&mut self, machine: *mut (), data: &[u8]) -> bool;
    fn gamepad_broadcast(
        &mut self,
        machine: *mut (),
        invocation: Object,
        skipped: Object,
    ) -> HitResult;
}

struct StateMachineLayerInstance {
    state_machine_instance: *mut StateMachineInstance,
    layer: Object,
    artboard_instance: Object,
    any_state_instance: Object,
    current_state: Object,
    state_from: Object,
    transition: Object,
    transition_duration_property: Object,
    animation_reset: Object,
    transition_completed: bool,
    hold_animation_from: bool,
    mix: f32,
    mix_from: f32,
    state_machine_changed_on_advance: bool,
    waiting_for_exit: bool,
    hold_animation: Object,
    hold_time: f32,
}

impl Default for StateMachineLayerInstance {
    fn default() -> Self {
        Self {
            state_machine_instance: std::ptr::null_mut(),
            layer: 0,
            artboard_instance: 0,
            any_state_instance: 0,
            current_state: 0,
            state_from: 0,
            transition: 0,
            transition_duration_property: 0,
            animation_reset: 0,
            transition_completed: false,
            hold_animation_from: false,
            mix: 1.0,
            mix_from: 1.0,
            state_machine_changed_on_advance: false,
            waiting_for_exit: false,
            hold_animation: 0,
            hold_time: 0.0,
        }
    }
}

impl StateMachineLayerInstance {
    const MAX_ITERATIONS: usize = 100;

    fn runtime(&mut self) -> &mut dyn StateMachineInstanceRuntime {
        unsafe { (&mut *self.state_machine_instance).runtime_mut() }
    }

    fn init(
        &mut self,
        state_machine_instance: *mut StateMachineInstance,
        layer: Object,
        artboard: Object,
    ) {
        self.state_machine_instance = state_machine_instance;
        self.artboard_instance = artboard;
        let runtime = self.runtime();
        let seed = if runtime.deterministic_mode() {
            1
        } else {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u32
        };
        runtime.seed_random(seed);
        debug_assert_eq!(self.layer, 0);
        self.any_state_instance =
            runtime.make_state_instance(runtime.layer_any_state(layer), artboard);
        unsafe {
            (&mut *state_machine_instance).build_state_keyframe_binds(self.any_state_instance)
        };
        self.layer = layer;
        let entry = self.runtime().layer_entry_state(layer);
        self.change_state(entry);
    }

    fn reset_state(&mut self) {
        if self.state_from != 0
            && self.state_from != self.any_state_instance
            && self.state_from != self.current_state
        {
            unsafe {
                (&mut *self.state_machine_instance).remove_state_keyframe_binds(self.state_from)
            };
            let state = self.state_from;
            self.runtime().delete_state_instance(state);
        }
        self.state_from = 0;
        if self.current_state != 0 && self.current_state != self.any_state_instance {
            unsafe {
                (&mut *self.state_machine_instance).remove_state_keyframe_binds(self.current_state)
            };
            let state = self.current_state;
            self.runtime().delete_state_instance(state);
        }
        self.current_state = 0;
        let entry = self.runtime().layer_entry_state(self.layer);
        self.change_state(entry);
    }

    fn resolved_duration(&mut self) -> u32 {
        if self.transition_duration_property != 0 {
            return self
                .runtime()
                .transition_property_value(self.transition_duration_property)
                .round()
                .max(0.0) as u32;
        }
        self.runtime().transition_duration(self.transition)
    }

    fn resolved_mix_time(&mut self) -> f32 {
        let duration = self.resolved_duration();
        if duration == 0 {
            return 0.0;
        }
        if self
            .runtime()
            .transition_duration_is_percentage(self.transition)
        {
            let animation = self.runtime().state_animation(self.state_from);
            let animation_duration = if animation == 0 {
                0.0
            } else {
                self.runtime().animation_duration_seconds(animation)
            };
            duration as f32 / 100.0 * animation_duration
        } else {
            duration as f32 / 1000.0
        }
    }

    fn update_mix(&mut self, seconds: f32) {
        if self.transition != 0 && self.state_from != 0 && self.resolved_duration() != 0 {
            let mix_time = self.resolved_mix_time();
            self.mix = if mix_time == 0.0 {
                1.0
            } else {
                (self.mix + seconds / mix_time).clamp(0.0, 1.0)
            };
            if self.mix == 1.0 && !self.transition_completed {
                self.transition_completed = true;
                self.clear_animation_reset();
                let events = self.runtime().transition_events(self.transition);
                self.fire_events(1, &events);
                let actions = self.runtime().transition_listener_actions(self.transition);
                self.perform_listener_actions(1, &actions);
            }
        } else {
            self.mix = 1.0;
        }
    }

    fn advance(&mut self, seconds: f32, new_frame: bool) -> bool {
        if new_frame {
            self.state_machine_changed_on_advance = false;
        }
        let current = self.current_state;
        let machine = self.state_machine_instance.cast();
        self.runtime().state_advance(current, seconds, machine);
        self.update_mix(seconds);
        if self.state_from != 0 && self.mix < 1.0 && !self.hold_animation_from {
            let from = self.state_from;
            self.runtime().state_advance(from, seconds, machine);
        }
        self.apply();
        let mut changed = false;
        for iteration in 0.. {
            if !self.update_state() {
                break;
            }
            changed = true;
            self.apply();
            if iteration == Self::MAX_ITERATIONS {
                let machine_name = unsafe { (&*self.state_machine_instance).name() };
                let layer = self.layer;
                let artboard = self.artboard_instance;
                eprintln!(
                    "{} StateMachine exceeded max iterations in layer {} on artboard {}",
                    machine_name, layer, artboard
                );
                return false;
            }
        }
        let current = self.current_state;
        self.runtime().state_clear_spilled_time(current);
        changed
            || self.mix != 1.0
            || self.waiting_for_exit
            || (self.current_state != 0 && self.runtime().state_keep_going(self.current_state))
    }

    fn is_transitioning(&mut self) -> bool {
        self.transition != 0
            && self.state_from != 0
            && self.resolved_duration() != 0
            && self.mix < 1.0
    }

    fn update_state(&mut self) -> bool {
        if self.is_transitioning() && !self.runtime().transition_enable_early_exit(self.transition)
        {
            return false;
        }
        self.waiting_for_exit = false;
        if self.try_change_state_from(self.any_state_instance) {
            return true;
        }
        self.try_change_state_from(self.current_state)
    }

    fn fire_events(&mut self, occurrence: u8, events: &[Object]) {
        for &event in events {
            if self.runtime().fire_action_occurs(event) == occurrence {
                let machine = self.state_machine_instance.cast();
                self.runtime().fire_action_perform(event, machine);
            }
        }
    }

    fn perform_listener_actions(&mut self, occurrence: u8, actions: &[Object]) {
        for &action in actions {
            if self.runtime().listener_action_matches(action, occurrence) {
                let machine = self.state_machine_instance.cast();
                let invocation = self.runtime().listener_invocation_none();
                self.runtime()
                    .listener_action_perform(action, machine, invocation);
            }
        }
    }

    fn can_change_state(&mut self, state_to: Object) -> bool {
        self.current_state == 0 || self.runtime().state_definition(self.current_state) != state_to
    }

    fn change_state(&mut self, state_to: Object) {
        if self.current_state != 0
            && self.runtime().state_definition(self.current_state) == state_to
        {
            return;
        }
        if self.current_state != 0 {
            let state = self.runtime().state_definition(self.current_state);
            let events = self.runtime().state_events(state);
            self.fire_events(1, &events);
            let actions = self.runtime().state_listener_actions(state);
            self.perform_listener_actions(1, &actions);
        }
        self.current_state = if state_to == 0 {
            0
        } else {
            self.runtime()
                .make_state_instance(state_to, self.artboard_instance)
        };
        if self.current_state != 0 {
            unsafe {
                (&mut *self.state_machine_instance).build_state_keyframe_binds(self.current_state)
            };
            let state = self.runtime().state_definition(self.current_state);
            let events = self.runtime().state_events(state);
            self.fire_events(0, &events);
            let actions = self.runtime().state_listener_actions(state);
            self.perform_listener_actions(0, &actions);
        }
    }

    fn find_random_transition(&mut self, from_instance: Object) -> Object {
        let state = self.runtime().state_definition(from_instance);
        let mut total_weight = 0;
        for index in 0..self.runtime().state_transition_count(state) {
            let transition = self.runtime().state_transition(state, index);
            if self.can_change_state(self.runtime().transition_state_to(transition)) {
                let allowed = self.runtime().transition_allowed(
                    transition,
                    from_instance,
                    self.state_machine_instance.cast(),
                    (self as *mut Self).cast(),
                );
                if allowed == 2 {
                    let weight = self.runtime().transition_random_weight(transition);
                    self.runtime()
                        .set_transition_evaluated_weight(transition, weight);
                    total_weight += weight;
                } else {
                    self.runtime()
                        .set_transition_evaluated_weight(transition, 0);
                    if allowed == 1 {
                        self.waiting_for_exit = true;
                    }
                }
            } else {
                self.runtime()
                    .set_transition_evaluated_weight(transition, 0);
            }
        }
        if total_weight == 0 {
            return 0;
        }
        let random_weight = self.runtime().random_value() * total_weight as f64;
        let mut current_weight = 0.0;
        for index in 0..self.runtime().state_transition_count(state) {
            let transition = self.runtime().state_transition(state, index);
            let weight = self.runtime().transition_evaluated_weight(transition) as f64;
            if current_weight + weight > random_weight {
                self.runtime().transition_use_layer(
                    transition,
                    self.state_machine_instance.cast(),
                    (self as *mut Self).cast(),
                );
                return transition;
            }
            current_weight += weight;
        }
        0
    }

    fn find_allowed_transition(&mut self, from_instance: Object) -> Object {
        let state = self.runtime().state_definition(from_instance);
        if self.runtime().state_flags(state) & 1 != 0 {
            return self.find_random_transition(from_instance);
        }
        for index in 0..self.runtime().state_transition_count(state) {
            let transition = self.runtime().state_transition(state, index);
            if !self.can_change_state(self.runtime().transition_state_to(transition)) {
                continue;
            }
            let allowed = self.runtime().transition_allowed(
                transition,
                from_instance,
                self.state_machine_instance.cast(),
                (self as *mut Self).cast(),
            );
            if allowed == 2 {
                let weight = self.runtime().transition_random_weight(transition);
                self.runtime()
                    .set_transition_evaluated_weight(transition, weight);
                self.runtime().transition_use_layer(
                    transition,
                    self.state_machine_instance.cast(),
                    (self as *mut Self).cast(),
                );
                return transition;
            }
            self.runtime()
                .set_transition_evaluated_weight(transition, 0);
            if allowed == 1 {
                self.waiting_for_exit = true;
            }
        }
        0
    }

    fn clear_animation_reset(&mut self) {
        if self.animation_reset != 0 {
            let reset = self.animation_reset;
            self.runtime().release_animation_reset(reset);
            self.animation_reset = 0;
        }
    }

    fn try_change_state_from(&mut self, from_instance: Object) -> bool {
        if from_instance == 0 {
            return false;
        }
        let out_state = self.current_state;
        let transition = self.find_allowed_transition(from_instance);
        if transition == 0 {
            return false;
        }
        self.clear_animation_reset();
        let state_to = self.runtime().transition_state_to(transition);
        self.change_state(state_to);
        self.state_machine_changed_on_advance = true;
        self.transition = transition;
        self.transition_duration_property = self.runtime().transition_property_instance(
            unsafe { &*self.state_machine_instance },
            transition,
            StateTransitionBase::DURATION_PROPERTY_KEY as u32,
        );
        let events = self.runtime().transition_events(transition);
        self.fire_events(0, &events);
        let actions = self.runtime().transition_listener_actions(transition);
        self.perform_listener_actions(0, &actions);
        self.transition_completed = self.resolved_duration() == 0;
        if self.transition_completed {
            self.fire_events(1, &events);
            self.perform_listener_actions(1, &actions);
        }
        if self.state_from != 0 && self.state_from != self.any_state_instance {
            unsafe {
                (&mut *self.state_machine_instance).remove_state_keyframe_binds(self.state_from)
            };
            let old = self.state_from;
            self.runtime().delete_state_instance(old);
        }
        self.state_from = out_state;
        if !self.transition_completed {
            self.animation_reset = self.runtime().make_animation_reset(
                self.state_from,
                self.current_state,
                self.artboard_instance,
            );
        }
        if out_state != 0
            && self
                .runtime()
                .transition_apply_exit_condition(transition, out_state)
        {
            let instance = self.runtime().state_animation_instance(self.state_from);
            self.hold_animation = self.runtime().state_animation(self.state_from);
            self.hold_time = self.runtime().animation_instance_time(instance);
        }
        self.mix_from = self.mix;
        if self.mix != 0.0 {
            self.hold_animation_from = self.runtime().transition_pause_on_exit(transition);
        }
        if self.current_state != 0 {
            let advance_time = if self.state_from == 0 {
                0.0
            } else {
                self.runtime().state_spilled_time(self.state_from)
            };
            self.runtime().state_advance(
                self.current_state,
                advance_time,
                self.state_machine_instance.cast(),
            );
        }
        self.mix = 0.0;
        self.update_mix(0.0);
        self.waiting_for_exit = false;
        true
    }

    fn apply(&mut self) {
        if self.animation_reset != 0 {
            self.runtime()
                .apply_animation_reset(self.animation_reset, self.artboard_instance);
        }
        if self.hold_animation != 0 {
            self.runtime().animation_apply(
                self.hold_animation,
                self.artboard_instance,
                self.hold_time,
                self.mix_from,
            );
            self.hold_animation = 0;
        }
        let interpolator = if self.transition == 0 {
            0
        } else {
            self.runtime().transition_interpolator(self.transition)
        };
        if self.state_from != 0 && self.mix < 1.0 {
            let mix = if interpolator == 0 {
                self.mix_from
            } else {
                self.runtime()
                    .interpolator_transform(interpolator, self.mix_from)
            };
            self.runtime()
                .state_apply(self.state_from, self.artboard_instance, mix);
        }
        if self.current_state != 0 {
            let mix = if interpolator == 0 {
                self.mix
            } else {
                self.runtime()
                    .interpolator_transform(interpolator, self.mix)
            };
            self.runtime()
                .state_apply(self.current_state, self.artboard_instance, mix);
        }
    }

    fn current_state(&mut self) -> Object {
        if self.current_state == 0 {
            0
        } else {
            self.runtime().state_definition(self.current_state)
        }
    }

    fn current_animation(&mut self) -> Object {
        if self.current_state == 0 {
            0
        } else {
            self.runtime().state_animation_instance(self.current_state)
        }
    }
}

impl Drop for StateMachineLayerInstance {
    fn drop(&mut self) {
        if self.state_machine_instance.is_null() {
            return;
        }
        let any = self.any_state_instance;
        let current = self.current_state;
        let from = self.state_from;
        if any != 0 {
            self.runtime().delete_state_instance(any);
        }
        if current != 0 {
            self.runtime().delete_state_instance(current);
        }
        if from != 0 {
            self.runtime().delete_state_instance(from);
        }
    }
}

pub trait HitComponent {
    fn component(&self) -> Object;
    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult;
    fn process_gamepad_invocation(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        invocation: Object,
        already_dispatched: Object,
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

struct HitDrawable {
    component: Object,
    state_machine_instance: *mut StateMachineInstance,
    drawable: Object,
    hit_radius: f32,
    is_hovered: bool,
    can_early_out: bool,
    has_down_listener: bool,
    has_up_listener: bool,
    is_opaque: bool,
    listeners: Vec<Object>,
    hit_path: bool,
    hit_clip: bool,
}

type HitExpandable = HitDrawable;
type HitTextRun = HitExpandable;
type HitLayout = HitDrawable;

impl HitDrawable {
    fn new(
        runtime: &dyn StateMachineInstanceRuntime,
        drawable: Object,
        component: Object,
        machine: *mut StateMachineInstance,
        is_opaque: bool,
        hit_path: bool,
        hit_clip: bool,
    ) -> Self {
        Self {
            component,
            state_machine_instance: machine,
            drawable,
            hit_radius: 2.0,
            is_hovered: false,
            can_early_out: !runtime.component_is_target_opaque(drawable),
            has_down_listener: false,
            has_up_listener: false,
            is_opaque,
            listeners: Vec::new(),
            hit_path,
            hit_clip,
        }
    }

    fn add_listener(&mut self, runtime: &dyn StateMachineInstanceRuntime, group: Object) {
        if !runtime.listener_group_can_early_out(group, self.component) {
            self.can_early_out = false;
        } else {
            if runtime.listener_group_needs_down(group, self.component) {
                self.has_down_listener = true;
            }
            if runtime.listener_group_needs_up(group, self.component) {
                self.has_up_listener = true;
            }
        }
        self.listeners.push(group);
    }
}

impl HitComponent for HitDrawable {
    fn component(&self) -> Object {
        self.component
    }

    fn hit_test(&self, runtime: &dyn StateMachineInstanceRuntime, position: Vec2D) -> bool {
        runtime.component_hit_test(self.component, position, self.hit_path, self.hit_clip)
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
            return;
        }
        self.is_hovered = hit_type != ListenerType::Exit && self.hit_test(runtime, position);
        if self.is_hovered {
            for &listener in &self.listeners {
                runtime.listener_group_hover(listener, pointer_id);
            }
        }
    }

    fn process_gamepad_invocation(
        &mut self,
        _runtime: &mut dyn StateMachineInstanceRuntime,
        _invocation: Object,
        _already_dispatched: Object,
    ) -> HitResult {
        HitResult::None
    }

    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
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
        for &listener in &self.listeners {
            if runtime.listener_group_consumed(listener) {
                continue;
            }
            if runtime.listener_group_process(
                listener,
                self.component,
                position,
                pointer_id,
                hit_type,
                can_hit,
                timestamp,
                self.state_machine_instance.cast(),
            ) == ProcessEventResult::Scroll
            {
                blocking = true;
            }
        }
        if !self.is_hovered || !can_hit {
            HitResult::None
        } else if self.is_opaque || runtime.component_is_target_opaque(self.drawable) || blocking {
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
        for &listener in &self.listeners {
            runtime.listener_group_enable(listener, pointer_id);
        }
    }

    fn disable_pointer_events(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        pointer_id: i32,
    ) {
        for &listener in &self.listeners {
            runtime.listener_group_disable(listener, pointer_id);
        }
    }
}

struct HitNestedArtboard {
    component: Object,
    state_machine_instance: *mut StateMachineInstance,
}

impl HitComponent for HitNestedArtboard {
    fn component(&self) -> Object {
        self.component
    }

    fn hit_test(&self, runtime: &dyn StateMachineInstanceRuntime, position: Vec2D) -> bool {
        if runtime.component_is_collapsed(self.component)
            || runtime.component_is_paused(self.component)
        {
            return false;
        }
        let Some(local) = runtime.component_world_to_local(self.component, position, None) else {
            return false;
        };
        runtime
            .nested_animations(self.component)
            .into_iter()
            .filter(|&animation| runtime.nested_is_state_machine(animation))
            .any(|animation| {
                let instance = runtime.nested_state_machine_instance(animation);
                !instance.is_null() && unsafe { (&*instance).hit_test(local) }
            })
    }

    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult {
        if runtime.component_is_collapsed(self.component)
            || runtime.component_is_paused(self.component)
        {
            return HitResult::None;
        }
        let Some(local) = runtime.component_world_to_local(self.component, position, None) else {
            return HitResult::None;
        };
        let mut result = HitResult::None;
        for animation in runtime.nested_animations(self.component) {
            if !runtime.nested_is_state_machine(animation) {
                continue;
            }
            let instance = runtime.nested_state_machine_instance(animation);
            if instance.is_null() {
                continue;
            }
            let nested = unsafe { &mut *instance };
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
        }
        result
    }

    fn process_gamepad_invocation(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        invocation: Object,
        already_dispatched: Object,
    ) -> HitResult {
        for animation in runtime.nested_animations(self.component) {
            if runtime.nested_is_state_machine(animation) {
                let instance = runtime.nested_state_machine_instance(animation);
                if !instance.is_null() {
                    unsafe {
                        (&mut *instance).broadcast_gamepad_to_scripted_drawables(
                            invocation,
                            already_dispatched,
                        );
                    }
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
    component: Object,
    state_machine_instance: *mut StateMachineInstance,
}

impl HitComponent for HitComponentList {
    fn component(&self) -> Object {
        self.component
    }

    fn hit_test(&self, runtime: &dyn StateMachineInstanceRuntime, position: Vec2D) -> bool {
        if runtime.component_is_collapsed(self.component) {
            return false;
        }
        for index in runtime
            .component_ordered_indices(self.component)
            .into_iter()
            .rev()
        {
            let Some(local) =
                runtime.component_world_to_local(self.component, position, Some(index))
            else {
                continue;
            };
            let machine = runtime.component_state_machine(self.component, index);
            if !machine.is_null() && unsafe { (&*machine).hit_test(local) } {
                return true;
            }
        }
        false
    }

    fn process_event(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        position: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        timestamp: f32,
        pointer_id: i32,
    ) -> HitResult {
        if runtime.component_is_collapsed(self.component) {
            return HitResult::None;
        }
        let mut result = HitResult::None;
        let mut running_can_hit = can_hit;
        for index in runtime
            .component_ordered_indices(self.component)
            .into_iter()
            .rev()
        {
            let Some(local) =
                runtime.component_world_to_local(self.component, position, Some(index))
            else {
                continue;
            };
            let machine = runtime.component_state_machine(self.component, index);
            if machine.is_null() {
                continue;
            }
            let nested = unsafe { &mut *machine };
            let item = if running_can_hit {
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
                    ListenerType::Down | ListenerType::Up | ListenerType::Move | ListenerType::Exit
                ) {
                    nested.pointer_exit(local, pointer_id);
                }
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

    fn process_gamepad_invocation(
        &mut self,
        runtime: &mut dyn StateMachineInstanceRuntime,
        invocation: Object,
        already_dispatched: Object,
    ) -> HitResult {
        if runtime.component_is_collapsed(self.component) {
            return HitResult::None;
        }
        let mut result = HitResult::None;
        let mut running_can_hit = true;
        for index in runtime
            .component_ordered_indices(self.component)
            .into_iter()
            .rev()
        {
            let machine = runtime.component_state_machine(self.component, index);
            if machine.is_null() {
                continue;
            }
            let item = if running_can_hit {
                unsafe {
                    (&mut *machine)
                        .broadcast_gamepad_to_scripted_drawables(invocation, already_dispatched)
                }
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

struct ListenerViewModelPropertyBinding {
    parent: *mut ListenerViewModel,
    view_model_instance_value: Object,
    path_owner: Object,
}

impl ListenerViewModelPropertyBinding {
    fn new(
        parent: *mut ListenerViewModel,
        value: Object,
        path_owner: Object,
        runtime: &mut dyn StateMachineInstanceRuntime,
    ) -> Self {
        let mut binding = Self {
            parent,
            view_model_instance_value: value,
            path_owner,
        };
        runtime.view_model_add_dependent(value, (&mut binding as *mut Self) as Object);
        binding
    }

    fn clear_data_context(&mut self, runtime: &mut dyn StateMachineInstanceRuntime) {
        if self.view_model_instance_value != 0 {
            runtime.view_model_remove_dependent(
                self.view_model_instance_value,
                (self as *mut Self) as Object,
            );
            self.view_model_instance_value = 0;
        }
    }

    fn relink_data_bind(&mut self, runtime: &mut dyn StateMachineInstanceRuntime) {
        if self.parent.is_null() {
            return;
        }
        let context = unsafe { (&*self.parent).data_context };
        if context == 0 {
            return;
        }
        let value = runtime.data_context_property(context, self.path_owner);
        if value != self.view_model_instance_value {
            self.clear_data_context(runtime);
            if value != 0 {
                self.view_model_instance_value = value;
                runtime.view_model_add_dependent(value, (self as *mut Self) as Object);
            }
        }
    }

    fn add_dirt(&mut self) {
        if !self.parent.is_null() && self.view_model_instance_value != 0 {
            unsafe { (&mut *self.parent).report_to_state_machine(self.view_model_instance_value) };
        }
    }
}

struct ListenerViewModelPropertyBindingListener(ListenerViewModelPropertyBinding);
struct ListenerViewModelPropertyBindingInput(ListenerViewModelPropertyBinding);

enum ListenerViewModelBinding {
    Listener(ListenerViewModelPropertyBindingListener),
    Input(ListenerViewModelPropertyBindingInput),
}

impl ListenerViewModelBinding {
    fn binding(&self) -> &ListenerViewModelPropertyBinding {
        match self {
            Self::Listener(value) => &value.0,
            Self::Input(value) => &value.0,
        }
    }

    fn binding_mut(&mut self) -> &mut ListenerViewModelPropertyBinding {
        match self {
            Self::Listener(value) => &mut value.0,
            Self::Input(value) => &mut value.0,
        }
    }
}

struct ListenerViewModel {
    state_machine_instance: *mut StateMachineInstance,
    listener: Object,
    data_context: Object,
    property_bindings: Vec<ListenerViewModelBinding>,
}

impl ListenerViewModel {
    fn new(machine: *mut StateMachineInstance, listener: Object) -> Self {
        Self {
            state_machine_instance: machine,
            listener,
            data_context: 0,
            property_bindings: Vec::new(),
        }
    }

    fn clear_data_context(&mut self) {
        if self.state_machine_instance.is_null() {
            self.property_bindings.clear();
            return;
        }
        let runtime = unsafe { (&mut *self.state_machine_instance).runtime_mut() };
        for binding in &mut self.property_bindings {
            binding.binding_mut().clear_data_context(runtime);
        }
        self.property_bindings.clear();
    }

    fn bind_from_context(&mut self, context: Object) {
        self.data_context = context;
        self.clear_data_context();
        let runtime = unsafe { (&mut *self.state_machine_instance).runtime_mut() };
        let self_ptr = self as *mut Self;
        if runtime.listener_is_single(self.listener) {
            let value = runtime.data_context_property(context, self.listener);
            if value != 0 {
                self.property_bindings
                    .push(ListenerViewModelBinding::Listener(
                        ListenerViewModelPropertyBindingListener(
                            ListenerViewModelPropertyBinding::new(
                                self_ptr,
                                value,
                                self.listener,
                                runtime,
                            ),
                        ),
                    ));
            }
        } else {
            for input in runtime.listener_view_model_inputs(self.listener) {
                let value = runtime.data_context_property(context, input);
                if value != 0 {
                    self.property_bindings.push(ListenerViewModelBinding::Input(
                        ListenerViewModelPropertyBindingInput(
                            ListenerViewModelPropertyBinding::new(self_ptr, value, input, runtime),
                        ),
                    ));
                }
            }
        }
        let pending: Vec<Object> = self
            .property_bindings
            .iter()
            .map(|binding| binding.binding().view_model_instance_value)
            .filter(|&value| {
                runtime.view_model_value_is_trigger(value)
                    && runtime.view_model_trigger_value(value) != 0
            })
            .collect();
        for value in pending {
            self.report_to_state_machine(value);
        }
    }

    fn report_to_state_machine(&mut self, value: Object) {
        let runtime = unsafe { (&mut *self.state_machine_instance).runtime_mut() };
        if !runtime.view_model_value_is_trigger(value)
            || runtime.view_model_trigger_value(value) != 0
        {
            unsafe {
                (&mut *self.state_machine_instance)
                    .report_listener_view_model((self as *mut Self) as Object)
            };
        }
    }
}

impl Drop for ListenerViewModel {
    fn drop(&mut self) {
        self.clear_data_context();
    }
}

pub struct StateMachineInstance {
    runtime: Box<dyn StateMachineInstanceRuntime>,
    reported_events: Vec<EventReport>,
    reporting_events: Vec<EventReport>,
    events_applied_during_loop: Vec<EventReport>,
    machine: Object,
    artboard_instance: Object,
    needs_advance: bool,
    input_instances: Vec<Option<InputInstance>>,
    layers: Vec<StateMachineLayerInstance>,
    hit_components: Vec<Box<dyn HitComponent>>,
    listener_groups: Vec<Object>,
    parent_state_machine_instance: *mut StateMachineInstance,
    parent_nested_artboard: Object,
    data_context: Object,
    data_binds: Vec<Object>,
    listener_view_models: Vec<Box<ListenerViewModel>>,
    reported_listener_view_models: Vec<Object>,
    reporting_listener_view_models: Vec<Object>,
    bindable_property_instances: HashMap<Object, Object>,
    scripted_objects_map: HashMap<Object, Object>,
    bindable_data_binds_to_target: HashMap<Object, Object>,
    bindable_data_binds_to_source: HashMap<Object, Object>,
    transition_property_instances: HashMap<Object, HashMap<u32, Object>>,
    state_keyframe_data_binds: HashMap<Object, Vec<Object>>,
    draw_order_change_counter: u8,
    focus_manager: Object,
    external_focus_manager: Object,
    focus_listener_groups: Vec<Object>,
    keyboard_listener_groups: Vec<Object>,
    gamepad_listener_groups: Vec<Object>,
    gamepad_scripted_drawables: Vec<Object>,
    embedder_gamepads: HashMap<i32, Object>,
    semantic_manager: Object,
    external_semantic_manager: Object,
    queued_focus_events: Vec<QueuedFocusEvent>,
    semantic_listener_groups: Vec<Object>,
    queued_semantic_events: Vec<QueuedSemanticEvent>,
    nested_event_listeners: Vec<*mut StateMachineInstance>,
    nested_artboard: Object,
    #[cfg(feature = "rive_tools")]
    input_changed_callback: Option<fn(*mut StateMachineInstance, u64)>,
}

impl StateMachineInstance {
    pub fn new(
        machine: Object,
        artboard_instance: Object,
        runtime: Box<dyn StateMachineInstanceRuntime>,
    ) -> Box<Self> {
        let mut instance = Box::new(Self {
            runtime,
            reported_events: Vec::new(),
            reporting_events: Vec::new(),
            events_applied_during_loop: Vec::new(),
            machine,
            artboard_instance,
            needs_advance: false,
            input_instances: Vec::new(),
            layers: Vec::new(),
            hit_components: Vec::new(),
            listener_groups: Vec::new(),
            parent_state_machine_instance: std::ptr::null_mut(),
            parent_nested_artboard: 0,
            data_context: 0,
            data_binds: Vec::new(),
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
            focus_manager: 0,
            external_focus_manager: 0,
            focus_listener_groups: Vec::new(),
            keyboard_listener_groups: Vec::new(),
            gamepad_listener_groups: Vec::new(),
            gamepad_scripted_drawables: Vec::new(),
            embedder_gamepads: HashMap::new(),
            semantic_manager: 0,
            external_semantic_manager: 0,
            queued_focus_events: Vec::new(),
            semantic_listener_groups: Vec::new(),
            queued_semantic_events: Vec::new(),
            nested_event_listeners: Vec::new(),
            nested_artboard: 0,
            #[cfg(feature = "rive_tools")]
            input_changed_callback: None,
        });
        instance.focus_manager = instance.runtime.focus_manager_new();
        let machine_ptr = (&mut *instance as *mut Self).cast();

        let input_count = instance.runtime.machine_input_count(machine);
        instance.input_instances.resize(input_count, None);
        for index in 0..input_count {
            let input = instance.runtime.machine_input(machine, index);
            if input == 0 {
                continue;
            }
            instance.input_instances[index] =
                instance.runtime.make_input_instance(input, machine_ptr);
        }

        let layer_count = instance.runtime.machine_layer_count(machine);
        instance.layers.resize_with(layer_count, Default::default);
        for index in 0..layer_count {
            let layer = instance.runtime.machine_layer(machine, index);
            let ptr = &mut *instance as *mut Self;
            instance.layers[index].init(ptr, layer, artboard_instance);
        }

        instance.initialize_data_binds();
        let mut hit_lookup = HashMap::new();
        instance.initialize_listeners(&mut hit_lookup);
        instance.initialize_component_provided_listeners(&mut hit_lookup);
        instance.initialize_nested_hit_components();
        #[cfg(feature = "rive_text")]
        instance.initialize_text_inputs();
        instance.initialize_scripted_objects();
        instance.sort_hit_components();
        let manager = instance.focus_manager();
        instance
            .runtime
            .artboard_build_focus_tree(artboard_instance, manager, 0);
        instance
    }

    fn runtime_mut(&mut self) -> &mut dyn StateMachineInstanceRuntime {
        self.runtime.as_mut()
    }

    fn initialize_data_binds(&mut self) {
        for index in 0..self.runtime.machine_data_bind_count(self.machine) {
            let source = self.runtime.machine_data_bind(self.machine, index);
            let original_target = self.runtime.data_bind_target(source);
            if original_target == 0 {
                continue;
            }
            let clone = self.runtime.clone_data_bind(source);
            self.add_data_bind(clone);
            if self.runtime.data_bind_bindable_target(source) {
                let property = *self
                    .bindable_property_instances
                    .entry(original_target)
                    .or_insert_with(|| self.runtime.clone_bindable_property(original_target));
                let property_key = self.runtime.data_bind_property_key(clone);
                self.runtime
                    .configure_data_bind_target(clone, property, property_key);
                if self.runtime.data_bind_flags(clone) & 1 != 0 {
                    self.bindable_data_binds_to_source.insert(property, clone);
                } else {
                    self.bindable_data_binds_to_target.insert(property, clone);
                }
            } else if self.runtime.data_bind_is_transition_target(source) {
                let property = self.runtime.make_transition_property();
                self.transition_property_instances
                    .entry(original_target)
                    .or_default()
                    .insert(self.runtime.data_bind_property_key(source), property);
                self.runtime.configure_data_bind_target(
                    clone,
                    property,
                    BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY as u32,
                );
            }
        }
    }

    fn initialize_listeners(&mut self, hit_lookup: &mut HashMap<Object, usize>) {
        let machine_ptr = (self as *mut Self).cast();
        for index in 0..self.runtime.machine_listener_count(self.machine) {
            let listener = self.runtime.machine_listener(self.machine, index);
            if self.runtime.listener_has(listener, ListenerType::Event) {
                continue;
            }
            if self.runtime.listener_has(listener, ListenerType::ViewModel) {
                self.listener_view_models
                    .push(Box::new(ListenerViewModel::new(self, listener)));
                continue;
            }
            let target = self.runtime.artboard_resolve(
                self.artboard_instance,
                self.runtime.listener_target_id(listener),
            );
            if self.runtime.listener_has(listener, ListenerType::Focus)
                || self.runtime.listener_has(listener, ListenerType::Blur)
            {
                let focus_data = self.runtime.resolve_focus_data(target);
                if focus_data != 0 {
                    let group =
                        self.runtime
                            .make_focus_listener_group(focus_data, listener, machine_ptr);
                    self.focus_listener_groups.push(group);
                }
            }
            if self.runtime.listener_has(listener, ListenerType::Keyboard)
                || self.runtime.listener_has(listener, ListenerType::TextInput)
            {
                let focus_data = self.runtime.resolve_focus_data(target);
                if focus_data != 0 {
                    let group = self.runtime.make_keyboard_listener_group(
                        focus_data,
                        listener,
                        machine_ptr,
                    );
                    self.keyboard_listener_groups.push(group);
                }
            }
            if self
                .runtime
                .listener_has(listener, ListenerType::SemanticAction)
            {
                let semantic_data = self.runtime.resolve_semantic_data(target);
                if semantic_data != 0 {
                    let group = self.runtime.make_semantic_listener_group(
                        semantic_data,
                        listener,
                        machine_ptr,
                    );
                    self.semantic_listener_groups.push(group);
                }
            }
            if self
                .runtime
                .listener_has_any(listener, &POINTER_HIT_LISTENER_TYPES)
            {
                let group = self.runtime.make_listener_group(listener);
                if target != 0 {
                    let is_layout = self.runtime.component_is_layout(target);
                    let target = if is_layout {
                        self.runtime.component_proxy(target)
                    } else {
                        target
                    };
                    self.add_to_hit_lookup(target, is_layout, hit_lookup, group, false);
                }
                self.listener_groups.push(group);
            }
            if self.runtime.listener_has(listener, ListenerType::Gamepad) {
                let focus_data = self.runtime.resolve_focus_data(target);
                if focus_data != 0 {
                    let group =
                        self.runtime
                            .make_gamepad_listener_group(focus_data, listener, machine_ptr);
                    self.gamepad_listener_groups.push(group);
                }
            }
        }
    }

    fn initialize_component_provided_listeners(&mut self, hit_lookup: &mut HashMap<Object, usize>) {
        let machine_ptr = (self as *mut Self).cast();
        let providers: Vec<Object> = self
            .runtime
            .artboard_objects(self.artboard_instance)
            .into_iter()
            .map(|object| self.runtime.object_listener_provider(object))
            .filter(|&provider| provider != 0)
            .collect();
        for provider in providers {
            for (group, targets) in self.runtime.listener_groups_from_provider(provider) {
                for &(target, opaque) in &targets {
                    let layout = self.runtime.component_is_layout(target)
                        || self.runtime.component_is_drawable_proxy(target);
                    self.add_to_hit_lookup(target, layout, hit_lookup, group, opaque);
                }
                self.listener_groups.push(group);
            }
            self.hit_components
                .extend(self.runtime.provided_hit_components(provider, machine_ptr));
        }
    }

    fn initialize_nested_hit_components(&mut self) {
        for nested in self
            .runtime
            .artboard_nested_artboards(self.artboard_instance)
        {
            self.hit_components.push(Box::new(HitNestedArtboard {
                component: nested,
                state_machine_instance: self,
            }));
            for animation in self.runtime.nested_animations(nested) {
                if self.runtime.nested_is_state_machine(animation) {
                    let notifier = self.runtime.nested_state_machine_instance(animation);
                    if !notifier.is_null() {
                        unsafe {
                            (&mut *notifier).set_parent_nested_artboard(nested);
                            (&mut *notifier).add_nested_event_listener(self);
                        }
                    }
                } else {
                    self.runtime
                        .nested_add_event_listener(animation, (self as *mut Self).cast());
                }
            }
        }
        for list in self
            .runtime
            .artboard_component_lists(self.artboard_instance)
        {
            self.hit_components.push(Box::new(HitComponentList {
                component: list,
                state_machine_instance: self,
            }));
        }
    }

    #[cfg(feature = "rive_text")]
    fn initialize_text_inputs(&mut self) {
        let machine_ptr = (self as *mut Self).cast();
        for text_input in self.runtime.artboard_text_inputs(self.artboard_instance) {
            let group = self
                .runtime
                .make_text_input_listener_group(text_input, machine_ptr);
            let mut hit = HitDrawable::new(
                self.runtime.as_ref(),
                text_input,
                text_input,
                self,
                true,
                true,
                true,
            );
            hit.add_listener(self.runtime.as_ref(), group);
            self.hit_components.push(Box::new(hit));
            self.listener_groups.push(group);
        }
    }

    fn initialize_scripted_objects(&mut self) {
        let machine_ptr = (self as *mut Self).cast();
        for source in self.runtime.machine_scripted_objects(self.machine) {
            let clone = self.runtime.scripted_clone(source, machine_ptr);
            self.scripted_objects_map.insert(source, clone);
        }
        for &object in self.scripted_objects_map.values() {
            self.runtime
                .scripted_set_data_context(object, self.data_context);
        }
        self.init_scripted_objects();
        for object in self.runtime.artboard_objects(self.artboard_instance) {
            let scripted = self.runtime.object_scripted(object);
            if scripted == 0 {
                continue;
            }
            if self.runtime.scripted_wants_keyboard(scripted)
                || self.runtime.scripted_wants_text(scripted)
            {
                let focus_data = self.runtime.resolve_focus_data(object);
                if focus_data != 0 {
                    let group =
                        self.runtime
                            .make_keyboard_listener_group(focus_data, 0, machine_ptr);
                    self.keyboard_listener_groups.push(group);
                }
            }
            if self.runtime.scripted_wants_gamepad(scripted) {
                self.gamepad_scripted_drawables.push(object);
            }
        }
    }

    fn add_to_hit_lookup(
        &mut self,
        target: Object,
        is_layout_component: bool,
        hit_lookup: &mut HashMap<Object, usize>,
        listener_group: Object,
        is_opaque: bool,
    ) {
        if is_layout_component {
            let index = if let Some(&index) = hit_lookup.get(&target) {
                index
            } else {
                let hit = HitDrawable::new(
                    self.runtime.as_ref(),
                    target,
                    target,
                    self,
                    is_opaque,
                    false,
                    true,
                );
                let index = self.hit_components.len();
                self.hit_components.push(Box::new(hit));
                hit_lookup.insert(target, index);
                index
            };
            let drawable = self.hit_components[index].as_mut().as_any_hit_drawable();
            if let Some(drawable) = drawable {
                drawable.add_listener(self.runtime.as_ref(), listener_group);
                drawable.is_opaque |= is_opaque;
            }
            return;
        }
        if self.runtime.component_is_shape(target) || self.runtime.component_is_text_run(target) {
            let index = if let Some(&index) = hit_lookup.get(&target) {
                index
            } else {
                self.runtime.component_mark_hit_path(target);
                let drawable = if self.runtime.component_is_text_run(target) {
                    self.runtime.text_run_text_component(target)
                } else {
                    target
                };
                let hit = HitDrawable::new(
                    self.runtime.as_ref(),
                    drawable,
                    target,
                    self,
                    false,
                    true,
                    true,
                );
                let index = self.hit_components.len();
                self.hit_components.push(Box::new(hit));
                hit_lookup.insert(target, index);
                index
            };
            if let Some(drawable) = self.hit_components[index].as_mut().as_any_hit_drawable() {
                drawable.add_listener(self.runtime.as_ref(), listener_group);
            }
            return;
        }
        if self.runtime.component_is_container(target) {
            for child in self.runtime.component_children(target) {
                self.add_to_hit_lookup(
                    child,
                    self.runtime.component_is_layout(child),
                    hit_lookup,
                    listener_group,
                    is_opaque,
                );
            }
        }
    }

    fn normalize_pointer_position(&self, mut position: Vec2D) -> Vec2D {
        if self.runtime.artboard_frame_origin(self.artboard_instance) {
            let origin = self.runtime.artboard_origin(self.artboard_instance);
            let size = self.runtime.artboard_layout_size(self.artboard_instance);
            position = Vec2D::new(
                position.x - origin.x * size.x,
                position.y - origin.y * size.y,
            );
        }
        self.runtime
            .artboard_inverse_self_transform(self.artboard_instance, position)
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
        for &group in &self.listener_groups {
            self.runtime.listener_group_reset(group, pointer_id);
        }
        for component in &mut self.hit_components {
            component.prepare_event(self.runtime.as_mut(), position, hit_type, pointer_id);
        }
        let mut hit_something = false;
        let mut hit_opaque = false;
        for component in &mut self.hit_components {
            let result = component.process_event(
                self.runtime.as_mut(),
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
        if hit_type == ListenerType::Exit {
            for &group in &self.listener_groups {
                self.runtime.listener_group_release(group, pointer_id);
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
            .any(|component| component.hit_test(self.runtime.as_ref(), position))
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
        let components: Vec<Object> = self
            .hit_components
            .iter()
            .map(|component| component.component())
            .collect();
        let order = self
            .runtime
            .artboard_ordered_hit_components(self.artboard_instance, &components);
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
        let machine = (self as *mut Self).cast();
        self.runtime.artboard_update_data_binds(machine, false);
        let mut changed = false;
        for layer in &mut self.layers {
            changed |= layer.update_state();
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
            let machine = (self as *mut Self).cast();
            self.runtime.artboard_update_data_binds(machine, false);
            self.reporting_events = std::mem::take(&mut self.reported_events);
            self.reporting_listener_view_models =
                std::mem::take(&mut self.reported_listener_view_models);
            if iteration > 1 {
                self.events_applied_during_loop
                    .extend(self.reporting_events.iter().copied());
            }
            let events = self.reporting_events.clone();
            let view_models = self.reporting_listener_view_models.clone();
            self.notify_event_listeners(&events, 0);
            self.notify_listener_view_models(&view_models);
        }
        if iteration >= 100 {
            eprintln!(
                "{} StateMachine exceeded max event iterations on artboard {}",
                self.name(),
                self.runtime.artboard_name(self.artboard_instance)
            );
        }
    }

    pub fn set_external_focus_manager(&mut self, manager: Object) {
        if self.external_focus_manager == manager {
            return;
        }
        if self.artboard_instance != 0
            && self.runtime.artboard_focus_manager(self.artboard_instance) != 0
        {
            self.runtime
                .artboard_cleanup_focus_tree(self.artboard_instance);
        }
        self.external_focus_manager = manager;
        if self.artboard_instance != 0 {
            let focus_manager = self.focus_manager();
            self.runtime
                .artboard_build_focus_tree(self.artboard_instance, focus_manager, 0);
        }
    }

    pub fn focus_manager(&self) -> Object {
        if self.external_focus_manager != 0 {
            self.external_focus_manager
        } else {
            self.focus_manager
        }
    }

    pub fn internal_focus_manager(&self) -> Object {
        self.focus_manager
    }

    pub fn has_external_focus_manager(&self) -> bool {
        self.external_focus_manager != 0
    }

    pub fn enable_semantics(&mut self) {
        if self.semantic_manager() != 0 {
            return;
        }
        self.semantic_manager = self.runtime.semantic_manager_new();
        if self.artboard_instance != 0 {
            let manager = self.semantic_manager();
            self.runtime
                .artboard_build_semantic_tree(self.artboard_instance, manager, 0);
        }
    }

    pub fn semantic_manager(&self) -> Object {
        if self.external_semantic_manager != 0 {
            self.external_semantic_manager
        } else {
            self.semantic_manager
        }
    }

    pub fn set_external_semantic_manager(&mut self, manager: Object, parent_node: Object) {
        if self.external_semantic_manager == manager {
            return;
        }
        if self.artboard_instance != 0
            && self
                .runtime
                .artboard_semantic_manager(self.artboard_instance)
                != 0
        {
            self.runtime
                .artboard_cleanup_semantic_tree(self.artboard_instance);
        }
        self.external_semantic_manager = manager;
        if self.artboard_instance != 0 {
            let manager = self.semantic_manager();
            self.runtime
                .artboard_build_semantic_tree(self.artboard_instance, manager, parent_node);
        }
    }

    pub fn queue_focus_event(&mut self, group: Object, is_focus: bool) {
        self.queued_focus_events
            .push(QueuedFocusEvent { group, is_focus });
        self.needs_advance = true;
    }

    pub fn set_focus(&mut self, focus_data: Object) {
        let manager = self.focus_manager();
        if focus_data != 0 {
            self.runtime.focus_manager_set_focus(manager, focus_data);
        } else {
            self.runtime.focus_manager_clear(manager);
        }
    }

    pub fn focus_state(&self) -> FocusState {
        self.runtime.focus_manager_state(self.focus_manager())
    }

    fn process_focus_events(&mut self) {
        let events = std::mem::take(&mut self.queued_focus_events);
        for event in events {
            let listener = self.runtime.listener_for_focus_group(event.group);
            if (event.is_focus && self.runtime.listener_has(listener, ListenerType::Focus))
                || (!event.is_focus && self.runtime.listener_has(listener, ListenerType::Blur))
            {
                let invocation = self
                    .runtime
                    .listener_invocation_focus(event.group, event.is_focus);
                self.runtime.listener_perform_changes(
                    listener,
                    (self as *mut Self).cast(),
                    invocation,
                );
            }
        }
    }

    pub fn queue_semantic_event(&mut self, group: Object, action_type: u8) {
        self.queued_semantic_events
            .push(QueuedSemanticEvent { group, action_type });
        self.needs_advance = true;
    }

    fn process_semantic_events(&mut self) {
        let events = std::mem::take(&mut self.queued_semantic_events);
        for event in events {
            if event.group == 0 {
                continue;
            }
            let listener = self.runtime.listener_for_semantic_group(event.group);
            if listener == 0 {
                continue;
            }
            let invocation = self
                .runtime
                .listener_invocation_semantic(event.group, event.action_type);
            self.runtime
                .listener_perform_changes(listener, (self as *mut Self).cast(), invocation);
        }
    }

    pub fn fire_semantic_action(&mut self, node_id: u32, action_type: u8) {
        let manager = self.semantic_manager();
        if manager != 0 {
            self.runtime
                .semantic_fire_action(manager, node_id, action_type);
        }
    }

    pub fn advance(&mut self, seconds: f32, new_frame: bool) -> bool {
        let counter = self
            .runtime
            .artboard_draw_order_change_counter(self.artboard_instance);
        if self.draw_order_change_counter != counter {
            self.draw_order_change_counter = counter;
            self.sort_hit_components();
        }
        if new_frame {
            self.process_focus_events();
            self.process_semantic_events();
            self.apply_events();
            self.needs_advance = false;
        }
        let machine = (self as *mut Self).cast();
        self.runtime.artboard_update_data_binds(machine, false);
        for layer in &mut self.layers {
            if layer.advance(seconds, new_frame) {
                self.needs_advance = true;
            }
        }
        if self.runtime.artboard_advance_data_binds(machine, seconds) {
            self.needs_advance = true;
        }
        for input in self.input_instances.iter().flatten().copied() {
            self.runtime.input_advanced(input);
        }
        self.needs_advance
            || !self.reported_events.is_empty()
            || !self.reported_listener_view_models.is_empty()
    }

    pub fn advance_seconds(&mut self, seconds: f32) -> bool {
        self.advance(seconds, true)
    }

    pub fn advanced_data_context(&mut self) {
        if self.data_context != 0 {
            self.runtime.data_context_advanced(self.data_context);
        }
    }

    pub fn reset(&mut self) {
        self.advanced_data_context();
        self.runtime.artboard_reset(self.artboard_instance);
    }

    pub fn advance_and_apply(&mut self, seconds: f32) -> bool {
        self.advance_and_apply_view_models(seconds, true)
    }

    pub fn advance_and_apply_view_models(
        &mut self,
        seconds: f32,
        advance_view_models: bool,
    ) -> bool {
        const IS_ROOT: u32 = 1;
        const ANIMATE: u32 = 2;
        const ADVANCE_NESTED: u32 = 4;
        const NEW_FRAME: u32 = 8;
        let mut keep_going = self.advance(seconds, true) || seconds == 0.0;
        let manager = self.focus_manager();
        self.runtime.focus_manager_drop_hidden(manager);
        if self.runtime.artboard_advance_internal(
            self.artboard_instance,
            seconds,
            IS_ROOT | ANIMATE | ADVANCE_NESTED | NEW_FRAME,
        ) {
            keep_going = true;
        }
        for _ in 0..5 {
            if self
                .runtime
                .artboard_update_pass(self.artboard_instance, true)
            {
                keep_going = true;
            }
            if self.try_change_state() {
                self.advance(0.0, false);
                keep_going = true;
            }
            if self.runtime.artboard_advance_internal(
                self.artboard_instance,
                0.0,
                IS_ROOT | ANIMATE | ADVANCE_NESTED,
            ) {
                keep_going = true;
            }
            if advance_view_models {
                self.reset();
            } else {
                self.runtime.artboard_reset(self.artboard_instance);
            }
            if !self
                .runtime
                .artboard_has_component_dirt(self.artboard_instance)
            {
                break;
            }
        }
        if advance_view_models {
            self.runtime
                .artboard_advance_scripted_view_models(self.artboard_instance);
        }
        keep_going
            || !self.reported_events.is_empty()
            || !self.reported_listener_view_models.is_empty()
    }

    pub fn mark_needs_advance(&mut self) {
        self.needs_advance = true;
    }

    pub fn needs_advance(&self) -> bool {
        self.needs_advance
    }

    pub fn reset_state(&mut self) {
        for layer in &mut self.layers {
            layer.reset_state();
        }
    }

    pub fn name(&self) -> String {
        self.runtime.machine_name(self.machine)
    }

    pub fn state_machine(&self) -> Object {
        self.machine
    }

    pub fn artboard(&self) -> Object {
        self.artboard_instance
    }

    pub fn input_count(&self) -> usize {
        self.input_instances.len()
    }

    pub fn input(&self, index: usize) -> Option<*mut SMIInput> {
        self.input_instances
            .get(index)
            .copied()
            .flatten()
            .map(InputInstance::base)
    }

    pub fn get_bool(&self, name: &str) -> Option<*mut SMIBool> {
        self.input_instances.iter().flatten().find_map(|instance| {
            let InputInstance::Bool(value) = *instance else {
                return None;
            };
            (unsafe { (&*value).base.name() } == name).then_some(value)
        })
    }

    pub fn get_number(&self, name: &str) -> Option<*mut SMINumber> {
        self.input_instances.iter().flatten().find_map(|instance| {
            let InputInstance::Number(value) = *instance else {
                return None;
            };
            (unsafe { (&*value).base.name() } == name).then_some(value)
        })
    }

    pub fn get_trigger(&self, name: &str) -> Option<*mut SMITrigger> {
        self.input_instances.iter().flatten().find_map(|instance| {
            let InputInstance::Trigger(value) = *instance else {
                return None;
            };
            (unsafe { (&*value).base.name() } == name).then_some(value)
        })
    }

    pub fn set_parent_state_machine_instance(&mut self, instance: *mut StateMachineInstance) {
        self.parent_state_machine_instance = instance;
    }

    pub fn parent_state_machine_instance(&self) -> *mut StateMachineInstance {
        self.parent_state_machine_instance
    }

    pub fn set_parent_nested_artboard(&mut self, artboard: Object) {
        self.parent_nested_artboard = artboard;
    }

    pub fn parent_nested_artboard(&self) -> Object {
        self.parent_nested_artboard
    }

    pub fn add_nested_event_listener(&mut self, listener: *mut StateMachineInstance) {
        if !self.nested_event_listeners.contains(&listener) {
            self.nested_event_listeners.push(listener);
        }
    }

    pub fn remove_nested_event_listener(&mut self, listener: *mut StateMachineInstance) {
        self.nested_event_listeners
            .retain(|&candidate| candidate != listener);
    }

    pub fn set_nested_artboard(&mut self, artboard: Object) {
        self.nested_artboard = artboard;
    }

    pub fn report_event(&mut self, event: Object, seconds_delay: f32) {
        self.reported_events.push(EventReport {
            event,
            seconds_delay,
        });
    }

    fn report_listener_view_model(&mut self, listener: Object) {
        self.reported_listener_view_models.push(listener);
    }

    pub fn reported_event_count(&self) -> usize {
        self.events_applied_during_loop.len() + self.reported_events.len()
    }

    pub fn reported_event_at(&self, mut index: usize) -> EventReport {
        if index < self.events_applied_during_loop.len() {
            return self.events_applied_during_loop[index];
        }
        index -= self.events_applied_during_loop.len();
        self.reported_events.get(index).copied().unwrap_or_default()
    }

    pub fn notify(&mut self, events: &[EventReport], context: Object) {
        self.notify_event_listeners(events, context);
        let machine = (self as *mut Self).cast();
        self.runtime.artboard_update_data_binds(machine, false);
    }

    fn notify_listener_view_models(&mut self, events: &[Object]) {
        for &view_model in events {
            if view_model == 0 {
                continue;
            }
            let listener = unsafe { (*(view_model as *mut ListenerViewModel)).listener };
            let invocation = self.runtime.listener_invocation_view_model(view_model);
            self.runtime
                .listener_perform_changes(listener, (self as *mut Self).cast(), invocation);
        }
    }

    fn notify_event_listeners(&mut self, events: &[EventReport], source: Object) {
        if events.is_empty() {
            return;
        }
        for index in 0..self.runtime.machine_listener_count(self.machine) {
            let listener = self.runtime.machine_listener(self.machine, index);
            if listener == 0 || !self.runtime.listener_has(listener, ListenerType::Event) {
                continue;
            }
            let target = self.runtime.artboard_resolve(
                self.artboard_instance,
                self.runtime.listener_target_id(listener),
            );
            if source != 0 && source != target {
                continue;
            }
            let source_artboard = if source == 0 {
                self.artboard_instance
            } else {
                source
            };
            for report in events {
                if source == 0 {
                    let resolved_target = self.runtime.artboard_resolve(
                        source_artboard,
                        self.runtime.listener_target_id(listener),
                    );
                    if resolved_target != 0
                        && resolved_target != self.artboard_instance
                        && resolved_target != report.event
                    {
                        continue;
                    }
                }
                for event_id in self.runtime.listener_event_ids(listener) {
                    if self.runtime.artboard_resolve(source_artboard, event_id) == report.event {
                        let invocation = self
                            .runtime
                            .listener_invocation_event(report.event, report.seconds_delay);
                        self.runtime.listener_perform_changes(
                            listener,
                            (self as *mut Self).cast(),
                            invocation,
                        );
                        break;
                    }
                }
            }
        }
        let listeners = self.nested_event_listeners.clone();
        let nested_artboard = self.nested_artboard;
        for listener in listeners {
            if !listener.is_null() {
                unsafe { (&mut *listener).notify(events, nested_artboard) };
            }
        }
        for report in events {
            if self.runtime.event_is_audio(report.event) {
                self.runtime.event_play_audio(report.event);
            }
        }
    }

    pub fn current_animation_count(&mut self) -> usize {
        self.layers
            .iter_mut()
            .filter(|layer| layer.current_animation() != 0)
            .count()
    }

    pub fn current_animation_by_index(&mut self, index: usize) -> Object {
        self.layers
            .iter_mut()
            .filter_map(|layer| {
                let animation = layer.current_animation();
                (animation != 0).then_some(animation)
            })
            .nth(index)
            .unwrap_or(0)
    }

    pub fn state_changed_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| layer.state_machine_changed_on_advance)
            .count()
    }

    pub fn state_changed_by_index(&mut self, index: usize) -> Object {
        let mut count = 0;
        for layer in &mut self.layers {
            if layer.state_machine_changed_on_advance {
                if count == index {
                    return layer.current_state();
                }
                count += 1;
            }
        }
        0
    }

    pub fn enable_pointer_events(&mut self, pointer_id: i32) {
        for component in &mut self.hit_components {
            component.enable_pointer_events(self.runtime.as_mut(), pointer_id);
        }
    }

    pub fn disable_pointer_events(&mut self, pointer_id: i32) {
        for component in &mut self.hit_components {
            component.disable_pointer_events(self.runtime.as_mut(), pointer_id);
        }
    }

    pub fn has_listeners(&self) -> bool {
        !self.hit_components.is_empty()
    }

    pub fn has_focus_nodes(&self) -> bool {
        self.runtime.focus_manager_has_content(self.focus_manager())
    }

    pub fn focus_next(&mut self) -> bool {
        let manager = self.focus_manager();
        self.runtime.focus_manager_next(manager)
    }

    pub fn focus_previous(&mut self) -> bool {
        let manager = self.focus_manager();
        self.runtime.focus_manager_previous(manager)
    }

    pub fn clear_focus(&mut self) {
        let manager = self.focus_manager();
        self.runtime.focus_manager_clear(manager);
    }

    pub fn submit_gamepads_from_buffer(&mut self, data: &[u8]) -> bool {
        self.runtime
            .gamepad_submit_buffer((self as *mut Self).cast(), data)
    }

    pub fn broadcast_gamepad_to_scripted_drawables(
        &mut self,
        invocation: Object,
        already_dispatched: Object,
    ) -> HitResult {
        self.runtime
            .gamepad_broadcast((self as *mut Self).cast(), invocation, already_dispatched)
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

    pub fn set_view_model_instance(&mut self, view_model_instance: Object) {
        if view_model_instance == 0 {
            return;
        }
        if self.data_context == 0 {
            self.data_context = self.runtime.data_context_new(view_model_instance);
            self.runtime
                .data_context_add_container(self.data_context, (self as *mut Self).cast());
            return;
        }
        self.runtime
            .data_context_set_main(self.data_context, view_model_instance);
    }

    pub fn set_global_view_model_instance(
        &mut self,
        name: &str,
        view_model_instance: Object,
    ) -> bool {
        let file = self.runtime.artboard_file(self.artboard_instance);
        if file == 0 {
            return false;
        }
        let Some(slot) = self.runtime.view_model_slot(file, name) else {
            return false;
        };
        if !self.runtime.view_model_is_global(file, slot) {
            return false;
        }
        if self.data_context == 0 {
            if view_model_instance == 0 {
                return true;
            }
            self.data_context = self.runtime.data_context_new(0);
            self.runtime
                .data_context_add_container(self.data_context, (self as *mut Self).cast());
        }
        self.runtime
            .data_context_set_slot(self.data_context, slot, view_model_instance);
        true
    }

    pub fn bind(&mut self) {
        if self.data_context == 0 {
            self.data_context = self.runtime.data_context_new(0);
            self.runtime
                .data_context_add_container(self.data_context, (self as *mut Self).cast());
        }
        self.complete_view_model_instances();
        self.runtime
            .artboard_set_data_context(self.artboard_instance, self.data_context);
        self.internal_data_context(self.data_context);
    }

    fn complete_view_model_instances(&mut self) {
        let file = self.runtime.artboard_file(self.artboard_instance);
        if file == 0 {
            return;
        }
        if self.runtime.data_context_main(self.data_context) == 0 {
            let main = self.runtime.complete_default_main(self.artboard_instance);
            if main != 0 {
                self.runtime.data_context_set_main(self.data_context, main);
            }
        }
        for view_model in self.runtime.global_view_models(file) {
            let name = self.runtime.view_model_name(view_model);
            let Some(slot) = self.runtime.view_model_slot(file, &name) else {
                continue;
            };
            if self.runtime.data_context_slot(self.data_context, slot) != 0 {
                continue;
            }
            let instance = self.runtime.create_default_view_model(file, view_model);
            if instance != 0 {
                self.runtime
                    .data_context_set_slot(self.data_context, slot, instance);
            }
        }
    }

    pub fn bind_view_model_instance(&mut self, view_model_instance: Object) {
        if view_model_instance == 0 {
            self.clear_data_context();
            self.runtime
                .artboard_clear_data_context(self.artboard_instance);
            return;
        }
        self.set_view_model_instance(view_model_instance);
        self.bind();
    }

    pub fn global_view_model_instance(&self, name: &str) -> Object {
        if self.data_context == 0 {
            return 0;
        }
        let file = self.runtime.artboard_file(self.artboard_instance);
        if file == 0 {
            return 0;
        }
        self.runtime
            .view_model_slot(file, name)
            .map(|slot| self.runtime.data_context_slot(self.data_context, slot))
            .unwrap_or(0)
    }

    pub fn bind_data_context(&mut self, data_context: Object) {
        self.clear_data_context();
        self.runtime
            .data_context_add_container(data_context, (self as *mut Self).cast());
        self.runtime
            .artboard_clear_data_context(self.artboard_instance);
        self.runtime
            .artboard_set_data_context(self.artboard_instance, data_context);
        self.internal_data_context(data_context);
    }

    pub fn inherit_data_context(&mut self, data_context: Object) {
        if data_context == 0 {
            return;
        }
        self.runtime
            .data_context_add_container(data_context, (self as *mut Self).cast());
        self.internal_data_context(data_context);
    }

    pub fn set_data_context(&mut self, data_context: Object) {
        self.clear_data_context();
        self.internal_data_context(data_context);
    }

    pub fn data_context(&self) -> Object {
        self.data_context
    }

    fn init_scripted_objects(&mut self) {
        for &object in self.scripted_objects_map.values() {
            self.runtime.scripted_initialize(object);
            self.runtime.scripted_hydrate_inputs(object);
        }
    }

    fn internal_data_context(&mut self, data_context: Object) {
        self.data_context = data_context;
        self.runtime
            .bind_data_binds_from_context((self as *mut Self).cast(), data_context);
        for listener in &mut self.listener_view_models {
            listener.bind_from_context(data_context);
        }
        for &object in self.scripted_objects_map.values() {
            self.runtime.scripted_set_data_context(object, data_context);
        }
        self.init_scripted_objects();
    }

    pub fn rebind(&mut self) {
        self.runtime
            .artboard_clear_data_context(self.artboard_instance);
        self.runtime
            .artboard_set_data_context(self.artboard_instance, self.data_context);
        self.internal_data_context(self.data_context);
    }

    pub fn clear_data_context(&mut self) {
        if self.data_context != 0 {
            self.runtime
                .data_context_remove_container(self.data_context, (self as *mut Self).cast());
            self.data_context = 0;
        }
        for listener in &mut self.listener_view_models {
            listener.clear_data_context();
        }
    }

    pub fn relink_data_context(&mut self) {
        self.runtime
            .artboard_relink_data_context(self.artboard_instance);
        for listener in &mut self.listener_view_models {
            for binding in &mut listener.property_bindings {
                binding
                    .binding_mut()
                    .relink_data_bind(self.runtime.as_mut());
            }
        }
    }

    pub fn rebuild_data_bind(&mut self, data_bind: Object) {
        if data_bind != 0 && self.data_context != 0 {
            self.runtime
                .bind_data_binds_from_context((self as *mut Self).cast(), self.data_context);
        }
    }

    fn unbind(&mut self) {
        self.clear_data_context();
        self.runtime.unbind_data_binds((self as *mut Self).cast());
    }

    fn add_data_bind(&mut self, data_bind: Object) {
        self.runtime
            .add_data_bind((self as *mut Self).cast(), data_bind);
        self.data_binds.push(data_bind);
    }

    pub fn bindable_property_instance(&self, property: Object) -> Object {
        self.bindable_property_instances
            .get(&property)
            .copied()
            .unwrap_or(0)
    }

    pub fn bindable_data_bind_to_source(&self, property: Object) -> Object {
        self.bindable_data_binds_to_source
            .get(&property)
            .copied()
            .unwrap_or(0)
    }

    pub fn bindable_data_bind_to_target(&self, property: Object) -> Object {
        self.bindable_data_binds_to_target
            .get(&property)
            .copied()
            .unwrap_or(0)
    }

    pub fn find_transition_property_instance(
        &self,
        transition: Object,
        property_key: u32,
    ) -> Object {
        self.transition_property_instances
            .get(&transition)
            .and_then(|properties| properties.get(&property_key))
            .copied()
            .unwrap_or(0)
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

    pub fn build_state_keyframe_binds(&mut self, state_instance: Object) {
        if state_instance == 0 || self.artboard_instance == 0 {
            return;
        }
        let mut first_bind_by_target = HashMap::new();
        for data_bind in self
            .runtime
            .artboard_source_data_binds(self.artboard_instance)
        {
            let target = self.runtime.data_bind_target(data_bind);
            if target != 0 && self.runtime.data_bind_is_keyframe_target(data_bind) {
                first_bind_by_target.entry(target).or_insert(data_bind);
            }
        }
        if first_bind_by_target.is_empty() {
            return;
        }
        let machine = self as *mut Self;
        self.runtime.state_for_each_animation_instance(
            state_instance,
            &mut |runtime, animation_instance| {
                for keyframe in runtime.animation_keyframes(animation_instance) {
                    let keyframe_type = runtime.keyframe_type(keyframe);
                    let holder_property_key = Self::keyframe_holder_property_key(keyframe_type);
                    if holder_property_key == 0 {
                        continue;
                    }
                    let Some(&source_bind) = first_bind_by_target.get(&keyframe) else {
                        continue;
                    };
                    let holder = runtime.make_keyframe_holder(keyframe_type);
                    runtime.add_keyframe_holder(animation_instance, keyframe, holder);
                    let clone = runtime.clone_data_bind(source_bind);
                    runtime.configure_data_bind_target(clone, holder, holder_property_key);
                    runtime.add_data_bind(machine.cast(), clone);
                    unsafe {
                        (&mut *machine).data_binds.push(clone);
                        (&mut *machine)
                            .state_keyframe_data_binds
                            .entry(state_instance)
                            .or_default()
                            .push(clone);
                    }
                }
            },
        );
    }

    pub fn remove_state_keyframe_binds(&mut self, state_instance: Object) {
        let Some(data_binds) = self.state_keyframe_data_binds.remove(&state_instance) else {
            return;
        };
        for data_bind in data_binds {
            self.runtime
                .remove_data_bind((self as *mut Self).cast(), data_bind);
            self.data_binds.retain(|&candidate| candidate != data_bind);
            self.runtime.delete_data_bind(data_bind);
        }
    }

    pub fn scripted_object(&self, source: Object) -> Object {
        self.scripted_objects_map.get(&source).copied().unwrap_or(0)
    }

    pub fn dispose(&mut self) {
        self.remove_event_listeners();
    }

    fn random_value(&mut self) -> f64 {
        self.runtime.random_value()
    }

    fn find_random_transition(&mut self, state_from: Object, layer_index: usize) -> Object {
        if layer_index >= self.layers.len() {
            return 0;
        }
        self.layers[layer_index].find_random_transition(state_from)
    }

    fn find_allowed_transition(&mut self, state_from: Object, layer_index: usize) -> Object {
        if layer_index >= self.layers.len() {
            return 0;
        }
        self.layers[layer_index].find_allowed_transition(state_from)
    }

    #[cfg(feature = "testing")]
    pub fn hit_components_count(&self) -> usize {
        self.hit_components.len()
    }

    #[cfg(feature = "testing")]
    pub fn hit_component(&self, index: usize) -> Option<&dyn HitComponent> {
        self.hit_components.get(index).map(Box::as_ref)
    }

    #[cfg(feature = "testing")]
    pub fn layer_state(&mut self, index: usize) -> Object {
        self.layers
            .get_mut(index)
            .map(StateMachineLayerInstance::current_state)
            .unwrap_or(0)
    }

    fn remove_event_listeners(&mut self) {
        for nested in self
            .runtime
            .artboard_nested_artboards(self.artboard_instance)
        {
            if nested == 0 {
                continue;
            }
            for animation in self.runtime.nested_animations(nested) {
                if animation != 0 {
                    self.runtime
                        .nested_remove_event_listener(animation, (self as *mut Self).cast());
                }
            }
        }
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_input_changed(&mut self, callback: Option<fn(*mut StateMachineInstance, u64)>) {
        self.input_changed_callback = callback;
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_data_bind_changed(&mut self, callback: fn()) {
        for &data_bind in &self.data_binds {
            self.runtime.data_bind_on_changed(data_bind, callback);
        }
    }
}

impl InputInstanceMachine for StateMachineInstance {
    fn mark_needs_advance(&mut self) {
        StateMachineInstance::mark_needs_advance(self);
    }

    #[cfg(feature = "rive_tools")]
    fn input_changed(&mut self, index: u64) {
        if let Some(callback) = self.input_changed_callback {
            callback(self, index);
        }
    }
}

impl Drop for StateMachineInstance {
    fn drop(&mut self) {
        if self.external_focus_manager == 0 && self.artboard_instance != 0 {
            self.runtime
                .artboard_cleanup_focus_tree(self.artboard_instance);
        }
        if self.external_semantic_manager == 0
            && self.semantic_manager != 0
            && self.artboard_instance != 0
        {
            self.runtime
                .artboard_cleanup_semantic_tree(self.artboard_instance);
        }
        self.embedder_gamepads.clear();
        self.unbind();
        for input in self.input_instances.drain(..).flatten() {
            unsafe {
                match input {
                    InputInstance::Bool(value) => drop(Box::from_raw(value)),
                    InputInstance::Number(value) => drop(Box::from_raw(value)),
                    InputInstance::Trigger(value) => drop(Box::from_raw(value)),
                }
            }
        }
        for group in self.listener_groups.drain(..) {
            self.runtime.delete_owned_object(group);
        }
        self.runtime
            .delete_all_data_binds((self as *mut Self).cast());
        self.data_binds.clear();
        self.state_keyframe_data_binds.clear();
        self.layers.clear();
        for (_, property) in self.bindable_property_instances.drain() {
            self.runtime.delete_owned_object(property);
        }
        for (_, properties) in self.transition_property_instances.drain() {
            for (_, property) in properties {
                self.runtime.delete_owned_object(property);
            }
        }
        self.listener_view_models.clear();
        for (_, object) in self.scripted_objects_map.drain() {
            self.runtime.scripted_delete(object);
        }
    }
}

trait HitDrawableDowncast {
    fn as_any_hit_drawable(&mut self) -> Option<&mut HitDrawable>;
}

impl HitDrawableDowncast for dyn HitComponent {
    fn as_any_hit_drawable(&mut self) -> Option<&mut HitDrawable> {
        let pointer = self as *mut dyn HitComponent as *mut HitDrawable;
        Some(unsafe { &mut *pointer })
    }
}
