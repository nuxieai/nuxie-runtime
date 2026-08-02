//! Embedded PathComposer callback owner. The draw coordinator invokes its
//! dependency-ordered update to rebuild local, local-clockwise, and world
//! ShapePaintPaths; drawing never reconstructs those paths.

use std::{collections::BTreeMap, sync::Arc};

use nuxie_graph::{ArtboardGraph, ShapePaintPathKind};

use crate::{
    ArtboardInstance, ComponentDirt, RuntimeLayoutBounds,
    draw::{RuntimeShapePaintPathKind, RuntimeShapePathState, runtime_shape_paint_path_kind_slot},
    math::raw_path::{prune_empty_path_segments, runtime_raw_path_from_commands},
};

impl ArtboardInstance {
    /// Pinned C++ `PathComposer::update` geometry composition
    /// (`src/shapes/path_composer.cpp:38-112`). The embedded composer is a
    /// real dependency node; all CPU geometry settles here, never in
    /// `Shape::draw`. Its separate `m_deferredPathDirt` lifecycle remains in
    /// the manifest's FL-E PathComposer slice (`path_composer.cpp:29-49`).
    pub(crate) fn update_runtime_path_composer(
        &self,
        shape_local: usize,
        dirt: ComponentDirt,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) {
        if (dirt & (ComponentDirt::PATH | ComponentDirt::N_SLICER | ComponentDirt::FILTHY))
            .is_empty()
        {
            return;
        }

        let Some(shape) = self.runtime_shapes.get(shape_local) else {
            return;
        };
        shape.world_bounds.set(None);
        shape.world_length.set(None);
        let _path_flags = self.runtime_shape_paint_container_path_flags(shape_local);
        for (_flag, path_kind, runtime_kind) in [
            (
                crate::shapes::shape_paint_container::PATH_FLAG_LOCAL,
                ShapePaintPathKind::Local,
                RuntimeShapePaintPathKind::Local,
            ),
            (
                crate::shapes::shape_paint_container::PATH_FLAG_LOCAL_CLOCKWISE,
                ShapePaintPathKind::LocalClockwise,
                RuntimeShapePaintPathKind::LocalClockwise,
            ),
            (
                crate::shapes::shape_paint_container::PATH_FLAG_WORLD,
                ShapePaintPathKind::World,
                RuntimeShapePaintPathKind::World,
            ),
        ] {
            // C++ materializes only spaces named by `pathFlags`. Rust's
            // renderer-neutral geometry and hit-test APIs share these three
            // retained CPU slots, so they remain eagerly available even when
            // no current paint selects one. The aggregate above is still the
            // authoritative Shape::isFlagged value used by deferral.
            let mut commands = self.runtime_shape_path_commands_from_owners(
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
        crate::shapes::shape_paint_container::invalidate_stroke_effects(
            shape.paint_container_family.unwrap_or(
                crate::shapes::shape_paint_container::RuntimeShapePaintContainerFamily::Shape,
            ),
            &shape.paint_owners,
            |paint| paint.invalidate_all_effects(),
        );
    }
}

pub(crate) fn needs_clockwise_reversal(
    determinant: f32,
    designed_clockwise: bool,
    is_hole: bool,
) -> bool {
    let winding = if designed_clockwise { 1.0 } else { -1.0 };
    (determinant * winding < 0.0) != is_hole
}
