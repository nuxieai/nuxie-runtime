use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::condition_state::ConditionState;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::sccp_interpreter::SccpInterpreter;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> SccpInterpreter<'a> {
    pub fn evaluate_xeqk_condition(
        &mut self,
        func: &crate::records::bc_function::BcFunction,
        inst_op: BcOp,
    ) -> ConditionState {
        let inst = &func.instructions[inst_op.index as usize];
        let val_const = unsafe { &mut *self.state }.operand_lattice(&inst.ops[0]);
        let condition = |value| {
            if value {
                ConditionState::AlwaysTrue
            } else {
                ConditionState::AlwaysFalse
            }
        };

        match inst.op {
            LuauOpcode::LOP_JUMPXEQKNIL => {
                let nil = self.impl_.make_nil();
                if val_const.kind == Constness::VmConstant
                    && self
                        .impl_
                        .falsey(&val_const.vm_const.expect("VM constant lattice"))
                    && self
                        .impl_
                        .kind_equals(&val_const.vm_const.expect("VM constant lattice"), &nil)
                {
                    return ConditionState::AlwaysTrue;
                } else if val_const.kind == Constness::ImmConstant
                    || (val_const.kind == Constness::VmConstant
                        && !self
                            .impl_
                            .kind_equals(&val_const.vm_const.expect("VM constant lattice"), &nil))
                {
                    return ConditionState::AlwaysFalse;
                }
            }
            LuauOpcode::LOP_JUMPXEQKB => {
                let cmp_imm_op = inst.ops[3];
                LUAU_ASSERT!(cmp_imm_op.kind == BcOpKind::Imm);
                if val_const.kind == Constness::ImmConstant
                    && val_const.imm_const.expect("immediate lattice").kind == BcImmKind::Boolean
                {
                    // Preserve the upstream 0.730 implementation, including
                    // its immediate-lattice access through vmConst.
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("upstream vmConst access"),
                        &cmp_imm_op,
                    ) {
                        return condition(eq);
                    }
                } else if val_const.kind == Constness::VmConstant {
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("VM constant lattice"),
                        &cmp_imm_op,
                    ) {
                        return condition(eq);
                    }
                }
            }
            LuauOpcode::LOP_JUMPXEQKN => {
                let cmp_const_op = inst.ops[3];
                LUAU_ASSERT!(cmp_const_op.kind == BcOpKind::VmConst);
                if val_const.kind == Constness::VmConstant {
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("VM constant lattice"),
                        &cmp_const_op,
                    ) {
                        return condition(eq);
                    }
                } else if val_const.kind == Constness::ImmConstant
                    && val_const.imm_const.expect("immediate lattice").kind == BcImmKind::Int
                {
                    if let Some(eq) = self.impl_.eq_int(&cmp_const_op, unsafe {
                        val_const
                            .imm_const
                            .expect("immediate lattice")
                            .value
                            .valueInt
                    }) {
                        return condition(eq);
                    }
                }
            }
            LuauOpcode::LOP_JUMPXEQKS => {
                let cmp_const_op = inst.ops[3];
                LUAU_ASSERT!(cmp_const_op.kind == BcOpKind::VmConst);
                if val_const.kind == Constness::VmConstant {
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("VM constant lattice"),
                        &cmp_const_op,
                    ) {
                        return condition(eq);
                    }
                }
            }
            _ => {}
        }

        ConditionState::Unknown
    }
}
