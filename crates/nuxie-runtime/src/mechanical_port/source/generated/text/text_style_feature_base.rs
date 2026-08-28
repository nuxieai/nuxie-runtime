use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader,
    text::text_style_feature::TextStyleFeature,
};

pub trait TextStyleFeatureBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn tag_changed(&mut self) {}
    fn feature_value_changed(&mut self) {}
}

pub struct TextStyleFeatureBase {
    pub base: Component,
    tag: u32,
    feature_value: u32,
}

impl Default for TextStyleFeatureBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            tag: 0,
            feature_value: 1,
        }
    }
}

impl TextStyleFeatureBase {
    pub const TYPE_KEY: u16 = 164;
    pub const TAG_PROPERTY_KEY: u16 = 356;
    pub const FEATURE_VALUE_PROPERTY_KEY: u16 = 357;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn tag(&self) -> u32 {
        self.tag
    }
    pub fn set_tag(&mut self, value: u32, callbacks: &mut impl TextStyleFeatureBaseCallbacks) {
        if !self.set_tag_value(value) {
            return;
        }
        callbacks.tag_changed();
        callbacks.notify_property_changed(Self::TAG_PROPERTY_KEY);
    }

    pub(crate) fn set_tag_value(&mut self, value: u32) -> bool {
        if self.tag == value {
            return false;
        }
        self.tag = value;
        true
    }
    pub fn feature_value(&self) -> u32 {
        self.feature_value
    }
    pub fn set_feature_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TextStyleFeatureBaseCallbacks,
    ) {
        if !self.set_feature_value_value(value) {
            return;
        }
        callbacks.feature_value_changed();
        callbacks.notify_property_changed(Self::FEATURE_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_feature_value_value(&mut self, value: u32) -> bool {
        if self.feature_value == value {
            return false;
        }
        self.feature_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TextStyleFeatureBaseCallbacks,
    ) -> TextStyleFeature {
        let mut cloned = TextStyleFeature::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextStyleFeatureBaseCallbacks) {
        self.tag = object.tag;
        self.feature_value = object.feature_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextStyleFeatureBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TAG_PROPERTY_KEY => {
                self.tag = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::FEATURE_VALUE_PROPERTY_KEY => {
                self.feature_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextStyleFeatureBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleFeatureBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
