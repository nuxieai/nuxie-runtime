use super::*;
use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, ComponentDirt};

/// Direct port of `NSlicedNode::markPathDirtyRecursive`. The nearest layout
/// ancestor is the C++ `LayoutComponent::markLayoutNodeDirty` recipient.
pub(crate) fn mark_path_dirty_recursive(
    instance: &mut ArtboardInstance,
    local_id: usize,
    send_to_layout: bool,
) -> bool {
    let mut changed = instance.add_dirt(local_id, ComponentDirt::N_SLICER, true);
    if !send_to_layout {
        return changed;
    }

    let mut parent = instance.component_parent_local(local_id);
    let mut visited = BTreeSet::new();
    while let Some(parent_local) = parent {
        if !visited.insert(parent_local) {
            break;
        }
        let Some(component) = instance.component(parent_local) else {
            break;
        };
        if component.type_name == "LayoutComponent" {
            changed |= instance.mark_layout_node_changed(parent_local);
            break;
        }
        parent = instance.component_parent_local(parent_local);
    }
    changed
}

pub(crate) fn axis_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    mark_path_dirty_recursive(instance, local_id, true)
}

/// Direct port of `NSlicedNode::{width,height}Changed`.
pub(crate) fn size_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    let is_size = property_key_for_name("NSlicedNode", "width") == Some(property_key)
        || property_key_for_name("NSlicedNode", "height") == Some(property_key);
    is_size.then(|| mark_path_dirty_recursive(instance, local_id, true))
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeNSlicedNodeContext {
    world: Mat2D,
    inverse_world: Mat2D,
    width: f32,
    height: f32,
    scale_x: f32,
    scale_y: f32,
    x_px_stops: Vec<f32>,
    y_px_stops: Vec<f32>,
    x_scale_info: RuntimeNSlicerScaleInfo,
    y_scale_info: RuntimeNSlicerScaleInfo,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeNSlicerScaleInfo {
    pub(super) use_scale: bool,
    pub(super) scale_factor: f32,
    pub(super) fallback_size: f32,
}

impl WeightedPathContext<'_> {
    pub(super) fn translation(&self, vertex_local: usize) -> Option<(f32, f32)> {
        Some(
            self.instance
                .runtime_vertex_weight_state(vertex_local)?
                .translation,
        )
    }

    pub(super) fn cubic_translations(
        &self,
        vertex_local: usize,
    ) -> Option<((f32, f32), (f32, f32))> {
        let state = self.instance.runtime_vertex_weight_state(vertex_local)?;
        Some((state.in_translation, state.out_translation))
    }
}

impl RuntimeNSlicedNodeContext {
    fn deform_world_point(&self, x: f32, y: f32) -> (f32, f32) {
        let (local_x, local_y) = self.inverse_world.map_point(x, y);
        let sliced_x = if self.scale_x == 0.0 {
            0.0
        } else {
            runtime_nslicer_map_value(
                &self.x_px_stops,
                self.x_scale_info,
                self.width.abs(),
                local_x,
            ) * runtime_copysign_one(self.scale_x)
        };
        let sliced_y = if self.scale_y == 0.0 {
            0.0
        } else {
            runtime_nslicer_map_value(
                &self.y_px_stops,
                self.y_scale_info,
                self.height.abs(),
                local_y,
            ) * runtime_copysign_one(self.scale_y)
        };
        self.world.map_point(sliced_x, sliced_y)
    }
}

