use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::condition_state::ConditionState;
use crate::records::bc_op::BcOp;
use crate::records::jump_target::JumpTarget;
use crate::records::sccp::Sccp;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn conditional_targets(
        &self,
        inst_op: BcOp,
        target: BcOp,
        condition: ConditionState,
        target_taken_on_true: bool,
    ) -> Vec<JumpTarget> {
        LUAU_ASSERT!(target.kind == BcOpKind::Block);
        let inst = &self.func().instructions[inst_op.index as usize];
        let fallthrough = self
            .get_fallthrough(inst.block)
            .expect("conditional branch fallthrough");
        let mut target_dead = false;
        let mut fallthrough_dead = false;
        if condition == ConditionState::AlwaysTrue {
            target_dead = !target_taken_on_true;
            fallthrough_dead = target_taken_on_true;
        } else if condition == ConditionState::AlwaysFalse {
            target_dead = target_taken_on_true;
            fallthrough_dead = !target_taken_on_true;
        }
        vec![
            JumpTarget {
                dead: target_dead,
                block_op: target,
                condition,
            },
            JumpTarget {
                dead: fallthrough_dead,
                block_op: fallthrough,
                condition,
            },
        ]
    }
}
