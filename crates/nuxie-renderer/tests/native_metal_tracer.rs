#![cfg(all(
    feature = "native-metal-experimental",
    any(target_os = "ios", target_os = "macos")
))]

use nuxie_render_api::Factory;
use nuxie_render_stream::RenderStream;
#[cfg(feature = "rust-wgpu")]
use nuxie_renderer::WgpuFactory;
use nuxie_renderer::{
    NativeMetalContextOptions, NativeMetalExecutionInventory, NativeMetalFactory, RenderMode,
    RendererError, ShaderCompilationMode,
};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

fn read_png(path: impl AsRef<std::path::Path>) -> Vec<u8> {
    let reference = File::open(path).expect("open renderer reference");
    let decoder = png::Decoder::new(BufReader::new(reference));
    let mut reader = decoder.read_info().expect("read renderer reference info");
    let mut pixels = vec![0; reader.output_buffer_size().expect("reference buffer size")];
    let info = reader
        .next_frame(&mut pixels)
        .expect("decode renderer reference");
    pixels.truncate(info.buffer_size());
    pixels
}

fn assert_rgba8_with_tolerance(
    actual: &[u8],
    expected: &[u8],
    maximum_allowed_delta: u8,
    maximum_different_pixels: Option<usize>,
    require_exact_occupancy: bool,
    label: &str,
) {
    assert_eq!(actual.len(), expected.len(), "{label}: byte length");
    let mut maximum_delta = 0u8;
    let mut first_excess = None;
    let mut first_occupancy_mismatch = None;
    let mut different_pixels = 0usize;
    for (pixel_index, (actual, expected)) in actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .enumerate()
    {
        different_pixels += usize::from(actual != expected);
        let actual_occupied = actual[..3].iter().any(|channel| *channel != 0);
        let expected_occupied = expected[..3].iter().any(|channel| *channel != 0);
        if actual_occupied != expected_occupied && first_occupancy_mismatch.is_none() {
            first_occupancy_mismatch = Some((pixel_index, actual.to_vec(), expected.to_vec()));
        }
        for channel in 0..4 {
            let delta = actual[channel].abs_diff(expected[channel]);
            maximum_delta = maximum_delta.max(delta);
            if delta > maximum_allowed_delta && first_excess.is_none() {
                first_excess = Some((pixel_index, channel, actual[channel], expected[channel]));
            }
        }
    }
    if require_exact_occupancy {
        assert_eq!(
            first_occupancy_mismatch, None,
            "{label}: geometry/coverage occupancy mismatch"
        );
    }
    if let Some(maximum_different_pixels) = maximum_different_pixels {
        assert!(
            different_pixels <= maximum_different_pixels,
            "{label}: {different_pixels} pixels differ; maximum is {maximum_different_pixels}"
        );
    }
    assert_eq!(
        first_excess, None,
        "{label}: exceeded {maximum_allowed_delta}-LSB RGBA8 tolerance; maximum delta={maximum_delta}"
    );
}

fn assert_clear_color_occupancy(actual: &[u8], expected: &[u8], clear: [u8; 4], label: &str) {
    let mismatch = actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .enumerate()
        .find(|(_, (actual, expected))| (*actual != clear) != (*expected != clear));
    assert_eq!(mismatch, None, "{label}: differs-from-clear occupancy");
}

fn assert_actual_atomic_inventory(
    inventory: &NativeMetalExecutionInventory,
    expected_groups: usize,
    expected_barriers: usize,
) {
    assert_eq!(inventory.mode, RenderMode::ClockwiseAtomic);
    assert!(inventory.atomic_draws > 0);
    assert!(inventory.atomic_draw_instances >= inventory.atomic_draws);
    assert_eq!(inventory.atomic_draw_groups, expected_groups);
    assert_eq!(inventory.atomic_barriers, expected_barriers);
    assert_eq!(
        inventory.atomic_barriers,
        inventory.atomic_memory_barriers
            + inventory.atomic_render_pass_breaks
            + inventory.atomic_raster_order_group_barriers
    );
}

#[test]
fn native_metal_frame_clears_and_reads_back() {
    let factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    let pixels = factory
        .begin_frame(0x1122_3344)
        .expect("acquire one command buffer for native Metal frame")
        .finish()
        .expect("finish native Metal frame");

    assert_eq!(
        pixels,
        [
            0x02, 0x03, 0x05, 0x11, 0x02, 0x03, 0x05, 0x11, 0x02, 0x03, 0x05, 0x11, 0x02, 0x03,
            0x05, 0x11,
        ]
    );
}

#[test]
fn native_metal_factory_decodes_an_image_through_the_context_texture_owner() {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .expect("write native Metal image fixture header")
            .write_image_data(&[
                0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff,
            ])
            .expect("write native Metal image fixture pixels");
    }

    let mut factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    let decoded = factory.decode_image(&encoded);
    #[cfg(feature = "rive-decoders")]
    {
        let image = decoded.expect("decode through the native Metal context texture owner");
        assert_eq!((image.width(), image.height()), (2, 2));
    }
    #[cfg(not(feature = "rive-decoders"))]
    {
        assert!(
            decoded.is_err(),
            "without the pinned RIVE_DECODERS branch, the default Metal platform hook must remain null"
        );
    }
}

#[test]
fn native_metal_factory_adopts_a_caller_texture_through_the_context_owner() {
    use objc2_metal::{
        MTLDevice, MTLPixelFormat, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage,
    };

    let factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    let descriptor = MTLTextureDescriptor::new();
    descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
    // SAFETY: this public-seam fixture uses nonzero NSUInteger-representable
    // dimensions and one mip level, matching the adopted image contract.
    unsafe {
        descriptor.setWidth(2);
        descriptor.setHeight(2);
        descriptor.setMipmapLevelCount(1);
    }
    descriptor.setUsage(MTLTextureUsage::ShaderRead);
    descriptor.setTextureType(MTLTextureType::Type2D);
    let texture = factory
        .retained_metal_device()
        .newTextureWithDescriptor(&descriptor)
        .expect("allocate caller-owned Metal texture");
    assert!(
        factory
            .adopt_metal_image_texture(texture.clone(), 0, 2)
            .is_none(),
        "zero-sized adopted images fail closed"
    );

    let image = factory
        .adopt_metal_image_texture(texture, 2, 2)
        .expect("adopt caller-owned texture through the native Metal context");
    assert_eq!((image.width(), image.height()), (2, 2));
}

