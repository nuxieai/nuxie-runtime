use crate::records::bc_inst_helper::BcInstHelper;
use crate::type_aliases::reg::Reg;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcInstHelper<'_> {
    pub fn get_out_reg(&self) -> Reg {
        let it = self.graph.regs.get(&self.inst.op);
        LUAU_ASSERT!(it.is_some());
        *it.unwrap()
    }
}
