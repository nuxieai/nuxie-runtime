//! Node: `cxx:Macro:Luau.VM:VM/src/lgc.h:91:lua_c_barrier`
//! Source: `VM/src/lgc.h`

// VM/src/lgc.h:91 —
// #define luaC_barrier(L, p, v)
//     { if (iscollectable(v) && isblack(obj2gco(p)) && iswhite(gcvalue(v)))
//           luaC_barrierf(L, obj2gco(p), gcvalue(v)); }
// obj2gco on a typed GC pointer is a plain cast in C.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_barrier {
    ($L:expr, $p:expr, $v:expr) => {
        if $crate::macros::iscollectable::iscollectable!($v)
            && $crate::macros::isblack::isblack!($p as *mut $crate::records::gc_object::GCObject)
            && $crate::macros::iswhite::iswhite!($crate::macros::gcvalue::gcvalue!($v))
        {
            $crate::functions::lua_c_barrierf::luaC_barrierf(
                $L,
                $p as *mut $crate::records::gc_object::GCObject,
                $crate::macros::gcvalue::gcvalue!($v),
            );
        }
    };
}

pub use luaC_barrier;
#[allow(unused_imports)]
pub use luaC_barrier as lua_c_barrier;