#[cfg(feature = "native-ore-metal-experimental")]
#[test]
fn native_metal_factory_makes_a_same_texture_render_canvas_owner() {
    use objc2_metal::{MTLPixelFormat, MTLResource, MTLStorageMode, MTLTexture, MTLTextureUsage};

    let factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    assert!(factory.make_metal_render_canvas(0, 5).is_err());
    let canvas = factory
        .make_metal_render_canvas(3, 5)
        .expect("create same-texture native Metal render canvas");

    assert_eq!((canvas.width(), canvas.height()), (3, 5));
    assert!(canvas.render_target_and_image_share_texture());
    let texture = canvas
        .retained_metal_texture()
        .expect("successful live allocation retains its Metal texture");
    assert_eq!(texture.pixelFormat(), MTLPixelFormat::RGBA8Unorm);
    assert_eq!(texture.storageMode(), MTLStorageMode::Private);
    assert!(texture.usage().contains(MTLTextureUsage::RenderTarget));
    assert!(texture.usage().contains(MTLTextureUsage::ShaderRead));
}

#[cfg(feature = "native-ore-metal-experimental")]
#[test]
fn native_metal_factory_makes_ore_context_from_its_retained_service() {
    use nuxie_ore_metal::context::{FrameDescriptor, ShaderTarget};

    let factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    let first = factory
        .with_ore_context(|ore| core::ptr::from_mut(ore) as usize)
        .expect("mechanical source ORE context is available");
    let second = factory
        .with_ore_context(|ore| core::ptr::from_mut(ore) as usize)
        .expect("cached mechanical source ORE context remains available");
    assert_eq!(
        first, second,
        "RenderContext::ore() returns its cached singleton"
    );
    let completion = factory
        .with_ore_context(|ore| {
            assert_eq!(ore.shaderTarget(), ShaderTarget::msl);
            ore.beginFrame(&FrameDescriptor {
                externalCommandBuffer: None,
                safeFrameNumber: 0,
                currentFrameNumber: 0,
            });
            ore.end_frame_with_completion()
                .expect("submit the retained renderer queue's command buffer")
        })
        .expect("scoped cached ORE access succeeds");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(result) = completion.result() {
            result.expect("empty ORE frame completes successfully");
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "retained renderer queue did not complete the ORE frame"
        );
        std::thread::yield_now();
    }
}

#[test]
fn native_metal_forced_atomic_culls_offscreen_draws_before_flush_shape_validation() {
    fn render(stream_text: &str) -> Vec<u8> {
        let stream = RenderStream::parse(stream_text).expect("parse offscreen atomic stream");
        let (width, height) = stream.frame_size.expect("offscreen atomic frame size");
        let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
            width,
            height,
            RenderMode::ClockwiseAtomic,
            NativeMetalContextOptions {
                shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
                ..NativeMetalContextOptions::default()
            },
        )
        .expect("force synchronous native Metal generic-atomic mode");
        let mut frame = factory
            .begin_frame(stream.clear_color.unwrap_or(0))
            .expect("acquire offscreen atomic frame");
        stream
            .replay_frame(0, &mut factory, &mut frame)
            .expect("replay offscreen atomic frame");
        frame.finish().expect("finish offscreen atomic frame")
    }

    let empty = render(
        "rive-golden-stream-v1\nframeSize width=8 height=8\nclearColor value=0xff123456\nframe\n",
    );
    let all_offscreen = render(
        "rive-golden-stream-v1\nframeSize width=8 height=8\nclearColor value=0xff123456\ndrawPath path={id=1,fillRule=0,path={verbs=[move,line,line,close],points=[(100,100),(120,100),(110,120)]}} paint={id=1,style=fill,color=0xffff0000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\nframe\n",
    );
    assert_eq!(
        all_offscreen, empty,
        "a fully culled flush remains clear-only"
    );
    let empty_path = render(
        "rive-golden-stream-v1\nframeSize width=8 height=8\nclearColor value=0xff123456\ndrawPath path={id=1,fillRule=0,path={verbs=[],points=[]}} paint={id=1,style=fill,color=0xffff0000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\nframe\n",
    );
    assert_eq!(empty_path, empty, "an empty path remains clear-only");

    let visible_only = render(
        "rive-golden-stream-v1\nframeSize width=8 height=8\nclearColor value=0xff123456\ndrawPath path={id=2,fillRule=0,path={verbs=[move,line,line,close],points=[(1,1),(7,1),(4,7)]}} paint={id=2,style=fill,color=0xff00ff00,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\nframe\n",
    );
    let offscreen_gradient_then_visible = render(
        "rive-golden-stream-v1\nframeSize width=8 height=8\nclearColor value=0xff123456\nmakeLinearGradient id=1 start=(100,100) end=(120,100) stops=[{color=0xffff0000,stop=0},{color=0xff0000ff,stop=1}]\ndrawPath path={id=1,fillRule=0,path={verbs=[move,cubic,line,cubic,close],points=[(100,110),(100,100),(120,100),(120,110),(120,115),(120,120),(100,120),(100,110)]}} paint={id=1,style=fill,color=0xffffffff,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=1}\ndrawPath path={id=2,fillRule=0,path={verbs=[move,line,line,close],points=[(1,1),(7,1),(4,7)]}} paint={id=2,style=fill,color=0xff00ff00,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\nframe\n",
    );
    assert_eq!(
        offscreen_gradient_then_visible, visible_only,
        "culled gradients do not contaminate visible flush-shape validation"
    );
}

