use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, shapes::clipping_shape::ClippingShape,
};

pub trait ClippingShapeBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
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
        if !self.set_source_id_value(value) {
            return;
        }
        callbacks.source_id_changed();
        ClippingShapeBaseCallbacks::notify_property_changed(
            callbacks,
            Self::SOURCE_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_source_id_value(&mut self, value: u32) -> bool {
        if self.source_id == value {
            return false;
        }
        self.source_id = value;
        true
    }
    pub fn fill_rule(&self) -> u32 {
        self.fill_rule
    }
    pub fn set_fill_rule(&mut self, value: u32, callbacks: &mut impl ClippingShapeBaseCallbacks) {
        if !self.set_fill_rule_value(value) {
            return;
        }
        callbacks.fill_rule_changed();
        ClippingShapeBaseCallbacks::notify_property_changed(
            callbacks,
            Self::FILL_RULE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_fill_rule_value(&mut self, value: u32) -> bool {
        if self.fill_rule == value {
            return false;
        }
        self.fill_rule = value;
        true
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn set_is_visible(&mut self, value: bool, callbacks: &mut impl ClippingShapeBaseCallbacks) {
        if !self.set_is_visible_value(value) {
            return;
        }
        callbacks.is_visible_changed();
        ClippingShapeBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IS_VISIBLE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_is_visible_value(&mut self, value: bool) -> bool {
        if self.is_visible == value {
            return false;
        }
        self.is_visible = value;
        true
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

impl std::ops::Deref for ClippingShapeBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ClippingShapeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
