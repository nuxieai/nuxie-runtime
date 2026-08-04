use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

impl BytecodeGraphParser<'_> {
    pub fn make_phi(&mut self, block: BcOp, reg: Reg) -> BcOp {
        let phi_op = self.func.add_phi();
        self.func.regs.insert(phi_op, reg);
        self.func.blocks[block.index as usize]
            .phis
            .push_back(phi_op);
        self.phi_block.insert(phi_op, block);
        phi_op
    }
}
