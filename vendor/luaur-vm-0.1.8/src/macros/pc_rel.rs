use crate::macros::cast_to::cast_to;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! pcRel {
    ($pc:expr, $p:expr) => {
        if !$pc.is_null() && $pc != (*$p).code {
            $crate::macros::cast_to::cast_to!(
                core::ffi::c_int,
                ($pc as usize).wrapping_sub((*$p).code as usize) / core::mem::size_of::<u32>()
            ) - 1
        } else {
            0
        }
    };
}

pub use pcRel;
