use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;
use std::collections::HashSet;

impl<'a> BytecodeGraphParser<'a> {
    pub fn find_forward_producer_in_range_bc_op_bc_op_bc_op_reg(
        &mut self,
        range_start: BcOp,
        range_end: BcOp,
        start_op: BcOp,
        reg: Reg,
    ) -> Option<BcOp> {
        let mut visited: HashSet<BcOp, BcOpHash> = HashSet::default();
        self.find_forward_producer_in_range_bc_op_bc_op_bc_op_reg_unordered_set_bc_op_bc_op_hash(
            range_start,
            range_end,
            start_op,
            reg,
            &mut visited,
        )
    }
}
