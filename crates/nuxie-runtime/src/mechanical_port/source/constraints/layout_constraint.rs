use crate::mechanical_port::source::core::CoreHandle;

pub trait LayoutConstraint {
    fn constraint_handle(&self) -> CoreHandle;
    fn layout_child_constrainer(&self) -> fn(&CoreHandle, CoreHandle) -> bool;
    fn add_layout_child(&mut self, child: CoreHandle);
}
