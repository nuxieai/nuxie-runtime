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

    struct UpstreamRand(u64);

    impl UpstreamRand {
        fn new(seed: u32) -> Self {
            Self(u64::from(seed.wrapping_sub(1)))
        }

        fn next(&mut self) -> u32 {
            self.0 = 6_364_136_223_846_793_005_u64
                .wrapping_mul(self.0)
                .wrapping_add(1);
            (self.0 >> 33) as u32
        }

        fn unit(&mut self) -> f32 {
            self.next() as f32 / 2_147_483_647.0
        }
    }

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

    #[test]
    fn upstream_find_max_scale_complete_randomized_sequence() {
        let identity = Mat2D::IDENTITY;
        assert_eq!(identity.find_max_scale(), 1.0);

        let scale = Mat2D([2.0, 0.0, 0.0, 4.0, 0.0, 0.0]);
        assert_eq!(scale.find_max_scale(), 4.0);

        let transpose = Mat2D([0.0, 3.0, 6.0, 0.0, f32::NAN, f32::INFINITY]);
        assert_eq!(transpose.find_max_scale(), 6.0);

        let rot90_scale = Mat2D([0.25, 0.0, 0.0, 0.5, 0.0, 0.0])
            .multiply(Mat2D::from_rotation(std::f32::consts::PI / 2.0));
        assert_eq!(rot90_scale.find_max_scale(), 0.5);

        let rotate = Mat2D::from_rotation(128.0 * std::f32::consts::PI / 180.0);
        assert!((rotate.find_max_scale() - 1.0).abs() <= 1.0 / 4096.0);

        let translate = Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, -5.0]);
        assert_eq!(translate.find_max_scale(), 1.0);

        let big = Mat2D([
            2.393_940_9e36,
            3.915_962e36,
            8.853_478e36,
            1.448_234_5e37,
            9.265_262e36,
            1.515_593_4e37,
        ]);
        assert_eq!(big.find_max_scale(), 0.0);

        let base_matrices = [scale, rot90_scale, rotate, translate];
        let mut matrices = [Mat2D::IDENTITY; 8];
        for i in 0..base_matrices.len() {
            matrices[i] = base_matrices[i];
            matrices[i + base_matrices.len()] = base_matrices[i].invert_or_identity();
            assert_ne!(base_matrices[i].determinant(), 0.0);
        }

        let mut random = UpstreamRand::new(0);
        for _ in 0..1000 {
            let mut matrix = Mat2D::IDENTITY;
            for _ in 0..4 {
                let index = random.next() as usize % matrices.len();
                matrix = matrices[index].multiply(matrix);
            }

            let max_scale = matrix.find_max_scale();
            assert!(max_scale >= 0.0);

            const VECTOR_SCALE_TOLERANCE: f32 = 1.05;
            const CLOSE_SCALE_TOLERANCE: f32 = 0.97;
            let mut max = 0.0_f32;
            let mut min = f32::MAX;
            let mut vectors = [(0.0_f32, 0.0_f32); 1000];
            for vector in &mut vectors {
                let mut x = random.unit() * 2.0 - 1.0;
                let mut y = random.unit() * 2.0 - 1.0;
                let length = x.hypot(y);
                x /= length;
                y /= length;
                *vector = matrix.transform_direction(x, y);
            }
            for (x, y) in vectors {
                let length = x.hypot(y);
                assert!(length / max_scale < VECTOR_SCALE_TOLERANCE);
                max = max.max(length);
                min = min.min(length);
            }
            assert!(max / max_scale >= CLOSE_SCALE_TOLERANCE);
            let _ = min;
        }
    }
}
