use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_helper::BcInstHelper;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BcInstHelper<'a> {
    pub fn operator_deref(&self) -> &BcInst {
        LUAU_ASSERT!((self.inst.op.index as usize) < self.inst.vec.len());
        &self.inst.vec[self.inst.op.index as usize]
    }
}
