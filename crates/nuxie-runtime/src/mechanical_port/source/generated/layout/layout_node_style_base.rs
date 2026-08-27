use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, layout::layout_node_style::LayoutNodeStyle,
    layout::layout_sizing_style::LayoutSizingStyle,
};

pub trait LayoutNodeStyleBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn width_changed(&mut self) {}
    fn height_changed(&mut self) {}
    fn fractional_width_changed(&mut self) {}
    fn fractional_height_changed(&mut self) {}
}

pub struct LayoutNodeStyleBase {
    pub base: LayoutSizingStyle,
    width: f32,
    height: f32,
    fractional_width: f32,
    fractional_height: f32,
}

impl Default for LayoutNodeStyleBase {
    fn default() -> Self {
        Self {
            base: LayoutSizingStyle::default(),
            width: 0.0,
            height: 0.0,
            fractional_width: 1.0,
            fractional_height: 1.0,
        }
    }
}

impl LayoutNodeStyleBase {
    pub const TYPE_KEY: u16 = 1057;
    pub const WIDTH_PROPERTY_KEY: u16 = 1066;
    pub const HEIGHT_PROPERTY_KEY: u16 = 1067;
    pub const FRACTIONAL_WIDTH_PROPERTY_KEY: u16 = 1057;
    pub const FRACTIONAL_HEIGHT_PROPERTY_KEY: u16 = 1058;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 1056 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn set_width(&mut self, value: f32, callbacks: &mut impl LayoutNodeStyleBaseCallbacks) {
        if self.width == value {
            return;
        }
        self.width = value;
        callbacks.width_changed();
        callbacks.notify_property_changed(Self::WIDTH_PROPERTY_KEY);
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn set_height(&mut self, value: f32, callbacks: &mut impl LayoutNodeStyleBaseCallbacks) {
        if self.height == value {
            return;
        }
        self.height = value;
        callbacks.height_changed();
        callbacks.notify_property_changed(Self::HEIGHT_PROPERTY_KEY);
    }
    pub fn fractional_width(&self) -> f32 {
        self.fractional_width
    }
    pub fn set_fractional_width(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutNodeStyleBaseCallbacks,
    ) {
        if self.fractional_width == value {
            return;
        }
        self.fractional_width = value;
        callbacks.fractional_width_changed();
        callbacks.notify_property_changed(Self::FRACTIONAL_WIDTH_PROPERTY_KEY);
    }
    pub fn fractional_height(&self) -> f32 {
        self.fractional_height
    }
    pub fn set_fractional_height(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutNodeStyleBaseCallbacks,
    ) {
        if self.fractional_height == value {
            return;
        }
        self.fractional_height = value;
        callbacks.fractional_height_changed();
        callbacks.notify_property_changed(Self::FRACTIONAL_HEIGHT_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl LayoutNodeStyleBaseCallbacks) -> LayoutNodeStyle {
        let mut cloned = LayoutNodeStyle::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LayoutNodeStyleBaseCallbacks) {
        self.width = object.width;
        self.height = object.height;
        self.fractional_width = object.fractional_width;
        self.fractional_height = object.fractional_height;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LayoutNodeStyleBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::WIDTH_PROPERTY_KEY => {
                self.width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::HEIGHT_PROPERTY_KEY => {
                self.height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
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
