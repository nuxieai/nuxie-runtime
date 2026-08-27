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
        (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + 3.0 * a1) * t
    }

    fn slope(t: f32, a1: f32, a2: f32) -> f32 {
        3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + 3.0 * a1
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
