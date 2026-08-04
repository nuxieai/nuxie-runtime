use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_vm_const_input_raw(&mut self, insn: &mut BcInst, index: u8) -> u32 {
        LUAU_ASSERT!((index as usize) < insn.ops.len());
        let inp: BcOp = insn.ops[index as usize];
        LUAU_ASSERT!(inp.kind == BcOpKind::VmConst);
        LUAU_ASSERT!((inp.index as usize) < self.func.constants.len());
        if let Some(consts) = &self.consts {
            LUAU_ASSERT!((inp.index as usize) < consts.len());
            consts[inp.index as usize] as u32
        } else {
            inp.index
        }
    }
}
