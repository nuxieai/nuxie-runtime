//! Cold, lazily allocated state owned by a StateMachineInstance.
use super::{
    focus_listener_group::RuntimeFocusListenerGroupHandle,
    gamepad_listener_group::RuntimeGamepadListenerGroupHandle,
    keyboard_listener_group::RuntimeKeyboardListenerGroupHandle,
    semantic_listener_group::{RuntimeSemanticListenerGroupHandle, SemanticActionType},
    state_machine_instance::{
        EventReport, RuntimeListenerViewModelHandle, RuntimeListenerViewModelWeakHandle,
    },
};
use crate::mechanical_port::source::{
    core::CoreHandle, input::gamepad_batch::GamepadBatchState,
    semantic::semantic_manager::RuntimeSemanticManagerHandle,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default)]
pub(super) struct SMIReporting {
    pub reported_events: Vec<EventReport>,
    // Keep separate pending/delivering batches: callbacks can report again.
    pub reporting_events: Vec<EventReport>,
    pub events_applied_during_loop: Vec<EventReport>,
    pub listener_view_models: Vec<RuntimeListenerViewModelHandle>,
    pub reported_listener_view_models: Rc<RefCell<Vec<RuntimeListenerViewModelWeakHandle>>>,
    pub reporting_listener_view_models: Vec<RuntimeListenerViewModelWeakHandle>,
}

#[derive(Default)]
pub(super) struct SMIBindables {
    pub property_instances: HashMap<CoreHandle, CoreHandle>,
    pub data_binds_to_target: HashMap<CoreHandle, CoreHandle>,
    pub data_binds_to_source: HashMap<CoreHandle, CoreHandle>,
    pub transition_property_instances: HashMap<CoreHandle, HashMap<u32, CoreHandle>>,
}

#[derive(Clone)]
pub(super) struct QueuedFocusEvent {
    pub group: RuntimeFocusListenerGroupHandle,
    pub is_focus: bool,
}

#[derive(Clone)]
pub(super) struct QueuedSemanticEvent {
    pub group: RuntimeSemanticListenerGroupHandle,
    pub action_type: SemanticActionType,
}

#[derive(Default)]
pub(super) struct SMIInputExtras {
    pub focus_listener_groups: Vec<RuntimeFocusListenerGroupHandle>,
    pub keyboard_listener_groups: Vec<RuntimeKeyboardListenerGroupHandle>,
    pub gamepad_listener_groups: Vec<RuntimeGamepadListenerGroupHandle>,
    pub semantic_listener_groups: Vec<RuntimeSemanticListenerGroupHandle>,
    pub gamepad_scripted_drawables: Vec<CoreHandle>,
    pub embedder_gamepads: Rc<GamepadBatchState>,
    pub semantic_manager: Option<RuntimeSemanticManagerHandle>,
    pub external_semantic_manager: Option<RuntimeSemanticManagerHandle>,
    pub queued_focus_events: Vec<QueuedFocusEvent>,
    pub queued_semantic_events: Vec<QueuedSemanticEvent>,
}

#[derive(Default)]
pub(super) struct SMIScripting {
    // Source/instance pairs retain authored order for context binding and init.
    pub objects: Vec<(CoreHandle, CoreHandle)>,
}

impl SMIScripting {
    pub fn find(&self, source: &CoreHandle) -> Option<CoreHandle> {
        self.objects
            .iter()
            .find(|(candidate, _)| candidate == source)
            .map(|(_, instance)| instance.clone())
    }
}
