use crate::mechanical_port::source::{
    component::{Component, ComponentOccurrenceHandle},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    layout::layout_participant::LayoutParticipant,
    math::{aabb::Aabb, mat2d::Mat2D},
    shapes::{paint::shape_paint_path::ShapePaintPath, path_flags::PathFlags},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct RuntimePathComposerHandle(Rc<RefCell<PathComposer>>);

impl RuntimePathComposerHandle {
    pub fn new() -> Self {
        Self(Rc::new_cyclic(|weak| {
            let mut component = Component::default();
            component
                .bind_runtime_occurrence(ComponentOccurrenceHandle::PathComposer(weak.clone()));
            RefCell::new(PathComposer {
                component,
                shape: None,
                local_path: ShapePaintPath::new(true),
                world_path: ShapePaintPath::new(false),
                local_clockwise_path: ShapePaintPath::new(true),
                deferred_path_dirt: false,
                shape_notified: false,
                local_inputs: Vec::new(),
                scratch_inputs: Vec::new(),
                reported_intrinsic_bounds: Aabb::default(),
                has_reported_intrinsic_bounds: false,
                built_local_flags: PathFlags::NONE,
                has_local_inputs: false,
            })
        }))
    }
    pub fn occurrence(&self) -> ComponentOccurrenceHandle {
        ComponentOccurrenceHandle::PathComposer(Rc::downgrade(&self.0))
    }
    pub fn with<R>(&self, f: impl FnOnce(&PathComposer) -> R) -> R {
        f(&self.0.borrow())
    }
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut PathComposer) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
    pub fn bind_shape(&self, shape: CoreHandle) {
        self.0.borrow_mut().shape = Some(shape);
    }
    pub fn add_dependent(&self, dependent: impl Into<ComponentOccurrenceHandle>) {
        self.0.borrow_mut().component.add_dependent(dependent);
    }
    pub fn add_dirt_from_shape(
        &self,
        shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        let dirty = self.with_mut(|helper| helper.component.add_dirt_state(value));
        let Some(_) = dirty else {
            return false;
        };
        if self.with_mut(|helper| helper.dirty_shape()).is_some() {
            shape.path_changed();
        }
        let occurrence = self.occurrence();
        occurrence.notify_artboard();
        if recurse {
            for dependent in occurrence
                .with_component(Component::dependents_snapshot)
                .unwrap_or_default()
            {
                dependent.add_dirt_from_shape(shape, value, true);
            }
        }
        true
    }
    pub fn collapse_from_shape(
        &self,
        shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
        value: bool,
    ) -> bool {
        if self
            .with_mut(|helper| helper.component.collapse_state(value))
            .is_none()
        {
            return false;
        }
        if self.with_mut(|helper| helper.dirty_shape()).is_some() {
            shape.path_changed();
        }
        self.occurrence().notify_artboard();
        self.with_mut(|helper| helper.component.update_collapsables());
        true
    }
}
impl Default for RuntimePathComposerHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct LocalPathInput {
    pub transform: Mat2D,
    pub geometry_version: u32,
    pub skipped: bool,
    pub linear_tolerance: f32,
    pub translation_tolerance: f32,
}

impl LocalPathInput {
    pub fn matches(&self, other: &Self) -> bool {
        if self.geometry_version != other.geometry_version || self.skipped != other.skipped {
            return false;
        }
        for i in 0..4 {
            if (self.transform[i] - other.transform[i]).abs() > other.linear_tolerance {
                return false;
            }
        }
        (self.transform[4] - other.transform[4]).abs() <= other.translation_tolerance
            && (self.transform[5] - other.transform[5]).abs() <= other.translation_tolerance
    }
}

pub struct PathComposer {
    pub component: Component,
    shape: Option<CoreHandle>,
    local_path: ShapePaintPath,
    world_path: ShapePaintPath,
    local_clockwise_path: ShapePaintPath,
    deferred_path_dirt: bool,
    shape_notified: bool,
    local_inputs: Vec<LocalPathInput>,
    scratch_inputs: Vec<LocalPathInput>,
    reported_intrinsic_bounds: Aabb,
    has_reported_intrinsic_bounds: bool,
    built_local_flags: PathFlags,
    has_local_inputs: bool,
}

