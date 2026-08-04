use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::condition_state::ConditionState;
use crate::records::bc_op::BcOp;
use crate::records::jump_target::JumpTarget;
use crate::records::sccp::Sccp;
use alloc::vec::Vec;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn jump_targets(&mut self, inst_op: BcOp) -> Vec<JumpTarget> {
        let (op, ops) = {
            let inst = &self.func().instructions[inst_op.index as usize];
            (inst.op, inst.ops.clone())
        };
        match op {
            LuauOpcode::LOP_JUMP | LuauOpcode::LOP_JUMPBACK => {
                LUAU_ASSERT!(ops[0].kind == BcOpKind::Block);
                vec![JumpTarget {
                    dead: false,
                    block_op: ops[0],
                    condition: ConditionState::AlwaysTrue,
                }]
            }
            LuauOpcode::LOP_JUMPIF | LuauOpcode::LOP_JUMPIFNOT => {
                let condition = self.interpreter.evaluate_condition(ops[0]);
                self.conditional_targets(inst_op, ops[1], condition, op == LuauOpcode::LOP_JUMPIF)
            }
            LuauOpcode::LOP_JUMPIFEQ
            | LuauOpcode::LOP_JUMPIFLE
            | LuauOpcode::LOP_JUMPIFLT
            | LuauOpcode::LOP_JUMPIFNOTEQ
            | LuauOpcode::LOP_JUMPIFNOTLE
            | LuauOpcode::LOP_JUMPIFNOTLT => {
                let condition = self
                    .interpreter
                    .evaluate_comparison_condition(op, ops[0], ops[1]);
                let negated = matches!(
                    op,
                    LuauOpcode::LOP_JUMPIFNOTEQ
                        | LuauOpcode::LOP_JUMPIFNOTLE
                        | LuauOpcode::LOP_JUMPIFNOTLT
                );
                self.conditional_targets(inst_op, ops[2], condition, !negated)
            }
            LuauOpcode::LOP_JUMPXEQKNIL
            | LuauOpcode::LOP_JUMPXEQKB
            | LuauOpcode::LOP_JUMPXEQKN
            | LuauOpcode::LOP_JUMPXEQKS => {
                let condition = self
                    .interpreter
                    .evaluate_xeqk_condition(unsafe { &*self.func }, inst_op);
                let negated = unsafe {
                    self.func().immediates[ops[1].index as usize]
                        .value
                        .valueBoolean
                };
                self.conditional_targets(inst_op, ops[2], condition, !negated)
            }
            LuauOpcode::LOP_FORNPREP
            | LuauOpcode::LOP_FORNLOOP
            | LuauOpcode::LOP_FORGPREP
            | LuauOpcode::LOP_FORGPREP_NEXT
            | LuauOpcode::LOP_FORGPREP_INEXT => {
                self.conditional_targets(inst_op, ops[3], ConditionState::Unknown, true)
            }
            LuauOpcode::LOP_FORGLOOP => {
                self.conditional_targets(inst_op, ops[5], ConditionState::Unknown, true)
            }
            LuauOpcode::LOP_CMPPROTO => {
                self.conditional_targets(inst_op, ops[2], ConditionState::Unknown, true)
            }
            LuauOpcode::LOP_JUMPX => {
                LUAU_ASSERT!(false, "Should have never parsed this");
                Vec::new()
            }
            _ => Vec::new(),
        }
    }
}
