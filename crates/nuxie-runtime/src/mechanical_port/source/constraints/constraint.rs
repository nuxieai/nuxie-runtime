use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    component::ComponentDirt,
    core::{Core, CoreHandle, CoreObject},
    core_context::{CoreContext, StatusCode},
    generated::{
        component_base::ComponentBaseCallbacks,
        constraints::constraint_base::{ConstraintBase, ConstraintBaseCallbacks},
        core_registry::CoreCapabilities,
    },
    math::{mat2d::Mat2D, transform_components::TransformComponents},
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

    pub(crate) fn on_dirty_from_layout(
        owner: &CoreHandle,
        _dirt: ComponentDirt,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        // Release this Constraint's arena slot before dirtying its parent. In
        // C++ the parent can be the Layout object whose setter is already on
        // the stack; the active-owner path preserves that reentrant call.
        let parent = owner
            .with(|owner| {
                owner
                    .as_component()
                    .expect("Constraint inherits Component")
                    .parent_handle()
            })
            .flatten()
            .expect("Constraint parent was validated");
        TransformComponent::mark_transform_dirty_from_layout(&parent, active, active_handle);
    }

    pub fn handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.handle()
    }

    pub fn compose_keeping_anchor(component: &mut dyn CoreObject, composed: &TransformComponents) {
        let anchor = component.transform_component_local_anchor();
        let component = component
            .as_transform_component_mut()
            .expect("constraint TransformComponent");
        let world = component.mutable_world_transform();
        if anchor.x == 0.0 && anchor.y == 0.0 {
            *world = Mat2D::compose(composed);
            return;
        }
        let before = *world * anchor;
        let mut result = Mat2D::compose(composed);
        let after = result * anchor;
        result[4] += before.x - after.x;
        result[5] += before.y - after.y;
        *world = result;
    }

    pub fn land_anchor(component: &mut dyn CoreObject, strength: f32) {
        let anchor = component.transform_component_local_anchor();
        if anchor.x == 0.0 && anchor.y == 0.0 {
            return;
        }
        let world = component
            .as_transform_component_mut()
            .expect("constraint TransformComponent")
            .mutable_world_transform();
        world[4] -= (world[0] * anchor.x + world[2] * anchor.y) * strength;
        world[5] -= (world[1] * anchor.x + world[3] * anchor.y) * strength;
    }

    pub fn compose_landing_anchor(
        component: &mut dyn CoreObject,
        composed: &TransformComponents,
        strength: f32,
    ) {
        *component
            .as_transform_component_mut()
            .expect("constraint TransformComponent")
            .mutable_world_transform() = Mat2D::compose(composed);
        Self::land_anchor(component, strength);
    }

    pub fn offset_in_parent_frame(component: &TransformComponent, offset: &Mat2D) -> Mat2D {
        let parent_world = get_parent_world(component);
        let delta = (parent_world * *offset).translation() - parent_world.translation();
        let mut result = *component.world_transform();
        result[4] += delta.x;
        result[5] += delta.y;
        result
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
