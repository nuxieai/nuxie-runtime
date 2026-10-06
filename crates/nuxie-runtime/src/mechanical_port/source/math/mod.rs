pub mod aabb;
pub mod bezier_utils;
pub mod bit_field_loc;
pub mod bitwise;
pub mod circle_constant;
pub mod contour_measure;
pub mod cubic_utilities;
pub mod hit_test;
pub mod mat2d;
pub mod mat2d_scale;
pub mod mat4;
pub mod math_types;
pub mod n_slicer_helpers;
pub mod path_measure;
pub mod path_types;
pub mod random;
pub mod raw_path;
pub mod raw_path_utils;
pub mod rectangles_to_contour;
pub mod simd;
pub mod simd_gvec_polyfill;
pub mod transform_components;
pub mod vec2d;
pub mod wangs_formula;

/// Rust compiler boundary for ordinary source operations that the pinned
/// shipping C++ build contracts. The strict lane mirrors --no_ffp_contract;
/// it is selected explicitly, not inferred from optimization or cfg(test).
pub(crate) trait FloatContract {
    fn contracted_mul_add(self, multiplier: f32, summand: f32) -> f32;
}

impl FloatContract for f32 {
    #[inline]
    fn contracted_mul_add(self, multiplier: f32, summand: f32) -> f32 {
        #[cfg(feature = "strict-fp")]
        {
            self * multiplier + summand
        }
        #[cfg(not(feature = "strict-fp"))]
        {
            self.mul_add(multiplier, summand)
        }
    }
}
