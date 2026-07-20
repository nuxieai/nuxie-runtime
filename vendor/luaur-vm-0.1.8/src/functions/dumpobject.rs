use crate::functions::dumpref::dumpref;
use crate::functions::dumprefs::dumprefs;
use crate::macros::obj_2_gco::obj2gco;
use crate::type_aliases::luau_object::LuauObject;
use core::ffi::{c_char, c_int, c_void};

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpobject(f: *mut c_void, inst: *mut LuauObject) {
    extern "C" {
        fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    }

    let inst_ref = &*inst;

    fprintf(
        f,
        b"{\"type\":\"object\",\"cat\":%d,\"size\":%d\0".as_ptr() as *const c_char,
        inst_ref.memcat as c_int,
        core::mem::size_of::<LuauObject>() as c_int,
    );

    fprintf(f, b",\"class\":\0".as_ptr() as *const c_char);
    dumpref(f, obj2gco!(inst_ref.lclass));

    fprintf(f, b",\"members\":\0".as_ptr() as *const c_char);
    dumprefs(f, inst_ref.members, inst_ref.numberofmembers as usize);

    fprintf(f, b"]}\0".as_ptr() as *const c_char);
}