#[test]
fn native_metal_solid_rectangle_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/first-light-rectangle.rive-stream"))
            .expect("read rectangle stream"),
    )
    .expect("parse rectangle stream");
    let (width, height) = stream.frame_size.expect("rectangle frame size");
    let mut factory = NativeMetalFactory::new(width, height).expect("create native Metal renderer");
    let mut frame = factory
        .begin_frame(stream.clear_color.unwrap_or(0))
        .expect("acquire one command buffer for native Metal frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay rectangle through Factory/Renderer seam");
    let actual = frame.finish().expect("finish native Metal frame");

    let reference = File::open(fixture_root.join("reference/metal/first-light-rectangle.png"))
        .expect("open pinned C++ Metal oracle");
    let decoder = png::Decoder::new(BufReader::new(reference));
    let mut reader = decoder.read_info().expect("read oracle info");
    let mut expected = vec![0; reader.output_buffer_size().expect("oracle buffer size")];
    let info = reader.next_frame(&mut expected).expect("decode oracle");
    expected.truncate(info.buffer_size());

    assert_eq!(
        actual, expected,
        "native Rust Metal versus pinned C++ Metal"
    );
    #[cfg(feature = "rust-wgpu")]
    {
        let mut wgpu_factory = WgpuFactory::new(width, height).expect("create Rust-wgpu oracle");
        let mut wgpu_frame = wgpu_factory.begin_frame(stream.clear_color.unwrap_or(0));
        stream
            .replay_frame(0, &mut wgpu_factory, &mut wgpu_frame)
            .expect("replay rectangle through Rust-wgpu oracle");
        let wgpu_pixels = wgpu_frame.finish().expect("finish Rust-wgpu oracle");
        assert_eq!(actual, wgpu_pixels, "native Rust Metal versus Rust-wgpu");
    }
}

#[test]
fn native_metal_forced_atomic_triangle_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/first-light-triangle.rive-stream"))
            .expect("read triangle stream"),
    )
    .expect("parse triangle stream");
    let (width, height) = stream.frame_size.expect("triangle frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    assert_eq!(factory.render_mode(), RenderMode::ClockwiseAtomic);

    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay triangle through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal triangle");
    assert_actual_atomic_inventory(&output.execution_inventory, 1, 2);
    assert!(!output.execution_inventory.color_ramp_pipeline);
    assert!(!output.execution_inventory.gradient_texture);
    assert!(!output.execution_inventory.atomic_color_plane);
    assert!(!output.execution_inventory.advanced_blend_pipeline);
    assert!(!output.execution_inventory.hsl_blend_pipeline);
    assert!(output.execution_inventory.fixed_function_color_output);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    assert!(output.execution_inventory.render_pass_initialize_pipeline);
    assert!(output.execution_inventory.midpoint_fan_pipeline);
    assert!(output.execution_inventory.render_pass_resolve_pipeline);
    let cpp_metal =
        read_png(fixture_root.join("reference/metal/first-light-triangle-clockwise-atomic.png"));
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        2,
        Some(32),
        true,
        "forced generic-atomic Rust Metal triangle versus pinned C++ Metal",
    );
}

#[test]
fn native_metal_atomic_buffers_are_lazily_allocated_then_bound_by_the_live_flush() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/first-light-triangle.rive-stream"))
            .expect("read triangle stream"),
    )
    .expect("parse triangle stream");
    let (width, height) = stream.frame_size.expect("triangle frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");

    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire first forced-atomic frame before any atomic buffer can be warm");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay triangle through the public Factory/Renderer seam");
    let drawn = frame
        .finish_for_benchmark()
        .expect("finish drawn forced-atomic frame");
    assert!(drawn.execution_inventory.atomic_clip_plane);
    assert!(drawn.execution_inventory.atomic_coverage_plane);
}

#[test]
fn native_metal_forced_atomic_gradient_cubic_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(
            fixture_root.join("streams/first-light-gradient-cubic.rive-stream"),
        )
        .expect("read gradient cubic stream"),
    )
    .expect("parse gradient cubic stream");
    let (width, height) = stream.frame_size.expect("gradient cubic frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");

    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal gradient frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay gradient cubic through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal gradient cubic");
    assert_actual_atomic_inventory(&output.execution_inventory, 1, 2);
    assert!(output.execution_inventory.color_ramp_pipeline);
    assert!(output.execution_inventory.gradient_texture);
    assert!(!output.execution_inventory.atomic_color_plane);
    assert!(!output.execution_inventory.advanced_blend_pipeline);
    assert!(!output.execution_inventory.hsl_blend_pipeline);
    assert!(output.execution_inventory.fixed_function_color_output);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    assert!(output.execution_inventory.render_pass_initialize_pipeline);
    assert!(output.execution_inventory.midpoint_fan_pipeline);
    assert!(output.execution_inventory.render_pass_resolve_pipeline);
    let cpp_metal = read_png(fixture_root.join("reference/metal/first-light-gradient-cubic.png"));
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        1,
        Some(3),
        true,
        "forced generic-atomic Rust Metal gradient versus pinned C++ Metal under validation",
    );
}

