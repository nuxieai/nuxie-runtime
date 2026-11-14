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

/// A component placement flying in Scale mode stretches inside its placed
/// root's own transform, as a flying unit's root does in a flight slot: so
/// with the root turned, an uneven stretch runs along the root's own axes.
/// "v"'s root, turned by 0.35 and stretched by (1.5, 0.75), draws its own
/// transform with the stretch innermost: its linear part is the unstretched
/// one's times the stretch, not the stretch times the turn.
#[test]
fn a_placement_stretches_inside_its_roots_own_transform() {
    use crate::source::{
        generated::core_registry::CoreRegistry,
        math::{mat2d::Mat2D, vec2d::Vec2D},
    };
    let page = slot_page();
    let root = page
        .placement
        .with(|object| {
            object
                .as_nested_artboard()
                .and_then(|nested| nested.artboard_instance_default())
        })
        .flatten()
        .expect("v's root");
    assert!(CoreRegistry::set_double_handle(
        &root.core_handle(),
        15,
        0.35
    ));
    page.machine.advance_and_apply(0.0);
    let stretch = Vec2D::new(1.5, 0.75);
    let (own, stretched) =
        super::own_transforms(&page.placement, Vec2D::new(120.0, 45.0), stretch).unwrap();
    let linear = |m: Mat2D| {
        let v = m.values();
        [v[0], v[1], v[2], v[3]]
    };
    let mut expected = own;
    expected.scale_by_values(stretch.x, stretch.y);
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-4);
    assert!(
        near(linear(stretched), linear(expected)),
        "the stretch innermost: {:?} against {:?}",
        linear(stretched),
        linear(expected)
    );
    let turn = linear(own);
    assert!(
        turn[1].abs() > 0.1,
        "the root's turn is in its own transform: {turn:?}"
    );
    let outside = Mat2D::from_scale(stretch.x, stretch.y) * own;
    assert!(
        !near(linear(stretched), linear(outside)),
        "not the stretch outside the turn"
    );
}
