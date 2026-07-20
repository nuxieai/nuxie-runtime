//! Source: `VM/include/lualib.h:104` (hand-ported)
// #define luaL_addchar(B, c) ((void)((B)->p < (B)->end || luaL_prepbuffsize(B, 1)), (*(B)->p++ = (char)(c)))
#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_addchar {
    ($B:expr, $c:expr) => {{
        if !((*$B).p < (*$B).end) {
            $crate::functions::lua_l_prepbuffsize::lua_l_prepbuffsize($B, 1);
        }
        *(*$B).p = $c as core::ffi::c_char;
        (*$B).p = (*$B).p.add(1);
    }};
}
pub use luaL_addchar;
#[allow(unused_imports)]
pub use luaL_addchar as lua_l_addchar;
