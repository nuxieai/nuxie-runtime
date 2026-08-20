#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_render_api::{BlendMode, Factory, FillRule, RawPath, Renderer};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_renderer::{
    NativeMetalContextOptions, NativeMetalExecutionInventory, NativeMetalFactory, RenderMode,
    ShaderCompilationMode,
};
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
        let render_canvas = factory
            .make_metal_render_canvas(8, 8)
            .expect("create same-texture native Metal render canvas");
        assert_eq!((render_canvas.width(), render_canvas.height()), (8, 8));
        assert!(render_canvas.render_target_and_image_share_texture());
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
        let mut atomic_factory = NativeMetalFactory::new_with_mode_and_context_options(
            64,
            64,
            RenderMode::ClockwiseAtomic,
            NativeMetalContextOptions {
                shader_compilation_mode: ShaderCompilationMode::OnlyUbershaders,
                disable_framebuffer_reads: false,
            },
        )
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
                advanced_blend_pipeline: false,
                hsl_blend_pipeline: false,
                fixed_function_color_output: true,
                atomic_clip_plane: true,
                atomic_coverage_plane: true,
                render_pass_initialize_pipeline: true,
                midpoint_fan_pipeline: true,
                render_pass_resolve_pipeline: true,
                clipped_path_pipeline_set: false,
                clip_rect_pipeline: false,
                outer_curve_pipeline: false,
                interior_triangulation_pipeline: false,
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
                advanced_blend_pipeline: false,
                hsl_blend_pipeline: false,
                fixed_function_color_output: true,
                atomic_clip_plane: true,
                atomic_coverage_plane: true,
                render_pass_initialize_pipeline: true,
                midpoint_fan_pipeline: true,
                render_pass_resolve_pipeline: true,
                clipped_path_pipeline_set: false,
                clip_rect_pipeline: false,
                outer_curve_pipeline: false,
                interior_triangulation_pipeline: false,
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

        // Root one fixed-function flush that mixes solid and gradient paints.
        // The tall gradient path deliberately crosses the canonical interior
        // threshold so the unclipped outer/interior specialization cannot be
        // dead-stripped from the Apple product artifact.
        let mut mixed_background = RawPath::new();
        mixed_background.move_to(0.0, 0.0);
        mixed_background.line_to(64.0, 0.0);
        mixed_background.line_to(64.0, 64.0);
        mixed_background.line_to(0.0, 64.0);
        mixed_background.close();
        let mixed_background = atomic_factory.make_render_path(mixed_background, FillRule::NonZero);
        let mut mixed_background_paint = atomic_factory.make_render_paint();
        mixed_background_paint.color(0xff31_3131);
        let mut mixed_gradient_path = RawPath::new();
        mixed_gradient_path.move_to(0.0, 16.0);
        mixed_gradient_path.line_to(64.0, 16.0);
        mixed_gradient_path.line_to(64.0, 5_016.0);
        mixed_gradient_path.line_to(0.0, 5_016.0);
        mixed_gradient_path.close();
        let mixed_gradient_path =
            atomic_factory.make_render_path(mixed_gradient_path, FillRule::NonZero);
        let mixed_gradient = atomic_factory.make_linear_gradient(
            32.0,
            16.0,
            32.0,
            64.0,
            &[0xffff_ffff, 0xff00_0000],
            &[0.0, 1.0],
        );
        let mut mixed_gradient_paint = atomic_factory.make_render_paint();
        mixed_gradient_paint.shader(Some(mixed_gradient.as_ref()));
        let mut mixed_foreground = RawPath::new();
        mixed_foreground.move_to(0.0, 0.0);
        mixed_foreground.line_to(64.0, 0.0);
        mixed_foreground.line_to(64.0, 16.0);
        mixed_foreground.line_to(0.0, 16.0);
        mixed_foreground.close();
        let mixed_foreground = atomic_factory.make_render_path(mixed_foreground, FillRule::NonZero);
        let mut mixed_foreground_paint = atomic_factory.make_render_paint();
        mixed_foreground_paint.color(0xffff_0000);
        let mut mixed_frame = atomic_factory
            .begin_frame(0)
            .expect("acquire forced-atomic native Metal mixed-gradient command buffer");
        mixed_frame.draw_path(mixed_background.as_ref(), mixed_background_paint.as_ref());
        mixed_frame.draw_path(mixed_gradient_path.as_ref(), mixed_gradient_paint.as_ref());
        mixed_frame.draw_path(mixed_foreground.as_ref(), mixed_foreground_paint.as_ref());
        let mixed_output = mixed_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic native Metal mixed-gradient flush");
        assert_eq!(mixed_output.execution_inventory.atomic_draws, 3);
        assert_eq!(mixed_output.execution_inventory.atomic_draw_groups, 4);
        assert_eq!(mixed_output.execution_inventory.atomic_barriers, 5);
        assert!(mixed_output.execution_inventory.color_ramp_pipeline);
        assert!(!mixed_output.execution_inventory.clipped_path_pipeline_set);
        assert!(mixed_output.execution_inventory.outer_curve_pipeline);
        assert!(
            mixed_output
                .execution_inventory
                .interior_triangulation_pipeline
        );
        assert!(!mixed_output.execution_inventory.atomic_color_plane);

        // Root one same-flush multi-gradient frame with repeated simple and
        // complex ramp content. Geometry differs per occurrence, while the
        // retained 512x2 ramp texture contains one shared row of each kind.
        let complex_forward = atomic_factory.make_linear_gradient(
            0.0,
            0.0,
            64.0,
            64.0,
            &[0xffff_0000, 0xff00_ff00, 0xff00_00ff],
            &[0.0, 0.5, 1.0],
        );
        let complex_reverse = atomic_factory.make_linear_gradient(
            64.0,
            0.0,
            0.0,
            64.0,
            &[0xffff_0000, 0xff00_ff00, 0xff00_00ff],
            &[0.0, 0.5, 1.0],
        );
        let simple_reverse = atomic_factory.make_linear_gradient(
            64.0,
            0.0,
            0.0,
            64.0,
            &[0xff00_0000, 0xffff_ffff],
            &[0.0, 1.0],
        );
        let simple_forward = atomic_factory.make_linear_gradient(
            0.0,
            0.0,
            64.0,
            64.0,
            &[0xff00_0000, 0xffff_ffff],
            &[0.0, 1.0],
        );
        let gradients = [
            complex_forward,
            complex_reverse,
            simple_reverse,
            simple_forward,
        ];
        let mut gradient_paints = Vec::new();
        for gradient in &gradients {
            let mut paint = atomic_factory.make_render_paint();
            paint.shader(Some(gradient.as_ref()));
            gradient_paints.push(paint);
        }
        let mut multi_gradient_frame = atomic_factory
            .begin_frame(0xffff_ffff)
            .expect("acquire forced-atomic multi-gradient command buffer");
        for paint in &gradient_paints {
            multi_gradient_frame.draw_path(mixed_background.as_ref(), paint.as_ref());
        }
        let multi_gradient_output = multi_gradient_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic multi-gradient flush");
        assert!(
            multi_gradient_output
                .execution_inventory
                .color_ramp_pipeline
        );
        assert!(multi_gradient_output.execution_inventory.gradient_texture);
        assert!(
            multi_gradient_output
                .execution_inventory
                .fixed_function_color_output
        );
        assert!(!multi_gradient_output.execution_inventory.atomic_color_plane);
        assert_eq!(multi_gradient_output.execution_inventory.atomic_draws, 4);
        assert_eq!(
            multi_gradient_output.execution_inventory.atomic_draw_groups,
            4
        );
        assert_eq!(multi_gradient_output.execution_inventory.atomic_barriers, 5);

        // Root the non-fixed-function generic-atomic branch with both an
        // advanced RGB blend and HSL blends, including translucent paint.
        let advanced_specs = [
            (0xffff_ffff, BlendMode::SrcOver),
            (0xff10_f040, BlendMode::Exclusion),
            (0x70ee_905a, BlendMode::Saturation),
            (0xb090_5aee, BlendMode::Luminosity),
        ];
        let mut advanced_paints = Vec::new();
        for (color, blend_mode) in advanced_specs {
            let mut paint = atomic_factory.make_render_paint();
            paint.color(color);
            paint.blend_mode(blend_mode);
            advanced_paints.push(paint);
        }
        let mut advanced_frame = atomic_factory
            .begin_frame(0x2020_2020)
            .expect("acquire forced-atomic advanced/HSL command buffer");
        for paint in &advanced_paints {
            advanced_frame.draw_path(mixed_background.as_ref(), paint.as_ref());
        }
        let advanced_output = advanced_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic advanced/HSL flush");
        assert!(advanced_output.execution_inventory.atomic_color_plane);
        assert!(advanced_output.execution_inventory.advanced_blend_pipeline);
        assert!(advanced_output.execution_inventory.hsl_blend_pipeline);
        assert!(
            !advanced_output
                .execution_inventory
                .fixed_function_color_output
        );
        assert_eq!(advanced_output.execution_inventory.atomic_draws, 4);
        assert_eq!(advanced_output.execution_inventory.atomic_draw_groups, 4);
        assert_eq!(advanced_output.execution_inventory.atomic_barriers, 5);
        assert!(!advanced_output.execution_inventory.outer_curve_pipeline);
        assert!(
            !advanced_output
                .execution_inventory
                .interior_triangulation_pipeline
        );

        // Root a clip rectangle introduced after content has begun. This is
        // the platform-specific ENABLE_CLIP_RECT path: draw state is snapped
        // per occurrence, the second group receives a bounded scissor, and
        // resolve restores the full target.
        let mut clip_background = RawPath::new();
        clip_background.move_to(4.0, 4.0);
        clip_background.line_to(60.0, 4.0);
        clip_background.line_to(60.0, 60.0);
        clip_background.line_to(4.0, 60.0);
        clip_background.close();
        let clip_background = atomic_factory.make_render_path(clip_background, FillRule::NonZero);
        let mut clip_foreground = RawPath::new();
        clip_foreground.move_to(16.0, 16.0);
        clip_foreground.line_to(48.0, 16.0);
        clip_foreground.line_to(48.0, 48.0);
        clip_foreground.line_to(16.0, 48.0);
        clip_foreground.close();
        let clip_foreground = atomic_factory.make_render_path(clip_foreground, FillRule::NonZero);
        let mut clip_rect = RawPath::new();
        clip_rect.move_to(24.0, 24.0);
        clip_rect.line_to(40.0, 24.0);
        clip_rect.line_to(40.0, 40.0);
        clip_rect.line_to(24.0, 40.0);
        clip_rect.close();
        let clip_rect = atomic_factory.make_render_path(clip_rect, FillRule::NonZero);
        let mut magenta = atomic_factory.make_render_paint();
        magenta.color(0xffb0_00b0);
        let mut yellow = atomic_factory.make_render_paint();
        yellow.color(0xfff0_b000);
        let mut clip_rect_frame = atomic_factory
            .begin_frame(0xffff_ffff)
            .expect("acquire forced-atomic clip-rect command buffer");
        clip_rect_frame.save();
        clip_rect_frame.draw_path(clip_background.as_ref(), magenta.as_ref());
        clip_rect_frame.clip_path(clip_rect.as_ref());
        clip_rect_frame.draw_path(clip_foreground.as_ref(), yellow.as_ref());
        clip_rect_frame.restore();
        let clip_rect_output = clip_rect_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic native Metal clip rectangle");
        assert!(clip_rect_output.execution_inventory.clip_rect_pipeline);
        assert!(
            clip_rect_output
                .execution_inventory
                .fixed_function_color_output
        );
        assert_eq!(clip_rect_output.execution_inventory.atomic_draws, 2);
        assert_eq!(clip_rect_output.execution_inventory.atomic_draw_groups, 2);
        assert_eq!(clip_rect_output.execution_inventory.atomic_barriers, 3);
        assert_eq!(
            clip_rect_output.pixels[(32 * 64 + 32) * 4..][..4],
            [240, 176, 0, 255]
        );
        assert_eq!(
            clip_rect_output.pixels[(32 * 64 + 23) * 4..][..4],
            [176, 0, 176, 255]
        );

        // Root flush-wide generic-atomic clipping plus both physical geometry
        // families. This is the checked-in nested-clip tracer shape: the
        // outer clip and content require outer curves plus interior triangles,
        // while the nested clip remains a midpoint fan.
        atomic_factory
            .resize(640, 640)
            .expect("resize forced-atomic nested-clip target");
        let mut outer_clip = RawPath::new();
        outer_clip.move_to(40.0, 60.0);
        outer_clip.line_to(600.0, 60.0);
        outer_clip.line_to(600.0, 280.0);
        outer_clip.line_to(380.0, 280.0);
        outer_clip.line_to(380.0, 600.0);
        outer_clip.line_to(40.0, 600.0);
        outer_clip.close();
        outer_clip.move_to(420.0, 420.0);
        outer_clip.line_to(580.0, 420.0);
        outer_clip.line_to(580.0, 580.0);
        outer_clip.line_to(420.0, 580.0);
        outer_clip.close();
        let outer_clip = atomic_factory.make_render_path(outer_clip, FillRule::Clockwise);
        let mut nested_clip = RawPath::new();
        nested_clip.move_to(140.0, 160.0);
        nested_clip.line_to(520.0, 160.0);
        nested_clip.line_to(520.0, 520.0);
        nested_clip.line_to(440.0, 520.0);
        nested_clip.line_to(440.0, 320.0);
        nested_clip.line_to(300.0, 320.0);
        nested_clip.line_to(300.0, 520.0);
        nested_clip.line_to(140.0, 520.0);
        nested_clip.close();
        let nested_clip = atomic_factory.make_render_path(nested_clip, FillRule::Clockwise);
        let mut clipped_content = RawPath::new();
        clipped_content.move_to(0.0, 0.0);
        clipped_content.line_to(640.0, 0.0);
        clipped_content.line_to(640.0, 640.0);
        clipped_content.line_to(0.0, 640.0);
        clipped_content.close();
        let clipped_content = atomic_factory.make_render_path(clipped_content, FillRule::Clockwise);
        let mut clipped_paint = atomic_factory.make_render_paint();
        clipped_paint.color(0xffff_ffff);
        let mut clipped_frame = atomic_factory
            .begin_frame(0)
            .expect("acquire forced-atomic nested-clip command buffer");
        clipped_frame.clip_path(outer_clip.as_ref());
        clipped_frame.clip_path(nested_clip.as_ref());
        clipped_frame.draw_path(clipped_content.as_ref(), clipped_paint.as_ref());
        let clipped_output = clipped_frame
            .finish_for_benchmark()
            .expect("finish forced-atomic native Metal nested clip");
        assert_eq!(clipped_output.execution_inventory.atomic_draws, 3);
        assert_eq!(clipped_output.execution_inventory.atomic_draw_groups, 5);
        assert_eq!(clipped_output.execution_inventory.atomic_barriers, 6);
        assert!(clipped_output.execution_inventory.clipped_path_pipeline_set);
        assert!(clipped_output.execution_inventory.outer_curve_pipeline);
        assert!(
            clipped_output
                .execution_inventory
                .interior_triangulation_pipeline
        );
        assert_eq!(clipped_output.pixels[..4], [0, 0, 0, 0]);
        assert_eq!(
            clipped_output.pixels[(512 * 640 + 512) * 4..][..4],
            [255, 255, 255, 255]
        );

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
