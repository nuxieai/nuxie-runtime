use crate::functions::dumpgco::dumpgco;
use crate::functions::dumpref::dumpref;
use crate::functions::lua_m_visitgco::lua_m_visitgco;
use crate::macros::gcvalue::gcvalue;
use crate::macros::lua_memory_categories::LUA_MEMORY_CATEGORIES;
use crate::macros::obj_2_gco::obj2gco;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int, c_void};

#[allow(non_snake_case)]
pub unsafe fn lua_c_dump(
    l: *mut lua_State,
    file: *mut c_void,
    category_name: Option<unsafe extern "C" fn(*mut lua_State, u8) -> *const c_char>,
) {
    let g = (*l).global;
    let f = file;

    extern "C" {
        fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    }

    fprintf(f, b"{\"objects\":{\n\0".as_ptr() as *const c_char);

    // The C++ code uses obj2gco(g->mainthread).
    // In Rust, obj2gco! expects a pointer to a type where (*p).tt exists or is accessible.
    // Since lua_State's first field is hdr (GCheader) which contains tt, we cast to *mut GCObject directly
    // to satisfy the macro's expectation of a collectable object pointer.
    let mainthread_gco = (*g).mainthread as *mut crate::records::gc_object::GCObject;
    dumpgco(f, core::ptr::null_mut(), mainthread_gco);

    lua_m_visitgco(l, f, dumpgco as *mut c_void);

    fprintf(
        f,
        b"\"0\":{\"type\":\"userdata\",\"cat\":0,\"size\":0}\n},\"roots\":{\n\"mainthread\":\0"
            .as_ptr() as *const c_char,
    );
    dumpref(f, mainthread_gco);
    fprintf(f, b",\"registry\":\0".as_ptr() as *const c_char);
    dumpref(f, gcvalue!(&(*g).registry));

    fprintf(
        f,
        b"},\"stats\":{\n\"size\":%d,\n\0".as_ptr() as *const c_char,
        (*g).totalbytes as c_int,
    );

    fprintf(f, b"\"categories\":{\n\0".as_ptr() as *const c_char);
    for i in 0..LUA_MEMORY_CATEGORIES {
        let bytes = (*g).memcatbytes[i as usize];
        if bytes != 0 {
            if let Some(cat_name_fn) = category_name {
                let name = cat_name_fn(l, i as u8);
                fprintf(
                    f,
                    b"\"%d\":{\"name\":\"%s\", \"size\":%d},\n\0".as_ptr() as *const c_char,
                    i,
                    name,
                    bytes as c_int,
                );
            } else {
                fprintf(
                    f,
                    b"\"%d\":{\"size\":%d},\n\0".as_ptr() as *const c_char,
                    i,
                    bytes as c_int,
                );
            }
        }
    }
    fprintf(f, b"\"none\":{}\n}\n}}\n\0".as_ptr() as *const c_char);
}

#[allow(non_snake_case)]
pub unsafe fn luaC_dump(
    L: *mut lua_State,
    file: *mut c_void,
    categoryName: Option<unsafe extern "C" fn(*mut lua_State, u8) -> *const c_char>,
) {
    lua_c_dump(L, file, categoryName);
}
