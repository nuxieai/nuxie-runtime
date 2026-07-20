use crate::enums::lua_type::lua_Type;
use crate::macros::cast_to::cast_to;
use crate::macros::ci_func::ci_func;
use crate::macros::is_lua::isLua;
use crate::records::call_info::CallInfo;
use crate::type_aliases::proto::Proto;

pub(crate) unsafe fn get_lua_proto(ci: *mut CallInfo) -> *mut Proto {
    if isLua!(ci) {
        let cl = ci_func!(ci);
        let lcl = core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
        cast_to!(*mut Proto, (*lcl).p)
    } else {
        core::ptr::null_mut()
    }
}
