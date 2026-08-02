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
