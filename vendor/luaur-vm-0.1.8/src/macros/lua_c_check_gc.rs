//! Source: `VM/src/lgc.h:77` (hand-ported)
// #define luaC_checkGC(L)
//     { condhardstacktests(...); if (luaC_needsGC(L)) { condhardmemtests(...); luaC_step(L, true); }
//       else { condhardmemtests(...); } }
// condhard*tests are no-ops in default builds.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_checkGC {
    ($L:expr) => {
        if $crate::macros::lua_c_needs_gc::luaC_needsGC!($L) {
            $crate::functions::lua_c_step::luaC_step($L, true);
        }
    };
}
pub use luaC_checkGC;
#[allow(unused_imports)]
pub use luaC_checkGC as lua_c_check_gc;
