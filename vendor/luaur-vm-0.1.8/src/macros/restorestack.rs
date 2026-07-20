//! Source: `VM/src/ldo.h` — #define restorestack(L, n) ((TValue*)((char*)L->stack + (n)))
//! (hand-fixed: the generated body had a cast-precedence error and unqualified
//! types that break at expansion sites)

#[allow(non_snake_case)]
#[macro_export]
macro_rules! restorestack {
    ($L:expr, $n:expr) => {
        (((*$L).stack as *mut u8).offset($n as isize) as *mut $crate::type_aliases::t_value::TValue)
    };
}

pub use restorestack;
