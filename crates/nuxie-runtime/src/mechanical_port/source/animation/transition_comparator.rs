use crate::mechanical_port::source::{
    animation::state_machine_instance::{
        RuntimeStateMachineLayerInstanceWeakHandle, StateMachineInstance,
    },
    animation::transition_viewmodel_condition::TransitionViewModelCondition,
    generated::animation::transition_comparator_base::TransitionComparatorBase,
    importers::{
        import_stack::ImportStack,
        transition_viewmodel_condition_importer::TransitionViewModelConditionImporter,
    },
    status_code::StatusCode,
};

#[derive(Default)]
pub struct TransitionComparator {
    pub base: TransitionComparatorBase,
}

impl TransitionComparator {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack
            .latest::<TransitionViewModelConditionImporter>(crate::mechanical_port::source::generated::animation::transition_viewmodel_condition_base::TransitionViewModelConditionBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.set_comparator(this);
        self.base.base.import(import_stack)
    }

    pub fn use_in_layer(
        &self,
        _state_machine_instance: &mut StateMachineInstance,
        _layer_instance: RuntimeStateMachineLayerInstanceWeakHandle,
    ) {
    }
}
impl std::ops::Deref for TransitionComparator {
    type Target = TransitionComparatorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionComparator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