pub(super) fn runtime_nsliced_node_context_for_shape(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    shape_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Option<RuntimeNSlicedNodeContext> {
    if graph.n_slicer_details.is_empty() {
        return None;
    }

    let runtime = instance.runtime_file()?;
    let deformer = graph.shape_deformers.iter().find(|deformer| {
        deformer.shape_local == shape_local && deformer.deformer_type_name == Some("NSlicedNode")
    })?;
    let deformer_local = deformer.deformer_local?;
    let details = instance.runtime_meshes.details(deformer_local)?;
    if details.type_name != "NSlicedNode" {
        return None;
    }
    let object = runtime.object(details.global_id as usize);
    let initial_width = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "initialWidth",
        0.0,
    );
    let initial_height = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "initialHeight",
        0.0,
    );
    if initial_width <= 0.0 || initial_height <= 0.0 || object.is_none() {
        return None;
    }

    let authored_width = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "width",
        0.0,
    );
    let authored_height = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "height",
        0.0,
    );
    let control_size =
        runtime_nsliced_node_layout_control_size(instance, graph, details.local_id, layout_bounds);
    let width = control_size
        .map(|bounds| bounds.width)
        .unwrap_or(authored_width);
    let height = control_size
        .map(|bounds| bounds.height)
        .unwrap_or(authored_height);
    let world = instance.runtime_component_world_transform_with_bounds(
        details.local_id,
        graph,
        layout_bounds,
    );
    let inverse_world = runtime_mat2d_invert(world)?;
    let scale_x = width / initial_width;
    let scale_y = height / initial_height;
    let x_px_stops = runtime_nslicer_px_stops(runtime, instance, &details.x_axes, initial_width);
    let y_px_stops = runtime_nslicer_px_stops(runtime, instance, &details.y_axes, initial_height);
    let x_uv_stops = runtime_nslicer_uv_stops(runtime, instance, &details.x_axes, initial_width);
    let y_uv_stops = runtime_nslicer_uv_stops(runtime, instance, &details.y_axes, initial_height);

    Some(RuntimeNSlicedNodeContext {
        world,
        inverse_world,
        width,
        height,
        scale_x,
        scale_y,
        x_px_stops,
        y_px_stops,
        x_scale_info: runtime_nslicer_analyze_uv_stops(&x_uv_stops, initial_width, scale_x.abs()),
        // Mirrors C++ NSlicedNode::updateMapWorldPoint, including the width
        // argument used for Y scale analysis.
        y_scale_info: runtime_nslicer_analyze_uv_stops(&y_uv_stops, initial_width, scale_y.abs()),
    })
}

pub(super) fn runtime_nsliced_node_layout_control_size(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    nsliced_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Option<RuntimeLayoutBounds> {
    let layout_bounds = layout_bounds?;
    let mut current_local = graph
        .components
        .iter()
        .find(|component| component.local_id == nsliced_local)?
        .parent_local?;
    let mut visited = BTreeSet::new();
    loop {
        // Same malformed parent-cycle boundary as
        // `runtime_layout_control_size_for_path` above.
        if !visited.insert(current_local) {
            return None;
        }
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == current_local)?;
        match component.type_name {
            "LayoutComponent" => {
                return layout_bounds.get(&current_local).copied().or_else(|| {
                    Some(instance.runtime_layout_component_bounds(current_local, graph))
                });
            }
            "Artboard" => return None,
            "Node" => return None,
            _ => current_local = component.parent_local?,
        }
    }
}

pub(super) fn runtime_deform_path_commands_with_nsliced_node(
    commands: &mut [RuntimePathCommand],
    context: &RuntimeNSlicedNodeContext,
    path_kind: ShapePaintPathKind,
    shape_world: Mat2D,
    inverse_shape_world: Mat2D,
) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                (*x, *y) = runtime_deform_path_point_with_nsliced_node(
                    *x,
                    *y,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                (*x1, *y1) = runtime_deform_path_point_with_nsliced_node(
                    *x1,
                    *y1,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
                (*x2, *y2) = runtime_deform_path_point_with_nsliced_node(
                    *x2,
                    *y2,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
                (*x3, *y3) = runtime_deform_path_point_with_nsliced_node(
                    *x3,
                    *y3,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
            }
            RuntimePathCommand::Close => {}
        }
    }
}

pub(super) fn runtime_deform_path_point_with_nsliced_node(
    x: f32,
    y: f32,
    context: &RuntimeNSlicedNodeContext,
    path_kind: ShapePaintPathKind,
    shape_world: Mat2D,
    inverse_shape_world: Mat2D,
) -> (f32, f32) {
    if path_kind == ShapePaintPathKind::World {
        return context.deform_world_point(x, y);
    }
    let (world_x, world_y) = shape_world.map_point(x, y);
    let (deformed_x, deformed_y) = context.deform_world_point(world_x, world_y);
    inverse_shape_world.map_point(deformed_x, deformed_y)
}

