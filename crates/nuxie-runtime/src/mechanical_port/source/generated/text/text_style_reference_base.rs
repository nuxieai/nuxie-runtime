use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
};

pub struct TextStyleReferenceBase {
    pub base: ContainerComponent,
}

impl Default for TextStyleReferenceBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
        }
    }
}

impl TextStyleReferenceBase {
    pub const TYPE_KEY: u16 = 1100;

    pub fn is_type_of(type_key: u16) -> bool {
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for TextStyleReferenceBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleReferenceBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
