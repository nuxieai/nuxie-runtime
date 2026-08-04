use crate::records::bc_inst::BcInst;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_reg_input(&mut self, insn: &mut BcInst, index: u8) -> u8 {
        LUAU_ASSERT!((index as usize) < insn.ops.len());
        self.get_register(insn.ops[index as usize]).into()
    }
}