#[test]
fn native_metal_forced_atomic_mixed_gradient_flush_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/riv/deterministic_mode.rive-stream"))
            .expect("read deterministic-mode stream"),
    )
    .expect("parse deterministic-mode stream");
    let (width, height) = stream.frame_size.expect("deterministic-mode frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");

    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal mixed-gradient frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay mixed gradient through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal mixed-gradient flush");
    assert_actual_atomic_inventory(&output.execution_inventory, 4, 5);
    assert!(output.execution_inventory.color_ramp_pipeline);
    assert!(output.execution_inventory.gradient_texture);
    assert!(!output.execution_inventory.atomic_color_plane);
    assert!(!output.execution_inventory.advanced_blend_pipeline);
    assert!(!output.execution_inventory.hsl_blend_pipeline);
    assert!(output.execution_inventory.fixed_function_color_output);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    assert!(output.execution_inventory.render_pass_initialize_pipeline);
    assert!(output.execution_inventory.midpoint_fan_pipeline);
    assert!(output.execution_inventory.render_pass_resolve_pipeline);
    assert!(output.execution_inventory.outer_curve_pipeline);
    assert!(output.execution_inventory.interior_triangulation_pipeline);
    let cpp_metal = read_png(
        fixture_root.join("reference/metal/riv/deterministic_mode-frame-0-clockwise-atomic.png"),
    );
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        2,
        Some(32),
        true,
        "forced generic-atomic Rust Metal mixed gradient versus pinned C++ Metal",
    );
}

#[test]
fn native_metal_forced_atomic_rect_grad_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/gm/rect_grad.rive-stream"))
            .expect("read rect-grad stream"),
    )
    .expect("parse rect-grad stream");
    let (width, height) = stream.frame_size.expect("rect-grad frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal rect-grad frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay rect-grad through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal rect-grad");
    assert_actual_atomic_inventory(&output.execution_inventory, 4, 5);
    assert!(output.execution_inventory.color_ramp_pipeline);
    assert!(output.execution_inventory.gradient_texture);
    assert!(!output.execution_inventory.atomic_color_plane);
    assert!(!output.execution_inventory.advanced_blend_pipeline);
    assert!(!output.execution_inventory.hsl_blend_pipeline);
    assert!(output.execution_inventory.fixed_function_color_output);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    assert!(output.execution_inventory.render_pass_initialize_pipeline);
    assert!(output.execution_inventory.midpoint_fan_pipeline);
    assert!(output.execution_inventory.render_pass_resolve_pipeline);
    let cpp_metal =
        read_png(fixture_root.join("reference/metal/gm/rect_grad-clockwise-atomic.png"));
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        2,
        Some(32),
        false,
        "forced generic-atomic Rust Metal rect-grad versus pinned C++ Metal",
    );
    assert_clear_color_occupancy(
        &output.pixels,
        &cpp_metal,
        [255, 255, 255, 255],
        "forced generic-atomic Rust Metal rect-grad versus pinned C++ Metal",
    );
}

#[test]
fn native_metal_forced_atomic_gamma_correction_clip_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/gm/gamma_correction_clip.rive-stream"))
            .expect("read gamma-correction clip stream"),
    )
    .expect("parse gamma-correction clip stream");
    let (width, height) = stream.frame_size.expect("gamma-correction clip frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal gamma-correction clip frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay gamma-correction clip through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal gamma-correction clip");

    let cpp_metal = read_png(
        fixture_root.join("reference/metal/gm/gamma_correction_clip-clockwise-atomic.png"),
    );
    assert_actual_atomic_inventory(&output.execution_inventory, 2, 3);
    assert!(!output.execution_inventory.color_ramp_pipeline);
    assert!(!output.execution_inventory.gradient_texture);
    assert!(!output.execution_inventory.atomic_color_plane);
    assert!(!output.execution_inventory.advanced_blend_pipeline);
    assert!(!output.execution_inventory.hsl_blend_pipeline);
    assert!(output.execution_inventory.fixed_function_color_output);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    assert!(output.execution_inventory.render_pass_initialize_pipeline);
    assert!(output.execution_inventory.midpoint_fan_pipeline);
    assert!(output.execution_inventory.render_pass_resolve_pipeline);
    assert!(output.execution_inventory.clip_rect_pipeline);
    assert_clear_color_occupancy(
        &output.pixels,
        &cpp_metal,
        [255, 255, 255, 255],
        "forced generic-atomic Rust Metal gamma-correction clip versus pinned C++ Metal",
    );
    let pixel = |x: usize, y: usize| &output.pixels[(y * width as usize + x) * 4..][..4];
    let is_magenta = |pixel: &[u8]| pixel[0] == pixel[2] && pixel[1] == 0 && pixel[3] == 255;
    let is_yellow =
        |pixel: &[u8]| pixel[0] > pixel[1] && pixel[1] != 0 && pixel[2] == 0 && pixel[3] == 255;
    let yellow = [240, 176, 0, 255];
    assert!(is_magenta(pixel(239, 250)));
    assert_eq!(pixel(240, 250), yellow);
    assert_eq!(pixel(259, 250), yellow);
    assert!(is_magenta(pixel(260, 250)));
    assert_eq!(
        output
            .pixels
            .chunks_exact(4)
            .filter(|pixel| is_yellow(pixel))
            .count(),
        400,
        "the 20x20 rectangular clip must admit exactly 400 yellow pixels"
    );
}

#[test]
fn native_metal_forced_atomic_supports_non_rectangular_clip_after_content_publicly() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream_text = std::fs::read_to_string(
        fixture_root.join("streams/gm/gamma_correction_clip.rive-stream"),
    )
    .expect("read gamma-correction clip stream")
    .replace(
        "verbs=[move,line,line,line,close],points=[(240,240),(260,240),(260,260),(240,260)]",
        "verbs=[move,line,line,close],points=[(240,240),(260,240),(250,260)]",
    );
    let stream = RenderStream::parse(&stream_text).expect("parse non-rectangular clip stream");
    let (width, height) = stream.frame_size.expect("non-rectangular clip frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force generic-atomic mode");
    let mut frame = factory
        .begin_frame(stream.clear_color.unwrap_or(0))
        .expect("begin non-rectangular clip frame");

    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("renderer records unsupported state for finish");
    let output = frame
        .finish_for_benchmark()
        .expect("source-shaped renderer supports non-rectangular clips after content");
    assert!(output.execution_inventory.clipped_path_pipeline_set);
}

