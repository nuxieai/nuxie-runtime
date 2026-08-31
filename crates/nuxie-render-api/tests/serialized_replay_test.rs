//! `tests/unit_tests/runtime/serialized_replay_test.cpp` at e949498e.
use nuxie_render_api::serialized_replay::{SerializedReplayHooks, replay_serialized_commands};
use nuxie_render_api::*;

#[test]
fn serialized_2d_commands_replay_byte_identically() {
    let mut a = SerializingFactory::default();
    a.frame_size(256, 256);
    a.add_frame();
    let mut renderer_a = a.make_renderer();
    let mut paint = a.make_render_paint();
    paint.color(0xff112233);
    paint.style(RenderPaintStyle::Stroke);
    paint.thickness(3.5);
    paint.join(StrokeJoin::Round);
    paint.cap(StrokeCap::Square);
    paint.blend_mode(BlendMode::Multiply);
    paint.feather(2.0);

    let mut rp = RawPath::new();
    rp.move_to(0.0, 0.0);
    rp.line_to(10.0, 0.0);
    rp.cubic_to(10.0, 5.0, 5.0, 10.0, 0.0, 10.0);
    rp.close();
    let path = a.make_render_path(rp, FillRule::EvenOdd);
    let mut clip = a.make_empty_render_path();
    let mut cp = RawPath::new();
    cp.move_to(0.0, 0.0);
    cp.line_to(20.0, 0.0);
    cp.line_to(20.0, 20.0);
    cp.close();
    clip.add_raw_path(&cp);
    let grad = a.make_linear_gradient(
        0.0,
        0.0,
        100.0,
        100.0,
        &[0xffff0000, 0xff0000ff],
        &[0.0, 1.0],
    );
    let mut paint2 = a.make_render_paint();
    paint2.shader(Some(grad.as_ref()));
    renderer_a.save();
    renderer_a.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 5.0, 7.0]));
    renderer_a.clip_path(clip.as_ref());
    renderer_a.modulate_opacity(0.5);
    renderer_a.draw_path(path.as_ref(), paint.as_ref());
    renderer_a.draw_path(path.as_ref(), paint2.as_ref());
    renderer_a.restore();

    let mut b = PersistentFactory::new(SerializingFactory::default());
    let mut renderer_b = b.borrow().make_renderer();
    let frame_factory = b.clone();
    let size_factory = b.clone();
    let mut hooks = SerializedReplayHooks {
        on_frame: Some(Box::new(move || frame_factory.borrow_mut().add_frame())),
        on_frame_size: Some(Box::new(move |w, h| {
            size_factory.borrow_mut().frame_size(w, h)
        })),
    };
    assert!(replay_serialized_commands(
        &a.bytes(),
        &mut b,
        &mut renderer_b,
        &mut hooks
    ));
    let sa = a.bytes();
    let b = b.borrow();
    let sb = b.bytes();
    assert_eq!(sa.len(), sb.len());
    assert_eq!(&*sa, &*sb);
}

#[test]
fn serialized_replay_rejects_a_bad_header() {
    let garbage = [b'X', b'X', b'X', b'X', 1, 0, 0, 0];
    let mut b = SerializingFactory::default();
    let mut renderer = b.make_renderer();
    assert!(!replay_serialized_commands(
        &garbage,
        &mut b,
        &mut renderer,
        &mut SerializedReplayHooks::default()
    ));
}
