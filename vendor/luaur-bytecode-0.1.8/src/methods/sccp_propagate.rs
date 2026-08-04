use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::sccp::Sccp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn propagate(&mut self) {
        let entry = self.func().entry_block;
        let exit = self.func().exit_block;
        self.block_uses.get_or_insert(entry.index).insert(entry);
        self.block_uses.get_or_insert(exit.index).insert(exit);
        self.flow_worklist.push_back(entry);

        while !self.flow_worklist.is_empty() || !self.ssa_worklist.is_empty() {
            while let Some(block_op) = self.flow_worklist.pop_front() {
                if self.flow_worklist_set.contains(&block_op) {
                    continue;
                }
                let (phis, ops, successors) = {
                    let block = &self.func().blocks[block_op.index as usize];
                    (
                        block.phis.clone(),
                        block.ops.clone(),
                        block.successors.clone(),
                    )
                };
                for phi_op in phis {
                    LUAU_ASSERT!(phi_op.kind == BcOpKind::Phi);
                    self.visit_phi(phi_op);
                }
                for op in ops.iter() {
                    LUAU_ASSERT!(op.kind == BcOpKind::Inst);
                    self.visit_inst(*op);
                }
                let block_ends_with_branch = ops
                    .back()
                    .copied()
                    .map(|op| !self.jump_targets(op).is_empty())
                    .unwrap_or(false);

                for successor in successors.iter() {
                    if successor.kind == BcBlockEdgeKind::Fallthrough {
                        let successor_index = successor.target.index;
                        if block_ends_with_branch
                            && !self
                                .block_uses
                                .get_or_insert(successor_index)
                                .contains(&block_op)
                        {
                            continue;
                        }
                        self.block_uses
                            .get_or_insert(successor_index)
                            .insert(block_op);
                        if !self.flow_worklist_set.contains(&successor.target) {
                            self.flow_worklist.push_back(successor.target);
                        }
                    }
                }
                self.flow_worklist_set.insert(block_op);
            }

            while let Some(op) = self.ssa_worklist.pop_front() {
                if op.kind == BcOpKind::Inst {
                    self.visit_inst(op);
                } else if op.kind == BcOpKind::Phi {
                    self.visit_phi(op);
                }
            }
        }
    }
}
