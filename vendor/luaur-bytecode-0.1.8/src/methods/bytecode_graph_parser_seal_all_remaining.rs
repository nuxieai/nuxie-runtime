use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl BytecodeGraphParser<'_> {
    pub fn seal_all_remaining(&mut self) {
        for block_index in 0..self.func.blocks.len() {
            if !self.producers[block_index].sealed {
                self.seal_block(BcOp::bc_op_bc_op_kind_u32(
                    BcOpKind::Block,
                    block_index as u32,
                ));
            }
        }
    }
}
