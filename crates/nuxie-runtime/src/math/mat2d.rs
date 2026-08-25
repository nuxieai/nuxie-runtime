use crate::components::TransformComponents;

#[inline(always)]
fn map_points_fma(point_lane: f32, matrix_lane: f32, addend_lane: f32) -> f32 {
    point_lane.mul_add(matrix_lane, addend_lane)
}

#[inline(always)]
fn map_bounds_simd_min(first: f32, second: f32) -> f32 {
    if first.is_nan() {
        second
    } else if second.is_nan() {
        first
    } else if first == 0.0 && second == 0.0 {
        f32::from_bits(first.to_bits() | second.to_bits())
    } else if second < first {
        second
    } else {
        first
    }
}

#[inline(always)]
fn map_bounds_simd_max(first: f32, second: f32) -> f32 {
    if first.is_nan() {
        second
    } else if second.is_nan() {
        first
    } else if first == 0.0 && second == 0.0 {
        f32::from_bits(first.to_bits() & second.to_bits())
    } else if first < second {
        second
    } else {
        first
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2D(pub [f32; 6]);

impl Mat2D {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub const fn from_scale(scale_x: f32, scale_y: f32) -> Self {
        Self([scale_x, 0.0, 0.0, scale_y, 0.0, 0.0])
    }

    pub const fn from_translation(translation_x: f32, translation_y: f32) -> Self {
        Self([1.0, 0.0, 0.0, 1.0, translation_x, translation_y])
    }

    pub const fn from_scale_and_translation(
        scale_x: f32,
        scale_y: f32,
        translation_x: f32,
        translation_y: f32,
    ) -> Self {
        Self([scale_x, 0.0, 0.0, scale_y, translation_x, translation_y])
    }

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

    pub const fn values(&self) -> &[f32; 6] {
        &self.0
    }

    pub const fn xx(self) -> f32 {
        self.0[0]
    }

    pub const fn xy(self) -> f32 {
        self.0[1]
    }

    pub const fn yx(self) -> f32 {
        self.0[2]
    }

    pub const fn yy(self) -> f32 {
        self.0[3]
    }

    pub const fn tx(self) -> f32 {
        self.0[4]
    }

    pub const fn ty(self) -> f32 {
        self.0[5]
    }

    pub const fn translation(self) -> (f32, f32) {
        (self.0[4], self.0[5])
    }

    pub fn set_xx(&mut self, value: f32) {
        self.0[0] = value;
    }

    pub fn set_xy(&mut self, value: f32) {
        self.0[1] = value;
    }

    pub fn set_yx(&mut self, value: f32) {
        self.0[2] = value;
    }

    pub fn set_yy(&mut self, value: f32) {
        self.0[3] = value;
    }

    pub fn set_tx(&mut self, value: f32) {
        self.0[4] = value;
    }

    pub fn set_ty(&mut self, value: f32) {
        self.0[5] = value;
    }

    pub fn scale(self, vector: (f32, f32)) -> Self {
        let [a, b, c, d, e, f] = self.0;
        Self([a * vector.0, b * vector.0, c * vector.1, d * vector.1, e, f])
    }

    pub fn translate(self, vector: (f32, f32)) -> Self {
        let [a, b, c, d, e, f] = self.0;
        Self([a, b, c, d, e + vector.0, f + vector.1])
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
        let denom = m0.mul_add(m0, m1 * m1);
        let scale_x = denom.sqrt();
        let scale_y = if scale_x == 0.0 {
            0.0
        } else {
            m0.mul_add(m3, -(m2 * m1)) / scale_x
        };
        let skew = m0.mul_add(m2, m1 * m3).atan2(denom);
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
            result.0[2] = result.0[0].mul_add(components.skew, result.0[2]);
            result.0[3] = result.0[1].mul_add(components.skew, result.0[3]);
        }
        result
    }

    pub fn determinant(self) -> f32 {
        self.0[0].mul_add(self.0[3], -(self.0[1] * self.0[2]))
    }

    /// Safe-Rust form of pinned `Mat2D::invert(Mat2D*)`: `None` represents
    /// the source's false return while leaving its output parameter unchanged.
    pub fn invert(self) -> Option<Self> {
        let determinant = self.determinant();
        if determinant == 0.0 {
            return None;
        }

        let [a, b, c, d, e, f] = self.0;
        let determinant = 1.0 / determinant;
        Some(Self([
            d * determinant,
            -b * determinant,
            -c * determinant,
            a * determinant,
            c.mul_add(f, -(d * e)) * determinant,
            b.mul_add(e, -(a * f)) * determinant,
        ]))
    }

    pub fn invert_or_identity(self) -> Self {
        self.invert().unwrap_or(Self::IDENTITY)
    }

    pub fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        let [xx, yx, xy, yy, tx, ty] = self.0;
        // Pinned C++ contracts the two linear products, then adds translation.
        (tx + xx.mul_add(x, xy * y), ty + yx.mul_add(x, yy * y))
    }

    pub fn map_point(self, x: f32, y: f32) -> (f32, f32) {
        let source = [(x, y)];
        let mut destination = [(0.0, 0.0)];
        self.map_points(&mut destination, &source);
        destination[0]
    }

    /// Exact out-of-line owner for pinned `Mat2D::mapPoints`.
    ///
    /// The odd point is mapped first, then the remaining points are loaded and
    /// stored in pairs. NaN payload selection remains native Rust/compiler
    /// behavior; it is not a portable semantic contract of either source.
    #[inline(never)]
    pub fn map_points(self, destination: &mut [(f32, f32)], source: &[(f32, f32)]) {
        assert_eq!(destination.len(), source.len());
        // SAFETY: both slices have the asserted common length and safe Rust
        // guarantees they do not overlap.
        unsafe {
            self.map_points_raw(destination.as_mut_ptr(), source.as_ptr(), source.len());
        }
    }

    /// In-place form of the same pinned out-of-line owner.
    #[inline(never)]
    pub fn map_points_in_place(self, points: &mut [(f32, f32)]) {
        // SAFETY: source and destination are the same valid slice. The raw
        // owner loads each one- or two-point batch before writing either lane.
        unsafe {
            self.map_points_raw(points.as_mut_ptr(), points.as_ptr(), points.len());
        }
    }

    #[inline(never)]
    unsafe fn map_points_raw(
        self,
        destination: *mut (f32, f32),
        source: *const (f32, f32),
        count: usize,
    ) {
        let [scale_x, skew_y, skew_x, scale_y, translate_x, translate_y] = self.0;
        let no_skew = skew_y == 0.0 && skew_x == 0.0;
        let map = |(x, y): (f32, f32)| {
            if no_skew {
                (
                    map_points_fma(x, scale_x, translate_x),
                    map_points_fma(y, scale_y, translate_y),
                )
            } else {
                let skewed_x = map_points_fma(y, skew_x, translate_x);
                let skewed_y = map_points_fma(x, skew_y, translate_y);
                (
                    map_points_fma(x, scale_x, skewed_x),
                    map_points_fma(y, scale_y, skewed_y),
                )
            }
        };

        let mut index = 0;
        if count & 1 != 0 {
            // SAFETY: count is nonzero and both pointers cover `count` values.
            let point = unsafe { *source };
            unsafe { *destination = map(point) };
            index = 1;
        }
        while index < count {
            // Load both source lanes before either store so exact in-place
            // mapping has the same alias behavior as the source float4 load.
            let first = unsafe { *source.add(index) };
            let second = unsafe { *source.add(index + 1) };
            let mapped_first = map(first);
            let mapped_second = map(second);
            unsafe {
                *destination.add(index) = mapped_first;
                *destination.add(index + 1) = mapped_second;
            }
            index += 2;
        }
    }

    /// Return the tight transformed bounds of `points` as
    /// `(left, top, right, bottom)`.
    ///
    /// Exact scalar spelling of pinned
    /// `Mat2D::mapBoundingBox(const Vec2D[], size_t)`. Pair-lane
    /// initialization/reduction, SIMD min/max selection, fused affine
    /// grouping, translation-after-extrema, and final debug contracts are all
    /// preserved.
    pub fn map_bounding_box(self, points: &[(f32, f32)]) -> (f32, f32, f32, f32) {
        let [scale_x, skew_y, skew_x, scale_y, translate_x, translate_y] = self.0;
        let no_skew = skew_y == 0.0 && skew_x == 0.0;
        let mut mins = [f32::INFINITY; 4];
        let mut maxes = [f32::NEG_INFINITY; 4];
        let mut index = 0;

        if points.len() & 1 != 0 {
            let (x, y) = points[0];
            let mapped = if no_skew {
                [scale_x * x, scale_y * y]
            } else {
                [
                    scale_x.mul_add(x, skew_x * y),
                    scale_y.mul_add(y, skew_y * x),
                ]
            };
            mins[0] = mapped[0];
            mins[1] = mapped[1];
            maxes[0] = mapped[0];
            maxes[1] = mapped[1];
            index = 1;
        }

        while index < points.len() {
            let (first_x, first_y) = points[index];
            let (second_x, second_y) = points[index + 1];
            let mapped = if no_skew {
                [
                    scale_x * first_x,
                    scale_y * first_y,
                    scale_x * second_x,
                    scale_y * second_y,
                ]
            } else {
                [
                    scale_x.mul_add(first_x, skew_x * first_y),
                    scale_y.mul_add(first_y, skew_y * first_x),
                    scale_x.mul_add(second_x, skew_x * second_y),
                    scale_y.mul_add(second_y, skew_y * second_x),
                ]
            };
            for lane in 0..4 {
                mins[lane] = map_bounds_simd_min(mapped[lane], mins[lane]);
                maxes[lane] = map_bounds_simd_max(mapped[lane], maxes[lane]);
            }
            index += 2;
        }

        let min_x = map_bounds_simd_min(mins[0], mins[2]);
        let min_y = map_bounds_simd_min(mins[1], mins[3]);
        let max_x = map_bounds_simd_max(maxes[0], maxes[2]);
        let max_y = map_bounds_simd_max(maxes[1], maxes[3]);

        let bounds = if !(max_x - min_x >= 0.0 && max_y - min_y >= 0.0) {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            (
                min_x + translate_x,
                min_y + translate_y,
                max_x + translate_x,
                max_y + translate_y,
            )
        };
        debug_assert!(bounds.2 - bounds.0 >= 0.0);
        debug_assert!(bounds.3 - bounds.1 >= 0.0);
        bounds
    }

    /// Four-corner overload corresponding to pinned
    /// `Mat2D::mapBoundingBox(const AABB&)`.
    pub fn map_bounds(self, bounds: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
        let (left, top, right, bottom) = bounds;
        self.map_bounding_box(&[(left, top), (right, top), (right, bottom), (left, bottom)])
    }

    pub fn transform_direction(self, x: f32, y: f32) -> (f32, f32) {
        let [xx, yx, xy, yy, _, _] = self.0;
        // Pinned Vec2D::transformDir contracts each two-product sum.
        (xx.mul_add(x, xy * y), yx.mul_add(x, yy * y))
    }
}

