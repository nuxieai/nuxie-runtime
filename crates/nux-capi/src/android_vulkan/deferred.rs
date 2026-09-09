//! Android's recorded producer and native Vulkan replay boundary.
use super::{ApiFailure, renderer_failure};
use crate::NuxStatus;
use nuxie::render_api::{
    BlendMode, ImageSampler, Mat2D, PersistentFactoryContext, RawPath, RenderCanvasFrame,
    RenderCanvasHandle,
};
use nuxie::{
    ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasShader, GpuCanvasShaderArtifact,
    GpuCanvasShaderLoad, GpuCanvasShaderProfile, ImageDecodeError, PersistentFactory, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderGpuCanvasShader, RenderImage, RenderPaint,
    RenderPath, RenderShader, Renderer,
};
use nuxie_renderer::deferred::cmd::{
    deferred_replayer::{DeferredFrameSink, DeferredReplayer},
    deferred_session::{DeferredSession, ReplayCaps},
    render_replay::RendererOwner,
};
use nuxie_renderer::{NativeVulkanFactory, NativeVulkanFrame, RenderMode, RendererError};
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub(crate) struct AndroidVulkanFactory {
    // The import factory is the source DeferredSession; its real render
    // context remains a separately borrowable owner used only by replay.
    pub(super) session: DeferredSession,
    pub(super) replayer: Rc<RefCell<DeferredReplayer>>,
    pub(super) native: PersistentFactory<NativeVulkanFactory>,
}

impl AndroidVulkanFactory {
    pub(super) fn new(inner: NativeVulkanFactory) -> Self {
        let mut native = PersistentFactory::new(inner);
        let caps = native
            .ore()
            .map(|ore| ReplayCaps::from(&*ore.borrow()))
            .unwrap_or_default();
        let mut session = DeferredSession::with_caps(caps);
        session.bind_render_context(native.persistent_context());
        Self {
            session,
            replayer: Rc::new(RefCell::new(DeferredReplayer::default())),
            native,
        }
    }

    pub(super) fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        self.native.borrow_mut().resize(width, height)
    }

    fn upload_rgba8_premul_srgb(
        &mut self,
        width: u32,
        height: u32,
        row_bytes: u32,
        pixels: &[u8],
    ) -> Result<Box<dyn RenderImage>, RendererError> {
        // Explicit host image uploads remain real retained foreign images.
        // DeferredRenderer registers them when a recorded draw references one.
        self.native
            .borrow_mut()
            .upload_canonical_rgba8_premul_srgb(width, height, row_bytes, pixels)
    }
}

impl crate::asset_hooks::AssetUploadFactory for AndroidVulkanFactory {
    fn upload_rgba8_premul_srgb(
        &mut self,
        width: u32,
        height: u32,
        row_bytes: u32,
        pixels: &[u8],
    ) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.upload_rgba8_premul_srgb(width, height, row_bytes, pixels)
            .map_err(|_| ImageDecodeError)
    }
}

impl Factory for AndroidVulkanFactory {
    fn ore(&mut self) -> Option<nuxie::render_api::OreContextHandle> {
        self.session.ore()
    }
    fn render_context(&mut self) -> Option<nuxie::render_api::PersistentFactoryContext> {
        self.session.render_context()
    }
    fn deferred_canvas_host(&mut self) -> Option<nuxie::render_api::DeferredCanvasHostHandle> {
        self.session.deferred_canvas_host()
    }
    fn make_render_canvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Box<dyn nuxie::render_api::RenderCanvas>, nuxie::render_api::RenderCanvasError>
    {
        self.native.make_render_canvas(width, height)
    }
    fn make_deferred_render_canvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Box<dyn nuxie::render_api::RenderCanvas>, nuxie::render_api::RenderCanvasError>
    {
        self.native.make_deferred_render_canvas(width, height)
    }
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.session
            .make_render_buffer(buffer_type, flags, size_in_bytes)
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
        self.session
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.session
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.session.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.session.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.session.make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.session.decode_image(data)
    }

    fn make_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.session.make_gpu_canvas_shader(shader)
    }

    fn load_gpu_canvas_shader(&mut self, shader: &GpuCanvasShader) -> GpuCanvasShaderLoad {
        self.session.load_gpu_canvas_shader(shader)
    }

    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        self.session.gpu_canvas_shader_profile()
    }
    fn make_gpu_canvas_shader_artifact(
        &mut self,
        shader: &GpuCanvasShaderArtifact,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.session.make_gpu_canvas_shader_artifact(shader)
    }
    fn load_gpu_canvas_shader_artifact(
        &mut self,
        shader: &GpuCanvasShaderArtifact,
    ) -> GpuCanvasShaderLoad {
        self.session.load_gpu_canvas_shader_artifact(shader)
    }
    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.session.make_gpu_canvas_shader_occurrence(prepared)
    }
}

