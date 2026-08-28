use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, draw_target::DrawTarget,
};

pub trait DrawTargetBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn drawable_id_changed(&mut self) {}
    fn placement_value_changed(&mut self) {}
}

pub struct DrawTargetBase {
    pub base: Component,
    drawable_id: u32,
    placement_value: u32,
}

impl Default for DrawTargetBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            drawable_id: u32::MAX,
            placement_value: 0,
        }
    }
}

impl DrawTargetBase {
    pub const TYPE_KEY: u16 = 48;
    pub const DRAWABLE_ID_PROPERTY_KEY: u16 = 119;
    pub const PLACEMENT_VALUE_PROPERTY_KEY: u16 = 120;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn drawable_id(&self) -> u32 {
        self.drawable_id
    }
    pub fn set_drawable_id(&mut self, value: u32, callbacks: &mut impl DrawTargetBaseCallbacks) {
        if !self.set_drawable_id_value(value) {
            return;
        }
        callbacks.drawable_id_changed();
        callbacks.notify_property_changed(Self::DRAWABLE_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_drawable_id_value(&mut self, value: u32) -> bool {
        if self.drawable_id == value {
            return false;
        }
        self.drawable_id = value;
        true
    }
    pub fn placement_value(&self) -> u32 {
        self.placement_value
    }
    pub fn set_placement_value(
        &mut self,
        value: u32,
        callbacks: &mut impl DrawTargetBaseCallbacks,
    ) {
        if !self.set_placement_value_value(value) {
            return;
        }
        callbacks.placement_value_changed();
        callbacks.notify_property_changed(Self::PLACEMENT_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_placement_value_value(&mut self, value: u32) -> bool {
        if self.placement_value == value {
            return false;
        }
        self.placement_value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl DrawTargetBaseCallbacks) -> DrawTarget {
        let mut cloned = DrawTarget::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DrawTargetBaseCallbacks) {
        self.drawable_id = object.drawable_id;
        self.placement_value = object.placement_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DrawTargetBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::DRAWABLE_ID_PROPERTY_KEY => {
                self.drawable_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::PLACEMENT_VALUE_PROPERTY_KEY => {
                self.placement_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for DrawTargetBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DrawTargetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
