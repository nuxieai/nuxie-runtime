use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl<'a> BytecodeGraphParser<'a> {
    pub fn make_block(&mut self, pc: u32) -> BcOp {
        let new_block_op = self.func.add_block();
        *self.block_by_pc.get_or_insert(pc) = new_block_op;
        let new_block = self.func.block_op(new_block_op);
        new_block.sortkey = pc;
        new_block_op
    }
}
