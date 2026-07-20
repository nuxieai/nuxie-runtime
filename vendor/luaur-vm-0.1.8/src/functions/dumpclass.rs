use crate::functions::dumpref::dumpref;
use crate::functions::dumprefs::dumprefs;
use crate::functions::dumpstringdata::dumpstringdata;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::gc_object::GCObject;
use crate::records::luau_class::LuauClass;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpclass(f: *mut core::ffi::c_void, lco: *mut LuauClass) {
    let lco = &*lco;

    extern "C" {
        fn fprintf(
            stream: *mut core::ffi::c_void,
            format: *const core::ffi::c_char,
            ...
        ) -> core::ffi::c_int;
        fn fputc(character: core::ffi::c_int, stream: *mut core::ffi::c_void) -> core::ffi::c_int;
    }

    fprintf(
        f,
        b"{\"type\":\"class\",\"cat\":%d,\"size\":%d\0".as_ptr() as *const core::ffi::c_char,
        lco.memcat as core::ffi::c_int,
        core::mem::size_of::<LuauClass>() as core::ffi::c_int,
    );

    fprintf(f, b",\"name\":\0".as_ptr() as *const core::ffi::c_char);
    dumpstringdata(f, (*lco.name).data.as_ptr(), (*lco.name).len as usize);

    fprintf(
        f,
        b",\"membernames\":[\0".as_ptr() as *const core::ffi::c_char,
    );
    for i in 0..lco.numberofallmembers {
        if i != 0 {
            fputc(',' as core::ffi::c_int, f);
        }
        dumpref(f, *lco.offsettomember.add(i as usize) as *mut GCObject);
    }

    fprintf(
        f,
        b"],\"staticmembers\":[\0".as_ptr() as *const core::ffi::c_char,
    );
    dumprefs(
        f,
        lco.staticmembers,
        (lco.numberofallmembers - lco.numberofinstancemembers) as usize,
    );

    fprintf(
        f,
        b"],\"metatable\":\0".as_ptr() as *const core::ffi::c_char,
    );
    dumpref(f, lco.metatable as *mut GCObject);

    fprintf(
        f,
        b",\"instancemetatable\":\0".as_ptr() as *const core::ffi::c_char,
    );
    if !lco.instancemetatable.is_null() {
        dumpref(f, lco.instancemetatable as *mut GCObject);
    } else {
        fprintf(f, b"null\0".as_ptr() as *const core::ffi::c_char);
    }

    fprintf(
        f,
        b",\"memberstooffset\":\0".as_ptr() as *const core::ffi::c_char,
    );
    dumpref(f, lco.memberstooffset as *mut GCObject);

    fprintf(f, b"}\0".as_ptr() as *const core::ffi::c_char);
}
