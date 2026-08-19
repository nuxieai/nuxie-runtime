#![cfg(all(
    feature = "native-metal-experimental",
    any(target_os = "ios", target_os = "macos")
))]

use nuxie_render_stream::RenderStream;
use nuxie_renderer::{NativeMetalFactory, WgpuFactory};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[test]
fn native_metal_frame_clears_and_reads_back() {
    let factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    let pixels = factory
        .begin_frame(0x1122_3344)
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
    let mut frame = factory.begin_frame(stream.clear_color.unwrap_or(0));
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
fn native_metal_resize_and_abandoned_frame_leave_factory_reusable() {
    let mut factory = NativeMetalFactory::new(2, 2).expect("create native Metal renderer");
    drop(factory.begin_frame(0xffff_ffff));

    factory.resize(3, 1).expect("resize native Metal target");
    let pixels = factory
        .begin_frame(0xff12_3456)
        .finish()
        .expect("submit frame after abandonment and resize");

    assert_eq!(pixels.len(), 3 * 4);
    assert!(pixels
        .chunks_exact(4)
        .all(|pixel| pixel == [0x12, 0x34, 0x56, 0xff]));
}
