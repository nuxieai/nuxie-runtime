use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_upval_input(&mut self, insn: &mut BcInst, index: u8) -> u8 {
        LUAU_ASSERT!((index as usize) < insn.ops.len());
        let inp: BcOp = insn.ops[index as usize];
        LUAU_ASSERT!(inp.kind == BcOpKind::VmUpvalue);
        LUAU_ASSERT!(inp.index < self.func.nups as u32);
        inp.index as u8
    }
}
