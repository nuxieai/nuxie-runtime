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

    /// Recreates only the extent-dependent target resources, preserving this
    /// factory's device, exact-source context, and render-resource domain.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        self.core.resize(width, height)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static LIVE_VULKAN_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock_live_vulkan_test() -> std::sync::MutexGuard<'static, ()> {
        LIVE_VULKAN_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn resize_preserves_factory_and_changes_the_readback_extent() {
        let _live_vulkan_test = lock_live_vulkan_test();
        let Ok(mut factory) = NativeVulkanFactory::new(2, 2) else {
            return;
        };

        factory.resize(3, 2).expect("resize Vulkan target");
        let pixels = factory
            .begin_frame(0xff102030, RenderMode::Msaa)
            .expect("begin resized frame")
            .finish()
            .expect("finish resized frame");

        assert_eq!(pixels.len(), 3 * 2 * 4);
        assert_eq!(&pixels[..4], &[16, 32, 48, 255]);
    }

    #[test]
    fn direct_rect_draw_probe() {
        let _live_vulkan_test = lock_live_vulkan_test();
        let Ok(mut factory) = NativeVulkanFactory::new(200, 200) else {
            return;
        };
        let mut frame = factory
            .begin_frame(0xFFFF00FF, RenderMode::Msaa)
            .expect("begin");
        let mut path = RawPath::new();
        path.move_to(50.0, 50.0);
        path.line_to(150.0, 50.0);
        path.line_to(150.0, 150.0);
        path.line_to(50.0, 150.0);
        path.close();
        let mut render_path = factory.make_render_path(path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.color(0xFF00FF00);
        paint.blend_mode(BlendMode::SrcOver);
        frame.draw_path(render_path.as_mut(), paint.as_mut());
        let pixels = frame.finish().expect("finish");
        let mut colors = std::collections::HashSet::new();
        for px in pixels.chunks_exact(4) {
            colors.insert([px[0], px[1], px[2], px[3]]);
        }
        let center = &pixels[((100 * 200 + 100) * 4)..((100 * 200 + 100) * 4 + 4)];
        eprintln!(
            "PROBE-DIRECT: {} colors, center={:02x?}",
            colors.len(),
            center
        );
        assert!(colors.len() >= 2, "nothing drawn at all");
        assert_ne!(
            center,
            [0xFFu8, 0x00, 0xFF, 0xFF],
            "center is still the clear color: rect not drawn"
        );
    }

    #[test]
    fn artboard_like_draw_probe() {
        let _live_vulkan_test = lock_live_vulkan_test();
        let Ok(mut factory) = NativeVulkanFactory::new(200, 200) else {
            return;
        };
        let mut frame = factory
            .begin_frame(0xFFFF00FF, RenderMode::Msaa)
            .expect("begin");
        // Mimic the artboard sequence: fit transform, save, clip to the
        // artboard bounds, background, then a nested-transform shape.
        frame.transform(Mat2D([2.0, 0.0, 0.0, 2.0, 0.0, 50.0]));
        frame.save();
        let mut clip = RawPath::new();
        clip.move_to(0.0, 0.0);
        clip.line_to(100.0, 0.0);
        clip.line_to(100.0, 50.0);
        clip.line_to(0.0, 50.0);
        clip.close();
        let mut clip_path = factory.make_render_path(clip, FillRule::NonZero);
        frame.clip_path(clip_path.as_mut());
        // Background.
        let mut bg = RawPath::new();
        bg.move_to(0.0, 0.0);
        bg.line_to(100.0, 0.0);
        bg.line_to(100.0, 50.0);
        bg.line_to(0.0, 50.0);
        bg.close();
        let mut bg_path = factory.make_render_path(bg, FillRule::NonZero);
        let mut bg_paint = factory.make_render_paint();
        bg_paint.color(0xFF000000);
        frame.draw_path(bg_path.as_mut(), bg_paint.as_mut());
        // Nested-transform shape (the panel analog).
        frame.save();
        frame.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 10.0]));
        let mut rect = RawPath::new();
        rect.move_to(0.0, 0.0);
        rect.line_to(60.0, 0.0);
        rect.line_to(60.0, 20.0);
        rect.line_to(0.0, 20.0);
        rect.close();
        let mut rect_path = factory.make_render_path(rect, FillRule::NonZero);
        let mut rect_paint = factory.make_render_paint();
        rect_paint.color(0xFF808080);
        frame.draw_path(rect_path.as_mut(), rect_paint.as_mut());
        frame.restore();
        frame.restore();
        let pixels = frame.finish().expect("finish");
        // Panel analog center: artboard (30,20) -> device (60, 90).
        let panel = &pixels[((90 * 200 + 60) * 4)..((90 * 200 + 60) * 4 + 4)];
        // Background sample away from panel: artboard (80,45) -> device (160, 140).
        let bg_px = &pixels[((140 * 200 + 160) * 4)..((140 * 200 + 160) * 4 + 4)];
        eprintln!("PROBE-ARTLIKE: panel={panel:02x?} bg={bg_px:02x?}");
        assert_eq!(bg_px, [0x00u8, 0x00, 0x00, 0xFF], "background missing");
        assert_eq!(
            panel,
            [0x80u8, 0x80, 0x80, 0xFF],
            "clipped nested shape missing"
        );
    }

    #[test]
    fn three_overlapping_solid_paths_land_in_one_frame() {
        let _live_vulkan_test = lock_live_vulkan_test();
        let Ok(mut factory) = NativeVulkanFactory::new(200, 200) else {
            return;
        };
        let mut paths_and_paints = Vec::new();
        for (inset, color) in [(20.0, 0xffff0000), (50.0, 0xff00ff00), (80.0, 0xff0000ff)] {
            let mut path = RawPath::new();
            path.move_to(inset, inset);
            path.line_to(200.0 - inset, inset);
            path.line_to(200.0 - inset, 200.0 - inset);
            path.line_to(inset, 200.0 - inset);
            path.close();
            let render_path = factory.make_render_path(path, FillRule::NonZero);
            let mut paint = factory.make_render_paint();
            paint.color(color);
            paths_and_paints.push((render_path, paint));
        }

        let mut frame = factory
            .begin_frame(0xffff00ff, RenderMode::Msaa)
            .expect("begin");
        for (path, paint) in &mut paths_and_paints {
            frame.draw_path(path.as_mut(), paint.as_mut());
        }
        let pixels = frame.finish().expect("finish");
        let pixel = |x: usize, y: usize| &pixels[(y * 200 + x) * 4..][..4];

        assert_eq!(pixel(30, 30), [0xff, 0x00, 0x00, 0xff]);
        assert_eq!(pixel(60, 60), [0x00, 0xff, 0x00, 0xff]);
        assert_eq!(pixel(100, 100), [0x00, 0x00, 0xff, 0xff]);
    }
}
