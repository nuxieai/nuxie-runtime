use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

impl BytecodeGraphParser<'_> {
    pub fn add_phi_operands(&mut self, reg: Reg, phi_op: BcOp, block: BcOp) -> BcOp {
        luaur_common::LUAU_ASSERT!(phi_op.kind == BcOpKind::Phi);

        let predecessors = self.func.blocks[block.index as usize].predecessors.clone();
        for predecessor in &predecessors {
            if let Some(value) = self.read_variable(predecessor.target, reg) {
                self.func.add_use_phi(phi_op, value);
            }
        }

        self.try_remove_trivial_phi(phi_op)
    }
}
