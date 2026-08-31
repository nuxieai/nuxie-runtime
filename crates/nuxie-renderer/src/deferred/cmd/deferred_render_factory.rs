//! renderer/cmd/deferred_render_factory.hpp at e949498e.
use super::{
    command_stream::WirePod,
    deferred_cmd::sniff_image_size,
    deferred_render_resource::*,
    foreign_image_registry::ForeignImageRegistry,
    id_allocator::IdAllocator,
    render_command_buffer::{RenderCommandBuffer, SharedIdAllocator},
    render_commands::*,
    render_handle::INVALID_RENDER_HANDLE,
};
use nuxie_render_api::*;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

pub struct DeferredFactory {
    pub buffer: SharedRenderCommandBuffer,
    path_ids: SharedIdAllocator,
    paint_ids: SharedIdAllocator,
    shader_ids: SharedIdAllocator,
    image_ids: SharedIdAllocator,
    buffer_ids: SharedIdAllocator,
}
impl Default for DeferredFactory {
    fn default() -> Self {
        Self::new()
    }
}
impl DeferredFactory {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(RenderCommandBuffer::default())),
            path_ids: Arc::new(Mutex::new(IdAllocator::default())),
            paint_ids: Arc::new(Mutex::new(IdAllocator::default())),
            shader_ids: Arc::new(Mutex::new(IdAllocator::default())),
            image_ids: Arc::new(Mutex::new(IdAllocator::default())),
            buffer_ids: Arc::new(Mutex::new(IdAllocator::default())),
        }
    }
    pub fn make_renderer(
        &self,
        canvases: Option<Rc<RefCell<ForeignImageRegistry>>>,
    ) -> DeferredRenderer {
        DeferredRenderer::new(self.buffer.clone(), canvases, None, SCREEN_TARGET)
    }
    pub fn reset_frame(&mut self) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.reset();
        buffer.drain_destroys();
    }
    fn allocate(&self, kind: ResourceKind, allocator: &SharedIdAllocator) -> DeferredResourceBase {
        let allocation = allocator.lock().unwrap().alloc();
        DeferredResourceBase::new(
            kind,
            allocation.id,
            allocation.generation,
            &self.buffer,
            allocator.clone(),
        )
    }
    fn gradient_blobs(&self, colors: &[ColorInt], stops: &[f32]) -> (u64, u64) {
        let mut color_bytes = Vec::with_capacity(colors.len() * 4);
        for color in colors {
            color.encode(&mut color_bytes);
        }
        let mut stop_bytes = Vec::with_capacity(stops.len() * 4);
        for stop in stops {
            stop.encode(&mut stop_bytes);
        }
        let mut buffer = self.buffer.lock().unwrap();
        (
            buffer.append_blob(&color_bytes),
            buffer.append_blob(&stop_bytes),
        )
    }
}
impl Drop for DeferredFactory {
    fn drop(&mut self) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.unregister_recorder();
        buffer.drain_destroys();
    }
}
impl Factory for DeferredFactory {
    fn make_render_path(&mut self, path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        let base = self.allocate(ResourceKind::Path, &self.path_ids);
        let (verbs, points) = raw_path_bytes(&path);
        let mut buffer = self.buffer.lock().unwrap();
        let blob_offset = buffer.append_blob(&verbs);
        let points_offset = buffer.append_blob(&points);
        buffer.append(
            RenderCmd::MakePath,
            &MakePathPod {
                id: base.id,
                generation: base.generation(),
                blob_offset,
                points_offset,
                verb_count: path.verbs().len() as u32,
                point_count: path.points().len() as u32,
                fill_rule: fill_rule as u32,
                pad: 0,
            },
        );
        drop(buffer);
        Box::new(DeferredRenderPath::new(base))
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        let base = self.allocate(ResourceKind::Path, &self.path_ids);
        base.append(
            RenderCmd::MakeEmptyPath,
            &MakeIdPod {
                id: base.id,
                generation: base.generation(),
            },
        );
        Box::new(DeferredRenderPath::new(base))
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let base = self.allocate(ResourceKind::Paint, &self.paint_ids);
        base.append(
            RenderCmd::MakePaint,
            &MakeIdPod {
                id: base.id,
                generation: base.generation(),
            },
        );
        Box::new(DeferredRenderPaint::new(base))
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        let base = self.allocate(ResourceKind::Shader, &self.shader_ids);
        let (blob_offset, stops_offset) = self.gradient_blobs(colors, stops);
        base.append(
            RenderCmd::MakeLinearGradient,
            &LinearGradientPod {
                id: base.id,
                generation: base.generation(),
                sx,
                sy,
                ex,
                ey,
                blob_offset,
                stops_offset,
                count: colors.len() as u32,
                pad: 0,
            },
        );
        Box::new(DeferredRenderShader {
            base: Rc::new(base),
        })
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        let base = self.allocate(ResourceKind::Shader, &self.shader_ids);
        let (blob_offset, stops_offset) = self.gradient_blobs(colors, stops);
        base.append(
            RenderCmd::MakeRadialGradient,
            &RadialGradientPod {
                id: base.id,
                generation: base.generation(),
                cx,
                cy,
                radius,
                count: colors.len() as u32,
                blob_offset,
                stops_offset,
            },
        );
        Box::new(DeferredRenderShader {
            base: Rc::new(base),
        })
    }
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        let base = self.allocate(ResourceKind::Buffer, &self.buffer_ids);
        base.append(
            RenderCmd::MakeBuffer,
            &MakeBufferPod {
                id: base.id,
                generation: base.generation(),
                buffer_type: buffer_type as u8,
                flags: flags as u8,
                size_in_bytes: size_in_bytes as u32,
            },
        );
        Box::new(DeferredRenderBuffer::new(
            base,
            buffer_type,
            flags,
            size_in_bytes,
        ))
    }
    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let base = self.allocate(ResourceKind::Image, &self.image_ids);
        let mut width = 0;
        let mut height = 0;
        #[cfg(feature = "rive-decoders")]
        if let Some(bitmap) = nuxie_image_codec::decode_image_rgba_unbounded(data) {
            width = bitmap.width;
            height = bitmap.height;
        }
        if width == 0 || height == 0 {
            if let Some((w, h)) = sniff_image_size(data) {
                width = w;
                height = h;
            }
        }
        #[cfg(debug_assertions)]
        if width == 0 || height == 0 {
            static WARNED: AtomicUsize = AtomicUsize::new(0);
            if WARNED.fetch_add(1, Ordering::Relaxed) == 0 {
                eprintln!(
                    "DeferredFactory::decodeImage: image dims unknown at record time (decode failed, or built without RIVE_DECODERS); size-dependent layout will be wrong"
                );
            }
        }
        let mut buffer = self.buffer.lock().unwrap();
        let blob_offset = buffer.append_blob(data);
        buffer.append(
            RenderCmd::DecodeImage,
            &DecodeImagePod {
                id: base.id,
                generation: base.generation(),
                blob_offset,
                byte_count: data.len() as u32,
                width,
                height,
                pad: 0,
            },
        );
        drop(buffer);
        Ok(Box::new(DeferredRenderImage {
            base: Rc::new(base),
            width,
            height,
        }))
    }
}

