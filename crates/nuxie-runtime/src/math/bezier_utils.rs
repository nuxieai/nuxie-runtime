// Direct source-correspondence owner for pinned `src/math/bezier_utils.cpp`
// and `include/rive/math/bezier_utils.hpp`.
mod bezier_utils_owner {
    type Point = (f32, f32);

    #[derive(Debug, Clone, Copy)]
    pub(super) struct CubicCoeffs {
        pub(super) a: Point,
        pub(super) b: Point,
        pub(super) c: Point,
    }

    impl CubicCoeffs {
        pub(super) fn new(points: [Point; 4]) -> Self {
            let c = sub(points[1], points[0]);
            let d = sub(points[2], points[1]);
            let e = sub(points[3], points[0]);
            Self {
                a: scale_add(d, -3.0, e),
                b: sub(d, c),
                c,
            }
        }
    }

    // This boundary preserves `rive::math::EvalCubic` separately from the
    // unrelated `rive::EvalCubic` in `raw_path_utils.hpp`.
    #[derive(Debug, Clone, Copy)]
    pub(super) struct EvalCubic {
        pub(super) a: Point,
        pub(super) b: Point,
        pub(super) c: Point,
        d: Point,
    }

    impl EvalCubic {
        pub(super) fn new(points: [Point; 4]) -> Self {
            Self::from_coeffs(CubicCoeffs::new(points), points[0])
        }

        pub(super) fn from_coeffs(coeffs: CubicCoeffs, p0: Point) -> Self {
            Self {
                a: coeffs.a,
                b: scale(coeffs.b, 3.0),
                c: scale(coeffs.c, 3.0),
                d: p0,
            }
        }

        pub(super) fn evaluate(&self, t: f32) -> Point {
            scale_add(
                scale_add(scale_add(self.a, t, self.b), t, self.c),
                t,
                self.d,
            )
        }

        pub(super) fn evaluate_pair(&self, t: [f32; 2]) -> [Point; 2] {
            [self.evaluate(t[0]), self.evaluate(t[1])]
        }

        pub(super) fn at(&self, t: [f32; 2]) -> [Point; 2] {
            [self.evaluate(t[0]), self.evaluate(t[1])]
        }
    }

    pub(super) fn calc_polar_segments_per_radian<const PRECISION: i32>(
        approx_dev_stroke_radius: f32,
    ) -> f32 {
        let cos_theta = 1.0 - (1.0 / PRECISION as f32) / approx_dev_stroke_radius;
        0.5 / cpp_std_max(cos_theta, -1.0).acos()
    }

    pub(super) fn eval_cubic_at(points: [Point; 4], t: f32) -> Point {
        let a = sub(
            scale_add(sub(points[1], points[2]), 3.0, points[3]),
            points[0],
        );
        let b = scale(add(scale_add(points[1], -2.0, points[2]), points[0]), 3.0);
        let c = scale(sub(points[1], points[0]), 3.0);
        scale_add(scale_add(scale_add(a, t, b), t, c), t, points[0])
    }

    pub(super) fn chop_cubic_at(points: [Point; 4], t: f32) -> [Point; 7] {
        debug_assert!(0.0 <= t && t <= 1.0);

        if t == 1.0 {
            return [
                points[0], points[1], points[2], points[3], points[3], points[3], points[3],
            ];
        }

        let ab = mix(points[0], points[1], t);
        let bc = mix(points[1], points[2], t);
        let cd = mix(points[2], points[3], t);
        let abc = mix(ab, bc, t);
        let bcd = mix(bc, cd, t);
        let abcd = mix(abc, bcd, t);
        [points[0], ab, abc, abcd, bcd, cd, points[3]]
    }

