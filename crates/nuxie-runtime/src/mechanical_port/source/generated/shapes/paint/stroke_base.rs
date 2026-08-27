use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::paint::shape_paint::ShapePaint,
    shapes::paint::stroke::Stroke,
};

pub trait StrokeBaseCallbacks {
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
        if self.thickness == value {
            return;
        }
        self.thickness = value;
        callbacks.thickness_changed();
        callbacks.notify_property_changed(Self::THICKNESS_PROPERTY_KEY);
    }
    pub fn cap(&self) -> u32 {
        self.cap
    }
    pub fn set_cap(&mut self, value: u32, callbacks: &mut impl StrokeBaseCallbacks) {
        if self.cap == value {
            return;
        }
        self.cap = value;
        callbacks.cap_changed();
        callbacks.notify_property_changed(Self::CAP_PROPERTY_KEY);
    }
    pub fn join(&self) -> u32 {
        self.join
    }
    pub fn set_join(&mut self, value: u32, callbacks: &mut impl StrokeBaseCallbacks) {
        if self.join == value {
            return;
        }
        self.join = value;
        callbacks.join_changed();
        callbacks.notify_property_changed(Self::JOIN_PROPERTY_KEY);
    }
    pub fn transform_affects_stroke(&self) -> bool {
        self.transform_affects_stroke
    }
    pub fn set_transform_affects_stroke(
        &mut self,
        value: bool,
        callbacks: &mut impl StrokeBaseCallbacks,
    ) {
        if self.transform_affects_stroke == value {
            return;
        }
        self.transform_affects_stroke = value;
        callbacks.transform_affects_stroke_changed();
        callbacks.notify_property_changed(Self::TRANSFORM_AFFECTS_STROKE_PROPERTY_KEY);
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
