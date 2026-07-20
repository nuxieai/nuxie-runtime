use crate::macros::cast_byte::cast_byte;
use crate::macros::lua_c_white::luaC_white;
use crate::macros::maskmarks::maskmarks;
use crate::records::gc_object::GCObject;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! makewhite {
    ($g:expr, $x:expr) => {
        (*$x).gch.marked = $crate::macros::cast_byte::cast_byte!(
            ((*$x).gch.marked & $crate::macros::maskmarks::maskmarks!())
                | $crate::macros::lua_c_white::luaC_white!($g)
        )
    };
}

pub use makewhite;
