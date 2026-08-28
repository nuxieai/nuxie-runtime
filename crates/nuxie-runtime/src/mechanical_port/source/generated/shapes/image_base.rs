use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, drawable::Drawable, shapes::image::Image,
};

pub trait ImageBaseCallbacks:
    crate::mechanical_port::source::generated::drawable_base::DrawableBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn asset_id_changed(&mut self) {}
    fn origin_x_changed(&mut self) {}
    fn origin_y_changed(&mut self) {}
    fn fit_changed(&mut self) {}
    fn alignment_x_changed(&mut self) {}
    fn alignment_y_changed(&mut self) {}
}

pub struct ImageBase {
    pub base: Drawable,
    asset_id: u32,
    origin_x: f32,
    origin_y: f32,
    fit: u32,
    alignment_x: f32,
    alignment_y: f32,
}

impl Default for ImageBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            asset_id: u32::MAX,
            origin_x: 0.5,
            origin_y: 0.5,
            fit: 0,
            alignment_x: 0.0,
            alignment_y: 0.0,
        }
    }
}

impl ImageBase {
    pub const TYPE_KEY: u16 = 100;
    pub const ASSET_ID_PROPERTY_KEY: u16 = 206;
    pub const ORIGIN_X_PROPERTY_KEY: u16 = 380;
    pub const ORIGIN_Y_PROPERTY_KEY: u16 = 381;
    pub const FIT_PROPERTY_KEY: u16 = 974;
    pub const ALIGNMENT_X_PROPERTY_KEY: u16 = 975;
    pub const ALIGNMENT_Y_PROPERTY_KEY: u16 = 976;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn asset_id(&self) -> u32 {
        self.asset_id
    }
    pub fn set_asset_id(&mut self, value: u32, callbacks: &mut impl ImageBaseCallbacks) {
        if !self.set_asset_id_value(value) {
            return;
        }
        callbacks.asset_id_changed();
        callbacks.notify_property_changed(Self::ASSET_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_asset_id_value(&mut self, value: u32) -> bool {
        if self.asset_id == value {
            return false;
        }
        self.asset_id = value;
        true
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn set_origin_x(&mut self, value: f32, callbacks: &mut impl ImageBaseCallbacks) {
        if !self.set_origin_x_value(value) {
            return;
        }
        callbacks.origin_x_changed();
        callbacks.notify_property_changed(Self::ORIGIN_X_PROPERTY_KEY);
    }

    pub(crate) fn set_origin_x_value(&mut self, value: f32) -> bool {
        if self.origin_x == value {
            return false;
        }
        self.origin_x = value;
        true
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn set_origin_y(&mut self, value: f32, callbacks: &mut impl ImageBaseCallbacks) {
        if !self.set_origin_y_value(value) {
            return;
        }
        callbacks.origin_y_changed();
        callbacks.notify_property_changed(Self::ORIGIN_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_origin_y_value(&mut self, value: f32) -> bool {
        if self.origin_y == value {
            return false;
        }
        self.origin_y = value;
        true
    }
    pub fn fit(&self) -> u32 {
        self.fit
    }
    pub fn set_fit(&mut self, value: u32, callbacks: &mut impl ImageBaseCallbacks) {
        if !self.set_fit_value(value) {
            return;
        }
        callbacks.fit_changed();
        callbacks.notify_property_changed(Self::FIT_PROPERTY_KEY);
    }

    pub(crate) fn set_fit_value(&mut self, value: u32) -> bool {
        if self.fit == value {
            return false;
        }
        self.fit = value;
        true
    }
    pub fn alignment_x(&self) -> f32 {
        self.alignment_x
    }
    pub fn set_alignment_x(&mut self, value: f32, callbacks: &mut impl ImageBaseCallbacks) {
        if !self.set_alignment_x_value(value) {
            return;
        }
        callbacks.alignment_x_changed();
        callbacks.notify_property_changed(Self::ALIGNMENT_X_PROPERTY_KEY);
    }

    pub(crate) fn set_alignment_x_value(&mut self, value: f32) -> bool {
        if self.alignment_x == value {
            return false;
        }
        self.alignment_x = value;
        true
    }
    pub fn alignment_y(&self) -> f32 {
        self.alignment_y
    }
    pub fn set_alignment_y(&mut self, value: f32, callbacks: &mut impl ImageBaseCallbacks) {
        if !self.set_alignment_y_value(value) {
            return;
        }
        callbacks.alignment_y_changed();
        callbacks.notify_property_changed(Self::ALIGNMENT_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_alignment_y_value(&mut self, value: f32) -> bool {
        if self.alignment_y == value {
            return false;
        }
        self.alignment_y = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl ImageBaseCallbacks) -> Image {
        let mut cloned = Image::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ImageBaseCallbacks) {
        self.asset_id = object.asset_id;
        self.origin_x = object.origin_x;
        self.origin_y = object.origin_y;
        self.fit = object.fit;
        self.alignment_x = object.alignment_x;
        self.alignment_y = object.alignment_y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ImageBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ASSET_ID_PROPERTY_KEY => {
                self.asset_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::ORIGIN_X_PROPERTY_KEY => {
                self.origin_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_Y_PROPERTY_KEY => {
                self.origin_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::FIT_PROPERTY_KEY => {
                self.fit = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::ALIGNMENT_X_PROPERTY_KEY => {
                self.alignment_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ALIGNMENT_Y_PROPERTY_KEY => {
                self.alignment_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ImageBase {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ImageBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
