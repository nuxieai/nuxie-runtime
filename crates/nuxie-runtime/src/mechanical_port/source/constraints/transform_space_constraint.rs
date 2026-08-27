use crate::mechanical_port::source::{
    generated::constraints::transform_space_constraint_base::TransformSpaceConstraintBase,
    transform_space::TransformSpace,
};

pub struct TransformSpaceConstraint {
    pub base: TransformSpaceConstraintBase,
}

impl TransformSpaceConstraint {
    pub fn source_space(&self) -> TransformSpace {
        TransformSpace::from(self.base.source_space_value())
    }

    pub fn dest_space(&self) -> TransformSpace {
        TransformSpace::from(self.base.dest_space_value())
    }
}
