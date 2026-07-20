use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! savestack {
    ($L:expr, $p:expr) => {{
        let L_ptr = $L as *mut $crate::records::lua_state::lua_State;
        let p_ptr = $p as *mut $crate::type_aliases::t_value::TValue;
        $crate::macros::check_exp::check_exp!(
            (p_ptr as usize >= (*L_ptr).stack as usize)
                && (p_ptr as usize
                    <= (*L_ptr).stack as usize
                        + ((*L_ptr).stacksize as usize
                            * core::mem::size_of::<$crate::type_aliases::t_value::TValue>())),
            p_ptr as isize - (*L_ptr).stack as isize
        )
    }};
}

pub use savestack;
