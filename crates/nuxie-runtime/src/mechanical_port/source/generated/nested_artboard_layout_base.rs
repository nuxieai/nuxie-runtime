use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, nested_artboard::NestedArtboard,
    nested_artboard_layout::NestedArtboardLayout,
};

pub trait NestedArtboardLayoutBaseCallbacks:
    crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn instance_width_changed(&mut self) {}
    fn instance_height_changed(&mut self) {}
    fn instance_width_units_value_changed(&mut self) {}
    fn instance_height_units_value_changed(&mut self) {}
    fn instance_width_scale_type_changed(&mut self) {}
    fn instance_height_scale_type_changed(&mut self) {}
}

pub struct NestedArtboardLayoutBase {
    pub base: NestedArtboard,
    instance_width: f32,
    instance_height: f32,
    instance_width_units_value: u32,
    instance_height_units_value: u32,
    instance_width_scale_type: u32,
    instance_height_scale_type: u32,
}

impl Default for NestedArtboardLayoutBase {
    fn default() -> Self {
        Self {
            base: NestedArtboard::default(),
            instance_width: -1.0,
            instance_height: -1.0,
            instance_width_units_value: 1,
            instance_height_units_value: 1,
            instance_width_scale_type: 0,
            instance_height_scale_type: 0,
        }
    }
}

impl NestedArtboardLayoutBase {
    pub const TYPE_KEY: u16 = 452;
    pub const INSTANCE_WIDTH_PROPERTY_KEY: u16 = 663;
    pub const INSTANCE_HEIGHT_PROPERTY_KEY: u16 = 664;
    pub const INSTANCE_WIDTH_UNITS_VALUE_PROPERTY_KEY: u16 = 665;
    pub const INSTANCE_HEIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 666;
    pub const INSTANCE_WIDTH_SCALE_TYPE_PROPERTY_KEY: u16 = 667;
    pub const INSTANCE_HEIGHT_SCALE_TYPE_PROPERTY_KEY: u16 = 668;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 92 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn instance_width(&self) -> f32 {
        self.instance_width
    }
    pub fn set_instance_width(
        &mut self,
        value: f32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if !self.set_instance_width_value(value) {
            return;
        }
        callbacks.instance_width_changed();
        NestedArtboardLayoutBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INSTANCE_WIDTH_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_instance_width_value(&mut self, value: f32) -> bool {
        if self.instance_width == value {
            return false;
        }
        self.instance_width = value;
        true
    }
    pub fn instance_height(&self) -> f32 {
        self.instance_height
    }
    pub fn set_instance_height(
        &mut self,
        value: f32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if !self.set_instance_height_value(value) {
            return;
        }
        callbacks.instance_height_changed();
        NestedArtboardLayoutBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INSTANCE_HEIGHT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_instance_height_value(&mut self, value: f32) -> bool {
        if self.instance_height == value {
            return false;
        }
        self.instance_height = value;
        true
    }
    pub fn instance_width_units_value(&self) -> u32 {
        self.instance_width_units_value
    }
    pub fn set_instance_width_units_value(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if !self.set_instance_width_units_value_value(value) {
            return;
        }
        callbacks.instance_width_units_value_changed();
        NestedArtboardLayoutBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INSTANCE_WIDTH_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_instance_width_units_value_value(&mut self, value: u32) -> bool {
        if self.instance_width_units_value == value {
            return false;
        }
        self.instance_width_units_value = value;
        true
    }
    pub fn instance_height_units_value(&self) -> u32 {
        self.instance_height_units_value
    }
    pub fn set_instance_height_units_value(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if !self.set_instance_height_units_value_value(value) {
            return;
        }
        callbacks.instance_height_units_value_changed();
        NestedArtboardLayoutBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INSTANCE_HEIGHT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_instance_height_units_value_value(&mut self, value: u32) -> bool {
        if self.instance_height_units_value == value {
            return false;
        }
        self.instance_height_units_value = value;
        true
    }
    pub fn instance_width_scale_type(&self) -> u32 {
        self.instance_width_scale_type
    }
    pub fn set_instance_width_scale_type(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if !self.set_instance_width_scale_type_value(value) {
            return;
        }
        callbacks.instance_width_scale_type_changed();
        NestedArtboardLayoutBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INSTANCE_WIDTH_SCALE_TYPE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_instance_width_scale_type_value(&mut self, value: u32) -> bool {
        if self.instance_width_scale_type == value {
            return false;
        }
        self.instance_width_scale_type = value;
        true
    }
    pub fn instance_height_scale_type(&self) -> u32 {
        self.instance_height_scale_type
    }
    pub fn set_instance_height_scale_type(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if !self.set_instance_height_scale_type_value(value) {
            return;
        }
        callbacks.instance_height_scale_type_changed();
        NestedArtboardLayoutBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INSTANCE_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_instance_height_scale_type_value(&mut self, value: u32) -> bool {
        if self.instance_height_scale_type == value {
            return false;
        }
        self.instance_height_scale_type = value;
        true
    }
    pub fn clone_into(source: &NestedArtboardLayout) -> NestedArtboardLayout {
        let mut cloned = NestedArtboardLayout::default();
        cloned.base.instance_width = source.base.instance_width;
        cloned.base.instance_height = source.base.instance_height;
        cloned.base.instance_width_units_value = source.base.instance_width_units_value;
        cloned.base.instance_height_units_value = source.base.instance_height_units_value;
        cloned.base.instance_width_scale_type = source.base.instance_width_scale_type;
        cloned.base.instance_height_scale_type = source.base.instance_height_scale_type;
        let mut nested_base = std::mem::take(&mut cloned.base.base.base);
        nested_base.copy(&source.base.base, &mut cloned.base.base);
        cloned.base.base.base = nested_base;
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedArtboardLayoutBaseCallbacks) {
        self.instance_width = object.instance_width;
        self.instance_height = object.instance_height;
        self.instance_width_units_value = object.instance_width_units_value;
        self.instance_height_units_value = object.instance_height_units_value;
        self.instance_width_scale_type = object.instance_width_scale_type;
        self.instance_height_scale_type = object.instance_height_scale_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INSTANCE_WIDTH_PROPERTY_KEY => {
                self.instance_width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::INSTANCE_HEIGHT_PROPERTY_KEY => {
                self.instance_height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::INSTANCE_WIDTH_UNITS_VALUE_PROPERTY_KEY => {
                self.instance_width_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::INSTANCE_HEIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.instance_height_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::INSTANCE_WIDTH_SCALE_TYPE_PROPERTY_KEY => {
                self.instance_width_scale_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::INSTANCE_HEIGHT_SCALE_TYPE_PROPERTY_KEY => {
                self.instance_height_scale_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NestedArtboardLayoutBase {
    type Target = NestedArtboard;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedArtboardLayoutBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
