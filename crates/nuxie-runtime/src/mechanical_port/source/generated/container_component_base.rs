use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub struct ContainerComponentBase {
    pub base: Component,
}

impl Default for ContainerComponentBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
        }
    }
}

impl ContainerComponentBase {
    pub const TYPE_KEY: u16 = 11;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
