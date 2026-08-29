use crate::mechanical_port::source::viewmodel::viewmodel_component::ViewModelComponent;

use crate::mechanical_port::source::{core::Core, core::binary_reader::BinaryReader};

pub trait ViewModelComponentBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn name_changed(&mut self) {}
}

pub struct ViewModelComponentBase {
    pub base: Core,
    name: String,
}

impl Default for ViewModelComponentBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            name: "".to_owned(),
        }
    }
}

impl ViewModelComponentBase {
    pub const TYPE_KEY: u16 = 429;
    pub const NAME_PROPERTY_KEY: u16 = 557;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(
        &mut self,
        value: String,
        callbacks: &mut impl ViewModelComponentBaseCallbacks,
    ) {
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
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelComponentBaseCallbacks,
    ) -> ViewModelComponent {
        let mut cloned = ViewModelComponent::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ViewModelComponentBaseCallbacks) {
        self.name.clone_from(&object.name);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelComponentBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::NAME_PROPERTY_KEY => {
                self.name = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for ViewModelComponentBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelComponentBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
