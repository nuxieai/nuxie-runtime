use core::ops::{Index, IndexMut, Mul};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mat4 {
    buffer: [f32; 16],
}
impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}
impl Mat4 {
    pub const fn identity() -> Self {
        Self {
            buffer: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        c0x: f32,
        c0y: f32,
        c0z: f32,
        c0w: f32,
        c1x: f32,
        c1y: f32,
        c1z: f32,
        c1w: f32,
        c2x: f32,
        c2y: f32,
        c2z: f32,
        c2w: f32,
        c3x: f32,
        c3y: f32,
        c3z: f32,
        c3w: f32,
    ) -> Self {
        Self {
            buffer: [
                c0x, c0y, c0z, c0w, c1x, c1y, c1z, c1w, c2x, c2y, c2z, c2w, c3x, c3y, c3z, c3w,
            ],
        }
    }
    pub fn values(&self) -> &[f32; 16] {
        &self.buffer
    }
    pub fn values_mut(&mut self) -> &mut [f32; 16] {
        &mut self.buffer
    }
    pub fn from_translation(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::identity();
        m[12] = x;
        m[13] = y;
        m[14] = z;
        m
    }
    pub fn from_scale(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::identity();
        m[0] = x;
        m[5] = y;
        m[10] = z;
        m
    }
    pub fn from_rotation_x(rad: f32) -> Self {
        let (c, s) = (rad.cos(), rad.sin());
        let mut m = Self::identity();
        m[5] = c;
        m[6] = s;
        m[9] = -s;
        m[10] = c;
        m
    }
    pub fn from_rotation_y(rad: f32) -> Self {
        let (c, s) = (rad.cos(), rad.sin());
        let mut m = Self::identity();
        m[0] = c;
        m[2] = -s;
        m[8] = s;
        m[10] = c;
        m
    }
    pub fn from_rotation_z(rad: f32) -> Self {
        let (c, s) = (rad.cos(), rad.sin());
        let mut m = Self::identity();
        m[0] = c;
        m[1] = s;
        m[4] = -s;
        m[5] = c;
        m
    }
    pub fn perspective(
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
        depth_zero_to_one: bool,
    ) -> Self {
        let f = 1.0 / (fov_y * 0.5).tan();
        let nf = 1.0 / (near - far);
        let mut m = Self { buffer: [0.0; 16] };
        m[0] = f / aspect;
        m[5] = f;
        if depth_zero_to_one {
            m[10] = far * nf;
            m[14] = far * near * nf;
        } else {
            m[10] = (far + near) * nf;
            m[14] = 2.0 * far * near * nf;
        }
        m[11] = -1.0;
        m
    }
    pub fn perspective_reverse_z(fov_y: f32, aspect: f32, near: f32) -> Self {
        let f = 1.0 / (fov_y * 0.5).tan();
        let mut m = Self { buffer: [0.0; 16] };
        m[0] = f / aspect;
        m[5] = f;
        m[10] = 0.0;
        m[11] = -1.0;
        m[14] = near;
        m
    }
    pub fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> Self {
        let mut f = [center[0] - eye[0], center[1] - eye[1], center[2] - eye[2]];
        let inverse = 1.0 / (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        for value in &mut f {
            *value *= inverse;
        }
        let mut s = [
            f[1] * up[2] - f[2] * up[1],
            f[2] * up[0] - f[0] * up[2],
            f[0] * up[1] - f[1] * up[0],
        ];
        let inverse = 1.0 / (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
        for value in &mut s {
            *value *= inverse;
        }
        let u = [
            s[1] * f[2] - s[2] * f[1],
            s[2] * f[0] - s[0] * f[2],
            s[0] * f[1] - s[1] * f[0],
        ];
        let mut m = Self::identity();
        m[0] = s[0];
        m[1] = u[0];
        m[2] = -f[0];
        m[4] = s[1];
        m[5] = u[1];
        m[6] = -f[1];
        m[8] = s[2];
        m[9] = u[2];
        m[10] = -f[2];
        m[12] = -(s[0] * eye[0] + s[1] * eye[1] + s[2] * eye[2]);
        m[13] = -(u[0] * eye[0] + u[1] * eye[1] + u[2] * eye[2]);
        m[14] = f[0] * eye[0] + f[1] * eye[1] + f[2] * eye[2];
        m
    }
    pub fn ortho(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
        depth_zero_to_one: bool,
    ) -> Self {
        let mut m = Self::identity();
        m[0] = 2.0 / (right - left);
        m[5] = 2.0 / (top - bottom);
        m[12] = -(right + left) / (right - left);
        m[13] = -(top + bottom) / (top - bottom);
        if depth_zero_to_one {
            m[10] = -1.0 / (far - near);
            m[14] = -near / (far - near);
        } else {
            m[10] = -2.0 / (far - near);
            m[14] = -(far + near) / (far - near);
        }
        m
    }
    pub fn multiply(lhs: Self, rhs: Self) -> Self {
        let mut output = Self::identity();
        for column in 0..4 {
            for row in 0..4 {
                output[column * 4 + row] = lhs[row] * rhs[column * 4]
                    + lhs[4 + row] * rhs[column * 4 + 1]
                    + lhs[8 + row] * rhs[column * 4 + 2]
                    + lhs[12 + row] * rhs[column * 4 + 3];
            }
        }
        output
    }
    pub fn multiply_affine(lhs: Self, rhs: Self) -> Self {
        let mut output = Self::identity();
        for column in 0..3 {
            for row in 0..4 {
                output[column * 4 + row] = lhs[row] * rhs[column * 4]
                    + lhs[4 + row] * rhs[column * 4 + 1]
                    + lhs[8 + row] * rhs[column * 4 + 2];
            }
        }
        for row in 0..4 {
            output[12 + row] = lhs[row] * rhs[12]
                + lhs[4 + row] * rhs[13]
                + lhs[8 + row] * rhs[14]
                + lhs[12 + row];
        }
        output
    }
    pub fn transform_vec4(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4] {
        core::array::from_fn(|row| {
            self[row] * x + self[4 + row] * y + self[8 + row] * z + self[12 + row] * w
        })
    }
    pub fn transposed(self) -> Self {
        let mut output = Self::identity();
        for row in 0..4 {
            for column in 0..4 {
                output[row * 4 + column] = self[column * 4 + row];
            }
        }
        output
    }
    pub fn invert(&self, result: &mut Self) -> bool {
        let m = &self.buffer;
        let mut inverse = [0.0; 16];
        inverse[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
            + m[9] * m[7] * m[14]
            + m[13] * m[6] * m[11]
            - m[13] * m[7] * m[10];
        inverse[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
            - m[8] * m[7] * m[14]
            - m[12] * m[6] * m[11]
            + m[12] * m[7] * m[10];
        inverse[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
            + m[8] * m[7] * m[13]
            + m[12] * m[5] * m[11]
            - m[12] * m[7] * m[9];
        inverse[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
            - m[8] * m[6] * m[13]
            - m[12] * m[5] * m[10]
            + m[12] * m[6] * m[9];
        inverse[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
            - m[9] * m[3] * m[14]
            - m[13] * m[2] * m[11]
            + m[13] * m[3] * m[10];
        inverse[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
            + m[8] * m[3] * m[14]
            + m[12] * m[2] * m[11]
            - m[12] * m[3] * m[10];
        inverse[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
            - m[8] * m[3] * m[13]
            - m[12] * m[1] * m[11]
            + m[12] * m[3] * m[9];
        inverse[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
            + m[8] * m[2] * m[13]
            + m[12] * m[1] * m[10]
            - m[12] * m[2] * m[9];
        inverse[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
            + m[5] * m[3] * m[14]
            + m[13] * m[2] * m[7]
            - m[13] * m[3] * m[6];
        inverse[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
            - m[4] * m[3] * m[14]
            - m[12] * m[2] * m[7]
            + m[12] * m[3] * m[6];
        inverse[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
            + m[4] * m[3] * m[13]
            + m[12] * m[1] * m[7]
            - m[12] * m[3] * m[5];
        inverse[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
            - m[4] * m[2] * m[13]
            - m[12] * m[1] * m[6]
            + m[12] * m[2] * m[5];
        inverse[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
            - m[5] * m[3] * m[10]
            - m[9] * m[2] * m[7]
            + m[9] * m[3] * m[6];
        inverse[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
            + m[4] * m[3] * m[10]
            + m[8] * m[2] * m[7]
            - m[8] * m[3] * m[6];
        inverse[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
            - m[4] * m[3] * m[9]
            - m[8] * m[1] * m[7]
            + m[8] * m[3] * m[5];
        inverse[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
            + m[4] * m[2] * m[9]
            + m[8] * m[1] * m[6]
            - m[8] * m[2] * m[5];
        let determinant =
            m[0] * inverse[0] + m[1] * inverse[4] + m[2] * inverse[8] + m[3] * inverse[12];
        if determinant == 0.0 {
            return false;
        }
        let inverse_determinant = 1.0 / determinant;
        for index in 0..16 {
            result[index] = inverse[index] * inverse_determinant;
        }
        true
    }
    pub fn invert_affine(&self, result: &mut Self) -> bool {
        let m = &self.buffer;
        let c00 = m[5] * m[10] - m[6] * m[9];
        let c10 = m[6] * m[8] - m[4] * m[10];
        let c20 = m[4] * m[9] - m[5] * m[8];
        let determinant = m[0] * c00 + m[1] * c10 + m[2] * c20;
        if determinant == 0.0 {
            return false;
        }
        let inverse = 1.0 / determinant;
        let c01 = m[2] * m[9] - m[1] * m[10];
        let c02 = m[1] * m[6] - m[2] * m[5];
        let c11 = m[0] * m[10] - m[2] * m[8];
        let c12 = m[2] * m[4] - m[0] * m[6];
        let c21 = m[1] * m[8] - m[0] * m[9];
        let c22 = m[0] * m[5] - m[1] * m[4];
        let (r00, r01, r02) = (c00 * inverse, c10 * inverse, c20 * inverse);
        let (r10, r11, r12) = (c01 * inverse, c11 * inverse, c21 * inverse);
        let (r20, r21, r22) = (c02 * inverse, c12 * inverse, c22 * inverse);
        let (tx, ty, tz) = (m[12], m[13], m[14]);
        let (ix, iy, iz) = (
            -(r00 * tx + r01 * ty + r02 * tz),
            -(r10 * tx + r11 * ty + r12 * tz),
            -(r20 * tx + r21 * ty + r22 * tz),
        );
        result.buffer = [
            r00, r10, r20, 0.0, r01, r11, r21, 0.0, r02, r12, r22, 0.0, ix, iy, iz, 1.0,
        ];
        true
    }
}
impl Index<usize> for Mat4 {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        &self.buffer[index]
    }
}
impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        &mut self.buffer[index]
    }
}
impl Mul for Mat4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::multiply(self, rhs)
    }
}
