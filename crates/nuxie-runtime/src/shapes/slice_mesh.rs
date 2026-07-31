use super::*;
use nuxie_render_api::RenderBuffer;

/// Concrete uniquely-owned SliceMesh backend and settled CPU state.
pub(crate) struct RuntimeSliceMeshOwner {
    pub(crate) local_id: usize,
    pub(crate) vertices: Option<Box<dyn RenderBuffer>>,
    pub(crate) uv_coords: Option<Box<dyn RenderBuffer>>,
    pub(crate) indices: Option<Box<dyn RenderBuffer>>,
    pub(crate) vertex_count: u32,
    pub(crate) index_count: u32,
    pub(crate) context_id: Option<u64>,
    pub(crate) dirty: bool,
    pub(crate) settled_update: Option<RuntimeSliceMeshUpdate>,
}

impl RuntimeSliceMeshOwner {
    pub(crate) fn new(local_id: usize) -> Self {
        Self {
            local_id,
            vertices: None,
            uv_coords: None,
            indices: None,
            vertex_count: 0,
            index_count: 0,
            context_id: None,
            dirty: true,
            settled_update: None,
        }
    }
}

impl Clone for RuntimeSliceMeshOwner {
    fn clone(&self) -> Self {
        // NSlicer constructs a new uniquely-owned SliceMesh; backend and CPU
        // state do not cross artboard occurrences.
        Self::new(self.local_id)
    }
}

