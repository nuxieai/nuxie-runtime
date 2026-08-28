use crate::mechanical_port::source::{
    animation::{
        state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle,
        transition_condition_op::TransitionConditionOp,
    },
    generated::animation::transition_focus_condition_base::TransitionFocusConditionBase,
};
pub trait FocusConditionStateMachine {
    fn right_component_comparator_object_id(&self) -> Option<u32>;
    fn left_component_comparator_object_id(&self) -> Option<u32>;
    fn target_or_descendant_has_focus(&self, object_id: u32) -> bool;
}
#[derive(Default)]
pub struct TransitionFocusCondition {
    pub base: TransitionFocusConditionBase,
}
impl TransitionFocusCondition {
    pub fn evaluate(
        &self,
        machine: Option<&dyn FocusConditionStateMachine>,
        _layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(machine) = machine else { return false };
        let Some(object_id) = machine
            .right_component_comparator_object_id()
            .or_else(|| machine.left_component_comparator_object_id())
        else {
            return false;
        };
        let focused = machine.target_or_descendant_has_focus(object_id);
        if self.base.base.op() == TransitionConditionOp::Equal {
            focused
        } else {
            !focused
        }
    }
}
