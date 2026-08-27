#[derive(Debug, Clone, Copy)]
struct RuntimeElasticEase {
    amplitude: f32,
    period: f32,
    s: f32,
}

impl RuntimeElasticEase {
    fn new(amplitude: f32, period: f32) -> Self {
        let s = if amplitude < 1.0 {
            period / 4.0
        } else {
            period / (2.0 * std::f32::consts::PI) * (1.0 / amplitude).asin()
        };
        Self {
            amplitude,
            period,
            s,
        }
    }

    fn compute_actual_amplitude(self, time: f32) -> f32 {
        if self.amplitude < 1.0 {
            let shift_abs = self.s.abs();
            let time_abs = time.abs();
            if time_abs < shift_abs {
                let l = time_abs / shift_abs;
                return (self.amplitude * l) + (1.0 - l);
            }
        }

        self.amplitude
    }

    fn ease_out(self, factor: f32) -> f32 {
        let time = factor;
        let actual_amplitude = self.compute_actual_amplitude(time);
        actual_amplitude
            * 2.0_f32.powf(10.0 * -time)
            * ((time - self.s) * (2.0 * std::f32::consts::PI) / self.period).sin()
            + 1.0
    }

    fn ease_in(self, factor: f32) -> f32 {
        let time = factor - 1.0;
        let actual_amplitude = self.compute_actual_amplitude(time);
        -(actual_amplitude
            * 2.0_f32.powf(10.0 * time)
            * ((-time - self.s) * (2.0 * std::f32::consts::PI) / self.period).sin())
    }

    fn ease_in_out(self, factor: f32) -> f32 {
        let time = factor * 2.0 - 1.0;
        let actual_amplitude = self.compute_actual_amplitude(time);
        if time < 0.0 {
            -0.5 * actual_amplitude
                * 2.0_f32.powf(10.0 * time)
                * ((-time - self.s) * (2.0 * std::f32::consts::PI) / self.period).sin()
        } else {
            0.5 * (actual_amplitude
                * 2.0_f32.powf(10.0 * -time)
                * ((time - self.s) * (2.0 * std::f32::consts::PI) / self.period).sin())
                + 1.0
        }
    }
}

#[cfg(test)]
fn elastic_actual_amplitude(time: f32, amplitude: f32, shift: f32) -> f32 {
    RuntimeElasticEase {
        amplitude,
        period: 0.0,
        s: shift,
    }
    .compute_actual_amplitude(time)
}

#[cfg(test)]
fn elastic_ease_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    RuntimeElasticEase {
        amplitude,
        period,
        s: shift,
    }
    .ease_out(factor)
}

#[cfg(test)]
fn elastic_ease_in(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    RuntimeElasticEase {
        amplitude,
        period,
        s: shift,
    }
    .ease_in(factor)
}

#[cfg(test)]
fn elastic_ease_in_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    RuntimeElasticEase {
        amplitude,
        period,
        s: shift,
    }
    .ease_in_out(factor)
}
