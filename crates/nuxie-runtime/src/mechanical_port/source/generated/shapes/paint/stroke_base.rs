use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::paint::shape_paint::ShapePaint,
    shapes::paint::stroke::Stroke,
};

pub trait StrokeBaseCallbacks:
    crate::mechanical_port::source::generated::shapes::paint::shape_paint_base::ShapePaintBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn thickness_changed(&mut self) {}
    fn cap_changed(&mut self) {}
    fn join_changed(&mut self) {}
    fn transform_affects_stroke_changed(&mut self) {}
}

pub struct StrokeBase {
    pub base: ShapePaint,
    thickness: f32,
    cap: u32,
    join: u32,
    transform_affects_stroke: bool,
}

impl Default for StrokeBase {
    fn default() -> Self {
        Self {
            base: ShapePaint::default(),
            thickness: 1.0,
            cap: 0,
            join: 0,
            transform_affects_stroke: true,
        }
    }
}

impl StrokeBase {
    pub const TYPE_KEY: u16 = 24;
    pub const THICKNESS_PROPERTY_KEY: u16 = 47;
    pub const CAP_PROPERTY_KEY: u16 = 48;
    pub const JOIN_PROPERTY_KEY: u16 = 49;
    pub const TRANSFORM_AFFECTS_STROKE_PROPERTY_KEY: u16 = 50;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 21 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn thickness(&self) -> f32 {
        self.thickness
    }
    pub fn set_thickness(&mut self, value: f32, callbacks: &mut impl StrokeBaseCallbacks) {
        if !self.set_thickness_value(value) {
            return;
        }
        callbacks.thickness_changed();
        StrokeBaseCallbacks::notify_property_changed(callbacks, Self::THICKNESS_PROPERTY_KEY);
    }

    pub(crate) fn set_thickness_value(&mut self, value: f32) -> bool {
        if self.thickness == value {
            return false;
        }
        self.thickness = value;
        true
    }
    pub fn cap(&self) -> u32 {
        self.cap
    }
    pub fn set_cap(&mut self, value: u32, callbacks: &mut impl StrokeBaseCallbacks) {
        if !self.set_cap_value(value) {
            return;
        }
        callbacks.cap_changed();
        StrokeBaseCallbacks::notify_property_changed(callbacks, Self::CAP_PROPERTY_KEY);
    }

    pub(crate) fn set_cap_value(&mut self, value: u32) -> bool {
        if self.cap == value {
            return false;
        }
        self.cap = value;
        true
    }
    pub fn join(&self) -> u32 {
        self.join
    }
    pub fn set_join(&mut self, value: u32, callbacks: &mut impl StrokeBaseCallbacks) {
        if !self.set_join_value(value) {
            return;
        }
        callbacks.join_changed();
        StrokeBaseCallbacks::notify_property_changed(callbacks, Self::JOIN_PROPERTY_KEY);
    }

    pub(crate) fn set_join_value(&mut self, value: u32) -> bool {
        if self.join == value {
            return false;
        }
        self.join = value;
        true
    }
    pub fn transform_affects_stroke(&self) -> bool {
        self.transform_affects_stroke
    }
    pub fn set_transform_affects_stroke(
        &mut self,
        value: bool,
        callbacks: &mut impl StrokeBaseCallbacks,
    ) {
        if !self.set_transform_affects_stroke_value(value) {
            return;
        }
        callbacks.transform_affects_stroke_changed();
        StrokeBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRANSFORM_AFFECTS_STROKE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_transform_affects_stroke_value(&mut self, value: bool) -> bool {
        if self.transform_affects_stroke == value {
            return false;
        }
        self.transform_affects_stroke = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl StrokeBaseCallbacks) -> Stroke {
        let mut cloned = Stroke::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StrokeBaseCallbacks) {
        self.thickness = object.thickness;
        self.cap = object.cap;
        self.join = object.join;
        self.transform_affects_stroke = object.transform_affects_stroke;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StrokeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::THICKNESS_PROPERTY_KEY => {
                self.thickness = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CAP_PROPERTY_KEY => {
                self.cap = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::JOIN_PROPERTY_KEY => {
                self.join = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::TRANSFORM_AFFECTS_STROKE_PROPERTY_KEY => {
                self.transform_affects_stroke = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for StrokeBase {
    type Target = ShapePaint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StrokeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