impl std::fmt::Debug for RuntimeSliceMeshOwner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeSliceMeshOwner")
            .field("local_id", &self.local_id)
            .field("dirty", &self.dirty)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeSliceMeshUpdate {
    pub(crate) vertex_bytes: Vec<u8>,
    pub(crate) uv_bytes: Vec<u8>,
    pub(crate) index_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeSliceMeshGeometry {
    pub(crate) vertices: Vec<(f32, f32)>,
    pub(crate) uvs: Vec<(f32, f32)>,
    pub(crate) indices: Vec<u16>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeSliceMeshVertex {
    pub(crate) vertex: (f32, f32),
    pub(crate) uv: (f32, f32),
}

/// Appends the fixed clockwise two-triangle topology used by every SliceMesh
/// quad. Checked addition preserves the C++ u16 index boundary.
pub(crate) fn push_triangulation(indices: &mut Vec<u16>, start: u16) {
    for offset in [0_u16, 1, 3, 1, 2, 3] {
        if let Some(index) = start.checked_add(offset) {
            indices.push(index);
        }
    }
}

pub(super) fn runtime_slice_mesh_geometry(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    details: &n_slicer_details::RuntimeNSlicerDetailsOwner,
    image_width: f32,
    image_height: f32,
    render_scale_x: f32,
    render_scale_y: f32,
) -> RuntimeSliceMeshGeometry {
    // Ported line-for-line from C++ `src/shapes/slice_mesh.cpp::calc`.
    if image_width == 0.0 || image_height == 0.0 || render_scale_x == 0.0 || render_scale_y == 0.0 {
        return RuntimeSliceMeshGeometry {
            vertices: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        };
    }

    let us = super::n_sliced_node::runtime_nslicer_uv_stops(
        runtime,
        instance,
        &details.x_axes,
        image_width,
    );
    let vs = super::n_sliced_node::runtime_nslicer_uv_stops(
        runtime,
        instance,
        &details.y_axes,
        image_height,
    );
    let xs = runtime_slice_mesh_vertex_stops(&us, image_width, render_scale_x);
    let ys = runtime_slice_mesh_vertex_stops(&vs, image_height, render_scale_y);
    if us.len() != xs.len() || vs.len() != ys.len() {
        return RuntimeSliceMeshGeometry {
            vertices: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        };
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_index = 0_u16;

    'patches: for patch_y in 0..vs.len().saturating_sub(1) {
        for patch_x in 0..us.len().saturating_sub(1) {
            let patch_index = details.patch_index(patch_x, patch_y).unwrap_or(u64::MAX);
            let tile_mode = details
                .tile_modes
                .get(&patch_index)
                .map_or(0, |mode| mode.style);
            if tile_mode == 2 {
                continue;
            }
            let patch_vertices = [
                RuntimeSliceMeshVertex {
                    vertex: (xs[patch_x], ys[patch_y]),
                    uv: (us[patch_x], vs[patch_y]),
                },
                RuntimeSliceMeshVertex {
                    vertex: (xs[patch_x + 1], ys[patch_y]),
                    uv: (us[patch_x + 1], vs[patch_y]),
                },
                RuntimeSliceMeshVertex {
                    vertex: (xs[patch_x + 1], ys[patch_y + 1]),
                    uv: (us[patch_x + 1], vs[patch_y + 1]),
                },
                RuntimeSliceMeshVertex {
                    vertex: (xs[patch_x], ys[patch_y + 1]),
                    uv: (us[patch_x], vs[patch_y + 1]),
                },
            ];
            if tile_mode == 1 {
                let added = runtime_slice_mesh_tile_repeat(
                    &mut vertices,
                    &mut indices,
                    patch_vertices,
                    vertex_index,
                    image_width,
                    image_height,
                    render_scale_x,
                    render_scale_y,
                );
                let Some(next) = vertex_index.checked_add(added) else {
                    break 'patches;
                };
                vertex_index = next;
                continue;
            }

            let Some(next) = vertex_index.checked_add(4) else {
                break 'patches;
            };
            vertices.extend_from_slice(&patch_vertices);
            push_triangulation(&mut indices, vertex_index);
            vertex_index = next;
        }
    }

    RuntimeSliceMeshGeometry {
        vertices: vertices.iter().map(|vertex| vertex.vertex).collect(),
        uvs: vertices.iter().map(|vertex| vertex.uv).collect(),
        indices,
    }
}

pub(super) fn runtime_slice_mesh_vertex_stops(
    normalized_stops: &[f32],
    image_size: f32,
    image_scale: f32,
) -> Vec<f32> {
    if image_size == 0.0 || image_scale == 0.0 {
        return Vec::new();
    }
    let scale_info = super::n_sliced_node::runtime_nslicer_analyze_uv_stops(
        normalized_stops,
        image_size,
        image_scale,
    );
    let mut vertices = Vec::with_capacity(normalized_stops.len());
    let mut vertex = 0.0;
    let mut vertex_in_bounds = 0.0;
    for index in 0..normalized_stops.len().saturating_sub(1) {
        vertices.push(vertex_in_bounds);
        let segment =
            image_size * (normalized_stops[index + 1] - normalized_stops[index]) / image_scale;
        if super::n_sliced_node::runtime_nslicer_is_fixed_segment(index) {
            vertex += segment;
        } else if scale_info.use_scale {
            vertex += segment * scale_info.scale_factor;
        } else {
            vertex += scale_info.fallback_size;
        }
        vertex_in_bounds = vertex.clamp(0.0, image_size);
    }
    vertices.push(vertex_in_bounds);
    vertices
}

#[allow(clippy::too_many_arguments)]
pub(super) fn runtime_slice_mesh_tile_repeat(
    vertices: &mut Vec<RuntimeSliceMeshVertex>,
    indices: &mut Vec<u16>,
    patch: [RuntimeSliceMeshVertex; 4],
    start: u16,
    image_width: f32,
    image_height: f32,
    render_scale_x: f32,
    render_scale_y: f32,
) -> u16 {
    let (start_x, start_y) = patch[0].vertex;
    let (end_x, end_y) = patch[2].vertex;
    let (start_u, start_v) = patch[0].uv;
    let (end_u, end_v) = patch[2].uv;
    if render_scale_x == 0.0 || render_scale_y == 0.0 {
        return 0;
    }
    let size_x = image_width * (end_u - start_u) / render_scale_x;
    let size_y = image_height * (end_v - start_v) / render_scale_y;
    if size_x.abs() < 1.0 || size_y.abs() < 1.0 {
        return 0;
    }

    let mut cur_y = start_y;
    let mut cur_vertex = u32::from(start);
    let mut escape = 10_000;
    while cur_y < end_y && escape > 0 {
        escape -= 1;
        let frac_y = if cur_y + size_y > end_y {
            (end_y - cur_y) / size_y
        } else {
            1.0
        };
        let mut cur_x = start_x;
        while cur_x < end_x && escape > 0 {
            escape -= 1;
            if cur_vertex > u32::from(u16::MAX) - 3 {
                return u16::MAX - start;
            }
            let frac_x = if cur_x + size_x > end_x {
                (end_x - cur_x) / size_x
            } else {
                1.0
            };
            let end_u1 = start_u + (end_u - start_u) * frac_x;
            let end_v1 = start_v + (end_v - start_v) * frac_y;
            let end_x1 = cur_x + size_x * frac_x;
            let end_y1 = cur_y + size_y * frac_y;
            let v0 = cur_vertex as u16;
            vertices.extend_from_slice(&[
                RuntimeSliceMeshVertex {
                    vertex: (cur_x, cur_y),
                    uv: (start_u, start_v),
                },
                RuntimeSliceMeshVertex {
                    vertex: (end_x1, cur_y),
                    uv: (end_u1, start_v),
                },
                RuntimeSliceMeshVertex {
                    vertex: (end_x1, end_y1),
                    uv: (end_u1, end_v1),
                },
                RuntimeSliceMeshVertex {
                    vertex: (cur_x, end_y1),
                    uv: (start_u, end_v1),
                },
            ]);
            push_triangulation(indices, v0);
            cur_vertex += 4;
            cur_x += size_x;
        }
        cur_y += size_y;
    }
    u16::try_from(cur_vertex - u32::from(start)).unwrap_or(u16::MAX - start)
}

pub(super) fn runtime_slice_mesh_update(
    geometry: RuntimeSliceMeshGeometry,
    uv_transform: RenderMat2D,
) -> RuntimeSliceMeshUpdate {
    let mut vertex_bytes = Vec::with_capacity(geometry.vertices.len() * 8);
    for (x, y) in geometry.vertices {
        push_f32_pair_bytes(&mut vertex_bytes, x, y);
    }
    let mut uv_bytes = Vec::with_capacity(geometry.uvs.len() * 8);
    for (u, v) in geometry.uvs {
        let uv = uv_transform.transform_point(RenderVec2D::new(u, v));
        push_f32_pair_bytes(&mut uv_bytes, uv.x, uv.y);
    }
    let mut index_bytes = Vec::with_capacity(geometry.indices.len() * 2);
    for index in geometry.indices {
        index_bytes.extend_from_slice(&index.to_le_bytes());
    }
    RuntimeSliceMeshUpdate {
        vertex_bytes,
        uv_bytes,
        index_bytes,
    }
}

pub(super) fn runtime_update_slice_mesh_render_buffers(
    factory: &mut dyn RenderFactory,
    owner: &mut RuntimeSliceMeshOwner,
    backend_context_id: u64,
) {
    if owner.context_id != Some(backend_context_id) {
        owner.vertices = None;
        owner.uv_coords = None;
        owner.indices = None;
        owner.context_id = Some(backend_context_id);
    }
    let Some(update) = owner.settled_update.as_ref() else {
        return;
    };
    runtime_update_slice_mesh_render_buffer(
        factory,
        &mut owner.vertices,
        RenderBufferType::Vertex,
        &update.vertex_bytes,
    );
    runtime_update_slice_mesh_render_buffer(
        factory,
        &mut owner.uv_coords,
        RenderBufferType::Vertex,
        &update.uv_bytes,
    );
    runtime_update_slice_mesh_render_buffer(
        factory,
        &mut owner.indices,
        RenderBufferType::Index,
        &update.index_bytes,
    );
    owner.vertex_count = u32::try_from(update.vertex_bytes.len() / 8).unwrap_or(u32::MAX);
    owner.index_count = u32::try_from(update.index_bytes.len() / 2).unwrap_or(u32::MAX);
}

pub(super) fn runtime_update_slice_mesh_render_buffer(
    factory: &mut dyn RenderFactory,
    buffer: &mut Option<Box<dyn RenderBuffer>>,
    buffer_type: RenderBufferType,
    bytes: &[u8],
) {
    if buffer
        .as_ref()
        .is_some_and(|buffer| buffer.size_in_bytes() != bytes.len())
    {
        *buffer = None;
    }
    if buffer.is_none() && !bytes.is_empty() {
        *buffer =
            Some(factory.make_render_buffer(buffer_type, RenderBufferFlags::None, bytes.len()));
    }
    if let Some(buffer) = buffer.as_deref_mut() {
        write_render_buffer_bytes(buffer, bytes);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn runtime_draw_slice_mesh_image(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    image_local: usize,
    image_object: Option<&RuntimeObject>,
    _resolved_image_asset_global: Option<u32>,
    owner: &RuntimeSliceMeshOwner,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    image: &dyn RenderImage,
    path_cache: &mut RuntimeArtboardPathState,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    let (Some(vertices), Some(uv_coords), Some(indices)) = (
        owner.vertices.as_deref(),
        owner.uv_coords.as_deref(),
        owner.indices.as_deref(),
    ) else {
        return Ok(());
    };
    let world = path_cache
        .image_world_transform_with_bounds(runtime, instance, graph, image_local, layout_bounds)?
        .unwrap_or_else(|| {
            path_cache.component_world_transform_with_bounds(
                instance,
                graph,
                image_local,
                layout_bounds,
            )
        });
    renderer.transform(runtime_render_mat(world));

    let origin_x_key =
        runtime_draw_property_key_for_name("Image", "originX").context("missing Image.originX")?;
    let origin_y_key =
        runtime_draw_property_key_for_name("Image", "originY").context("missing Image.originY")?;
    let origin_x = instance
        .double_property(image_local, origin_x_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_x_key)
            })
        })
        .unwrap_or(0.5);
    let origin_y = instance
        .double_property(image_local, origin_y_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_y_key)
            })
        })
        .unwrap_or(0.5);
    renderer.transform(RenderMat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        -(image.width() as f32 * origin_x),
        -(image.height() as f32 * origin_y),
    ]));

    let blend_mode_key = runtime_draw_property_key_for_name("Drawable", "blendModeValue")
        .context("missing Drawable.blendModeValue")?;
    let blend_mode_value = instance
        .uint_property(image_local, blend_mode_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_uint_property_by_key(object, blend_mode_key)
            })
        })
        .unwrap_or(3);
    let opacity = instance
        .component(image_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    renderer.draw_image_mesh(
        Some(image),
        RenderImageSampler::LINEAR_CLAMP,
        Some(vertices),
        Some(uv_coords),
        Some(indices),
        owner.vertex_count,
        owner.index_count,
        runtime_blend_mode(u32::try_from(blend_mode_value).unwrap_or(3))?,
        opacity,
    );
    Ok(())
}
