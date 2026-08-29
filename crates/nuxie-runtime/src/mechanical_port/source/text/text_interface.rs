use crate::mechanical_port::source::{core::CoreHandle, math::aabb::Aabb};
pub trait TextInterface {
    fn mark_paint_dirty(&mut self);
    fn mark_shape_dirty(&mut self);
    fn local_bounds(&self) -> Aabb;
}
impl dyn TextInterface {
    pub fn from_core(component: Option<CoreHandle>) -> Option<CoreHandle> {
        let component = component?;
        component
            .with(|component| component.is_text_interface())
            .unwrap_or(false)
            .then_some(component)
    }
}
