use crate::mechanical_port::source::{
    animation::blend_animation::BlendAnimation,
    animation::blend_animation_direct::BlendAnimationDirect, core::binary_reader::BinaryReader,
};

pub trait BlendAnimationDirectBaseCallbacks: crate::mechanical_port::source::generated::animation::blend_animation_base::BlendAnimationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn input_id_changed(&mut self) {}
    fn mix_value_changed(&mut self) {}
    fn blend_source_changed(&mut self) {}
}

pub struct BlendAnimationDirectBase {
    pub base: BlendAnimation,
    input_id: u32,
    mix_value: f32,
    blend_source: u32,
}

impl Default for BlendAnimationDirectBase {
    fn default() -> Self {
        Self {
            base: BlendAnimation::default(),
            input_id: u32::MAX,
            mix_value: 100.0,
            blend_source: 0,
        }
    }
}

impl BlendAnimationDirectBase {
    pub const TYPE_KEY: u16 = 77;
    pub const INPUT_ID_PROPERTY_KEY: u16 = 168;
    pub const MIX_VALUE_PROPERTY_KEY: u16 = 297;
    pub const BLEND_SOURCE_PROPERTY_KEY: u16 = 298;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 74)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn input_id(&self) -> u32 {
        self.input_id
    }
    pub fn set_input_id(
        &mut self,
        value: u32,
        callbacks: &mut impl BlendAnimationDirectBaseCallbacks,
    ) {
        if !self.set_input_id_value(value) {
            return;
        }
        callbacks.input_id_changed();
        BlendAnimationDirectBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INPUT_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_input_id_value(&mut self, value: u32) -> bool {
        if self.input_id == value {
            return false;
        }
        self.input_id = value;
        true
    }
    pub fn mix_value(&self) -> f32 {
        self.mix_value
    }
    pub fn set_mix_value(
        &mut self,
        value: f32,
        callbacks: &mut impl BlendAnimationDirectBaseCallbacks,
    ) {
        if !self.set_mix_value_value(value) {
            return;
        }
        callbacks.mix_value_changed();
        BlendAnimationDirectBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MIX_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_mix_value_value(&mut self, value: f32) -> bool {
        if self.mix_value == value {
            return false;
        }
        self.mix_value = value;
        true
    }
    pub fn blend_source(&self) -> u32 {
        self.blend_source
    }
    pub fn set_blend_source(
        &mut self,
        value: u32,
        callbacks: &mut impl BlendAnimationDirectBaseCallbacks,
    ) {
        if !self.set_blend_source_value(value) {
            return;
        }
        callbacks.blend_source_changed();
        BlendAnimationDirectBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BLEND_SOURCE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_blend_source_value(&mut self, value: u32) -> bool {
        if self.blend_source == value {
            return false;
        }
        self.blend_source = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl BlendAnimationDirectBaseCallbacks,
    ) -> BlendAnimationDirect {
        let mut cloned = BlendAnimationDirect::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl BlendAnimationDirectBaseCallbacks) {
        self.input_id = object.input_id;
        self.mix_value = object.mix_value;
        self.blend_source = object.blend_source;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BlendAnimationDirectBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INPUT_ID_PROPERTY_KEY => {
                self.input_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::MIX_VALUE_PROPERTY_KEY => {
                self.mix_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BLEND_SOURCE_PROPERTY_KEY => {
                self.blend_source = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for BlendAnimationDirectBase {
    type Target = BlendAnimation;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendAnimationDirectBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
