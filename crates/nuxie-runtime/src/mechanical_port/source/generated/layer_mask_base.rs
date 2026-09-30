//! Translation of include/rive/generated/layer_mask_base.hpp and src/generated/layer_mask_base.cpp at 8398db3199cea4cd3eba53747aac562b5c0df3da.

use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, layer_mask::LayerMask,
};

pub trait LayerMaskBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn source_id_changed(&mut self) {}
    fn mask_flags_changed(&mut self) {}
    fn resolution_changed(&mut self) {}
    fn bounds_x_changed(&mut self) {}
    fn bounds_y_changed(&mut self) {}
    fn bounds_width_changed(&mut self) {}
    fn bounds_height_changed(&mut self) {}
}

pub struct LayerMaskBase {
    pub base: Component,
    source_id: u32,
    mask_flags: u32,
    resolution: f32,
    bounds_x: f32,
    bounds_y: f32,
    bounds_width: f32,
    bounds_height: f32,
}
impl Default for LayerMaskBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            source_id: u32::MAX,
            mask_flags: 4,
            resolution: 1.0,
            bounds_x: 0.0,
            bounds_y: 0.0,
            bounds_width: 0.0,
            bounds_height: 0.0,
        }
    }
}
impl LayerMaskBase {
    pub const TYPE_KEY: u16 = 154;
    pub const SOURCE_ID_PROPERTY_KEY: u16 = 459;
    pub const MASK_FLAGS_PROPERTY_KEY: u16 = 460;
    pub const RESOLUTION_PROPERTY_KEY: u16 = 458;
    pub const BOUNDS_X_PROPERTY_KEY: u16 = 462;
    pub const BOUNDS_Y_PROPERTY_KEY: u16 = 463;
    pub const BOUNDS_WIDTH_PROPERTY_KEY: u16 = 464;
    pub const BOUNDS_HEIGHT_PROPERTY_KEY: u16 = 465;
    pub const MASK_MODE_VALUE_PROPERTY_KEY: u16 = 455;
    pub const MASK_MODE_VALUE_BIT_OFFSET: u32 = 0;
    pub const MASK_MODE_VALUE_FIELD_MASK: u32 = 3;
    pub const IS_VISIBLE_PROPERTY_KEY: u16 = 456;
    pub const IS_VISIBLE_BITMASK: u32 = 1 << 2;
    pub const SOURCE_DRAWS_PROPERTY_KEY: u16 = 457;
    pub const SOURCE_DRAWS_BITMASK: u32 = 1 << 3;
    pub const USE_CUSTOM_BOUNDS_PROPERTY_KEY: u16 = 461;
    pub const USE_CUSTOM_BOUNDS_BITMASK: u32 = 1 << 4;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn source_id(&self) -> u32 {
        self.source_id
    }
    pub fn set_source_id(&mut self, value: u32, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_source_id_value(value) {
            callbacks.source_id_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::SOURCE_ID_PROPERTY_KEY,
            );
        }
    }
    pub(crate) fn set_source_id_value(&mut self, value: u32) -> bool {
        if self.source_id == value {
            return false;
        }
        self.source_id = value;
        true
    }
    pub fn mask_flags(&self) -> u32 {
        self.mask_flags
    }
    pub fn set_mask_flags(&mut self, value: u32, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_mask_flags_value(value) {
            callbacks.mask_flags_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::MASK_FLAGS_PROPERTY_KEY,
            );
        }
    }
    pub(crate) fn set_mask_flags_value(&mut self, value: u32) -> bool {
        if self.mask_flags == value {
            return false;
        }
        self.mask_flags = value;
        true
    }
    pub fn resolution(&self) -> f32 {
        self.resolution
    }
    pub fn set_resolution(&mut self, value: f32, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_resolution_value(value) {
            callbacks.resolution_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::RESOLUTION_PROPERTY_KEY,
            );
        }
    }
    pub(crate) fn set_resolution_value(&mut self, value: f32) -> bool {
        if self.resolution == value {
            return false;
        }
        self.resolution = value;
        true
    }
    pub fn bounds_x(&self) -> f32 {
        self.bounds_x
    }
    pub fn set_bounds_x(&mut self, value: f32, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_bounds_x_value(value) {
            callbacks.bounds_x_changed();
            LayerMaskBaseCallbacks::notify_property_changed(callbacks, Self::BOUNDS_X_PROPERTY_KEY);
        }
    }
    pub(crate) fn set_bounds_x_value(&mut self, value: f32) -> bool {
        if self.bounds_x == value {
            return false;
        }
        self.bounds_x = value;
        true
    }
    pub fn bounds_y(&self) -> f32 {
        self.bounds_y
    }
    pub fn set_bounds_y(&mut self, value: f32, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_bounds_y_value(value) {
            callbacks.bounds_y_changed();
            LayerMaskBaseCallbacks::notify_property_changed(callbacks, Self::BOUNDS_Y_PROPERTY_KEY);
        }
    }
    pub(crate) fn set_bounds_y_value(&mut self, value: f32) -> bool {
        if self.bounds_y == value {
            return false;
        }
        self.bounds_y = value;
        true
    }
    pub fn bounds_width(&self) -> f32 {
        self.bounds_width
    }
    pub fn set_bounds_width(&mut self, value: f32, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_bounds_width_value(value) {
            callbacks.bounds_width_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::BOUNDS_WIDTH_PROPERTY_KEY,
            );
        }
    }
    pub(crate) fn set_bounds_width_value(&mut self, value: f32) -> bool {
        if self.bounds_width == value {
            return false;
        }
        self.bounds_width = value;
        true
    }
    pub fn bounds_height(&self) -> f32 {
        self.bounds_height
    }
    pub fn set_bounds_height(&mut self, value: f32, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_bounds_height_value(value) {
            callbacks.bounds_height_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::BOUNDS_HEIGHT_PROPERTY_KEY,
            );
        }
    }
    pub(crate) fn set_bounds_height_value(&mut self, value: f32) -> bool {
        if self.bounds_height == value {
            return false;
        }
        self.bounds_height = value;
        true
    }
    pub fn mask_mode_value(&self) -> u8 {
        ((self.mask_flags & Self::MASK_MODE_VALUE_FIELD_MASK) >> Self::MASK_MODE_VALUE_BIT_OFFSET)
            as u8
    }
    pub(crate) fn set_mask_mode_value_value(&mut self, value: u8) -> bool {
        if self.mask_mode_value() == value {
            return false;
        }
        self.mask_flags = (self.mask_flags & !Self::MASK_MODE_VALUE_FIELD_MASK)
            | (((value as u32) << Self::MASK_MODE_VALUE_BIT_OFFSET)
                & Self::MASK_MODE_VALUE_FIELD_MASK);
        true
    }
    pub fn set_mask_mode_value(&mut self, value: u8, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_mask_mode_value_value(value) {
            callbacks.mask_flags_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::MASK_FLAGS_PROPERTY_KEY,
            );
        }
    }
    pub fn is_visible(&self) -> bool {
        self.mask_flags & Self::IS_VISIBLE_BITMASK != 0
    }
    pub(crate) fn set_is_visible_value(&mut self, value: bool) -> bool {
        if self.is_visible() == value {
            return false;
        }
        self.mask_flags = if value {
            self.mask_flags | Self::IS_VISIBLE_BITMASK
        } else {
            self.mask_flags & !Self::IS_VISIBLE_BITMASK
        };
        true
    }
    pub fn set_is_visible(&mut self, value: bool, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_is_visible_value(value) {
            callbacks.mask_flags_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::MASK_FLAGS_PROPERTY_KEY,
            );
        }
    }
    pub fn source_draws(&self) -> bool {
        self.mask_flags & Self::SOURCE_DRAWS_BITMASK != 0
    }
    pub(crate) fn set_source_draws_value(&mut self, value: bool) -> bool {
        if self.source_draws() == value {
            return false;
        }
        self.mask_flags = if value {
            self.mask_flags | Self::SOURCE_DRAWS_BITMASK
        } else {
            self.mask_flags & !Self::SOURCE_DRAWS_BITMASK
        };
        true
    }
    pub fn set_source_draws(&mut self, value: bool, callbacks: &mut impl LayerMaskBaseCallbacks) {
        if self.set_source_draws_value(value) {
            callbacks.mask_flags_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::MASK_FLAGS_PROPERTY_KEY,
            );
        }
    }
    pub fn use_custom_bounds(&self) -> bool {
        self.mask_flags & Self::USE_CUSTOM_BOUNDS_BITMASK != 0
    }
    pub(crate) fn set_use_custom_bounds_value(&mut self, value: bool) -> bool {
        if self.use_custom_bounds() == value {
            return false;
        }
        self.mask_flags = if value {
            self.mask_flags | Self::USE_CUSTOM_BOUNDS_BITMASK
        } else {
            self.mask_flags & !Self::USE_CUSTOM_BOUNDS_BITMASK
        };
        true
    }
    pub fn set_use_custom_bounds(
        &mut self,
        value: bool,
        callbacks: &mut impl LayerMaskBaseCallbacks,
    ) {
        if self.set_use_custom_bounds_value(value) {
            callbacks.mask_flags_changed();
            LayerMaskBaseCallbacks::notify_property_changed(
                callbacks,
                Self::MASK_FLAGS_PROPERTY_KEY,
            );
        }
    }
    pub fn clone_into(&self, callbacks: &mut impl LayerMaskBaseCallbacks) -> LayerMask {
        let mut cloned = LayerMask::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LayerMaskBaseCallbacks) {
        self.source_id = object.source_id;
        self.mask_flags = object.mask_flags;
        self.resolution = object.resolution;
        self.bounds_x = object.bounds_x;
        self.bounds_y = object.bounds_y;
        self.bounds_width = object.bounds_width;
        self.bounds_height = object.bounds_height;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LayerMaskBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SOURCE_ID_PROPERTY_KEY => {
                self.source_id = crate::mechanical_port::source::core::field_types::core_id_type::CoreIdType::runtime_deserialize(reader);
                true
            }
            Self::MASK_FLAGS_PROPERTY_KEY => {
                self.mask_flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::RESOLUTION_PROPERTY_KEY => {
                self.resolution = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BOUNDS_X_PROPERTY_KEY => {
                self.bounds_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BOUNDS_Y_PROPERTY_KEY => {
                self.bounds_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BOUNDS_WIDTH_PROPERTY_KEY => {
                self.bounds_width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BOUNDS_HEIGHT_PROPERTY_KEY => {
                self.bounds_height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
impl std::ops::Deref for LayerMaskBase {
    type Target = Component;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for LayerMaskBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
