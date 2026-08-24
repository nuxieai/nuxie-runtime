//! Public product seam for the exact native Vulkan translation.

use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, RawPath,
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPath,
    RenderShader, Renderer,
};

use crate::exact_source_adapter::{ExactSourceFactoryCore, ExactSourceFrameCore};
use crate::mechanical_port::vulkan::VulkanProductBackend;
use crate::{RenderMode, RendererError};

/// A headless exact-source Vulkan renderer factory.
pub struct NativeVulkanFactory {
    core: ExactSourceFactoryCore<VulkanProductBackend>,
    adapter_name: String,
}

impl NativeVulkanFactory {
    pub fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        let backend = VulkanProductBackend::new(width, height)?;
        let adapter_name = backend.adapter_name().to_owned();
        Ok(Self {
            core: ExactSourceFactoryCore::new(backend),
            adapter_name,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn begin_frame(
        &self,
        clear_color: u32,
        mode: RenderMode,
    ) -> Result<NativeVulkanFrame, RendererError> {
        self.core
            .begin_frame(clear_color, mode)
            .map(|core| NativeVulkanFrame { core })
    }
}

impl Factory for NativeVulkanFactory {
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
}

/// One active exact-source Vulkan frame.
pub struct NativeVulkanFrame {
    core: ExactSourceFrameCore<VulkanProductBackend>,
}

impl NativeVulkanFrame {
    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        self.core.finish()
    }
}

impl Renderer for NativeVulkanFrame {
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
