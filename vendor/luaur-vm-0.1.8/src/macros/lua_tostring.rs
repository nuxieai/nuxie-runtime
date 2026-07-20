//! Source: `VM/include/lua.h:447` (hand-ported)
// #define lua_tostring(L, i) lua_tolstring(L, (i), NULL)
#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_tostring {
    ($L:expr, $i:expr) => {
        $crate::functions::lua_tolstring::lua_tolstring($L, $i, core::ptr::null_mut())
    };
}
pub use lua_tostring;
