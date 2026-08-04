use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl BytecodeGraphParser<'_> {
    pub fn seal_block(&mut self, block: BcOp) {
        let block_index = block.index as usize;
        if self.producers[block_index].sealed {
            return;
        }

        self.producers[block_index].sealed = true;
        let pending = core::mem::take(&mut self.producers[block_index].incomplete_phis);
        for (reg, phi_op) in pending {
            let value = self.add_phi_operands(reg, phi_op, block);
            if self.producers[block_index].cached.get(&reg) == Some(&phi_op) {
                self.producers[block_index].cached.insert(reg, value);
            }
        }
    }
}
