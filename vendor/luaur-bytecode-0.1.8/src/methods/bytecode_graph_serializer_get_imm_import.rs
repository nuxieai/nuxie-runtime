use crate::enums::bc_imm_kind::BcImmKind;
use crate::records::bc_inst::BcInst;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_imm_import(&mut self, insn: &mut BcInst, index: u8) -> u32 {
        let imm = self.get_imm(insn, index);
        LUAU_ASSERT!(imm.kind == BcImmKind::Import);
        unsafe { imm.value.valueImport }
    }
}
