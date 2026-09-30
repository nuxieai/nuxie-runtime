//! tests/gm/clipstrokes.cpp at 51c02b50. Geometry uses gmutils.cpp's circle constant.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_render_factory::DeferredFactory,
    render_replay::{ReplayHooks, ResourceTable, replay_render_commands},
};

const DIM: u32 = 1200;
const C: f32 = 0.5519150244935105707435627;
const COLORS: [[u32; 3]; 3] = [
    [0xff881177, 0xffeedd00, 0xff00bbcc],
    [0xffaa3355, 0xff99dd55, 0xffee9944],
    [0xffcc6666, 0xff3366bb, 0xff22ccbb],
];

fn rect(l: f32, t: f32, r: f32, b: f32) -> RawPath {
    let mut p = RawPath::new();
    p.move_to(l, t);
    p.line_to(r, t);
    p.line_to(r, b);
    p.line_to(l, b);
    p.close();
    p
}
fn circle(x: f32, y: f32, radius: f32) -> RawPath {
    let points = [
        (1., 0.),
        (1., C),
        (C, 1.),
        (0., 1.),
        (-C, 1.),
        (-1., C),
        (-1., 0.),
        (-1., -C),
        (-C, -1.),
        (0., -1.),
        (C, -1.),
        (1., -C),
        (1., 0.),
    ];
    let mut p = RawPath::new();
    p.move_to(x + radius, y);
    for i in (1..=12).step_by(3) {
        let a = points[i];
        let b = points[i + 1];
        let c = points[i + 2];
        p.cubic_to(
            a.0 * radius + x,
            a.1 * radius + y,
            b.0 * radius + x,
            b.1 * radius + y,
            c.0 * radius + x,
            c.1 * radius + y,
        );
    }
    p.close();
    p
}
fn rounded_cell(x: usize, y: usize) -> RawPath {
    let l = (400 * x + 25) as f32;
    let t = (400 * y + 25) as f32;
    let r = (400 * (x + 1) - 25) as f32;
    let b = (400 * (y + 1) - 25) as f32;
    let rad = 50.;
    let mut p = RawPath::new();
    p.move_to(l + rad, t);
    p.line_to(r - rad, t);
    p.cubic_to(r - rad * (1. - C), t, r, t + rad * C, r, t + rad);
    p.line_to(r, b - rad);
    p.cubic_to(r, b - rad * (1. - C), r - rad * (1. - C), b, r - rad, b);
    p.line_to(l + rad, b);
    p.cubic_to(l + rad * C, b, l, b - rad * (1. - C), l, b - rad);
    p.line_to(l, t + rad);
    p.cubic_to(l, t + rad * C, l + rad * C, t, l + rad, t);
    p.close();
    p
}
fn background(f: &mut dyn Factory, r: &mut dyn Renderer) {
    {
        let mut paint = f.make_render_paint();
        paint.color(0xff000000);
        let path = f.make_render_path(rect(0., 0., DIM as f32, DIM as f32), FillRule::NonZero);
        r.draw_path(path.as_ref(), paint.as_ref());
    }
    let mut paint = f.make_render_paint();
    paint.color(0xff2c1642);
    paint.feather(10.);
    let mut y_step = 40.;
    let mut y = 20.;
    while y < DIM as f32 {
        let path = f.make_render_path(
            rect(-100., y, DIM as f32 + 100., y + y_step * 0.3),
            FillRule::Clockwise,
        );
        r.draw_path(path.as_ref(), paint.as_ref());
        y += y_step;
        y_step *= 1.12;
        drop(path);
        y += 1.; // The upstream for-loop increment is additional to y += yStep.
    }
}
#[derive(Clone, Copy)]
enum Scene {
    Basic,
    PathInStroke,
    StrokeInPath,
    StrokeInStroke,
    Open,
}

