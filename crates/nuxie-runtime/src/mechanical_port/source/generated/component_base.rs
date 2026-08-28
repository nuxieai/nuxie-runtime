use crate::mechanical_port::source::{core::binary_reader::BinaryReader, core::Core};

pub trait ComponentBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn name_changed(&mut self) {}
    fn parent_id_changed(&mut self) {}
}

pub struct ComponentBase {
    pub base: Core,
    name: String,
    parent_id: u32,
}

impl Default for ComponentBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            name: "".to_owned(),
            parent_id: 0,
        }
    }
}

impl ComponentBase {
    pub const TYPE_KEY: u16 = 10;
    pub const NAME_PROPERTY_KEY: u16 = 4;
    pub const PARENT_ID_PROPERTY_KEY: u16 = 5;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, value: String, callbacks: &mut impl ComponentBaseCallbacks) {
        if !self.set_name_value(value) {
            return;
        }
        callbacks.name_changed();
        callbacks.notify_property_changed(Self::NAME_PROPERTY_KEY);
    }

    pub(crate) fn set_name_value(&mut self, value: String) -> bool {
        if self.name == value {
            return false;
        }
        self.name = value;
        true
    }
    pub fn parent_id(&self) -> u32 {
        self.parent_id
    }
    pub fn set_parent_id(&mut self, value: u32, callbacks: &mut impl ComponentBaseCallbacks) {
        if !self.set_parent_id_value(value) {
            return;
        }
        callbacks.parent_id_changed();
        callbacks.notify_property_changed(Self::PARENT_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_parent_id_value(&mut self, value: u32) -> bool {
        if self.parent_id == value {
            return false;
        }
        self.parent_id = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ComponentBaseCallbacks) {
        self.name.clone_from(&object.name);
        self.parent_id = object.parent_id;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ComponentBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::NAME_PROPERTY_KEY => {
                self.name = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            Self::PARENT_ID_PROPERTY_KEY => {
                self.parent_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for ComponentBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ComponentBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
