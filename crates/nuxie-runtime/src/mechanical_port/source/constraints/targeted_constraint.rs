use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    core::{Core, CoreHandle},
    core_context::{CoreContext, StatusCode},
    generated::{
        component_base::ComponentBaseCallbacks,
        constraints::constraint_base::ConstraintBaseCallbacks,
        constraints::targeted_constraint_base::{
            TargetedConstraintBase, TargetedConstraintBaseCallbacks,
        },
        core_registry::CoreCapabilities,
    },
};

impl ComponentBaseCallbacks for TargetedConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl ConstraintBaseCallbacks for TargetedConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }

    fn strength_changed(&mut self) {
        crate::mechanical_port::source::constraints::constraint::Constraint::strength_changed(self);
    }
}

impl TargetedConstraintBaseCallbacks for TargetedConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

#[derive(Default)]
pub struct TargetedConstraint {
    pub base: TargetedConstraintBase,
    target: Option<CoreHandle>,
}

impl Deref for TargetedConstraint {
    type Target = TargetedConstraintBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TargetedConstraint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TargetedConstraint {
    pub fn new(base: TargetedConstraintBase) -> Self {
        Self { base, target: None }
    }

    pub fn requires_target(&self) -> bool {
        true
    }

    pub fn target(&self) -> Option<CoreHandle> {
        self.target.clone()
    }

    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TargetedConstraintBaseCallbacks) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut crate::mechanical_port::source::core::binary_reader::BinaryReader<'_>,
        callbacks: &mut impl TargetedConstraintBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }

    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        self.validate_with_requirement(context, true)
    }

    pub fn validate_with_requirement(
        &mut self,
        context: &mut dyn CoreContext,
        requires_target: bool,
    ) -> bool {
        if !self.base.validate(context) {
            return false;
        }
        let core_object = context.resolve(self.base.target_id());
        if core_object.as_ref().is_some_and(|object| {
            object
                .with(|object| object.as_transform_component().is_none())
                .unwrap_or(true)
        }) {
            return false;
        }
        !requires_target || core_object.is_some()
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.on_added_dirty_with_requirement(context, true)
    }

    pub fn on_added_dirty_with_requirement(
        &mut self,
        context: &mut dyn CoreContext,
        requires_target: bool,
    ) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let core_object = context.resolve(self.base.target_id());
        if requires_target && core_object.is_none() {
            return StatusCode::MissingObject;
        }
        self.target = core_object;
        StatusCode::Ok
    }

    pub fn build_dependencies(&mut self) {
        if let (Some(target), Some(parent)) = (&self.target, self.base.parent_handle()) {
            target
                .with_mut(|target| target.component_add_dependent(parent))
                .filter(|added| *added)
                .expect("validated TargetedConstraint target is a TransformComponent");
        }
    }
}
