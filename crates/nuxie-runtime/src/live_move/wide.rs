//! Matrices in f64, for the chains that cancel a matrix by its inverse: the
//! rest correction (`drawn.rs`) and a lift's transform (`in_place.rs`). In
//! f32 such a chain loses digits in proportion to the largest translation in
//! it rather than to its result, past 1e-3 on a Screen a few hundred points
//! wide. Each chain is composed here and rounded to f32 once.
use crate::source::math::mat2d::Mat2D;
use std::ops::Mul;

#[derive(Clone, Copy, Debug)]
pub(super) struct Wide([f64; 6]);

impl Wide {
    pub(super) fn of(matrix: Mat2D) -> Self {
        Self(matrix.values().map(f64::from))
    }
    /// None when it has no inverse.
    pub(super) fn inverse(self) -> Option<Self> {
        let m = self.0;
        let det = m[0] * m[3] - m[1] * m[2];
        (det != 0.0 && det.is_finite()).then(|| {
            let d = 1.0 / det;
            Self([
                m[3] * d,
                -m[1] * d,
                -m[2] * d,
                m[0] * d,
                (m[2] * m[5] - m[3] * m[4]) * d,
                (m[1] * m[4] - m[0] * m[5]) * d,
            ])
        })
    }
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn narrow(self) -> Mat2D {
        let m = self.0.map(|v| v as f32);
        Mat2D::new(m[0], m[1], m[2], m[3], m[4], m[5])
    }
}

impl Mul for Wide {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let (a, b) = (self.0, rhs.0);
        Self([
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
            a[0] * b[4] + a[2] * b[5] + a[4],
            a[1] * b[4] + a[3] * b[5] + a[5],
        ])
    }
}
