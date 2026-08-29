use crate::mechanical_port::source::constraints::constrainable_list::ConstrainableList;

pub trait ListConstraint {
    fn constrain_list(&mut self, _child: &mut dyn ConstrainableList) {}
}
