use crate::macros::check_exp::check_exp;
use crate::macros::ttisupval::ttisupval;

#[macro_export]
macro_rules! upvalue {
    ($o:expr) => {
        $crate::macros::check_exp::check_exp!(
            $crate::macros::ttisupval::ttisupval!($o),
            &mut (*(*$o).value.gc).uv
        )
    };
}

pub use upvalue;
