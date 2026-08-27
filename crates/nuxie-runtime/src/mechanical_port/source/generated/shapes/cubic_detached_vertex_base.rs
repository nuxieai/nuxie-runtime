use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::cubic_detached_vertex::CubicDetachedVertex,
    shapes::cubic_vertex::CubicVertex,
};

pub trait CubicDetachedVertexBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn in_rotation_changed(&mut self) {}
    fn in_distance_changed(&mut self) {}
    fn out_rotation_changed(&mut self) {}
    fn out_distance_changed(&mut self) {}
}

pub struct CubicDetachedVertexBase {
    pub base: CubicVertex,
    in_rotation: f32,
    in_distance: f32,
    out_rotation: f32,
    out_distance: f32,
}

impl Default for CubicDetachedVertexBase {
    fn default() -> Self {
        Self {
            base: CubicVertex::default(),
            in_rotation: 0.0,
            in_distance: 0.0,
            out_rotation: 0.0,
            out_distance: 0.0,
        }
    }
}

impl CubicDetachedVertexBase {
    pub const TYPE_KEY: u16 = 6;
    pub const IN_ROTATION_PROPERTY_KEY: u16 = 84;
    pub const IN_DISTANCE_PROPERTY_KEY: u16 = 85;
    pub const OUT_ROTATION_PROPERTY_KEY: u16 = 86;
    pub const OUT_DISTANCE_PROPERTY_KEY: u16 = 87;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 36 | 14 | 107 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn in_rotation(&self) -> f32 {
        self.in_rotation
    }
    pub fn set_in_rotation(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicDetachedVertexBaseCallbacks,
    ) {
        if self.in_rotation == value {
            return;
        }
        self.in_rotation = value;
        callbacks.in_rotation_changed();
        callbacks.notify_property_changed(Self::IN_ROTATION_PROPERTY_KEY);
    }
    pub fn in_distance(&self) -> f32 {
        self.in_distance
    }
    pub fn set_in_distance(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicDetachedVertexBaseCallbacks,
    ) {
        if self.in_distance == value {
            return;
        }
        self.in_distance = value;
        callbacks.in_distance_changed();
        callbacks.notify_property_changed(Self::IN_DISTANCE_PROPERTY_KEY);
    }
    pub fn out_rotation(&self) -> f32 {
        self.out_rotation
    }
    pub fn set_out_rotation(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicDetachedVertexBaseCallbacks,
    ) {
        if self.out_rotation == value {
            return;
        }
        self.out_rotation = value;
        callbacks.out_rotation_changed();
        callbacks.notify_property_changed(Self::OUT_ROTATION_PROPERTY_KEY);
    }
    pub fn out_distance(&self) -> f32 {
        self.out_distance
    }
    pub fn set_out_distance(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicDetachedVertexBaseCallbacks,
    ) {
        if self.out_distance == value {
            return;
        }
        self.out_distance = value;
        callbacks.out_distance_changed();
        callbacks.notify_property_changed(Self::OUT_DISTANCE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl CubicDetachedVertexBaseCallbacks,
    ) -> CubicDetachedVertex {
        let mut cloned = CubicDetachedVertex::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl CubicDetachedVertexBaseCallbacks) {
        self.in_rotation = object.in_rotation;
        self.in_distance = object.in_distance;
        self.out_rotation = object.out_rotation;
        self.out_distance = object.out_distance;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl CubicDetachedVertexBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::IN_ROTATION_PROPERTY_KEY => {
                self.in_rotation = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::IN_DISTANCE_PROPERTY_KEY => {
                self.in_distance = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OUT_ROTATION_PROPERTY_KEY => {
                self.out_rotation = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OUT_DISTANCE_PROPERTY_KEY => {
                self.out_distance = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
