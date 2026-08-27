use std::ptr::NonNull;

use crate::mechanical_port::source::{
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
            .latest::<TransitionViewModelConditionImporter>(TransitionViewModelCondition::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        importer.set_comparator(NonNull::from(&mut *self));
        self.base.base.import(import_stack)
    }

    pub fn use_in_layer(&self, _state_machine_instance: *const (), _layer_instance: *mut ()) {}
}
