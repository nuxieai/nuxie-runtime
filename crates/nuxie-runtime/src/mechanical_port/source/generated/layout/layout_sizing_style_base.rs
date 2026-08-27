use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub trait LayoutSizingStyleBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn min_width_changed(&mut self) {}
    fn max_width_changed(&mut self) {}
    fn min_height_changed(&mut self) {}
    fn max_height_changed(&mut self) {}
    fn layout_width_scale_type_changed(&mut self) {}
    fn layout_height_scale_type_changed(&mut self) {}
    fn width_units_value_changed(&mut self) {}
    fn height_units_value_changed(&mut self) {}
    fn min_width_units_value_changed(&mut self) {}
    fn max_width_units_value_changed(&mut self) {}
    fn min_height_units_value_changed(&mut self) {}
    fn max_height_units_value_changed(&mut self) {}
    fn justify_self_value_changed(&mut self) {}
    fn display_value_changed(&mut self) {}
}

pub struct LayoutSizingStyleBase {
    pub base: Component,
    min_width: f32,
    max_width: f32,
    min_height: f32,
    max_height: f32,
    layout_width_scale_type: u8,
    layout_height_scale_type: u8,
    width_units_value: u8,
    height_units_value: u8,
    min_width_units_value: u8,
    max_width_units_value: u8,
    min_height_units_value: u8,
    max_height_units_value: u8,
    justify_self_value: u8,
    display_value: u8,
}

impl Default for LayoutSizingStyleBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            min_width: 0.0,
            max_width: 0.0,
            min_height: 0.0,
            max_height: 0.0,
            layout_width_scale_type: 0,
            layout_height_scale_type: 0,
            width_units_value: 1,
            height_units_value: 1,
            min_width_units_value: 0,
            max_width_units_value: 0,
            min_height_units_value: 0,
            max_height_units_value: 0,
            justify_self_value: 6,
            display_value: 0,
        }
    }
}

