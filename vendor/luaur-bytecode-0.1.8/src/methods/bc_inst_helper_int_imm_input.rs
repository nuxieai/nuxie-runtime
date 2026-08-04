use crate::records::bc_inst_helper::BcInstHelper;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcInstHelper<'_> {
    pub fn int_imm_input(&mut self, input_idx: u32) -> i32 {
        let inst = self.inst.operator_deref();
        LUAU_ASSERT!((input_idx as usize) < inst.ops.len());

        let op = inst.ops[input_idx as usize];
        let imm = self.graph.imm_op(op);

        unsafe { imm.value.valueInt }
    }
}
