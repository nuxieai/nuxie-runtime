//! An element's ends in place, by what the element is.
use crate::live_move_fixture::{PaintCase, PaintShape, css_paint_fixture_shaped};
use crate::source::nested_artboard_layout::NestedArtboardLayout;
use crate::{File, RuntimeFactoryHandle};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

/// A component placement's end is where layout put it, not its authored x
/// and y. MixedModes' slot page rests a's unit on A in a placement "v",
/// absolute at (20, 36), its x and y authored at 0. Layout moves the
/// placement's world, not its local matrix, so a place read from that
/// matrix would put "v" at its parent's origin. The placement draws no
/// own transform, so its end is its page matrix, and 80 by 60.
#[test]
fn a_component_placements_end_is_where_layout_put_it() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let bytes = css_paint_fixture_shaped(PaintCase::MixedModes, PaintShape::SlotCopies).unwrap();
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let screen = crate::live_move::screens(&root).unwrap()["A"].clone();
    let placement = screen
        .with_artboard(|b| b.find_handle::<NestedArtboardLayout>("v"))
        .unwrap();
    let end = super::end(&placement).unwrap();
    let (page, _, _) = super::draw_pose(&placement).unwrap();
    let (screen_page, _, _) = super::draw_pose(&screen.core_handle()).unwrap();
    let at = |m: crate::source::math::mat2d::Mat2D| (m.values()[4], m.values()[5]);
    let near = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3;
    assert!(
        near(at(end.place), at(page)),
        "the end {:?} is the placement's page place {:?}",
        at(end.place),
        at(page)
    );
    let from_screen = (
        at(page).0 - at(screen_page).0,
        at(page).1 - at(screen_page).1,
    );
    assert!(
        near(from_screen, (20.0, 36.0)),
        "at (20, 36) on A: {from_screen:?}"
    );
    assert_eq!((end.size.x, end.size.y), (80.0, 60.0));
}
