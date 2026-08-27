use crate::mechanical_port::source::{
    animation::semantic_listener_group::SemanticActionType,
    generated::animation::listener_types::listener_input_type_semantic_base::ListenerInputTypeSemanticBase,
    inputs::semantic_input::SemanticInput,
};
use std::ptr::NonNull;
pub trait SemanticConstraintListener {
    fn semantic_input_types(&self) -> Vec<&ListenerInputTypeSemantic>;
}
#[derive(Default)]
pub struct ListenerInputTypeSemantic {
    pub base: ListenerInputTypeSemanticBase,
    semantic_inputs: Vec<NonNull<SemanticInput>>,
}
impl ListenerInputTypeSemantic {
    pub fn semantic_input_count(&self) -> usize {
        self.semantic_inputs.len()
    }
    pub fn semantic_input(&self, index: usize) -> Option<&SemanticInput> {
        self.semantic_inputs
            .get(index)
            .map(|value| unsafe { value.as_ref() })
    }
    pub fn add_semantic_input(&mut self, input: &mut SemanticInput) {
        let input = NonNull::from(input);
        if !self.semantic_inputs.contains(&input) {
            self.semantic_inputs.push(input);
        }
    }
    pub fn semantic_listener_constraints_met(
        listener: Option<&dyn SemanticConstraintListener>,
        action: SemanticActionType,
    ) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        let action = action as u32;
        for input_type in listener.semantic_input_types() {
            if input_type.semantic_input_count() == 0 {
                return true;
            }
            for index in 0..input_type.semantic_input_count() {
                if input_type
                    .semantic_input(index)
                    .is_some_and(|input| input.base.action_type() == action)
                {
                    return true;
                }
            }
        }
        false
    }
}
