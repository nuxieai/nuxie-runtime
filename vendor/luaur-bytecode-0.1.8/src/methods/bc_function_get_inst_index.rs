use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn get_inst_index(&self, inst: &BcInst) -> u32 {
        let inst_ptr = inst as *const BcInst;
        let start_ptr = self.instructions.as_ptr();
        let end_ptr = unsafe { start_ptr.add(self.instructions.len()) };

        LUAU_ASSERT!(inst_ptr >= start_ptr && inst_ptr <= end_ptr);

        let offset = unsafe { inst_ptr.offset_from(start_ptr) };
        offset as u32
    }
}
