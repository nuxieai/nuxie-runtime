use crate::components::TransformComponents;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2D(pub [f32; 6]);

impl Mat2D {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn from_rotation(radians: f32) -> Self {
        let (sin, cos) = if radians == 0.0 {
            (0.0, 1.0)
        } else {
            radians.sin_cos()
        };
        Self([cos, sin, -sin, cos, 0.0, 0.0])
    }

    pub fn multiply(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        Self([
            a[0].mul_add(b[0], a[2] * b[1]),
            a[1].mul_add(b[0], a[3] * b[1]),
            a[0].mul_add(b[2], a[2] * b[3]),
            a[1].mul_add(b[2], a[3] * b[3]),
            a[0].mul_add(b[4], a[2] * b[5]) + a[4],
            a[1].mul_add(b[4], a[3] * b[5]) + a[5],
        ])
    }

    pub fn scale_by_values(&mut self, scale_x: f32, scale_y: f32) {
        self.0[0] *= scale_x;
        self.0[1] *= scale_x;
        self.0[2] *= scale_y;
        self.0[3] *= scale_y;
    }

    pub(crate) fn decompose(self) -> TransformComponents {
        // Ported from C++ `src/math/mat2d.cpp`.
        let [m0, m1, m2, m3, x, y] = self.0;
        let rotation = m1.atan2(m0);
        let denom = m0 * m0 + m1 * m1;
        let scale_x = denom.sqrt();
        let scale_y = if scale_x == 0.0 {
            0.0
        } else {
            (m0 * m3 - m2 * m1) / scale_x
        };
        let skew = (m0 * m2 + m1 * m3).atan2(denom);
        TransformComponents {
            x,
            y,
            scale_x,
            scale_y,
            rotation,
            skew,
        }
    }

    pub(crate) fn compose(components: TransformComponents) -> Self {
        // Ported from C++ `src/math/mat2d.cpp`.
        let mut result = Self::from_rotation(components.rotation);
        result.0[4] = components.x;
        result.0[5] = components.y;
        result.scale_by_values(components.scale_x, components.scale_y);

        if components.skew != 0.0 {
            result.0[2] = result.0[0] * components.skew + result.0[2];
            result.0[3] = result.0[1] * components.skew + result.0[3];
        }
        result
    }

    pub fn determinant(self) -> f32 {
        self.0[0].mul_add(self.0[3], -(self.0[1] * self.0[2]))
    }

    pub fn invert_or_identity(self) -> Self {
        let determinant = self.determinant();
        if determinant == 0.0 {
            return Self::IDENTITY;
        }

        let [a, b, c, d, e, f] = self.0;
        let determinant = 1.0 / determinant;
        Self([
            d * determinant,
            -b * determinant,
            -c * determinant,
            a * determinant,
            c.mul_add(f, -(d * e)) * determinant,
            b.mul_add(e, -(a * f)) * determinant,
        ])
    }

    pub fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.0[0] * x + self.0[2] * y + self.0[4],
            self.0[1] * x + self.0[3] * y + self.0[5],
        )
    }

    pub fn map_point(self, x: f32, y: f32) -> (f32, f32) {
        let [a, b, c, d, e, f] = self.0;
        // Ported from src/math/mat2d.cpp Mat2D::mapPoints. The grouping matters
        // for cancellation-heavy local path composition.
        if b == 0.0 && c == 0.0 {
            (a.mul_add(x, e), d.mul_add(y, f))
        } else {
            (a.mul_add(x, c.mul_add(y, e)), d.mul_add(y, b.mul_add(x, f)))
        }
    }

    /// Maps a contiguous point buffer using the same affine specialization as
    /// [`Self::map_point`]. The slice boundary gives the optimizer the bulk
    /// operation shape used by Rive's `Mat2D::mapPoints`.
    pub fn map_points(self, destination: &mut [(f32, f32)], source: &[(f32, f32)]) {
        assert_eq!(destination.len(), source.len());
        let [a, b, c, d, e, f] = self.0;
        if b == 0.0 && c == 0.0 {
            for (destination, &(x, y)) in destination.iter_mut().zip(source) {
                *destination = (a.mul_add(x, e), d.mul_add(y, f));
            }
        } else {
            for (destination, &(x, y)) in destination.iter_mut().zip(source) {
                *destination = (a.mul_add(x, c.mul_add(y, e)), d.mul_add(y, b.mul_add(x, f)));
            }
        }
    }

    /// In-place form of [`Self::map_points`], equivalent to C++ `mapPoints`
    /// with the same source and destination pointer.
    pub fn map_points_in_place(self, points: &mut [(f32, f32)]) {
        let [a, b, c, d, e, f] = self.0;
        if b == 0.0 && c == 0.0 {
            for (x, y) in points {
                (*x, *y) = (a.mul_add(*x, e), d.mul_add(*y, f));
            }
        } else {
            for (x, y) in points {
                (*x, *y) = (
                    a.mul_add(*x, c.mul_add(*y, e)),
                    d.mul_add(*y, b.mul_add(*x, f)),
                );
            }
        }
    }

    pub fn transform_direction(self, x: f32, y: f32) -> (f32, f32) {
        (self.0[0] * x + self.0[2] * y, self.0[1] * x + self.0[3] * y)
    }
}

