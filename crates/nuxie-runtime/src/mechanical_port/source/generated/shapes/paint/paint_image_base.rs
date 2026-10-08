//! Mechanical translation of the pinned upstream generated PaintImageBase.
use crate::mechanical_port::source::core::field_types::{
    core_double_type::CoreDoubleType, core_uint_type::CoreUintType,
};
use crate::mechanical_port::source::{
    component::Component,
    core::{binary_reader::BinaryReader, field_types::core_id_type::CoreIdType},
    shapes::paint::paint_image::PaintImage,
};

pub trait PaintImageBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct PaintImageBase {
    pub base: Component,
    image_asset_id: u32,
    image_sampler_filter: u8,
    image_sampler_wrap_x: u8,
    image_sampler_wrap_y: u8,
    image_scale_x: f32,
    image_scale_y: f32,
    image_offset_x: f32,
    image_offset_y: f32,
    image_rotation: f32,
    image_size_mode: u8,
}
impl Default for PaintImageBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            image_asset_id: u32::MAX,
            image_sampler_filter: 0,
            image_sampler_wrap_x: 0,
            image_sampler_wrap_y: 0,
            image_scale_x: 1.0,
            image_scale_y: 1.0,
            image_offset_x: 0.0,
            image_offset_y: 0.0,
            image_rotation: 0.0,
            image_size_mode: 0,
        }
    }
}
impl PaintImageBase {
    pub const TYPE_KEY: u16 = 113;
    pub const IMAGE_ASSET_ID_PROPERTY_KEY: u16 = 415;
    pub const IMAGE_SAMPLER_FILTER_PROPERTY_KEY: u16 = 269;
    pub const IMAGE_SAMPLER_WRAP_X_PROPERTY_KEY: u16 = 270;
    pub const IMAGE_SAMPLER_WRAP_Y_PROPERTY_KEY: u16 = 271;
    pub const IMAGE_SCALE_X_PROPERTY_KEY: u16 = 416;
    pub const IMAGE_SCALE_Y_PROPERTY_KEY: u16 = 368;
    pub const IMAGE_OFFSET_X_PROPERTY_KEY: u16 = 369;
    pub const IMAGE_OFFSET_Y_PROPERTY_KEY: u16 = 410;
    pub const IMAGE_ROTATION_PROPERTY_KEY: u16 = 411;
    pub const IMAGE_SIZE_MODE_PROPERTY_KEY: u16 = 412;
    pub fn is_type_of(type_key: u16) -> bool {
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn image_asset_id(&self) -> u32 {
        self.image_asset_id
    }
    pub fn set_image_asset_id<C: PaintImageBaseCallbacks>(
        &mut self,
        value: u32,
        callbacks: &mut C,
    ) {
        if !self.set_image_asset_id_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_ASSET_ID_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_asset_id_value(&mut self, value: u32) -> bool {
        if self.image_asset_id == value {
            return false;
        }
        self.image_asset_id = value;
        true
    }
    pub fn image_sampler_filter(&self) -> u8 {
        self.image_sampler_filter
    }
    pub fn set_image_sampler_filter<C: PaintImageBaseCallbacks>(
        &mut self,
        value: u8,
        callbacks: &mut C,
    ) {
        if !self.set_image_sampler_filter_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_SAMPLER_FILTER_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_sampler_filter_value(&mut self, value: u8) -> bool {
        if self.image_sampler_filter == value {
            return false;
        }
        self.image_sampler_filter = value;
        true
    }
    pub fn image_sampler_wrap_x(&self) -> u8 {
        self.image_sampler_wrap_x
    }
    pub fn set_image_sampler_wrap_x<C: PaintImageBaseCallbacks>(
        &mut self,
        value: u8,
        callbacks: &mut C,
    ) {
        if !self.set_image_sampler_wrap_x_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_SAMPLER_WRAP_X_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_sampler_wrap_x_value(&mut self, value: u8) -> bool {
        if self.image_sampler_wrap_x == value {
            return false;
        }
        self.image_sampler_wrap_x = value;
        true
    }
    pub fn image_sampler_wrap_y(&self) -> u8 {
        self.image_sampler_wrap_y
    }
    pub fn set_image_sampler_wrap_y<C: PaintImageBaseCallbacks>(
        &mut self,
        value: u8,
        callbacks: &mut C,
    ) {
        if !self.set_image_sampler_wrap_y_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_SAMPLER_WRAP_Y_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_sampler_wrap_y_value(&mut self, value: u8) -> bool {
        if self.image_sampler_wrap_y == value {
            return false;
        }
        self.image_sampler_wrap_y = value;
        true
    }
    pub fn image_scale_x(&self) -> f32 {
        self.image_scale_x
    }
    pub fn set_image_scale_x<C: PaintImageBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if !self.set_image_scale_x_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_SCALE_X_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_scale_x_value(&mut self, value: f32) -> bool {
        if self.image_scale_x == value {
            return false;
        }
        self.image_scale_x = value;
        true
    }
    pub fn image_scale_y(&self) -> f32 {
        self.image_scale_y
    }
    pub fn set_image_scale_y<C: PaintImageBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if !self.set_image_scale_y_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_SCALE_Y_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_scale_y_value(&mut self, value: f32) -> bool {
        if self.image_scale_y == value {
            return false;
        }
        self.image_scale_y = value;
        true
    }
    pub fn image_offset_x(&self) -> f32 {
        self.image_offset_x
    }
    pub fn set_image_offset_x<C: PaintImageBaseCallbacks>(
        &mut self,
        value: f32,
        callbacks: &mut C,
    ) {
        if !self.set_image_offset_x_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_OFFSET_X_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_offset_x_value(&mut self, value: f32) -> bool {
        if self.image_offset_x == value {
            return false;
        }
        self.image_offset_x = value;
        true
    }
    pub fn image_offset_y(&self) -> f32 {
        self.image_offset_y
    }
    pub fn set_image_offset_y<C: PaintImageBaseCallbacks>(
        &mut self,
        value: f32,
        callbacks: &mut C,
    ) {
        if !self.set_image_offset_y_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_OFFSET_Y_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_offset_y_value(&mut self, value: f32) -> bool {
        if self.image_offset_y == value {
            return false;
        }
        self.image_offset_y = value;
        true
    }
    pub fn image_rotation(&self) -> f32 {
        self.image_rotation
    }
    pub fn set_image_rotation<C: PaintImageBaseCallbacks>(
        &mut self,
        value: f32,
        callbacks: &mut C,
    ) {
        if !self.set_image_rotation_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_ROTATION_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_rotation_value(&mut self, value: f32) -> bool {
        if self.image_rotation == value {
            return false;
        }
        self.image_rotation = value;
        true
    }
    pub fn image_size_mode(&self) -> u8 {
        self.image_size_mode
    }
    pub fn set_image_size_mode<C: PaintImageBaseCallbacks>(
        &mut self,
        value: u8,
        callbacks: &mut C,
    ) {
        if !self.set_image_size_mode_value(value) {
            return;
        }
        PaintImageBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IMAGE_SIZE_MODE_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_image_size_mode_value(&mut self, value: u8) -> bool {
        if self.image_size_mode == value {
            return false;
        }
        self.image_size_mode = value;
        true
    }
    pub fn clone_into<C: PaintImageBaseCallbacks>(&self, callbacks: &mut C) -> PaintImage {
        let mut cloned = PaintImage::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy<C: PaintImageBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.image_asset_id = object.image_asset_id;
        self.image_sampler_filter = object.image_sampler_filter;
        self.image_sampler_wrap_x = object.image_sampler_wrap_x;
        self.image_sampler_wrap_y = object.image_sampler_wrap_y;
        self.image_scale_x = object.image_scale_x;
        self.image_scale_y = object.image_scale_y;
        self.image_offset_x = object.image_offset_x;
        self.image_offset_y = object.image_offset_y;
        self.image_rotation = object.image_rotation;
        self.image_size_mode = object.image_size_mode;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize<C: PaintImageBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::IMAGE_ASSET_ID_PROPERTY_KEY => {
                self.image_asset_id = CoreIdType::runtime_deserialize(reader);
                true
            }
            Self::IMAGE_SAMPLER_FILTER_PROPERTY_KEY => {
                self.image_sampler_filter = CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::IMAGE_SAMPLER_WRAP_X_PROPERTY_KEY => {
                self.image_sampler_wrap_x = CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::IMAGE_SAMPLER_WRAP_Y_PROPERTY_KEY => {
                self.image_sampler_wrap_y = CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::IMAGE_SCALE_X_PROPERTY_KEY => {
                self.image_scale_x = CoreDoubleType::deserialize(reader);
                true
            }
            Self::IMAGE_SCALE_Y_PROPERTY_KEY => {
                self.image_scale_y = CoreDoubleType::deserialize(reader);
                true
            }
            Self::IMAGE_OFFSET_X_PROPERTY_KEY => {
                self.image_offset_x = CoreDoubleType::deserialize(reader);
                true
            }
            Self::IMAGE_OFFSET_Y_PROPERTY_KEY => {
                self.image_offset_y = CoreDoubleType::deserialize(reader);
                true
            }
            Self::IMAGE_ROTATION_PROPERTY_KEY => {
                self.image_rotation = CoreDoubleType::deserialize(reader);
                true
            }
            Self::IMAGE_SIZE_MODE_PROPERTY_KEY => {
                self.image_size_mode = CoreUintType::deserialize(reader) as u8;
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
impl std::ops::Deref for PaintImageBase {
    type Target = Component;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for PaintImageBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
