use crate::type_aliases::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn preinit_state(L: *mut lua_State, g: *mut global_State) {
    (*L).global = g;
    (*L).stack = core::ptr::null_mut();
    (*L).stacksize = 0;
    (*L).gt = core::ptr::null_mut();
    (*L).openupval = core::ptr::null_mut();
    (*L).size_ci = 0;
    (*L).nCcalls = 0;
    (*L).baseCcalls = 0;
    (*L).status = 0;
    (*L).base_ci = core::ptr::null_mut();
    (*L).ci = core::ptr::null_mut();
    (*L).namecall = core::ptr::null_mut();
    (*L).cachedslot = 0;
    (*L).singlestep = false;
    (*L).isactive = false;
    (*L).activememcat = 0;
    (*L).userdata = core::ptr::null_mut();
}
