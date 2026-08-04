use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphParser<'a> {
    pub fn find_producers_up_to_top(&mut self, block: BcOp, reg: Reg) -> Vec<BcOp> {
        // We assume it called only for search of var return calls.
        LUAU_ASSERT!(block.index < self.producers.len() as u32);

        let multi_return_start;
        let multi_return;
        {
            let block_producers = &self.producers[block.index as usize];
            LUAU_ASSERT!(
                block_producers.multiReturn.kind == crate::enums::bc_op_kind::BcOpKind::Inst
            );
            multi_return_start = block_producers.multiReturnStart;
            multi_return = block_producers.multiReturn;
        }

        // So we need to find all producers from reg to blockProducers.multiReturnStart.
        let mut res = Vec::new();
        res.reserve(multi_return_start as usize - reg as usize + 1);

        let mut r = reg;
        while r < multi_return_start {
            let static_reg_op = self.find_producer_bc_op_reg(block, r);
            LUAU_ASSERT!(static_reg_op.is_some());
            res.push(static_reg_op.unwrap());
            r += 1;
        }

        res.push(multi_return);

        // multireturn is consumed, clean it up
        let block_producers = &mut self.producers[block.index as usize];
        block_producers.multiReturn = BcOp::new();
        block_producers.multiReturnStart = 0xFF;

        res
    }
}