    pub(super) fn chop_cubic_at_two(points: [Point; 4], t0: f32, t1: f32) -> [Point; 10] {
        debug_assert!(0.0 <= t0 && t0 <= t1 && t1 <= 1.0);

        if t1 == 1.0 {
            let first = chop_cubic_at(points, t0);
            return [
                first[0], first[1], first[2], first[3], first[4], first[5], first[6], points[3],
                points[3], points[3],
            ];
        }

        let ab = [mix(points[0], points[1], t0), mix(points[0], points[1], t1)];
        let bc = [mix(points[1], points[2], t0), mix(points[1], points[2], t1)];
        let cd = [mix(points[2], points[3], t0), mix(points[2], points[3], t1)];
        let abc = [mix(ab[0], bc[0], t0), mix(ab[1], bc[1], t1)];
        let bcd = [mix(bc[0], cd[0], t0), mix(bc[1], cd[1], t1)];
        let abcd = [mix(abc[0], bcd[0], t0), mix(abc[1], bcd[1], t1)];
        let middle = [mix(abc[0], bcd[0], t1), mix(abc[1], bcd[1], t0)];

        [
            points[0], ab[0], abc[0], abcd[0], middle[0], middle[1], abcd[1], bcd[1], cd[1],
            points[3],
        ]
    }

    pub(super) fn chop_cubic_at_values(
        mut points: [Point; 4],
        mut destination: Option<&mut [Point]>,
        t_values: Option<&[f32]>,
        t_count: usize,
    ) {
        debug_assert!(t_values.is_none_or(|values| {
            values.len() >= t_count && values[..t_count].iter().all(|&t| t >= 0.0 && t <= 1.0)
        }));
        debug_assert!(t_values.is_none_or(|values| values[..t_count].is_sorted()));

        if let Some(destination) = destination.as_mut() {
            debug_assert!(destination.len() >= t_count * 3 + 4);
            if t_count == 0 {
                destination[..4].copy_from_slice(&points);
            } else {
                let mut i = 0;
                let mut last_t = 0.0;
                let mut destination_offset = 0;
                while i < t_count - 1 {
                    let t = if let Some(t_values) = t_values {
                        let t = [
                            simd_clamp((t_values[i] - last_t) / (1.0 - last_t), 0.0, 1.0),
                            simd_clamp((t_values[i + 1] - last_t) / (1.0 - last_t), 0.0, 1.0),
                        ];
                        last_t = t_values[i + 1];
                        t
                    } else {
                        let denominator = (t_count + 1 - i) as f32;
                        [1.0 / denominator, 2.0 / denominator]
                    };
                    let chopped = chop_cubic_at_two(points, t[0], t[1]);
                    destination[destination_offset..destination_offset + 10]
                        .copy_from_slice(&chopped);
                    points = [chopped[6], chopped[7], chopped[8], chopped[9]];
                    destination_offset += 6;
                    i += 2;
                }
                if i < t_count {
                    debug_assert_eq!(i + 1, t_count);
                    let t = t_values.map_or(0.5, |values| values[i]);
                    let t = simd_clamp((t - last_t) / (1.0 - last_t), 0.0, 1.0);
                    let chopped = chop_cubic_at(points, t);
                    destination[destination_offset..destination_offset + 7]
                        .copy_from_slice(&chopped);
                }
            }
        }
    }

    pub(super) fn measure_angle_between_vectors(a: Point, b: Point) -> f32 {
        let mut cos_theta = dot(a, b) / (dot(a, a) * dot(b, b)).sqrt();
        cos_theta = cpp_std_max(cpp_std_min(1.0, cos_theta), -1.0);
        cos_theta.acos()
    }

    const TESS_EPSILON: f32 = 1.0 / (1 << 10) as f32;

