use crate::mechanical_port::source::{component::Component, custom_property::CustomProperty};

#[derive(Default)]
pub struct CustomPropertyContainerState {
    custom_properties: Vec<*mut CustomProperty>,
}

pub trait CustomPropertyContainer {
    fn custom_property_container_state(&self) -> &CustomPropertyContainerState;
    fn custom_property_container_state_mut(&mut self) -> &mut CustomPropertyContainerState;

    fn container_children(&self) -> &[*mut Component] {
        &[]
    }

    fn sync_custom_properties(&mut self) {
        let properties = self
            .container_children()
            .iter()
            .filter_map(|child| unsafe { child.as_mut() })
            .filter_map(Component::as_custom_property_mut)
            .map(|property| property as *mut CustomProperty)
            .collect();
        self.custom_property_container_state_mut().custom_properties = properties;
    }

    fn add_property(&mut self, property: *mut CustomProperty) {
        let properties = &mut self.custom_property_container_state_mut().custom_properties;
        if !properties.contains(&property) {
            properties.push(property);
        }
    }

    fn remove_property(&mut self, property: *mut CustomProperty) {
        let properties = &mut self.custom_property_container_state_mut().custom_properties;
        if let Some(index) = properties.iter().position(|item| *item == property) {
            properties.remove(index);
        }
    }

    fn custom_properties(&self) -> &[*mut CustomProperty] {
        &self.custom_property_container_state().custom_properties
    }
}
