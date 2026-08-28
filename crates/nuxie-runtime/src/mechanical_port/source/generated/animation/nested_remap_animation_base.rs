use crate::mechanical_port::source::{
    animation::nested_linear_animation::NestedLinearAnimation,
    animation::nested_remap_animation::NestedRemapAnimation, core::binary_reader::BinaryReader,
};

pub trait NestedRemapAnimationBaseCallbacks: crate::mechanical_port::source::generated::animation::nested_linear_animation_base::NestedLinearAnimationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn time_changed(&mut self) {}
}

pub struct NestedRemapAnimationBase {
    pub base: NestedLinearAnimation,
    time: f32,
}

impl Default for NestedRemapAnimationBase {
    fn default() -> Self {
        Self {
            base: NestedLinearAnimation::default(),
            time: 0.0,
        }
    }
}

impl NestedRemapAnimationBase {
    pub const TYPE_KEY: u16 = 98;
    pub const TIME_PROPERTY_KEY: u16 = 202;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 97 | 93 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn time(&self) -> f32 {
        self.time
    }
    pub fn set_time(&mut self, value: f32, callbacks: &mut impl NestedRemapAnimationBaseCallbacks) {
        if !self.set_time_value(value) {
            return;
        }
        callbacks.time_changed();
        callbacks.notify_property_changed(Self::TIME_PROPERTY_KEY);
    }

    pub(crate) fn set_time_value(&mut self, value: f32) -> bool {
        if self.time == value {
            return false;
        }
        self.time = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl NestedRemapAnimationBaseCallbacks,
    ) -> NestedRemapAnimation {
        let mut cloned = NestedRemapAnimation::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedRemapAnimationBaseCallbacks) {
        self.time = object.time;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedRemapAnimationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TIME_PROPERTY_KEY => {
                self.time = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NestedRemapAnimationBase {
    type Target = NestedLinearAnimation;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedRemapAnimationBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
