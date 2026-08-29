const SPLINE_TABLE_SIZE: usize = 11;
const SAMPLE_STEP_SIZE: f32 = 1.0 / (SPLINE_TABLE_SIZE as f32 - 1.0);
const NEWTON_ITERATIONS: usize = 4;
const NEWTON_MIN_SLOPE: f32 = 0.001;
const SUBDIVISION_PRECISION: f32 = 0.0000001;
const SUBDIVISION_MAX_ITERATIONS: usize = 10;

#[derive(Default)]
pub struct CubicInterpolatorSolver {
    values: [f32; SPLINE_TABLE_SIZE],
    x1: f32,
    x2: f32,
}

impl CubicInterpolatorSolver {
    pub fn calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
        let coefficient_a = 3.0_f32.mul_add(a1, (-3.0_f32).mul_add(a2, 1.0));
        let coefficient_b = 3.0_f32.mul_add(a2, -(6.0 * a1));
        coefficient_a.mul_add(t, coefficient_b).mul_add(t, 3.0 * a1) * t
    }

    fn slope(t: f32, a1: f32, a2: f32) -> f32 {
        let coefficient_a = 3.0_f32.mul_add(a1, (-3.0_f32).mul_add(a2, 1.0));
        let coefficient_b = 3.0_f32.mul_add(a2, -(6.0 * a1));
        (2.0 * coefficient_b)
            .mul_add(t, 3.0 * coefficient_a * t * t)
            .mul_add(1.0, 3.0 * a1)
    }

    pub fn build(&mut self, x1: f32, x2: f32) {
        self.x1 = x1;
        self.x2 = x2;
        for (index, value) in self.values.iter_mut().enumerate() {
            *value = Self::calc_bezier(index as f32 * SAMPLE_STEP_SIZE, x1, x2);
        }
    }

    pub fn get_t(&self, x: f32) -> f32 {
        let mut interval_start = 0.0;
        let mut current_sample = 1;
        let last_sample = SPLINE_TABLE_SIZE - 1;
        while current_sample != last_sample && self.values[current_sample] <= x {
            current_sample += 1;
            interval_start += SAMPLE_STEP_SIZE;
        }
        current_sample -= 1;

        let distance = (x - self.values[current_sample])
            / (self.values[current_sample + 1] - self.values[current_sample]);
        let mut guess = interval_start + distance * SAMPLE_STEP_SIZE;
        let initial_slope = Self::slope(guess, self.x1, self.x2);
        if initial_slope >= NEWTON_MIN_SLOPE {
            for _ in 0..NEWTON_ITERATIONS {
                let current_slope = Self::slope(guess, self.x1, self.x2);
                if current_slope == 0.0 {
                    return guess;
                }
                let current_x = Self::calc_bezier(guess, self.x1, self.x2) - x;
                guess -= current_x / current_slope;
            }
            guess
        } else if initial_slope == 0.0 {
            guess
        } else {
            let mut upper = interval_start + SAMPLE_STEP_SIZE;
            let mut current_t = interval_start;
            for _ in 0..SUBDIVISION_MAX_ITERATIONS {
                current_t = interval_start + (upper - interval_start) / 2.0;
                let current_x = Self::calc_bezier(current_t, self.x1, self.x2) - x;
                if current_x > 0.0 {
                    upper = current_t;
                } else {
                    interval_start = current_t;
                }
                if current_x.abs() <= SUBDIVISION_PRECISION {
                    break;
                }
            }
            current_t
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CubicInterpolatorSolver, SAMPLE_STEP_SIZE};

    const X1: f32 = f32::from_bits(0x3ed7_0a3d);
    const X2: f32 = f32::from_bits(0x3f14_7ae1);

    #[test]
    fn calc_bezier_matches_pinned_fused_horner_bits() {
        let t = 3.0 * SAMPLE_STEP_SIZE;
        assert_eq!(
            CubicInterpolatorSolver::calc_bezier(t, X1, X2).to_bits(),
            0x3ea4_c836
        );
    }

    #[test]
    fn slope_matches_pinned_fused_polynomial_bits() {
        let t = f32::from_bits(0x3f40_6f8a);
        assert_eq!(
            CubicInterpolatorSolver::slope(t, X1, X2).to_bits(),
            0x3f78_055e
        );
    }

    #[test]
    fn solver_table_and_newton_result_match_pinned_bits() {
        let mut solver = CubicInterpolatorSolver::default();
        solver.build(X1, X2);
        assert_eq!(
            solver.values.map(f32::to_bits),
            [
                0x0000_0000,
                0x3df3_2378,
                0x3e66_5bea,
                0x3ea4_c836,
                0x3ed3_3093,
                0x3f00_0000,
                0x3f16_67b6,
                0x3f2d_9be4,
                0x3f46_6905,
                0x3f61_9b91,
                0x3f7f_ffff,
            ]
        );
        assert_eq!(
            solver.get_t(f32::from_bits(0x3f3a_2e8c)).to_bits(),
            0x3f40_6f8a
        );
    }
}
