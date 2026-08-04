use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use std::collections::HashSet;

impl<'a> BytecodeGraphParser<'a> {
    pub fn find_producer_bc_op_reg_unordered_set_bc_op_bc_op_hash(
        &mut self,
        block: BcOp,
        reg: Reg,
        visited: &mut HashSet<BcOp, BcOpHash>,
    ) -> Option<BcOp> {
        visited.insert(block);
        LUAU_ASSERT!(block.index < self.producers.len() as u32);
        let block_producers = &self.producers[block.index as usize];

        if (reg as i32) > block_producers.invalidAfter {
            return None;
        }

        if let Some(local) = block_producers.own.get(&reg) {
            return Some(*local);
        }

        if let Some(cached) = block_producers.cached.get(&reg) {
            return Some(*cached);
        }

        if block_producers.multiReturn.kind != BcOpKind::None
            && reg >= block_producers.multiReturnStart
        {
            return Some(self.func.add_proj(
                block_producers.multiReturn,
                (reg - block_producers.multiReturnStart) as u32,
            ));
        }

        // BcOp does not implement Ord, so we use a Vec and deduplicate manually to mimic std::unordered_set behavior
        // while avoiding the Ord requirement of BTreeSet.
        let mut results: Vec<BcOp> = Vec::new();
        let bl = self.func.block_op(block);
        let predecessors = bl.predecessors.clone();

        for edge in &predecessors {
            let ctrl = edge.kind;
            let pred = edge.target;

            if ctrl == BcBlockEdgeKind::Loop || visited.contains(&pred) {
                continue;
            }
            LUAU_ASSERT!(block != pred);

            if let Some(op) =
                self.find_producer_bc_op_reg_unordered_set_bc_op_bc_op_hash(pred, reg, visited)
            {
                if op.kind == BcOpKind::Phi {
                    let phi = self.func.phi_op(op);
                    for &proj in &phi.ops {
                        if !results.contains(&proj) {
                            results.push(proj);
                        }
                    }
                } else {
                    if !results.contains(&op) {
                        results.push(op);
                    }
                }
            }
        }

        if results.is_empty() {
            return None;
        }

        let res = if results.len() == 1 {
            results[0]
        } else {
            let res = self.func.add_phi();
            let phi = self.func.phi_op(res);
            for op in results {
                phi.ops.push_back(op);
            }
            res
        };

        let block_producers = &mut self.producers[block.index as usize];
        block_producers.cached.insert(reg, res);
        Some(res)
    }
}
