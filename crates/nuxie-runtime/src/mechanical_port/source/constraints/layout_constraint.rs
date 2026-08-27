use crate::mechanical_port::source::{
    constraints::constraint::Constraint,
    layout::layout_node_provider::LayoutNodeProvider,
};

pub trait LayoutConstraint {
    fn constrain_child(&mut self, _child: &mut dyn LayoutNodeProvider) {}
    fn add_layout_child(&mut self, _child: &mut dyn LayoutNodeProvider) {}
    fn constraint(&mut self) -> &mut dyn Constraint;
}
