use std::cell::Cell;
use std::sync::OnceLock;

use super::ArtboardInstance;
use crate::components::{Mat2D, TransformProperty};
use crate::properties::cached_property_key_for_name;

pub(crate) fn x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Node", "x")
}

pub(crate) fn y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Node", "y")
}

/// Direct `Node::xChanged` / `Node::yChanged` dispatch inherited by concrete
/// Node occurrences. RootBone owns distinct generated property keys and is
/// routed through its focused owner instead.
pub(crate) fn apply_position_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> bool {
    if ![x_property_key(), y_property_key()].contains(&Some(property_key)) {
        return false;
    }
    let Some(handle) = artboard.component_handle(local_id) else {
        return false;
    };
    artboard.mark_transform_dirty_handle(handle);
    true
}

pub(crate) fn is_position_property(property: TransformProperty) -> bool {
    matches!(property, TransformProperty::X | TransformProperty::Y)
}

/// Runtime-only state owned by the concrete C++ `Node` subobject.
///
/// This is deliberately distinct from the authored local transform:
/// `Node::m_LocalTransform` is a lazy query cache derived from the settled
/// world transform after constraints (`include/rive/node.hpp:8-13`).
#[derive(Debug, Clone)]
pub(crate) struct RuntimeNodeState {
    computed_local_transform: Cell<Mat2D>,
    computed_local_needs_recompute: Cell<bool>,
}

impl RuntimeNodeState {
    pub(crate) fn new() -> Self {
        Self {
            computed_local_transform: Cell::new(Mat2D::IDENTITY),
            computed_local_needs_recompute: Cell::new(false),
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::new()
    }

    pub(crate) fn mark_computed_local_dirty(&self) {
        self.computed_local_needs_recompute.set(true);
    }

    pub(crate) fn computed_local_transform(
        &self,
        parent_world: Option<Mat2D>,
        world: Mat2D,
    ) -> Mat2D {
        if self.computed_local_needs_recompute.replace(false) {
            // Pinned `Node::computeLocalTransform` falls back to identity both
            // when there is no parent transform and when inversion fails
            // (`src/node.cpp:26-45`).
            let local = parent_world
                .filter(|parent| parent.determinant() != 0.0)
                .map(|parent| parent.invert_or_identity().multiply(world))
                .unwrap_or(Mat2D::IDENTITY);
            self.computed_local_transform.set(local);
        }
        self.computed_local_transform.get()
    }
}

impl ArtboardInstance {
    pub(crate) fn runtime_node_computed_local_transform(&self, local_id: usize) -> Option<Mat2D> {
        let handle = self.component_handle(local_id)?;
        let component = self.objects.component(handle)?;
        let node = component.concrete.node.as_ref()?;
        let parent_world = component
            .parent_transform
            .and_then(|parent| self.objects.component(parent))
            .map(|parent| parent.transform.world_transform);
        Some(node.computed_local_transform(parent_world, component.transform.world_transform))
    }
}
