use crate::records::bc_op::BcOp;
use crate::records::block_producers::BlockProducers;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_producer(&mut self, reg: Reg, op: BcOp) {
        let block_producers: &mut BlockProducers =
            &mut self.producers[self.current_block.index as usize];

        block_producers.own.insert(reg, op);

        self.func.regs.insert(op, reg);

        block_producers.invalidAfter = core::cmp::max(reg as i32, block_producers.invalidAfter);
    }
}
