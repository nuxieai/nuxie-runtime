use crate::enums::luau_opcode::LuauOpcode;
use crate::functions::is_fast_call::isFastCall;
use crate::functions::is_jump_d::isJumpD;
use crate::functions::is_skip_c::isSkipC;
use crate::macros::luau_insn_c::LUAU_INSN_C;
use crate::macros::luau_insn_d::LUAU_INSN_D;
use crate::macros::luau_insn_e::LUAU_INSN_E;
use crate::macros::luau_insn_op::LUAU_INSN_OP;

#[allow(non_snake_case)]
#[inline]
pub fn getJumpTarget(insn: u32, pc: u32) -> i32 {
    let op_u8 = LUAU_INSN_OP(insn) as u8;
    if op_u8 >= LuauOpcode::LOP__COUNT as u8 {
        return -1;
    }

    // Safety: `LuauOpcode` is `#[repr(u8)]` with contiguous auto-assigned
    // discriminants `0..LOP__COUNT`, and the bounds check above guarantees
    // `op_u8` is one of those valid discriminants, so the transmute is sound.
    let op: LuauOpcode = unsafe { core::mem::transmute(op_u8) };

    if isJumpD(op) {
        (pc as i32).wrapping_add(LUAU_INSN_D(insn)).wrapping_add(1)
    } else if isFastCall(op) {
        (pc as i32)
            .wrapping_add(LUAU_INSN_C(insn) as i32)
            .wrapping_add(2)
    } else if isSkipC(op) && LUAU_INSN_C(insn) != 0 {
        (pc as i32)
            .wrapping_add(LUAU_INSN_C(insn) as i32)
            .wrapping_add(1)
    } else if op == LuauOpcode::LOP_JUMPX {
        (pc as i32).wrapping_add(LUAU_INSN_E(insn)).wrapping_add(1)
    } else {
        -1
    }
}

pub fn get_jump_target(insn: u32, pc: u32) -> i32 {
    getJumpTarget(insn, pc)
}
