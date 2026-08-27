use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, nested_animation::NestedAnimation,
};

pub trait NestedLinearAnimationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn mix_changed(&mut self) {}
}

pub struct NestedLinearAnimationBase {
    pub base: NestedAnimation,
    mix: f32,
}

impl Default for NestedLinearAnimationBase {
    fn default() -> Self {
        Self {
            base: NestedAnimation::default(),
            mix: 1.0,
        }
    }
}

impl NestedLinearAnimationBase {
    pub const TYPE_KEY: u16 = 97;
    pub const MIX_PROPERTY_KEY: u16 = 200;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 93 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn mix(&self) -> f32 {
        self.mix
    }
    pub fn set_mix(&mut self, value: f32, callbacks: &mut impl NestedLinearAnimationBaseCallbacks) {
        if self.mix == value {
            return;
        }
        self.mix = value;
        callbacks.mix_changed();
        callbacks.notify_property_changed(Self::MIX_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedLinearAnimationBaseCallbacks) {
        self.mix = object.mix;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedLinearAnimationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::MIX_PROPERTY_KEY => {
                self.mix = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
