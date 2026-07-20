use crate::functions::lua_d_growstack::lua_d_growstack;
use crate::records::call_context_lapi::CallContext;
use crate::type_aliases::lua_state::lua_State;

impl CallContext {
    #[allow(non_snake_case)]
    pub unsafe fn run_mut(l: *mut lua_State, ud: *mut core::ffi::c_void) {
        let ctx = ud as *mut CallContext;
        lua_d_growstack(l, (*ctx).size);
    }
}
