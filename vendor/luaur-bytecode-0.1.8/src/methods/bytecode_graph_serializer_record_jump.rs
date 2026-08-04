use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use crate::records::jump_info::JumpInfo;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn record_jump(&mut self, insn: &mut BcInst, index: u8) {
        LUAU_ASSERT!(index < insn.ops.len() as u8);
        let inp: BcOp = insn.ops[index as usize];
        LUAU_ASSERT!(inp.kind == BcOpKind::Block);
        self.jumps.push(JumpInfo {
            op: insn.op,
            instructionPC: self.bcb.get_instruction_count() as u32,
            targetBlock: inp,
        });
    }
}
