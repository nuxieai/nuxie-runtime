use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    shapes::paint::group_effect::GroupEffect,
};

pub struct GroupEffectBase {
    pub base: ContainerComponent,
}

impl Default for GroupEffectBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
        }
    }
}

impl GroupEffectBase {
    pub const TYPE_KEY: u16 = 645;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> GroupEffect {
        let mut cloned = GroupEffect::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for GroupEffectBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for GroupEffectBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
