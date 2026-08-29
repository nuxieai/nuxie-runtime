use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    constraints::{constraint::Constraint, transform_space_constraint::TransformSpaceConstraint},
    core::{Core, binary_reader::BinaryReader},
    generated::{
        component_base::ComponentBaseCallbacks,
        constraints::{
            constraint_base::ConstraintBaseCallbacks,
            targeted_constraint_base::TargetedConstraintBaseCallbacks,
            transform_component_constraint_base::{
                TransformComponentConstraintBase, TransformComponentConstraintBaseCallbacks,
            },
            transform_space_constraint_base::TransformSpaceConstraintBaseCallbacks,
        },
    },
    transform_space::TransformSpace,
};

#[derive(Default)]
pub struct TransformComponentConstraint {
    pub base: TransformComponentConstraintBase,
}

impl Deref for TransformComponentConstraint {
    type Target = TransformComponentConstraintBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TransformComponentConstraint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ComponentBaseCallbacks for TransformComponentConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl ConstraintBaseCallbacks for TransformComponentConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }

    fn strength_changed(&mut self) {
        Constraint::strength_changed(self);
    }
}

impl TargetedConstraintBaseCallbacks for TransformComponentConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformSpaceConstraintBaseCallbacks for TransformComponentConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformComponentConstraintBaseCallbacks for TransformComponentConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformComponentConstraint {
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }

    pub fn min_max_space(&self) -> TransformSpace {
        TransformSpace::from(self.base.min_max_space_value())
    }
}
