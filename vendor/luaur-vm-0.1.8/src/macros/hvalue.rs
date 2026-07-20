use crate::macros::check_exp::check_exp;
use crate::macros::ttistable::ttistable;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! hvalue {
    ($o:expr) => {
        // C++ `&(o)->value.gc->h` — a Table* into the GCObject union payload.
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttistable::ttistable!($o),
            core::ptr::addr_of_mut!((*(*$o).value.gc).h)
                as *mut $crate::records::lua_table::LuaTable
        )
    };
}

pub use hvalue;
