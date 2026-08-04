use crate::records::bc_inst_helper::BcInstHelper;
use crate::type_aliases::reg::Reg;

impl BcInstHelper<'_> {
    pub fn set_out_reg(&mut self, out: Reg) {
        self.graph.regs.insert(self.inst.op, out);
    }
}
