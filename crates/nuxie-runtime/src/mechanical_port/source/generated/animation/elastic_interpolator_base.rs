use crate::mechanical_port::source::{
    animation::elastic_interpolator::ElasticInterpolator, core::binary_reader::BinaryReader,
    key_frame_interpolator::KeyFrameInterpolator,
};

pub trait ElasticInterpolatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn easing_value_changed(&mut self) {}
    fn amplitude_changed(&mut self) {}
    fn period_changed(&mut self) {}
}

pub struct ElasticInterpolatorBase {
    pub base: KeyFrameInterpolator,
    easing_value: u32,
    amplitude: f32,
    period: f32,
}

impl Default for ElasticInterpolatorBase {
    fn default() -> Self {
        Self {
            base: KeyFrameInterpolator::default(),
            easing_value: 1,
            amplitude: 1.0,
            period: 1.0,
        }
    }
}

impl ElasticInterpolatorBase {
    pub const TYPE_KEY: u16 = 174;
    pub const EASING_VALUE_PROPERTY_KEY: u16 = 405;
    pub const AMPLITUDE_PROPERTY_KEY: u16 = 406;
    pub const PERIOD_PROPERTY_KEY: u16 = 407;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn easing_value(&self) -> u32 {
        self.easing_value
    }
    pub fn set_easing_value(
        &mut self,
        value: u32,
        callbacks: &mut impl ElasticInterpolatorBaseCallbacks,
    ) {
        if self.easing_value == value {
            return;
        }
        self.easing_value = value;
        callbacks.easing_value_changed();
        callbacks.notify_property_changed(Self::EASING_VALUE_PROPERTY_KEY);
    }
    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }
    pub fn set_amplitude(
        &mut self,
        value: f32,
        callbacks: &mut impl ElasticInterpolatorBaseCallbacks,
    ) {
        if self.amplitude == value {
            return;
        }
        self.amplitude = value;
        callbacks.amplitude_changed();
        callbacks.notify_property_changed(Self::AMPLITUDE_PROPERTY_KEY);
    }
    pub fn period(&self) -> f32 {
        self.period
    }
    pub fn set_period(
        &mut self,
        value: f32,
        callbacks: &mut impl ElasticInterpolatorBaseCallbacks,
    ) {
        if self.period == value {
            return;
        }
        self.period = value;
        callbacks.period_changed();
        callbacks.notify_property_changed(Self::PERIOD_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ElasticInterpolatorBaseCallbacks,
    ) -> ElasticInterpolator {
        let mut cloned = ElasticInterpolator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ElasticInterpolatorBaseCallbacks) {
        self.easing_value = object.easing_value;
        self.amplitude = object.amplitude;
        self.period = object.period;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ElasticInterpolatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::EASING_VALUE_PROPERTY_KEY => {
                self.easing_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::AMPLITUDE_PROPERTY_KEY => {
                self.amplitude = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::PERIOD_PROPERTY_KEY => {
                self.period = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
