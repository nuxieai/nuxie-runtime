use crate::type_aliases::instruction::Instruction;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn VM_PATCH_AUX_SLOT(pc: *const Instruction, k: u32, slot: i32) {
    *(pc as *mut Instruction) = k | ((slot as u32) << 16);
}
