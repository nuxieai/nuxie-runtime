//! New mesh and winding scenes from tests/gm/render_canvas.cpp at 7732f41e.
use super::ore_gm_helper::*;
use crate::deferred::cmd::deferred_replayer::DeferredFrameSink;

fn sampler() -> ImageSampler {
    ImageSampler {
        filter: ImageFilter::Nearest,
        ..Default::default()
    }
}

fn buffer(
    factory: &mut dyn Factory,
    kind: RenderBufferType,
    bytes: &[u8],
) -> Box<dyn RenderBuffer> {
    let mut buffer = factory.make_render_buffer(
        kind,
        RenderBufferFlags::MappedOnceAtInitialization,
        bytes.len(),
    );
    buffer.map_mut().copy_from_slice(bytes);
    buffer.unmap();
    buffer
}

fn pixel(pixels: &[u8], x: usize, y: usize) -> &[u8] {
    &pixels[(y * 256 + x) * 4..(y * 256 + x + 1) * 4]
}

#[test]
fn render_canvas_mesh() {
    let mut host = GmHost::new(0xffff0000);
    let canvas = host.canvas(256, 256);
    let renderer = host
        .begin_canvas_content(canvas.clone(), 0xff0000ff)
        .unwrap();
    let mut raw = RawPath::new();
    for i in 0..=40 {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / 40.0;
        let x = 128.0 + 60.0 * theta.cos();
        let y = 80.0 + 60.0 * theta.sin();
        if i == 0 {
            raw.move_to(x, y);
        } else {
            raw.line_to(x, y);
        }
    }
    let path = host.factory.make_render_path(raw, FillRule::NonZero);
    let mut green = host.factory.make_render_paint();
    green.color(0xff00ff00);
    renderer
        .borrow_mut()
        .draw_path(path.as_ref(), green.as_ref());
    host.end_canvas_content();
    let renderer = host.begin_screen_frame(0).unwrap();
    let pts: Vec<u8> = [0.0f32, 0.0, 256.0, 0.0, 0.0, 256.0, 256.0, 256.0]
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect();
    let uvs: Vec<u8> = [0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0]
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect();
    let indices: Vec<u8> = [0u16, 1, 2, 1, 2, 3]
        .into_iter()
        .flat_map(u16::to_ne_bytes)
        .collect();
    let pts = buffer(&mut host.factory, RenderBufferType::Vertex, &pts);
    let uvs = buffer(&mut host.factory, RenderBufferType::Vertex, &uvs);
    let indices = buffer(&mut host.factory, RenderBufferType::Index, &indices);
    renderer.borrow_mut().draw_image_mesh(
        Some(canvas.borrow().render_image().as_ref()),
        sampler(),
        Some(pts.as_ref()),
        Some(uvs.as_ref()),
        Some(indices.as_ref()),
        4,
        6,
        BlendMode::SrcOver,
        1.0,
    );
    let pixels = host.finish();
    assert_eq!(pixel(&pixels, 128, 80), &[0, 255, 0, 255]);
    assert_eq!(pixel(&pixels, 128, 176), &[0, 0, 255, 255]);
}

#[test]
fn render_canvas_winding() {
    let mut host = GmHost::new(0xffff0000);
    let canvas = host.canvas(256, 256);
    let renderer = host
        .begin_canvas_content(canvas.clone(), 0xff0000ff)
        .unwrap();
    renderer.borrow_mut().save();
    let mut raw = RawPath::new();
    raw.add_oval(Aabb::new(18.0, 18.0, 238.0, 238.0));
    let clip = host.factory.make_render_path(raw, FillRule::NonZero);
    renderer.borrow_mut().clip_path(clip.as_ref());
    for (points, color) in [
        (
            [(24.0, 24.0), (120.0, 24.0), (120.0, 120.0), (24.0, 120.0)],
            0xff00ff00,
        ),
        (
            [
                (136.0, 136.0),
                (136.0, 232.0),
                (232.0, 232.0),
                (232.0, 136.0),
            ],
            0xffffff00,
        ),
    ] {
        let mut raw = RawPath::new();
        raw.move_to(points[0].0, points[0].1);
        for (x, y) in &points[1..] {
            raw.line_to(*x, *y);
        }
        raw.close();
        let path = host.factory.make_render_path(raw, FillRule::Clockwise);
        let mut paint = host.factory.make_render_paint();
        paint.color(color);
        renderer
            .borrow_mut()
            .draw_path(path.as_ref(), paint.as_ref());
    }
    renderer.borrow_mut().restore();
    host.end_canvas_content();
    let renderer = host.begin_screen_frame(0).unwrap();
    renderer.borrow_mut().draw_image(
        Some(canvas.borrow().render_image().as_ref()),
        sampler(),
        BlendMode::SrcOver,
        1.0,
    );
    let pixels = host.finish();
    assert_eq!(pixel(&pixels, 80, 80), &[0, 255, 0, 255]);
    assert_eq!(pixel(&pixels, 176, 176), &[0, 0, 255, 255]);
    assert_eq!(pixel(&pixels, 25, 25), &[0, 0, 255, 255]);
}
