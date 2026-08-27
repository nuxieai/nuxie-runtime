use crate::mechanical_port::source::{
    container_component::ContainerComponent, custom_property_group::CustomPropertyGroup,
};

pub struct CustomPropertyGroupBase {
    pub base: ContainerComponent,
}

impl Default for CustomPropertyGroupBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
        }
    }
}

impl CustomPropertyGroupBase {
    pub const TYPE_KEY: u16 = 548;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        self.base.copy(&object.base);
    }
    pub fn clone_into(&self) -> CustomPropertyGroup {
        let mut cloned = CustomPropertyGroup::default();
        cloned.base.copy(self);
        cloned
    }
}
