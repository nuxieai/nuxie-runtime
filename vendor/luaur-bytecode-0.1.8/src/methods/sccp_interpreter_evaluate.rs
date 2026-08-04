use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::condition_state::ConditionState;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp_interpreter::SccpInterpreter;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> SccpInterpreter<'a> {
    pub fn evaluate(
        &mut self,
        op: LuauOpcode,
        func: &crate::records::bc_function::BcFunction,
        inst_op: BcOp,
    ) -> ConstnessLattice {
        let inst = &func.instructions[inst_op.index as usize];
        match op {
            LuauOpcode::LOP_LOADK | LuauOpcode::LOP_LOADKX => {
                let op = inst.ops[0];
                LUAU_ASSERT!(op.kind == BcOpKind::VmConst);
                ConstnessLattice::from_vm_const(Constness::VmConstant, op)
            }
            LuauOpcode::LOP_LOADB | LuauOpcode::LOP_LOADN => {
                let op = inst.ops[0];
                LUAU_ASSERT!(op.kind == BcOpKind::Imm);
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    *self.impl_.as_imm(op).operator_deref(),
                )
            }
            LuauOpcode::LOP_LOADNIL => {
                ConstnessLattice::from_vm_const(Constness::VmConstant, self.impl_.make_nil())
            }
            LuauOpcode::LOP_ADD
            | LuauOpcode::LOP_SUB
            | LuauOpcode::LOP_MUL
            | LuauOpcode::LOP_DIV
            | LuauOpcode::LOP_MOD
            | LuauOpcode::LOP_POW
            | LuauOpcode::LOP_IDIV => self.evaluate_arith(op, func, inst_op),
            LuauOpcode::LOP_MOVE => unsafe { &mut *self.state }.operand_lattice(&inst.ops[0]),
            LuauOpcode::LOP_JUMPIF | LuauOpcode::LOP_JUMPIFNOT => {
                let condition = self.evaluate_condition(inst.ops[0]);
                if condition == ConditionState::Unknown {
                    return ConstnessLattice::from_kind(
                        unsafe { &mut *self.state }.unknown_condition_constness([inst.ops[0]]),
                    );
                }
                let jumps_on_true = inst.op == LuauOpcode::LOP_JUMPIF;
                let takes_jump = (condition == ConditionState::AlwaysTrue) == jumps_on_true;
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_bool(takes_jump),
                )
            }
            LuauOpcode::LOP_JUMPIFEQ
            | LuauOpcode::LOP_JUMPIFLE
            | LuauOpcode::LOP_JUMPIFLT
            | LuauOpcode::LOP_JUMPIFNOTEQ
            | LuauOpcode::LOP_JUMPIFNOTLE
            | LuauOpcode::LOP_JUMPIFNOTLT => {
                let condition =
                    self.evaluate_comparison_condition(inst.op, inst.ops[0], inst.ops[1]);
                if condition == ConditionState::Unknown {
                    return ConstnessLattice::from_kind(
                        unsafe { &mut *self.state }
                            .unknown_condition_constness([inst.ops[0], inst.ops[1]]),
                    );
                }
                let negated = matches!(
                    inst.op,
                    LuauOpcode::LOP_JUMPIFNOTEQ
                        | LuauOpcode::LOP_JUMPIFNOTLE
                        | LuauOpcode::LOP_JUMPIFNOTLT
                );
                let takes_jump = (condition == ConditionState::AlwaysTrue) != negated;
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_bool(takes_jump),
                )
            }
            LuauOpcode::LOP_JUMPXEQKNIL
            | LuauOpcode::LOP_JUMPXEQKB
            | LuauOpcode::LOP_JUMPXEQKN
            | LuauOpcode::LOP_JUMPXEQKS => {
                let condition = self.evaluate_xeqk_condition(func, inst_op);
                if condition == ConditionState::Unknown {
                    return ConstnessLattice::from_kind(
                        unsafe { &mut *self.state }.unknown_condition_constness([inst.ops[0]]),
                    );
                }
                let negated = !self.impl_.falsey(&inst.ops[1]);
                let takes_jump = (condition == ConditionState::AlwaysTrue) != negated;
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_bool(takes_jump),
                )
            }
            _ => ConstnessLattice::from_kind(Constness::NotAConstant),
        }
    }
}
