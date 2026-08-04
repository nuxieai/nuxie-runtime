use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::vec::Vec;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::get_jump_target::getJumpTarget;
use luaur_common::functions::get_op_length::getOpLength;
use luaur_common::functions::is_fast_call::isFastCall;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_b::LUAU_INSN_B;
use luaur_common::macros::luau_insn_c::LUAU_INSN_C;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

impl BytecodeBuilder {
    pub fn validate_variadic(&self) {
        let mut variadic_seq = false;
        let mut insn_targets = Vec::with_capacity(self.insns.len());
        insn_targets.resize(self.insns.len(), false);

        let mut i = 0;
        while i < self.insns.len() {
            let insn = self.insns[i];
            let op = LuauOpcode::from(LUAU_INSN_OP(insn) as u8);

            let target = getJumpTarget(insn, i as u32);

            if target >= 0 && !isFastCall(op) {
                LUAU_ASSERT!((target as usize) < self.insns.len());
                insn_targets[target as usize] = true;
            }

            i += getOpLength(op) as usize;
            LUAU_ASSERT!(i <= self.insns.len());
        }

        let mut i = 0;
        while i < self.insns.len() {
            let insn = self.insns[i];
            let op = LuauOpcode::from(LUAU_INSN_OP(insn) as u8);

            if variadic_seq {
                LUAU_ASSERT!(!insn_targets[i]);
            }

            if op == LuauOpcode::LOP_CALL || op == LuauOpcode::LOP_CALLFB {
                if LUAU_INSN_B(insn) == 0 {
                    LUAU_ASSERT!(variadic_seq);
                    variadic_seq = false;
                } else {
                    LUAU_ASSERT!(!variadic_seq);
                }

                if LUAU_INSN_C(insn) == 0 {
                    LUAU_ASSERT!(!variadic_seq);
                    variadic_seq = true;
                }
            } else if op == LuauOpcode::LOP_GETVARARGS && LUAU_INSN_B(insn) == 0 {
                LUAU_ASSERT!(!variadic_seq);
                variadic_seq = true;
            } else if (op == LuauOpcode::LOP_RETURN && LUAU_INSN_B(insn) == 0)
                || (op == LuauOpcode::LOP_SETLIST && LUAU_INSN_C(insn) == 0)
            {
                LUAU_ASSERT!(variadic_seq);
                variadic_seq = false;
            } else if op == LuauOpcode::LOP_FASTCALL {
                let call_target = (i as i32 + LUAU_INSN_C(insn) as i32 + 1) as usize;
                LUAU_ASSERT!(
                    call_target < self.insns.len()
                        && LuauOpcode::from(LUAU_INSN_OP(self.insns[call_target]) as u8)
                            == LuauOpcode::LOP_CALL
                );

                if LUAU_INSN_B(self.insns[call_target]) == 0 {
                    LUAU_ASSERT!(variadic_seq);
                } else {
                    LUAU_ASSERT!(!variadic_seq);
                }
            } else if op == LuauOpcode::LOP_CLOSEUPVALS
                || op == LuauOpcode::LOP_NAMECALL
                || op == LuauOpcode::LOP_GETIMPORT
                || op == LuauOpcode::LOP_MOVE
                || op == LuauOpcode::LOP_GETUPVAL
                || op == LuauOpcode::LOP_GETGLOBAL
                || op == LuauOpcode::LOP_GETTABLEKS
                || op == LuauOpcode::LOP_COVERAGE
            {
            } else {
                LUAU_ASSERT!(!variadic_seq);
            }

            i += getOpLength(op) as usize;
            LUAU_ASSERT!(i <= self.insns.len());
        }

        LUAU_ASSERT!(!variadic_seq);
    }
}