enum ReplayFrame {
    Screen(NativeVulkanFrame),
    Canvas(Box<dyn RenderCanvasFrame>),
}

impl ReplayFrame {
    fn renderer(&mut self) -> &mut dyn Renderer {
        match self {
            Self::Screen(frame) => frame,
            Self::Canvas(frame) => frame.renderer(),
        }
    }
}

// The replayer owns renderer projections independently of the sink. Each
// projection retains the same frame slot, never a borrowed native pointer.
struct ReplayFrameRenderer(Rc<RefCell<Option<ReplayFrame>>>);

impl Renderer for ReplayFrameRenderer {
    fn save(&mut self) {
        self.0.borrow_mut().as_mut().unwrap().renderer().save();
    }
    fn restore(&mut self) {
        self.0.borrow_mut().as_mut().unwrap().renderer().restore();
    }
    fn transform(&mut self, transform: Mat2D) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .transform(transform);
    }
    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .draw_path(path, paint);
    }
    fn clip_path(&mut self, path: &dyn RenderPath) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .clip_path(path);
    }
    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend: BlendMode,
        opacity: f32,
    ) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .draw_image(image, sampler, blend, opacity);
    }
    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uvs: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend: BlendMode,
        opacity: f32,
    ) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .draw_image_mesh(
                image,
                sampler,
                vertices,
                uvs,
                indices,
                vertex_count,
                index_count,
                blend,
                opacity,
            );
    }
    fn modulate_opacity(&mut self, opacity: f32) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .modulate_opacity(opacity);
    }
}

// Replay GPU and offscreen content before opening the screen readback frame.
pub(super) struct AndroidVulkanFrameSink {
    native: PersistentFactory<NativeVulkanFactory>,
    clear_color: ColorInt,
    screen: Rc<RefCell<Option<ReplayFrame>>>,
    canvas: Rc<RefCell<Option<ReplayFrame>>>,
    failure: Option<ApiFailure>,
}

impl AndroidVulkanFrameSink {
    pub(super) fn new(
        native: PersistentFactory<NativeVulkanFactory>,
        clear_color: ColorInt,
    ) -> Self {
        Self {
            native,
            clear_color,
            screen: Rc::new(RefCell::new(None)),
            canvas: Rc::new(RefCell::new(None)),
            failure: None,
        }
    }

    pub(super) fn finish(mut self) -> Result<Vec<u8>, ApiFailure> {
        // Empty artboards still produce a cleared frame.
        if self.failure.is_none() && self.screen.borrow().is_none() {
            self.begin_screen_frame(0);
        }
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        let frame = self.screen.borrow_mut().take();
        let Some(ReplayFrame::Screen(frame)) = frame else {
            unreachable!("successful replay opened the screen frame")
        };
        frame.finish().map_err(renderer_failure)
    }
}

impl DeferredFrameSink for AndroidVulkanFrameSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.native.persistent_context().unwrap()
    }

    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.native.persistent_context()
    }

    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        assert_eq!(target, 0);
        if self.failure.is_some() {
            return None;
        }
        if self.screen.borrow().is_none() {
            let frame = self
                .native
                .borrow()
                .begin_frame(self.clear_color, RenderMode::Msaa);
            match frame {
                Ok(frame) => *self.screen.borrow_mut() = Some(ReplayFrame::Screen(frame)),
                Err(error) => {
                    self.failure = Some(renderer_failure(error));
                    return None;
                }
            }
        }
        Some(Rc::new(RefCell::new(Box::new(ReplayFrameRenderer(
            self.screen.clone(),
        )))))
    }

    fn begin_ore_frame(&mut self) {
        // Vulkan's native frame owns the external command buffer and brackets
        // ORE itself. Open it before replay, not with Metal's standalone ORE
        // descriptor (which has no Vulkan command buffer).
        self.begin_screen_frame(0);
    }

    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear_color: u32,
    ) -> Option<RendererOwner> {
        if self.failure.is_some() {
            return None;
        }
        let frame = canvas.borrow_mut().begin_frame(clear_color);
        match frame {
            Ok(frame) => *self.canvas.borrow_mut() = Some(ReplayFrame::Canvas(frame)),
            Err(error) => {
                self.failure = Some(ApiFailure::new(NuxStatus::RuntimeError, error.to_string()));
                return None;
            }
        }
        Some(Rc::new(RefCell::new(Box::new(ReplayFrameRenderer(
            self.canvas.clone(),
        )))))
    }

    fn end_canvas_content(&mut self) {
        let frame = self.canvas.borrow_mut().take();
        if let Some(ReplayFrame::Canvas(frame)) = frame {
            if let Err(error) = frame.finish() {
                self.failure.get_or_insert_with(|| {
                    ApiFailure::new(NuxStatus::RuntimeError, error.to_string())
                });
            }
        }
    }
}