impl PathComposer {
    /// Compare to the bounds last reported to layout, not the previous frame,
    /// so tolerated round-trip drift cannot accumulate.
    pub fn intrinsic_bounds_changed(&mut self) -> bool {
        fn moved_past_ulps(a: f32, b: f32) -> bool {
            // std::max retains its first operand for unordered comparisons.
            let mut magnitude = 1.0_f32;
            if magnitude < a.abs() {
                magnitude = a.abs();
            }
            if magnitude < b.abs() {
                magnitude = b.abs();
            }
            (a - b).abs() > (16.0 * f32::EPSILON) * magnitude
        }
        let bounds = self
            .shape()
            .with(|object| {
                let shape = object.as_shape().expect("PathComposer Shape");
                // Collapsing a path can change measured bounds without dropping
                // the participant's memoized host bounds. Always measure fresh.
                if let Some(participant) = shape.layout_participant() {
                    participant.with_downcast_mut::<LayoutParticipant, _>(
                        LayoutParticipant::invalidate_host_bounds,
                    );
                }
                shape.compute_intrinsic_bounds()
            })
            .expect("live PathComposer Shape");
        let last = self.reported_intrinsic_bounds;
        let changed = !self.has_reported_intrinsic_bounds
            || moved_past_ulps(bounds.min_x, last.min_x)
            || moved_past_ulps(bounds.min_y, last.min_y)
            || moved_past_ulps(bounds.max_x, last.max_x)
            || moved_past_ulps(bounds.max_y, last.max_y);
        if changed {
            self.reported_intrinsic_bounds = bounds;
            self.has_reported_intrinsic_bounds = true;
        }
        changed
    }
    pub fn local_inputs_changed(&mut self) -> bool {
        // Preserve std::max's operand selection, including unordered values.
        fn source_max(a: f32, b: f32) -> f32 {
            if a < b { b } else { a }
        }
        fn linear_magnitude(transform: Mat2D) -> f32 {
            source_max(
                source_max(
                    source_max(transform[0].abs(), transform[1].abs()),
                    transform[2].abs(),
                ),
                transform[3].abs(),
            )
        }
        let (paths, world, local_flags) = self
            .shape()
            .with(|object| {
                let shape = object.as_shape().expect("PathComposer Shape");
                (
                    shape.paths(),
                    *shape.world_transform(),
                    shape.paint_container.path_flags()
                        & (PathFlags::LOCAL | PathFlags::LOCAL_CLOCKWISE),
                )
            })
            .expect("live PathComposer Shape");
        let inverse = world.invert_or_identity();
        const ULPS: f32 = 16.0 * f32::EPSILON;
        let inverse_linear = linear_magnitude(inverse);
        let inverse_translation = source_max(inverse[4].abs(), inverse[5].abs());
        self.scratch_inputs.clear();
        self.scratch_inputs.reserve(paths.len());
        for handle in &paths {
            let input = handle
                .with(|object| {
                    let path = object.as_path().expect("Shape path");
                    let path_world =
                        crate::mechanical_port::source::shapes::path::Path::path_transform_for(
                            object,
                        );
                    let path_linear = linear_magnitude(path_world);
                    let path_translation = source_max(path_world[4].abs(), path_world[5].abs());
                    LocalPathInput {
                        transform: inverse * path_world,
                        geometry_version: path.geometry_version(),
                        skipped: path.is_hidden() || path.is_collapsed(),
                        linear_tolerance: ULPS * source_max(1.0, inverse_linear * path_linear),
                        translation_tolerance: ULPS
                            * source_max(
                                1.0,
                                inverse_linear * path_translation + inverse_translation,
                            ),
                    }
                })
                .expect("live Shape path");
            self.scratch_inputs.push(input);
        }
        let mut changed = !self.has_local_inputs
            || self.local_inputs.len() != paths.len()
            || !(local_flags & !self.built_local_flags).is_empty();
        for i in 0..paths.len() {
            if changed {
                break;
            }
            changed = !self.local_inputs[i].matches(&self.scratch_inputs[i]);
        }
        if changed {
            // Compare future frames to the inputs actually used for the build,
            // never to a running snapshot that would accumulate tolerated drift.
            self.local_inputs.clone_from(&self.scratch_inputs);
            self.has_local_inputs = true;
            self.built_local_flags = local_flags;
        }
        changed
    }
    pub fn shape(&self) -> CoreHandle {
        self.shape.clone().expect("arena-installed Shape")
    }

    pub(crate) fn dirty_shape(&mut self) -> Option<CoreHandle> {
        if !self.deferred_path_dirt || self.shape_notified {
            return None;
        }
        self.shape_notified = true;
        Some(self.shape())
    }

    /// Shape adds itself before this tail, while it already has its own mutable
    /// borrow. The remaining path edges require no reborrow of that Shape slot.
    pub(crate) fn build_path_dependencies(&mut self, paths: &[CoreHandle]) {
        let dependent = self
            .component
            .occurrence_handle()
            .expect("PathComposer occurrence");
        for path in paths {
            path.with_mut(|path| {
                path.as_component_mut()
                    .expect("Shape path Component")
                    .add_dependent(dependent.clone());
            });
        }
    }

