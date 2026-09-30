//! Mechanical translation of the pinned upstream generated ScriptedTransitionBase.
use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_id_type::CoreIdType},
    scripted::scripted_drawable::ScriptedDrawable,
    scripted::scripted_transition::ScriptedTransition,
};

pub trait ScriptedTransitionBaseCallbacks: crate::mechanical_port::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn active_component_id_changed(&mut self) {}
    fn list_source_changed(&mut self) {}
}

pub struct ScriptedTransitionBase {
    pub base: ScriptedDrawable,
    active_component_id: u32,
    list_source: u32,
}
impl Default for ScriptedTransitionBase {
    fn default() -> Self {
        Self {
            base: ScriptedDrawable::default(),
            active_component_id: 0,
            list_source: u32::MAX,
        }
    }
}
impl ScriptedTransitionBase {
    pub const TYPE_KEY: u16 = 110;
    pub const ACTIVE_COMPONENT_ID_PROPERTY_KEY: u16 = 273;
    pub const LIST_SOURCE_PROPERTY_KEY: u16 = 414;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 603 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn active_component_id(&self) -> u32 {
        self.active_component_id
    }
    pub fn set_active_component_id<C: ScriptedTransitionBaseCallbacks>(
        &mut self,
        value: u32,
        callbacks: &mut C,
    ) {
        if !self.set_active_component_id_value(value) {
            return;
        }
        callbacks.active_component_id_changed();
        ScriptedTransitionBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ACTIVE_COMPONENT_ID_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_active_component_id_value(&mut self, value: u32) -> bool {
        if self.active_component_id == value {
            return false;
        }
        self.active_component_id = value;
        true
    }
    pub fn list_source(&self) -> u32 {
        self.list_source
    }
    pub fn set_list_source<C: ScriptedTransitionBaseCallbacks>(
        &mut self,
        value: u32,
        callbacks: &mut C,
    ) {
        if !self.set_list_source_value(value) {
            return;
        }
        callbacks.list_source_changed();
        ScriptedTransitionBaseCallbacks::notify_property_changed(
            callbacks,
            Self::LIST_SOURCE_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_list_source_value(&mut self, value: u32) -> bool {
        if self.list_source == value {
            return false;
        }
        self.list_source = value;
        true
    }
    pub fn clone_into<C: ScriptedTransitionBaseCallbacks>(
        &self,
        callbacks: &mut C,
    ) -> ScriptedTransition {
        let mut cloned = ScriptedTransition::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy<C: ScriptedTransitionBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.active_component_id = object.active_component_id;
        self.list_source = object.list_source;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize<C: ScriptedTransitionBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::ACTIVE_COMPONENT_ID_PROPERTY_KEY => {
                self.active_component_id = CoreIdType::runtime_deserialize(reader);
                true
            }
            Self::LIST_SOURCE_PROPERTY_KEY => {
                self.list_source = CoreIdType::runtime_deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
impl std::ops::Deref for ScriptedTransitionBase {
    type Target = ScriptedDrawable;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ScriptedTransitionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
