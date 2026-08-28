use crate::mechanical_port::source::{
    animation::state_machine_instance::StateMachineInstance,
    generated::animation::{
        state_machine_fire_action_base::StateMachineFireActionBase,
        state_machine_layer_component_base::StateMachineLayerComponentBase,
    },
    importers::{
        import_stack::ImportStack,
        state_machine_layer_component_importer::StateMachineLayerComponentImporter,
    },
    status_code::StatusCode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StateMachineFireOccurance(pub i32);

impl StateMachineFireOccurance {
    pub const AT_START: Self = Self(0);
    pub const AT_END: Self = Self(1);
}

#[derive(Default)]
pub struct StateMachineFireAction {
    pub base: StateMachineFireActionBase,
}

pub trait StateMachineFireActionBehavior {
    fn perform(&self, state_machine_instance: &mut StateMachineInstance);
}

impl StateMachineFireAction {
    pub fn occurs(&self) -> StateMachineFireOccurance {
        StateMachineFireOccurance(self.base.occurs_value() as i32)
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack
            .latest::<StateMachineLayerComponentImporter>(StateMachineLayerComponentBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_fire_event(this);
        self.base.base.import(import_stack)
    }
}
impl std::ops::Deref for StateMachineFireAction {
    type Target = StateMachineFireActionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachineFireAction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::state_machine_fire_action_base::StateMachineFireActionBaseCallbacks for StateMachineFireAction { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
