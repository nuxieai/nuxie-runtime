use crate::macros::getstr::getstr;
use crate::records::closure::Closure;
use crate::type_aliases::proto::Proto;

pub fn getfuncname(cl: *mut Closure) -> *const core::ffi::c_char {
    unsafe {
        if cl.is_null() {
            return core::ptr::null();
        }

        if (*cl).isC != 0 {
            let c = &(*cl).inner.c;
            if luaur_common::FFlag::LuauManagedDebugNames.get() && !c.debugname.is_null() {
                getstr(c.debugname)
            } else if !luaur_common::FFlag::LuauManagedDebugNames.get()
                && !c.debugname_DEPRECATED.is_null()
            {
                c.debugname_DEPRECATED
            } else {
                core::ptr::null()
            }
        } else {
            let p: *mut Proto = (&(*cl).inner.l).p;

            if !p.is_null() {
                let p_debugname = (&(*p)).debugname;
                if !p_debugname.is_null() {
                    getstr(p_debugname)
                } else {
                    core::ptr::null()
                }
            } else {
                core::ptr::null()
            }
        }
    }
}