pub(super) fn runtime_deform_local_gradient_point_with_nsliced_node(
    x: f32,
    y: f32,
    context: &RuntimeNSlicedNodeContext,
    shape_world: Mat2D,
    inverse_shape_world: Mat2D,
) -> (f32, f32) {
    let (world_x, world_y) = shape_world.map_point(x, y);
    let (deformed_x, deformed_y) = context.deform_world_point(world_x, world_y);
    inverse_shape_world.map_point(deformed_x, deformed_y)
}

pub(super) fn runtime_nslicer_uv_stops(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axes: &[NSlicerAxisNode],
    size: f32,
) -> Vec<f32> {
    debug_assert!(
        axes.iter()
            .all(|axis| n_slicer_details::axis_bucket(axis.type_name).is_some())
    );
    let mut stops = vec![0.0];
    for axis in axes {
        let offset = runtime_axis_offset(runtime, instance, axis);
        if runtime_axis_normalized(runtime, instance, axis) {
            stops.push(offset.clamp(0.0, 1.0));
        } else {
            stops.push((offset / size).clamp(0.0, 1.0));
        }
    }
    stops.push(1.0);
    stops.sort_by(f32::total_cmp);
    stops
}

pub(super) fn runtime_nslicer_px_stops(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axes: &[NSlicerAxisNode],
    size: f32,
) -> Vec<f32> {
    let mut stops = vec![0.0];
    for axis in axes {
        let offset = runtime_axis_offset(runtime, instance, axis);
        if runtime_axis_normalized(runtime, instance, axis) {
            stops.push(offset.clamp(0.0, 1.0) * size);
        } else {
            stops.push(offset.clamp(0.0, size));
        }
    }
    stops.push(size);
    stops.sort_by(f32::total_cmp);
    stops
}

pub(super) fn runtime_nslicer_analyze_uv_stops(
    stops: &[f32],
    size: f32,
    scale: f32,
) -> RuntimeNSlicerScaleInfo {
    let mut fixed_pct = 0.0;
    let mut empty_patch_count = 0;
    for index in 0..stops.len().saturating_sub(1) {
        let range = stops[index + 1] - stops[index];
        if runtime_nslicer_is_fixed_segment(index) {
            fixed_pct += range;
        } else if range == 0.0 {
            empty_patch_count += 1;
        }
    }

    let fixed_size = fixed_pct * size;
    let scalable_size = size - fixed_size;
    let use_scale = scalable_size != 0.0;
    let scale_factor = if use_scale {
        size.mul_add(scale, -fixed_size) / scalable_size
    } else {
        0.0
    };
    let fallback_size = if !use_scale && empty_patch_count != 0 {
        (size - fixed_size / scale) / empty_patch_count as f32
    } else {
        0.0
    };
    RuntimeNSlicerScaleInfo {
        use_scale,
        scale_factor,
        fallback_size,
    }
}

pub(super) fn runtime_nslicer_map_value(
    stops: &[f32],
    scale_info: RuntimeNSlicerScaleInfo,
    size: f32,
    value: f32,
) -> f32 {
    let Some(first) = stops.first().copied() else {
        return value;
    };
    let Some(last) = stops.last().copied() else {
        return value;
    };
    if value < first - 0.01 {
        return value;
    }
    if value > last + 0.01 {
        return value - last + size;
    }

    let mut result = 0.0;
    for index in 0..stops.len().saturating_sub(1) {
        let found = value <= stops[index + 1];
        let span = if found {
            value - stops[index]
        } else {
            stops[index + 1] - stops[index]
        };
        if runtime_nslicer_is_fixed_segment(index) {
            result += span;
        } else if scale_info.use_scale {
            result = scale_info.scale_factor.mul_add(span, result);
        } else {
            result += scale_info.fallback_size;
        }
        if found {
            break;
        }
    }
    result
}

pub(super) fn runtime_nslicer_is_fixed_segment(index: usize) -> bool {
    index % 2 == 0
}

pub(super) fn runtime_axis_offset(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axis: &NSlicerAxisNode,
) -> f32 {
    runtime_component_double_property(
        runtime,
        instance,
        axis.local_id,
        axis.global_id,
        "Axis",
        "offset",
        0.0,
    )
}

pub(super) fn runtime_axis_normalized(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axis: &NSlicerAxisNode,
) -> bool {
    runtime_component_bool_property(
        runtime,
        instance,
        axis.local_id,
        axis.global_id,
        "Axis",
        "normalized",
        false,
    )
}
