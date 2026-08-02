//! Retained Path lifecycle owner.
//!
//! CPU `RawPath` state lives on the occurrence-owned `RuntimePathOwner` in the
//! draw coordinator. Setter callbacks enter through the concrete vertex and
//! parametric modules, then schedule this owner with `ComponentDirt::PATH`.

use std::{collections::BTreeMap, sync::Arc};

use nuxie_graph::{ArtboardGraph, PathGeometryNode};

use crate::{
    ArtboardInstance, ComponentDirt, Mat2D, RuntimeLayoutBounds,
    components::ComponentHandle,
    draw::{
        RuntimeOwnedPath, WeightedPathContext, cubic_vertex_points,
        parametric_path_with_control_size, path_commands, runtime_path_geometry,
        vertex_translation,
    },
    math::raw_path::{prune_empty_path_segments, runtime_raw_path_from_commands},
    properties::property_key_for_name,
};

impl ArtboardInstance {
    /// Literal `Path::canDeferPathUpdate` / `Shape::canDeferPathUpdate`.
    ///
    /// The decision reads only occurrence-owned Shape/Path pointers, flags,
    /// opacity, and dependency relations. FollowPath and clipping producers
    /// therefore prevent deferral at the same owner boundary as C++
    /// (`src/shapes/path.cpp:111-125`; `src/shapes/shape.cpp:35-51`).
    pub(crate) fn runtime_path_can_defer_update(&self, path_handle: ComponentHandle) -> bool {
        let Some(path) = self
            .objects
            .component(path_handle)
            .and_then(|component| component.concrete.path.as_ref())
        else {
            return false;
        };
        let Some(shape_handle) = path.shape else {
            return false;
        };
        let Some(shape_component) = self.objects.component(shape_handle) else {
            return false;
        };
        let Some(_shape) = shape_component.concrete.shape.as_ref() else {
            return false;
        };
        let container_path_flags =
            self.runtime_shape_paint_container_path_flags(shape_component.local_id);
        let clipping_or_never_defer = container_path_flags
            & u64::from(
                crate::components::RuntimeShapeState::CLIPPING
                    | crate::components::RuntimeShapeState::NEVER_DEFER_UPDATE,
            )
            != 0;
        let has_skinned_path_dependent =
            shape_component.dependents.iter().copied().any(|dependent| {
                self.objects.component(dependent).is_some_and(|component| {
                    component.type_name == "PointsPath"
                        && component
                            .concrete
                            .skinnable
                            .as_ref()
                            .and_then(|skinnable| skinnable.skin)
                            .is_some()
                })
            });
        let follow_path_consumer = container_path_flags
            & u64::from(crate::components::RuntimeShapeState::FOLLOW_PATH)
            != 0
            || path.is_flagged(
                crate::components::RuntimePathState::FOLLOW_PATH
                    | crate::components::RuntimePathState::CLIPPING,
            );
        crate::shapes::shape::can_defer_path_update(
            shape_component.transform.render_opacity,
            clipping_or_never_defer,
            has_skinned_path_dependent,
            follow_path_consumer,
        )
    }

    /// Direct port of pinned C++ `Path::update`
    /// (`src/shapes/path.cpp:336-380`). The source RawPath belongs to the
    /// runtime Path occurrence and is settled before its PathComposer node.
    pub(crate) fn update_runtime_path_owner(
        &mut self,
        path_handle: ComponentHandle,
        dirt: ComponentDirt,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) {
        let Some(path_local) = self.objects.component_local_id(path_handle) else {
            return;
        };
        let Some(path) = graph.paths.iter().find(|path| path.local_id == path_local) else {
            return;
        };
        let shape_local = self
            .objects
            .component(path_handle)
            .and_then(|component| component.concrete.path.as_ref())
            .and_then(|path| path.shape)
            .and_then(|shape| self.objects.component_local_id(shape));
        let has_deformer = shape_local.is_some_and(|shape_local| {
            crate::draw::n_sliced_node::runtime_nsliced_node_context_for_shape(
                self,
                graph,
                shape_local,
                layout_bounds,
            )
            .is_some()
        });
        let Some((owner_dirty, owner_has_path)) = self
            .runtime_shapes
            .paths_by_local
            .get(path_local)
            .and_then(Option::as_ref)
            .map(|owner| (owner.dirty.get(), owner.retained.borrow().is_some()))
        else {
            return;
        };
        let should_rebuild = !owner_has_path
            || !(dirt & (ComponentDirt::PATH | ComponentDirt::N_SLICER)).is_empty()
            || (has_deformer && dirt.contains(ComponentDirt::WORLD_TRANSFORM));
        if !owner_dirty && !should_rebuild {
            return;
        }
        if !should_rebuild {
            if let Some(owner) = self
                .runtime_shapes
                .paths_by_local
                .get(path_local)
                .and_then(Option::as_ref)
            {
                owner.dirty.set(false);
            }
            return;
        }

        if self.runtime_path_can_defer_update(path_handle) {
            if let Some(path) = self
                .objects
                .component_mut(path_handle)
                .and_then(|component| component.concrete.path.as_mut())
            {
                path.deferred_path_dirt.set(true);
            }
            if let Some(owner) = self
                .runtime_shapes
                .paths_by_local
                .get(path_local)
                .and_then(Option::as_ref)
            {
                owner.dirty.set(false);
            }
            return;
        }

        if let Some(path) = self
            .objects
            .component_mut(path_handle)
            .and_then(|component| component.concrete.path.as_mut())
        {
            path.deferred_path_dirt.set(false);
        }

        let runtime_path = self.runtime_path_geometry_with_layout_control(path, layout_bounds);
        if self.runtime_skinnable_handle_has_skin(path_handle) {
            for vertex in &runtime_path.vertices {
                self.deform_runtime_vertex_weight(
                    vertex.local_id,
                    vertex_translation(vertex),
                    cubic_vertex_points(vertex),
                );
            }
        }
        let weighted_context = self
            .runtime_skinnable_handle_has_skin(path_handle)
            .then_some(WeightedPathContext { instance: self });
        let mut commands = path_commands(
            &runtime_path,
            nuxie_graph::ShapePaintPathKind::World,
            Mat2D::IDENTITY,
            weighted_context.as_ref(),
        );
        prune_empty_path_segments(&mut commands);
        let owner = self
            .runtime_shapes
            .paths_by_local
            .get(path_local)
            .and_then(Option::as_ref)
            .expect("Path owner must remain live during its update");
        owner.retained.replace(Some(RuntimeOwnedPath {
            path: Arc::new(runtime_path),
            raw_path: Arc::new(runtime_raw_path_from_commands(&commands)),
            has_weighted_context: weighted_context.is_some(),
        }));
        owner.dirty.set(false);
    }

    pub(crate) fn runtime_path_geometry_with_layout_control(
        &self,
        path: &PathGeometryNode,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> PathGeometryNode {
        let mut runtime_path = runtime_path_geometry(self, path);
        let Some(layout_bounds) = layout_bounds else {
            return runtime_path;
        };
        let Some(control_size) =
            self.runtime_layout_control_size_for_path(path.local_id, layout_bounds)
        else {
            return runtime_path;
        };
        if let Some(parametric) = runtime_path.parametric.take() {
            runtime_path.parametric = Some(parametric_path_with_control_size(
                parametric,
                control_size.width,
                control_size.height,
            ));
        }
        runtime_path
    }
}

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Path", "isHole") != Some(property_key) {
        return None;
    }
    Some(super::mark_path_dirty(artboard, local_id))
}
