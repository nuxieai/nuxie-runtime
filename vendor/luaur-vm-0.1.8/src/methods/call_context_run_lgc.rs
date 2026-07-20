use crate::functions::shrinkstack::shrinkstack;
use crate::records::call_context_lgc::CallContext;
use crate::type_aliases::lua_state::lua_State;

impl CallContext {
    #[allow(non_snake_case)]
    pub unsafe extern "C" fn run(l: *mut lua_State, _ud: *mut core::ffi::c_void) {
        shrinkstack(l);
    }
}
