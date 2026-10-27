//! Our layer: the exact affine decomposition every flight blend uses.
//!
//! Upstream's `Mat2D::decompose` returns skew as an angle (`atan2`) while
//! `Mat2D::compose` applies it as a shear coefficient, so a skewed matrix
//! does not survive a round trip. Upstream keeps that pair unchanged; flights
//! use this one instead, which recomposes the matrix it decomposed.
use crate::source::math::mat2d::Mat2D;

/// `Translate(x, y) x Rotate(rotation) x Scale(scale_x, scale_y) x Shear(shear)`,
/// where `Shear(k)` maps `(u, v)` to `(u + k v, v)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Affine {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub shear: f32,
}

/// `(1 - t) a + t b`: exactly `a` at 0 and exactly `b` at 1.
pub(super) fn mix(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

impl Affine {
    pub fn decompose(m: &Mat2D) -> Self {
        let (m0, m1, m2, m3) = (m[0], m[1], m[2], m[3]);
        let scale_x = m0.hypot(m1);
        if scale_x == 0.0 {
            // A zero first column: the second alone gives rotation and y scale.
            return Self {
                x: m[4],
                y: m[5],
                rotation: (-m2).atan2(m3),
                scale_x: 0.0,
                scale_y: m2.hypot(m3),
                shear: 0.0,
            };
        }
        Self {
            x: m[4],
            y: m[5],
            rotation: m1.atan2(m0),
            scale_x,
            // The determinant keeps a mirrored matrix's sign in y.
            scale_y: (m0 * m3 - m2 * m1) / scale_x,
            // A coefficient, not an angle.
            shear: (m0 * m2 + m1 * m3) / (scale_x * scale_x),
        }
    }

    pub fn compose(&self) -> Mat2D {
        let (sin, cos) = self.rotation.sin_cos();
        let (x_axis_x, x_axis_y) = (cos * self.scale_x, sin * self.scale_x);
        Mat2D::new(
            x_axis_x,
            x_axis_y,
            self.shear * x_axis_x - sin * self.scale_y,
            self.shear * x_axis_y + cos * self.scale_y,
            self.x,
            self.y,
        )
    }

    /// Every value blends as `(1 - t) a + t b`; rotation the short way.
    pub fn blend(a: &Self, b: &Self, t: f32) -> Self {
        let turn = (b.rotation - a.rotation + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        Self {
            x: mix(a.x, b.x, t),
            y: mix(a.y, b.y, t),
            rotation: mix(a.rotation, a.rotation + turn, t),
            scale_x: mix(a.scale_x, b.scale_x, t),
            scale_y: mix(a.scale_y, b.scale_y, t),
            shear: mix(a.shear, b.shear, t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_matrix(got: Mat2D, want: Mat2D) {
        for i in 0..6 {
            assert!(
                (got[i] - want[i]).abs() <= 1e-5 * want[i].abs().max(1.0),
                "entry {i}: expected {want:?}, got {got:?}"
            );
        }
    }

    #[test]
    fn exact_pair_rebuilds_skewed_rotated_mirrored_and_unequally_scaled_matrices() {
        let skewed = Mat2D::new(2.0, 0.0, 0.0, 1.0, 0.0, 0.0) * Mat2D::from_rotation(0.5);
        for matrix in [
            Mat2D::identity(),
            Mat2D::new(1.0, 0.0, 0.75, 1.0, 12.0, -3.0),
            skewed,
            Mat2D::from_rotation(2.8) * Mat2D::from_scale(3.0, 0.25),
            Mat2D::from_scale(-1.0, 1.0),
            Mat2D::from_scale(1.0, -2.0) * Mat2D::from_rotation(-1.2),
            Mat2D::from_scale(-1.5, -1.5),
            Mat2D::new(0.3, -1.7, 2.2, 0.4, 400.0, 250.0),
            // A zero first column keeps its second.
            Mat2D::new(0.0, 0.0, -1.5, 0.5, 4.0, 5.0),
            Mat2D::new(0.0, 0.0, 0.0, 0.0, 4.0, 5.0),
        ] {
            assert_matrix(Affine::decompose(&matrix).compose(), matrix);
        }
        // Upstream's pair is the one this replaces: its skew is off at rest.
        let upstream = Mat2D::compose(&skewed.decompose());
        assert!((0..4).any(|i| (upstream[i] - skewed[i]).abs() > 1e-3));
    }

    #[test]
    fn blend_is_exact_at_both_ends_and_turns_the_short_way() {
        let a = Mat2D::from_rotation(3.0) * Mat2D::new(1.5, 0.0, 0.4, 0.5, 10.0, 20.0);
        let b = Mat2D::from_rotation(-3.0) * Mat2D::from_scale(2.0, 2.0);
        let (da, db) = (Affine::decompose(&a), Affine::decompose(&b));
        assert_eq!(Affine::blend(&da, &db, 0.0), da);
        assert_matrix(Affine::blend(&da, &db, 1.0).compose(), b);
        let half = Affine::blend(&da, &db, 0.5);
        // From 3 to -3 radians through pi, not through zero.
        assert!(half.rotation.cos() < -0.99, "{}", half.rotation);
    }
}
