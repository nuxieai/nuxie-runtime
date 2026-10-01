//! Public product seam for the exact native Dawn WebGPU translation.

use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasPipelineShaders,
    GpuCanvasPlan, GpuCanvasShader, GpuCanvasShaderArtifact, GpuCanvasShaderProfile,
    ImageDecodeError, ImageSampler, Mat2D, RawPath, RenderBuffer, RenderBufferFlags,
    RenderBufferType, RenderCanvas, RenderCanvasError, RenderGpuCanvasShader, RenderImage,
    RenderPaint, RenderPath, RenderShader, Renderer,
};
use std::sync::Arc;

use crate::exact_source_adapter::{ExactSourceFactoryCore, ExactSourceFrameCore};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use crate::external_image::ExternalImageTextures;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;
use crate::mechanical_port::webgpu::WebGpuProductBackend;
use crate::{RenderMode, RendererError};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use std::rc::Rc;

/// A headless exact-source native Dawn WebGPU renderer factory.
pub struct NativeWebGpuFactory {
    core: ExactSourceFactoryCore<WebGpuProductBackend>,
    adapter_name: String,
}

impl NativeWebGpuFactory {
    pub fn ore_target_desc(&self) -> nuxie_ore_metal::context::TargetDesc {
        self.core.ore_target_desc()
    }

    pub fn ore_render_target(&self) -> Option<nuxie_ore_metal::context::RenderTargetInfo> {
        self.core.ore_render_target()
    }

    pub fn set_target_preserved(&self, preserved: bool) {
        self.core.set_target_preserved(preserved);
    }
    pub fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        let backend = WebGpuProductBackend::new(width, height)?;
        let adapter_name = backend.adapter_name().to_owned();
        Ok(Self {
            core: ExactSourceFactoryCore::new(backend),
            adapter_name,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        self.core.resize(width, height)
    }

    /// Copy a browser image source (a canvas, a WebCodecs `VideoFrame`, an
    /// `ImageBitmap`, a video element) of `width` by `height`
    /// display pixels into one of `textures` with
    /// `GPUQueue.copyExternalImageToTexture`, and return it as an image the
    /// ordinary image paint draws. It is stored as decoded images are: sRGB
    /// with premultiplied alpha, top row first. No pixels pass through the CPU.
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    pub fn copy_external_image(
        &self,
        textures: &mut ExternalImageTextures,
        source: &wasm_bindgen::JsValue,
        width: u32,
        height: u32,
    ) -> Result<Rc<dyn RenderImage>, RendererError> {
        self.core
            .check_texture_extent("external image", width, height)?;
        let image = textures.0.next(
            |image| {
                self.core.owns_image(image) && (image.width(), image.height()) == (width, height)
            },
            RiveRenderImageHandle::is_sole_owner,
            || {
                let texture = self.core.with_backend_mut(|backend| {
                    backend.make_external_image_texture(width, height)
                })?;
                // SAFETY: the backend returned a live texture of this device.
                unsafe { self.core.adopt_texture(texture) }
            },
        )?;
        self.core.with_backend_mut(|backend| {
            backend.copy_external_image(source, &image, width, height)
        })?;
        Ok(image as Rc<dyn RenderImage>)
    }

    /// Upload a decoded SDR frame into this factory's resource domain.
    pub fn upload_canonical_rgba8_premul_srgb(
        &self,
        width: u32,
        height: u32,
        row_bytes: u32,
        pixels: &[u8],
    ) -> Result<Box<dyn RenderImage>, RendererError> {
        self.core
            .upload_rgba8_premul_srgb(width, height, row_bytes, pixels)
    }

    pub fn begin_frame(
        &self,
        clear_color: u32,
        mode: RenderMode,
    ) -> Result<NativeWebGpuFrame, RendererError> {
        self.core
            .begin_frame(clear_color, mode)
            .map(|core| NativeWebGpuFrame { core })
    }
}

impl Factory for NativeWebGpuFactory {
    fn supports_layer_mask(&self) -> bool {
        self.core.supports_layer_mask()
    }
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

    fn make_gpu_canvas_image_view(
        &mut self,
        image: std::rc::Rc<dyn RenderImage>,
    ) -> Result<std::rc::Rc<dyn RenderImage>, GpuCanvasError> {
        self.core.make_gpu_canvas_image_view(image)
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
    fn ensure_canvas_backing(&mut self, canvas: &nuxie_render_api::RenderCanvasHandle) {
        self.core.ensure_canvas_backing(canvas);
    }
}

/// One active exact-source native Dawn WebGPU frame.
pub struct NativeWebGpuFrame {
    core: ExactSourceFrameCore<WebGpuProductBackend>,
}

impl NativeWebGpuFrame {
    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        self.core.finish()
    }

    /// Presents one browser frame without forcing an asynchronous GPU readback.
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    pub fn finish_present(self) -> Result<(), RendererError> {
        self.core.finish_without_readback()
    }
}

impl Renderer for NativeWebGpuFrame {
    fn apply_layer_mask(
        &mut self,
        mask: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        mode: nuxie_render_api::LayerMaskMode,
    ) {
        self.core.apply_layer_mask(mask, sampler, mode);
    }
    fn draw_image_mesh_instanced(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        instances: Option<&nuxie_render_api::ImageMeshInstancesHandle>,
    ) {
        self.core.draw_image_mesh_instanced(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            instances,
        );
    }
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

    fn clip_stroke(&mut self, path: &dyn RenderPath, params: &nuxie_render_api::StrokeParams) {
        self.core.clip_stroke(path, params);
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

    fn draw_image_with_additiveness(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
        additiveness: f32,
    ) {
        self.core
            .draw_image_with_additiveness(image, sampler, blend_mode, opacity, additiveness);
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

    fn draw_image_mesh_with_additiveness(
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
        additiveness: f32,
    ) {
        self.core.draw_image_mesh_with_additiveness(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            blend_mode,
            opacity,
            additiveness,
        );
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.core.modulate_opacity(opacity);
    }
    fn modulate_color(&mut self, color: u32, replace: bool) {
        self.core.modulate_color(color, replace);
    }

    fn current_transform(&self) -> Option<Mat2D> {
        self.core.current_transform()
    }

    fn current_modulated_opacity(&self) -> Option<f32> {
        self.core.current_modulated_opacity()
    }
}
