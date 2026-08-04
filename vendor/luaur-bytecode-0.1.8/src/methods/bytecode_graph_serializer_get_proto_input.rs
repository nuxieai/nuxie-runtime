use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> crate::records::bytecode_graph_serializer::BytecodeGraphSerializer<'a> {
    pub fn get_proto_input(&mut self, insn: &mut BcInst, index: u8) -> u16 {
        LUAU_ASSERT!(index < insn.ops.len() as u8);
        let inp = insn.ops[index as usize];
        LUAU_ASSERT!(inp.kind == BcOpKind::VmProto);

        if inp.index > 0xffff {
            self.error = true;
        }

        inp.index as u16
    }
}
