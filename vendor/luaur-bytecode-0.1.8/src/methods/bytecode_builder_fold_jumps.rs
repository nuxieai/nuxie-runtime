use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::jump::Jump;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_d::LUAU_INSN_D;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

impl BytecodeBuilder {
    pub fn fold_jumps(&mut self) {
        // if our function has long jumps, some processing below can make jump instructions not-jumps (e.g. JUMP->RETURN)
        // it's safer to skip this processing
        if self.has_long_jumps {
            return;
        }

        for jump in &mut self.jumps {
            let jump_label: u32 = jump.source;
            let jump_insn: u32 = self.insns[jump_label as usize];

            // follow jump target through forward unconditional jumps
            // we only follow forward jumps to make sure the process terminates
            // NB: C++ computes this with SIGNED `int` — `LUAU_INSN_D` is the signed
            // jump offset (negative for backward jumps), so `jumpLabel + 1 + D`
            // must be signed arithmetic. The model used `u32` with `D as u32`,
            // which overflows on any backward jump (every loop's back-edge).
            let mut target_label: i32 = jump_label as i32 + 1 + LUAU_INSN_D(jump_insn);
            LUAU_ASSERT!((target_label as usize) < self.insns.len());
            let mut target_insn: u32 = self.insns[target_label as usize];

            while LUAU_INSN_OP(target_insn) == LuauOpcode::LOP_JUMP as u32
                && LUAU_INSN_D(target_insn) >= 0
            {
                target_label = target_label + 1 + LUAU_INSN_D(target_insn);
                LUAU_ASSERT!((target_label as usize) < self.insns.len());
                target_insn = self.insns[target_label as usize];
            }

            let offset: i32 = target_label - jump_label as i32 - 1;

            // for unconditional jumps to RETURN, we can replace JUMP with RETURN
            if LUAU_INSN_OP(jump_insn) == LuauOpcode::LOP_JUMP as u32
                && LUAU_INSN_OP(target_insn) == LuauOpcode::LOP_RETURN as u32
            {
                self.insns[jump_label as usize] = target_insn;
            } else if (offset as i16) as i32 == offset {
                let mut insn = self.insns[jump_label as usize];
                insn &= 0xffff;
                insn |= ((offset as u16) as u32) << 16;
                self.insns[jump_label as usize] = insn;
            }

            jump.target = target_label as u32;
        }
    }
}
