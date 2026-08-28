use crate::mechanical_port::source::{core::CoreHandle, math::mat2d::Mat2D};

#[derive(Default)]
pub struct ConstrainableListState {
    pub list_constraints: Vec<CoreHandle>,
}

pub trait ConstrainableList {
    fn constrainable_list_state(&mut self) -> &mut ConstrainableListState;
    fn list_transform(&self) -> &Mat2D;
    fn list_item_transforms<'a>(&'a mut self, transforms: &mut Vec<&'a mut Mat2D>);

    fn add_list_constraint(&mut self, constraint: CoreHandle) {
        let constraints = &mut self.constrainable_list_state().list_constraints;
        assert!(!constraints.contains(&constraint));
        constraints.push(constraint);
    }
}
