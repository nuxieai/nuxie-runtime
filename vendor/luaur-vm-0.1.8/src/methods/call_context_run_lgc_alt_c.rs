use crate::functions::lua_h_resizehash::lua_h_resizehash;
use crate::records::call_context_lgc_alt_c::CallContext;
use crate::type_aliases::lua_state::lua_State;

impl CallContext {
    #[allow(non_snake_case)]
    pub unsafe fn run(l: *mut lua_State, ud: *mut core::ffi::c_void) {
        let ctx = ud as *mut CallContext;
        lua_h_resizehash(l, (*ctx).t, (*ctx).nhsize);
    }
}