    pub fn update(&mut self, value: ComponentDirt) -> Option<(CoreHandle, bool)> {
        self.shape_notified = false;
        if !value.intersects(ComponentDirt::PATH | ComponentDirt::N_SLICER) {
            return None;
        }
        let shape_handle = self.shape();
        let inputs = shape_handle
            .with(|shape| {
                let shape = shape.as_shape().expect("PathComposer Shape");
                if shape.can_defer_path_update() {
                    return None;
                }
                Some((
                    shape.is_flagged(PathFlags::LOCAL),
                    shape.is_flagged(PathFlags::LOCAL_CLOCKWISE),
                    shape.is_flagged(PathFlags::WORLD),
                    *shape.world_transform(),
                    shape.paths(),
                ))
            })
            .expect("live PathComposer Shape");
        let Some((local, clockwise, world, transform, paths)) = inputs else {
            self.deferred_path_dirt = true;
            return None;
        };
        self.deferred_path_dirt = false;
        let rebuild_local = (local || clockwise) && self.local_inputs_changed();
        if local && rebuild_local {
            self.local_path.rewind();
            let inverse = transform.invert_or_identity();
            for handle in &paths {
                handle.with(|object| {
                    let path = object.as_path().expect("Shape path");
                    if !path.is_hidden() && !path.is_collapsed() {
                        let path_transform = object
                            .as_points_path()
                            .map(|points| *points.path_transform())
                            .unwrap_or_else(|| path.path_transform());
                        self.local_path
                            .add_path(path.raw_path(), Some(&(inverse * path_transform)));
                    }
                });
            }
        }
        if clockwise && rebuild_local {
            self.local_clockwise_path.rewind();
            let inverse = transform.invert_or_identity();
            for handle in &paths {
                handle.with_mut(|object| {
                    let path = object.as_path().expect("Shape path");
                    if path.is_hidden() || path.is_collapsed() {
                        return;
                    }
                    let path_transform = object
                        .as_points_path()
                        .map(|points| *points.path_transform())
                        .unwrap_or_else(|| path.path_transform());
                    let local_transform = inverse * path_transform;
                    let not_clockwise = object.as_points_path_mut().is_some_and(|points| {
                        local_transform.determinant() * (points.winding() as f32) < 0.0
                    });
                    let path = object.as_path().expect("Shape path");
                    if not_clockwise != path.is_hole() {
                        self.local_clockwise_path
                            .add_path_backwards(path.raw_path(), Some(&local_transform));
                    } else {
                        self.local_clockwise_path
                            .add_path(path.raw_path(), Some(&local_transform));
                    }
                });
            }
        }
        if world {
            self.world_path.rewind();
            for handle in &paths {
                handle.with(|object| {
                    let path = object.as_path().expect("Shape path");
                    if !path.is_hidden() && !path.is_collapsed() {
                        let path_transform = object
                            .as_points_path()
                            .map(|points| *points.path_transform())
                            .unwrap_or_else(|| path.path_transform());
                        self.world_path
                            .add_path(path.raw_path(), Some(&path_transform));
                    }
                });
            }
        }
        let has_participant = shape_handle
            .with(|object| {
                object
                    .as_shape()
                    .expect("PathComposer Shape")
                    .has_layout_participant()
            })
            .expect("live PathComposer Shape");
        let measured_geometry_changed = !has_participant || self.intrinsic_bounds_changed();
        // Carry the source's bounds-dirt choice across the existing Rust
        // borrow handoff; notifying layout can recursively touch the composer.
        Some((shape_handle, measured_geometry_changed))
    }

    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        &mut self.local_path
    }
    pub fn world_path(&mut self) -> &mut ShapePaintPath {
        &mut self.world_path
    }
    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        &mut self.local_clockwise_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::{core::CoreArena, shapes::shape::Shape};

    #[test]
    fn deferred_update_preserves_one_notification_per_update() {
        let arena = CoreArena::default();
        let shape = arena.insert(Shape::default());
        assert!(
            shape
                .with_downcast::<Shape, _>(Shape::can_defer_path_update)
                .unwrap()
        );
        let composer = RuntimePathComposerHandle::new();
        composer.bind_shape(shape.clone());
        composer.with_mut(|composer| {
            assert!(composer.update(ComponentDirt::PATH).is_none());
            assert!(composer.deferred_path_dirt);
            assert!(!composer.shape_notified);
            assert_eq!(composer.dirty_shape(), Some(shape.clone()));
            assert!(composer.dirty_shape().is_none());
            assert!(composer.update(ComponentDirt::N_SLICER).is_none());
            assert!(composer.deferred_path_dirt);
            assert!(!composer.shape_notified);
            assert_eq!(composer.dirty_shape(), Some(shape));
            assert!(composer.dirty_shape().is_none());
        });
    }
}
