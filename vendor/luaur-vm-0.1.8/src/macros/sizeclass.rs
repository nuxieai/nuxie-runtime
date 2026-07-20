use crate::records::size_class_config::SizeClassConfig;

#[allow(non_upper_case_globals)]
pub const SIZECLASS: () = ();

#[macro_export]
macro_rules! sizeclass {
    ($sz:expr) => {{
        // (size_t((sz) - 1) < kMaxSmallSizeUsed ? kSizeClassConfig.classForSize[sz] : -1)
        let __sz = $sz;
        let __idx = (__sz as usize).wrapping_sub(1);
        if __idx < crate::records::size_class_config::kMaxSmallSize as usize {
            crate::records::size_class_config::kSizeClassConfig.classForSize[__sz as usize]
        } else {
            -1 as core::ffi::c_char
        }
    }};
}

pub use sizeclass;
