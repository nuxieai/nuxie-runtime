//! Node: `cxx:Macro:Luau.VM:VM/src/lstring.h:21:lua_s_updateatom`
//! Source: `VM/src/lstring.h` (lstring.h:21-25, hand-ported)

// #define luaS_updateatom(L, ts)
//     { if (ts->atom == ATOM_UNDEF)
//           ts->atom = L->global->cb.useratom ? L->global->cb.useratom(L, ts->data, ts->len) : -1; }
#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaS_updateatom {
    ($L:expr, $ts:expr) => {
        if (*$ts).atom as i32 == $crate::macros::atom_undef::ATOM_UNDEF {
            (*$ts).atom = match (*(*$L).global).cb.useratom {
                Some(useratom) => useratom(
                    $L,
                    core::ptr::addr_of!((*$ts).data) as *const core::ffi::c_char,
                    (*$ts).len as usize,
                ),
                None => -1,
            };
        }
    };
}

pub use luaS_updateatom;
#[allow(unused_imports)]
pub use luaS_updateatom as lua_s_updateatom;
