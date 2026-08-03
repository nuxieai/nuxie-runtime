use nuxie_binary::RuntimeFile;
use nuxie_runtime::*;
use nuxie_runtime::{
    ArtboardInstance, FocusState, RuntimeEventProperty, RuntimeGeometryHit,
    RuntimeGeometryHitOccurrence, RuntimeGeometryHitPathSegment,
    RuntimeImportedViewModelInstanceContext, RuntimeOwnedViewModelContext,
    RuntimeOwnedViewModelContextHandle, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    RuntimeStateMachine, ScriptArtboardDataContext, ScriptError, ScriptHost, ScriptInstance,
    ScriptListenerActionDefinition, ScriptListenerActionHydration, ScriptListenerInputSnapshot,
    ScriptListenerInvocation, ScriptValue, StateMachineEventContext,
    StateMachineEventStringProperty, StateMachineInputInstance, StateMachineInstance,
    StateMachineReportedEvent,
};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

macro_rules! public_methods_are_reachable {
    ($owner:ty; $($method:ident),+ $(,)?) => {
        $(let _ = <$owner>::$method;)+
    };
}

macro_rules! exact_public_signature {
    ($method:path => $signature:ty) => {
        let _: $signature = $method;
    };
}

#[test]
fn fl_c5_public_reexports_are_downstream_visible_after_file_split() {
    // This is the compile-time FL-C5 public-signature inventory. Do not
    // replace these exact coercions with untyped method-item references:
    // parameter, return, receiver, and ownership changes must break this
    // downstream test.
    // BEGIN exhaustive W4 exact signatures
    fn definition_global_id_is_exact(machine: RuntimeStateMachine) -> u32 {
        machine.global_id
    }
    fn definition_name_is_exact(machine: RuntimeStateMachine) -> Option<Arc<str>> {
        machine.name
    }
    fn definition_inputs_are_exact(
        machine: RuntimeStateMachine,
    ) -> Arc<Vec<Option<RuntimeStateMachineInput>>> {
        machine.inputs
    }
    fn definition_layers_are_exact(
        machine: RuntimeStateMachine,
    ) -> Arc<Vec<RuntimeStateMachineLayer>> {
        machine.layers
    }
    let _ = definition_global_id_is_exact as fn(RuntimeStateMachine) -> u32;
    let _ = definition_name_is_exact as fn(RuntimeStateMachine) -> Option<Arc<str>>;
    let _ = definition_inputs_are_exact
        as fn(RuntimeStateMachine) -> Arc<Vec<Option<RuntimeStateMachineInput>>>;
    let _ = definition_layers_are_exact
        as fn(RuntimeStateMachine) -> Arc<Vec<RuntimeStateMachineLayer>>;
    fn is_clone<T: Clone>() {}
    is_clone::<StateMachineInstance>();
    let focus = FocusState::default();
    let _ = (focus.has_focus, focus.expects_keyboard_input);

    exact_public_signature!(
        RuntimeStateMachine::scripted_objects
            => for<'a> fn(&'a RuntimeStateMachine) -> &'a [ScriptListenerActionDefinition]
    );
    exact_public_signature!(
        RuntimeStateMachine::scripted_listener_actions
            => for<'a> fn(&'a RuntimeStateMachine) -> &'a [ScriptListenerActionDefinition]
    );
    exact_public_signature!(StateMachineInstance::state_machine_index => fn(&StateMachineInstance) -> usize);
    exact_public_signature!(
        StateMachineInstance::input_index_named
            => fn(&StateMachineInstance, &str) -> Option<usize>
    );
    exact_public_signature!(
        StateMachineInstance::get_bool
            => for<'a> fn(&'a StateMachineInstance, &str) -> Option<&'a StateMachineInputInstance>
    );
    exact_public_signature!(
        StateMachineInstance::get_number
            => for<'a> fn(&'a StateMachineInstance, &str) -> Option<&'a StateMachineInputInstance>
    );
    exact_public_signature!(
        StateMachineInstance::get_trigger
            => for<'a> fn(&'a StateMachineInstance, &str) -> Option<&'a StateMachineInputInstance>
    );
    exact_public_signature!(StateMachineInstance::set_bool => fn(&mut StateMachineInstance, usize, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_number => fn(&mut StateMachineInstance, usize, f32) -> bool);
    exact_public_signature!(StateMachineInstance::fire_trigger => fn(&mut StateMachineInstance, usize) -> bool);
    exact_public_signature!(StateMachineInstance::focus_up => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::focus_down => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::focus_left => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::focus_right => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(
        StateMachineInstance::key_input
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, u32, u32, bool, bool) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::text_input
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, &str) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::gamepad_dispatch
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, ScriptListenerInvocation) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::submit_gamepads_from_buffer
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, &[u8]) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::pointer_down
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::pointer_down_with_event_context
            => fn(
                &mut StateMachineInstance,
                &mut ArtboardInstance,
                f32,
                f32,
                i32,
                &StateMachineEventContext,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::pointer_down_with_owned_view_model_context
            => fn(
                &mut StateMachineInstance,
                &mut ArtboardInstance,
                f32,
                f32,
                i32,
                &mut RuntimeOwnedViewModelInstance,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::try_pointer_down_with_script_host
            => fn(
                &mut StateMachineInstance,
                &mut ArtboardInstance,
                f32,
                f32,
                i32,
                &mut dyn ScriptHost,
            ) -> Result<bool, ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::pointer_move
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, f32, i32) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::pointer_up
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::pointer_up_with_event_context
            => fn(
                &mut StateMachineInstance,
                &mut ArtboardInstance,
                f32,
                f32,
                i32,
                &StateMachineEventContext,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::pointer_exit
            => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::take_reported_events
            => fn(&mut StateMachineInstance, &ArtboardInstance) -> Vec<StateMachineReportedEvent>
    );
    exact_public_signature!(
        StateMachineInstance::reported_event_snapshot
            => for<'a> fn(&'a StateMachineInstance, usize) -> Option<&'a StateMachineReportedEvent>
    );
    exact_public_signature!(
        StateMachineInstance::has_pending_listener_view_model_reports
            => fn(&StateMachineInstance) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::script_error
            => for<'a> fn(&'a StateMachineInstance) -> Option<&'a ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::retain_scripted_object_data_context_error
            => fn(&mut StateMachineInstance, ScriptError)
    );
    exact_public_signature!(
        StateMachineInstance::scripted_objects
            => for<'a> fn(&'a StateMachineInstance) -> &'a [ScriptListenerActionDefinition]
    );
    exact_public_signature!(
        StateMachineInstance::scripted_listener_actions
            => for<'a> fn(&'a StateMachineInstance) -> &'a [ScriptListenerActionDefinition]
    );
    exact_public_signature!(
        StateMachineInstance::synchronize_scripted_input_groups
            => fn(&mut StateMachineInstance, &ArtboardInstance)
    );
    exact_public_signature!(
        StateMachineInstance::set_script_instance_for_global
            => fn(&mut StateMachineInstance, u32, Box<dyn ScriptInstance>)
    );
    exact_public_signature!(
        StateMachineInstance::set_script_input_for_global
            => fn(&mut StateMachineInstance, u32, &str, ScriptValue) -> Result<(), ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::set_scripted_listener_action_instance
            => fn(
                &mut StateMachineInstance,
                u32,
                Box<dyn ScriptInstance>,
            ) -> Result<(), ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::set_scripted_object_instance
            => fn(
                &mut StateMachineInstance,
                u32,
                Box<dyn ScriptInstance>,
            ) -> Result<(), ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::install_scripted_object_data_context
            => fn(
                &mut StateMachineInstance,
                u32,
                &ScriptListenerActionHydration,
            ) -> Result<(), ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::scripted_listener_action_input_snapshots
            => fn(&StateMachineInstance, u32) -> Option<Vec<ScriptListenerInputSnapshot>>
    );
    exact_public_signature!(
        StateMachineInstance::bind_scripted_listener_action_sources
            => fn(
                &mut StateMachineInstance,
                &RuntimeFile,
                Option<&RuntimeOwnedViewModelInstance>,
                bool,
            )
    );
    exact_public_signature!(
        StateMachineInstance::bind_scripted_listener_input_source
            => fn(
                &mut StateMachineInstance,
                &RuntimeFile,
                Option<&RuntimeOwnedViewModelInstance>,
                u32,
                u32,
                bool,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_scripted_listener_converter_own_sources
            => fn(
                &mut StateMachineInstance,
                &RuntimeFile,
                Option<&RuntimeOwnedViewModelInstance>,
                u32,
                u32,
                &[usize],
                bool,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::finalize_scripted_listener_input_sources
            => fn(&mut StateMachineInstance, u32, u32) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::begin_scripted_object_data_context_bind
            => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelHandle) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::begin_retained_scripted_object_data_context_rebind
            => fn(&mut StateMachineInstance) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::finish_scripted_object_data_context_bind
            => fn(&mut StateMachineInstance) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::adopt_scripted_listener_action_state_from
            => fn(
                &mut StateMachineInstance,
                &StateMachineInstance,
            ) -> Result<(), ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::rehome_owned_data_context_for_transaction
            => fn(
                &mut StateMachineInstance,
                &[(RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelHandle)],
            )
    );
    exact_public_signature!(StateMachineInstance::bind_empty_data_context => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::bind_default_view_model_context => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(
        StateMachineInstance::bind_view_model_instance_context
            => fn(&mut StateMachineInstance, &RuntimeFile, usize, usize) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_imported_view_model_context
            => fn(
                &mut StateMachineInstance,
                &RuntimeFile,
                &RuntimeImportedViewModelInstanceContext,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_owned_view_model_context
            => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelInstance) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_owned_view_model_handle
            => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelHandle) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_owned_view_model_context_handle
            => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelContextHandle) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_owned_view_model_context_mut
            => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_owned_view_model_contexts
            => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelContext) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_script_artboard_data_context
            => fn(&mut StateMachineInstance, &ScriptArtboardDataContext) -> bool
    );
    exact_public_signature!(StateMachineInstance::set_bindable_number_for_data_bind => fn(&mut StateMachineInstance, usize, f32) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_boolean_for_data_bind => fn(&mut StateMachineInstance, usize, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_integer_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_color_for_data_bind => fn(&mut StateMachineInstance, usize, u32) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_string_for_data_bind => fn(&mut StateMachineInstance, usize, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_enum_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_asset_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_artboard_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_list_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_trigger_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_view_model_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(
        StateMachineInstance::view_model_trigger_property_id
            => fn(&StateMachineInstance, usize) -> Option<u32>
    );
    exact_public_signature!(
        StateMachineInstance::bind_state_machine_data_bind_source
            => fn(&mut StateMachineInstance, usize) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::bind_state_machine_data_converter_own_sources
            => fn(
                &mut StateMachineInstance,
                &RuntimeFile,
                Option<&RuntimeOwnedViewModelInstance>,
                usize,
                &[usize],
                bool,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::finalize_state_machine_data_bind_source
            => fn(&mut StateMachineInstance, usize) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::rebind_state_machine_data_converter_final_input
            => fn(
                &mut StateMachineInstance,
                &RuntimeFile,
                Option<&RuntimeOwnedViewModelInstance>,
                usize,
                &[usize],
                usize,
                usize,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::update_data_binds_apply_target_to_source
            => fn(&mut StateMachineInstance) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::set_data_bind_formula_random_values
            => fn(&mut StateMachineInstance, &[f32])
    );
    exact_public_signature!(
        StateMachineInstance::data_bind_formula_random_call_count
            => fn(&StateMachineInstance) -> usize
    );
    exact_public_signature!(
        StateMachineInstance::transition_duration_binding_count
            => fn(&StateMachineInstance) -> usize
    );
    exact_public_signature!(
        StateMachineInstance::transition_duration_binding_value
            => fn(&StateMachineInstance, usize) -> Option<f32>
    );
    exact_public_signature!(
        StateMachineInstance::view_model_trigger_count
            => fn(&StateMachineInstance, usize) -> Option<u64>
    );
    exact_public_signature!(
        StateMachineInstance::view_model_trigger_value_count
            => fn(&StateMachineInstance) -> usize
    );
    exact_public_signature!(
        StateMachineInstance::bindable_number_value_for_data_bind
            => fn(&StateMachineInstance, usize) -> Option<f32>
    );
    exact_public_signature!(
        StateMachineInstance::default_view_model_number_source_value_for_data_bind
            => fn(&StateMachineInstance, usize) -> Option<f32>
    );
    exact_public_signature!(
        StateMachineInstance::set_default_view_model_number_source_for_data_bind
            => fn(&mut StateMachineInstance, usize, f32) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::set_imported_view_model_context_number_source_for_data_bind
            => fn(
                &mut StateMachineInstance,
                &mut RuntimeImportedViewModelInstanceContext,
                usize,
                f32,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineInstance::set_owned_view_model_context_number_source_for_data_bind
            => fn(
                &mut StateMachineInstance,
                &mut RuntimeOwnedViewModelInstance,
                usize,
                f32,
            ) -> bool
    );
    exact_public_signature!(
        StateMachineEventContext::from_geometry_hit
            => fn(&RuntimeGeometryHit) -> StateMachineEventContext
    );
    exact_public_signature!(
        StateMachineEventContext::path
            => for<'a> fn(&'a StateMachineEventContext) -> &'a [RuntimeGeometryHitPathSegment]
    );
    exact_public_signature!(
        StateMachineEventContext::occurrence
            => for<'a> fn(&'a StateMachineEventContext) -> &'a [RuntimeGeometryHitOccurrence]
    );
    exact_public_signature!(
        StateMachineEventStringProperty::name
            => for<'a> fn(&'a StateMachineEventStringProperty) -> &'a str
    );
    exact_public_signature!(
        StateMachineEventStringProperty::value
            => for<'a> fn(&'a StateMachineEventStringProperty) -> &'a str
    );
    exact_public_signature!(
        StateMachineReportedEvent::event_local_index
            => fn(&StateMachineReportedEvent) -> usize
    );
    exact_public_signature!(
        StateMachineReportedEvent::event_core_type
            => fn(&StateMachineReportedEvent) -> u32
    );
    exact_public_signature!(
        StateMachineReportedEvent::name
            => for<'a> fn(&'a StateMachineReportedEvent) -> Option<&'a str>
    );
    exact_public_signature!(
        StateMachineReportedEvent::url
            => for<'a> fn(&'a StateMachineReportedEvent) -> Option<&'a str>
    );
    exact_public_signature!(
        StateMachineReportedEvent::target
            => for<'a> fn(&'a StateMachineReportedEvent) -> Option<&'a str>
    );
    exact_public_signature!(
        StateMachineReportedEvent::string_properties
            => for<'a> fn(
                &'a StateMachineReportedEvent,
            ) -> &'a [StateMachineEventStringProperty]
    );
    exact_public_signature!(
        StateMachineReportedEvent::properties
            => for<'a> fn(&'a StateMachineReportedEvent) -> &'a [RuntimeEventProperty]
    );
    exact_public_signature!(
        StateMachineReportedEvent::seconds_delay
            => fn(&StateMachineReportedEvent) -> f32
    );
    exact_public_signature!(
        StateMachineReportedEvent::context
            => for<'a> fn(
                &'a StateMachineReportedEvent,
            ) -> Option<&'a StateMachineEventContext>
    );

    exact_public_signature!(StateMachineInstance::state_machine_index => fn(&StateMachineInstance) -> usize);
    exact_public_signature!(StateMachineInstance::input_index_named => fn(&StateMachineInstance, &str) -> Option<usize>);
    exact_public_signature!(StateMachineInstance::set_bool => fn(&mut StateMachineInstance, usize, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_number => fn(&mut StateMachineInstance, usize, f32) -> bool);
    exact_public_signature!(StateMachineInstance::fire_trigger => fn(&mut StateMachineInstance, usize) -> bool);
    exact_public_signature!(StateMachineInstance::focus_up => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::focus_down => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::focus_left => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::focus_right => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::set_focus => fn(&mut StateMachineInstance, Option<usize>) -> bool);
    exact_public_signature!(StateMachineInstance::focus_state => fn(&StateMachineInstance) -> FocusState);
    exact_public_signature!(StateMachineInstance::internal_focus_manager => fn(&StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::has_external_focus_manager => fn(&StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::clear_external_focus_manager => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::semantic_manager => fn(&StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::enable_semantics => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::set_external_semantic_manager => fn(&mut StateMachineInstance, Option<u64>, Option<u32>) -> bool);
    exact_public_signature!(StateMachineInstance::fire_semantic_action => fn(&mut StateMachineInstance, u32, u32) -> bool);
    exact_public_signature!(StateMachineInstance::key_input => fn(&mut StateMachineInstance, &mut ArtboardInstance, u32, u32, bool, bool) -> bool);
    exact_public_signature!(StateMachineInstance::text_input => fn(&mut StateMachineInstance, &mut ArtboardInstance, &str) -> bool);
    exact_public_signature!(StateMachineInstance::gamepad_dispatch => fn(&mut StateMachineInstance, &mut ArtboardInstance, ScriptListenerInvocation) -> bool);
    exact_public_signature!(StateMachineInstance::submit_gamepads_from_buffer => fn(&mut StateMachineInstance, &mut ArtboardInstance, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_down => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_down_with_event_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &StateMachineEventContext) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_down_with_owned_view_model_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_down_with_owned_view_model_and_event_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance, &StateMachineEventContext) -> bool);
    exact_public_signature!(StateMachineInstance::try_pointer_down_with_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_down_with_timestamp_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, f32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_down_with_owned_view_model_context_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::pointer_move => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, f32, i32) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_move_with_owned_view_model_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, f32, i32, &mut RuntimeOwnedViewModelInstance) -> bool);
    exact_public_signature!(StateMachineInstance::try_pointer_move_with_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_move_with_timestamp_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, f32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_move_with_owned_view_model_context_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::pointer_up => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_up_with_event_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &StateMachineEventContext) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_up_with_owned_view_model_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_up_with_owned_view_model_and_event_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance, &StateMachineEventContext) -> bool);
    exact_public_signature!(StateMachineInstance::try_pointer_up_with_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_up_with_timestamp_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, f32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_up_with_owned_view_model_context_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::pointer_exit => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32) -> bool);
    exact_public_signature!(StateMachineInstance::pointer_exit_with_owned_view_model_context => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance) -> bool);
    exact_public_signature!(StateMachineInstance::try_pointer_exit_with_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_exit_with_timestamp_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, f32, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::try_pointer_exit_with_owned_view_model_context_and_script_host => fn(&mut StateMachineInstance, &mut ArtboardInstance, f32, f32, i32, &mut RuntimeOwnedViewModelInstance, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::take_reported_events => fn(&mut StateMachineInstance, &ArtboardInstance) -> Vec<StateMachineReportedEvent>);
    exact_public_signature!(StateMachineInstance::reported_event_snapshot => for<'a> fn(&'a StateMachineInstance, usize) -> Option<&'a StateMachineReportedEvent>);
    exact_public_signature!(StateMachineInstance::has_pending_listener_view_model_reports => fn(&StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::script_error => for<'a> fn(&'a StateMachineInstance) -> Option<&'a ScriptError>);
    exact_public_signature!(StateMachineInstance::retain_scripted_object_data_context_error => fn(&mut StateMachineInstance, ScriptError) -> ());
    exact_public_signature!(StateMachineInstance::scripted_objects => for<'a> fn(&'a StateMachineInstance) -> &'a [ScriptListenerActionDefinition]);
    exact_public_signature!(StateMachineInstance::scripted_listener_actions => for<'a> fn(&'a StateMachineInstance) -> &'a [ScriptListenerActionDefinition]);
    exact_public_signature!(StateMachineInstance::set_script_instance_for_global => fn(&mut StateMachineInstance, u32, Box<dyn ScriptInstance>) -> ());
    exact_public_signature!(StateMachineInstance::set_script_input_for_global => fn(&mut StateMachineInstance, u32, &str, ScriptValue) -> Result<(), ScriptError>);
    exact_public_signature!(StateMachineInstance::set_scripted_listener_action_instance => fn(&mut StateMachineInstance, u32, Box<dyn ScriptInstance>) -> Result<(), ScriptError>);
    exact_public_signature!(StateMachineInstance::set_scripted_object_instance => fn(&mut StateMachineInstance, u32, Box<dyn ScriptInstance>) -> Result<(), ScriptError>);
    exact_public_signature!(StateMachineInstance::install_scripted_object_data_context => fn(&mut StateMachineInstance, u32, &ScriptListenerActionHydration) -> Result<(), ScriptError>);
    exact_public_signature!(StateMachineInstance::synchronize_scripted_input_groups => fn(&mut StateMachineInstance, &ArtboardInstance) -> ());
    exact_public_signature!(StateMachineInstance::scripted_listener_action_input_snapshots => fn(&StateMachineInstance, u32) -> Option<Vec<ScriptListenerInputSnapshot>>);
    exact_public_signature!(StateMachineInstance::bind_scripted_listener_action_sources => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, bool) -> ());
    exact_public_signature!(StateMachineInstance::bind_scripted_listener_input_source => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, u32, u32, bool) -> bool);
    exact_public_signature!(StateMachineInstance::bind_scripted_listener_converter_own_sources => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, u32, u32, &[usize], bool) -> bool);
    exact_public_signature!(StateMachineInstance::finalize_scripted_listener_input_sources => fn(&mut StateMachineInstance, u32, u32) -> bool);
    exact_public_signature!(StateMachineInstance::scripted_listener_data_context_view_models => fn(&StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelHandle>) -> (Option<ScriptViewModel>, Vec<Option<ScriptViewModel>>));
    exact_public_signature!(StateMachineInstance::scripted_listener_artboard_parent_context => fn(&StateMachineInstance, Option<&RuntimeOwnedViewModelContextHandle>) -> Option<ScriptArtboardParentContext>);
    exact_public_signature!(StateMachineInstance::scripted_listener_bound_view_model => fn(&StateMachineInstance, &RuntimeFile, &ScriptInputViewModelPropertyPath, Option<&RuntimeOwnedViewModelContextHandle>) -> Option<Option<ScriptViewModel>>);
    exact_public_signature!(StateMachineInstance::resolve_scripted_listener_scalar_binding => fn(&mut StateMachineInstance, &RuntimeFile, &RuntimeOwnedViewModelInstance, u32, u32, bool) -> Result<Option<ScriptValue>, ScriptError>);
    exact_public_signature!(StateMachineInstance::resolve_scripted_listener_artboard_binding => fn(&mut StateMachineInstance, &RuntimeFile, &RuntimeOwnedViewModelInstance, u32, u32, bool) -> Result<Option<u64>, ScriptError>);
    exact_public_signature!(StateMachineInstance::resolve_scripted_listener_trigger_binding => fn(&mut StateMachineInstance, &RuntimeFile, &RuntimeOwnedViewModelInstance, u32, u32, bool) -> Result<Option<u64>, ScriptError>);
    exact_public_signature!(StateMachineInstance::apply_scripted_listener_action_source_updates => fn(&mut StateMachineInstance, &ArtboardInstance, Option<&RuntimeOwnedViewModelInstance>, &mut dyn ScriptHost) -> Result<bool, ScriptError>);
    exact_public_signature!(StateMachineInstance::set_scripted_listener_artboard_resolver => fn(&mut StateMachineInstance, Box<dyn ScriptArtboardResolver>) -> ());
    exact_public_signature!(StateMachineInstance::scripted_listener_data_converter_targets => fn(&StateMachineInstance) -> Vec<(u32, u32, Vec<usize>, u32, bool)>);
    exact_public_signature!(StateMachineInstance::scripted_listener_data_converter_occurrences => fn(&StateMachineInstance) -> Vec<(u32, u32, Vec<usize>, u32, bool, bool)>);
    exact_public_signature!(StateMachineInstance::scripted_listener_data_converter_bind_steps => fn(&StateMachineInstance) -> Vec<RuntimeScriptedListenerDataConverterBindStep>);
    exact_public_signature!(StateMachineInstance::scripted_listener_data_converter_input_snapshots => fn(&StateMachineInstance, u32, u32, &[usize]) -> Option<Vec<ScriptListenerInputSnapshot>>);
    exact_public_signature!(StateMachineInstance::set_scripted_listener_data_converter_instance => fn(&mut StateMachineInstance, u32, u32, &[usize], u32, Box<dyn ScriptInstance>) -> Result<(), ScriptError>);
    exact_public_signature!(StateMachineInstance::has_scripted_listener_data_converter_instance => fn(&StateMachineInstance, u32, u32, &[usize]) -> bool);
    exact_public_signature!(StateMachineInstance::rebind_scripted_listener_data_converter_final_input => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, u32, u32, &[usize], usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::scripted_data_converter_occurrence_snapshots => fn(&StateMachineInstance) -> Vec<RuntimeScriptedDataConverterOccurrenceSnapshot>);
    exact_public_signature!(StateMachineInstance::scripted_data_converter_input_snapshots => fn(&StateMachineInstance, usize, &[usize]) -> Option<Vec<ScriptListenerInputSnapshot>>);
    exact_public_signature!(StateMachineInstance::set_scripted_data_converter_instance => fn(&mut StateMachineInstance, usize, &[usize], u32, Box<dyn ScriptInstance>) -> Result<(), ScriptError>);
    exact_public_signature!(StateMachineInstance::has_scripted_data_converter_instance => fn(&StateMachineInstance, usize, &[usize]) -> bool);
    exact_public_signature!(StateMachineInstance::bind_scripted_data_converter_sources => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, usize, &[usize], bool) -> bool);
    exact_public_signature!(StateMachineInstance::rebind_scripted_data_converter_final_inputs => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, usize, &[usize]) -> bool);
    exact_public_signature!(StateMachineInstance::begin_scripted_object_data_context_bind => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelHandle) -> bool);
    exact_public_signature!(StateMachineInstance::begin_retained_scripted_object_data_context_rebind => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::finish_scripted_object_data_context_bind => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::adopt_scripted_listener_action_state_from => fn(&mut StateMachineInstance, &StateMachineInstance) -> Result<(), ScriptError>);
    exact_public_signature!(StateMachineInstance::rehome_owned_data_context_for_transaction => fn(&mut StateMachineInstance, &[(RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelHandle)]) -> ());
    exact_public_signature!(StateMachineInstance::bind_empty_data_context => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::bind_default_view_model_context => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::bind_view_model_instance_context => fn(&mut StateMachineInstance, &RuntimeFile, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::bind_imported_view_model_context => fn(&mut StateMachineInstance, &RuntimeFile, &RuntimeImportedViewModelInstanceContext) -> bool);
    exact_public_signature!(StateMachineInstance::bind_owned_view_model_context => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelInstance) -> bool);
    exact_public_signature!(StateMachineInstance::bind_owned_view_model_handle => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelHandle) -> bool);
    exact_public_signature!(StateMachineInstance::bind_owned_view_model_context_handle => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelContextHandle) -> bool);
    exact_public_signature!(StateMachineInstance::bind_owned_view_model_context_mut => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance) -> bool);
    exact_public_signature!(StateMachineInstance::bind_owned_view_model_contexts => fn(&mut StateMachineInstance, &RuntimeOwnedViewModelContext) -> bool);
    exact_public_signature!(StateMachineInstance::bind_script_artboard_data_context => fn(&mut StateMachineInstance, &ScriptArtboardDataContext) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_number_for_data_bind => fn(&mut StateMachineInstance, usize, f32) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_boolean_for_data_bind => fn(&mut StateMachineInstance, usize, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_integer_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_color_for_data_bind => fn(&mut StateMachineInstance, usize, u32) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_string_for_data_bind => fn(&mut StateMachineInstance, usize, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_enum_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_asset_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_artboard_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_list_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_trigger_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_bindable_view_model_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::bindable_number_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<f32>);
    exact_public_signature!(StateMachineInstance::bindable_boolean_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<bool>);
    exact_public_signature!(StateMachineInstance::bindable_integer_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::bindable_color_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u32>);
    exact_public_signature!(StateMachineInstance::bindable_string_value_for_data_bind => for<'a> fn(&'a StateMachineInstance, usize) -> Option<&'a [u8]>);
    exact_public_signature!(StateMachineInstance::bindable_enum_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::bindable_asset_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::bindable_artboard_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::bindable_list_property_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<usize>);
    exact_public_signature!(StateMachineInstance::bindable_trigger_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::bindable_view_model_instance_index_for_data_bind => fn(&StateMachineInstance, usize) -> Option<usize>);
    exact_public_signature!(StateMachineInstance::default_view_model_number_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<f32>);
    exact_public_signature!(StateMachineInstance::default_view_model_boolean_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<bool>);
    exact_public_signature!(StateMachineInstance::default_view_model_string_source_value_for_data_bind => for<'a> fn(&'a StateMachineInstance, usize) -> Option<&'a [u8]>);
    exact_public_signature!(StateMachineInstance::default_view_model_color_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u32>);
    exact_public_signature!(StateMachineInstance::default_view_model_enum_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::default_view_model_symbol_list_index_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::default_view_model_asset_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::default_view_model_artboard_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::default_view_model_list_source_item_count_for_data_bind => fn(&StateMachineInstance, usize) -> Option<usize>);
    exact_public_signature!(StateMachineInstance::default_view_model_trigger_source_value_for_data_bind => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::default_view_model_view_model_source_instance_index_for_data_bind => fn(&StateMachineInstance, usize) -> Option<usize>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_number_source_for_data_bind => fn(&mut StateMachineInstance, usize, f32) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_number_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, f32) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_number_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelNumberSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_number_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelNumberSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_number_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelNumberSourceHandle, f32) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_boolean_source_for_data_bind => fn(&mut StateMachineInstance, usize, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_boolean_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, bool) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_boolean_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelBooleanSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_boolean_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelBooleanSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_boolean_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelBooleanSourceHandle, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_string_source_for_data_bind => fn(&mut StateMachineInstance, usize, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_string_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_string_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelStringSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_string_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelStringSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_string_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelStringSourceHandle, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_color_source_for_data_bind => fn(&mut StateMachineInstance, usize, u32) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_color_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, u32) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_color_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelColorSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_color_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelColorSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_color_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelColorSourceHandle, u32) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_enum_source_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_enum_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, u64) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_enum_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelEnumSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_enum_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelEnumSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_enum_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelEnumSourceHandle, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_symbol_list_index_source_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_symbol_list_index_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, u64) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_symbol_list_index_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_symbol_list_index_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_symbol_list_index_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelSymbolListIndexSourceHandle, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_asset_source_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_asset_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, u64) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_asset_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelAssetSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_asset_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelAssetSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_asset_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelAssetSourceHandle, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_artboard_source_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_artboard_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, u64) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_artboard_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelArtboardSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_artboard_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelArtboardSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_artboard_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelArtboardSourceHandle, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_trigger_source_for_data_bind => fn(&mut StateMachineInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_trigger_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, u64) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_trigger_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelTriggerSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_trigger_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelTriggerSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_trigger_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelTriggerSourceHandle, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_list_source_item_count_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_list_source_item_count_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, usize) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_list_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelListSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_list_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelListSourceHandle>);
    exact_public_signature!(StateMachineInstance::set_default_view_model_list_source_item_count_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelListSourceHandle, usize) -> bool);
    exact_public_signature!(StateMachineInstance::set_default_view_model_view_model_source_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::relink_default_view_model_view_model_source_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::relink_default_view_model_view_model_source_by_property_name => fn(&mut StateMachineInstance, &RuntimeFile, &str, usize) -> bool);
    exact_public_signature!(StateMachineInstance::default_view_model_view_model_source_handle_by_property_name => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelViewModelSourceHandle>);
    exact_public_signature!(StateMachineInstance::default_view_model_view_model_source_handle_by_property_name_path => fn(&StateMachineInstance, &RuntimeFile, &str) -> Option<RuntimeDefaultViewModelViewModelSourceHandle>);
    exact_public_signature!(StateMachineInstance::relink_default_view_model_view_model_source_by_source_handle => fn(&mut StateMachineInstance, &RuntimeDefaultViewModelViewModelSourceHandle, usize) -> bool);
    exact_public_signature!(StateMachineInstance::relink_view_model_instance_view_model_source_for_data_bind => fn(&mut StateMachineInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::relink_imported_view_model_context_view_model_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_number_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, f32) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_boolean_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_string_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_color_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, u32) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_enum_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_symbol_list_index_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_asset_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_artboard_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_trigger_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_imported_view_model_context_list_source_item_count_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeImportedViewModelInstanceContext, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_number_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, f32) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_boolean_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, bool) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_string_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, &[u8]) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_color_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, u32) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_enum_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_symbol_list_index_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_asset_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_artboard_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_trigger_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, u64) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_list_source_item_count_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::set_owned_view_model_context_view_model_source_for_data_bind => fn(&mut StateMachineInstance, &mut RuntimeOwnedViewModelInstance, usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::bind_state_machine_data_bind_source => fn(&mut StateMachineInstance, usize) -> bool);
    exact_public_signature!(StateMachineInstance::bind_state_machine_data_converter_own_sources => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, usize, &[usize], bool) -> bool);
    exact_public_signature!(StateMachineInstance::finalize_state_machine_data_bind_source => fn(&mut StateMachineInstance, usize) -> bool);
    exact_public_signature!(StateMachineInstance::rebind_state_machine_data_converter_final_input => fn(&mut StateMachineInstance, &RuntimeFile, Option<&RuntimeOwnedViewModelInstance>, usize, &[usize], usize, usize) -> bool);
    exact_public_signature!(StateMachineInstance::update_data_binds_apply_target_to_source => fn(&mut StateMachineInstance) -> bool);
    exact_public_signature!(StateMachineInstance::set_data_bind_formula_random_values => fn(&mut StateMachineInstance, &[f32]) -> ());
    exact_public_signature!(StateMachineInstance::data_bind_formula_random_call_count => fn(&StateMachineInstance) -> usize);
    exact_public_signature!(StateMachineInstance::transition_duration_binding_count => fn(&StateMachineInstance) -> usize);
    exact_public_signature!(StateMachineInstance::transition_duration_binding_value => fn(&StateMachineInstance, usize) -> Option<f32>);
    exact_public_signature!(StateMachineInstance::view_model_trigger_count => fn(&StateMachineInstance, usize) -> Option<u64>);
    exact_public_signature!(StateMachineInstance::view_model_trigger_value_count => fn(&StateMachineInstance) -> usize);
    exact_public_signature!(StateMachineInstance::view_model_trigger_property_id => fn(&StateMachineInstance, usize) -> Option<u32>);
    public_methods_are_reachable!(RuntimeStateMachine;
        scripted_objects,
        scripted_listener_actions,
    );
    public_methods_are_reachable!(StateMachineInstance;
        state_machine_index,
        input_index_named,
        get_bool,
        get_number,
        get_trigger,
        set_bool,
        set_number,
        fire_trigger,
        focus_up,
        focus_down,
        focus_left,
        focus_right,
        key_input,
        text_input,
        gamepad_dispatch,
        submit_gamepads_from_buffer,
        pointer_down,
        pointer_down_with_event_context,
        pointer_down_with_owned_view_model_context,
        try_pointer_down_with_script_host,
        pointer_move,
        pointer_up,
        pointer_up_with_event_context,
        pointer_exit,
        take_reported_events,
        reported_event_snapshot,
        has_pending_listener_view_model_reports,
        script_error,
        retain_scripted_object_data_context_error,
        scripted_objects,
        scripted_listener_actions,
        set_script_instance_for_global,
        set_script_input_for_global,
        set_scripted_listener_action_instance,
        set_scripted_object_instance,
        install_scripted_object_data_context,
        synchronize_scripted_input_groups,
        scripted_listener_action_input_snapshots,
        bind_scripted_listener_action_sources,
        bind_scripted_listener_input_source,
        bind_scripted_listener_converter_own_sources,
        finalize_scripted_listener_input_sources,
        begin_scripted_object_data_context_bind,
        begin_retained_scripted_object_data_context_rebind,
        finish_scripted_object_data_context_bind,
        adopt_scripted_listener_action_state_from,
        rehome_owned_data_context_for_transaction,
        bind_empty_data_context,
        bind_default_view_model_context,
        bind_view_model_instance_context,
        bind_imported_view_model_context,
        bind_owned_view_model_context,
        bind_owned_view_model_handle,
        bind_owned_view_model_context_handle,
        bind_owned_view_model_context_mut,
        bind_owned_view_model_contexts,
        bind_script_artboard_data_context,
        set_bindable_number_for_data_bind,
        set_bindable_boolean_for_data_bind,
        set_bindable_integer_for_data_bind,
        set_bindable_color_for_data_bind,
        set_bindable_string_for_data_bind,
        set_bindable_enum_for_data_bind,
        set_bindable_asset_for_data_bind,
        set_bindable_artboard_for_data_bind,
        set_bindable_list_for_data_bind,
        set_bindable_trigger_for_data_bind,
        set_bindable_view_model_for_data_bind,
        bind_state_machine_data_bind_source,
        bind_state_machine_data_converter_own_sources,
        finalize_state_machine_data_bind_source,
        rebind_state_machine_data_converter_final_input,
        update_data_binds_apply_target_to_source,
        set_data_bind_formula_random_values,
        data_bind_formula_random_call_count,
        transition_duration_binding_count,
        transition_duration_binding_value,
        view_model_trigger_count,
        view_model_trigger_value_count,
        view_model_trigger_property_id,
        bindable_number_value_for_data_bind,
        default_view_model_number_source_value_for_data_bind,
        set_default_view_model_number_source_for_data_bind,
        set_imported_view_model_context_number_source_for_data_bind,
        set_owned_view_model_context_number_source_for_data_bind,
    );

    type HydrationFactory =
        fn(&StateMachineInstance) -> Result<ScriptListenerActionHydration, ScriptError>;
    exact_public_signature!(
        StateMachineInstance::hydrate_and_initialize_scripted_data_converter_instance::<HydrationFactory>
            => fn(
                &mut StateMachineInstance,
                usize,
                &[usize],
                ScriptListenerActionHydration,
                bool,
                Option<&mut dyn nuxie_render_api::Factory>,
                HydrationFactory,
            ) -> Result<bool, ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::hydrate_and_initialize_scripted_listener_data_converter_instance::<HydrationFactory>
            => fn(
                &mut StateMachineInstance,
                u32,
                u32,
                &[usize],
                ScriptListenerActionHydration,
                bool,
                Option<&mut dyn nuxie_render_api::Factory>,
                HydrationFactory,
            ) -> Result<bool, ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::hydrate_and_initialize_scripted_listener_action_instance::<HydrationFactory>
            => fn(
                &mut StateMachineInstance,
                u32,
                ScriptListenerActionHydration,
                bool,
                Option<&mut dyn nuxie_render_api::Factory>,
                HydrationFactory,
            ) -> Result<bool, ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::hydrate_and_initialize_scripted_object_instance::<HydrationFactory>
            => fn(
                &mut StateMachineInstance,
                u32,
                ScriptListenerActionHydration,
                bool,
                Option<&mut dyn nuxie_render_api::Factory>,
                HydrationFactory,
            ) -> Result<bool, ScriptError>
    );
    exact_public_signature!(
        StateMachineInstance::hydrate_and_initialize_scripted_object_instance_after_context_install::<HydrationFactory>
            => fn(
                &mut StateMachineInstance,
                u32,
                bool,
                Option<&mut dyn nuxie_render_api::Factory>,
                HydrationFactory,
            ) -> Result<bool, ScriptError>
    );

    struct ExactFnOnceToken<'a> {
        borrowed: &'a mut (),
        not_send_or_sync: Rc<Cell<()>>,
    }

    fn exact_fn_once_token(marker: &mut ()) -> ExactFnOnceToken<'_> {
        ExactFnOnceToken {
            borrowed: marker,
            not_send_or_sync: Rc::new(Cell::new(())),
        }
    }

    // Every closure consumes a captured token that is non-Clone, non-Send,
    // non-Sync, and tied to a local borrow. It is therefore FnOnce-only and
    // non-'static. Narrowing to Fn or adding any of those extra bounds must
    // make this downstream inventory fail to compile.
    fn hydration_methods_accept_fn_once(machine: &mut StateMachineInstance) {
        let mut marker = ();
        let token = exact_fn_once_token(&mut marker);
        let _ = machine.hydrate_and_initialize_scripted_data_converter_instance(
            0,
            &[],
            ScriptListenerActionHydration::new(None, Vec::new()),
            false,
            None,
            move |_| {
                let _ = (&token.borrowed, &token.not_send_or_sync);
                drop(token);
                Ok(ScriptListenerActionHydration::new(None, Vec::new()))
            },
        );

        let mut marker = ();
        let token = exact_fn_once_token(&mut marker);
        let _ = machine.hydrate_and_initialize_scripted_listener_data_converter_instance(
            0,
            0,
            &[],
            ScriptListenerActionHydration::new(None, Vec::new()),
            false,
            None,
            move |_| {
                let _ = (&token.borrowed, &token.not_send_or_sync);
                drop(token);
                Ok(ScriptListenerActionHydration::new(None, Vec::new()))
            },
        );

        let mut marker = ();
        let token = exact_fn_once_token(&mut marker);
        let _ = machine.hydrate_and_initialize_scripted_listener_action_instance(
            0,
            ScriptListenerActionHydration::new(None, Vec::new()),
            false,
            None,
            move |_| {
                let _ = (&token.borrowed, &token.not_send_or_sync);
                drop(token);
                Ok(ScriptListenerActionHydration::new(None, Vec::new()))
            },
        );

        let mut marker = ();
        let token = exact_fn_once_token(&mut marker);
        let _ = machine.hydrate_and_initialize_scripted_object_instance(
            0,
            ScriptListenerActionHydration::new(None, Vec::new()),
            false,
            None,
            move |_| {
                let _ = (&token.borrowed, &token.not_send_or_sync);
                drop(token);
                Ok(ScriptListenerActionHydration::new(None, Vec::new()))
            },
        );

        let mut marker = ();
        let token = exact_fn_once_token(&mut marker);
        let _ = machine.hydrate_and_initialize_scripted_object_instance_after_context_install(
            0,
            false,
            None,
            move |_| {
                let _ = (&token.borrowed, &token.not_send_or_sync);
                drop(token);
                Ok(ScriptListenerActionHydration::new(None, Vec::new()))
            },
        );
    }
    let _ = hydration_methods_accept_fn_once as fn(&mut StateMachineInstance);
    // END exhaustive W4 exact signatures

    public_methods_are_reachable!(StateMachineEventContext;
        from_geometry_hit,
        path,
        occurrence,
    );
    public_methods_are_reachable!(StateMachineEventStringProperty;
        name,
        value,
    );
    public_methods_are_reachable!(StateMachineReportedEvent;
        event_local_index,
        event_core_type,
        name,
        url,
        target,
        string_properties,
        properties,
        seconds_delay,
        context,
    );
}
