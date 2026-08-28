use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, drawable::Drawable, layout_component::LayoutComponent,
};

pub trait LayoutComponentBaseCallbacks:
    crate::mechanical_port::source::generated::drawable_base::DrawableBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn clip_changed(&mut self) {}
    fn width_changed(&mut self) {}
    fn height_changed(&mut self) {}
    fn style_id_changed(&mut self) {}
    fn fractional_width_changed(&mut self) {}
    fn fractional_height_changed(&mut self) {}
}

pub struct LayoutComponentBase {
    pub base: Drawable,
    clip: bool,
    width: f32,
    height: f32,
    style_id: u32,
    fractional_width: f32,
    fractional_height: f32,
}

impl Default for LayoutComponentBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            clip: false,
            width: 0.0,
            height: 0.0,
            style_id: u32::MAX,
            fractional_width: 1.0,
            fractional_height: 1.0,
        }
    }
}

impl LayoutComponentBase {
    pub const TYPE_KEY: u16 = 409;
    pub const CLIP_PROPERTY_KEY: u16 = 196;
    pub const WIDTH_PROPERTY_KEY: u16 = 7;
    pub const HEIGHT_PROPERTY_KEY: u16 = 8;
    pub const STYLE_ID_PROPERTY_KEY: u16 = 494;
    pub const FRACTIONAL_WIDTH_PROPERTY_KEY: u16 = 706;
    pub const FRACTIONAL_HEIGHT_PROPERTY_KEY: u16 = 707;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clip(&self) -> bool {
        self.clip
    }
    pub fn set_clip(&mut self, value: bool, callbacks: &mut impl LayoutComponentBaseCallbacks) {
        if !self.set_clip_value(value) {
            return;
        }
        callbacks.clip_changed();
        callbacks.notify_property_changed(Self::CLIP_PROPERTY_KEY);
    }

    pub(crate) fn set_clip_value(&mut self, value: bool) -> bool {
        if self.clip == value {
            return false;
        }
        self.clip = value;
        true
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn set_width(&mut self, value: f32, callbacks: &mut impl LayoutComponentBaseCallbacks) {
        if !self.set_width_value(value) {
            return;
        }
        callbacks.width_changed();
        callbacks.notify_property_changed(Self::WIDTH_PROPERTY_KEY);
    }

    pub(crate) fn set_width_value(&mut self, value: f32) -> bool {
        if self.width == value {
            return false;
        }
        self.width = value;
        true
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn set_height(&mut self, value: f32, callbacks: &mut impl LayoutComponentBaseCallbacks) {
        if !self.set_height_value(value) {
            return;
        }
        callbacks.height_changed();
        callbacks.notify_property_changed(Self::HEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_height_value(&mut self, value: f32) -> bool {
        if self.height == value {
            return false;
        }
        self.height = value;
        true
    }
    pub fn style_id(&self) -> u32 {
        self.style_id
    }
    pub fn set_style_id(&mut self, value: u32, callbacks: &mut impl LayoutComponentBaseCallbacks) {
        if !self.set_style_id_value(value) {
            return;
        }
        callbacks.style_id_changed();
        callbacks.notify_property_changed(Self::STYLE_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_style_id_value(&mut self, value: u32) -> bool {
        if self.style_id == value {
            return false;
        }
        self.style_id = value;
        true
    }
    pub fn fractional_width(&self) -> f32 {
        self.fractional_width
    }
    pub fn set_fractional_width(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentBaseCallbacks,
    ) {
        if !self.set_fractional_width_value(value) {
            return;
        }
        callbacks.fractional_width_changed();
        callbacks.notify_property_changed(Self::FRACTIONAL_WIDTH_PROPERTY_KEY);
    }

    pub(crate) fn set_fractional_width_value(&mut self, value: f32) -> bool {
        if self.fractional_width == value {
            return false;
        }
        self.fractional_width = value;
        true
    }
    pub fn fractional_height(&self) -> f32 {
        self.fractional_height
    }
    pub fn set_fractional_height(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentBaseCallbacks,
    ) {
        if !self.set_fractional_height_value(value) {
            return;
        }
        callbacks.fractional_height_changed();
        callbacks.notify_property_changed(Self::FRACTIONAL_HEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_fractional_height_value(&mut self, value: f32) -> bool {
        if self.fractional_height == value {
            return false;
        }
        self.fractional_height = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl LayoutComponentBaseCallbacks) -> LayoutComponent {
        let mut cloned = LayoutComponent::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LayoutComponentBaseCallbacks) {
        self.clip = object.clip;
        self.width = object.width;
        self.height = object.height;
        self.style_id = object.style_id;
        self.fractional_width = object.fractional_width;
        self.fractional_height = object.fractional_height;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LayoutComponentBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::CLIP_PROPERTY_KEY => {
                self.clip = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::WIDTH_PROPERTY_KEY => {
                self.width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::HEIGHT_PROPERTY_KEY => {
                self.height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::STYLE_ID_PROPERTY_KEY => {
                self.style_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::FRACTIONAL_WIDTH_PROPERTY_KEY => {
                self.fractional_width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::FRACTIONAL_HEIGHT_PROPERTY_KEY => {
                self.fractional_height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for LayoutComponentBase {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for LayoutComponentBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
