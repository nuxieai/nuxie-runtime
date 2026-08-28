use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::polygon::Polygon, shapes::star::Star,
};

pub trait StarBaseCallbacks:
    crate::mechanical_port::source::generated::shapes::polygon_base::PolygonBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn inner_radius_changed(&mut self) {}
}

pub struct StarBase {
    pub base: Polygon,
    inner_radius: f32,
}

impl Default for StarBase {
    fn default() -> Self {
        Self {
            base: Polygon::default(),
            inner_radius: 0.5,
        }
    }
}

impl StarBase {
    pub const TYPE_KEY: u16 = 52;
    pub const INNER_RADIUS_PROPERTY_KEY: u16 = 127;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(
            type_key,
            Self::TYPE_KEY | 51 | 15 | 12 | 2 | 38 | 91 | 11 | 10
        )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn inner_radius(&self) -> f32 {
        self.inner_radius
    }
    pub fn set_inner_radius(&mut self, value: f32, callbacks: &mut impl StarBaseCallbacks) {
        if !self.set_inner_radius_value(value) {
            return;
        }
        callbacks.inner_radius_changed();
        callbacks.notify_property_changed(Self::INNER_RADIUS_PROPERTY_KEY);
    }

    pub(crate) fn set_inner_radius_value(&mut self, value: f32) -> bool {
        if self.inner_radius == value {
            return false;
        }
        self.inner_radius = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl StarBaseCallbacks) -> Star {
        let mut cloned = Star::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StarBaseCallbacks) {
        self.inner_radius = object.inner_radius;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StarBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INNER_RADIUS_PROPERTY_KEY => {
                self.inner_radius = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for StarBase {
    type Target = Polygon;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StarBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
