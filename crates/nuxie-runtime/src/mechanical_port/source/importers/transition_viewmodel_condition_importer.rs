use std::any::Any;

use crate::mechanical_port::source::{core::CoreHandle, status_code::StatusCode};

use super::import_stack::ImportStackObject;

pub struct TransitionViewModelConditionImporter {
    condition: CoreHandle,
}

impl TransitionViewModelConditionImporter {
    pub fn new(condition: CoreHandle) -> Self {
        Self { condition }
    }

    pub fn set_comparator(&mut self, comparator: CoreHandle) {
        self.condition
            .with_mut(|condition| {
                condition.transition_viewmodel_condition_set_comparator(comparator)
            })
            .filter(|set| *set)
            .expect("imported condition derives from TransitionViewModelCondition");
    }
}

impl ImportStackObject for TransitionViewModelConditionImporter {
    fn resolve(&mut self) -> StatusCode {
        self.condition
            .with_mut(|condition| condition.transition_viewmodel_condition_initialize())
            .filter(|initialized| *initialized)
            .expect("imported condition derives from TransitionViewModelCondition");
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
