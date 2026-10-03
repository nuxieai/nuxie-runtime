//! Complete `tests/gm/stroke_position_shapes.cpp` restored by cdeabe75.
use super::{clipstrokes::rect, ore_gm_helper::*};
use crate::deferred::cmd::{
    deferred_render_factory::DeferredFactory,
    render_replay::{ReplayHooks, ResourceTable, replay_render_commands},
};

const CELL: f32 = 160.0;
const THICKNESS: f32 = 18.0;

fn open_zigzag() -> RawPath {
    let mut p = RawPath::new();
    p.move_to(24., 50.);
    p.line_to(60., 120.);
    p.line_to(96., 50.);
    p.line_to(136., 120.);
    p
}
fn notched_square() -> RawPath {
    let mut p = RawPath::new();
    p.move_to(30., 30.);
    p.line_to(130., 30.);
    p.line_to(130., 130.);
    p.line_to(80., 80.);
    p.line_to(30., 130.);
    p.close();
    p
}
fn square_with_hole() -> RawPath {
    let mut p = rect(30., 30., 130., 130.);
    p.move_to(60., 60.);
    p.line_to(100., 60.);
    p.line_to(100., 100.);
    p.line_to(60., 100.);
    p.close();
    p
}
fn opposite_squares() -> RawPath {
    let mut p = rect(20., 40., 90., 110.);
    p.move_to(110., 70.);
    p.line_to(110., 120.);
    p.line_to(140., 120.);
    p.line_to(140., 70.);
    p.close();
    p
}
fn background(f: &mut dyn Factory, r: &mut dyn Renderer, height: f32) {
    let mut bg = f.make_render_paint();
    bg.color(0xff000000);
    let path = f.make_render_path(rect(0., 0., 3. * CELL, height), FillRule::NonZero);
    r.draw_path(path.as_ref(), bg.as_ref());
}
fn cell(
    f: &mut dyn Factory,
    r: &mut dyn Renderer,
    stroke_path: &dyn RenderPath,
    fill_path: &dyn RenderPath,
    params: StrokeParams,
    feather: f32,
) {
    let mut stroke = f.make_render_paint();
    stroke.color(0xffefefef);
    stroke.stroke(&params);
    stroke.feather(feather);
    r.draw_path(stroke_path, stroke.as_ref());
    let mut fill = f.make_render_paint();
    fill.color(0x5f881177);
    r.draw_path(fill_path, fill.as_ref());
    let mut line = f.make_render_paint();
    line.color(0xffff3030);
    line.stroke(&StrokeParams {
        thickness: 1.5,
        ..StrokeParams::default()
    });
    r.draw_path(stroke_path, line.as_ref());
}
const POSITIONS: [StrokePosition; 3] = [
    StrokePosition::Inside,
    StrokePosition::Center,
    StrokePosition::Outside,
];
fn draw_shapes_by_join(f: &mut dyn Factory, r: &mut dyn Renderer, feather: f32) {
    background(f, r, 9. * CELL);
    let shapes = [
        f.make_render_path(open_zigzag(), FillRule::NonZero),
        f.make_render_path(notched_square(), FillRule::NonZero),
        f.make_render_path(square_with_hole(), FillRule::EvenOdd),
    ];
    for (s, shape) in shapes.iter().enumerate() {
        for (j, join) in [StrokeJoin::Miter, StrokeJoin::Round, StrokeJoin::Bevel]
            .into_iter()
            .enumerate()
        {
            for (p, position) in POSITIONS.into_iter().enumerate() {
                r.save();
                r.translate(p as f32 * CELL, (s * 3 + j) as f32 * CELL);
                cell(
                    f,
                    r,
                    shape.as_ref(),
                    shape.as_ref(),
                    StrokeParams {
                        thickness: THICKNESS,
                        join,
                        position,
                        ..StrokeParams::default()
                    },
                    feather,
                );
                r.restore();
            }
        }
    }
}
fn draw_winding(f: &mut dyn Factory, r: &mut dyn Renderer) {
    background(f, r, 2. * CELL);
    let hole = f.make_render_path(square_with_hole(), FillRule::NonZero);
    let hole_fill = f.make_render_path(square_with_hole(), FillRule::EvenOdd);
    let pair = f.make_render_path(opposite_squares(), FillRule::NonZero);
    for (p, position) in POSITIONS.into_iter().enumerate() {
        let params = StrokeParams {
            thickness: 12.,
            position,
            ..StrokeParams::default()
        };
        r.save();
        r.translate(p as f32 * CELL, 0.);
        cell(f, r, hole.as_ref(), hole_fill.as_ref(), params, 0.);
        r.translate(0., CELL);
        cell(f, r, pair.as_ref(), pair.as_ref(), params, 0.);
        r.restore();
    }
}
fn check(name: &str, height: u32, draw: impl Fn(&mut dyn Factory, &mut dyn Renderer)) {
    let render = |deferred| {
        let mut host = GmHost::with_size(0, true, 480, height);
        let screen = host.screen();
        if deferred {
            let mut factory = DeferredFactory::new();
            let mut renderer = factory.make_renderer(None);
            draw(&mut factory, &mut renderer);
            let buffer = factory.buffer.lock().unwrap();
            replay_render_commands(
                &mut host.factory,
                Some(screen.borrow_mut().as_mut()),
                buffer.command_bytes(),
                buffer.blob_bytes(),
                &mut ResourceTable::default(),
                &mut ReplayHooks::default(),
            );
        } else {
            draw(&mut host.factory, screen.borrow_mut().as_mut());
        }
        host.finish()
    };
    let immediate = render(false);
    assert_eq!(immediate, render(true), "{name}: immediate/deferred pixels");
    assert_cpp_gm_pixels_with_size(name, 480, height, immediate);
}
#[test]
fn stroke_position_shapes() {
    check("stroke_position_shapes", 1440, |f, r| {
        draw_shapes_by_join(f, r, 0.)
    });
}
#[test]
fn stroke_position_shapes_feathered() {
    check("stroke_position_shapes_feathered", 1440, |f, r| {
        draw_shapes_by_join(f, r, 6.)
    });
}
#[test]
fn stroke_position_winding() {
    check("stroke_position_winding", 320, draw_winding);
}