impl Default for Mat2D {
    fn default() -> Self {
        Self::IDENTITY
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
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    struct UpstreamAabb {
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    }

    impl UpstreamAabb {
        fn from_points(points: &[(f32, f32)]) -> Self {
            let mut bounds = Self {
                left: 1e9,
                top: 1e9,
                right: -1e9,
                bottom: -1e9,
            };
            for &(x, y) in points {
                bounds.left = bounds.left.min(x);
                bounds.top = bounds.top.min(y);
                bounds.right = bounds.right.max(x);
                bounds.bottom = bounds.bottom.max(y);
            }
            bounds
        }

        fn width(self) -> f32 {
            self.right - self.left
        }

        fn height(self) -> f32 {
            self.bottom - self.top
        }
    }

    fn upstream_map_bounding_box(_matrix: Mat2D, _points: &[(f32, f32)]) -> UpstreamAabb {
        // Mat2D has no production mapBoundingBox owner yet. The default result
        // makes the complete upstream contract executable and observably red.
        UpstreamAabb::default()
    }

    fn upstream_map_aabb(matrix: Mat2D, bounds: UpstreamAabb) -> UpstreamAabb {
        upstream_map_bounding_box(
            matrix,
            &[
                (bounds.left, bounds.top),
                (bounds.left, bounds.bottom),
                (bounds.right, bounds.bottom),
                (bounds.right, bounds.top),
            ],
        )
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= f32::EPSILON,
            "{actual} != {expected}"
        );
    }

    #[test]
    fn upstream_map_points_complete_sequence() {
        let mut random = UpstreamRand::new(1);
        let mut test_points = vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
        for _ in 0..100 {
            test_points.push((
                (random.next() % 201) as f32 - 100.0,
                (random.next() % 201) as f32 - 100.0,
            ));
        }
        let n = test_points.len();
        let mut destination = vec![(0.0, 0.0); n];
        let mut expected = vec![(0.0, 0.0); n];

        let check_matrix =
            |matrix: Mat2D, destination: &mut Vec<(f32, f32)>, expected: &mut Vec<(f32, f32)>| {
                matrix.map_points(&mut destination[..1], &test_points[2..3]);
                expected[0] = matrix.map_point(test_points[2].0, test_points[2].1);
                assert_eq!(destination[0], expected[0]);

                matrix.map_points(&mut destination[..n - 1], &test_points[1..]);
                for i in 0..n - 1 {
                    expected[i] = matrix.map_point(test_points[i + 1].0, test_points[i + 1].1);
                }
                assert_eq!(&destination[..n - 1], &expected[..n - 1]);

                matrix.map_points(destination, &test_points);
                for i in 0..n {
                    expected[i] = matrix.map_point(test_points[i].0, test_points[i].1);
                }
                assert_eq!(destination, expected);
            };

        for matrix in [
            Mat2D::IDENTITY,
            Mat2D([1.0, 0.0, 0.0, 1.0, 2.0, -3.0]),
            Mat2D([4.0, 0.0, 0.0, -5.0, 0.0, 0.0]),
            Mat2D([4.0, 0.0, 0.0, 5.0, -6.0, 7.0]),
            Mat2D([0.0, 8.0, 9.0, 0.0, 10.0, 11.0]),
            Mat2D([-12.0, -13.0, -14.0, -15.0, -16.0, -17.0]),
            Mat2D([18.0, 19.0, 20.0, 21.0, 22.0, 23.0]),
            Mat2D([-25.0, 26.0, 27.0, -28.0, 29.0, -30.0]),
        ] {
            check_matrix(matrix, &mut destination, &mut expected);
        }
    }

