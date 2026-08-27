use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, text::text_shape_modifier::TextShapeModifier,
    text::text_variation_modifier::TextVariationModifier,
};

pub trait TextVariationModifierBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn axis_tag_changed(&mut self) {}
    fn axis_value_changed(&mut self) {}
}

pub struct TextVariationModifierBase {
    pub base: TextShapeModifier,
    axis_tag: u32,
    axis_value: f32,
}

impl Default for TextVariationModifierBase {
    fn default() -> Self {
        Self {
            base: TextShapeModifier::default(),
            axis_tag: 0,
            axis_value: 0.0,
        }
    }
}

impl TextVariationModifierBase {
    pub const TYPE_KEY: u16 = 162;
    pub const AXIS_TAG_PROPERTY_KEY: u16 = 320;
    pub const AXIS_VALUE_PROPERTY_KEY: u16 = 321;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 161 | 160 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn axis_tag(&self) -> u32 {
        self.axis_tag
    }
    pub fn set_axis_tag(
        &mut self,
        value: u32,
        callbacks: &mut impl TextVariationModifierBaseCallbacks,
    ) {
        if self.axis_tag == value {
            return;
        }
        self.axis_tag = value;
        callbacks.axis_tag_changed();
        callbacks.notify_property_changed(Self::AXIS_TAG_PROPERTY_KEY);
    }
    pub fn axis_value(&self) -> f32 {
        self.axis_value
    }
    pub fn set_axis_value(
        &mut self,
        value: f32,
        callbacks: &mut impl TextVariationModifierBaseCallbacks,
    ) {
        if self.axis_value == value {
            return;
        }
        self.axis_value = value;
        callbacks.axis_value_changed();
        callbacks.notify_property_changed(Self::AXIS_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TextVariationModifierBaseCallbacks,
    ) -> TextVariationModifier {
        let mut cloned = TextVariationModifier::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextVariationModifierBaseCallbacks) {
        self.axis_tag = object.axis_tag;
        self.axis_value = object.axis_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextVariationModifierBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::AXIS_TAG_PROPERTY_KEY => {
                self.axis_tag = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::AXIS_VALUE_PROPERTY_KEY => {
                self.axis_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
