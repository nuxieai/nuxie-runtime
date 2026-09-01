//! Regression witnesses for the e949498e command/resource source contracts.
use super::super::{
    command_stream::WirePod,
    deferred_render_factory::DeferredFactory,
    deferred_render_resource::{DeferredRenderPath, DeferredRenderShader},
    render_command_buffer::RenderCommandReader,
    render_commands::*,
    render_replay::{replay_render_commands, ReplayHooks, ResourceTable},
};
use nuxie_render_api::*;

#[test]
fn shader_identity_survives_retention_and_wrapper_address_reuse() {
    let mut factory = DeferredFactory::new();
    let mut paint = factory.make_render_paint();
    let first = factory.make_linear_gradient(0., 0., 1., 1., &[], &[]);
    let mut wrapper = first
        .as_any()
        .downcast_ref::<DeferredRenderShader>()
        .unwrap()
        .clone();
    let address = &wrapper as *const DeferredRenderShader;
    let first_identity = wrapper.shader_identity();
    paint.shader(Some(&wrapper));
    drop(first);
    factory.reset_frame();

    let retained = wrapper.retain_shader();
    assert_eq!(retained.shader_identity(), first_identity);
    paint.shader(Some(retained.as_ref()));
    assert!(factory.buffer.lock().unwrap().command_bytes().is_empty());
    drop(retained);

    let second = factory.make_radial_gradient(0., 0., 2., &[], &[]);
    wrapper = second
        .as_any()
        .downcast_ref::<DeferredRenderShader>()
        .unwrap()
        .clone();
    assert_eq!(&wrapper as *const DeferredRenderShader, address);
    assert_ne!(wrapper.shader_identity(), first_identity);
    let shader_id = wrapper.base.id;
    drop(second);
    factory.reset_frame();
    paint.shader(Some(&wrapper));

    let buffer = factory.buffer.lock().unwrap();
    let mut reader = RenderCommandReader::new(buffer.command_bytes(), buffer.blob_bytes());
    assert_eq!(reader.next_u8(), Some(RenderCmd::PaintShader as u8));
    assert_eq!(reader.read::<PaintShaderPod>().shader, shader_id);
    assert!(reader.next_u8().is_none());
}

#[test]
fn paint_destroys_its_last_shader_reference_before_its_resource_base() {
    let mut factory = DeferredFactory::new();
    let mut paint = factory.make_render_paint();
    let shader = factory.make_linear_gradient(0., 0., 1., 1., &[], &[]);
    paint.shader(Some(shader.as_ref()));
    drop(shader);
    factory.reset_frame();
    drop(paint);
    factory.buffer.lock().unwrap().drain_destroys();

    let buffer = factory.buffer.lock().unwrap();
    let mut reader = RenderCommandReader::new(buffer.command_bytes(), buffer.blob_bytes());
    for kind in [ResourceKind::Shader, ResourceKind::Paint] {
        assert_eq!(reader.next_u8(), Some(RenderCmd::DestroyResource as u8));
        assert_eq!(reader.read::<DestroyResourcePod>().kind, kind as u8);
    }
    assert!(reader.next_u8().is_none());
}

#[test]
fn transform_recorders_use_cpp_xx_xy_yx_yy_tx_ty_wire_order() {
    let mut factory = DeferredFactory::new();
    let mut renderer = factory.make_renderer(None);
    let mut destination = factory.make_empty_render_path();
    let source = factory.make_empty_render_path();
    factory.reset_frame();
    let matrix = Mat2D([1., 2., 3., 4., 5., 6.]);
    renderer.transform(matrix);
    destination.add_render_path(source.as_ref(), matrix);

    let buffer = factory.buffer.lock().unwrap();
    let bytes = buffer.command_bytes();
    // Source TransformPOD and PathAddPathPOD store six consecutive floats in
    // accessor order, which is also Mat2D's storage order in both runtimes.
    let expected: Vec<_> = matrix.0.iter().flat_map(|v| v.to_ne_bytes()).collect();
    assert_eq!(bytes[0], RenderCmd::Transform as u8);
    assert_eq!(&bytes[1..1 + TransformPod::SIZE], expected.as_slice());
    let path_start = 1 + TransformPod::SIZE;
    assert_eq!(bytes[path_start], RenderCmd::PathAddRenderPath as u8);
    assert_eq!(&bytes[path_start + 1 + 8..], expected.as_slice());
}

#[test]
fn replay_consumes_cpp_transform_fields_without_transposing_off_diagonals() {
    let mut factory = DeferredFactory::new();
    let _destination = factory.make_empty_render_path();
    let mut source = factory.make_empty_render_path();
    source.move_to(7., 11.);
    DeferredRenderPath::flush_scratch_of(source.as_ref());
    let mut buffer = factory.buffer.lock().unwrap();
    buffer.append(
        RenderCmd::Transform,
        &TransformPod {
            xx: 1.,
            xy: 2.,
            yx: 3.,
            yy: 4.,
            tx: 5.,
            ty: 6.,
        },
    );
    buffer.append(
        RenderCmd::PathAddRenderPath,
        &PathAddPathPod {
            path: 0,
            src: 1,
            xx: 1.,
            xy: 2.,
            yx: 3.,
            yy: 4.,
            tx: 5.,
            ty: 6.,
        },
    );

    let mut actual = SerializingFactory::new();
    let mut renderer = actual.make_renderer();
    replay_render_commands(
        &mut actual,
        Some(&mut renderer),
        buffer.command_bytes(),
        buffer.blob_bytes(),
        &mut ResourceTable::default(),
        &mut ReplayHooks::default(),
    );
    let mut expected = SerializingFactory::new();
    let mut destination = expected.make_empty_render_path();
    let mut source = expected.make_empty_render_path();
    source.move_to(7., 11.);
    let matrix = Mat2D([1., 2., 3., 4., 5., 6.]);
    expected.make_renderer().transform(matrix);
    destination.add_render_path(source.as_ref(), matrix);
    assert_eq!(&*actual.bytes(), &*expected.bytes());
}

#[test]
fn replay_self_append_matches_a_frozen_non_deferred_source_copy() {
    let mut source_geometry = RawPath::new();
    source_geometry.move_to(1.0, 2.0);
    source_geometry.line_to(3.0, 4.0);
    let matrix = Mat2D([2.0, 0.5, -0.25, 3.0, 5.0, 6.0]);

    let mut recorded = DeferredFactory::new();
    let mut path = recorded.make_render_path(source_geometry.clone(), FillRule::EvenOdd);
    path.add_render_path_self(matrix);

    let buffer = recorded.buffer.lock().unwrap();
    let mut actual = SerializingFactory::new();
    replay_render_commands(
        &mut actual,
        None,
        buffer.command_bytes(),
        buffer.blob_bytes(),
        &mut ResourceTable::default(),
        &mut ReplayHooks::default(),
    );

    let mut expected = SerializingFactory::new();
    let mut expected_path = expected.make_render_path(source_geometry.clone(), FillRule::EvenOdd);
    let mut transformed_copy = RawPath::new();
    transformed_copy.add_path(&source_geometry, matrix);
    expected_path.add_raw_path(&transformed_copy);

    assert_eq!(&*actual.bytes(), &*expected.bytes());
}
