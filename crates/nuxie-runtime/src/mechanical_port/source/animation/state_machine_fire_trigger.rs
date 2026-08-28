use crate::mechanical_port::source::{
    core::CoreHandle, data_bind_path_referencer::DataBindPathReferencer,
    generated::animation::state_machine_fire_trigger_base::StateMachineFireTriggerBase,
    importers::import_stack::ImportStack, status_code::StatusCode,
    viewmodel::viewmodel_instance_trigger::ViewModelInstanceTrigger,
};

pub enum FireTriggerViewModelProperty {
    Trigger(CoreHandle),
    Other,
}

pub trait FireTriggerDataContext {
    fn get_view_model_property(&mut self, path: &[u32]) -> Option<FireTriggerViewModelProperty>;
}

pub trait FireTriggerStateMachine {
    fn data_context(&mut self) -> Option<&mut dyn FireTriggerDataContext>;
}

#[derive(Default)]
pub struct StateMachineFireTrigger {
    pub base: StateMachineFireTriggerBase,
    pub data_bind_path_referencer: DataBindPathReferencer,
}

impl StateMachineFireTrigger {
    pub fn perform(&self, state_machine_instance: &mut dyn FireTriggerStateMachine) {
        let Some(data_context) = state_machine_instance.data_context() else {
            return;
        };
        let Some(path) = self
            .data_bind_path_referencer
            .with_data_bind_path(|path| path.path().clone())
        else {
            return;
        };
        let Some(FireTriggerViewModelProperty::Trigger(trigger)) =
            data_context.get_view_model_property(&path)
        else {
            return;
        };
        let _ =
            trigger.with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| trigger.trigger());
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.data_bind_path_referencer
            .import_data_bind_path(import_stack);
        self.base.base.import(import_stack)
    }

    pub fn decode_view_model_path_ids(&mut self, value: &[u8]) {
        self.data_bind_path_referencer.decode_data_bind_path(value);
    }

    pub fn copy_view_model_path_ids(&mut self, object: &Self) {
        self.data_bind_path_referencer
            .copy_data_bind_path(&object.data_bind_path_referencer);
    }
}
