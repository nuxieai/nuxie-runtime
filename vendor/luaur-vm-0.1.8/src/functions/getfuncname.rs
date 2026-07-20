use crate::macros::getstr::getstr;
use crate::records::closure::Closure;
use crate::type_aliases::proto::Proto;

pub fn getfuncname(cl: *mut Closure) -> *const core::ffi::c_char {
    unsafe {
        if cl.is_null() {
            return core::ptr::null();
        }

        if (*cl).isC != 0 {
            let c_debugname = (&(*cl).inner.c).debugname;
            if !c_debugname.is_null() {
                c_debugname
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
