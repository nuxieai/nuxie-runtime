use crate::mechanical_port::source::{
    component::Component, component_origin::ComponentOrigin, core::binary_reader::BinaryReader,
};

pub trait ComponentOriginBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn origin_x_changed(&mut self) {}
    fn origin_y_changed(&mut self) {}
}

pub struct ComponentOriginBase {
    pub base: Component,
    origin_x: f32,
    origin_y: f32,
}

impl Default for ComponentOriginBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            origin_x: 0.0,
            origin_y: 0.0,
        }
    }
}

impl ComponentOriginBase {
    pub const TYPE_KEY: u16 = 1039;
    pub const ORIGIN_X_PROPERTY_KEY: u16 = 1040;
    pub const ORIGIN_Y_PROPERTY_KEY: u16 = 1041;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn set_origin_x(&mut self, value: f32, callbacks: &mut impl ComponentOriginBaseCallbacks) {
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
    pub fn set_origin_y(&mut self, value: f32, callbacks: &mut impl ComponentOriginBaseCallbacks) {
        if self.origin_y == value {
            return;
        }
        self.origin_y = value;
        callbacks.origin_y_changed();
        callbacks.notify_property_changed(Self::ORIGIN_Y_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl ComponentOriginBaseCallbacks) -> ComponentOrigin {
        let mut cloned = ComponentOrigin::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ComponentOriginBaseCallbacks) {
        self.origin_x = object.origin_x;
        self.origin_y = object.origin_y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ComponentOriginBaseCallbacks,
    ) -> bool {
        match property_key {
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
