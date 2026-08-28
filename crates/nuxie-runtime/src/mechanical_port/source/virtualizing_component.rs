use crate::mechanical_port::source::artboard_component_list::ArtboardComponentList;
use crate::mechanical_port::source::component::Component;
use crate::mechanical_port::source::math::vec2d::Vec2D;

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
    fn item(&mut self, index: i32) -> Option<&mut dyn Virtualizable>;
    fn size(&self) -> Vec2D;
    fn item_size(&self, index: i32) -> Vec2D;
    fn set_item_size(&mut self, size: Vec2D, index: i32);
    fn add_virtualizable(&mut self, index: i32);
    fn virtualizable_changed(&mut self);
    fn remove_virtualizable(&mut self, index: i32);
    fn set_visible_indices(&mut self, start: i32, end: i32);
    fn set_virtualizable_position(&mut self, index: i32, position: Vec2D);
}

pub fn from(component: &mut Component) -> Option<&mut dyn VirtualizingComponent> {
    if component.core_type() == ArtboardComponentList::TYPE_KEY {
        component
            .as_artboard_component_list_mut()
            .map(|component| component as &mut dyn VirtualizingComponent)
    } else {
        None
    }
}
