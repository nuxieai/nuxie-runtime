use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn VM_PROTECT_PC(L: *mut lua_State, pc: *const u32) {
    (*(*L).ci).savedpc = pc;
}
