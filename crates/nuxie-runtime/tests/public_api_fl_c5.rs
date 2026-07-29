use nuxie_runtime::{
    FocusState, RuntimeStateMachine, ScriptError, ScriptListenerActionHydration,
    StateMachineEventContext, StateMachineEventStringProperty, StateMachineInstance,
    StateMachineReportedEvent,
};

macro_rules! public_methods_are_reachable {
    ($owner:ty; $($method:ident),+ $(,)?) => {
        $(let _ = <$owner>::$method;)+
    };
}

#[test]
fn fl_c5_public_reexports_are_downstream_visible_after_file_split() {
    fn definition_fields_are_public(machine: &RuntimeStateMachine) {
        let _ = machine.global_id;
        let _ = &machine.name;
        let _ = &machine.inputs;
        let _ = &machine.layers;
    }
    let _ = definition_fields_are_public as fn(&RuntimeStateMachine);
    fn is_clone<T: Clone>() {}
    is_clone::<StateMachineInstance>();
    let focus = FocusState::default();
    let _ = (focus.has_focus, focus.expects_keyboard_input);

    public_methods_are_reachable!(RuntimeStateMachine;
        scripted_objects,
        scripted_listener_actions,
    );
    public_methods_are_reachable!(StateMachineInstance;
        state_machine_index,
        input_index_named,
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
        bind_script_artboard_data_context,
        set_bindable_number_for_data_bind,
        set_bindable_boolean_for_data_bind,
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
        bindable_number_value_for_data_bind,
        default_view_model_number_source_value_for_data_bind,
        set_default_view_model_number_source_for_data_bind,
        set_imported_view_model_context_number_source_for_data_bind,
        set_owned_view_model_context_number_source_for_data_bind,
    );

    type HydrationFactory =
        fn(&StateMachineInstance) -> Result<ScriptListenerActionHydration, ScriptError>;
    let _ = StateMachineInstance::hydrate_and_initialize_scripted_data_converter_instance::<
        HydrationFactory,
    >;
    let _ = StateMachineInstance::hydrate_and_initialize_scripted_listener_data_converter_instance::<
        HydrationFactory,
    >;
    let _ = StateMachineInstance::hydrate_and_initialize_scripted_listener_action_instance::<
        HydrationFactory,
    >;
    let _ =
        StateMachineInstance::hydrate_and_initialize_scripted_object_instance::<HydrationFactory>;

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
