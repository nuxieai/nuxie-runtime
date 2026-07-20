use crate::type_aliases::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::up_val::UpVal;

#[allow(non_snake_case)]
pub unsafe fn correctstack(L: *mut lua_State, oldstack: *mut TValue) {
    let stack_bytes = (*L).stack as *mut u8;
    let oldstack_addr = oldstack as isize;

    (*L).top = stack_bytes.wrapping_offset((*L).top as isize - oldstack_addr) as *mut TValue;

    let mut up: *mut UpVal = (*L).openupval;
    while !up.is_null() {
        (*up).v = stack_bytes.wrapping_offset((*up).v as isize - oldstack_addr) as *mut TValue;
        up = (*up).u.open.threadnext;
    }

    let mut ci: *mut CallInfo = (*L).base_ci;
    while ci <= (*L).ci {
        (*ci).top = stack_bytes.wrapping_offset((*ci).top as isize - oldstack_addr) as *mut TValue;
        (*ci).base =
            stack_bytes.wrapping_offset((*ci).base as isize - oldstack_addr) as *mut TValue;
        (*ci).func =
            stack_bytes.wrapping_offset((*ci).func as isize - oldstack_addr) as *mut TValue;
        ci = ci.wrapping_add(1);
    }

    (*L).base = stack_bytes.wrapping_offset((*L).base as isize - oldstack_addr) as *mut TValue;
}
