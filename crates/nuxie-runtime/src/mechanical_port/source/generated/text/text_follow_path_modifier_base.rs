use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, text::text_follow_path_modifier::TextFollowPathModifier,
    text::text_target_modifier::TextTargetModifier,
};

pub trait TextFollowPathModifierBaseCallbacks: crate::mechanical_port::source::generated::text::text_target_modifier_base::TextTargetModifierBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn radial_changed(&mut self) {}
    fn orient_changed(&mut self) {}
    fn start_changed(&mut self) {}
    fn end_changed(&mut self) {}
    fn strength_changed(&mut self) {}
    fn offset_changed(&mut self) {}
}

pub struct TextFollowPathModifierBase {
    pub base: TextTargetModifier,
    radial: bool,
    orient: bool,
    start: f32,
    end: f32,
    strength: f32,
    offset: f32,
}

impl Default for TextFollowPathModifierBase {
    fn default() -> Self {
        Self {
            base: TextTargetModifier::default(),
            radial: false,
            orient: true,
            start: 0.0,
            end: 1.0,
            strength: 1.0,
            offset: 0.0,
        }
    }
}

impl TextFollowPathModifierBase {
    pub const TYPE_KEY: u16 = 547;
    pub const RADIAL_PROPERTY_KEY: u16 = 779;
    pub const ORIENT_PROPERTY_KEY: u16 = 782;
    pub const START_PROPERTY_KEY: u16 = 783;
    pub const END_PROPERTY_KEY: u16 = 784;
    pub const STRENGTH_PROPERTY_KEY: u16 = 785;
    pub const OFFSET_PROPERTY_KEY: u16 = 786;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 546 | 160 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn radial(&self) -> bool {
        self.radial
    }
    pub fn set_radial(
        &mut self,
        value: bool,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) {
        if !self.set_radial_value(value) {
            return;
        }
        callbacks.radial_changed();
        callbacks.notify_property_changed(Self::RADIAL_PROPERTY_KEY);
    }

    pub(crate) fn set_radial_value(&mut self, value: bool) -> bool {
        if self.radial == value {
            return false;
        }
        self.radial = value;
        true
    }
    pub fn orient(&self) -> bool {
        self.orient
    }
    pub fn set_orient(
        &mut self,
        value: bool,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) {
        if !self.set_orient_value(value) {
            return;
        }
        callbacks.orient_changed();
        callbacks.notify_property_changed(Self::ORIENT_PROPERTY_KEY);
    }

    pub(crate) fn set_orient_value(&mut self, value: bool) -> bool {
        if self.orient == value {
            return false;
        }
        self.orient = value;
        true
    }
    pub fn start(&self) -> f32 {
        self.start
    }
    pub fn set_start(
        &mut self,
        value: f32,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) {
        if !self.set_start_value(value) {
            return;
        }
        callbacks.start_changed();
        callbacks.notify_property_changed(Self::START_PROPERTY_KEY);
    }

    pub(crate) fn set_start_value(&mut self, value: f32) -> bool {
        if self.start == value {
            return false;
        }
        self.start = value;
        true
    }
    pub fn end(&self) -> f32 {
        self.end
    }
    pub fn set_end(
        &mut self,
        value: f32,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) {
        if !self.set_end_value(value) {
            return;
        }
        callbacks.end_changed();
        callbacks.notify_property_changed(Self::END_PROPERTY_KEY);
    }

    pub(crate) fn set_end_value(&mut self, value: f32) -> bool {
        if self.end == value {
            return false;
        }
        self.end = value;
        true
    }
    pub fn strength(&self) -> f32 {
        self.strength
    }
    pub fn set_strength(
        &mut self,
        value: f32,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) {
        if !self.set_strength_value(value) {
            return;
        }
        callbacks.strength_changed();
        callbacks.notify_property_changed(Self::STRENGTH_PROPERTY_KEY);
    }

    pub(crate) fn set_strength_value(&mut self, value: f32) -> bool {
        if self.strength == value {
            return false;
        }
        self.strength = value;
        true
    }
    pub fn offset(&self) -> f32 {
        self.offset
    }
    pub fn set_offset(
        &mut self,
        value: f32,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) {
        if !self.set_offset_value(value) {
            return;
        }
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
    }

    pub(crate) fn set_offset_value(&mut self, value: f32) -> bool {
        if self.offset == value {
            return false;
        }
        self.offset = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) -> TextFollowPathModifier {
        let mut cloned = TextFollowPathModifier::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) {
        self.radial = object.radial;
        self.orient = object.orient;
        self.start = object.start;
        self.end = object.end;
        self.strength = object.strength;
        self.offset = object.offset;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextFollowPathModifierBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::RADIAL_PROPERTY_KEY => {
                self.radial = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::ORIENT_PROPERTY_KEY => {
                self.orient = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::START_PROPERTY_KEY => {
                self.start = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::END_PROPERTY_KEY => {
                self.end = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::STRENGTH_PROPERTY_KEY => {
                self.strength = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OFFSET_PROPERTY_KEY => {
                self.offset = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextFollowPathModifierBase {
    type Target = TextTargetModifier;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextFollowPathModifierBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
