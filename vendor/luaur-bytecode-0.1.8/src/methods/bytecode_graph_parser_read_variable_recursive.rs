use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

impl BytecodeGraphParser<'_> {
    pub fn read_variable_recursive(&mut self, block: BcOp, reg: Reg) -> BcOp {
        let block_index = block.index as usize;

        if !self.producers[block_index].sealed {
            let phi_op = self.make_phi(block, reg);
            self.producers[block_index]
                .incomplete_phis
                .insert(reg, phi_op);
            self.producers[block_index].cached.insert(reg, phi_op);
            return phi_op;
        }

        let predecessors = self.func.blocks[block_index].predecessors.clone();
        let undefined = BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmReg, reg as u32);

        if predecessors.is_empty() {
            return undefined;
        }

        if predecessors.len() == 1 {
            self.producers[block_index].cached.insert(reg, undefined);
            let value = self
                .read_variable(predecessors[0].target, reg)
                .unwrap_or(undefined);
            self.producers[block_index].cached.insert(reg, value);
            return value;
        }

        let phi_op = self.make_phi(block, reg);
        self.producers[block_index].cached.insert(reg, phi_op);
        let value = self.add_phi_operands(reg, phi_op, block);
        self.producers[block_index].cached.insert(reg, value);
        value
    }
}
