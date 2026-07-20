use crate::macros::check_exp::check_exp;
use crate::macros::ttisvector::ttisvector;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! vvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisvector::ttisvector!($o),
            // C's `(o)->value.v` is a float* into value.v that consumers read up to
            // [2]/[3] via the contiguous value+extra layout. Return a reference over
            // the whole component run (NOT a by-value [f32;2] copy, whose temporary
            // would make .as_ptr().offset(2) read freed stack — the z-garbage bug).
            &*(($o) as *const $crate::type_aliases::t_value::TValue
                as *const [f32; $crate::macros::lua_vector_size::LUA_VECTOR_SIZE as usize])
        )
    };
}

pub use vvalue;
