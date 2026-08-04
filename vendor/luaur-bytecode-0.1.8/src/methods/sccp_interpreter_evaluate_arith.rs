use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp_interpreter::SccpInterpreter;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> SccpInterpreter<'a> {
    pub fn evaluate_arith(
        &mut self,
        opcode: LuauOpcode,
        func: &crate::records::bc_function::BcFunction,
        inst_op: BcOp,
    ) -> ConstnessLattice {
        let inst = &func.instructions[inst_op.index as usize];
        let lhs = inst.ops[0];
        let rhs = inst.ops[1];
        let state = unsafe { &mut *self.state };
        let lhs_constness = state.operand_lattice(&lhs);
        let rhs_constness = state.operand_lattice(&rhs);

        if lhs_constness.kind == Constness::ImmConstant
            && rhs_constness.kind == Constness::ImmConstant
        {
            let lhs_imm = lhs_constness.imm_const.expect("immediate lattice");
            let rhs_imm = rhs_constness.imm_const.expect("immediate lattice");

            if lhs_imm.kind == BcImmKind::Int && rhs_imm.kind == BcImmKind::Int {
                let lv = unsafe { lhs_imm.value.valueInt };
                let rv = unsafe { rhs_imm.value.valueInt };

                if rv == 0
                    && matches!(
                        opcode,
                        LuauOpcode::LOP_DIV | LuauOpcode::LOP_MOD | LuauOpcode::LOP_IDIV
                    )
                {
                    return ConstnessLattice::from_kind(Constness::NotAConstant);
                }

                if matches!(opcode, LuauOpcode::LOP_DIV | LuauOpcode::LOP_POW) {
                    return ConstnessLattice::from_kind(Constness::NotAConstant);
                }

                let result = match opcode {
                    LuauOpcode::LOP_ADD => i64::from(lv) + i64::from(rv),
                    LuauOpcode::LOP_SUB => i64::from(lv) - i64::from(rv),
                    LuauOpcode::LOP_MUL => i64::from(lv) * i64::from(rv),
                    LuauOpcode::LOP_MOD => {
                        let mut remainder = i64::from(lv) % i64::from(rv);
                        if remainder != 0 && ((lv < 0) != (rv < 0)) {
                            remainder += i64::from(rv);
                        }
                        remainder
                    }
                    LuauOpcode::LOP_IDIV => {
                        let mut result = i64::from(lv) / i64::from(rv);
                        if result < 0 && (i64::from(lv) % i64::from(rv)) != 0 {
                            result -= 1;
                        }
                        result
                    }
                    _ => {
                        LUAU_ASSERT!(false, "Unhandled opcode");
                        return ConstnessLattice::from_kind(Constness::NotAConstant);
                    }
                };

                if result < i64::from(i16::MIN) || result > i64::from(i16::MAX) {
                    return ConstnessLattice::from_kind(Constness::NotAConstant);
                }

                return ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_int(result as i32),
                );
            }
        } else if lhs_constness.kind == Constness::VmConstant
            && rhs_constness.kind == Constness::VmConstant
        {
            if let Some(vm_const) = self.impl_.evaluate(
                &lhs_constness.vm_const.expect("VM constant lattice"),
                &rhs_constness.vm_const.expect("VM constant lattice"),
                opcode,
            ) {
                return ConstnessLattice::from_vm_const(Constness::VmConstant, vm_const);
            }
            return ConstnessLattice::from_kind(Constness::NotAConstant);
        } else if lhs_constness.kind == Constness::Undetermined
            && rhs_constness.kind == Constness::Undetermined
        {
            return ConstnessLattice::from_kind(Constness::Undetermined);
        }

        ConstnessLattice::from_kind(Constness::NotAConstant)
    }
}
