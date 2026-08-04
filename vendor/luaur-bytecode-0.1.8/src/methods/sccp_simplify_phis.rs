use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn simplify_phis(&mut self) {
        let blocks: Vec<BcOp> = self.flow_worklist_set.iter().copied().collect();
        for block_op in blocks {
            let phis: Vec<BcOp> = self.func().blocks[block_op.index as usize]
                .phis
                .iter()
                .copied()
                .collect();
            for op in phis {
                LUAU_ASSERT!(op.kind == BcOpKind::Phi);
                let operands = self.func().phis[op.index as usize].ops.clone();
                if operands.is_empty() {
                    continue;
                }
                let unique = operands[0];
                if operands.iter().skip(1).any(|operand| *operand != unique) {
                    continue;
                }
                for use_op in self.uses_of(op) {
                    if use_op.kind == BcOpKind::Inst {
                        self.replace_operand(use_op, op, unique);
                    } else if use_op.kind == BcOpKind::Phi {
                        self.replace_phi_operand(use_op, op, unique);
                    }
                    self.push_use(unique, use_op);
                }
                self.func_mut().phis[op.index as usize].uses.clear();
                self.func_mut().blocks[block_op.index as usize]
                    .phis
                    .retain(|candidate| *candidate != op);
            }
        }
    }
}
