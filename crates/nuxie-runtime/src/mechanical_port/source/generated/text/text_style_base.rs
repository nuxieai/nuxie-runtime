use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    text::text_style::TextStyle,
};

pub trait TextStyleBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn font_size_changed(&mut self) {}
    fn line_height_changed(&mut self) {}
    fn letter_spacing_changed(&mut self) {}
    fn font_asset_id_changed(&mut self) {}
}

pub struct TextStyleBase {
    pub base: ContainerComponent,
    font_size: f32,
    line_height: f32,
    letter_spacing: f32,
    font_asset_id: u32,
}

impl Default for TextStyleBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            font_size: 12.0,
            line_height: -1.0,
            letter_spacing: 0.0,
            font_asset_id: u32::MAX,
        }
    }
}

impl TextStyleBase {
    pub const TYPE_KEY: u16 = 573;
    pub const FONT_SIZE_PROPERTY_KEY: u16 = 274;
    pub const LINE_HEIGHT_PROPERTY_KEY: u16 = 370;
    pub const LETTER_SPACING_PROPERTY_KEY: u16 = 390;
    pub const FONT_ASSET_ID_PROPERTY_KEY: u16 = 279;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
    pub fn set_font_size(&mut self, value: f32, callbacks: &mut impl TextStyleBaseCallbacks) {
        if !self.set_font_size_value(value) {
            return;
        }
        callbacks.font_size_changed();
        callbacks.notify_property_changed(Self::FONT_SIZE_PROPERTY_KEY);
    }

    pub(crate) fn set_font_size_value(&mut self, value: f32) -> bool {
        if self.font_size == value {
            return false;
        }
        self.font_size = value;
        true
    }
    pub fn line_height(&self) -> f32 {
        self.line_height
    }
    pub fn set_line_height(&mut self, value: f32, callbacks: &mut impl TextStyleBaseCallbacks) {
        if !self.set_line_height_value(value) {
            return;
        }
        callbacks.line_height_changed();
        callbacks.notify_property_changed(Self::LINE_HEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_line_height_value(&mut self, value: f32) -> bool {
        if self.line_height == value {
            return false;
        }
        self.line_height = value;
        true
    }
    pub fn letter_spacing(&self) -> f32 {
        self.letter_spacing
    }
    pub fn set_letter_spacing(&mut self, value: f32, callbacks: &mut impl TextStyleBaseCallbacks) {
        if !self.set_letter_spacing_value(value) {
            return;
        }
        callbacks.letter_spacing_changed();
        callbacks.notify_property_changed(Self::LETTER_SPACING_PROPERTY_KEY);
    }

    pub(crate) fn set_letter_spacing_value(&mut self, value: f32) -> bool {
        if self.letter_spacing == value {
            return false;
        }
        self.letter_spacing = value;
        true
    }
    pub fn font_asset_id(&self) -> u32 {
        self.font_asset_id
    }
    pub fn set_font_asset_id(&mut self, value: u32, callbacks: &mut impl TextStyleBaseCallbacks) {
        if !self.set_font_asset_id_value(value) {
            return;
        }
        callbacks.font_asset_id_changed();
        callbacks.notify_property_changed(Self::FONT_ASSET_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_font_asset_id_value(&mut self, value: u32) -> bool {
        if self.font_asset_id == value {
            return false;
        }
        self.font_asset_id = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl TextStyleBaseCallbacks) -> TextStyle {
        let mut cloned = TextStyle::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextStyleBaseCallbacks) {
        self.font_size = object.font_size;
        self.line_height = object.line_height;
        self.letter_spacing = object.letter_spacing;
        self.font_asset_id = object.font_asset_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextStyleBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FONT_SIZE_PROPERTY_KEY => {
                self.font_size = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::LINE_HEIGHT_PROPERTY_KEY => {
                self.line_height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::LETTER_SPACING_PROPERTY_KEY => {
                self.letter_spacing = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::FONT_ASSET_ID_PROPERTY_KEY => {
                self.font_asset_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextStyleBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
