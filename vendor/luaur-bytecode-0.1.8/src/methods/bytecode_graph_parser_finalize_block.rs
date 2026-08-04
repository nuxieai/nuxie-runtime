use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl BytecodeGraphParser<'_> {
    pub fn finalize_block(&mut self, block: BcOp) {
        let successors = self.func.blocks[block.index as usize].successors.clone();
        for successor in &successors {
            let successor_index = successor.target.index as usize;
            if self.producers[successor_index].unsealed_preds > 0 {
                self.producers[successor_index].unsealed_preds -= 1;
                if self.producers[successor_index].unsealed_preds == 0 {
                    self.seal_block(successor.target);
                }
            }
        }
    }
}
