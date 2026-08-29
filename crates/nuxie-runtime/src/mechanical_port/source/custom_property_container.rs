use crate::mechanical_port::source::{
    core::CoreHandle, generated::custom_property_base::CustomPropertyBase,
};

#[derive(Default)]
pub struct CustomPropertyContainerState {
    custom_properties: Vec<CoreHandle>,
}

pub trait CustomPropertyContainer {
    fn custom_property_container_state(&self) -> &CustomPropertyContainerState;
    fn custom_property_container_state_mut(&mut self) -> &mut CustomPropertyContainerState;

    fn container_children(&self) -> &[CoreHandle] {
        &[]
    }

    fn sync_custom_properties(&mut self) {
        let properties = self
            .container_children()
            .iter()
            .filter(|child| {
                child
                    .with(|child| {
                        crate::mechanical_port::source::core::CoreObject::is_type_of(
                            child,
                            CustomPropertyBase::TYPE_KEY,
                        )
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        self.custom_property_container_state_mut().custom_properties = properties;
    }

    fn add_property(&mut self, property: CoreHandle) {
        let properties = &mut self.custom_property_container_state_mut().custom_properties;
        if !properties.contains(&property) {
            properties.push(property);
        }
    }

    fn remove_property(&mut self, property: &CoreHandle) {
        let properties = &mut self.custom_property_container_state_mut().custom_properties;
        if let Some(index) = properties.iter().position(|item| item == property) {
            properties.remove(index);
        }
    }

    fn custom_properties(&self) -> &[CoreHandle] {
        &self.custom_property_container_state().custom_properties
    }
}