    #[test]
    #[ignore = "expected-red: Rust Mat2D has no production mapBoundingBox owner"]
    fn upstream_map_bounding_box_complete_sequence() {
        let test_points = [
            (0.0, 0.0),
            (1.0, 0.0),
            (0.0, 1.0),
            (-1.0, 0.0),
            (0.0, -1.0),
            (1.0, 1.0),
            (-1.0, -1.0),
        ];
        let mut mapped_points = vec![(0.0, 0.0); test_points.len()];

        let check_matrix = |matrix: Mat2D, mapped_points: &mut Vec<(f32, f32)>| {
            let mut mapped = upstream_map_bounding_box(matrix, &[]);
            assert_eq!(mapped.left, 0.0);
            assert_eq!(mapped.top, 0.0);
            assert_eq!(mapped.right, 0.0);
            assert_eq!(mapped.bottom, 0.0);

            for point in test_points {
                mapped = upstream_map_bounding_box(matrix, &[point]);
                let mapped_point = matrix.map_point(point.0, point.1);
                assert_close(mapped.left, mapped_point.0);
                assert_close(mapped.top, mapped_point.1);
                assert_close(mapped.right, mapped_point.0);
                assert_close(mapped.bottom, mapped_point.1);

                mapped = upstream_map_bounding_box(matrix, std::slice::from_ref(&point));
                assert_close(mapped.left, mapped_point.0);
                assert_close(mapped.top, mapped_point.1);
                assert_close(mapped.right, mapped_point.0);
                assert_close(mapped.bottom, mapped_point.1);
            }

            matrix.map_points(
                &mut mapped_points[..test_points.len() - 1],
                &test_points[1..],
            );
            let test_bounds = UpstreamAabb::from_points(&mapped_points[..test_points.len() - 1]);
            mapped = upstream_map_bounding_box(matrix, &test_points[1..]);
            assert_close(mapped.left, test_bounds.left);
            assert_close(mapped.top, test_bounds.top);
            assert_close(mapped.right, test_bounds.right);
            assert_close(mapped.bottom, test_bounds.bottom);

            mapped = upstream_map_bounding_box(matrix, &test_points[1..]);
            assert_close(mapped.left, test_bounds.left);
            assert_close(mapped.top, test_bounds.top);
            assert_close(mapped.right, test_bounds.right);
            assert_close(mapped.bottom, test_bounds.bottom);

            matrix.map_points(mapped_points, &test_points);
            let test_bounds = UpstreamAabb::from_points(mapped_points);
            mapped = upstream_map_bounding_box(matrix, &test_points);
            assert_close(mapped.left, test_bounds.left);
            assert_close(mapped.top, test_bounds.top);
            assert_close(mapped.right, test_bounds.right);
            assert_close(mapped.bottom, test_bounds.bottom);

            mapped = upstream_map_bounding_box(matrix, &test_points);
            assert_close(mapped.left, test_bounds.left);
            assert_close(mapped.top, test_bounds.top);
            assert_close(mapped.right, test_bounds.right);
            assert_close(mapped.bottom, test_bounds.bottom);

            let bbox_points = [(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)];
            let mapped_from_points = upstream_map_bounding_box(matrix, &bbox_points);
            let mapped_from_aabb = upstream_map_aabb(
                matrix,
                UpstreamAabb {
                    left: 0.0,
                    top: 0.0,
                    right: 1.0,
                    bottom: 1.0,
                },
            );
            assert_close(mapped_from_points.left, mapped_from_aabb.left);
            assert_close(mapped_from_points.top, mapped_from_aabb.top);
            assert_close(mapped_from_points.right, mapped_from_aabb.right);
            assert_close(mapped_from_points.bottom, mapped_from_aabb.bottom);
        };

        for matrix in [
            Mat2D::IDENTITY,
            Mat2D([1.0, 0.0, 0.0, 1.0, 2.0, -3.0]),
            Mat2D([4.0, 0.0, 0.0, -5.0, 0.0, 0.0]),
            Mat2D([4.0, 0.0, 0.0, 5.0, -6.0, 7.0]),
            Mat2D([0.0, 8.0, 9.0, 0.0, 10.0, 11.0]),
            Mat2D([-12.0, -13.0, -14.0, -15.0, -16.0, -17.0]),
            Mat2D([18.0, 19.0, 20.0, 21.0, 22.0, 23.0]),
            Mat2D([-25.0, 26.0, 27.0, -28.0, 29.0, -30.0]),
        ] {
            check_matrix(matrix, &mut mapped_points);
        }

        assert_eq!(
            upstream_map_bounding_box(Mat2D::IDENTITY, &[]),
            UpstreamAabb::default()
        );
        let nan = f32::NAN;
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: nan,
                    top: nan,
                    right: nan,
                    bottom: nan
                },
            ),
            UpstreamAabb::default()
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: -1.0,
                    top: -1.0,
                    right: 1.0,
                    bottom: 1.0
                }
            ),
            UpstreamAabb {
                left: -1.0,
                top: -1.0,
                right: 1.0,
                bottom: 1.0
            }
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: nan,
                    top: -1.0,
                    right: 1.0,
                    bottom: 1.0
                }
            ),
            UpstreamAabb {
                left: 1.0,
                top: -1.0,
                right: 1.0,
                bottom: 1.0
            }
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: -1.0,
                    top: nan,
                    right: 1.0,
                    bottom: 1.0
                }
            ),
            UpstreamAabb {
                left: -1.0,
                top: 1.0,
                right: 1.0,
                bottom: 1.0
            }
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: -1.0,
                    top: -1.0,
                    right: nan,
                    bottom: 1.0
                }
            ),
            UpstreamAabb {
                left: -1.0,
                top: -1.0,
                right: -1.0,
                bottom: 1.0
            }
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: -1.0,
                    top: -1.0,
                    right: 1.0,
                    bottom: nan
                }
            ),
            UpstreamAabb {
                left: -1.0,
                top: -1.0,
                right: 1.0,
                bottom: -1.0
            }
        );

        let inf = f32::INFINITY;
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: 0.0,
                    top: inf,
                    right: 0.0,
                    bottom: nan
                }
            )
            .height(),
            0.0
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: 0.0,
                    top: -inf,
                    right: 0.0,
                    bottom: nan
                }
            )
            .height(),
            0.0
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: inf,
                    top: 0.0,
                    right: nan,
                    bottom: 0.0
                }
            )
            .width(),
            0.0
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: -inf,
                    top: 0.0,
                    right: nan,
                    bottom: 0.0
                }
            )
            .width(),
            0.0
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: inf,
                    top: 0.0,
                    right: inf,
                    bottom: 0.0
                }
            )
            .width(),
            0.0
        );
        assert_eq!(
            upstream_map_aabb(
                Mat2D::IDENTITY,
                UpstreamAabb {
                    left: 0.0,
                    top: -inf,
                    right: 0.0,
                    bottom: -inf
                }
            )
            .height(),
            0.0
        );
    }

    #[test]
    fn bulk_map_matches_scalar_for_distinct_and_in_place_buffers() {
        let matrix = Mat2D([2.0, -3.0, -4.0, 5.0, 6.0, -7.0]);
        let source = [(1.0, 2.0), (-3.0, 4.0), (0.0, 0.0)];
        let expected = source.map(|(x, y)| matrix.map_point(x, y));
        let mut destination = [(0.0, 0.0); 3];

        matrix.map_points(&mut destination, &source);
        assert_eq!(destination, expected);

        matrix.map_points_in_place(&mut destination);
        assert_eq!(destination, expected.map(|(x, y)| matrix.map_point(x, y)));
    }

    #[test]
    fn inverse_and_multiply_match_cpp_contraction_order() {
        // Values from the first local path in joel_signed.riv. These expected
        // bits come from C++ Mat2D::invert and Mat2D::multiply compiled with
        // the release runner's default `-ffp-contract=on` on arm64.
        let shape_world = Mat2D([
            0.6845234,
            0.35772082,
            -0.35772082,
            0.6845234,
            -130.04749,
            -135.59448,
        ]);
        let path_world = Mat2D([
            0.6845234,
            0.35772082,
            -0.35772082,
            0.6845234,
            6.7375793,
            -313.59125,
        ]);

        let inverse = shape_world.invert_or_identity();
        assert_eq!(
            inverse.0.map(f32::to_bits),
            [
                0x3f92_e129,
                0xbf19_8383,
                0x3f19_8383,
                0x3f92_e129,
                0x4366_8a3d,
                0x429b_3811,
            ]
        );

        let local = inverse.multiply(path_world);
        assert_eq!(
            local.0.map(f32::to_bits),
            [
                0x3f80_0000,
                0x2fc5_dc80,
                0xb19c_ac38,
                0x3f80_0000,
                0x4248_e3a0,
                0xc38f_2347,
            ]
        );
    }
}