#[test]
fn native_metal_forced_atomic_supports_incompatible_post_content_clip_transactionally() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let baseline_text =
        std::fs::read_to_string(fixture_root.join("streams/gm/gamma_correction_clip.rive-stream"))
            .expect("read gamma-correction clip stream");
    let stream_text = baseline_text.replace(
        "clipPath path={id=2,fillRule=0,path={verbs=[move,line,line,line,close],points=[(240,240),(260,240),(260,260),(240,260)]}}\n",
        "clipPath path={id=2,fillRule=0,path={verbs=[move,line,line,line,close],points=[(240,240),(260,240),(260,260),(240,260)]}}\ntransform matrix=[0.70710677,0.70710677,-0.70710677,0.70710677,250,-103.55338]\nclipPath path={id=3,fillRule=0,path={verbs=[move,line,line,line,close],points=[(240,240),(260,240),(260,260),(240,260)]}}\n",
    );
    assert_ne!(stream_text, baseline_text);
    let stream = RenderStream::parse(&stream_text).expect("parse incompatible clip stream");
    let (width, height) = stream.frame_size.expect("incompatible clip frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force generic-atomic mode");
    let mut frame = factory
        .begin_frame(stream.clear_color.unwrap_or(0))
        .expect("begin incompatible clip frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("renderer records unsupported state for finish");
    let first = frame
        .finish_for_benchmark()
        .expect("source-shaped renderer supports transformed clips after content");
    assert!(first.execution_inventory.clipped_path_pipeline_set);

    let baseline = RenderStream::parse(&baseline_text).expect("parse baseline clip stream");
    let mut retry = factory
        .begin_frame(baseline.clear_color.unwrap_or(0))
        .expect("begin valid clip frame after transactional rejection");
    baseline
        .replay_frame(0, &mut factory, &mut retry)
        .expect("replay valid clip frame after transactional rejection");
    let output = retry
        .finish_for_benchmark()
        .expect("finish valid clip frame after transactional rejection");
    assert!(output.execution_inventory.clip_rect_pipeline);
    assert_eq!(
        output
            .pixels
            .chunks_exact(4)
            .filter(|pixel| pixel[0] > pixel[1] && pixel[1] != 0 && pixel[2] == 0)
            .count(),
        400
    );
}

#[test]
fn native_metal_forced_atomic_overfill_opaque_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/gm/overfill_opaque.rive-stream"))
            .expect("read overfill opaque stream"),
    )
    .expect("parse overfill opaque stream");
    let (width, height) = stream.frame_size.expect("overfill opaque frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal overfill frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay overfill through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal overfill");
    assert_actual_atomic_inventory(&output.execution_inventory, 4, 5);
    assert!(!output.execution_inventory.color_ramp_pipeline);
    assert!(!output.execution_inventory.gradient_texture);
    assert!(!output.execution_inventory.atomic_color_plane);
    assert!(!output.execution_inventory.advanced_blend_pipeline);
    assert!(!output.execution_inventory.hsl_blend_pipeline);
    assert!(output.execution_inventory.fixed_function_color_output);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    assert!(output.execution_inventory.render_pass_initialize_pipeline);
    assert!(output.execution_inventory.midpoint_fan_pipeline);
    assert!(output.execution_inventory.render_pass_resolve_pipeline);
    let cpp_metal =
        read_png(fixture_root.join("reference/metal/gm/overfill_opaque-clockwise-atomic.png"));
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        2,
        Some(48),
        false,
        "forced generic-atomic Rust Metal overfill versus pinned C++ Metal",
    );
    assert_clear_color_occupancy(
        &output.pixels,
        &cpp_metal,
        [0xff; 4],
        "forced generic-atomic Rust Metal overfill versus pinned C++ Metal",
    );
}

#[test]
fn native_metal_forced_atomic_overfill_blendmodes_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/gm/overfill_blendmodes.rive-stream"))
            .expect("read overfill blendmodes stream"),
    )
    .expect("parse overfill blendmodes stream");
    let (width, height) = stream.frame_size.expect("overfill blendmodes frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal blendmodes frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay blendmodes through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal blendmodes");
    assert_actual_atomic_inventory(&output.execution_inventory, 4, 5);
    assert!(!output.execution_inventory.color_ramp_pipeline);
    assert!(!output.execution_inventory.gradient_texture);
    assert!(output.execution_inventory.atomic_color_plane);
    assert!(output.execution_inventory.advanced_blend_pipeline);
    assert!(output.execution_inventory.hsl_blend_pipeline);
    assert!(!output.execution_inventory.fixed_function_color_output);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    assert!(output.execution_inventory.render_pass_initialize_pipeline);
    assert!(output.execution_inventory.midpoint_fan_pipeline);
    assert!(output.execution_inventory.render_pass_resolve_pipeline);
    let cpp_metal =
        read_png(fixture_root.join("reference/metal/gm/overfill_blendmodes-clockwise-atomic.png"));
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        2,
        Some(32),
        false,
        "forced generic-atomic Rust Metal advanced blend versus pinned C++ Metal",
    );
    assert_clear_color_occupancy(
        &output.pixels,
        &cpp_metal,
        [4, 4, 4, 32],
        "forced generic-atomic Rust Metal advanced blend versus pinned C++ Metal",
    );
}

