use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

impl BytecodeGraphParser<'_> {
    pub fn read_variable(&mut self, block: BcOp, reg: Reg) -> Option<BcOp> {
        let block_index = block.index as usize;

        if reg as i32 > self.producers[block_index].invalidAfter {
            return None;
        }
        if let Some(local) = self.producers[block_index].own.get(&reg) {
            return Some(*local);
        }
        if let Some(cached) = self.producers[block_index].cached.get(&reg) {
            return Some(*cached);
        }

        let multi_return = self.producers[block_index].multiReturn;
        let multi_return_start = self.producers[block_index].multiReturnStart;
        if multi_return.kind != BcOpKind::None && reg >= multi_return_start {
            let projection = self
                .func
                .add_proj(multi_return, (reg - multi_return_start) as u32);
            self.producers[block_index].cached.insert(reg, projection);
            return Some(projection);
        }

        Some(self.read_variable_recursive(block, reg))
    }
}
