use crate::type_aliases::instruction::Instruction;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn VM_PATCH_E(pc: *const Instruction, slot: i32) {
    *(pc as *mut Instruction) = ((slot as u32) << 8) | (0x000000ffu32 & *pc);
}
