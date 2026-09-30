use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, Core},
    selection_style::SelectionStyle,
};

pub trait SelectionStyleBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn highlight_color_changed(&mut self) {}
    fn corner_radius_changed(&mut self) {}
}

pub struct SelectionStyleBase {
    pub base: Core,
    highlight_color: i32,
    corner_radius: f32,
}
impl Default for SelectionStyleBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            highlight_color: 0x663B82F6,
            corner_radius: 0.0,
        }
    }
}
impl SelectionStyleBase {
    pub const TYPE_KEY: u16 = 153;
    pub const HIGHLIGHT_COLOR_PROPERTY_KEY: u16 = 447;
    pub const CORNER_RADIUS_PROPERTY_KEY: u16 = 448;
    pub fn is_type_of(key: u16) -> bool {
        key == Self::TYPE_KEY
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn highlight_color(&self) -> i32 {
        self.highlight_color
    }
    pub fn corner_radius(&self) -> f32 {
        self.corner_radius
    }
    pub(crate) fn set_highlight_color_value(&mut self, value: i32) -> bool {
        if self.highlight_color == value {
            return false;
        }
        self.highlight_color = value;
        true
    }
    pub(crate) fn set_corner_radius_value(&mut self, value: f32) -> bool {
        if self.corner_radius == value {
            return false;
        }
        self.corner_radius = value;
        true
    }
    pub fn set_highlight_color(
        &mut self,
        value: i32,
        callbacks: &mut impl SelectionStyleBaseCallbacks,
    ) {
        if self.set_highlight_color_value(value) {
            callbacks.highlight_color_changed();
            callbacks.notify_property_changed(Self::HIGHLIGHT_COLOR_PROPERTY_KEY);
        }
    }
    pub fn set_corner_radius(
        &mut self,
        value: f32,
        callbacks: &mut impl SelectionStyleBaseCallbacks,
    ) {
        if self.set_corner_radius_value(value) {
            callbacks.corner_radius_changed();
            callbacks.notify_property_changed(Self::CORNER_RADIUS_PROPERTY_KEY);
        }
    }
    pub fn copy(&mut self, object: &Self) {
        self.highlight_color = object.highlight_color;
        self.corner_radius = object.corner_radius;
    }
    pub fn clone_into(&self) -> SelectionStyle {
        let mut cloned = SelectionStyle::default();
        cloned.base.copy(self);
        cloned
    }
    pub fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match key {
            Self::HIGHLIGHT_COLOR_PROPERTY_KEY => {
                self.highlight_color = crate::mechanical_port::source::core::field_types::core_color_type::CoreColorType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_PROPERTY_KEY => {
                self.corner_radius = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}
impl std::ops::Deref for SelectionStyleBase {
    type Target = Core;
    fn deref(&self) -> &Core {
        &self.base
    }
}
impl std::ops::DerefMut for SelectionStyleBase {
    fn deref_mut(&mut self) -> &mut Core {
        &mut self.base
    }
}
