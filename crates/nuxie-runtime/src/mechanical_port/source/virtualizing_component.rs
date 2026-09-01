use crate::mechanical_port::source::component::Component;
use crate::mechanical_port::source::math::vec2d::Vec2D;
use crate::mechanical_port::source::{
    artboard::RuntimeArtboardInstanceHandle, artboard_component_list::ArtboardComponentList,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualizedDirection {
    Horizontal,
    Vertical,
}

pub trait Virtualizable {
    fn virtualizable_component(&mut self) -> &mut Component;
    fn layout_x(&self) -> f32;
    fn layout_y(&self) -> f32;
}

pub trait VirtualizingComponent {
    fn virtualization_enabled(&self) -> bool;
    fn item_count(&self) -> i32;
    fn item(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle>;
    fn size(&self) -> Vec2D;
    fn item_size(&self, index: i32) -> Vec2D;
    fn set_item_size(&mut self, size: Vec2D, index: i32);
    fn virtualizable_changed(&mut self);
    fn remove_virtualizable(&mut self, index: i32);
    /// Items on screen. Only these report their measured size back, otherwise
    /// realizing an off screen item would change the sizes the virtualizer
    /// sums to pick this very range.
    fn set_visible_indices(&mut self, start: i32, end: i32);
    /// Items that exist: the visible range plus the buffer on either side.
    /// These are the ones that get drawn.
    fn set_realized_indices(&mut self, start: i32, end: i32);
    fn set_virtualizable_position(&mut self, index: i32, position: Vec2D);
}

/// addVirtualizable synchronously rebuilds the parent's layout children, which
/// includes this provider. Enter through its identity without a provider borrow.
pub fn add_virtualizable_handle(
    component: &crate::mechanical_port::source::core::CoreHandle,
    index: i32,
) -> bool {
    if component.core_type() != Some(ArtboardComponentList::TYPE_KEY) {
        return false;
    }
    ArtboardComponentList::add_virtualizable_occurrence(component, index);
    true
}

pub fn from(
    component: &mut dyn crate::mechanical_port::source::core::CoreObject,
) -> Option<&mut dyn VirtualizingComponent> {
    if component.core_type() == ArtboardComponentList::TYPE_KEY {
        component
            .as_any_mut()
            .downcast_mut::<ArtboardComponentList>()
            .map(|component| component as &mut dyn VirtualizingComponent)
    } else {
        None
    }
}
