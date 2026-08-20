#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_render_api::{Factory, FillRule, RawPath, Renderer};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_renderer::NativeMetalFactory;

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn main() {
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
    let mut frame = factory
        .begin_frame(0)
        .expect("acquire native Metal tracer command buffer");
    frame.draw_path(path.as_ref(), paint.as_ref());
    let pixels = frame.finish().expect("finish native Metal tracer");
    assert_eq!(pixels[(32 * 64 + 32) * 4..][..4], [0, 255, 0, 255]);
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn main() {}
