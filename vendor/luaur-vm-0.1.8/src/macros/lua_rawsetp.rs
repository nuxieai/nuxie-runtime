//! Source: `VM/include/lua.h:442` (hand-ported)
// #define lua_rawsetp(L, idx, p) lua_rawsetptagged(L, idx, p, 0)
#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_rawsetp {
    ($L:expr, $idx:expr, $p:expr) => {
        $crate::functions::lua_rawsetptagged::lua_rawsetptagged($L, $idx, $p, 0)
    };
}
pub use lua_rawsetp;
