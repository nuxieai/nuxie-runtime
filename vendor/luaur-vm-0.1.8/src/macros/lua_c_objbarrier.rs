//! Source: `VM/src/lgc.h:109` (hand-ported)
// #define luaC_objbarrier(L, p, o)
//     { if (isblack(obj2gco(p)) && iswhite(obj2gco(o))) luaC_barrierf(L, obj2gco(p), obj2gco(o)); }
// obj2gco on typed GC pointers is a plain cast in C.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_objbarrier {
    ($L:expr, $p:expr, $o:expr) => {
        if $crate::macros::isblack::isblack!($p as *mut $crate::records::gc_object::GCObject)
            && $crate::macros::iswhite::iswhite!($o as *mut $crate::records::gc_object::GCObject)
        {
            $crate::functions::lua_c_barrierf::luaC_barrierf(
                $L,
                $p as *mut $crate::records::gc_object::GCObject,
                $o as *mut $crate::records::gc_object::GCObject,
            );
        }
    };
}
pub use luaC_objbarrier;
#[allow(unused_imports)]
pub use luaC_objbarrier as lua_c_objbarrier;
