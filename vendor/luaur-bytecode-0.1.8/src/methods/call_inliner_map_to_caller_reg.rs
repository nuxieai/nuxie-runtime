use crate::records::call_inliner::CallInliner;
use crate::type_aliases::reg::Reg;

impl<'a> CallInliner<'a> {
    pub fn map_to_caller_reg(&self, reg: Reg) -> Reg {
        let vararg_offset = if self.target.is_vararg {
            self.call_params.len() as u8
        } else {
            0
        };

        let caller_reg = self.target_reg + 1 + vararg_offset + reg;
        luaur_common::LUAU_ASSERT!(caller_reg < self.caller.maxstacksize);
        caller_reg
    }
}