#[test]
fn native_metal_atomic_inventory_reports_current_flush_after_advanced_then_fixed() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let advanced = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/gm/overfill_blendmodes.rive-stream"))
            .expect("read advanced blend stream"),
    )
    .expect("parse advanced blend stream");
    let fixed = RenderStream::parse(
        &std::fs::read_to_string(fixture_root.join("streams/gm/overfill_opaque.rive-stream"))
            .expect("read fixed blend stream"),
    )
    .expect("parse fixed blend stream");
    let (width, height) = advanced.frame_size.expect("advanced frame size");
    assert_eq!(fixed.frame_size, Some((width, height)));
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("create reusable generic-atomic factory");

    let mut advanced_frame = factory
        .begin_frame(advanced.clear_color.unwrap_or(0))
        .expect("begin advanced frame");
    advanced
        .replay_frame(0, &mut factory, &mut advanced_frame)
        .expect("replay advanced frame");
    let advanced_output = advanced_frame
        .finish_for_benchmark()
        .expect("finish advanced frame");
    assert!(advanced_output.execution_inventory.atomic_color_plane);
    assert!(advanced_output.execution_inventory.advanced_blend_pipeline);
    assert!(
        !advanced_output
            .execution_inventory
            .fixed_function_color_output
    );

    let mut fixed_frame = factory
        .begin_frame(fixed.clear_color.unwrap_or(0))
        .expect("begin fixed frame on retained target");
    fixed
        .replay_frame(0, &mut factory, &mut fixed_frame)
        .expect("replay fixed frame on retained target");
    let fixed_output = fixed_frame
        .finish_for_benchmark()
        .expect("finish fixed frame on retained target");
    assert!(fixed_output.execution_inventory.fixed_function_color_output);
    assert!(!fixed_output.execution_inventory.advanced_blend_pipeline);
    assert!(!fixed_output.execution_inventory.hsl_blend_pipeline);
    assert!(
        !fixed_output.execution_inventory.atomic_color_plane,
        "inventory describes this fixed flush, not the color owner retained from the prior frame"
    );
    assert!(fixed_output.execution_inventory.atomic_clip_plane);
    assert!(fixed_output.execution_inventory.atomic_coverage_plane);
}

#[test]
fn native_metal_forced_atomic_supports_advanced_blend_with_gradient_publicly() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream_text =
        std::fs::read_to_string(fixture_root.join("streams/gm/overfill_blendmodes.rive-stream"))
            .expect("read advanced blend stream");
    let stream_text = stream_text
        .replace(
            "clearColor value=0x20202020\n",
            "clearColor value=0x20202020\nmakeLinearGradient id=1 start=(0,0) end=(200,200) stops=[{color=0xffffffff,stop=0},{color=0xff000000,stop=1}]\n",
        )
        .replace("blendMode=23,shader=0}", "blendMode=23,shader=1}");
    let stream = RenderStream::parse(&stream_text).expect("parse advanced-gradient stream");
    let (width, height) = stream.frame_size.expect("advanced-gradient frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force generic-atomic mode");
    let mut frame = factory
        .begin_frame(stream.clear_color.unwrap_or(0))
        .expect("begin advanced-gradient frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay advanced-gradient stream through public seam");
    let output = frame
        .finish_for_benchmark()
        .expect("source-shaped renderer supports advanced-blend gradients");
    assert!(output.execution_inventory.gradient_texture);
    assert!(output.execution_inventory.advanced_blend_pipeline);
    assert!(output.execution_inventory.atomic_color_plane);
}

#[test]
fn native_metal_forced_atomic_nested_clip_matches_pinned_cpp_metal_oracle() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(
            fixture_root.join("streams/first-light-nested-clip-probe.rive-stream"),
        )
        .expect("read nested clip stream"),
    )
    .expect("parse nested clip stream");
    let (width, height) = stream.frame_size.expect("nested clip frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire forced-atomic native Metal nested-clip frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay nested clip through the public Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish forced-atomic native Metal nested clip");

    assert_actual_atomic_inventory(&output.execution_inventory, 5, 6);
    assert!(output.execution_inventory.clipped_path_pipeline_set);
    assert!(output.execution_inventory.outer_curve_pipeline);
    assert!(output.execution_inventory.interior_triangulation_pipeline);
    assert!(!output.execution_inventory.atomic_color_plane);
    assert!(output.execution_inventory.atomic_clip_plane);
    assert!(output.execution_inventory.atomic_coverage_plane);
    let cpp_metal = read_png(
        fixture_root.join("reference/metal/first-light-nested-clip-probe-clockwise-atomic.png"),
    );
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        0,
        Some(0),
        true,
        "forced generic-atomic Rust Metal nested clip versus pinned C++ Metal",
    );
}

#[test]
fn native_metal_forced_atomic_nested_clip_supports_restore_after_content() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream_text = std::fs::read_to_string(
        fixture_root.join("streams/first-light-nested-clip-probe.rive-stream"),
    )
    .expect("read nested clip stream");
    let stream_text = stream_text
        .replace(
            "clearColor value=0x00000000\n",
            "clearColor value=0x00000000\nsave\n",
        )
        .replace("\nframe\n", "\nrestore\nframe\n");
    let stream = RenderStream::parse(&stream_text).expect("parse restore-after-content stream");
    let (width, height) = stream.frame_size.expect("nested clip frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    let mut frame = factory
        .begin_frame(stream.clear_color.unwrap_or(0))
        .expect("acquire forced-atomic nested-clip frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay restore-after-content stream through the public seam");

    let output = frame
        .finish_for_benchmark()
        .expect("source-shaped renderer supports restore after clipped content");
    assert!(output.execution_inventory.clipped_path_pipeline_set);
}

#[test]
fn native_metal_forced_atomic_nested_clip_supports_gradient_content() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream_text = std::fs::read_to_string(
        fixture_root.join("streams/first-light-nested-clip-probe.rive-stream"),
    )
    .expect("read nested clip stream");
    let stream_text = stream_text
        .replace(
            "clearColor value=0x00000000\n",
            "clearColor value=0x00000000\nmakeLinearGradient id=1 start=(0,0) end=(640,640) stops=[{color=0xffffffff,stop=0},{color=0xff000000,stop=1}]\n",
        )
        .replacen("shader=0}\nframe", "shader=1}\nframe", 1);
    let stream = RenderStream::parse(&stream_text).expect("parse clipped-gradient stream");
    let (width, height) = stream.frame_size.expect("nested clip frame size");
    let mut factory = NativeMetalFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
        .expect("force native Metal generic-atomic mode");
    let mut frame = factory
        .begin_frame(stream.clear_color.unwrap_or(0))
        .expect("acquire forced-atomic clipped-gradient frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay clipped gradient through the public seam");

    let output = frame
        .finish_for_benchmark()
        .expect("source-shaped renderer supports gradients under nested clips");
    assert!(output.execution_inventory.clipped_path_pipeline_set);
    assert!(output.execution_inventory.gradient_texture);
}

