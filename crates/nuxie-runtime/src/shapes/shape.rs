//! Shape membership and PathComposer ownership are occurrence-local. The
//! retained owner is represented by `RuntimeShapeList`; this direct module is
//! the callback/lifecycle correspondence point for pinned `shape.cpp`.

use std::collections::BTreeMap;

use anyhow::Result;
use nuxie_graph::ArtboardGraph;
use nuxie_render_api::{Factory as RenderFactory, Renderer};

use crate::{
    ArtboardInstance, RuntimeLayoutBounds,
    draw::{
        RuntimeDrawable, runtime_live_owned_shape_paint_path_kind,
        runtime_owned_shape_paint_is_visible,
    },
};

/// Direct port of C++ `Shape::draw` (`src/shapes/shape.cpp:137-159`). Shape
/// memberships and paths are already retained by the clone-owned Shape; this
/// adapter draws those live paints without constructing a drawable dispatch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn runtime_draw_live_owned_shape(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    drawable: &RuntimeDrawable,
    shape_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    drawable_needs_save_operation: bool,
    backend_context_id: u64,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    let paint_count = instance
        .runtime_shapes
        .get(shape_local)
        .map_or(0, |shape| shape.paint_owners.len());
    let needs_save_operation = needs_save_operation(drawable_needs_save_operation, paint_count);

    for owner_index in 0..paint_count {
        // C++ `Shape::draw` reads `worldTransform()` directly from the
        // inherited Component (`shape.cpp:137-159`). Use the retained
        // Drawable->Component link for the ordinary path. Layout-backed
        // transforms keep the existing compatibility adapter until their
        // owner publishes the solved transform into Component.
        let shape_world = instance
            .retained_drawable_component(drawable)
            .filter(|component| {
                component.type_name != "LayoutComponent"
                    && component.type_name != "NestedArtboardLayout"
                    && (layout_bounds.is_none() || component.layout_ancestors.is_empty())
            })
            .map(|component| component.transform.world_transform)
            .unwrap_or_else(|| {
                instance.runtime_component_world_transform_with_bounds(
                    shape_local,
                    graph,
                    layout_bounds,
                )
            });
        let Some(owner) = instance
            .runtime_shapes
            .get(shape_local)
            .and_then(|shape| shape.paint_owners.get(owner_index))
        else {
            continue;
        };
        // Direct C++ `Shape::draw` read: visibility belongs to the live
        // ShapePaint, not to a prepared draw command.
        if !runtime_owned_shape_paint_is_visible(instance, owner) {
            continue;
        }
        let live_path_kind = runtime_live_owned_shape_paint_path_kind(instance, owner);
        let Some(source_path) = instance
            .runtime_shapes
            .paint_path_owner(shape_local, live_path_kind)
        else {
            continue;
        };
        super::paint::shape_paint::runtime_draw_live_owned_shape_paint(
            instance,
            shape_world,
            owner,
            source_path,
            live_path_kind,
            needs_save_operation,
            backend_context_id,
            factory,
            renderer,
        )?;
    }
    Ok(())
}

pub(crate) fn can_defer_path_update(
    render_opacity: f32,
    clipping_or_never_defer: bool,
    has_skinned_path_dependent: bool,
    follow_path_consumer: bool,
) -> bool {
    render_opacity == 0.0
        && !clipping_or_never_defer
        && !has_skinned_path_dependent
        && !follow_path_consumer
}

pub(crate) fn needs_save_operation(container_needs_save: bool, paint_count: usize) -> bool {
    container_needs_save || paint_count > 1
}
