use crate::mechanical_port::source::{
    artboard::NestedArtboard,
    artboard_component_list::ArtboardComponentList,
    component::Component,
    custom_property_trigger::CustomPropertyTrigger,
    generated::{
        artboard_component_list_base::ArtboardComponentListBase,
        custom_property_trigger_base::CustomPropertyTriggerBase,
    },
    nested_artboard_layout::NestedArtboardLayout,
    nested_artboard_leaf::NestedArtboardLeaf,
};

pub trait ResettingComponent {
    fn reset(&mut self);
}

pub fn from(component: &mut Component) -> Option<&mut dyn ResettingComponent> {
    match component.core_type() {
        NestedArtboardLeaf::TYPE_KEY
        | NestedArtboardLayout::TYPE_KEY
        | NestedArtboard::TYPE_KEY => component
            .as_nested_artboard_mut()
            .map(|component| component as &mut dyn ResettingComponent),
        ArtboardComponentListBase::TYPE_KEY => component
            .as_artboard_component_list_mut()
            .map(|component: &mut ArtboardComponentList| component as &mut dyn ResettingComponent),
        CustomPropertyTriggerBase::TYPE_KEY => component
            .as_custom_property_trigger_mut()
            .map(|component: &mut CustomPropertyTrigger| component as &mut dyn ResettingComponent),
        _ => None,
    }
}
