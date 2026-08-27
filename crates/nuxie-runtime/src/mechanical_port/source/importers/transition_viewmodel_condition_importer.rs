use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    animation::{
        transition_comparator::TransitionComparator,
        transition_viewmodel_condition::TransitionViewModelCondition,
    },
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct TransitionViewModelConditionImporter {
    condition: NonNull<TransitionViewModelCondition>,
}

impl TransitionViewModelConditionImporter {
    pub fn new(condition: NonNull<TransitionViewModelCondition>) -> Self {
        Self { condition }
    }

    pub fn set_comparator(&mut self, comparator: NonNull<TransitionComparator>) {
        unsafe { self.condition.as_mut().set_comparator(comparator) };
    }
}

impl ImportStackObject for TransitionViewModelConditionImporter {
    fn resolve(&mut self) -> StatusCode {
        unsafe { self.condition.as_mut().initialize() };
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
