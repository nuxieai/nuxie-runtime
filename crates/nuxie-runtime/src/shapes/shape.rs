use std::cell::Cell;

use crate::ArtboardInstance;
use crate::components::ComponentHandle;
use crate::draw::{
    RuntimePathMeasure, RuntimeShapePaintOwner, RuntimeShapePaintPathKind,
    RuntimeShapePaintPathOwner, runtime_path_commands_from_raw_path,
    runtime_shape_paint_path_kind_slot,
};
use crate::objects::InstanceObjectArena;

use super::path::RuntimePathState;

/// Runtime-only fields owned by C++ `Shape`.
///
/// Paths register in authored order during `Path::onAddedClean`; flags are
/// accumulated by clipping/follow-path/hit-test owners on this exact
/// occurrence (`src/shapes/shape.cpp:20-51`).
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeShapeState {
    pub(crate) paths: Vec<ComponentHandle>,
    pub(crate) flags: Cell<u8>,
}

impl RuntimeShapeState {
    pub(crate) const CLIPPING: u8 = RuntimePathState::CLIPPING;
    pub(crate) const FOLLOW_PATH: u8 = RuntimePathState::FOLLOW_PATH;
    pub(crate) const NEVER_DEFER_UPDATE: u8 = 1 << 5;

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }

    pub(crate) fn add_flags(&self, flags: u8) -> bool {
        let previous = self.flags.get();
        self.flags.set(previous | flags);
        previous & flags != flags
    }

    pub(crate) fn is_flagged(&self, flags: u8) -> bool {
        self.flags.get() & flags != 0
    }
}

/// Occurrence-owned C++ `Shape` state. The paint/path backend slots stay
/// renderer-owned; Shape owns their ordered membership and derived caches.
#[derive(Debug)]
pub(crate) struct RuntimeShape {
    pub(crate) path_locals: Vec<usize>,
    pub(crate) paint_container_index: Option<usize>,
    pub(crate) paint_paths: [RuntimeShapePaintPathOwner; 3],
    pub(crate) paint_owners: Vec<RuntimeShapePaintOwner>,
    pub(crate) world_bounds: Cell<Option<nuxie_render_api::Aabb>>,
    pub(crate) world_length: Cell<Option<f32>>,
}

impl Clone for RuntimeShape {
    fn clone(&self) -> Self {
        Self {
            path_locals: Vec::new(),
            paint_container_index: self.paint_container_index,
            paint_paths: std::array::from_fn(|_| RuntimeShapePaintPathOwner::default()),
            paint_owners: self.paint_owners.clone(),
            world_bounds: Cell::new(None),
            world_length: Cell::new(None),
        }
    }
}

/// `Shape::onAddedDirty` forwards the lifecycle phase to its embedded
/// PathComposer immediately after the Shape's Component base.
pub(crate) fn on_added_dirty(
    objects: &mut InstanceObjectArena,
    shape_local: usize,
    root: ComponentHandle,
) {
    if let Some(composer) = objects.path_composer_handle(shape_local) {
        objects.link_parent(composer, root);
    }
}

pub(crate) fn add_path(
    objects: &mut InstanceObjectArena,
    shape: ComponentHandle,
    path: ComponentHandle,
) {
    let paths = &mut objects
        .component_mut(shape)
        .expect("Shape handle was validated")
        .concrete
        .shape
        .as_mut()
        .expect("Shape occurrence owns Shape state")
        .paths;
    assert!(
        !paths.contains(&path),
        "C++ Shape::addPath requires unique Path registration"
    );
    paths.push(path);
}

pub(crate) fn embedded_path_composer(
    objects: &InstanceObjectArena,
    shape: ComponentHandle,
) -> Option<ComponentHandle> {
    let shape_local = objects.component_local_id(shape)?;
    objects.path_composer_handle(shape_local)
}

/// Literal `Path::canDeferPathUpdate` / `Shape::canDeferPathUpdate`.
pub(crate) fn can_defer_path_update(
    objects: &InstanceObjectArena,
    path_handle: ComponentHandle,
) -> bool {
    let Some(path) = objects
        .component(path_handle)
        .and_then(|component| component.concrete.path.as_ref())
    else {
        return false;
    };
    let Some(shape_handle) = path.shape else {
        return false;
    };
    let Some(shape_component) = objects.component(shape_handle) else {
        return false;
    };
    let Some(shape) = shape_component.concrete.shape.as_ref() else {
        return false;
    };
    if shape_component.transform.render_opacity != 0.0
        || shape.is_flagged(RuntimeShapeState::CLIPPING | RuntimeShapeState::NEVER_DEFER_UPDATE)
    {
        return false;
    }
    if shape_component.dependents.iter().copied().any(|dependent| {
        objects.component(dependent).is_some_and(|component| {
            component.type_name == "PointsPath"
                && component
                    .concrete
                    .skinnable
                    .as_ref()
                    .and_then(|skinnable| skinnable.skin)
                    .is_some()
        })
    }) {
        return false;
    }
    !shape.is_flagged(RuntimeShapeState::FOLLOW_PATH)
        && !path.is_flagged(RuntimePathState::FOLLOW_PATH | RuntimePathState::CLIPPING)
}

pub(crate) fn length_with_layout(artboard: &ArtboardInstance, shape_local: usize) -> Option<f32> {
    let shape = artboard.runtime_shapes.get(shape_local)?;
    if let Some(length) = shape.world_length.get() {
        return Some(length);
    }
    let world_path = shape.paint_paths
        [runtime_shape_paint_path_kind_slot(RuntimeShapePaintPathKind::World)]
    .retained
    .borrow();
    let world_path = world_path.as_ref()?;
    let commands = runtime_path_commands_from_raw_path(world_path.raw_path.as_ref());
    let length = RuntimePathMeasure::from_commands(&commands).length();
    shape.world_length.set(Some(length));
    Some(length)
}
