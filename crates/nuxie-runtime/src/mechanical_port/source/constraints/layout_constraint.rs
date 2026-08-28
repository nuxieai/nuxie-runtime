use crate::mechanical_port::source::core::CoreHandle;

pub trait LayoutConstraint {
    fn constrain_layout_child(&mut self, child: CoreHandle);
    fn add_layout_child(&mut self, child: CoreHandle);
}