    pub(super) fn find_cubic_convex_180_chops(
        points: [Point; 4],
        t: &mut [f32; 2],
        are_cusps: &mut bool,
    ) -> usize {
        const IEEE_ONE_MINUS_2_EPSILON: u32 = (127 << 23) - 2 * (1 << (24 - 10));
        debug_assert_eq!(
            f32::from_bits(IEEE_ONE_MINUS_2_EPSILON),
            1.0 - 2.0 * TESS_EPSILON
        );

        let coefficients = CubicCoeffs::new(points);
        let mut a = cross(coefficients.a, coefficients.b);
        let b = cross(coefficients.a, coefficients.c);
        let mut c = cross(coefficients.b, coefficients.c);
        let mut b_over_minus_2 = -0.5 * b;
        let mut discr_over_4 = b_over_minus_2.mul_add(b_over_minus_2, -(a * c));

        let mut cusp_threshold = a * (TESS_EPSILON / 2.0);
        cusp_threshold *= cusp_threshold;

        if discr_over_4 < -cusp_threshold {
            *are_cusps = false;
            let root = c / b_over_minus_2;
            if (root - TESS_EPSILON).to_bits() < IEEE_ONE_MINUS_2_EPSILON {
                t[0] = root;
                return 1;
            }
            return 0;
        }

        *are_cusps = discr_over_4 <= cusp_threshold;
        if *are_cusps {
            if a != 0.0 || b_over_minus_2 != 0.0 || c != 0.0 {
                let root = b_over_minus_2 / a;
                if (root - TESS_EPSILON).to_bits() < IEEE_ONE_MINUS_2_EPSILON {
                    t[0] = root;
                    return 1;
                }
                *are_cusps = false;
                return 0;
            }

            let base = sub(points[3], points[0]);
            let dot_products = [
                dot(points[0], base),
                dot(points[1], base),
                dot(points[2], base),
                dot(points[3], base),
            ];
            if dot_products[1] > dot_products[0]
                && dot_products[2] > dot_products[1]
                && dot_products[3] > dot_products[2]
            {
                *are_cusps = false;
                return 0;
            }

            let tan0 = if coefficients.c.0 != 0.0 || coefficients.c.1 != 0.0 {
                coefficients.c
            } else {
                sub(points[2], points[0])
            };
            a = dot(tan0, coefficients.a);
            b_over_minus_2 = -dot(tan0, coefficients.b);
            c = dot(tan0, coefficients.c);
            discr_over_4 = cpp_std_max(b_over_minus_2.mul_add(b_over_minus_2, -(a * c)), 0.0);
        }

        let mut q = discr_over_4.sqrt();
        q = q.copysign(b_over_minus_2);
        q += b_over_minus_2;
        let mut roots = [q / a, c / q];

        let inside = [
            roots[0] > TESS_EPSILON && roots[0] < 1.0 - TESS_EPSILON,
            roots[1] > TESS_EPSILON && roots[1] < 1.0 - TESS_EPSILON,
        ];
        if inside[0] {
            if inside[1] && roots[0] != roots[1] {
                if roots[0] > roots[1] {
                    roots.swap(0, 1);
                }
                *t = roots;
                return 2;
            }
            t[0] = roots[0];
            return 1;
        }
        if inside[1] {
            t[0] = roots[1];
            return 1;
        }
        0
    }

    pub(super) fn find_cubic_tan0(points: [Point; 4]) -> Point {
        let tangent_to = if points[0] != points[1] {
            points[1]
        } else if points[1] != points[2] {
            points[2]
        } else {
            points[3]
        };
        sub(tangent_to, points[0])
    }

    pub(super) fn find_cubic_tan1(points: [Point; 4]) -> Point {
        let tangent_from = if points[3] != points[2] {
            points[2]
        } else if points[2] != points[1] {
            points[1]
        } else {
            points[0]
        };
        sub(points[3], tangent_from)
    }

    pub(super) fn find_cubic_tangents(points: [Point; 4]) -> [Point; 2] {
        [find_cubic_tan0(points), find_cubic_tan1(points)]
    }

    pub(super) const fn pow2(x: f32) -> f32 {
        x * x
    }

    pub(super) const fn pow3(x: f32) -> f32 {
        x * pow2(x)
    }

    pub(super) const fn length_pow2(vector: Point) -> f32 {
        pow2(vector.0) + pow2(vector.1)
    }

