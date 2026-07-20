use crate::functions::dumpobj::dumpobj;
use crate::functions::dumpref::dumpref;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use core::ffi::c_void;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpgco(
    context: *mut c_void,
    _page: *mut lua_Page,
    gco: *mut GCObject,
) -> bool {
    let f = context as *mut core::ffi::c_void;

    dumpref(f, gco);
    extern "C" {
        fn fputc(c: core::ffi::c_int, stream: *mut core::ffi::c_void) -> core::ffi::c_int;
    }
    fputc(':' as core::ffi::c_int, f);
    dumpobj(f, gco);
    fputc(',' as core::ffi::c_int, f);
    fputc('\n' as core::ffi::c_int, f);

    false
}
