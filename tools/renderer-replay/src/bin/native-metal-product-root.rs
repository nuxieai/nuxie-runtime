#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_render_api::{Factory, FillRule, RawPath, Renderer};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_renderer::{NativeMetalExecutionInventory, NativeMetalFactory, RenderMode};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::rc::autoreleasepool;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_core_foundation::CGSize;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_metal::MTLPixelFormat;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_quartz_core::CAMetalLayer;

#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe extern "C" {
    fn pthread_main_np() -> std::ffi::c_int;
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn require_caller_drawable<T>(drawable: Option<T>) -> Result<T, &'static str> {
    drawable.ok_or("caller-owned CAMetalLayer returned no drawable")
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn main() {
    autoreleasepool(|_| {
        // SAFETY: `pthread_main_np` takes no arguments and only reports
        // whether this process entry point is executing on Darwin's main
        // thread; it does not retain or dereference Rust storage.
        assert_eq!(unsafe { pthread_main_np() }, 1);
        let mut factory = NativeMetalFactory::new(64, 64).expect("create native Metal tracer");
        let mut path = RawPath::new();
        path.move_to(8.0, 8.0);
        path.line_to(56.0, 8.0);
        path.line_to(56.0, 56.0);
        path.line_to(8.0, 56.0);
        path.close();
        let path = factory.make_render_path(path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.color(0xff00_ff00);

        // Root deterministic readback and its pixel oracle in the same final
        // executable that roots the caller-owned product presentation path.
        let mut frame = factory
            .begin_frame(0)
            .expect("acquire native Metal tracer command buffer");
        frame.draw_path(path.as_ref(), paint.as_ref());
        let pixels = frame.finish().expect("finish native Metal tracer");
        assert_eq!(pixels[(32 * 64 + 32) * 4..][..4], [0, 255, 0, 255]);

        // Root successful forced generic-atomic execution in the final
        // product-shaped Mach-O so size/no-WGPU checks cannot dead-strip it.
        let mut atomic_factory =
            NativeMetalFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic)
                .expect("create forced-atomic native Metal tracer");
        let mut atomic_path = RawPath::new();
        atomic_path.move_to(4.0, 4.0);
        atomic_path.line_to(60.0, 4.0);
        atomic_path.line_to(32.0, 60.0);
        atomic_path.close();
        let atomic_path = atomic_factory.make_render_path(atomic_path, FillRule::NonZero);
        let mut atomic_paint = atomic_factory.make_render_paint();
        atomic_paint.color(0xff00_ff00);
        let mut atomic_frame = atomic_factory
            .begin_frame(0)
            .expect("acquire forced-atomic native Metal command buffer");
        atomic_frame.draw_path(atomic_path.as_ref(), atomic_paint.as_ref());
        let atomic_output = atomic_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic native Metal triangle");
        assert_eq!(
            atomic_output.execution_inventory,
            NativeMetalExecutionInventory {
                mode: RenderMode::ClockwiseAtomic,
                color_ramp_pipeline: false,
                gradient_texture: false,
                atomic_color_plane: false,
                atomic_clip_plane: true,
                atomic_coverage_plane: true,
                render_pass_initialize_pipeline: true,
                midpoint_fan_pipeline: true,
                render_pass_resolve_pipeline: true,
                atomic_draws: 1,
                atomic_draw_groups: 1,
                atomic_barriers: 2,
                atomic_memory_barriers: 0,
                atomic_render_pass_breaks: 0,
            }
        );
        assert_eq!(
            atomic_output.pixels[(24 * 64 + 32) * 4..][..4],
            [0, 255, 0, 255]
        );

        // Root the retained color-ramp resource and generated fragment texture
        // binding on the same forced-atomic product path.
        let mut gradient_path = RawPath::new();
        gradient_path.move_to(8.0, 32.0);
        gradient_path.cubic_to(8.0, 8.0, 56.0, 8.0, 56.0, 32.0);
        gradient_path.line_to(56.0, 40.0);
        gradient_path.cubic_to(56.0, 56.0, 8.0, 56.0, 8.0, 32.0);
        gradient_path.close();
        let gradient_path = atomic_factory.make_render_path(gradient_path, FillRule::NonZero);
        let gradient = atomic_factory.make_linear_gradient(
            8.0,
            32.0,
            56.0,
            32.0,
            &[0xffff_0000, 0xff00_00ff],
            &[0.0, 1.0],
        );
        let mut gradient_paint = atomic_factory.make_render_paint();
        gradient_paint.shader(Some(gradient.as_ref()));
        let mut gradient_frame = atomic_factory
            .begin_frame(0xff00_0000)
            .expect("acquire forced-atomic native Metal gradient command buffer");
        gradient_frame.draw_path(gradient_path.as_ref(), gradient_paint.as_ref());
        let gradient_output = gradient_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic native Metal gradient");
        assert_eq!(
            gradient_output.execution_inventory,
            NativeMetalExecutionInventory {
                mode: RenderMode::ClockwiseAtomic,
                color_ramp_pipeline: true,
                gradient_texture: true,
                atomic_color_plane: false,
                atomic_clip_plane: true,
                atomic_coverage_plane: true,
                render_pass_initialize_pipeline: true,
                midpoint_fan_pipeline: true,
                render_pass_resolve_pipeline: true,
                atomic_draws: 1,
                atomic_draw_groups: 1,
                atomic_barriers: 2,
                atomic_memory_barriers: 0,
                atomic_render_pass_breaks: 0,
            }
        );

        // Root a real overlapping multi-draw flush and its canonical group
        // transition; this prevents product dead stripping from retaining only
        // the one-path generic-atomic specialization.
        let mut overlap_path = RawPath::new();
        overlap_path.move_to(12.0, 12.0);
        overlap_path.line_to(52.0, 12.0);
        overlap_path.line_to(32.0, 52.0);
        overlap_path.close();
        let overlap_path = atomic_factory.make_render_path(overlap_path, FillRule::NonZero);
        let mut overlap_paint = atomic_factory.make_render_paint();
        overlap_paint.color(0xffff_0000);
        let mut multi_frame = atomic_factory
            .begin_frame(0xffff_ffff)
            .expect("acquire forced-atomic native Metal multi-draw command buffer");
        multi_frame.draw_path(atomic_path.as_ref(), atomic_paint.as_ref());
        multi_frame.draw_path(overlap_path.as_ref(), overlap_paint.as_ref());
        let multi_output = multi_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic native Metal multi-draw flush");
        assert_eq!(multi_output.execution_inventory.atomic_draws, 2);
        assert_eq!(multi_output.execution_inventory.atomic_draw_groups, 2);
        assert_eq!(multi_output.execution_inventory.atomic_barriers, 3);
        assert!(!multi_output.execution_inventory.atomic_color_plane);
        assert!(multi_output.execution_inventory.atomic_clip_plane);
        assert!(multi_output.execution_inventory.atomic_coverage_plane);

        factory
            .resize(32, 32)
            .expect("resize native Metal product target");
        let layer = CAMetalLayer::new();
        let device = factory.retained_metal_device();
        layer.setDevice(Some(&device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setFramebufferOnly(true);
        layer.setDrawableSize(CGSize::new(32.0, 32.0));
        layer.setMaximumDrawableCount(2);
        layer.setAllowsNextDrawableTimeout(true);

        let mut product_path = RawPath::new();
        product_path.move_to(4.0, 4.0);
        product_path.line_to(28.0, 4.0);
        product_path.line_to(28.0, 28.0);
        product_path.line_to(4.0, 28.0);
        product_path.close();
        let product_path = factory.make_render_path(product_path, FillRule::NonZero);

        assert_eq!(
            require_caller_drawable(None::<()>),
            Err("caller-owned CAMetalLayer returned no drawable")
        );
        let drawable = require_caller_drawable(layer.nextDrawable())
            .expect("the product-surface root requires a drawable");
        let mut product_frame = factory
            .begin_drawable_frame(&drawable, 0)
            .expect("explicitly select native Metal for the Apple drawable");
        product_frame.draw_path(product_path.as_ref(), paint.as_ref());
        product_frame
            .finish()
            .expect("finish and present native Metal product frame");
        drop(drawable);
        drop(
            layer
                .nextDrawable()
                .expect("completed product frame must leave the layer reusable"),
        );
    });
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn main() {
    panic!("native Metal product root requires an Apple target");
}
