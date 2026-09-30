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
pub(super) struct BindablePropertyInstances {
    entries: Vec<Option<(CoreHandle, CoreHandle)>>,
    count: usize,
}
impl BindablePropertyInstances {
    pub fn find(&self, shared: &CoreHandle) -> Option<CoreHandle> {
        if self.entries.is_empty() {
            return None;
        }
        let mask = self.entries.len() - 1;
        let mut i = Self::slot_of(shared) & mask;
        loop {
            match &self.entries[i] {
                Some((key, instance)) if key == shared => return Some(instance.clone()),
                None => return None,
                _ => i = (i + 1) & mask,
            }
        }
    }
    // The source requires that shared is not already present.
    pub fn insert(&mut self, shared: CoreHandle, instance: CoreHandle) {
        if (self.count + 1) * 2 > self.entries.len() {
            self.grow();
        }
        self.place(shared, instance);
        self.count += 1;
    }
    pub fn for_each_instance(&self, mut visit: impl FnMut(&CoreHandle)) {
        for (_, instance) in self.entries.iter().flatten() {
            visit(instance);
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.count = 0;
    }
    fn slot_of(shared: &CoreHandle) -> usize {
        // Stable arena-slot allocation addresses stand in for C++ object
        // pointers, without dereferencing them. Full generation-aware handle
        // equality still decides identity when a slot is reused.
        ((shared.slot_address() as u64).wrapping_mul(0x9E3779B97F4A7C15) >> 32) as usize
    }
    fn place(&mut self, shared: CoreHandle, instance: CoreHandle) {
        let mask = self.entries.len() - 1;
        let mut i = Self::slot_of(&shared) & mask;
        while self.entries[i].is_some() {
            i = (i + 1) & mask;
        }
        self.entries[i] = Some((shared, instance));
    }
    fn grow(&mut self) {
        let old = std::mem::take(&mut self.entries);
        self.entries = vec![None; if old.is_empty() { 8 } else { old.len() * 2 }];
        for (shared, instance) in old.into_iter().flatten() {
            self.place(shared, instance);
        }
    }
}

#[derive(Default)]
pub(super) struct SMIBindables {
    pub property_instances: BindablePropertyInstances,
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

#[cfg(test)]
mod bindable_property_instances_test {
    use super::BindablePropertyInstances;
    use crate::mechanical_port::source::{
        core::CoreArena, data_bind::bindable_property_number::BindablePropertyNumber,
    };
    use std::collections::HashSet;

    // tests/unit_tests/runtime/bindable_property_instances_test.cpp, 955d6a05.
    // Opaque arena handles replace addresses in the source's fake-property
    // buffers. The table compares identity without dereferencing the objects.
    #[test]
    fn bindable_property_instances_find_what_was_inserted() {
        const COUNT: usize = 1000;
        let shared_arena = CoreArena::default();
        let instance_arena = CoreArena::default();
        let shared: Vec<_> = (0..COUNT * 2)
            .map(|_| shared_arena.insert(BindablePropertyNumber::default()))
            .collect();
        let instances: Vec<_> = (0..COUNT)
            .map(|_| instance_arena.insert(BindablePropertyNumber::default()))
            .collect();
        let mut table = BindablePropertyInstances::default();
        assert_eq!(table.find(&shared[0]), None);
        for i in 0..COUNT {
            table.insert(shared[i * 2].clone(), instances[i].clone());
        }
        for i in 0..COUNT {
            assert_eq!(table.find(&shared[i * 2]), Some(instances[i].clone()));
            assert_eq!(table.find(&shared[i * 2 + 1]), None);
        }
        let mut visited = HashSet::new();
        table.for_each_instance(|instance| {
            visited.insert(instance.clone());
        });
        assert_eq!(visited.len(), COUNT);
        table.clear();
        assert_eq!(table.find(&shared[0]), None);
        let mut remaining = 0;
        table.for_each_instance(|_| remaining += 1);
        assert_eq!(remaining, 0);
    }
}
