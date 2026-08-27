use crate::mechanical_port::source::{
    generated::constraints::transform_component_constraint_base::TransformComponentConstraintBase,
    transform_space::TransformSpace,
};

pub struct TransformComponentConstraint {
    pub base: TransformComponentConstraintBase,
}

impl TransformComponentConstraint {
    pub fn min_max_space(&self) -> TransformSpace {
        TransformSpace::from(self.base.min_max_space_value())
    }
}
