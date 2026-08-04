use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use std::collections::HashSet;

impl<'a> BytecodeGraphParser<'a> {
    pub fn has_producer_before_bc_op_bc_op_bc_op_reg_bool_unordered_set_bc_op_bc_op_hash(
        &mut self,
        range_start: BcOp,
        range_end: BcOp,
        start_op: BcOp,
        reg: Reg,
        check_cached: bool,
        visited: &mut HashSet<BcOp, BcOpHash>,
    ) -> bool {
        LUAU_ASSERT!(start_op.kind == BcOpKind::Inst);
        visited.insert(range_end);
        LUAU_ASSERT!((range_end.index as usize) < self.producers.len());

        let block_producers = &self.producers[range_end.index as usize];
        if (reg as i32) > block_producers.invalidAfter {
            return false;
        }

        if block_producers.multiReturn.kind != BcOpKind::None
            && reg >= block_producers.multiReturnStart
        {
            return true;
        }

        if check_cached {
            if block_producers.own.contains_key(&reg) {
                return true;
            }
        } else {
            // To avoid borrowing self.func while iterating over bl.ops, we clone the ops.
            // BcOp is a small Copy type, and VecDeque clone is acceptable here to satisfy the borrow checker.
            let ops = self.func.block_op(range_end).ops.clone();
            for op in &ops {
                if *op == start_op {
                    break;
                }
                if let Some(&op_reg) = self.func.regs.get(op) {
                    if op_reg == reg {
                        return true;
                    }
                }
            }
        }

        if range_end == range_start {
            return false;
        }

        // To avoid holding a reference to bl.predecessors (which borrows self.func)
        // while calling self recursively, we clone the predecessors.
        let predecessors = self.func.block_op(range_end).predecessors.clone();
        for edge in &predecessors {
            if edge.kind == BcBlockEdgeKind::Loop || visited.contains(&edge.target) {
                continue;
            }
            if self.has_producer_before_bc_op_bc_op_bc_op_reg_bool_unordered_set_bc_op_bc_op_hash(
                range_start,
                edge.target,
                start_op,
                reg,
                true,
                visited,
            ) {
                return true;
            }
        }

        false
    }
}
