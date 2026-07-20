use crate::type_aliases::instruction::Instruction;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn VM_PATCH_C(pc: *const Instruction, slot: i32) {
    *(pc as *mut Instruction) = (((slot as u8) as u32) << 24) | (0x00ffffffu32 & *pc);
}
