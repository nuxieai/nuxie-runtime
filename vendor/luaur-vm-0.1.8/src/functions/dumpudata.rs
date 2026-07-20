use crate::functions::dumpref::dumpref;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::sizeudata::sizeudata;
use crate::records::udata::Udata;
use core::ffi::{c_char, c_int, c_void};

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpudata(f: *mut c_void, u: *mut Udata) {
    let u = &*u;
    let fmt = "{\"type\":\"userdata\",\"cat\":%d,\"size\":%d,\"tag\":%d";

    extern "C" {
        fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    }

    fprintf(
        f,
        fmt.as_ptr() as *const c_char,
        u.memcat as c_int,
        sizeudata(u.len as usize) as c_int,
        u.tag as c_int,
    );

    if !u.metatable.is_null() {
        fprintf(f, b",\"metatable\":".as_ptr() as *const c_char);
        let mt = u.metatable as *mut crate::records::gc_object::GCObject;
        dumpref(f, mt);
    }

    fprintf(f, b"}".as_ptr() as *const c_char);
}
