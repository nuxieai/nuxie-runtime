#[derive(Debug, Clone, Copy)]
struct RuntimeElasticInterpolator {
    easing_value: u64,
    amplitude: f32,
    period: f32,
    elastic: RuntimeElasticEase,
}

impl RuntimeElasticInterpolator {
    fn on_added_dirty(easing_value: u64, amplitude: f32, period: f32) -> Self {
        let mut interpolator = Self {
            easing_value,
            amplitude,
            period,
            elastic: RuntimeElasticEase::new(1.0, 0.5),
        };
        interpolator.initialize();
        interpolator
    }

    fn initialize(&mut self) {
        self.elastic = RuntimeElasticEase::new(
            self.amplitude,
            if self.period == 0.0 {
                0.5
            } else {
                self.period
            },
        );
    }

    fn transform_value(self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        value_from + (value_to - value_from) * self.transform(factor)
    }

    fn transform(self, factor: f32) -> f32 {
        match self.easing_value {
            0 => self.elastic.ease_in(factor),
            1 => self.elastic.ease_out(factor),
            2 => self.elastic.ease_in_out(factor),
            _ => factor,
        }
    }
}

fn elastic_interpolator_transform(
    factor: f32,
    amplitude: f32,
    serialized_period: f32,
    easing_value: u64,
) -> f32 {
    RuntimeElasticInterpolator::on_added_dirty(easing_value, amplitude, serialized_period)
        .transform(factor)
}
