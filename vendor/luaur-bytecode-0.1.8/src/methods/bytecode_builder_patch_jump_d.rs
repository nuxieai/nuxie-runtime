use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::jump::Jump;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::is_jump_d::isJumpD;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_d::LUAU_INSN_D;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

impl BytecodeBuilder {
    pub fn patch_jump_d(&mut self, jump_label: usize, target_label: usize) -> bool {
        LUAU_ASSERT!(jump_label < self.insns.len());

        let jump_insn = self.insns[jump_label];
        #[allow(path_statements)]
        {
            let _ = jump_insn;
        }

        LUAU_ASSERT!(isJumpD(unsafe {
            core::mem::transmute(LUAU_INSN_OP(jump_insn) as u8)
        }));
        LUAU_ASSERT!(LUAU_INSN_D(jump_insn) == 0);

        LUAU_ASSERT!(target_label <= self.insns.len());

        let offset = target_label as i32 - jump_label as i32 - 1;

        if (offset as i16) as i32 == offset {
            self.insns[jump_label] |= ((offset as u16) as u32) << 16;
        } else if offset.abs() < (1 << 23) {
            // our jump doesn't fit into 16 bits; we will need to repatch the bytecode sequence with jump trampolines, see expandJumps
            // C++ `kMaxJumpDistance = 1 << 23` (not 32767): jumps up to ~8M instructions are
            // handled via JUMPX trampolines in expandJumps; only beyond that is it an error.
            self.has_long_jumps = true;
        } else {
            return false;
        }

        self.jumps.push(Jump {
            source: jump_label as u32,
            target: target_label as u32,
        });

        true
    }
}
