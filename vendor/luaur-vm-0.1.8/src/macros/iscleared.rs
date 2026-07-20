//! Source: `VM/src/lgc.cpp:619` (hand-ported)
// #define iscleared(o) (iscollectable(o) && isobjcleared(gcvalue(o)))
#[allow(non_snake_case)]
#[macro_export]
macro_rules! iscleared {
    ($o:expr) => {
        $crate::macros::iscollectable::iscollectable!($o)
            && $crate::functions::isobjcleared::isobjcleared($crate::macros::gcvalue::gcvalue!($o))
                != 0
    };
}
pub use iscleared;
