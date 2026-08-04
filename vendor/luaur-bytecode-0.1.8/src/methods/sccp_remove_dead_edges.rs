use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;
use crate::type_aliases::bc_edges::BcEdges;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn remove_dead_edges(&mut self, inst_op: BcOp) {
        let targets = self.jump_targets(inst_op);
        let block_op = self.func().instructions[inst_op.index as usize].block;
        let mut live_target = None;
        for target in targets {
            if target.dead {
                let successors = &self.func().blocks[block_op.index as usize].successors;
                let filtered: BcEdges = successors
                    .iter()
                    .copied()
                    .filter(|edge| edge.target != target.block_op)
                    .collect();
                self.func_mut().blocks[block_op.index as usize].successors = filtered;
            } else {
                live_target = Some(target.block_op);
            }
        }
        let Some(live_target) = live_target else {
            return;
        };
        let block = &mut self.func_mut().blocks[block_op.index as usize];
        if let Some(edge) = block
            .successors
            .iter_mut()
            .find(|edge| edge.target == live_target)
        {
            edge.kind = BcBlockEdgeKind::Fallthrough;
        } else {
            block.successors.push(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: live_target,
            });
        }
    }
}
