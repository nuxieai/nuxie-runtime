use crate::mechanical_port::source::{
    core::CoreHandle,
    layout::{axis::AxisType, n_slicer::NSlicer, n_slicer_tile_mode::NSlicerTileModeType},
    math::{mat2d::Mat2D, n_slicer_helpers::NSlicerHelpers, vec2d::Vec2D},
    shapes::mesh_drawable::{MeshType, RuntimeRenderBufferHandle},
};
use nuxie_render_api::{
    BlendMode, ImageSampler, RenderBufferFlags, RenderBufferType, RenderImage, Renderer,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
pub struct SliceMeshVertex {
    pub id: i32,
    pub uv: Vec2D,
    pub vertex: Vec2D,
}
#[derive(Clone, Copy)]
pub struct Corner {
    pub x: usize,
    pub y: usize,
}
const PATCH_CORNERS: [Corner; 4] = [
    Corner { x: 0, y: 0 },
    Corner { x: 1, y: 0 },
    Corner { x: 1, y: 1 },
    Corner { x: 0, y: 1 },
];
const TRIANGULATION: [u16; 6] = [0, 1, 3, 1, 2, 3];

pub struct SliceMesh {
    nslicer: Option<CoreHandle>,
    uvs: Vec<Vec2D>,
    vertices: Vec<Vec2D>,
    indices: Vec<u16>,
    vertex_render_buffer: Option<RuntimeRenderBufferHandle>,
    uv_render_buffer: Option<RuntimeRenderBufferHandle>,
    index_render_buffer: Option<RuntimeRenderBufferHandle>,
}

impl SliceMesh {
    pub fn empty() -> Self {
        Self {
            nslicer: None,
            uvs: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_render_buffer: None,
            uv_render_buffer: None,
            index_render_buffer: None,
        }
    }
    pub fn new(nslicer: CoreHandle) -> Self {
        Self {
            nslicer: Some(nslicer),
            ..Self::empty()
        }
    }

    pub fn nslicer_handle(&self) -> Option<CoreHandle> {
        self.nslicer.clone()
    }
    pub fn mesh_type(&self) -> MeshType {
        MeshType::NSlice
    }
    pub fn draw(
        &self,
        nslicer: &mut NSlicer,
        renderer: &mut dyn Renderer,
        render_image: &dyn RenderImage,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let Some(image) = nslicer.image_handle() else {
            return;
        };
        if self.vertex_render_buffer.is_none()
            || self.uv_render_buffer.is_none()
            || self.index_render_buffer.is_none()
        {
            return;
        }
        let Some((world, offset_x, offset_y)) =
            image.with_downcast::<crate::mechanical_port::source::shapes::image::Image, _>(
                |image| {
                    (
                        image.base.world_transform(),
                        -image.width() * image.base.origin_x(),
                        -image.height() * image.base.origin_y(),
                    )
                },
            )
        else {
            return;
        };
        renderer.transform(nuxie_render_api::Mat2D(*world.values()));
        renderer.translate(offset_x, offset_y);
        let vertex = self
            .vertex_render_buffer
            .as_ref()
            .map(|buffer| buffer.borrow());
        let uv = self.uv_render_buffer.as_ref().map(|buffer| buffer.borrow());
        let index = self
            .index_render_buffer
            .as_ref()
            .map(|buffer| buffer.borrow());
        renderer.draw_image_mesh(
            Some(render_image),
            sampler,
            vertex.as_deref().map(Box::as_ref),
            uv.as_deref().map(Box::as_ref),
            index.as_deref().map(Box::as_ref),
            self.vertices.len() as u32,
            self.indices.len() as u32,
            blend_mode,
            opacity,
        );
    }
    pub fn on_asset_loaded(&mut self, _image: Option<&dyn RenderImage>) {}

    fn update_buffers(&mut self, nslicer: &mut NSlicer) {
        let Some(factory) = nslicer
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
        else {
            return;
        };
        let vertex_bytes = self.vertices.len() * std::mem::size_of::<Vec2D>();
        if self
            .vertex_render_buffer
            .as_ref()
            .is_some_and(|b| b.borrow().size_in_bytes() != vertex_bytes)
        {
            self.vertex_render_buffer = None;
        }
        if self.vertex_render_buffer.is_none() && vertex_bytes != 0 {
            self.vertex_render_buffer =
                Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                    factory.make_render_buffer(
                        RenderBufferType::Vertex,
                        RenderBufferFlags::None,
                        vertex_bytes,
                    )
                }))));
        }
        if let Some(buffer) = self.vertex_render_buffer.as_ref() {
            let mut buffer = buffer.borrow_mut();
            {
                let mapped = buffer.map_mut();
                for (output, vertex) in mapped.chunks_exact_mut(8).zip(&self.vertices) {
                    output[..4].copy_from_slice(&vertex.x.to_ne_bytes());
                    output[4..].copy_from_slice(&vertex.y.to_ne_bytes());
                }
            }
            buffer.unmap();
        }
        let uv_bytes = self.uvs.len() * std::mem::size_of::<Vec2D>();
        if self
            .uv_render_buffer
            .as_ref()
            .is_some_and(|b| b.borrow().size_in_bytes() != uv_bytes)
        {
            self.uv_render_buffer = None;
        }
        if self.uv_render_buffer.is_none() && uv_bytes != 0 {
            self.uv_render_buffer =
                Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                    factory.make_render_buffer(
                        RenderBufferType::Vertex,
                        RenderBufferFlags::None,
                        uv_bytes,
                    )
                }))));
        }
        let uv_transform = nslicer
            .image_handle()
            .and_then(|image| {
                image.with_downcast::<crate::mechanical_port::source::shapes::image::Image, _>(
                    |image| image.render_image().map(|image| image.uv_transform()),
                )
            })
            .flatten()
            .unwrap_or_default();
        if let Some(buffer) = self.uv_render_buffer.as_ref() {
            let mut buffer = buffer.borrow_mut();
            {
                let mapped = buffer.map_mut();
                for (output, uv) in mapped.chunks_exact_mut(8).zip(&self.uvs) {
                    let uv = uv_transform.transform_point(nuxie_render_api::Vec2D::new(uv.x, uv.y));
                    output[..4].copy_from_slice(&uv.x.to_ne_bytes());
                    output[4..].copy_from_slice(&uv.y.to_ne_bytes());
                }
            }
            buffer.unmap();
        }
        let index_bytes = self.indices.len() * std::mem::size_of::<u16>();
        if self
            .index_render_buffer
            .as_ref()
            .is_some_and(|b| b.borrow().size_in_bytes() != index_bytes)
        {
            self.index_render_buffer = None;
        }
        if self.index_render_buffer.is_none() && index_bytes != 0 {
            self.index_render_buffer =
                Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                    factory.make_render_buffer(
                        RenderBufferType::Index,
                        RenderBufferFlags::None,
                        index_bytes,
                    )
                }))));
        }
        if let Some(buffer) = self.index_render_buffer.as_ref() {
            let mut buffer = buffer.borrow_mut();
            {
                let mapped = buffer.map_mut();
                for (output, index) in mapped.chunks_exact_mut(2).zip(&self.indices) {
                    output.copy_from_slice(&index.to_ne_bytes());
                }
            }
            buffer.unmap();
        }
    }

    fn uv_stops(&self, nslicer: &mut NSlicer, axis: AxisType) -> Vec<f32> {
        let Some((size, scale)) = nslicer.image_handle().and_then(|image| {
            image.with_downcast::<crate::mechanical_port::source::shapes::image::Image, _>(
                |image| {
                    if axis == AxisType::X {
                        (image.width(), image.render_scale_x().abs())
                    } else {
                        (image.height(), image.render_scale_y().abs())
                    }
                },
            )
        }) else {
            return Vec::new();
        };
        if size == 0.0 || scale == 0.0 {
            return Vec::new();
        }
        NSlicerHelpers::uv_stops(
            if axis == AxisType::X {
                nslicer.xs()
            } else {
                nslicer.ys()
            },
            size,
        )
    }
    fn vertex_stops(&self, nslicer: &mut NSlicer, stops: &[f32], axis: AxisType) -> Vec<f32> {
        let Some((size, scale)) = nslicer.image_handle().and_then(|image| {
            image.with_downcast::<crate::mechanical_port::source::shapes::image::Image, _>(
                |image| {
                    if axis == AxisType::X {
                        (image.width(), image.render_scale_x().abs())
                    } else {
                        (image.height(), image.render_scale_y().abs())
                    }
                },
            )
        }) else {
            return Vec::new();
        };
        if size == 0.0 || scale == 0.0 {
            return Vec::new();
        }
        let info = NSlicerHelpers::analyze_uv_stops(stops, size, scale);
        let mut result = Vec::new();
        let mut vertex = 0.0;
        let mut in_bounds = 0.0;
        for i in 0..stops.len() - 1 {
            result.push(in_bounds);
            let segment = size * (stops[i + 1] - stops[i]) / scale;
            if NSlicerHelpers::is_fixed_segment(i as i32) {
                vertex += segment
            } else if info.use_scale {
                vertex += segment * info.scale_factor
            } else {
                vertex += info.fallback_size
            }
            in_bounds = vertex.clamp(0.0, size);
        }
        result.push(in_bounds);
        result
    }

    fn tile_repeat(
        &self,
        nslicer: &mut NSlicer,
        vertices: &mut Vec<SliceMeshVertex>,
        indices: &mut Vec<u16>,
        box_vertices: &[SliceMeshVertex],
        start: u16,
    ) -> u16 {
        assert_eq!(box_vertices.len(), 4);
        let start_x = box_vertices[0].vertex.x;
        let start_y = box_vertices[0].vertex.y;
        let end_x = box_vertices[2].vertex.x;
        let end_y = box_vertices[2].vertex.y;
        let start_u = box_vertices[0].uv.x;
        let start_v = box_vertices[0].uv.y;
        let end_u = box_vertices[2].uv.x;
        let end_v = box_vertices[2].uv.y;
        let Some((width, height, sx, sy)) = nslicer.image_handle().and_then(|image| {
            image.with_downcast::<crate::mechanical_port::source::shapes::image::Image, _>(
                |image| {
                    (
                        image.width(),
                        image.height(),
                        image.render_scale_x().abs(),
                        image.render_scale_y().abs(),
                    )
                },
            )
        }) else {
            return 0;
        };
        if sx == 0.0 || sy == 0.0 {
            return 0;
        }
        let size_x = width * (end_u - start_u) / sx;
        let size_y = height * (end_v - start_v) / sy;
        if size_x.abs() < 1.0 || size_y.abs() < 1.0 {
            return 0;
        }
        let mut cur_y = start_y;
        let mut cur_vertex = start;
        let mut escape = 10000;
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
                let v0 = cur_vertex;
                let frac_x = if cur_x + size_x > end_x {
                    (end_x - cur_x) / size_x
                } else {
                    1.0
                };
                let end_u1 = start_u + (end_u - start_u) * frac_x;
                let end_v1 = start_v + (end_v - start_v) * frac_y;
                let end_x1 = cur_x + size_x * frac_x;
                let end_y1 = cur_y + size_y * frac_y;
                vertices.extend(
                    [
                        (Vec2D::new(start_u, start_v), Vec2D::new(cur_x, cur_y)),
                        (Vec2D::new(end_u1, start_v), Vec2D::new(end_x1, cur_y)),
                        (Vec2D::new(end_u1, end_v1), Vec2D::new(end_x1, end_y1)),
                        (Vec2D::new(start_u, end_v1), Vec2D::new(cur_x, end_y1)),
                    ]
                    .into_iter()
                    .map(|(uv, vertex)| {
                        let value = SliceMeshVertex {
                            id: cur_vertex as i32,
                            uv,
                            vertex,
                        };
                        cur_vertex += 1;
                        value
                    }),
                );
                indices.extend(TRIANGULATION.map(|t| v0 + t));
                cur_x += size_x;
            }
            cur_y += size_y;
        }
        cur_vertex - start
    }

    fn calc(&mut self, nslicer: &mut NSlicer) {
        self.vertices.clear();
        self.indices.clear();
        self.uvs.clear();
        let us = self.uv_stops(nslicer, AxisType::X);
        let vs = self.uv_stops(nslicer, AxisType::Y);
        let xs = self.vertex_stops(nslicer, &us, AxisType::X);
        let ys = self.vertex_stops(nslicer, &vs, AxisType::Y);
        let mut expanded = Vec::new();
        let mut vertex_index = 0u16;
        for patch_y in 0..vs.len() - 1 {
            for patch_x in 0..us.len() - 1 {
                let mode = nslicer
                    .tile_modes()
                    .get(&nslicer.patch_index(patch_x as i32, patch_y as i32))
                    .copied()
                    .unwrap_or(NSlicerTileModeType::STRETCH);
                if mode == NSlicerTileModeType::HIDDEN {
                    continue;
                }
                let v0 = vertex_index;
                let mut patch = Vec::new();
                for corner in PATCH_CORNERS {
                    let x = patch_x + corner.x;
                    let y = patch_y + corner.y;
                    let id = if mode != NSlicerTileModeType::REPEAT {
                        let id = vertex_index;
                        vertex_index += 1;
                        id as i32
                    } else {
                        -1
                    };
                    patch.push(SliceMeshVertex {
                        id,
                        uv: Vec2D::new(us[x], vs[y]),
                        vertex: Vec2D::new(xs[x], ys[y]),
                    });
                }
                if mode == NSlicerTileModeType::REPEAT {
                    let mut new_indices = Vec::new();
                    vertex_index +=
                        self.tile_repeat(nslicer, &mut expanded, &mut new_indices, &patch, v0);
                    self.indices.extend(new_indices);
                } else {
                    expanded.extend(patch);
                    self.indices.extend(TRIANGULATION.map(|t| v0 + t));
                }
            }
        }
        for vertex in expanded {
            self.vertices.push(vertex.vertex);
            self.uvs.push(vertex.uv);
        }
    }
    pub fn update(&mut self, nslicer: &mut NSlicer) {
        if nslicer.image_handle().and_then(|image| {
            image.with_downcast::<crate::mechanical_port::source::shapes::image::Image, _>(
                |image| image.image_asset().is_some(),
            )
        }) != Some(true)
        {
            return;
        }
        self.calc(nslicer);
        self.update_buffers(nslicer);
    }
}

impl Default for SliceMesh {
    fn default() -> Self {
        Self::empty()
    }
}
