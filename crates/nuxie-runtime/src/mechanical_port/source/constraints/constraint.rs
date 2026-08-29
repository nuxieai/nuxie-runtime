use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    component::ComponentDirt,
    core::{Core, CoreHandle},
    core_context::{CoreContext, StatusCode},
    generated::{
        component_base::ComponentBaseCallbacks,
        constraints::constraint_base::{ConstraintBase, ConstraintBaseCallbacks},
        core_registry::CoreCapabilities,
    },
    math::mat2d::Mat2D,
    transform_component::TransformComponent,
};

impl ComponentBaseCallbacks for Constraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl ConstraintBaseCallbacks for Constraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }

    fn strength_changed(&mut self) {
        Constraint::strength_changed(self);
    }
}

/// Handwritten C++ `Constraint` base retained as an embedded Rust base.
/// Concrete virtual `constrain` dispatch is supplied centrally by
/// `CoreCapabilities::constraint_apply` over occurrence handles.
#[derive(Default)]
pub struct Constraint {
    pub base: ConstraintBase,
}

impl Deref for Constraint {
    type Target = ConstraintBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for Constraint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Constraint {
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ConstraintBaseCallbacks) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut crate::mechanical_port::source::core::binary_reader::BinaryReader<'_>,
        callbacks: &mut impl ConstraintBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.base.on_added_dirty(context);
        let Some(constraint) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        let Some(parent) = self.base.base.parent_handle() else {
            return StatusCode::InvalidObject;
        };
        let added = parent
            .with_mut(|parent| {
                let Some(parent) = parent.as_transform_component_mut() else {
                    return false;
                };
                parent.add_constraint(constraint);
                true
            })
            .unwrap_or(false);
        if !added {
            return StatusCode::InvalidObject;
        }
        result
    }

    pub fn mark_constraint_dirty(&mut self) {
        let parent = self
            .base
            .base
            .parent_handle()
            .expect("Constraint parent was validated");
        TransformComponent::mark_transform_dirty_occurrence(&parent);
    }

    pub fn strength_changed(&mut self) {
        self.mark_constraint_dirty();
    }

    pub(crate) fn mark_constraint_dirty_occurrence(owner: &CoreHandle) {
        let parent = owner
            .with(|owner| {
                owner
                    .as_component()
                    .expect("Constraint component")
                    .parent_handle()
            })
            .flatten()
            .expect("Constraint parent was validated");
        TransformComponent::mark_transform_dirty_occurrence(&parent);
    }

    pub fn build_dependencies(&mut self) {
        self.base.base.build_dependencies();
        let dependent = self
            .base
            .base
            .base
            .base
            .handle()
            .expect("arena-owned Constraint");
        self.base
            .base
            .parent_handle()
            .and_then(|parent| parent.with_mut(|parent| parent.component_add_dependent(dependent)))
            .filter(|added| *added)
            .expect("Constraint parent component");
    }

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {
        self.mark_constraint_dirty();
    }

    pub(crate) fn on_dirty_from_shape(
        owner: &CoreHandle,
        _dirt: ComponentDirt,
        active_shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
    ) {
        // A path composer can dirty a constraint while its Shape is active.
        // Release the constraint before synchronously dirtying its parent:
        // that parent's dependents can lead back to the same Shape's paths.
        let parent = owner
            .with(|owner| {
                owner
                    .as_component()
                    .expect("Constraint inherits Component")
                    .parent_handle()
            })
            .flatten()
            .expect("Constraint parent was validated");
        TransformComponent::mark_transform_dirty_from_shape(&parent, active_shape);
    }

    pub fn handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.handle()
    }
}

pub fn get_parent_world(component: &TransformComponent) -> Mat2D {
    component
        .parent_handle()
        .and_then(|parent| {
            parent.with(|parent| {
                parent
                    .as_world_transform_component()
                    .map(|parent| *parent.world_transform())
            })
        })
        .flatten()
        .unwrap_or_default()
}
