use crate::mechanical_port::source::{
    artboard_component_list::ArtboardComponentList,
    core::CoreObject,
    custom_property_trigger::CustomPropertyTrigger,
    generated::{
        artboard_component_list_base::ArtboardComponentListBase,
        custom_property_trigger_base::CustomPropertyTriggerBase,
        nested_artboard_base::NestedArtboardBase,
        nested_artboard_layout_base::NestedArtboardLayoutBase,
        nested_artboard_leaf_base::NestedArtboardLeafBase,
    },
};

pub trait ResettingComponent {
    fn reset(&mut self);
}

pub fn from(component: &mut dyn CoreObject) -> Option<&mut dyn ResettingComponent> {
    match component.core_type() {
        NestedArtboardLeafBase::TYPE_KEY
        | NestedArtboardLayoutBase::TYPE_KEY
        | NestedArtboardBase::TYPE_KEY => component
            .as_nested_artboard_mut()
            .map(|component| component as &mut dyn ResettingComponent),
        ArtboardComponentListBase::TYPE_KEY => component
            .as_artboard_component_list_mut()
            .map(|component: &mut ArtboardComponentList| component as &mut dyn ResettingComponent),
        CustomPropertyTriggerBase::TYPE_KEY => component
            .as_any_mut()
            .downcast_mut::<CustomPropertyTrigger>()
            .map(|component: &mut CustomPropertyTrigger| component as &mut dyn ResettingComponent),
        _ => None,
    }
}
