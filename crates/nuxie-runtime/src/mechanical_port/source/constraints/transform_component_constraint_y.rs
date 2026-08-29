use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    constraints::{
        constraint::Constraint, transform_component_constraint::TransformComponentConstraint,
    },
    core::{Core, binary_reader::BinaryReader},
    generated::{
        component_base::ComponentBaseCallbacks,
        constraints::{
            constraint_base::ConstraintBaseCallbacks,
            targeted_constraint_base::TargetedConstraintBaseCallbacks,
            transform_component_constraint_base::TransformComponentConstraintBaseCallbacks,
            transform_component_constraint_y_base::{
                TransformComponentConstraintYBase, TransformComponentConstraintYBaseCallbacks,
            },
            transform_space_constraint_base::TransformSpaceConstraintBaseCallbacks,
        },
    },
};

#[derive(Default)]
pub struct TransformComponentConstraintY {
    pub base: TransformComponentConstraintYBase,
}

impl Deref for TransformComponentConstraintY {
    type Target = TransformComponentConstraintYBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TransformComponentConstraintY {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ComponentBaseCallbacks for TransformComponentConstraintY {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl ConstraintBaseCallbacks for TransformComponentConstraintY {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }

    fn strength_changed(&mut self) {
        Constraint::strength_changed(self);
    }
}

impl TargetedConstraintBaseCallbacks for TransformComponentConstraintY {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformSpaceConstraintBaseCallbacks for TransformComponentConstraintY {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformComponentConstraintBaseCallbacks for TransformComponentConstraintY {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformComponentConstraintYBaseCallbacks for TransformComponentConstraintY {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl TransformComponentConstraintY {
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }
}
