use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use nuxie_graph::{ArtboardGraph, PathGeometryNode};
use nuxie_render_api::RawPath;

use crate::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle};
use crate::draw::RuntimeLayoutBounds;
use crate::objects::InstanceObjectArena;
use crate::properties::cached_property_key_for_name;
use std::sync::OnceLock;

/// Runtime-only fields owned by C++ `Path`.
///
/// `shape` is the occurrence-local `m_Shape` pointer rebuilt by
/// `Path::onAddedClean`; its embedded composer is reached through that Shape.
/// `deferred_path_dirt` and `flags` are the state read by
/// `Path::{onDirty,canDeferPathUpdate,update}`
/// (`src/shapes/path.cpp:76-125,300-372`).
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimePathState {
    pub(crate) shape: Option<ComponentHandle>,
    pub(crate) flags: Cell<u8>,
    pub(crate) deferred_path_dirt: Cell<bool>,
}

impl RuntimePathState {
    pub(crate) const CLIPPING: u8 = 1 << 3;
    pub(crate) const FOLLOW_PATH: u8 = 1 << 4;

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

/// The retained geometry custom fields owned by one concrete C++ `Path`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedPath {
    pub(crate) path: Arc<PathGeometryNode>,
    pub(crate) raw_path: Arc<RawPath>,
    pub(crate) has_weighted_context: bool,
}

/// Occurrence-owned C++ `Path` geometry state.
#[derive(Debug)]
pub(crate) struct RuntimePathOwner {
    pub(crate) dirty: Cell<bool>,
    pub(crate) retained: RefCell<Option<RuntimeOwnedPath>>,
}

impl Clone for RuntimePathOwner {
    fn clone(&self) -> Self {
        // Generated concrete Path clones copy only generated properties into
        // a fresh object (`include/rive/generated/shapes/path_base.hpp:
        // 62-67`); Path's custom `m_rawPath` and deferred dirt therefore start
        // fresh and are rebuilt by `Artboard::initialize`
        // (`include/rive/artboard.hpp:557-588`, `src/shapes/path.cpp:336-380`).
        Self::default()
    }
}

impl Default for RuntimePathOwner {
    fn default() -> Self {
        Self {
            dirty: Cell::new(true),
            retained: RefCell::new(None),
        }
    }
}

pub(crate) fn retained_follow_path_source(
    shapes: &crate::draw::RuntimeShapeList,
    path_local: usize,
) -> Option<(Arc<RawPath>, bool)> {
    let owner = shapes.path_owner(path_local)?;
    let retained = owner.retained.borrow();
    let retained = retained.as_ref()?;
    Some((
        Arc::clone(&retained.raw_path),
        retained.has_weighted_context,
    ))
}

/// Generated `PathBase::pathFlags` storage read from the live occurrence.
pub(crate) fn path_flags(artboard: &ArtboardInstance, local_id: usize, default: u64) -> u64 {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Path", "pathFlags")
        .and_then(|key| artboard.uint_property(local_id, key))
        .unwrap_or(default)
}

