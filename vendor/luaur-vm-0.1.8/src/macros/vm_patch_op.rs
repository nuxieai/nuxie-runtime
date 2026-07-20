use crate::type_aliases::instruction::Instruction;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn VM_PATCH_OP(pc: *const Instruction, op: u8) {
    *(pc as *mut Instruction) = (op as u32) | (0xffffff00u32 & *pc);
}