pub trait DeferredRouteHost {
    fn route_to(&mut self, target: u64);
}
pub const SCREEN_TARGET_FLAG: u64 = 1 << 63;
pub const SCREEN_TARGET: u64 = SCREEN_TARGET_FLAG;
pub const fn screen_target(id: u64) -> u64 {
    SCREEN_TARGET_FLAG | id
}
pub const fn is_screen_target(target: u64) -> bool {
    target & SCREEN_TARGET_FLAG != 0
}
pub const fn screen_target_id(target: u64) -> u64 {
    target & !SCREEN_TARGET_FLAG
}

pub struct DeferredRenderer {
    pub buffer: SharedRenderCommandBuffer,
    canvases: Option<Rc<RefCell<ForeignImageRegistry>>>,
    route_host: Option<Rc<RefCell<dyn DeferredRouteHost>>>,
    route_target: u64,
}
impl DeferredRenderer {
    pub fn new(
        buffer: SharedRenderCommandBuffer,
        canvases: Option<Rc<RefCell<ForeignImageRegistry>>>,
        route_host: Option<Rc<RefCell<dyn DeferredRouteHost>>>,
        route_target: u64,
    ) -> Self {
        Self {
            buffer,
            canvases,
            route_host,
            route_target,
        }
    }
    fn route(&self) {
        if let Some(host) = &self.route_host {
            host.borrow_mut().route_to(self.route_target);
        }
    }
    fn image_id(&self, image: Option<&dyn RenderImage>) -> u32 {
        let Some(image) = image else {
            return INVALID_RENDER_HANDLE;
        };
        if let Some(image) = image.as_any().downcast_ref::<DeferredRenderImage>() {
            return image.base.id;
        }
        self.canvases
            .as_ref()
            .map_or(INVALID_RENDER_HANDLE, |registry| {
                registry.borrow_mut().image_draw_id(image)
            })
    }
    fn warn_foreign(what: &str) {
        static WARNED: AtomicUsize = AtomicUsize::new(0);
        if WARNED.fetch_add(1, Ordering::Relaxed) < 16 {
            eprintln!(
                "rive deferred: {what} with a foreign resource (made by a different factory), draw will be dropped"
            );
        }
    }
}
impl Renderer for DeferredRenderer {
    fn save(&mut self) {
        self.route();
        self.buffer.lock().unwrap().append_type(RenderCmd::Save);
    }
    fn restore(&mut self) {
        self.route();
        self.buffer.lock().unwrap().append_type(RenderCmd::Restore);
    }
    fn transform(&mut self, transform: Mat2D) {
        self.route();
        let [xx, xy, yx, yy, tx, ty] = transform.0;
        self.buffer.lock().unwrap().append(
            RenderCmd::Transform,
            &TransformPod {
                xx,
                xy,
                yx,
                yy,
                tx,
                ty,
            },
        );
    }
    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        DeferredRenderPath::flush_scratch_of(path);
        let (Some(path), Some(paint)) = (
            path.as_any().downcast_ref::<DeferredRenderPath>(),
            paint.as_any().downcast_ref::<DeferredRenderPaint>(),
        ) else {
            Self::warn_foreign("drawPath");
            return;
        };
        path.resource.mark_drawn();
        paint.mark_drawn();
        self.route();
        self.buffer.lock().unwrap().append(
            RenderCmd::DrawPath,
            &DrawPathPod {
                path: path.resource.base.id,
                paint: paint.resource.base.id,
                path_version: path.resource.version(),
                paint_version: paint.resource.version(),
            },
        );
    }
    fn clip_path(&mut self, path: &dyn RenderPath) {
        DeferredRenderPath::flush_scratch_of(path);
        let deferred = path.as_any().downcast_ref::<DeferredRenderPath>();
        if let Some(path) = deferred {
            path.resource.mark_drawn();
        }
        self.route();
        self.buffer.lock().unwrap().append(
            RenderCmd::ClipPath,
            &ClipPathPod {
                path: DeferredRenderPath::id_of_path(path),
                version: deferred.map_or(0, |p| p.resource.version()),
            },
        );
    }
    fn modulate_opacity(&mut self, opacity: f32) {
        self.route();
        self.buffer
            .lock()
            .unwrap()
            .append(RenderCmd::ModulateOpacity, &OpacityPod { opacity });
    }
    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let image = self.image_id(image);
        if image == INVALID_RENDER_HANDLE {
            Self::warn_foreign("drawImage");
            return;
        }
        self.route();
        self.buffer.lock().unwrap().append(
            RenderCmd::DrawImage,
            &DrawImagePod {
                image,
                wrap_x: sampler.wrap_x as u8,
                wrap_y: sampler.wrap_y as u8,
                filter: sampler.filter as u8,
                blend_mode: blend_mode as u8,
                opacity,
            },
        );
    }
    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let image = self.image_id(image);
        fn downcast(buffer: Option<&dyn RenderBuffer>) -> Option<&DeferredRenderBuffer> {
            buffer.and_then(|b| b.as_any().downcast_ref::<DeferredRenderBuffer>())
        }
        let (Some(vertices), Some(uv_coords), Some(indices)) =
            (downcast(vertices), downcast(uv_coords), downcast(indices))
        else {
            Self::warn_foreign("drawImageMesh");
            return;
        };
        if image == INVALID_RENDER_HANDLE {
            Self::warn_foreign("drawImageMesh");
            return;
        }
        vertices.resource.mark_drawn();
        uv_coords.resource.mark_drawn();
        indices.resource.mark_drawn();
        self.route();
        self.buffer.lock().unwrap().append(
            RenderCmd::DrawImageMesh,
            &DrawImageMeshPod {
                image,
                vertices: vertices.resource.base.id,
                uv_coords: uv_coords.resource.base.id,
                indices: indices.resource.base.id,
                vertex_version: vertices.resource.version(),
                uv_version: uv_coords.resource.version(),
                index_version: indices.resource.version(),
                vertex_count,
                index_count,
                wrap_x: sampler.wrap_x as u8,
                wrap_y: sampler.wrap_y as u8,
                filter: sampler.filter as u8,
                blend_mode: blend_mode as u8,
                opacity,
            },
        );
    }
}
