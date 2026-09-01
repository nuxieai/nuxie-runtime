//! Source regressions for 309e901f deferred query-geometry retention.

use super::super::{
    command_stream::CommandReader,
    deferred_render_factory::DeferredFactory,
    deferred_render_resource::DeferredRenderPath,
    deferred_session::DeferredSession,
    render_commands::{MakeIdPod, PathAddPathPod, PathRawPod, RenderCmd},
};
use nuxie_render_api::{Aabb, Factory, FillRule, Mat2D, NullFactory, RawPath, RenderPath};

fn deferred(path: &dyn RenderPath) -> &DeferredRenderPath {
    path.as_any()
        .downcast_ref::<DeferredRenderPath>()
        .expect("deferred factory path")
}

fn deferred_mut(path: &mut dyn RenderPath) -> &mut DeferredRenderPath {
    path.as_any_mut()
        .downcast_mut::<DeferredRenderPath>()
        .expect("deferred factory path")
}

fn line_path(from: (f32, f32), to: (f32, f32)) -> RawPath {
    let mut path = RawPath::new();
    path.move_to(from.0, from.1);
    path.line_to(to.0, to.1);
    path
}

#[test]
fn retained_query_mirrors_mutations_in_source_order() {
    let mut session = DeferredSession::new(None);
    let mut path = session.make_empty_render_path();
    assert!(deferred(path.as_ref()).query_raw_path().is_none());
    deferred_mut(path.as_mut()).retain_query_geometry(None);

    path.fill_rule(FillRule::EvenOdd);
    path.move_to(1.0, 2.0);
    path.line_to(4.0, 2.0);

    let bulk = line_path((-2.0, -3.0), (6.0, 5.0));
    path.add_raw_path(&bulk);
    path.cubic_to(7.0, 8.0, 9.0, -4.0, 10.0, 1.0);
    path.close();

    let mut expected = RawPath::new();
    expected.move_to(1.0, 2.0);
    expected.line_to(4.0, 2.0);
    expected.add_path(&bulk, Mat2D::IDENTITY);
    expected.cubic_to(7.0, 8.0, 9.0, -4.0, 10.0, 1.0);
    expected.close();

    let path = deferred(path.as_ref());
    assert_eq!(path.current_fill_rule(), FillRule::EvenOdd);
    assert_eq!(path.query_raw_path(), Some(&expected));
    assert_eq!(
        path.query_raw_path().and_then(RawPath::bounds),
        Some(Aabb::new(-2.0, -4.0, 10.0, 8.0))
    );
}

#[test]
fn retaining_a_seed_refreshes_instead_of_appending_and_rewind_clears_it() {
    let mut session = DeferredSession::new(None);
    let first = line_path((1.0, 2.0), (3.0, 4.0));
    let second = line_path((-5.0, -6.0), (-7.0, -8.0));
    let mut path = session.make_render_path(first.clone(), FillRule::NonZero);

    assert!(deferred(path.as_ref()).query_raw_path().is_none());
    deferred_mut(path.as_mut()).retain_query_geometry(Some(&first));
    assert_eq!(deferred(path.as_ref()).query_raw_path(), Some(&first));

    path.line_to(9.0, 10.0);
    deferred_mut(path.as_mut()).retain_query_geometry(Some(&second));
    assert_eq!(deferred(path.as_ref()).query_raw_path(), Some(&second));

    path.rewind();
    let query = deferred(path.as_ref()).query_raw_path().unwrap();
    assert!(query.verbs().is_empty());
    assert!(query.points().is_empty());
}

