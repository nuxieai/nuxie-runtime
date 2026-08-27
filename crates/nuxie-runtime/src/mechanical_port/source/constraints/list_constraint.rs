use crate::mechanical_port::source::{
    component::Component,
    constraints::{
        constrainable_list::ConstrainableList,
        list_follow_path_constraint::{ListFollowPathConstraint, ListFollowPathConstraintBase},
    },
};

pub trait ListConstraint {
    fn constrain_list(&mut self, _child: &mut dyn ConstrainableList) {}
}

pub fn from(component: &mut Component) -> Option<&mut dyn ListConstraint> {
    match component.core_type() {
        ListFollowPathConstraintBase::TYPE_KEY => component
            .as_mut::<ListFollowPathConstraint>()
            .map(|constraint| constraint as &mut dyn ListConstraint),
        _ => None,
    }
}
