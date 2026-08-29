use crate::mechanical_port::source::{
    animation::{
        state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle,
        state_machine_instance::StateMachineInstance,
        transition_condition_op::TransitionConditionOp,
        transition_property_component_comparator::TransitionPropertyComponentComparator,
    },
    focus_data::FocusData,
    generated::animation::transition_focus_condition_base::TransitionFocusConditionBase,
};
#[derive(Default)]
pub struct TransitionFocusCondition {
    pub base: TransitionFocusConditionBase,
}
impl TransitionFocusCondition {
    pub fn evaluate(
        &self,
        machine: Option<&StateMachineInstance>,
        _layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(machine) = machine else { return false };
        let object_id = |comparator: crate::mechanical_port::source::core::CoreHandle| {
            comparator.with_downcast::<TransitionPropertyComponentComparator, _>(|comparator| {
                comparator.base.object_id()
            })
        };
        let Some(object_id) = self
            .base
            .base
            .right_comparator()
            .and_then(object_id)
            .or_else(|| self.base.base.left_comparator().and_then(object_id))
        else {
            return false;
        };
        let mut focused = false;
        if let Some(target) = machine.resolve_artboard_object(object_id) {
            if let Some(children) = target
                .with(|target| {
                    target.as_node()?;
                    Some(target.as_container_component()?.children().to_vec())
                })
                .flatten()
            {
                for child in children {
                    if let Some(node) =
                        child.with_downcast_mut::<FocusData, _>(FocusData::focus_node)
                    {
                        focused = machine
                            .focus_manager()
                            .with_focus_manager(|manager| manager.has_focus(&node));
                        break;
                    }
                }
            }
        }
        if self.base.base.op() == Some(TransitionConditionOp::Equal) {
            focused
        } else {
            !focused
        }
    }
}
