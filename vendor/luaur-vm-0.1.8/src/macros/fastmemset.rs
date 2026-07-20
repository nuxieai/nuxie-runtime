use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! fastmemset {
    ($dst:expr, $val:expr, $size:expr, $sizefast:expr) => {
        $crate::macros::check_exp::check_exp!(($size) <= $sizefast, unsafe {
            core::ptr::write_bytes($dst as *mut u8, $val as u8, $sizefast as usize)
        })
    };
}

pub use fastmemset;
