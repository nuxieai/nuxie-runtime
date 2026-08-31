//! Public product seam for the exact browser WebGL2 translation.

use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasPipelineShaders,
    GpuCanvasPlan, GpuCanvasShader, GpuCanvasShaderArtifact, GpuCanvasShaderProfile,
    ImageDecodeError, ImageSampler, Mat2D, RawPath, RenderBuffer, RenderBufferFlags,
    RenderBufferType, RenderCanvas, RenderCanvasError, RenderGpuCanvasShader, RenderImage,
    RenderPaint, RenderPath, RenderShader, Renderer,
};
use std::sync::Arc;
use web_sys::HtmlCanvasElement;

use crate::exact_source_adapter::{ExactSourceFactoryCore, ExactSourceFrameCore};
use crate::mechanical_port::webgl2::WebGl2ProductBackend;
use crate::{RenderMode, RendererError};

/// An exact-source WebGL2 renderer factory bound to one browser canvas.
pub struct WebGl2Factory {
    core: ExactSourceFactoryCore<WebGl2ProductBackend>,
    adapter_name: String,
}

impl WebGl2Factory {
    pub fn new(canvas: HtmlCanvasElement, width: u32, height: u32) -> Result<Self, RendererError> {
        let backend = WebGl2ProductBackend::new(canvas, width, height)?;
        let adapter_name = backend.adapter_name().to_owned();
        Ok(Self {
            core: ExactSourceFactoryCore::new(backend),
            adapter_name,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        self.core.resize(width, height)
    }

    pub fn begin_frame(
        &self,
        clear_color: u32,
        mode: RenderMode,
    ) -> Result<WebGl2Frame, RendererError> {
        self.core
            .begin_frame(clear_color, mode)
            .map(|core| WebGl2Frame { core })
    }
}

impl Factory for WebGl2Factory {
    fn is_render_context(&self) -> bool {
        true
    }
    fn ore(&mut self) -> Option<nuxie_render_api::OreContextHandle> {
        self.core.ore()
    }
    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        self.core.gpu_canvas_shader_profile()
    }

    fn make_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.core.make_gpu_canvas_shader(shader)
    }

    fn make_gpu_canvas_shader_artifact(
        &mut self,
        artifact: &GpuCanvasShaderArtifact,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.core.make_gpu_canvas_shader_artifact(artifact)
    }

    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.core.make_gpu_canvas_shader_occurrence(prepared)
    }

    fn make_gpu_canvas_image_with_pipelines(
        &mut self,
        pipelines: &[GpuCanvasPipelineShaders],
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        self.core
            .make_gpu_canvas_image_with_pipelines(pipelines, plan)
    }

    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.core
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
        self.core
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
        self.core
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.core.make_render_path(path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.core.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.core.make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.core.decode_image(data)
    }

    fn make_render_canvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Box<dyn RenderCanvas>, RenderCanvasError> {
        self.core.make_render_canvas(width, height)
    }
    fn make_deferred_render_canvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Box<dyn RenderCanvas>, RenderCanvasError> {
        self.core.make_deferred_render_canvas(width, height)
    }
}

/// One active exact-source browser WebGL2 frame.
pub struct WebGl2Frame {
    core: ExactSourceFrameCore<WebGl2ProductBackend>,
}

impl WebGl2Frame {
    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        self.core.finish()
    }

    /// Presents one browser frame without forcing a GPU-to-CPU readback.
    pub fn finish_present(self) -> Result<(), RendererError> {
        self.core.finish_without_readback()
    }
}

impl Renderer for WebGl2Frame {
    fn save(&mut self) {
        self.core.save();
    }

    fn restore(&mut self) {
        self.core.restore();
    }

    fn transform(&mut self, transform: Mat2D) {
        self.core.transform(transform);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.core.draw_path(path, paint);
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        self.core.clip_path(path);
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.core.draw_image(image, sampler, blend_mode, opacity);
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
        self.core.draw_image_mesh(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            blend_mode,
            opacity,
        );
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.core.modulate_opacity(opacity);
    }
}