impl LayoutSizingStyleBase {
    pub const TYPE_KEY: u16 = 1056;
    pub const MIN_WIDTH_PROPERTY_KEY: u16 = 502;
    pub const MAX_WIDTH_PROPERTY_KEY: u16 = 500;
    pub const MIN_HEIGHT_PROPERTY_KEY: u16 = 503;
    pub const MAX_HEIGHT_PROPERTY_KEY: u16 = 501;
    pub const LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY: u16 = 655;
    pub const LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY: u16 = 656;
    pub const WIDTH_UNITS_VALUE_PROPERTY_KEY: u16 = 607;
    pub const HEIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 608;
    pub const MIN_WIDTH_UNITS_VALUE_PROPERTY_KEY: u16 = 627;
    pub const MAX_WIDTH_UNITS_VALUE_PROPERTY_KEY: u16 = 629;
    pub const MIN_HEIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 628;
    pub const MAX_HEIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 630;
    pub const JUSTIFY_SELF_VALUE_PROPERTY_KEY: u16 = 1046;
    pub const DISPLAY_VALUE_PROPERTY_KEY: u16 = 596;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn min_width(&self) -> f32 {
        self.min_width
    }
    pub fn set_min_width(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.min_width == value {
            return;
        }
        self.min_width = value;
        callbacks.min_width_changed();
        callbacks.notify_property_changed(Self::MIN_WIDTH_PROPERTY_KEY);
    }
    pub fn max_width(&self) -> f32 {
        self.max_width
    }
    pub fn set_max_width(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.max_width == value {
            return;
        }
        self.max_width = value;
        callbacks.max_width_changed();
        callbacks.notify_property_changed(Self::MAX_WIDTH_PROPERTY_KEY);
    }
    pub fn min_height(&self) -> f32 {
        self.min_height
    }
    pub fn set_min_height(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.min_height == value {
            return;
        }
        self.min_height = value;
        callbacks.min_height_changed();
        callbacks.notify_property_changed(Self::MIN_HEIGHT_PROPERTY_KEY);
    }
    pub fn max_height(&self) -> f32 {
        self.max_height
    }
    pub fn set_max_height(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.max_height == value {
            return;
        }
        self.max_height = value;
        callbacks.max_height_changed();
        callbacks.notify_property_changed(Self::MAX_HEIGHT_PROPERTY_KEY);
    }
    pub fn layout_width_scale_type(&self) -> u8 {
        self.layout_width_scale_type
    }
    pub fn set_layout_width_scale_type(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.layout_width_scale_type == value {
            return;
        }
        self.layout_width_scale_type = value;
        callbacks.layout_width_scale_type_changed();
        callbacks.notify_property_changed(Self::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY);
    }
    pub fn layout_height_scale_type(&self) -> u8 {
        self.layout_height_scale_type
    }
    pub fn set_layout_height_scale_type(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.layout_height_scale_type == value {
            return;
        }
        self.layout_height_scale_type = value;
        callbacks.layout_height_scale_type_changed();
        callbacks.notify_property_changed(Self::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY);
    }
    pub fn width_units_value(&self) -> u8 {
        self.width_units_value
    }
    pub fn set_width_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.width_units_value == value {
            return;
        }
        self.width_units_value = value;
        callbacks.width_units_value_changed();
        callbacks.notify_property_changed(Self::WIDTH_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn height_units_value(&self) -> u8 {
        self.height_units_value
    }
    pub fn set_height_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.height_units_value == value {
            return;
        }
        self.height_units_value = value;
        callbacks.height_units_value_changed();
        callbacks.notify_property_changed(Self::HEIGHT_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn min_width_units_value(&self) -> u8 {
        self.min_width_units_value
    }
    pub fn set_min_width_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.min_width_units_value == value {
            return;
        }
        self.min_width_units_value = value;
        callbacks.min_width_units_value_changed();
        callbacks.notify_property_changed(Self::MIN_WIDTH_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn max_width_units_value(&self) -> u8 {
        self.max_width_units_value
    }
    pub fn set_max_width_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.max_width_units_value == value {
            return;
        }
        self.max_width_units_value = value;
        callbacks.max_width_units_value_changed();
        callbacks.notify_property_changed(Self::MAX_WIDTH_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn min_height_units_value(&self) -> u8 {
        self.min_height_units_value
    }
    pub fn set_min_height_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.min_height_units_value == value {
            return;
        }
        self.min_height_units_value = value;
        callbacks.min_height_units_value_changed();
        callbacks.notify_property_changed(Self::MIN_HEIGHT_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn max_height_units_value(&self) -> u8 {
        self.max_height_units_value
    }
    pub fn set_max_height_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.max_height_units_value == value {
            return;
        }
        self.max_height_units_value = value;
        callbacks.max_height_units_value_changed();
        callbacks.notify_property_changed(Self::MAX_HEIGHT_UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn justify_self_value(&self) -> u8 {
        self.justify_self_value
    }
    pub fn set_justify_self_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.justify_self_value == value {
            return;
        }
        self.justify_self_value = value;
        callbacks.justify_self_value_changed();
        callbacks.notify_property_changed(Self::JUSTIFY_SELF_VALUE_PROPERTY_KEY);
    }
    pub fn display_value(&self) -> u8 {
        self.display_value
    }
    pub fn set_display_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) {
        if self.display_value == value {
            return;
        }
        self.display_value = value;
        callbacks.display_value_changed();
        callbacks.notify_property_changed(Self::DISPLAY_VALUE_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LayoutSizingStyleBaseCallbacks) {
        self.min_width = object.min_width;
        self.max_width = object.max_width;
        self.min_height = object.min_height;
        self.max_height = object.max_height;
        self.layout_width_scale_type = object.layout_width_scale_type;
        self.layout_height_scale_type = object.layout_height_scale_type;
        self.width_units_value = object.width_units_value;
        self.height_units_value = object.height_units_value;
        self.min_width_units_value = object.min_width_units_value;
        self.max_width_units_value = object.max_width_units_value;
        self.min_height_units_value = object.min_height_units_value;
        self.max_height_units_value = object.max_height_units_value;
        self.justify_self_value = object.justify_self_value;
        self.display_value = object.display_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LayoutSizingStyleBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::MIN_WIDTH_PROPERTY_KEY => {
                self.min_width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MAX_WIDTH_PROPERTY_KEY => {
                self.max_width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MIN_HEIGHT_PROPERTY_KEY => {
                self.min_height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MAX_HEIGHT_PROPERTY_KEY => {
                self.max_height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY => {
                self.layout_width_scale_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY => {
                self.layout_height_scale_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::WIDTH_UNITS_VALUE_PROPERTY_KEY => {
                self.width_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::HEIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.height_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::MIN_WIDTH_UNITS_VALUE_PROPERTY_KEY => {
                self.min_width_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::MAX_WIDTH_UNITS_VALUE_PROPERTY_KEY => {
                self.max_width_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::MIN_HEIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.min_height_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::MAX_HEIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.max_height_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::JUSTIFY_SELF_VALUE_PROPERTY_KEY => {
                self.justify_self_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::DISPLAY_VALUE_PROPERTY_KEY => {
                self.display_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
