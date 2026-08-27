use crate::mechanical_port::source::{
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
    fn perform(&self, state_machine_instance: *mut ());
}

impl StateMachineFireAction {
    pub fn occurs(&self) -> StateMachineFireOccurance {
        StateMachineFireOccurance(self.base.occurs_value() as i32)
    }

    pub fn import(self: Box<Self>, import_stack: &mut ImportStack) -> StatusCode {
        let object = Box::into_raw(self);
        let Some(importer) = import_stack
            .latest::<StateMachineLayerComponentImporter>(StateMachineLayerComponentBase::TYPE_KEY)
        else {
            unsafe { drop(Box::from_raw(object)) };
            return StatusCode::MissingObject;
        };
        importer.add_fire_event(unsafe { Box::from_raw(object) });
        unsafe { (*object).base.base.import(import_stack) }
    }
}
