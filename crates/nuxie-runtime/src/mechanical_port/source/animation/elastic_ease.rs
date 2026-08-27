use std::f32::consts::PI;

pub struct ElasticEase {
    amplitude: f32,
    period: f32,
    phase_shift: f32,
}

impl ElasticEase {
    pub fn new(amplitude: f32, period: f32) -> Self {
        let phase_shift = if amplitude < 1.0 {
            period / 4.0
        } else {
            period / (2.0 * PI) * (1.0 / amplitude).asin()
        };
        Self {
            amplitude,
            period,
            phase_shift,
        }
    }

    pub fn compute_actual_amplitude(&self, time: f32) -> f32 {
        if self.amplitude < 1.0 {
            let threshold = self.phase_shift.abs();
            let absolute_time = time.abs();
            if absolute_time < threshold {
                let ratio = absolute_time / threshold;
                return self.amplitude * ratio + (1.0 - ratio);
            }
        }
        self.amplitude
    }

    pub fn ease_out(&self, factor: f32) -> f32 {
        let amplitude = self.compute_actual_amplitude(factor);
        amplitude
            * 2.0_f32.powf(-10.0 * factor)
            * ((factor - self.phase_shift) * (2.0 * PI) / self.period).sin()
            + 1.0
    }

    pub fn ease_in(&self, factor: f32) -> f32 {
        let time = factor - 1.0;
        let amplitude = self.compute_actual_amplitude(time);
        -(amplitude
            * 2.0_f32.powf(10.0 * time)
            * ((-time - self.phase_shift) * (2.0 * PI) / self.period).sin())
    }

    pub fn ease_in_out(&self, factor: f32) -> f32 {
        let time = factor * 2.0 - 1.0;
        let amplitude = self.compute_actual_amplitude(time);
        if time < 0.0 {
            -0.5 * amplitude
                * 2.0_f32.powf(10.0 * time)
                * ((-time - self.phase_shift) * (2.0 * PI) / self.period).sin()
        } else {
            0.5 * (amplitude
                * 2.0_f32.powf(-10.0 * time)
                * ((time - self.phase_shift) * (2.0 * PI) / self.period).sin())
                + 1.0
        }
    }
}
