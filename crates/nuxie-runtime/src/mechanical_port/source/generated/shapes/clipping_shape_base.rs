use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, shapes::clipping_shape::ClippingShape,
};

pub trait ClippingShapeBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn source_id_changed(&mut self) {}
    fn fill_rule_changed(&mut self) {}
    fn is_visible_changed(&mut self) {}
}

pub struct ClippingShapeBase {
    pub base: Component,
    source_id: u32,
    fill_rule: u32,
    is_visible: bool,
}

impl Default for ClippingShapeBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            source_id: u32::MAX,
            fill_rule: 0,
            is_visible: true,
        }
    }
}

impl ClippingShapeBase {
    pub const TYPE_KEY: u16 = 42;
    pub const SOURCE_ID_PROPERTY_KEY: u16 = 92;
    pub const FILL_RULE_PROPERTY_KEY: u16 = 93;
    pub const IS_VISIBLE_PROPERTY_KEY: u16 = 94;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn source_id(&self) -> u32 {
        self.source_id
    }
    pub fn set_source_id(&mut self, value: u32, callbacks: &mut impl ClippingShapeBaseCallbacks) {
        if self.source_id == value {
            return;
        }
        self.source_id = value;
        callbacks.source_id_changed();
        callbacks.notify_property_changed(Self::SOURCE_ID_PROPERTY_KEY);
    }
    pub fn fill_rule(&self) -> u32 {
        self.fill_rule
    }
    pub fn set_fill_rule(&mut self, value: u32, callbacks: &mut impl ClippingShapeBaseCallbacks) {
        if self.fill_rule == value {
            return;
        }
        self.fill_rule = value;
        callbacks.fill_rule_changed();
        callbacks.notify_property_changed(Self::FILL_RULE_PROPERTY_KEY);
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn set_is_visible(&mut self, value: bool, callbacks: &mut impl ClippingShapeBaseCallbacks) {
        if self.is_visible == value {
            return;
        }
        self.is_visible = value;
        callbacks.is_visible_changed();
        callbacks.notify_property_changed(Self::IS_VISIBLE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl ClippingShapeBaseCallbacks) -> ClippingShape {
        let mut cloned = ClippingShape::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ClippingShapeBaseCallbacks) {
        self.source_id = object.source_id;
        self.fill_rule = object.fill_rule;
        self.is_visible = object.is_visible;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ClippingShapeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SOURCE_ID_PROPERTY_KEY => {
                self.source_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::FILL_RULE_PROPERTY_KEY => {
                self.fill_rule = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::IS_VISIBLE_PROPERTY_KEY => {
                self.is_visible = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
