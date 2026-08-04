use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::records::bc_phi::BcPhi;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

use luaur_common::macros::luau_assert::LUAU_ASSERT;

use std::collections::HashSet;

impl<'a> BytecodeGraphParser<'a> {
    pub fn find_forward_producer_in_range_bc_op_bc_op_bc_op_reg_unordered_set_bc_op_bc_op_hash(
        &mut self,
        range_start: BcOp,
        range_end: BcOp,
        start_op: BcOp,
        reg: Reg,
        visited: &mut HashSet<BcOp, BcOpHash>,
    ) -> Option<BcOp> {
        LUAU_ASSERT!(start_op.kind == BcOpKind::Inst);

        visited.insert(range_end);

        LUAU_ASSERT!(range_end.index < self.producers.len() as u32);
        let block_producers = &self.producers[range_end.index as usize];

        if reg as i32 > block_producers.invalidAfter {
            return None;
        }

        if let Some(local) = block_producers.own.get(&reg) {
            return Some(*local);
        }

        if range_start == range_end {
            return None;
        }

        if block_producers.multiReturn.kind != BcOpKind::None
            && reg >= block_producers.multiReturnStart
        {
            return Some(block_producers.multiReturn);
        }

        // Clone predecessors to avoid holding a borrow of self.func (via BcBlock) during recursion
        let predecessors = self.func.block_op(range_end).predecessors.clone();

        let mut results: HashSet<BcOp, BcOpHash> = HashSet::with_hasher(BcOpHash::default());

        for edge in &predecessors {
            let ctrl = edge.kind;
            let pred = edge.target;

            if ctrl == BcBlockEdgeKind::Loop || visited.contains(&pred) {
                continue;
            }

            LUAU_ASSERT!(range_end != pred);

            if let Some(op) = self.find_forward_producer_in_range_bc_op_bc_op_bc_op_reg_unordered_set_bc_op_bc_op_hash(
                range_start,
                pred,
                start_op,
                reg,
                visited,
            ) {
                if op.kind == BcOpKind::Phi {
                    let phi: &mut BcPhi = self.func.phi_op(op);
                    for &proj in &phi.ops {
                        results.insert(proj);
                    }
                } else {
                    results.insert(op);
                }
            }
        }

        if results.is_empty() {
            return None;
        }

        if results.len() == 1 {
            return results.into_iter().next();
        }

        let res = self.func.add_phi();
        let phi: &mut BcPhi = self.func.phi_op(res);
        for op in results {
            phi.ops.push_back(op);
        }

        Some(res)
    }
}