#[test]
fn native_metal_gradient_cubic_matches_cpp_metal_and_rust_wgpu_oracles() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(
            fixture_root.join("streams/first-light-gradient-cubic.rive-stream"),
        )
        .expect("read gradient cubic stream"),
    )
    .expect("parse gradient cubic stream");
    let (width, height) = stream.frame_size.expect("gradient cubic frame size");

    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        width,
        height,
        RenderMode::ClockwiseAtomic,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..NativeMetalContextOptions::default()
        },
    )
    .expect("create synchronous native Metal clockwise-atomic renderer");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire native Metal gradient frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay gradient cubic through Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish native Metal gradient frame");
    let actual = output.pixels;

    let cpp_metal = read_png(fixture_root.join("reference/metal/first-light-gradient-cubic.png"));
    assert_rgba8_with_tolerance(
        &actual,
        &cpp_metal,
        1,
        None,
        true,
        "native Rust Metal versus pinned C++ Metal",
    );
    #[cfg(feature = "rust-wgpu")]
    {
        let mut wgpu_factory =
            WgpuFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
                .expect("create Rust-wgpu clockwise-atomic oracle");
        let mut wgpu_frame = wgpu_factory.begin_frame(stream.clear_color.unwrap_or(0));
        stream
            .replay_frame(0, &mut wgpu_factory, &mut wgpu_frame)
            .expect("replay gradient cubic through Rust-wgpu oracle");
        let wgpu_pixels = wgpu_frame.finish().expect("finish Rust-wgpu oracle");
        // Rust-wgpu's default factory path differs by up to 59 LSBs on this
        // antialiased cubic edge. It remains diagnostic-only.
        assert_rgba8_with_tolerance(
            &actual,
            &wgpu_pixels,
            64,
            None,
            false,
            "native Rust Metal versus Rust-wgpu",
        );
    }

    // Exercise the complete physical 1,2,0,1 upload-ring rollover through the
    // public synchronous frame seam. Every completion must make the next slot
    // reusable without changing the pinned same-backend output.
    for cycle in 2..=4 {
        let mut frame = factory
            .begin_frame(stream.clear_color.unwrap_or(0))
            .expect("acquire repeated native Metal gradient frame");
        stream
            .replay_frame(0, &mut factory, &mut frame)
            .expect("replay repeated gradient frame");
        assert_eq!(
            frame.finish().expect("finish repeated gradient frame"),
            actual,
            "native Metal gradient changed on ring cycle {cycle}"
        );
    }
}

#[test]
fn native_metal_atlas_feather_stroke_matches_cpp_metal_and_rust_wgpu_oracles() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(
            fixture_root.join("streams/first-light-atlas-feather-stroke.rive-stream"),
        )
        .expect("read atlas feather stroke stream"),
    )
    .expect("parse atlas feather stroke stream");
    let (width, height) = stream.frame_size.expect("atlas feather frame size");

    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        width,
        height,
        RenderMode::ClockwiseAtomic,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..NativeMetalContextOptions::default()
        },
    )
    .expect("create synchronous native Metal clockwise-atomic renderer");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire native Metal atlas frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay atlas feather stroke through Factory/Renderer seam");
    let output = frame
        .finish_for_benchmark()
        .expect("finish native Metal atlas feather frame");
    let cpp_metal = read_png(
        fixture_root.join("reference/metal/first-light-atlas-feather-stroke-clockwise-atomic.png"),
    );
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        2,
        Some(1024),
        true,
        "native Rust Metal atlas feather versus pinned C++ Metal",
    );

    #[cfg(feature = "rust-wgpu")]
    {
        let mut wgpu_factory =
            WgpuFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
                .expect("create Rust-wgpu clockwise-atomic oracle");
        let mut wgpu_frame = wgpu_factory.begin_frame(stream.clear_color.unwrap_or(0));
        stream
            .replay_frame(0, &mut wgpu_factory, &mut wgpu_frame)
            .expect("replay atlas feather stroke through Rust-wgpu oracle");
        let wgpu_pixels = wgpu_frame.finish().expect("finish Rust-wgpu oracle");
        assert_rgba8_with_tolerance(
            &output.pixels,
            &wgpu_pixels,
            2,
            Some(1024),
            true,
            "native Rust Metal atlas feather versus Rust-wgpu",
        );
    }

    // Exercise the new triangle upload ring through the complete physical
    // 1,2,0,1 rollover. Each frame advances seven active upload rings under one
    // shared reservation; synchronous completion releases that reservation
    // before the next frame. The solid fixture leaves gradient spans inactive.
    for cycle in 2..=4 {
        let mut frame = factory
            .begin_frame(stream.clear_color.unwrap_or(0))
            .expect("acquire repeated native Metal atlas frame");
        stream
            .replay_frame(0, &mut factory, &mut frame)
            .expect("replay repeated atlas frame");
        let cycle_pixels = frame.finish().expect("finish repeated atlas frame");
        assert_eq!(
            cycle_pixels, output.pixels,
            "native Metal atlas changed on triangle-ring cycle {cycle}"
        );
    }
}

