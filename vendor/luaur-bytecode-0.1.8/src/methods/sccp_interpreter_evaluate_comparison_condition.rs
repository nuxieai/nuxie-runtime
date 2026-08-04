use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::condition_state::ConditionState;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp_interpreter::SccpInterpreter;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> SccpInterpreter<'a> {
    pub fn evaluate_comparison_condition(
        &mut self,
        op: LuauOpcode,
        lhs: BcOp,
        rhs: BcOp,
    ) -> ConditionState {
        let state = unsafe { &mut *self.state };
        let lhs_const = state.operand_lattice(&lhs);
        let rhs_const = state.operand_lattice(&rhs);
        let is_ordering_op = matches!(
            op,
            LuauOpcode::LOP_JUMPIFLT
                | LuauOpcode::LOP_JUMPIFLE
                | LuauOpcode::LOP_JUMPIFNOTLT
                | LuauOpcode::LOP_JUMPIFNOTLE
        );
        let is_orderable = |lattice: &ConstnessLattice| {
            if lattice.kind == Constness::VmConstant {
                self.impl_
                    .is_orderable(&lattice.vm_const.expect("VM constant lattice"))
            } else if lattice.kind == Constness::ImmConstant {
                lattice.imm_const.expect("immediate lattice").kind == BcImmKind::Int
            } else {
                false
            }
        };

        if is_ordering_op && (!is_orderable(&lhs_const) || !is_orderable(&rhs_const)) {
            return ConditionState::Unknown;
        }

        if lhs_const.kind == Constness::VmConstant
            && rhs_const.kind == Constness::VmConstant
            && !self.impl_.kind_equals(
                &lhs_const.vm_const.expect("VM constant lattice"),
                &rhs_const.vm_const.expect("VM constant lattice"),
            )
        {
            return ConditionState::Unknown;
        }

        let apply_op = |cmp: i32| match op {
            LuauOpcode::LOP_JUMPIFEQ | LuauOpcode::LOP_JUMPIFNOTEQ => cmp == 0,
            LuauOpcode::LOP_JUMPIFLT | LuauOpcode::LOP_JUMPIFNOTLT => cmp < 0,
            LuauOpcode::LOP_JUMPIFLE | LuauOpcode::LOP_JUMPIFNOTLE => cmp <= 0,
            _ => {
                LUAU_ASSERT!(false, "Unhandled comparison opcode");
                false
            }
        };
        let condition = |value| {
            if value {
                ConditionState::AlwaysTrue
            } else {
                ConditionState::AlwaysFalse
            }
        };

        if lhs_const.kind == Constness::VmConstant && rhs_const.kind == Constness::VmConstant {
            let cmp = self.impl_.cmp_bc_op(
                &lhs_const.vm_const.expect("VM constant lattice"),
                &rhs_const.vm_const.expect("VM constant lattice"),
            );
            return condition(apply_op(cmp));
        } else if lhs_const.kind == Constness::ImmConstant
            && rhs_const.kind == Constness::ImmConstant
        {
            let lhs_imm = lhs_const.imm_const.expect("immediate lattice");
            let rhs_imm = rhs_const.imm_const.expect("immediate lattice");
            let cmp = if lhs_imm.kind == BcImmKind::Int && rhs_imm.kind == BcImmKind::Int {
                let lv = unsafe { lhs_imm.value.valueInt };
                let rv = unsafe { rhs_imm.value.valueInt };
                i32::from(lv > rv) - i32::from(lv < rv)
            } else if lhs_imm.kind == BcImmKind::Boolean && rhs_imm.kind == BcImmKind::Boolean {
                i32::from(
                    unsafe { lhs_imm.value.valueBoolean } != unsafe { rhs_imm.value.valueBoolean },
                )
            } else {
                return ConditionState::Unknown;
            };
            return condition(apply_op(cmp));
        } else if lhs_const.kind == Constness::VmConstant
            && rhs_const.kind == Constness::ImmConstant
        {
            let cmp = self.impl_.cmp_bc_imm(
                &lhs_const.vm_const.expect("VM constant lattice"),
                &rhs_const.imm_const.expect("immediate lattice"),
            );
            return condition(apply_op(cmp));
        } else if lhs_const.kind == Constness::ImmConstant
            && rhs_const.kind == Constness::VmConstant
        {
            let cmp = -self.impl_.cmp_bc_imm(
                &rhs_const.vm_const.expect("VM constant lattice"),
                &lhs_const.imm_const.expect("immediate lattice"),
            );
            return condition(apply_op(cmp));
        }

        ConditionState::Unknown
    }
}
