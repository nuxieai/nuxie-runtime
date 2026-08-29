use crate::mechanical_port::source::{
    custom_property_container::{CustomPropertyContainer, CustomPropertyContainerState},
    generated::custom_property_group_base::CustomPropertyGroupBase,
};

#[derive(Default)]
pub struct CustomPropertyGroup {
    pub base: CustomPropertyGroupBase,
    pub container: CustomPropertyContainerState,
}

impl CustomPropertyContainer for CustomPropertyGroup {
    fn custom_property_container_state(&self) -> &CustomPropertyContainerState {
        &self.container
    }

    fn custom_property_container_state_mut(&mut self) -> &mut CustomPropertyContainerState {
        &mut self.container
    }
}

impl std::ops::Deref for CustomPropertyGroup {
    type Target = CustomPropertyGroupBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
