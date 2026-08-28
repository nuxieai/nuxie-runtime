use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    constraints::{constraint::Constraint, targeted_constraint::TargetedConstraint},
    core::{Core, binary_reader::BinaryReader},
    generated::{
        component_base::ComponentBaseCallbacks,
        constraints::{
            constraint_base::ConstraintBaseCallbacks,
            targeted_constraint_base::TargetedConstraintBaseCallbacks,
            transform_space_constraint_base::{
                TransformSpaceConstraintBase, TransformSpaceConstraintBaseCallbacks,
            },
        },
    },
    transform_space::TransformSpace,
};

#[derive(Default)]
pub struct TransformSpaceConstraint {
    pub base: TransformSpaceConstraintBase,
}

impl Deref for TransformSpaceConstraint {
    type Target = TransformSpaceConstraintBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TransformSpaceConstraint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ComponentBaseCallbacks for TransformSpaceConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl ConstraintBaseCallbacks for TransformSpaceConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }

    fn strength_changed(&mut self) {
        Constraint::strength_changed(self);
    }
}

impl TargetedConstraintBaseCallbacks for TransformSpaceConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformSpaceConstraintBaseCallbacks for TransformSpaceConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformSpaceConstraint {
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransformSpaceConstraintBaseCallbacks,
    ) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransformSpaceConstraintBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }

    pub fn source_space(&self) -> TransformSpace {
        TransformSpace::from(self.base.source_space_value())
    }

    pub fn dest_space(&self) -> TransformSpace {
        TransformSpace::from(self.base.dest_space_value())
    }
}
