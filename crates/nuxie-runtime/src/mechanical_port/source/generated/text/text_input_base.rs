use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, drawable::Drawable, text::text_input::TextInput,
};

pub trait TextInputBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn text_changed(&mut self) {}
    fn selection_radius_changed(&mut self) {}
    fn multiline_changed(&mut self) {}
}

pub struct TextInputBase {
    pub base: Drawable,
    text: String,
    selection_radius: f32,
    multiline: bool,
}

impl Default for TextInputBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            text: "".to_owned(),
            selection_radius: 5.0,
            multiline: true,
        }
    }
}

impl TextInputBase {
    pub const TYPE_KEY: u16 = 569;
    pub const TEXT_PROPERTY_KEY: u16 = 817;
    pub const SELECTION_RADIUS_PROPERTY_KEY: u16 = 818;
    pub const MULTILINE_PROPERTY_KEY: u16 = 979;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text(&mut self, value: String, callbacks: &mut impl TextInputBaseCallbacks) {
        if self.text == value {
            return;
        }
        self.text = value;
        callbacks.text_changed();
        callbacks.notify_property_changed(Self::TEXT_PROPERTY_KEY);
    }
    pub fn selection_radius(&self) -> f32 {
        self.selection_radius
    }
    pub fn set_selection_radius(
        &mut self,
        value: f32,
        callbacks: &mut impl TextInputBaseCallbacks,
    ) {
        if self.selection_radius == value {
            return;
        }
        self.selection_radius = value;
        callbacks.selection_radius_changed();
        callbacks.notify_property_changed(Self::SELECTION_RADIUS_PROPERTY_KEY);
    }
    pub fn multiline(&self) -> bool {
        self.multiline
    }
    pub fn set_multiline(&mut self, value: bool, callbacks: &mut impl TextInputBaseCallbacks) {
        if self.multiline == value {
            return;
        }
        self.multiline = value;
        callbacks.multiline_changed();
        callbacks.notify_property_changed(Self::MULTILINE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl TextInputBaseCallbacks) -> TextInput {
        let mut cloned = TextInput::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextInputBaseCallbacks) {
        self.text.clone_from(&object.text);
        self.selection_radius = object.selection_radius;
        self.multiline = object.multiline;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextInputBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TEXT_PROPERTY_KEY => {
                self.text = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            Self::SELECTION_RADIUS_PROPERTY_KEY => {
                self.selection_radius = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MULTILINE_PROPERTY_KEY => {
                self.multiline = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
