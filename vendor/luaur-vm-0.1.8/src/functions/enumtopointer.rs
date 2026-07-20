use crate::enums::lua_type::lua_Type;
use crate::macros::gco_2_u::gco2u;
use crate::records::gc_object::GCObject;
use core::ffi::c_void;

#[inline]
pub fn enumtopointer(gco: *mut GCObject) -> *mut c_void {
    unsafe {
        if (*gco).gch.tt == (lua_Type::LUA_TUSERDATA as u8) {
            let u = gco2u!(gco);
            (*u).data.as_mut_ptr() as *mut c_void
        } else {
            gco as *mut c_void
        }
    }
}