impl Default for Mat2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Index<usize> for Mat2D {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Mat2D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl std::ops::Mul for Mat2D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

impl std::ops::MulAssign for Mat2D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.multiply(rhs);
    }
}

impl std::ops::Mul<(f32, f32)> for Mat2D {
    type Output = (f32, f32);

    fn mul(self, rhs: (f32, f32)) -> Self::Output {
        self.transform_point(rhs.0, rhs.1)
    }
}

#[cfg(test)]
mod tests {
    use super::{Mat2D, map_points_fma};
    use crate::components::TransformComponents;

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

    fn upstream_map_bounding_box(matrix: Mat2D, points: &[(f32, f32)]) -> UpstreamAabb {
        let (left, top, right, bottom) = matrix.map_bounding_box(points);
        UpstreamAabb {
            left,
            top,
            right,
            bottom,
        }
    }

    fn upstream_map_aabb(matrix: Mat2D, bounds: UpstreamAabb) -> UpstreamAabb {
        let (left, top, right, bottom) =
            matrix.map_bounds((bounds.left, bounds.top, bounds.right, bounds.bottom));
        UpstreamAabb {
            left,
            top,
            right,
            bottom,
        }
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
    fn map_bounding_box_preserves_pinned_pair_lanes_and_nonfinite_normalization() {
        let signed_zero_translation = Mat2D([1.0, 0.0, 0.0, 1.0, -0.0, -0.0]);
        let forward = signed_zero_translation
            .map_bounding_box(&[(0.0, 0.0), (-0.0, -0.0)]);
        assert_eq!(
            [
                forward.0.to_bits(),
                forward.1.to_bits(),
                forward.2.to_bits(),
                forward.3.to_bits(),
            ],
            [
                (-0.0_f32).to_bits(),
                (-0.0_f32).to_bits(),
                0.0_f32.to_bits(),
                0.0_f32.to_bits(),
            ]
        );

        let reverse = signed_zero_translation
            .map_bounding_box(&[(-0.0, -0.0), (0.0, 0.0)]);
        assert_eq!(
            [
                reverse.0.to_bits(),
                reverse.1.to_bits(),
                reverse.2.to_bits(),
                reverse.3.to_bits(),
            ],
            [
                (-0.0_f32).to_bits(),
                (-0.0_f32).to_bits(),
                0.0_f32.to_bits(),
                0.0_f32.to_bits(),
            ]
        );

        let infinite_x = Mat2D([f32::INFINITY, 0.0, 0.0, 1.0, 19.0, 23.0])
            .map_bounds((0.0, 0.0, 1.0, 1.0));
        assert_eq!(
            [
                infinite_x.0.to_bits(),
                infinite_x.1.to_bits(),
                infinite_x.2.to_bits(),
                infinite_x.3.to_bits(),
            ],
            [0; 4],
            "pinned source normalizes the nonfinite linear result before translation"
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn map_bounding_box_retains_pinned_post_translation_extent_assertions() {
        Mat2D([1.0, 0.0, 0.0, 1.0, f32::INFINITY, 0.0])
            .map_bounding_box(&[(0.0, 0.0), (1.0, 1.0)]);
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
    fn map_points_preserves_exceptional_classification_across_all_forms() {
        let matrix = std::hint::black_box(Mat2D([
            f32::from_bits(0xffc0_1234),
            f32::from_bits(0xffc0_1234),
            f32::from_bits(0x7fc0_bbbb),
            f32::from_bits(0x8000_0000),
            f32::from_bits(0x0080_0000),
            f32::from_bits(0xbf80_0000),
        ]));
        let point =
            std::hint::black_box((f32::from_bits(0xffff_ffff), f32::from_bits(0x3f80_0000)));
        let mapped = matrix.map_point(point.0, point.1);
        assert!(mapped.0.is_nan());
        assert!(mapped.1.is_nan());

        let source = [point, (2.0, 3.0), point];
        let mut distinct = [(0.0, 0.0); 3];
        matrix.map_points(&mut distinct, &source);
        assert!(distinct[0].0.is_nan());
        assert!(distinct[0].1.is_nan());
        assert!(distinct[2].0.is_nan());
        assert!(distinct[2].1.is_nan());

        let mut in_place = source;
        matrix.map_points_in_place(&mut in_place);
        for (actual, expected) in in_place.into_iter().zip(distinct) {
            assert_eq!(
                [actual.0.to_bits(), actual.1.to_bits()],
                [expected.0.to_bits(), expected.1.to_bits()]
            );
        }
    }

    #[test]
    fn map_points_preserves_fused_numeric_evaluation() {
        let point = f32::from_bits(0x3f80_0001);
        let matrix = f32::from_bits(0x3f7f_ffff);
        assert_eq!(map_points_fma(point, matrix, -1.0).to_bits(), 0x337f_fffe);
        assert_eq!((point * matrix - 1.0).to_bits(), 0x0000_0000);
    }

    #[test]
    fn constructors_scale_translate_invert_and_operators_match_cpp_contracts() {
        assert_eq!(Mat2D::default(), Mat2D::IDENTITY);
        assert_eq!(
            Mat2D::from_scale(2.0, 3.0),
            Mat2D([2.0, 0.0, 0.0, 3.0, 0.0, 0.0])
        );
        assert_eq!(
            Mat2D::from_translation(4.0, 5.0),
            Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 5.0])
        );
        assert_eq!(
            Mat2D::from_scale_and_translation(2.0, 3.0, 4.0, 5.0),
            Mat2D([2.0, 0.0, 0.0, 3.0, 4.0, 5.0])
        );

        let matrix = Mat2D([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(matrix.values(), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(
            (
                matrix.xx(),
                matrix.xy(),
                matrix.yx(),
                matrix.yy(),
                matrix.tx(),
                matrix.ty(),
            ),
            (1.0, 2.0, 3.0, 4.0, 5.0, 6.0)
        );
        assert_eq!(matrix.translation(), (5.0, 6.0));
        assert_eq!(matrix[2], 3.0);
        let mut fields = matrix;
        fields.set_xx(7.0);
        fields.set_xy(8.0);
        fields.set_yx(9.0);
        fields.set_yy(10.0);
        fields.set_tx(11.0);
        fields.set_ty(12.0);
        fields[0] = 13.0;
        assert_eq!(fields, Mat2D([13.0, 8.0, 9.0, 10.0, 11.0, 12.0]));
        assert_eq!(
            matrix.scale((7.0, 8.0)),
            Mat2D([7.0, 14.0, 24.0, 32.0, 5.0, 6.0])
        );
        let mut scaled_in_place = matrix;
        scaled_in_place.scale_by_values(7.0, 8.0);
        assert_eq!(scaled_in_place, matrix.scale((7.0, 8.0)));
        assert_eq!(
            matrix.translate((7.0, 8.0)),
            Mat2D([1.0, 2.0, 3.0, 4.0, 12.0, 14.0])
        );
        assert_eq!(matrix * (7.0, 8.0), (36.0, 52.0));

        let rhs = Mat2D([2.0, 0.0, 0.0, 3.0, 4.0, 5.0]);
        assert_eq!(matrix * rhs, matrix.multiply(rhs));
        let mut assigned = matrix;
        assigned *= rhs;
        assert_eq!(assigned, matrix.multiply(rhs));

        assert_eq!(Mat2D([0.0; 6]).invert(), None);
        assert_eq!(Mat2D([0.0; 6]).invert_or_identity(), Mat2D::IDENTITY);
        let invertible = Mat2D([2.0, 0.0, 0.0, 4.0, 6.0, 8.0]);
        assert_eq!(
            invertible.invert(),
            Some(Mat2D([0.5, -0.0, -0.0, 0.25, -3.0, -2.0]))
        );

        let components = Mat2D::from_scale_and_translation(2.0, 3.0, 4.0, 5.0).decompose();
        assert_eq!(
            (
                components.x,
                components.y,
                components.scale_x,
                components.scale_y,
                components.rotation,
                components.skew,
            ),
            (4.0, 5.0, 2.0, 3.0, 0.0, 0.0)
        );
        assert_eq!(
            Mat2D::compose(components),
            Mat2D::from_scale_and_translation(2.0, 3.0, 4.0, 5.0)
        );
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

    #[test]
    fn point_and_direction_transforms_preserve_pinned_cpp_contraction_bits() {
        // These are literal output bits from pinned arm64 C++ compiled with
        // `-O3 -ffp-contract=on`. The first case distinguishes the emitted
        // two-product contraction from Rust's ordinary `*`/`+` evaluation.
        let contraction = Mat2D([1.000_000_1, 1.000_000_1, 1.000_000_1, 1.000_000_1, 0.0, 0.0]);
        let point = (std::f32::consts::PI, -2.718_281_7);
        let transformed = contraction.transform_point(point.0, point.1);
        assert_eq!(
            [transformed.0.to_bits(), transformed.1.to_bits()],
            [0x3ed8_bc3d, 0x3ed8_bc3d],
            "expected bits captured from pinned C++ matrix-vector operator"
        );
        let transformed = contraction * point;
        assert_eq!(
            [transformed.0.to_bits(), transformed.1.to_bits()],
            [0x3ed8_bc3d, 0x3ed8_bc3d],
            "Mul must reach the same pinned matrix-vector owner"
        );
        let transformed = contraction.transform_direction(point.0, point.1);
        assert_eq!(
            [transformed.0.to_bits(), transformed.1.to_bits()],
            [0x3ed8_bc3d, 0x3ed8_bc3d],
            "expected bits captured from pinned C++ Vec2D::transformDir"
        );

        // The second pinned oracle distinguishes the operator's separate
        // final translation add from Mat2D::mapPoints' nested affine FMA.
        let translation_grouping = Mat2D([
            f32::from_bits(0xbf18_5aa5),
            f32::from_bits(0xbf18_5aa5),
            f32::from_bits(0x3f5b_24a3),
            f32::from_bits(0x3f5b_24a3),
            f32::from_bits(0x3f20_f4c4),
            f32::from_bits(0x3f20_f4c4),
        ]);
        let translated = translation_grouping
            .transform_point(f32::from_bits(0xbf33_ac98), f32::from_bits(0x3f3a_0788));
        assert_eq!(
            [translated.0.to_bits(), translated.1.to_bits()],
            [0x3fd5_90f7, 0x3fd5_90f7],
            "expected bits captured from pinned C++ matrix-vector operator"
        );

        // ARM64's final fadd preserves its left operand's qNaN payload. The
        // pinned operator places translation on that side of the final add.
        let linear_nan = f32::from_bits(0x7fc0_aaaa);
        let translation_nan = f32::from_bits(0x7fc0_bbbb);
        let nan_payload = Mat2D([
            linear_nan,
            linear_nan,
            0.0,
            0.0,
            translation_nan,
            translation_nan,
        ])
        .transform_point(1.0, 1.0);
        assert_eq!(
            [nan_payload.0.to_bits(), nan_payload.1.to_bits()],
            [0x7fc0_bbbb, 0x7fc0_bbbb],
            "expected qNaN payload bits captured from pinned ARM64 C++"
        );
    }

    #[test]
    fn decompose_preserves_pinned_cpp_contraction_bits() {
        let components = Mat2D([
            1.000_000_1,
            std::f32::consts::PI,
            std::f32::consts::E,
            -f32::EPSILON,
            5.0,
            6.0,
        ])
        .decompose();

        assert_eq!(
            [
                components.x.to_bits(),
                components.y.to_bits(),
                components.scale_x.to_bits(),
                components.scale_y.to_bits(),
                components.rotation.to_bits(),
                components.skew.to_bits(),
            ],
            [
                0x40a0_0000,
                0x40c0_0000,
                0x4053_008c,
                0xc025_c63e,
                0x3fa1_9dc5,
                0x3e7a_efac,
            ],
            "expected bits captured from pinned C++ Mat2D::decompose"
        );
    }

    #[test]
    fn compose_preserves_pinned_cpp_skew_contraction_bits() {
        let components = TransformComponents {
            x: 5.0,
            y: 6.0,
            scale_x: f32::from_bits(0x4053_008c),
            scale_y: f32::from_bits(0xc025_c63e),
            rotation: f32::from_bits(0x3fa1_9dc5),
            skew: f32::from_bits(0x3e7a_efac),
        };

        assert_eq!(
            Mat2D::compose(components).0.map(f32::to_bits),
            [
                0x3f80_0000,
                0x4049_0fdb,
                0x402d_a5fb,
                0xbc81_59e8,
                0x40a0_0000,
                0x40c0_0000,
            ],
            "expected bits captured from pinned C++ Mat2D::compose"
        );
    }
}
