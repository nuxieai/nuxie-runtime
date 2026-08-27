use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
};

pub trait VertexBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn x_changed(&mut self) {}
    fn y_changed(&mut self) {}
}

pub struct VertexBase {
    pub base: ContainerComponent,
    x: f32,
    y: f32,
}

impl Default for VertexBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            x: 0.0,
            y: 0.0,
        }
    }
}

impl VertexBase {
    pub const TYPE_KEY: u16 = 107;
    pub const X_PROPERTY_KEY: u16 = 24;
    pub const Y_PROPERTY_KEY: u16 = 25;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn set_x(&mut self, value: f32, callbacks: &mut impl VertexBaseCallbacks) {
        if self.x == value {
            return;
        }
        self.x = value;
        callbacks.x_changed();
        callbacks.notify_property_changed(Self::X_PROPERTY_KEY);
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn set_y(&mut self, value: f32, callbacks: &mut impl VertexBaseCallbacks) {
        if self.y == value {
            return;
        }
        self.y = value;
        callbacks.y_changed();
        callbacks.notify_property_changed(Self::Y_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl VertexBaseCallbacks) {
        self.x = object.x;
        self.y = object.y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl VertexBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::X_PROPERTY_KEY => {
                self.x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::Y_PROPERTY_KEY => {
                self.y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
