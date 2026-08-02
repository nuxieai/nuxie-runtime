use super::mat2d::Mat2D;

impl Mat2D {
    /// Largest singular value of this matrix's linear 2×2 portion.
    ///
    /// Literal port of pinned C++ `Mat2D::findMaxScale`
    /// (`src/math/mat2d_find_max_scale.cpp:25-71`). Translation is ignored,
    /// the axis-aligned fast path is preserved, and a non-finite intermediate
    /// returns zero exactly like the oracle.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn find_max_scale(self) -> f32 {
        let [xx, xy, yx, yy, _, _] = self.0;
        if xy == 0.0 && yx == 0.0 {
            return xx.abs().max(yy.abs());
        }

        let a = xx * xx + xy * xy;
        let b = xx * yx + yy * xy;
        let c = yx * yx + yy * yy;
        let b_squared = b * b;
        const CPP_EPSILON: f32 = 1.0 / 4096.0;
        let mut result = if b_squared <= CPP_EPSILON * CPP_EPSILON {
            a.max(c)
        } else {
            let a_minus_c = a - c;
            let a_plus_c_over_two = (a + c) * 0.5;
            let x = (a_minus_c * a_minus_c + 4.0 * b_squared).sqrt() * 0.5;
            a_plus_c_over_two + x
        };
        if !result.is_finite() {
            result = 0.0;
        }
        result.max(0.0).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::Mat2D;

    #[test]
    fn find_max_scale_matches_cpp_direct_fixtures() {
        // Direct fixtures from pinned C++
        // `tests/unit_tests/runtime/mat2d_test.cpp:49-101`.
        assert_eq!(Mat2D::IDENTITY.find_max_scale(), 1.0);
        assert_eq!(Mat2D([2.0, 0.0, 0.0, 4.0, 0.0, 0.0]).find_max_scale(), 4.0);
        assert_eq!(
            Mat2D([0.0, 3.0, 6.0, 0.0, f32::NAN, f32::INFINITY]).find_max_scale(),
            6.0
        );

        let rotate = Mat2D::from_rotation(128.0 * std::f32::consts::PI / 180.0);
        assert!((rotate.find_max_scale() - 1.0).abs() <= 1.0 / 4096.0);
        assert_eq!(
            Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, -5.0]).find_max_scale(),
            1.0
        );
        assert_eq!(
            Mat2D([
                2.393_940_9e36,
                3.915_962e36,
                8.853_478e36,
                1.448_234_5e37,
                9.265_262e36,
                1.515_593_4e37,
            ])
            .find_max_scale(),
            0.0
        );
    }
}
