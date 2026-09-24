//! Translation of include/rive/generated/bitmap_cache_base.hpp at upstream a4dbc3ff.

use crate::mechanical_port::source::{
    bitmap_cache::BitmapCache, component::Component, core::binary_reader::BinaryReader,
};

pub trait BitmapCacheBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn resolution_changed(&mut self) {}
    fn cache_flags_changed(&mut self) {}
}

pub struct BitmapCacheBase {
    pub base: Component,
    resolution: f32,
    cache_flags: u32,
}

impl Default for BitmapCacheBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            resolution: 1.0,
            cache_flags: 1,
        }
    }
}

impl BitmapCacheBase {
    pub const TYPE_KEY: u16 = 136;
    pub const RESOLUTION_PROPERTY_KEY: u16 = 417;
    pub const CACHE_FLAGS_PROPERTY_KEY: u16 = 418;
    pub const CACHE_ENABLED_PROPERTY_KEY: u16 = 419;
    pub const CACHE_ENABLED_BITMASK: u32 = 1 << 0;
    pub const DITHER_PROPERTY_KEY: u16 = 420;
    pub const DITHER_BITMASK: u32 = 1 << 1;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn resolution(&self) -> f32 {
        self.resolution
    }
    pub fn set_resolution(&mut self, value: f32, callbacks: &mut impl BitmapCacheBaseCallbacks) {
        if !self.set_resolution_value(value) {
            return;
        }
        callbacks.resolution_changed();
        BitmapCacheBaseCallbacks::notify_property_changed(callbacks, Self::RESOLUTION_PROPERTY_KEY);
    }

    pub(crate) fn set_resolution_value(&mut self, value: f32) -> bool {
        if self.resolution == value {
            return false;
        }
        self.resolution = value;
        true
    }
    pub fn cache_flags(&self) -> u32 {
        self.cache_flags
    }
    pub fn set_cache_flags(&mut self, value: u32, callbacks: &mut impl BitmapCacheBaseCallbacks) {
        if !self.set_cache_flags_value(value) {
            return;
        }
        callbacks.cache_flags_changed();
        BitmapCacheBaseCallbacks::notify_property_changed(
            callbacks,
            Self::CACHE_FLAGS_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_cache_flags_value(&mut self, value: u32) -> bool {
        if self.cache_flags == value {
            return false;
        }
        self.cache_flags = value;
        true
    }
    pub fn cache_enabled(&self) -> bool {
        (self.cache_flags & Self::CACHE_ENABLED_BITMASK) != 0
    }
    pub fn dither(&self) -> bool {
        (self.cache_flags & Self::DITHER_BITMASK) != 0
    }
    pub fn set_cache_enabled(
        &mut self,
        value: bool,
        callbacks: &mut impl BitmapCacheBaseCallbacks,
    ) {
        self.set_cache_flags_bit(Self::CACHE_ENABLED_BITMASK, value, callbacks);
    }
    pub fn set_dither(&mut self, value: bool, callbacks: &mut impl BitmapCacheBaseCallbacks) {
        self.set_cache_flags_bit(Self::DITHER_BITMASK, value, callbacks);
    }
    fn set_cache_flags_bit(
        &mut self,
        mask: u32,
        value: bool,
        callbacks: &mut impl BitmapCacheBaseCallbacks,
    ) {
        let prev = (self.cache_flags & mask) != 0;
        if prev == value {
            return;
        }
        self.cache_flags = if value {
            self.cache_flags | mask
        } else {
            self.cache_flags & !mask
        };
        callbacks.cache_flags_changed();
        BitmapCacheBaseCallbacks::notify_property_changed(
            callbacks,
            Self::CACHE_FLAGS_PROPERTY_KEY,
        );
    }
    pub fn clone_into(&self, callbacks: &mut impl BitmapCacheBaseCallbacks) -> BitmapCache {
        let mut cloned = BitmapCache::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl BitmapCacheBaseCallbacks) {
        self.resolution = object.resolution;
        self.cache_flags = object.cache_flags;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BitmapCacheBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::RESOLUTION_PROPERTY_KEY => {
                self.resolution = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CACHE_FLAGS_PROPERTY_KEY => {
                self.cache_flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for BitmapCacheBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BitmapCacheBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
