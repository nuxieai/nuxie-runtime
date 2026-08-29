use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, text::text_value_run::TextValueRun,
};

pub trait TextValueRunBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn style_id_changed(&mut self) {}
    fn text_changed(&mut self) {}
}

pub struct TextValueRunBase {
    pub base: Component,
    style_id: u32,
    text: String,
}

impl Default for TextValueRunBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            style_id: u32::MAX,
            text: "".to_owned(),
        }
    }
}

impl TextValueRunBase {
    pub const TYPE_KEY: u16 = 135;
    pub const STYLE_ID_PROPERTY_KEY: u16 = 272;
    pub const TEXT_PROPERTY_KEY: u16 = 268;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn style_id(&self) -> u32 {
        self.style_id
    }
    pub fn set_style_id(&mut self, value: u32, callbacks: &mut impl TextValueRunBaseCallbacks) {
        if !self.set_style_id_value(value) {
            return;
        }
        callbacks.style_id_changed();
        TextValueRunBaseCallbacks::notify_property_changed(callbacks, Self::STYLE_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_style_id_value(&mut self, value: u32) -> bool {
        if self.style_id == value {
            return false;
        }
        self.style_id = value;
        true
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text(&mut self, value: String, callbacks: &mut impl TextValueRunBaseCallbacks) {
        if !self.set_text_value(value) {
            return;
        }
        callbacks.text_changed();
        TextValueRunBaseCallbacks::notify_property_changed(callbacks, Self::TEXT_PROPERTY_KEY);
    }

    pub(crate) fn set_text_value(&mut self, value: String) -> bool {
        if self.text == value {
            return false;
        }
        self.text = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl TextValueRunBaseCallbacks) -> TextValueRun {
        let mut cloned = TextValueRun::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextValueRunBaseCallbacks) {
        self.style_id = object.style_id;
        self.text.clone_from(&object.text);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextValueRunBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::STYLE_ID_PROPERTY_KEY => {
                self.style_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::TEXT_PROPERTY_KEY => {
                self.text = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextValueRunBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextValueRunBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
