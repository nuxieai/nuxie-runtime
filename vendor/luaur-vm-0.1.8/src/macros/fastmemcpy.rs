use crate::macros::check_exp::check_exp;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! fastmemcpy {
    ($dst:expr, $src:expr, $size:expr, $sizefast:expr) => {
        $crate::macros::check_exp::check_exp!(($size) <= $sizefast, unsafe {
            core::ptr::copy_nonoverlapping($src as *const u8, $dst as *mut u8, $sizefast as usize)
        })
    };
}

pub use fastmemcpy;