/// Direct `Path::onAddedClean` ownership: find the live Shape ancestor, retain
/// it, then register this exact Path occurrence on that Shape in authored
/// order (`src/shapes/path.cpp:76-96`).
pub(crate) fn on_added_clean(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> Result<()> {
    if objects
        .component(handle)
        .and_then(|component| component.concrete.path.as_ref())
        .is_none()
    {
        return Ok(());
    }

    let mut ancestor = objects
        .component(handle)
        .and_then(|component| component.parent);
    let shape = loop {
        let Some(candidate) = ancestor else {
            anyhow::bail!("Path is missing its owning Shape");
        };
        if objects
            .component(candidate)
            .and_then(|component| component.concrete.shape.as_ref())
            .is_some()
        {
            break candidate;
        }
        ancestor = objects
            .component(candidate)
            .and_then(|component| component.parent);
    };
    let shape_local = objects
        .component_local_id(shape)
        .context("Shape handle is missing its object identity")?;
    objects
        .path_composer_handle(shape_local)
        .context("Shape is missing its embedded PathComposer")?;
    objects
        .component_mut(handle)
        .expect("Path handle was validated")
        .concrete
        .path
        .as_mut()
        .expect("Path occurrence owns Path state")
        .shape = Some(shape);
    super::shape::add_path(objects, shape, handle);
    Ok(())
}

pub(crate) fn has_deferred_path_dirt(
    objects: &InstanceObjectArena,
    handle: ComponentHandle,
) -> bool {
    objects
        .component(handle)
        .and_then(|component| component.concrete.path.as_ref())
        .is_some_and(|path| path.deferred_path_dirt.get())
}

pub(crate) fn dirt_affects_path_epoch(dirt: ComponentDirt) -> bool {
    // C++ `Path::update` rebuilds raw path geometry for path/nslicer dirt, and
    // only for world-transform dirt when a deformer is present. Plain transform
    // animation is applied at draw time and does not churn retained commands.
    !(dirt
        & (ComponentDirt::PATH
            | ComponentDirt::VERTICES
            | ComponentDirt::LAYOUT_STYLE
            | ComponentDirt::N_SLICER))
        .is_empty()
}

/// Direct base `Path::markPathDirty` dirt publication.
///
/// Shape/PathComposer propagation remains attached to the ordinary Component
/// on-dirty dispatch, preserving the existing occurrence schedule
/// (`src/shapes/path.cpp:327-334`).
pub(crate) fn mark_path_dirty(artboard: &mut ArtboardInstance, handle: ComponentHandle) {
    artboard.add_component_dirt(handle, ComponentDirt::PATH, false);
}

/// Direct port of pinned C++ `Path::update`
/// (`src/shapes/path.cpp:336-380`). Geometry construction remains a renderer
/// helper, while all Path dirt/defer/retention lifecycle stays on this owner.
pub(crate) fn update(
    artboard: &mut ArtboardInstance,
    path_handle: ComponentHandle,
    dirt: ComponentDirt,
    graph: &ArtboardGraph,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) {
    let Some(path_local) = artboard.objects.component_local_id(path_handle) else {
        return;
    };
    let Some(path) = graph.paths.iter().find(|path| path.local_id == path_local) else {
        return;
    };
    let shape_local = artboard
        .objects
        .component(path_handle)
        .and_then(|component| component.concrete.path.as_ref())
        .and_then(|path| path.shape)
        .and_then(|shape| artboard.objects.component_local_id(shape));
    let has_deformer = shape_local.is_some_and(|shape_local| {
        artboard.runtime_shape_has_nsliced_context(shape_local, graph, layout_bounds)
    });
    let Some(owner) = artboard.runtime_shapes.path_owner(path_local) else {
        return;
    };
    let owner_dirty = owner.dirty.get();
    let owner_has_path = owner.retained.borrow().is_some();
    let should_rebuild = !owner_has_path
        || !(dirt & (ComponentDirt::PATH | ComponentDirt::N_SLICER)).is_empty()
        || (has_deformer && dirt.contains(ComponentDirt::WORLD_TRANSFORM));
    if !owner_dirty && !should_rebuild {
        return;
    }
    if !should_rebuild {
        owner.dirty.set(false);
        return;
    }

    if super::shape::can_defer_path_update(&artboard.objects, path_handle) {
        if let Some(path) = artboard
            .objects
            .component_mut(path_handle)
            .and_then(|component| component.concrete.path.as_mut())
        {
            path.deferred_path_dirt.set(true);
        }
        owner.dirty.set(false);
        return;
    }

    if let Some(path) = artboard
        .objects
        .component_mut(path_handle)
        .and_then(|component| component.concrete.path.as_mut())
    {
        path.deferred_path_dirt.set(false);
    }

    let retained = artboard.build_runtime_owned_path(path_handle, path, layout_bounds);
    let owner = artboard
        .runtime_shapes
        .path_owner(path_local)
        .expect("Path owner must remain live during its update");
    owner.retained.replace(Some(retained));
    owner.dirty.set(false);
}
