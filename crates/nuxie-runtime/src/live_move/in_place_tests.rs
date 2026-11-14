//! An element's ends in place, by what the element is.
use crate::live_move_fixture::{PaintCase, PaintShape, css_paint_fixture_shaped};
use crate::source::nested_artboard_layout::NestedArtboardLayout;
use crate::{File, RuntimeFactoryHandle};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

/// MixedModes' slot page, played to its first frame, with A's placement
/// "v": the file, its factory and machine kept alive with it.
struct SlotPage {
    _factory: PersistentFactory<RecordingFactory>,
    _file: crate::RuntimeFileHandle,
    _root: crate::RuntimeArtboardInstanceHandle,
    machine: crate::RuntimeStateMachineInstanceHandle,
    screen: crate::RuntimeArtboardInstanceHandle,
    placement: crate::CoreHandle,
}

fn slot_page() -> SlotPage {
    let bytes = css_paint_fixture_shaped(PaintCase::MixedModes, PaintShape::SlotCopies).unwrap();
    page(&bytes, "v")
}

/// `bytes`' page, played to its first frame, with A's placement `name`.
fn page(bytes: &[u8], name: &str) -> SlotPage {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        bytes,
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
        .with_artboard(|b| b.find_handle::<NestedArtboardLayout>(name))
        .unwrap();
    SlotPage {
        _factory: factory,
        _file: file,
        _root: root,
        machine,
        screen,
        placement,
    }
}

/// A component placement's end is where layout put it, not its authored x
/// and y. MixedModes' slot page rests a's unit on A in a placement "v",
/// absolute at (20, 36), its x and y authored at 0. Layout moves the
/// placement's world, not its local matrix, so a place read from that
/// matrix would put "v" at its parent's origin. The placement draws no
/// own transform, so its end is its page matrix, and 80 by 60.
#[test]
fn a_component_placements_end_is_where_layout_put_it() {
    let SlotPage {
        screen, placement, ..
    } = slot_page();
    let end = super::end(&super::Flying::of(&placement).unwrap()).unwrap();
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

/// A component placement's place is its offset in its parent, read back
/// through the parent's world. Under a parent scaled to 0 there is no such
/// offset, so its end falls back to its local translation and the lift,
/// which can't be solved through a scale of 0, draws nothing until it can.
#[test]
fn a_placements_offset_under_a_parent_scaled_to_0_has_no_answer() {
    use crate::source::math::{mat2d::Mat2D, vec2d::Vec2D};
    let world = Mat2D::new(2.0, 0.0, 0.0, 2.0, 70.0, 60.0);
    let parent = Mat2D::new(2.0, 0.0, 0.0, 2.0, 10.0, 20.0);
    let offset = super::offset_in(parent, world).expect("an offset");
    assert_eq!((offset.x, offset.y), (30.0, 20.0));
    let flat = Mat2D::new(0.0, 0.0, 0.0, 0.0, 10.0, 20.0);
    assert!(super::offset_in(flat, Mat2D::from_translation(Vec2D::new(30.0, 40.0))).is_none());
}

/// A held placement's cover anchor is where it rests. The keyed page's
/// card on A is turned and places "unit x", a CSS artboard with its origin
/// at its center, so the pivot the turn goes about follows the root's size.
/// Held at twice its resting size, as a Layout-mode copy is laid out
/// mid-flight, its anchor is still the box it had before the hold.
#[test]
fn a_held_placements_cover_anchor_is_where_it_rests() {
    use crate::css_layout::flight_hold;
    use crate::live_move_fixture::{InPlaceFixture, KeyedKind, in_place_fixture};
    use crate::source::{artboard::Artboard, math::vec2d::Vec2D};
    let keyed = page(
        &in_place_fixture(InPlaceFixture::Keyed(KeyedKind::Placement)),
        "card",
    );
    let (screen, placement, machine) = (&keyed.screen, &keyed.placement, &keyed.machine);
    let flying = super::Flying::of(placement).unwrap();
    let root = flying.layout_node().clone();
    let board = screen.core_handle();
    let resting = super::super::follow::resting_anchor_box(&board, placement).expect("an anchor");
    let size = flying.resting_size();
    assert!(
        flight_hold::begin(&root, (size.x, size.y)),
        "the hold begins"
    );
    flight_hold::set_size(&root, (size.x * 2.0, size.y * 2.0));
    machine.advance_and_apply(0.0);
    let laid_out = root
        .with_downcast::<Artboard, _>(|root| Vec2D::new(root.layout_width(), root.layout_height()))
        .unwrap();
    assert_eq!(
        (laid_out.x, laid_out.y),
        (size.x * 2.0, size.y * 2.0),
        "held at twice its size"
    );
    let held = super::super::follow::resting_anchor_box(&board, placement).expect("an anchor");
    let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
    assert!(
        near(held.min_x, resting.min_x)
            && near(held.min_y, resting.min_y)
            && near(held.max_x, resting.max_x)
            && near(held.max_y, resting.max_y),
        "held {held:?}, at rest {resting:?}"
    );
    flight_hold::end(&root);
}

/// A placement of an artboard without its own CSS record is not admitted:
/// its root is laid out by its own layout, and its in-place page does not
/// draw as its slot page even at rest (`ElementCase::NativeRoot`: the slot
/// page draws a's root at (36, 20), the in-place page at (0, 0)).
#[test]
fn a_placement_of_a_native_layout_artboard_is_refused() {
    use crate::live_move_fixture::{ElementCase, ElementKind};
    let bytes = css_paint_fixture_shaped(
        PaintCase::Element(ElementKind::Placement, ElementCase::NativeRoot),
        PaintShape::InPlace,
    )
    .unwrap();
    let native = page(&bytes, "a");
    let error = super::Flying::of(&native.placement)
        .err()
        .expect("refused")
        .to_string();
    assert!(error.contains("without its own CSS record"), "{error}");
}
