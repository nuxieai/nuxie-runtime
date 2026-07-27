use anyhow::{Context, Result};
use nuxie_graph::{ArtboardGraph, PathGeometryNode};

use crate::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle, Mat2D};
use crate::draw::RuntimeLayoutBounds;
use crate::objects::InstanceObjectArena;
use std::collections::BTreeMap;

fn is_points_path(objects: &InstanceObjectArena, handle: ComponentHandle) -> bool {
    objects
        .component(handle)
        .is_some_and(|component| component.type_name == "PointsPath")
}

pub(crate) fn has_skin(objects: &InstanceObjectArena, handle: ComponentHandle) -> bool {
    is_points_path(objects, handle)
        && objects
            .component(handle)
            .and_then(|component| component.concrete.skinnable.as_ref())
            .is_some_and(|skinnable| skinnable.skin.is_some())
}

/// Concrete tail of `PointsPath::buildDependencies`.
///
/// The caller performs `Super::buildDependencies` first, then invokes this
/// function so the retained Skin edge follows the base parent edge exactly as
/// it does in pinned C++ (`src/shapes/points_path.cpp:12-19`).
pub(crate) fn build_dependencies(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> Result<()> {
    if !is_points_path(objects, handle) {
        return Ok(());
    }
    let skin = objects
        .component(handle)
        .and_then(|component| component.concrete.skinnable.as_ref())
        .and_then(|skinnable| skinnable.skin);
    if let Some(skin) = skin {
        objects.add_dependent(skin, handle);
    }
    objects
        .component(handle)
        .context("PointsPath handle disappeared during dependency construction")?;
    Ok(())
}

/// Direct `PointsPath::pathTransform` override.
///
/// Skinned vertices are already deformed into world space, so the concrete
/// path uses identity; an unskinned occurrence delegates to `Path`'s world
/// transform (`src/shapes/points_path.cpp:21-29`).
pub(crate) fn path_transform(
    objects: &InstanceObjectArena,
    handle: ComponentHandle,
    world_transform: Mat2D,
) -> Mat2D {
    if has_skin(objects, handle) {
        Mat2D::IDENTITY
    } else {
        world_transform
    }
}

/// Concrete deformation decision used by the existing retained-geometry
/// adapter.
///
/// Vertex/RawPath construction remains renderer-facing. Pinned C++ performs
/// this deformation before `Path::update` and only for Path dirt
/// (`src/shapes/points_path.cpp:31-41`); staging Rust still performs it while
/// materializing retained geometry. This file move preserves that known
/// timing/dirt-mask gap for the semantic FL-E owner-family closure.
pub(crate) fn deform_owned_geometry(
    artboard: &mut ArtboardInstance,
    handle: ComponentHandle,
    path: &PathGeometryNode,
) -> bool {
    if !has_skin(&artboard.objects, handle) {
        return false;
    }
    artboard.deform_runtime_points_path_vertices(path);
    true
}

/// Concrete `PointsPath::update` dispatch followed by its `Path` base.
///
/// The existing Rust geometry adapter performs deformation while materializing
/// the retained RawPath; keeping that call there preserves staging semantics
/// while the C++ override boundary is now explicit and directly findable.
pub(crate) fn update(
    artboard: &mut ArtboardInstance,
    handle: ComponentHandle,
    dirt: ComponentDirt,
    graph: &ArtboardGraph,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> bool {
    if !is_points_path(&artboard.objects, handle) {
        return false;
    }
    super::path::update(artboard, handle, dirt, graph, layout_bounds);
    true
}

/// Existing `PointsPath::markPathDirty` bridge into the `Path` base.
///
/// Pinned C++ first re-dirties the retained Skin
/// (`src/shapes/points_path.cpp:43-50`). The staging Rust behavior being moved
/// here only published Path dirt from `Skin::onDirty`; preserve that known
/// semantic gap rather than silently correcting it during a file move.
pub(crate) fn mark_path_dirty(artboard: &mut ArtboardInstance, handle: ComponentHandle) {
    super::path::mark_path_dirty(artboard, handle);
}

/// Direct `PointsPath::markSkinDirty` override
/// (`src/shapes/points_path.cpp:52`).
pub(crate) fn mark_skin_dirty(artboard: &mut ArtboardInstance, handle: ComponentHandle) {
    if is_points_path(&artboard.objects, handle) {
        mark_path_dirty(artboard, handle);
    }
}
