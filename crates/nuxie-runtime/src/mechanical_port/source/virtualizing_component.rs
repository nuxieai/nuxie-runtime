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
    fn realized_indices(&self, out: &mut Vec<i32>);
    fn clear_virtual_window(&mut self);
    fn add_to_virtual_window(&mut self, index: i32, visible: bool);
    fn set_virtualizable_cell(&mut self, index: i32, column: i32, row: i32);
    fn set_virtualizable_position(&mut self, index: i32, position: Vec2D);
}

/// addVirtualizable synchronously rebuilds the parent's layout children, which
/// includes this provider. Enter through its identity without a provider borrow.
pub fn add_virtualizable_handle(
    component: &crate::mechanical_port::source::core::CoreHandle,
    index: i32,
) -> bool {
    add_virtualizable_handle_with_scroll(component, index, None)
}

pub(crate) fn add_virtualizable_handle_with_scroll(
    component: &crate::source::core::CoreHandle,
    index: i32,
    active_scroll: Option<
        &crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint,
    >,
) -> bool {
    if component.core_type() != Some(ArtboardComponentList::TYPE_KEY) {
        return false;
    }
    ArtboardComponentList::add_virtualizable_with_scroll_occurrence(
        component,
        index,
        active_scroll,
    );
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

/// Source fromScrollChild: only a component-list provider is virtualizing.
pub fn from_scroll_child(
    child: Option<&crate::source::core::CoreHandle>,
) -> Option<crate::source::core::CoreHandle> {
    child
        .filter(|child| child.core_type() == Some(ArtboardComponentList::TYPE_KEY))
        .cloned()
}