    fn add(left: Point, right: Point) -> Point {
        (left.0 + right.0, left.1 + right.1)
    }

    fn sub(left: Point, right: Point) -> Point {
        (left.0 - right.0, left.1 - right.1)
    }

    fn scale(point: Point, value: f32) -> Point {
        (point.0 * value, point.1 * value)
    }

    fn scale_add(point: Point, value: f32, addend: Point) -> Point {
        (
            point.0.mul_add(value, addend.0),
            point.1.mul_add(value, addend.1),
        )
    }

    fn mix(a: Point, b: Point, t: f32) -> Point {
        debug_assert!(0.0 <= t && t < 1.0);
        ((b.0 - a.0).mul_add(t, a.0), (b.1 - a.1).mul_add(t, a.1))
    }

    fn dot(a: Point, b: Point) -> f32 {
        a.0.mul_add(b.0, a.1 * b.1)
    }

    fn cross(a: Point, b: Point) -> f32 {
        a.0 * b.1 - a.1 * b.0
    }

    fn simd_clamp(value: f32, low: f32, high: f32) -> f32 {
        let value = if low < value || low.is_nan() {
            value
        } else {
            low
        };
        if high < value || value.is_nan() {
            high
        } else {
            value
        }
    }

    fn cpp_std_min(first: f32, second: f32) -> f32 {
        if second < first { second } else { first }
    }

