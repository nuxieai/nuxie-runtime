//! `tests/gm/emptypath.cpp` at 7aa93402, including both addPath degenerate cases.
use super::ore_gm_helper::assert_cpp_gm_pixels_with_size;
use crate::{
    RenderMode,
    native_metal::{NativeMetalContextOptions, NativeMetalFactory, ShaderCompilationMode},
};
use nuxie_render_api::{
    Aabb, Factory, FillRule, Mat2D, RawPath, RenderPaintStyle, RenderPath, Renderer, StrokeCap,
    StrokeJoin,
};

fn path(factory: &mut dyn Factory, kind: usize) -> Box<dyn RenderPath> {
    let mut path = factory.make_empty_render_path();
    match kind {
        0..=2 => {
            for x in [40.0, 80.0, 120.0] {
                path.move_to(x, 40.0);
                if kind == 1 {
                    path.close();
                }
                if kind == 2 {
                    path.line_to(x, 40.0);
                }
            }
        }
        3 => {
            path.move_to(40.0, 40.0);
            path.move_to(80.0, 40.0);
            path.close();
            path.move_to(120.0, 40.0);
            path.line_to(120.0, 40.0);
        }
        4 => {
            let mut inner = factory.make_empty_render_path();
            inner.move_to(80.0, 40.0);
            inner.line_to(80.0, 40.0);
            path.move_to(40.0, 40.0);
            path.add_render_path(inner.as_ref(), Mat2D::IDENTITY);
            path.move_to(120.0, 40.0);
        }
        5 => {
            let mut inner = factory.make_empty_render_path();
            inner.move_to(40.0, 40.0);
            inner.line_to(40.0, 40.0);
            path.add_render_path(inner.as_ref(), Mat2D::IDENTITY);
            path.move_to(80.0, 40.0);
            path.move_to(120.0, 40.0);
            path.line_to(120.0, 40.0);
        }
        _ => unreachable!(),
    }
    path
}
fn render(name: &str, stroke: bool, feather: bool) {
    let mut factory = NativeMetalFactory::new_with_mode_and_context_options(
        180,
        780,
        RenderMode::RasterOrdering,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..Default::default()
        },
    )
    .expect("Metal empty-path GM");
    let mut renderer = factory.begin_frame(0xffffffff).expect("empty-path frame");
    let mut paint = factory.make_render_paint();
    if stroke {
        paint.style(RenderPaintStyle::Stroke);
        paint.thickness(21.0);
    }
    if feather {
        paint.feather(21.0);
    }
    let mut dot = factory.make_render_paint();
    dot.color(0xffff0000);
    dot.style(RenderPaintStyle::Fill);
    for j in 0..3 {
        paint.cap([StrokeCap::Butt, StrokeCap::Square, StrokeCap::Round][j]);
        paint.join([StrokeJoin::Miter, StrokeJoin::Round, StrokeJoin::Bevel][j]);
        for kind in 0..6 {
            for x in [40.0, 80.0, 120.0] {
                let mut raw = RawPath::new();
                raw.add_oval(Aabb::new(x - 3.5, 36.5, x + 3.5, 43.5));
                let oval = factory.make_render_path(raw, FillRule::NonZero);
                renderer.draw_path(oval.as_ref(), dot.as_ref());
            }
            let mut path = path(&mut factory, kind);
            path.fill_rule(FillRule::Clockwise);
            renderer.draw_path(path.as_ref(), paint.as_ref());
            renderer.translate(0.0, 40.0);
        }
    }
    assert_cpp_gm_pixels_with_size(
        name,
        180,
        780,
        renderer.finish().expect("empty-path pixels"),
    );
}
#[test]
fn emptystroke() {
    render("emptystroke", true, false);
}
#[test]
fn emptyfeather() {
    render("emptyfeather", false, true);
}
#[test]
fn emptystrokefeather() {
    render("emptystrokefeather", true, true);
}
