use crate::records::bc_block::BcBlock;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use crate::records::jump_info::JumpInfo;
use luaur_common::functions::is_jump_d::isJumpD;
use luaur_common::functions::is_skip_c::isSkipC;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn patch_jump(&mut self, jump: &JumpInfo) {
        let target = self.func.block_op(jump.targetBlock);
        LUAU_ASSERT!(target.startpc != BcBlock::K_BLOCK_NO_START_PC);

        if isJumpD(jump.op) {
            let patched = self
                .bcb
                .patch_jump_d(jump.instructionPC as usize, target.startpc as usize);
            LUAU_ASSERT!(patched);
        } else if isSkipC(jump.op) {
            let patched = self
                .bcb
                .patch_skip_c(jump.instructionPC as usize, target.startpc as usize);
            LUAU_ASSERT!(patched);
        }
    }
}
