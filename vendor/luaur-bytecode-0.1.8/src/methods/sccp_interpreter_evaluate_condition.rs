use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::condition_state::ConditionState;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::sccp_interpreter::SccpInterpreter;

impl<'a> SccpInterpreter<'a> {
    pub fn evaluate_condition(&mut self, op: BcOp) -> ConditionState {
        let lhs = unsafe { &mut *self.state }.operand_lattice(&op);
        if lhs.kind == Constness::VmConstant {
            if self
                .impl_
                .falsey(&lhs.vm_const.expect("VM constant lattice"))
            {
                ConditionState::AlwaysFalse
            } else {
                ConditionState::AlwaysTrue
            }
        } else if lhs.kind == Constness::ImmConstant {
            let imm = lhs.imm_const.expect("immediate lattice");
            if imm.kind == BcImmKind::Boolean {
                if unsafe { imm.value.valueBoolean } {
                    ConditionState::AlwaysTrue
                } else {
                    ConditionState::AlwaysFalse
                }
            } else {
                ConditionState::Unknown
            }
        } else {
            ConditionState::Unknown
        }
    }
}
