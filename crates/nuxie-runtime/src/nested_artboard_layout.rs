//! Runtime counterpart of pinned C++ `nested_artboard_layout.hpp/.cpp`.
//!
//! `Artboard` retains authored-order construction and advancing dispatch. This
//! owner retains the mounted layout-node transfer lifecycle and the generated
//! width/height override inputs consumed by the delegated layout engine.
//! Taffy construction, intrinsic measurement, draw traversal, and draw-time
//! command materialization remain renderer-owned.
//!
//! This is a behavior-preserving structural extraction, not yet the semantic
//! closure for pinned `StyleOverrider`: the existing Rust interpretation of
//! negative authored lengths and the pinned height/width scale quirk remain
//! explicit follow-up fidelity work.

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use super::{ArtboardInstance, RuntimeComponent, RuntimeLayoutBounds, property_key_for_name};
use crate::properties::cached_property_key_for_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RuntimeNestedLayoutBoundsCacheKey {
    pub(super) graph_global_id: u32,
    pub(super) layout_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RuntimeNestedLayoutDataTransferKey {
    pub(super) parent_layout: RuntimeNestedLayoutBoundsCacheKey,
    pub(super) assigned_bounds: RuntimeLayoutBounds,
    pub(super) child_layout_epoch: u64,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeNestedLayoutBoundsFrame {
    pub(super) key: RuntimeNestedLayoutBoundsCacheKey,
    pub(super) bounds: Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

impl ArtboardInstance {
    pub(super) fn runtime_nested_artboard_layout_bounds_frame(
        &mut self,
    ) -> RuntimeNestedLayoutBoundsFrame {
        let key = RuntimeNestedLayoutBoundsCacheKey {
            graph_global_id: self.graph_global_id,
            layout_epoch: self.layout_epoch,
        };
        if self
            .nested_layout_bounds
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            self.nested_layout_bounds = Some(RuntimeNestedLayoutBoundsFrame {
                key,
                bounds: Arc::new(self.compute_runtime_nested_artboard_layout_bounds()),
            });
        }

        self.nested_layout_bounds
            .as_ref()
            .expect("nested layout bounds frame was just populated")
            .clone()
    }

    pub(super) fn capture_initial_nested_artboard_layout_paint_frame(
        &mut self,
        host_local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        mut paint_evaluation: ArtboardInstance,
    ) {
        if !self
            .component(host_local_id)
            .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        {
            return;
        }
        let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&host_local_id).copied())
        else {
            return;
        };
        // C++ configures paints on this one mounted occurrence before
        // NestedArtboardLayout transfers its constraint space. Evaluate that
        // source-side shader state only on a script-free temporary occurrence.
        paint_evaluation.detach_initial_nested_layout_paint_binding_contexts();
        paint_evaluation.set_artboard_dimensions(bounds.width, bounds.height);
        if let Some(width_key) = property_key_for_name("LayoutComponent", "width") {
            paint_evaluation.set_double_property(0, width_key, bounds.width);
        }
        if let Some(height_key) = property_key_for_name("LayoutComponent", "height") {
            paint_evaluation.set_double_property(0, height_key, bounds.height);
        }
        paint_evaluation.update_components();
        let before_bind = paint_evaluation.capture_initial_nested_layout_paint_frame();
        paint_evaluation.advance_artboard_data_binds();
        paint_evaluation.update_components();
        let frame = paint_evaluation.capture_initial_nested_layout_paint_frame();
        if !frame.changed_from(&before_bind) {
            return;
        }
        if let Some(nested) = self.nested_artboards.get_mut(&host_local_id)
            && !nested.layout_data_transferred
            && nested.initial_layout_paint_frame.borrow().is_none()
        {
            nested.initial_layout_paint_frame.replace(Some(frame));
        }
    }

    pub(super) fn apply_nested_artboard_layout_bounds_after_parent_solve(&mut self) -> bool {
        if !self.nested_artboard_locals.iter().any(|host_local_id| {
            self.component(*host_local_id)
                .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        }) {
            return false;
        }
        let layout_frame = self.runtime_nested_artboard_layout_bounds_frame();
        let mut changed = false;
        for index in 0..self.nested_artboard_locals.len() {
            let host_local_id = self.nested_artboard_locals[index];
            if self
                .component(host_local_id)
                .is_some_and(RuntimeComponent::is_collapsed)
            {
                continue;
            }
            changed |= self.apply_nested_artboard_layout_bounds(
                host_local_id,
                layout_frame.bounds.as_ref().as_ref(),
                layout_frame.key,
            );
        }
        changed
    }

    pub(super) fn apply_nested_artboard_layout_bounds(
        &mut self,
        host_local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        parent_layout: RuntimeNestedLayoutBoundsCacheKey,
    ) -> bool {
        if !self
            .component(host_local_id)
            .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        {
            return false;
        }
        let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&host_local_id).copied())
        else {
            return false;
        };
        let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
            return false;
        };

        let first_transfer = !nested.layout_data_transferred;
        let refresh_constraint_bounds = nested.layout_data_transfer_key.is_none_or(|key| {
            key.parent_layout != parent_layout
                || key.assigned_bounds != bounds
                || key.child_layout_epoch != nested.child.layout_epoch
        });
        let mut changed = nested
            .child
            .set_artboard_dimensions(bounds.width, bounds.height);
        if first_transfer {
            // The recursive host bind above has applied the rounded initial
            // values but has not yet consumed their component dirt. Settle
            // that unconstrained component state before taking the one Yoga
            // layout snapshot owned by the parent.
            changed |= nested.child.update_components().did_update;
        }

        // Match NestedArtboardLayout's mounted ordering: the constraint space
        // exists before its root LayoutComponent width/height dirt is raised.
        // Reversing these two operations changes the first layout solve.
        if refresh_constraint_bounds {
            nested.child.refresh_layout_constraint_bounds();
            changed = true;
        } else {
            changed |= !nested.child.layout_constraint_bounds_enabled;
            nested.child.enable_layout_constraint_bounds();
        }
        if let Some(width_key) = property_key_for_name("LayoutComponent", "width") {
            changed |= nested.child.set_double_property(0, width_key, bounds.width);
        }
        if let Some(height_key) = property_key_for_name("LayoutComponent", "height") {
            changed |= nested
                .child
                .set_double_property(0, height_key, bounds.height);
        }
        nested.layout_data_transferred = true;
        if changed {
            nested.child.update_pass();
        }
        // Record after assigned-root writes and their child update pass. Those
        // writes dirty the transferred root node themselves; only a later
        // child layout generation should emulate C++ `markHostingLayoutDirty`
        // and request another parent-owned constraint refresh.
        nested.layout_data_transfer_key = Some(RuntimeNestedLayoutDataTransferKey {
            parent_layout,
            assigned_bounds: bounds,
            child_layout_epoch: nested.child.layout_epoch,
        });
        changed
    }
}

