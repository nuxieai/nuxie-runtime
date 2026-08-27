use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, nested_artboard::NestedArtboard,
    nested_artboard_layout::NestedArtboardLayout,
};

pub trait NestedArtboardLayoutBaseCallbacks {
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
        if self.instance_width == value {
            return;
        }
        self.instance_width = value;
        callbacks.instance_width_changed();
        callbacks.notify_property_changed(Self::INSTANCE_WIDTH_PROPERTY_KEY);
    }
    pub fn instance_height(&self) -> f32 {
        self.instance_height
    }
    pub fn set_instance_height(
        &mut self,
        value: f32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if self.instance_height == value {
            return;
        }
        self.instance_height = value;
        callbacks.instance_height_changed();
        callbacks.notify_property_changed(Self::INSTANCE_HEIGHT_PROPERTY_KEY);
    }
    pub fn instance_width_units_value(&self) -> u32 {
        self.instance_width_units_value
    }
    pub fn set_instance_width_units_value(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if self.instance_width_units_value == value {
            return;
        }
        self.instance_width_units_value = value;
        callbacks.instance_width_units_value_changed();
        callbacks.notify_property_changed(Self::INSTANCE_WIDTH_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn instance_height_units_value(&self) -> u32 {
        self.instance_height_units_value
    }
    pub fn set_instance_height_units_value(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if self.instance_height_units_value == value {
            return;
        }
        self.instance_height_units_value = value;
        callbacks.instance_height_units_value_changed();
        callbacks.notify_property_changed(Self::INSTANCE_HEIGHT_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn instance_width_scale_type(&self) -> u32 {
        self.instance_width_scale_type
    }
    pub fn set_instance_width_scale_type(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if self.instance_width_scale_type == value {
            return;
        }
        self.instance_width_scale_type = value;
        callbacks.instance_width_scale_type_changed();
        callbacks.notify_property_changed(Self::INSTANCE_WIDTH_SCALE_TYPE_PROPERTY_KEY);
    }
    pub fn instance_height_scale_type(&self) -> u32 {
        self.instance_height_scale_type
    }
    pub fn set_instance_height_scale_type(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) {
        if self.instance_height_scale_type == value {
            return;
        }
        self.instance_height_scale_type = value;
        callbacks.instance_height_scale_type_changed();
        callbacks.notify_property_changed(Self::INSTANCE_HEIGHT_SCALE_TYPE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl NestedArtboardLayoutBaseCallbacks,
    ) -> NestedArtboardLayout {
        let mut cloned = NestedArtboardLayout::default();
        cloned.base.copy(self, callbacks);
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
