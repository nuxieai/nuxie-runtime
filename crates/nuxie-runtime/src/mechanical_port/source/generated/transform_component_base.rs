use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, world_transform_component::WorldTransformComponent,
};

pub trait TransformComponentBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn rotation_changed(&mut self) {}
    fn scale_x_changed(&mut self) {}
    fn scale_y_changed(&mut self) {}
}

pub struct TransformComponentBase {
    pub base: WorldTransformComponent,
    rotation: f32,
    scale_x: f32,
    scale_y: f32,
}

impl Default for TransformComponentBase {
    fn default() -> Self {
        Self {
            base: WorldTransformComponent::default(),
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

impl TransformComponentBase {
    pub const TYPE_KEY: u16 = 38;
    pub const ROTATION_PROPERTY_KEY: u16 = 15;
    pub const SCALE_X_PROPERTY_KEY: u16 = 16;
    pub const SCALE_Y_PROPERTY_KEY: u16 = 17;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    pub fn set_rotation(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentBaseCallbacks,
    ) {
        if self.rotation == value {
            return;
        }
        self.rotation = value;
        callbacks.rotation_changed();
        callbacks.notify_property_changed(Self::ROTATION_PROPERTY_KEY);
    }
    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }
    pub fn set_scale_x(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentBaseCallbacks,
    ) {
        if self.scale_x == value {
            return;
        }
        self.scale_x = value;
        callbacks.scale_x_changed();
        callbacks.notify_property_changed(Self::SCALE_X_PROPERTY_KEY);
    }
    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }
    pub fn set_scale_y(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentBaseCallbacks,
    ) {
        if self.scale_y == value {
            return;
        }
        self.scale_y = value;
        callbacks.scale_y_changed();
        callbacks.notify_property_changed(Self::SCALE_Y_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TransformComponentBaseCallbacks) {
        self.rotation = object.rotation;
        self.scale_x = object.scale_x;
        self.scale_y = object.scale_y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransformComponentBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ROTATION_PROPERTY_KEY => {
                self.rotation = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::SCALE_X_PROPERTY_KEY => {
                self.scale_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::SCALE_Y_PROPERTY_KEY => {
                self.scale_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
