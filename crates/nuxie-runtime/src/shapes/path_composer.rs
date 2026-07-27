use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use nuxie_graph::{ArtboardGraph, DependencyNodeKind, ShapePaintPathKind};

use crate::artboard::ArtboardInstance;
use crate::components::{ComponentDirt, RuntimeComponent};
use crate::draw::{
    RuntimeLayoutBounds, RuntimeShapePaintPathKind, RuntimeShapePathState,
    prune_empty_path_segments, runtime_raw_path_from_commands, runtime_shape_paint_path_kind_slot,
};
use crate::objects::InstanceObjectArena;

/// Construct the one embedded PathComposer occurrence owned by each Shape.
pub(crate) fn attach_occurrences(
    objects: &mut InstanceObjectArena,
    graph: &ArtboardGraph,
) -> Result<()> {
    for composer in &graph.path_composers {
        objects
            .attach_path_composer(
                composer.shape_local,
                RuntimeComponent::embedded(
                    composer.shape_local,
                    composer.shape_global,
                    "PathComposer",
                ),
            )
            .with_context(|| {
                format!(
                    "shape local id {} cannot own its PathComposer",
                    composer.shape_local
                )
            })?;
    }
    Ok(())
}

/// PathComposer dependency edges are inserted during the owning Shape's
/// authored-order `buildDependencies` call.
pub(crate) fn dependency_edge_indices(graph: &ArtboardGraph, shape_local: usize) -> Vec<usize> {
    let Some(composer_node) = graph.dependency_nodes.iter().position(|node| {
        matches!(
            node.kind,
            DependencyNodeKind::PathComposer {
                shape_local: node_shape_local,
                ..
            } if node_shape_local == shape_local
        )
    }) else {
        return Vec::new();
    };

    graph
        .dependency_node_edges_in_insertion_order
        .iter()
        .enumerate()
        .filter_map(|(edge_index, edge)| {
            (edge.dependent_node == composer_node
                && matches!(
                    edge.kind,
                    nuxie_graph::DependencyKind::PathComposerShape
                        | nuxie_graph::DependencyKind::PathComposerPath
                ))
            .then_some(edge_index)
        })
        .collect()
}

/// Concrete Path callbacks own the transition from source dirt to their
/// embedded PathComposer's Path dirt.
pub(crate) fn on_component_dirty(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    accumulated: ComponentDirt,
) {
    for shape_local in artboard
        .runtime_shapes
        .on_component_dirty(local_id, accumulated)
    {
        if let Some(composer) = artboard.objects.path_composer_handle(shape_local) {
            let composer_dirt = ComponentDirt::PATH | (accumulated & ComponentDirt::N_SLICER);
            artboard.add_component_dirt(composer, composer_dirt, true);
        }
    }
}

/// Direct `Path::collapse` -> `Shape::pathCollapseChanged` ->
/// `PathComposer::pathCollapseChanged` forwarding, including the C++ forced
/// dependent notification when Path dirt is already present.
pub(crate) fn path_collapse_changed(artboard: &mut ArtboardInstance, path_local: usize) {
    let Some(shape_local) = artboard.runtime_shapes.path_collapse_changed(path_local) else {
        return;
    };
    let Some(composer) = artboard.objects.path_composer_handle(shape_local) else {
        return;
    };
    artboard.add_component_dirt(composer, ComponentDirt::PATH, false);
    let dependent_count = artboard.objects.dependent_len(composer);
    for index in 0..dependent_count {
        if let Some(dependent) = artboard.objects.dependent_at(composer, index) {
            artboard.add_component_dirt(dependent, ComponentDirt::PATH, true);
        }
    }
}

/// Pinned C++ `PathComposer::update` geometry composition
/// (`src/shapes/path_composer.cpp:38-112`). The embedded composer is a real
/// dependency node; all CPU geometry settles here, never in `Shape::draw`.
/// Its missing `m_deferredPathDirt` lifecycle remains a recorded semantic gap.
pub(crate) fn update(
    artboard: &ArtboardInstance,
    shape_local: usize,
    dirt: ComponentDirt,
    graph: &ArtboardGraph,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) {
    if (dirt & (ComponentDirt::PATH | ComponentDirt::N_SLICER | ComponentDirt::FILTHY)).is_empty() {
        return;
    }

    let Some(shape) = artboard.runtime_shapes.get(shape_local) else {
        return;
    };
    shape.world_bounds.set(None);
    shape.world_length.set(None);
    for (path_kind, runtime_kind) in [
        (ShapePaintPathKind::Local, RuntimeShapePaintPathKind::Local),
        (
            ShapePaintPathKind::LocalClockwise,
            RuntimeShapePaintPathKind::LocalClockwise,
        ),
        (ShapePaintPathKind::World, RuntimeShapePaintPathKind::World),
    ] {
        let mut commands = artboard.runtime_shape_path_commands_from_owners(
            shape_local,
            path_kind,
            graph,
            layout_bounds,
        );
        prune_empty_path_segments(&mut commands);
        shape.paint_paths[runtime_shape_paint_path_kind_slot(runtime_kind)].replace_retained(
            RuntimeShapePathState {
                raw_path: Arc::new(runtime_raw_path_from_commands(&commands)),
            },
        );
    }
    for paint in &shape.paint_owners {
        paint.invalidate_all_effects();
    }
}