#[test]
fn transformed_append_requires_query_geometry_on_both_deferred_paths() {
    let mut session = DeferredSession::new(None);
    let retained_geometry = line_path((1.0, 2.0), (3.0, 4.0));
    let mut retained_source =
        session.make_render_path(retained_geometry.clone(), FillRule::NonZero);
    deferred_mut(retained_source.as_mut()).retain_query_geometry(Some(&retained_geometry));

    let ignored_geometry = line_path((20.0, 30.0), (40.0, 50.0));
    let ignored_source = session.make_render_path(ignored_geometry, FillRule::NonZero);

    let mut destination = session.make_empty_render_path();
    deferred_mut(destination.as_mut()).retain_query_geometry(None);
    let transform = Mat2D([2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    destination.add_render_path(retained_source.as_ref(), transform);

    let mut expected = RawPath::new();
    expected.add_path_with_transform(&retained_geometry, transform);
    assert_eq!(
        deferred(destination.as_ref()).query_raw_path(),
        Some(&expected)
    );

    destination.add_render_path(ignored_source.as_ref(), Mat2D::IDENTITY);
    assert_eq!(
        deferred(destination.as_ref()).query_raw_path(),
        Some(&expected)
    );

    let mut non_deferred_factory = NullFactory::new();
    let non_deferred_source = non_deferred_factory
        .make_render_path(line_path((60.0, 70.0), (80.0, 90.0)), FillRule::NonZero);
    destination.add_render_path(non_deferred_source.as_ref(), Mat2D::IDENTITY);
    assert_eq!(
        deferred(destination.as_ref()).query_raw_path(),
        Some(&expected)
    );
}

#[test]
fn self_append_records_the_same_id_and_appends_a_frozen_query_copy() {
    let mut session = DeferredSession::new(None);
    let mut path = session.make_empty_render_path();
    deferred_mut(path.as_mut()).retain_query_geometry(None);
    path.move_to(1.0, 2.0);
    path.line_to(3.0, 4.0);

    let transform = Mat2D([2.0, 0.0, 0.0, 3.0, 5.0, 7.0]);
    path.add_render_path_self(transform);

    let original = line_path((1.0, 2.0), (3.0, 4.0));
    let mut expected = original.clone();
    expected.add_path_with_transform(&original, transform);
    assert_eq!(deferred(path.as_ref()).query_raw_path(), Some(&expected));

    let commands = session.command_buffer();
    let commands = commands.lock().unwrap();
    let mut reader = CommandReader::new(commands.command_bytes(), commands.blob_bytes());
    assert_eq!(reader.next_u8(), Some(RenderCmd::MakeEmptyPath as u8));
    let _: MakeIdPod = reader.read();
    assert_eq!(reader.next_u8(), Some(RenderCmd::PathAddRawPath as u8));
    let _: PathRawPod = reader.read();
    assert_eq!(reader.next_u8(), Some(RenderCmd::PathAddRenderPath as u8));
    let append: PathAddPathPod = reader.read();
    assert_eq!(append.path, append.src);
    assert_eq!(reader.next_u8(), None);
}

fn recorded_bytes(retain_query: bool) -> (Vec<u8>, Vec<u8>) {
    let mut factory = DeferredFactory::new();
    let mut path = factory.make_empty_render_path();
    if retain_query {
        deferred_mut(path.as_mut()).retain_query_geometry(None);
    }
    path.fill_rule(FillRule::Clockwise);
    path.move_to(-1.0, -2.0);
    path.line_to(3.0, 4.0);
    path.add_raw_path(&line_path((5.0, 6.0), (7.0, 8.0)));
    path.add_render_path_self(Mat2D([2.0, 0.0, 0.0, 3.0, 11.0, 12.0]));
    path.rewind();
    path.move_to(9.0, 10.0);
    DeferredRenderPath::flush_scratch_of(path.as_ref());

    let buffer = factory.buffer.lock().unwrap();
    (
        buffer.command_bytes().to_vec(),
        buffer.blob_bytes().to_vec(),
    )
}

#[test]
fn query_sidecar_does_not_change_recorded_wire_bytes() {
    assert_eq!(recorded_bytes(false), recorded_bytes(true));
}

#[test]
fn identity_render_path_append_uses_the_non_null_cpp_map_path() {
    let mut source_geometry = RawPath::new();
    source_geometry.move_to(-0.0, -0.0);
    source_geometry.line_to(1.0, -0.0);

    let mut session = DeferredSession::new(None);
    let mut source = session.make_render_path(source_geometry.clone(), FillRule::NonZero);
    deferred_mut(source.as_mut()).retain_query_geometry(Some(&source_geometry));
    let mut destination = session.make_empty_render_path();
    deferred_mut(destination.as_mut()).retain_query_geometry(None);

    destination.add_render_path(source.as_ref(), Mat2D::IDENTITY);

    let points = deferred(destination.as_ref())
        .query_raw_path()
        .unwrap()
        .points();
    assert_eq!(points[0].x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(points[0].y.to_bits(), 0.0_f32.to_bits());
    assert_eq!(points[1].y.to_bits(), 0.0_f32.to_bits());

    source.add_render_path_self(Mat2D::IDENTITY);
    let self_points = deferred(source.as_ref()).query_raw_path().unwrap().points();
    assert_eq!(self_points[0].x.to_bits(), (-0.0_f32).to_bits());
    assert_eq!(self_points[2].x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(self_points[2].y.to_bits(), 0.0_f32.to_bits());
    assert_eq!(self_points[3].y.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn seed_and_bulk_add_leave_query_contour_bookkeeping_at_source_defaults() {
    fn assert_bulk_then_line(query: &RawPath) {
        assert_eq!(
            query.verbs(),
            &[
                nuxie_render_api::PathVerb::Move,
                nuxie_render_api::PathVerb::Line,
                nuxie_render_api::PathVerb::Move,
                nuxie_render_api::PathVerb::Line,
                nuxie_render_api::PathVerb::Close,
            ]
        );
        assert_eq!(
            query.points(),
            &[
                nuxie_render_api::Vec2D::new(4.0, 5.0),
                nuxie_render_api::Vec2D::new(6.0, 7.0),
                nuxie_render_api::Vec2D::new(4.0, 5.0),
                nuxie_render_api::Vec2D::new(8.0, 9.0),
            ]
        );
    }

    let geometry = line_path((4.0, 5.0), (6.0, 7.0));
    let mut session = DeferredSession::new(None);

    let mut seeded = session.make_render_path(geometry.clone(), FillRule::NonZero);
    deferred_mut(seeded.as_mut()).retain_query_geometry(Some(&geometry));
    seeded.close(); // bulk construction leaves the query contour closed
    seeded.line_to(8.0, 9.0);
    seeded.close();
    assert_bulk_then_line(deferred(seeded.as_ref()).query_raw_path().unwrap());

    let mut appended = session.make_empty_render_path();
    deferred_mut(appended.as_mut()).retain_query_geometry(None);
    appended.add_raw_path(&geometry);
    appended.close(); // addPath does not copy the source contour state
    appended.line_to(8.0, 9.0);
    appended.close();
    assert_bulk_then_line(deferred(appended.as_ref()).query_raw_path().unwrap());
}
