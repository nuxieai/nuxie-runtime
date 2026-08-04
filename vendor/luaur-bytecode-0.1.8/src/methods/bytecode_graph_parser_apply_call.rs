use crate::records::bc_op::BcOp;
use crate::records::block_producers::BlockProducers;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;

impl<'a> BytecodeGraphParser<'a> {
    pub fn apply_call(
        &mut self,
        producers: &mut BlockProducers,
        call_op: BcOp,
        target_reg: Reg,
        nresults: i32,
    ) {
        producers.own.retain(|&reg, _| reg < target_reg);
        producers.cached.retain(|&reg, _| reg < target_reg);

        if nresults < 0 {
            producers.multiReturn = call_op;
            producers.multiReturnStart = target_reg;
            producers.invalidAfter = 255;
        } else {
            producers.invalidAfter = (target_reg as i32) - 1 + nresults;
        }
    }
}
