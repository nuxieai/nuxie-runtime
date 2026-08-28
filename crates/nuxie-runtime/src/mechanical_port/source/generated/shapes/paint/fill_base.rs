use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::paint::fill::Fill,
    shapes::paint::shape_paint::ShapePaint,
};

pub trait FillBaseCallbacks:
    crate::mechanical_port::source::generated::shapes::paint::shape_paint_base::ShapePaintBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn fill_rule_changed(&mut self) {}
}

pub struct FillBase {
    pub base: ShapePaint,
    fill_rule: u32,
}

impl Default for FillBase {
    fn default() -> Self {
        Self {
            base: ShapePaint::default(),
            fill_rule: 0,
        }
    }
}

impl FillBase {
    pub const TYPE_KEY: u16 = 20;
    pub const FILL_RULE_PROPERTY_KEY: u16 = 40;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 21 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn fill_rule(&self) -> u32 {
        self.fill_rule
    }
    pub fn set_fill_rule(&mut self, value: u32, callbacks: &mut impl FillBaseCallbacks) {
        if !self.set_fill_rule_value(value) {
            return;
        }
        callbacks.fill_rule_changed();
        callbacks.notify_property_changed(Self::FILL_RULE_PROPERTY_KEY);
    }

    pub(crate) fn set_fill_rule_value(&mut self, value: u32) -> bool {
        if self.fill_rule == value {
            return false;
        }
        self.fill_rule = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl FillBaseCallbacks) -> Fill {
        let mut cloned = Fill::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FillBaseCallbacks) {
        self.fill_rule = object.fill_rule;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FillBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FILL_RULE_PROPERTY_KEY => {
                self.fill_rule = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for FillBase {
    type Target = ShapePaint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FillBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
