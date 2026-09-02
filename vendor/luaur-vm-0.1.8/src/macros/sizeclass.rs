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
                as i32
        } else {
            // `c_char` is unsigned on Android/ARM64. Returning the C table's
            // element type here would turn the `-1` sentinel into 255 and
            // make a zero-size realloc attempt to free a null small block.
            -1_i32
        }
    }};
}

pub use sizeclass;

#[cfg(test)]
mod tests {
    use crate::macros::sizeclass::sizeclass;
    use crate::records::size_class_config::kMaxSmallSize;

    #[test]
    fn returns_a_signed_no_class_sentinel() {
        let zero: i32 = sizeclass!(0usize);
        let oversized_size = std::hint::black_box(kMaxSmallSize + 1);
        let oversized: i32 = sizeclass!(oversized_size);

        assert_eq!(zero, -1);
        assert_eq!(oversized, -1);
    }

    #[test]
    fn preserves_valid_small_class_indexes() {
        let smallest: i32 = sizeclass!(1usize);
        let largest: i32 = sizeclass!(kMaxSmallSize);

        assert_eq!(smallest, 0);
        assert!(largest >= 0);
    }
}
