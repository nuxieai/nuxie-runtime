use crate::mechanical_port::source::{
    animation::{easing::Easing, elastic_ease::ElasticEase},
    generated::animation::elastic_interpolator_base::ElasticInterpolatorBase,
    status_code::StatusCode,
};

pub struct ElasticInterpolator {
    pub base: ElasticInterpolatorBase,
    elastic: ElasticEase,
}

impl Default for ElasticInterpolator {
    fn default() -> Self {
        Self {
            base: ElasticInterpolatorBase::default(),
            elastic: ElasticEase::new(1.0, 0.5),
        }
    }
}

impl ElasticInterpolator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn easing(&self) -> Option<Easing> {
        match self.base.easing_value() {
            0 => Some(Easing::EaseIn),
            1 => Some(Easing::EaseOut),
            2 => Some(Easing::EaseInOut),
            _ => None,
        }
    }
    pub fn initialize(&mut self) {
        self.elastic = ElasticEase::new(
            self.base.amplitude(),
            if self.base.period() == 0.0 {
                0.5
            } else {
                self.base.period()
            },
        );
    }
    pub fn on_added_dirty(&mut self) -> StatusCode {
        self.initialize();
        StatusCode::Ok
    }
    pub fn transform_value(&self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        value_from + (value_to - value_from) * self.transform(factor)
    }
    pub fn transform(&self, factor: f32) -> f32 {
        match self.easing() {
            Some(Easing::EaseIn) => self.elastic.ease_in(factor),
            Some(Easing::EaseOut) => self.elastic.ease_out(factor),
            Some(Easing::EaseInOut) => self.elastic.ease_in_out(factor),
            None => factor,
        }
    }
}
