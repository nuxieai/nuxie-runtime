use crate::mechanical_port::source::{core::Core, core::binary_reader::BinaryReader};

pub trait BlendAnimationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn animation_id_changed(&mut self) {}
}

pub struct BlendAnimationBase {
    pub base: Core,
    animation_id: u32,
}

impl Default for BlendAnimationBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            animation_id: u32::MAX,
        }
    }
}

impl BlendAnimationBase {
    pub const TYPE_KEY: u16 = 74;
    pub const ANIMATION_ID_PROPERTY_KEY: u16 = 165;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn animation_id(&self) -> u32 {
        self.animation_id
    }
    pub fn set_animation_id(
        &mut self,
        value: u32,
        callbacks: &mut impl BlendAnimationBaseCallbacks,
    ) {
        if !self.set_animation_id_value(value) {
            return;
        }
        callbacks.animation_id_changed();
        callbacks.notify_property_changed(Self::ANIMATION_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_animation_id_value(&mut self, value: u32) -> bool {
        if self.animation_id == value {
            return false;
        }
        self.animation_id = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl BlendAnimationBaseCallbacks) {
        self.animation_id = object.animation_id;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BlendAnimationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ANIMATION_ID_PROPERTY_KEY => {
                self.animation_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for BlendAnimationBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendAnimationBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