fn draw_scene(f: &mut dyn Factory, r: &mut dyn Renderer, scene: Scene) {
    let open = if matches!(scene, Scene::Open) {
        let mut raw = RawPath::new();
        raw.move_to(50., 50.);
        raw.line_to(350., 50.);
        raw.line_to(350., 350.);
        raw.line_to(50., 350.);
        Some(f.make_render_path(raw, FillRule::NonZero))
    } else {
        None
    };
    background(f, r);
    if matches!(scene, Scene::Basic | Scene::Open) {
        for y in 0..3 {
            for x in 0..3 {
                r.save();
                let clip = if matches!(scene, Scene::Basic) {
                    Some(f.make_render_path(rounded_cell(x, y), FillRule::NonZero))
                } else {
                    None
                };
                if matches!(scene, Scene::Open) {
                    r.transform(Mat2D([1., 0., 0., 1., (400 * x) as f32, (400 * y) as f32]));
                    r.clip_stroke(
                        open.as_ref().unwrap().as_ref(),
                        &StrokeParams {
                            thickness: 50.,
                            join: [StrokeJoin::Miter, StrokeJoin::Round, StrokeJoin::Bevel][x],
                            cap: [StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square][y],
                        },
                    );
                } else {
                    r.clip_stroke(
                        clip.as_ref().unwrap().as_ref(),
                        &StrokeParams {
                            thickness: (y * 3 + x + 1) as f32 * 3.,
                            ..StrokeParams::default()
                        },
                    );
                }
                let mut paint = f.make_render_paint();
                paint.color(COLORS[x][y]);
                let radius = if matches!(scene, Scene::Open) {
                    r.transform(Mat2D([
                        1.,
                        0.,
                        0.,
                        1.,
                        -((400 * x) as f32),
                        -((400 * y) as f32),
                    ]));
                    3000.
                } else {
                    480.
                };
                let path = f.make_render_path(circle(600., 600., radius), FillRule::Clockwise);
                r.draw_path(path.as_ref(), paint.as_ref());
                drop(path);
                drop(paint);
                drop(clip);
                r.restore();
            }
        }
        return;
    }
    let colors = [
        0xffeedd00, 0xffee9944, 0xffcc6666, 0xffaa3355, 0xff881177, 0xff3366bb, 0xff00bbcc,
        0xff22ccbb, 0xff99dd55,
    ];
    let outer = f.make_render_path(circle(600., 600., 480.), FillRule::NonZero);
    let inner = f.make_render_path(circle(50., 50., 50.), FillRule::NonZero);
    r.save();
    if matches!(scene, Scene::StrokeInPath) {
        r.clip_path(outer.as_ref());
    } else {
        r.clip_stroke(
            outer.as_ref(),
            &StrokeParams {
                thickness: 200.,
                ..StrokeParams::default()
            },
        );
    }
    for y_index in 0..3 {
        for x_index in 0..3 {
            r.save();
            let clip = f.make_render_path(rounded_cell(x_index, y_index), FillRule::NonZero);
            if matches!(scene, Scene::PathInStroke) {
                r.clip_path(clip.as_ref());
            } else {
                r.clip_stroke(
                    clip.as_ref(),
                    &StrokeParams {
                        thickness: 20.,
                        ..StrokeParams::default()
                    },
                );
            }
            let mut color_index = 0;
            for y in (5..DIM).step_by(120) {
                for x in (5..DIM).step_by(120) {
                    let mut paint = f.make_render_paint();
                    paint.color(colors[color_index % 9]);
                    color_index += 1;
                    r.save();
                    r.transform(Mat2D([1., 0., 0., 1., x as f32, y as f32]));
                    r.draw_path(inner.as_ref(), paint.as_ref());
                    r.restore();
                }
            }
            drop(clip);
            r.restore();
        }
    }
    r.restore();
}
fn render(scene: Scene, deferred: bool) -> Vec<u8> {
    let mut host = GmHost::with_size(0, true, DIM, DIM);
    let screen = host.screen();
    if deferred {
        let mut f = DeferredFactory::new();
        let mut r = f.make_renderer(None);
        draw_scene(&mut f, &mut r, scene);
        let buffer = f.buffer.lock().unwrap();
        replay_render_commands(
            &mut host.factory,
            Some(screen.borrow_mut().as_mut()),
            buffer.command_bytes(),
            buffer.blob_bytes(),
            &mut ResourceTable::default(),
            &mut ReplayHooks::default(),
        );
    } else {
        draw_scene(&mut host.factory, screen.borrow_mut().as_mut(), scene);
    }
    host.finish()
}
fn check(scene: Scene) {
    // This existing host supports native Metal raster ordering, not depth/stencil.
    // The shared depth/stencil rules have focused GPU state assertions separately.
    let immediate = render(scene, false);
    let deferred = render(scene, true);
    assert_eq!(immediate.len(), (DIM * DIM * 4) as usize);
    assert_eq!(immediate, deferred, "immediate/deferred clip stroke GM");
}
#[test]
fn clip_stroke_basic() {
    check(Scene::Basic);
}
#[test]
fn clip_stroke_nested_a() {
    check(Scene::PathInStroke);
}
#[test]
fn clip_stroke_nested_b() {
    check(Scene::StrokeInPath);
}
#[test]
fn clip_stroke_nested_c() {
    check(Scene::StrokeInStroke);
}
#[test]
fn clip_stroke_open() {
    check(Scene::Open);
}
