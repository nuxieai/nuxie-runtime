use crate::mechanical_port::source::{
    artboard_component_list::{ArtboardComponentList, ArtboardComponentListBase},
    component::Component,
    constraints::list_constraint::ListConstraint,
    math::mat2d::Mat2D,
};

#[derive(Default)]
pub struct ConstrainableListState {
    pub list_constraints: Vec<*mut dyn ListConstraint>,
}

pub trait ConstrainableList {
    fn constrainable_list_state(&mut self) -> &mut ConstrainableListState;
    fn list_transform(&self) -> &Mat2D;
    fn list_item_transforms<'a>(&'a mut self, transforms: &mut Vec<&'a mut Mat2D>);

    fn add_list_constraint(&mut self, constraint: *mut dyn ListConstraint) {
        let constraints = &mut self.constrainable_list_state().list_constraints;
        assert!(!constraints.contains(&constraint));
        constraints.push(constraint);
    }
}

pub fn from(component: &mut Component) -> Option<&mut dyn ConstrainableList> {
    match component.core_type() {
        ArtboardComponentListBase::TYPE_KEY => component
            .as_mut::<ArtboardComponentList>()
            .map(|list| list as &mut dyn ConstrainableList),
        _ => None,
    }
}
