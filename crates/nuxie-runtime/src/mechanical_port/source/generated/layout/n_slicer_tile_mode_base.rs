use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader,
    layout::n_slicer_tile_mode::NSlicerTileMode,
};

pub trait NSlicerTileModeBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn patch_index_changed(&mut self) {}
    fn style_changed(&mut self) {}
}

pub struct NSlicerTileModeBase {
    pub base: Component,
    patch_index: u32,
    style: u32,
}

impl Default for NSlicerTileModeBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            patch_index: 0,
            style: 0,
        }
    }
}

impl NSlicerTileModeBase {
    pub const TYPE_KEY: u16 = 491;
    pub const PATCH_INDEX_PROPERTY_KEY: u16 = 672;
    pub const STYLE_PROPERTY_KEY: u16 = 673;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn patch_index(&self) -> u32 {
        self.patch_index
    }
    pub fn set_patch_index(
        &mut self,
        value: u32,
        callbacks: &mut impl NSlicerTileModeBaseCallbacks,
    ) {
        if !self.set_patch_index_value(value) {
            return;
        }
        callbacks.patch_index_changed();
        callbacks.notify_property_changed(Self::PATCH_INDEX_PROPERTY_KEY);
    }

    pub(crate) fn set_patch_index_value(&mut self, value: u32) -> bool {
        if self.patch_index == value {
            return false;
        }
        self.patch_index = value;
        true
    }
    pub fn style(&self) -> u32 {
        self.style
    }
    pub fn set_style(&mut self, value: u32, callbacks: &mut impl NSlicerTileModeBaseCallbacks) {
        if !self.set_style_value(value) {
            return;
        }
        callbacks.style_changed();
        callbacks.notify_property_changed(Self::STYLE_PROPERTY_KEY);
    }

    pub(crate) fn set_style_value(&mut self, value: u32) -> bool {
        if self.style == value {
            return false;
        }
        self.style = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl NSlicerTileModeBaseCallbacks) -> NSlicerTileMode {
        let mut cloned = NSlicerTileMode::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NSlicerTileModeBaseCallbacks) {
        self.patch_index = object.patch_index;
        self.style = object.style;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NSlicerTileModeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PATCH_INDEX_PROPERTY_KEY => {
                self.patch_index = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::STYLE_PROPERTY_KEY => {
                self.style = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NSlicerTileModeBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NSlicerTileModeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
