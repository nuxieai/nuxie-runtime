const CUBIC_INTERPOLATOR_SPLINE_TABLE_SIZE: usize = 11;
const CUBIC_INTERPOLATOR_SAMPLE_STEP_SIZE: f32 =
    1.0 / (CUBIC_INTERPOLATOR_SPLINE_TABLE_SIZE as f32 - 1.0);

#[derive(Debug, Clone, Copy, PartialEq)]
struct RuntimeCubicInterpolatorSolver {
    values: [f32; CUBIC_INTERPOLATOR_SPLINE_TABLE_SIZE],
    x1: f32,
    x2: f32,
}

impl RuntimeCubicInterpolatorSolver {
    fn build(x1: f32, x2: f32) -> Self {
        let mut values = [0.0; CUBIC_INTERPOLATOR_SPLINE_TABLE_SIZE];
        for (index, value) in values.iter_mut().enumerate() {
            *value = Self::calc_bezier(index as f32 * CUBIC_INTERPOLATOR_SAMPLE_STEP_SIZE, x1, x2);
        }
        Self { values, x1, x2 }
    }

    fn calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
        cubic_interpolator_calc_bezier(t, a1, a2)
    }

    fn get_t(self, x: f32) -> f32 {
        const NEWTON_ITERATIONS: usize = 4;
        const NEWTON_MIN_SLOPE: f32 = 0.001;
        const SUBDIVISION_PRECISION: f32 = 0.0000001;
        const SUBDIVISION_MAX_ITERATIONS: usize = 10;

        let mut interval_start = 0.0;
        let mut current_sample = 1;
        let last_sample = CUBIC_INTERPOLATOR_SPLINE_TABLE_SIZE - 1;
        while current_sample != last_sample && self.values[current_sample] <= x {
            interval_start += CUBIC_INTERPOLATOR_SAMPLE_STEP_SIZE;
            current_sample += 1;
        }
        current_sample -= 1;

        let dist = (x - self.values[current_sample])
            / (self.values[current_sample + 1] - self.values[current_sample]);
        let mut guess_for_t = interval_start + dist * CUBIC_INTERPOLATOR_SAMPLE_STEP_SIZE;
        let initial_slope = cubic_interpolator_slope(guess_for_t, self.x1, self.x2);
        if initial_slope >= NEWTON_MIN_SLOPE {
            for _ in 0..NEWTON_ITERATIONS {
                let current_slope = cubic_interpolator_slope(guess_for_t, self.x1, self.x2);
                if current_slope == 0.0 {
                    return guess_for_t;
                }
                let current_x = Self::calc_bezier(guess_for_t, self.x1, self.x2) - x;
                guess_for_t -= current_x / current_slope;
            }
            guess_for_t
        } else if initial_slope == 0.0 {
            guess_for_t
        } else {
            let mut upper_bound = interval_start + CUBIC_INTERPOLATOR_SAMPLE_STEP_SIZE;
            let mut iterations = 0;
            loop {
                let current_t = interval_start + (upper_bound - interval_start) / 2.0;
                let current_x = Self::calc_bezier(current_t, self.x1, self.x2) - x;
                if current_x > 0.0 {
                    upper_bound = current_t;
                } else {
                    interval_start = current_t;
                }
                iterations += 1;
                if current_x.abs() <= SUBDIVISION_PRECISION
                    || iterations >= SUBDIVISION_MAX_ITERATIONS
                {
                    return current_t;
                }
            }
        }
    }
}

fn cubic_interpolator_get_t(x: f32, x1: f32, x2: f32) -> f32 {
    RuntimeCubicInterpolatorSolver::build(x1, x2).get_t(x)
}