fn runtime_nested_artboard_layout_property_key_for_name(property_name: &str) -> Option<u16> {
    fn cached(key: &'static OnceLock<Option<u16>>, property_name: &'static str) -> Option<u16> {
        cached_property_key_for_name(key, "NestedArtboardLayout", property_name)
    }

    match property_name {
        "instanceWidthScaleType" => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached(&KEY, "instanceWidthScaleType")
        }
        "instanceHeightScaleType" => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached(&KEY, "instanceHeightScaleType")
        }
        "instanceWidthUnitsValue" => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached(&KEY, "instanceWidthUnitsValue")
        }
        "instanceHeightUnitsValue" => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached(&KEY, "instanceHeightUnitsValue")
        }
        "instanceWidth" => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached(&KEY, "instanceWidth")
        }
        "instanceHeight" => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached(&KEY, "instanceHeight")
        }
        _ => property_key_for_name("NestedArtboardLayout", property_name),
    }
}

/// Retained generated-property inputs for the renderer-owned layout adapter.
///
/// Keeping these reads beside the pinned owner makes the override contract
/// discoverable without moving Taffy construction or layout traversal out of
/// the renderer boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeNestedArtboardAxisOverride {
    Fixed {
        length: f32,
        units: u64,
        uses_intrinsic_size: bool,
    },
    Fill,
    Hug,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeNestedArtboardLayoutOverrides {
    pub(crate) width: RuntimeNestedArtboardAxisOverride,
    pub(crate) height: RuntimeNestedArtboardAxisOverride,
}

pub(crate) fn runtime_layout_overrides(
    instance: &ArtboardInstance,
    local: usize,
) -> Option<RuntimeNestedArtboardLayoutOverrides> {
    let width = axis_override(instance, local, true)?;
    let height = axis_override(instance, local, false)?;
    Some(RuntimeNestedArtboardLayoutOverrides { width, height })
}

fn axis_override(
    instance: &ArtboardInstance,
    local: usize,
    width_axis: bool,
) -> Option<RuntimeNestedArtboardAxisOverride> {
    match axis_scale(instance, local, width_axis) {
        0 => {
            let length = axis_length(instance, local, width_axis);
            Some(RuntimeNestedArtboardAxisOverride::Fixed {
                length,
                units: axis_units(instance, local, width_axis),
                // Preserve the existing Rust adapter's behavior for this
                // structural move. Pinned StyleOverrider recognizes exactly
                // -1.0; semantic closure owns narrowing this predicate.
                uses_intrinsic_size: length < 0.0,
            })
        }
        1 => Some(RuntimeNestedArtboardAxisOverride::Fill),
        2 => Some(RuntimeNestedArtboardAxisOverride::Hug),
        _ => None,
    }
}

fn axis_scale(instance: &ArtboardInstance, local: usize, width_axis: bool) -> u64 {
    runtime_nested_artboard_layout_property_key_for_name(if width_axis {
        "instanceWidthScaleType"
    } else {
        "instanceHeightScaleType"
    })
    .and_then(|key| instance.uint_property(local, key))
    .unwrap_or(0)
}

fn axis_units(instance: &ArtboardInstance, local: usize, width_axis: bool) -> u64 {
    runtime_nested_artboard_layout_property_key_for_name(if width_axis {
        "instanceWidthUnitsValue"
    } else {
        "instanceHeightUnitsValue"
    })
    .and_then(|key| instance.uint_property(local, key))
    .unwrap_or(1)
}

fn axis_length(instance: &ArtboardInstance, local: usize, width_axis: bool) -> f32 {
    runtime_nested_artboard_layout_property_key_for_name(if width_axis {
        "instanceWidth"
    } else {
        "instanceHeight"
    })
    .and_then(|key| instance.double_property(local, key))
    .unwrap_or(-1.0)
}
