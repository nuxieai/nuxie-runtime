use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_imm::BcImm;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_imm(&mut self, insn: &mut BcInst, index: u8) -> &mut BcImm {
        LUAU_ASSERT!((index as usize) < insn.ops.len());
        let inp: BcOp = insn.ops[index as usize];
        LUAU_ASSERT!(inp.kind == BcOpKind::Imm);
        self.func.imm_op(inp)
    }
}
