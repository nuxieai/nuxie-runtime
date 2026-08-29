use super::mat2d::Mat2D;

const EPSILON: f32 = 1.0 / 4096.0;

fn scalar_dot(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * b + c * d
}

fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

impl Mat2D {
    pub fn find_max_scale(&self) -> f32 {
        if self.xy() == 0.0 && self.yx() == 0.0 {
            return cpp_max(self.xx().abs(), self.yy().abs());
        }

        let a = scalar_dot(self.xx(), self.xx(), self.xy(), self.xy());
        let b = scalar_dot(self.xx(), self.yx(), self.yy(), self.xy());
        let c = scalar_dot(self.yx(), self.yx(), self.yy(), self.yy());
        let b_squared = b * b;
        let mut result = if b_squared <= EPSILON * EPSILON {
            cpp_max(a, c)
        } else {
            let a_minus_c = a - c;
            let a_plus_c_over_2 = (a + c) * 0.5;
            let x = (a_minus_c * a_minus_c + 4.0 * b_squared).sqrt() * 0.5;
            a_plus_c_over_2 + x
        };
        if !result.is_finite() {
            result = 0.0;
        }
        result = cpp_max(result, 0.0);
        result.sqrt()
    }
}
