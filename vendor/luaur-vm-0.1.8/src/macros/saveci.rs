//! Source: `VM/src/ldo.h` — #define saveci(L, p) ((char*)(p) - (char*)L->base_ci)
//! (hand-fixed: generated body dereferenced fields off a raw pointer without (* ))

#[allow(non_snake_case)]
#[macro_export]
macro_rules! saveci {
    ($L:expr, $p:expr) => {
        (($p as *const u8).offset_from((*$L).base_ci as *const u8)) as isize
    };
}

pub use saveci;
