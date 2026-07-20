use crate::functions::lua_s_resize::luaS_resize;
use crate::records::call_context_lgc_alt_d::CallContext;
use crate::type_aliases::lua_state::lua_State;

impl CallContext {
    #[allow(non_snake_case)]
    pub unsafe fn run(l: *mut lua_State, ud: *mut core::ffi::c_void) {
        let ctx = ud as *mut CallContext;
        luaS_resize(l, (*ctx).newsize);
    }
}
