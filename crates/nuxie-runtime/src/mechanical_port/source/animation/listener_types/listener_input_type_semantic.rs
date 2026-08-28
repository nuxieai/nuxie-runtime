use crate::mechanical_port::source::{
    animation::semantic_listener_group::SemanticActionType, core::CoreHandle,
    generated::animation::listener_types::listener_input_type_semantic_base::ListenerInputTypeSemanticBase,
};
pub trait SemanticConstraintListener {
    fn semantic_input_types(&self) -> Vec<CoreHandle>;
}
#[derive(Default)]
pub struct ListenerInputTypeSemantic {
    pub base: ListenerInputTypeSemanticBase,
    semantic_inputs: Vec<CoreHandle>,
}
impl ListenerInputTypeSemantic {
    pub fn semantic_input_count(&self) -> usize {
        self.semantic_inputs.len()
    }
    pub fn semantic_input(&self, index: usize) -> Option<CoreHandle> {
        self.semantic_inputs.get(index).cloned()
    }
    pub fn add_semantic_input(&mut self, input: CoreHandle) {
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
            let Some(matched) = input_type.with_downcast::<ListenerInputTypeSemantic, _>(|input_type| {
                if input_type.semantic_input_count() == 0 {
                    return true;
                }
                input_type.semantic_inputs.iter().any(|input| {
                    input
                        .with_downcast::<crate::mechanical_port::source::inputs::semantic_input::SemanticInput, _>(|input| input.action_type() == action)
                        .unwrap_or(false)
                })
            }) else {
                continue;
            };
            if matched {
                return true;
            }
        }
        false
    }
}
