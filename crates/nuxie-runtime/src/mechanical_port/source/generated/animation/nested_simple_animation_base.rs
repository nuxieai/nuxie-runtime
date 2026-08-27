use crate::mechanical_port::source::{
    animation::nested_linear_animation::NestedLinearAnimation,
    animation::nested_simple_animation::NestedSimpleAnimation, core::binary_reader::BinaryReader,
};

pub trait NestedSimpleAnimationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn speed_changed(&mut self) {}
    fn is_playing_changed(&mut self) {}
}

pub struct NestedSimpleAnimationBase {
    pub base: NestedLinearAnimation,
    speed: f32,
    is_playing: bool,
}

impl Default for NestedSimpleAnimationBase {
    fn default() -> Self {
        Self {
            base: NestedLinearAnimation::default(),
            speed: 1.0,
            is_playing: false,
        }
    }
}

impl NestedSimpleAnimationBase {
    pub const TYPE_KEY: u16 = 96;
    pub const SPEED_PROPERTY_KEY: u16 = 199;
    pub const IS_PLAYING_PROPERTY_KEY: u16 = 201;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 97 | 93 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn set_speed(
        &mut self,
        value: f32,
        callbacks: &mut impl NestedSimpleAnimationBaseCallbacks,
    ) {
        if self.speed == value {
            return;
        }
        self.speed = value;
        callbacks.speed_changed();
        callbacks.notify_property_changed(Self::SPEED_PROPERTY_KEY);
    }
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
    pub fn set_is_playing(
        &mut self,
        value: bool,
        callbacks: &mut impl NestedSimpleAnimationBaseCallbacks,
    ) {
        if self.is_playing == value {
            return;
        }
        self.is_playing = value;
        callbacks.is_playing_changed();
        callbacks.notify_property_changed(Self::IS_PLAYING_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl NestedSimpleAnimationBaseCallbacks,
    ) -> NestedSimpleAnimation {
        let mut cloned = NestedSimpleAnimation::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedSimpleAnimationBaseCallbacks) {
        self.speed = object.speed;
        self.is_playing = object.is_playing;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedSimpleAnimationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SPEED_PROPERTY_KEY => {
                self.speed = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::IS_PLAYING_PROPERTY_KEY => {
                self.is_playing = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
