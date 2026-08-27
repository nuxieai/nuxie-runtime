use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::parametric_path::ParametricPath,
    shapes::polygon::Polygon,
};

pub trait PolygonBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn points_changed(&mut self) {}
    fn corner_radius_changed(&mut self) {}
}

pub struct PolygonBase {
    pub base: ParametricPath,
    points: u32,
    corner_radius: f32,
}

impl Default for PolygonBase {
    fn default() -> Self {
        Self {
            base: ParametricPath::default(),
            points: 5,
            corner_radius: 0.0,
        }
    }
}

impl PolygonBase {
    pub const TYPE_KEY: u16 = 51;
    pub const POINTS_PROPERTY_KEY: u16 = 125;
    pub const CORNER_RADIUS_PROPERTY_KEY: u16 = 126;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 15 | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn points(&self) -> u32 {
        self.points
    }
    pub fn set_points(&mut self, value: u32, callbacks: &mut impl PolygonBaseCallbacks) {
        if self.points == value {
            return;
        }
        self.points = value;
        callbacks.points_changed();
        callbacks.notify_property_changed(Self::POINTS_PROPERTY_KEY);
    }
    pub fn corner_radius(&self) -> f32 {
        self.corner_radius
    }
    pub fn set_corner_radius(&mut self, value: f32, callbacks: &mut impl PolygonBaseCallbacks) {
        if self.corner_radius == value {
            return;
        }
        self.corner_radius = value;
        callbacks.corner_radius_changed();
        callbacks.notify_property_changed(Self::CORNER_RADIUS_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl PolygonBaseCallbacks) -> Polygon {
        let mut cloned = Polygon::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl PolygonBaseCallbacks) {
        self.points = object.points;
        self.corner_radius = object.corner_radius;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl PolygonBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::POINTS_PROPERTY_KEY => {
                self.points = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_PROPERTY_KEY => {
                self.corner_radius = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
