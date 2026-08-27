use crate::mechanical_port::source::{core::binary_reader::BinaryReader, shapes::path::Path};

pub trait ParametricPathBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn width_changed(&mut self) {}
    fn height_changed(&mut self) {}
    fn origin_x_changed(&mut self) {}
    fn origin_y_changed(&mut self) {}
}

pub struct ParametricPathBase {
    pub base: Path,
    width: f32,
    height: f32,
    origin_x: f32,
    origin_y: f32,
}

impl Default for ParametricPathBase {
    fn default() -> Self {
        Self {
            base: Path::default(),
            width: 0.0,
            height: 0.0,
            origin_x: 0.5,
            origin_y: 0.5,
        }
    }
}

impl ParametricPathBase {
    pub const TYPE_KEY: u16 = 15;
    pub const WIDTH_PROPERTY_KEY: u16 = 20;
    pub const HEIGHT_PROPERTY_KEY: u16 = 21;
    pub const ORIGIN_X_PROPERTY_KEY: u16 = 123;
    pub const ORIGIN_Y_PROPERTY_KEY: u16 = 124;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn set_width(&mut self, value: f32, callbacks: &mut impl ParametricPathBaseCallbacks) {
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
    pub fn set_height(&mut self, value: f32, callbacks: &mut impl ParametricPathBaseCallbacks) {
        if self.height == value {
            return;
        }
        self.height = value;
        callbacks.height_changed();
        callbacks.notify_property_changed(Self::HEIGHT_PROPERTY_KEY);
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn set_origin_x(&mut self, value: f32, callbacks: &mut impl ParametricPathBaseCallbacks) {
        if self.origin_x == value {
            return;
        }
        self.origin_x = value;
        callbacks.origin_x_changed();
        callbacks.notify_property_changed(Self::ORIGIN_X_PROPERTY_KEY);
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn set_origin_y(&mut self, value: f32, callbacks: &mut impl ParametricPathBaseCallbacks) {
        if self.origin_y == value {
            return;
        }
        self.origin_y = value;
        callbacks.origin_y_changed();
        callbacks.notify_property_changed(Self::ORIGIN_Y_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ParametricPathBaseCallbacks) {
        self.width = object.width;
        self.height = object.height;
        self.origin_x = object.origin_x;
        self.origin_y = object.origin_y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ParametricPathBaseCallbacks,
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
            Self::ORIGIN_X_PROPERTY_KEY => {
                self.origin_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_Y_PROPERTY_KEY => {
                self.origin_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
