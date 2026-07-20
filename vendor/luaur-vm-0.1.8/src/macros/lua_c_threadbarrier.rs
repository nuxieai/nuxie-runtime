use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::macros::isblack::isblack;
use crate::macros::obj_2_gco::obj2gco;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_c_threadbarrier {
    ($L:expr) => {
        unsafe {
            let obj = $crate::macros::obj_2_gco::obj2gco!($L);
            if $crate::macros::isblack::isblack!(obj) {
                $crate::functions::lua_c_barrierback::lua_c_barrierback($L, obj, &mut (*$L).gclist);
            }
        }
    };
}

pub use lua_c_threadbarrier;

// C name
#[allow(unused_imports)]
pub use lua_c_threadbarrier as luaC_threadbarrier;
