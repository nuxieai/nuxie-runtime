use crate::mechanical_port::source::{core::CoreHandle, math::aabb::Aabb};
use std::ptr::NonNull;
pub trait TextInterface {
    fn mark_paint_dirty(&mut self);
    fn mark_shape_dirty(&mut self);
    fn local_bounds(&self) -> Aabb;
}
impl dyn TextInterface {
    pub fn from_core(component: Option<CoreHandle>) -> Option<NonNull<dyn TextInterface>> {
        let component = component?;
        match component.core_type() { crate::mechanical_port::source::generated::text::text_base::TextBase::TYPE_KEY => component.as_text_interface(), crate::mechanical_port::source::generated::text::text_input_base::TextInputBase::TYPE_KEY => component.as_text_input_interface(), _ => None }
    }
}