    fn cpp_std_max(first: f32, second: f32) -> f32 {
        if first < second { second } else { first }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn endpoint_chops_preserve_the_pinned_special_cases() {
            let points = [(0.0, 0.0), (1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
            assert_eq!(
                chop_cubic_at(points, 1.0),
                [
                    points[0], points[1], points[2], points[3], points[3], points[3], points[3]
                ]
            );
            let two = chop_cubic_at_two(points, 0.25, 1.0);
            assert_eq!(&two[7..], &[points[3], points[3], points[3]]);
        }

        #[test]
        fn null_destination_and_uniform_chops_follow_the_pointer_overload() {
            let points = [(0.0, 0.0), (1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
            chop_cubic_at_values(points, None, None, 3);

            let mut destination = [(0.0, 0.0); 13];
            chop_cubic_at_values(points, Some(&mut destination), None, 3);
            assert_eq!(destination[0], points[0]);
            assert_eq!(destination[3], eval_cubic_at(points, 0.25));
            assert_eq!(destination[6], eval_cubic_at(points, 0.5));
            assert_eq!(destination[9], eval_cubic_at(points, 0.75));
            assert_eq!(destination[12], points[3]);
        }

        #[test]
        fn zero_vector_angle_and_flat_line_degeneracy_match_source() {
            assert_eq!(measure_angle_between_vectors((0.0, 0.0), (1.0, 0.0)), 0.0);

            let mut t = [123.0, 456.0];
            let mut are_cusps = true;
            assert_eq!(
                find_cubic_convex_180_chops(
                    [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)],
                    &mut t,
                    &mut are_cusps,
                ),
                0
            );
            assert!(!are_cusps);
            assert_eq!(t, [123.0, 456.0]);
        }

        #[test]
        fn header_helpers_keep_coefficient_and_tangent_correspondence() {
            let points = [(0.0, 0.0), (1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
            let eval = EvalCubic::new(points);
            assert_eq!(eval.evaluate(0.375), eval_cubic_at(points, 0.375));
            assert_eq!(eval.evaluate_pair([0.25, 0.75]), eval.at([0.25, 0.75]));
            assert_eq!(find_cubic_tangents(points), [(1.0, 2.0), (2.0, 2.0)]);
            assert_eq!(pow3(3.0), 27.0);
            assert_eq!(length_pow2((3.0, 4.0)), 25.0);
            assert!(calc_polar_segments_per_radian::<4>(2.0).is_finite());
        }

        #[test]
        fn adversarial_values_match_the_pinned_cpp_owner() {
            let points = [
                (-1.349_232_5e20, std::f32::consts::PI),
                (2.718_281_7e-20, -9.109_383e-31),
                (6.022_140_8e19, f32::MIN_POSITIVE),
                (-3.402_823e30, std::f32::consts::SQRT_2),
            ];
            let expected = [
                (0.0, [0xe0ea_0df6, 0x4049_0fdb]),
                (1.0 / 1024.0, [0xe333_176a, 0x4048_7935]),
                (0.375, [0xf010_f483, 0x3f57_7102]),
                (1.0 - 1.0 / 1024.0, [0xf22b_4bbc, 0x3fb4_7d51]),
                (1.0, [0xf22b_cc75, 0x3fb5_04f2]),
            ];
            for (t, expected) in expected {
                let actual = eval_cubic_at(points, t);
                assert_eq!([actual.0.to_bits(), actual.1.to_bits()], expected);
            }

            let chopped = chop_cubic_at(points, 0.375);
            let expected = [
                [0xe0ea_0df6, 0x4049_0fdb],
                [0xe092_48ba, 0x3ffb_53d2],
                [0xe019_7940, 0x3f9d_1463],
                [0xf010_f483, 0x3f57_7101],
                [0xf0c1_4604, 0x3e4b_a591],
                [0xf180_d958, 0x3f07_c3b6],
                [0xf22b_cc75, 0x3fb5_04f3],
            ];
            for (actual, expected) in chopped.into_iter().zip(expected) {
                assert_eq!([actual.0.to_bits(), actual.1.to_bits()], expected);
            }

            let two = chop_cubic_at_two(points, 0.125, 0.625);
            let expected = [
                [0xe0ea_0df6, 0x4049_0fdb],
                [0xe0cc_cc37, 0x402f_ede0],
                [0xe0b1_90d1, 0x4019_f024],
                [0xedab_cc75, 0x4006_df61],
                [0xeed6_bf92, 0x3f6a_714f],
                [0xf006_37bb, 0x3ee9_4693],
                [0xf127_c5aa, 0x3f02_ccc3],
                [0xf186_37bb, 0x3f0d_6bde],
                [0xf1d6_bf92, 0x3f62_4630],
                [0xf22b_cc75, 0x3fb5_04f3],
            ];
            for (actual, expected) in two.into_iter().zip(expected) {
                assert_eq!([actual.0.to_bits(), actual.1.to_bits()], expected);
            }

            let mut many = [(0.0, 0.0); 13];
            chop_cubic_at_values(points, Some(&mut many), Some(&[0.125, 0.625, 0.875]), 3);
            let expected = [
                [0xe0ea_0df6, 0x4049_0fdb],
                [0xe0cc_cc37, 0x402f_ede0],
                [0xe0b1_90d1, 0x4019_f024],
                [0xedab_cc75, 0x4006_df61],
                [0xeed6_bf92, 0x3f6a_714f],
                [0xf006_37bb, 0x3ee9_4693],
                [0xf127_c5aa, 0x3f02_ccc3],
                [0xf16a_e187, 0x3f09_e180],
                [0xf1a4_6aac, 0x3f31_f41c],
                [0xf1e6_2ef0, 0x3f74_1bc1],
                [0xf203_8889, 0x3f8a_97ca],
                [0xf216_52e6, 0x3f9e_6455],
                [0xf22b_cc75, 0x3fb5_04f3],
            ];
            for (actual, expected) in many.into_iter().zip(expected) {
                assert_eq!([actual.0.to_bits(), actual.1.to_bits()], expected);
            }

            let mut t = [123.0, 456.0];
            let mut are_cusps = false;
            assert_eq!(
                find_cubic_convex_180_chops(
                    [(0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 0.0)],
                    &mut t,
                    &mut are_cusps,
                ),
                2
            );
            assert!(are_cusps);
            assert_eq!([t[0].to_bits(), t[1].to_bits()], [0x3e58_658b, 0x3f49_e69d]);
        }
    }
}
