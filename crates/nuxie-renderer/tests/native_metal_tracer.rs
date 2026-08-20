#![cfg(all(
    feature = "native-metal-experimental",
    any(target_os = "ios", target_os = "macos")
))]

use nuxie_render_stream::RenderStream;
use nuxie_renderer::{NativeMetalFactory, WgpuFactory};
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
    require_exact_occupancy: bool,
    label: &str,
) {
    assert_eq!(actual.len(), expected.len(), "{label}: byte length");
    let mut maximum_delta = 0u8;
    let mut first_excess = None;
    let mut first_occupancy_mismatch = None;
    for (pixel_index, (actual, expected)) in actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .enumerate()
    {
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
    assert_eq!(
        first_excess, None,
        "{label}: exceeded {maximum_allowed_delta}-LSB RGBA8 tolerance; maximum delta={maximum_delta}"
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
            0x02, 0x03, 0x04, 0x11, 0x02, 0x03, 0x04, 0x11, 0x02, 0x03, 0x04, 0x11, 0x02, 0x03,
            0x04, 0x11,
        ]
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

    let mut wgpu_factory = WgpuFactory::new(width, height).expect("create Rust-wgpu oracle");
    let mut wgpu_frame = wgpu_factory.begin_frame(stream.clear_color.unwrap_or(0));
    stream
        .replay_frame(0, &mut wgpu_factory, &mut wgpu_frame)
        .expect("replay rectangle through Rust-wgpu oracle");
    let wgpu_pixels = wgpu_frame.finish().expect("finish Rust-wgpu oracle");

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
    assert_eq!(actual, wgpu_pixels, "native Rust Metal versus Rust-wgpu");
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

    let mut wgpu_factory = WgpuFactory::new(width, height).expect("create Rust-wgpu oracle");
    let mut wgpu_frame = wgpu_factory.begin_frame(stream.clear_color.unwrap_or(0));
    stream
        .replay_frame(0, &mut wgpu_factory, &mut wgpu_frame)
        .expect("replay gradient cubic through Rust-wgpu oracle");
    let wgpu_pixels = wgpu_frame.finish().expect("finish Rust-wgpu oracle");

    let mut factory = NativeMetalFactory::new(width, height).expect("create native Metal renderer");
    let mut frame = factory
        .begin_frame(stream.clear_color.unwrap_or(0))
        .expect("acquire native Metal gradient frame");
    stream
        .replay_frame(0, &mut factory, &mut frame)
        .expect("replay gradient cubic through Factory/Renderer seam");
    let actual = frame.finish().expect("finish native Metal gradient frame");

    let cpp_metal = read_png(fixture_root.join("reference/metal/first-light-gradient-cubic.png"));
    assert_rgba8_with_tolerance(
        &actual,
        &cpp_metal,
        1,
        true,
        "native Rust Metal versus pinned C++ Metal",
    );
    // Rust-wgpu's default factory path differs by up to 59 LSBs on this
    // antialiased cubic edge. Keep WGPU as a secondary
    // cross-backend guard without pretending its rasterization is byte-exact.
    assert_rgba8_with_tolerance(
        &actual,
        &wgpu_pixels,
        64,
        false,
        "native Rust Metal versus Rust-wgpu",
    );
}

#[test]
fn native_metal_resize_and_abandoned_frame_leave_factory_reusable() {
    let mut factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    let old_frame = factory
        .begin_frame(0xffff_ffff)
        .expect("acquire abandoned frame command buffer");

    factory.resize(3, 1).expect("resize native Metal target");
    assert_eq!(factory.dimensions(), (3, 1));

    let old_pixels = old_frame
        .finish()
        .expect("old target generation remains usable after resize");
    assert_eq!(old_pixels.len(), 2 * 2 * 4);

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
