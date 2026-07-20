//! Source: `VM/src/ldo.h` — #define restoreci(L, n) ((CallInfo*)((char*)L->base_ci + (n)))
//! (hand-fixed alongside saveci)

#[allow(non_snake_case)]
#[macro_export]
macro_rules! restoreci {
    ($L:expr, $n:expr) => {
        (((*$L).base_ci as *mut u8).offset($n as isize)
            as *mut $crate::records::call_info::CallInfo)
    };
}

pub use restoreci;