#[test]
fn native_metal_two_atlas_feather_strokes_match_cpp_metal_and_rust_wgpu_oracles() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/renderer");
    let stream = RenderStream::parse(
        &std::fs::read_to_string(
            fixture_root.join("streams/first-light-two-atlas-feather-strokes.rive-stream"),
        )
        .expect("read two-atlas-stroke stream"),
    )
    .expect("parse two-atlas-stroke stream");
    let (width, height) = stream.frame_size.expect("two-atlas-stroke frame size");

    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        width,
        height,
        RenderMode::ClockwiseAtomic,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..NativeMetalContextOptions::default()
        },
    )
    .expect("create synchronous native Metal clockwise-atomic renderer");
    let mut frame = factory
        .begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true)
        .expect("acquire native Metal two-atlas-stroke frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay two atlas strokes through native Metal");
    let output = frame
        .finish_for_benchmark()
        .expect("finish native Metal two-atlas-stroke frame");

    let cpp_metal = read_png(
        fixture_root
            .join("reference/metal/first-light-two-atlas-feather-strokes-clockwise-atomic.png"),
    );
    assert_rgba8_with_tolerance(
        &output.pixels,
        &cpp_metal,
        2,
        Some(2_048),
        true,
        "native Rust Metal two-atlas-stroke versus pinned C++ Metal",
    );

    #[cfg(feature = "rust-wgpu")]
    {
        let mut wgpu_factory =
            WgpuFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic)
                .expect("create Rust-wgpu clockwise-atomic oracle");
        let mut wgpu_frame = wgpu_factory.begin_frame(stream.clear_color.unwrap_or(0));
        stream
            .replay_frame(0, &mut wgpu_factory, &mut wgpu_frame)
            .expect("replay two atlas strokes through Rust-wgpu oracle");
        let wgpu_pixels = wgpu_frame.finish().expect("finish Rust-wgpu oracle");
        assert_rgba8_with_tolerance(
            &output.pixels,
            &wgpu_pixels,
            2,
            Some(2_048),
            true,
            "native Rust Metal two-atlas-stroke versus Rust-wgpu",
        );
    }
}

#[test]
fn native_metal_mixed_feather_atlas_prepass_encodes_fill_then_stroke() {
    let stream = RenderStream::parse(
        "rive-golden-stream-v1\n\
frameSize width=64 height=64\n\
clearColor value=0xff000000\n\
drawPath path={id=1,fillRule=0,path={verbs=[move,line,line,line,close],points=[(8,8),(32,8),(32,32),(8,32)]}} paint={id=1,style=fill,color=0xff00ff00,thickness=1,join=0,cap=0,feather=24,blendMode=3,shader=0}\n\
drawPath path={id=2,fillRule=0,path={verbs=[move,line,line,line,close],points=[(24,24),(56,24),(56,56),(24,56)]}} paint={id=2,style=stroke,color=0xffffffff,thickness=8,join=0,cap=0,feather=24,blendMode=3,shader=0}\n\
frame\n",
    )
    .expect("parse mixed feather-atlas stream");
    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        64,
        64,
        RenderMode::ClockwiseAtomic,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..NativeMetalContextOptions::default()
        },
    )
    .expect("create synchronous native Metal clockwise-atomic renderer");
    let mut frame = factory
        .begin_frame_for_benchmark(0xff00_0000, true)
        .expect("begin mixed feather-atlas frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("record mixed feather-atlas frame");

    let output = frame
        .finish_for_benchmark()
        .expect("encode fill then stroke in one feather-atlas pass");
    #[cfg(feature = "rust-wgpu")]
    {
        let mut wgpu_factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic)
            .expect("create Rust-wgpu clockwise-atomic mixed oracle");
        let mut wgpu_frame = wgpu_factory.begin_frame(0xff00_0000);
        stream
            .replay_frame(0, &mut wgpu_factory, &mut wgpu_frame)
            .expect("replay mixed feather-atlas frame through Rust-wgpu");
        let wgpu_pixels = wgpu_frame.finish().expect("finish Rust-wgpu mixed oracle");
        assert_rgba8_with_tolerance(
            &output.pixels,
            &wgpu_pixels,
            2,
            None,
            true,
            "native Rust Metal mixed feather-atlas versus Rust-wgpu",
        );
    }

    let fill_only = &output.pixels[(16 * 64 + 16) * 4..(16 * 64 + 16) * 4 + 4];
    assert!(fill_only[1] > fill_only[0].saturating_add(50));
    assert!(fill_only[1] > fill_only[2].saturating_add(50));
    let stroke_only = &output.pixels[(40 * 64 + 56) * 4..(40 * 64 + 56) * 4 + 4];
    assert!(stroke_only[..3].iter().all(|channel| *channel != 0));
    assert!(
        stroke_only[..3].iter().max().unwrap() - stroke_only[..3].iter().min().unwrap() <= 2,
        "stroke-only sample remains neutral white over the clear color"
    );
}

#[test]
fn native_metal_resize_and_abandoned_frame_leave_factory_reusable() {
    let mut factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    let old_frame = factory
        .begin_frame(0xffff_ffff)
        .expect("acquire abandoned frame command buffer");

    assert!(matches!(
        factory.resize(3, 1),
        Err(RendererError::NativeMetal(_))
    ));
    assert_eq!(factory.dimensions(), (2, 2));

    let old_pixels = old_frame
        .finish()
        .expect("old target generation remains usable after resize");
    assert_eq!(old_pixels.len(), 2 * 2 * 4);

    factory
        .resize(3, 1)
        .expect("resize native Metal target after frame completion");
    assert_eq!(factory.dimensions(), (3, 1));

    let abandoned = factory
        .begin_frame(0xff00_0000)
        .expect("acquire abandoned frame after resize");
    drop(abandoned);

    let pixels = factory
        .begin_frame(0xff12_3456)
        .expect("factory remains reusable after abandonment")
        .finish()
        .expect("submit frame after abandonment and resize");

    assert_eq!(pixels.len(), 3 * 4);
    assert!(pixels
        .chunks_exact(4)
        .all(|pixel| pixel == [0x12, 0x34, 0x56, 0xff]));
}
