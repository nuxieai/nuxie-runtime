//! Direct ports of every pinned `layout_participant_test.cpp` case.
//! LayoutNodeProvider queries inspect the actual translated owners.
use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    artboard_component_list::ArtboardComponentList,
    core::CoreType,
    generated::{core_registry::CoreRegistry, node_base::NodeBase},
    layout::layout_node_provider,
    layout_component::LayoutComponent,
    math::aabb::Aabb,
    shapes::{points_path::PointsPath, shape::Shape},
    solo::Solo,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

struct Fixture {
    artboard: CoreHandle,
    _file: RuntimeFileHandle,
}
impl Fixture {
    fn find<T: CoreType>(&self) -> Vec<CoreHandle> {
        self.artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<T>())
            .expect("native Artboard")
    }
    fn advance(&self, seconds: f32) {
        Artboard::advance_handle(
            &self.artboard,
            seconds,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
    }
}

fn fixture(name: &str, artboard_name: Option<&str>, advance: bool) -> Fixture {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(
        &pinned_fixture(name),
        retained,
        Some(&mut result),
        None,
        None,
    )
    .unwrap_or_else(|| panic!("{name} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let artboard = file
        .with_file(|file| match artboard_name {
            Some(name) => file.artboard_named_source(name),
            None => file.artboard(),
        })
        .expect("native source artboard");
    let fixture = Fixture {
        artboard,
        _file: file,
    };
    if advance {
        fixture.advance(0.0);
    }
    fixture
}

fn provider(owner: &CoreHandle) -> CoreHandle {
    layout_node_provider::from_component(owner).expect("actual LayoutNodeProvider")
}
fn bounds(owner: &CoreHandle) -> Aabb {
    provider(owner)
        .with_mut(|provider| {
            provider
                .as_layout_node_provider_mut()
                .expect("native layout provider")
                .layout_bounds()
        })
        .expect("live provider")
}
fn parent(owner: &CoreHandle) -> Option<CoreHandle> {
    owner
        .with(|owner| owner.component_parent_handle())
        .flatten()
}
fn collapsed(owner: &CoreHandle) -> bool {
    owner
        .with(|owner| owner.as_component().expect("Component").is_collapsed())
        .expect("live Component")
}
fn only_shape(fixture: &Fixture) -> CoreHandle {
    let shapes = fixture.find::<Shape>();
    assert_eq!(shapes.len(), 1, "upstream requires exactly one Shape");
    shapes[0].clone()
}
fn only_solo(fixture: &Fixture) -> CoreHandle {
    let solos = fixture.find::<Solo>();
    assert_eq!(solos.len(), 1);
    solos[0].clone()
}
fn active(solo: &CoreHandle) -> CoreHandle {
    solo.with_downcast::<Solo, _>(Solo::active_component)
        .flatten()
        .expect("active Solo child")
}
fn animated_participant(name: &str) -> (Fixture, CoreHandle, CoreHandle) {
    let fixture = fixture(name, None, true);
    let shape = only_shape(&fixture);
    let _ = provider(&shape);
    let container = fixture
        .find::<LayoutComponent>()
        .into_iter()
        .find(|owner| {
            !owner.is_type_of(<Artboard as CoreType>::TYPE_KEY)
                && owner
                    .with(|owner| {
                        owner
                            .as_layout_component()
                            .unwrap()
                            .style_handle()
                            .is_some()
                    })
                    .expect("live LayoutComponent")
        })
        .expect("non-artboard styled LayoutComponent");
    (fixture, shape, container)
}
fn set_width(container: &CoreHandle, value: f32) {
    assert!(CoreRegistry::set_double_handle(container,
        i32::from(nuxie_runtime::source::generated::layout_component_base::LayoutComponentBase::WIDTH_PROPERTY_KEY), value));
}
fn approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= f32::EPSILON,
        "{actual} != {expected}"
    );
}

fn pinned_approx(actual: f32, expected: f32) {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    assert!((actual - expected).abs() <= f64::from(f32::EPSILON) * 100.0 * expected.abs());
}

#[test]
fn a_fill_participant_fills_a_stack_cell_from_a_riv_file() {
    let fixture = fixture("layout/stack_participant.riv", None, true);
    let bounds = bounds(&only_shape(&fixture));
    assert_eq!(bounds.width(), 200.0);
    assert_eq!(bounds.height(), 200.0);
}

#[test]
fn a_solos_active_child_is_laid_out_through_it_from_a_riv_file() {
    let fixture = fixture("layout/solo_participant.riv", None, true);
    let solo = only_solo(&fixture);
    assert!(layout_node_provider::from_component(&solo).is_none());
    let bounds = bounds(&active(&solo));
    assert_eq!(bounds.width(), 200.0);
    assert_eq!(bounds.height(), 200.0);
}

#[test]
fn a_hug_participant_hugs_its_content_from_a_riv_file() {
    let fixture = fixture("layout/hug_participant.riv", None, true);
    let bounds = bounds(&only_shape(&fixture));
    approx(bounds.width(), 10.0);
    approx(bounds.height(), 10.0);
}

#[test]
fn a_fixed_size_participant_keeps_its_size_from_a_riv_file() {
    let fixture = fixture("layout/fixed_participant.riv", None, true);
    let bounds = bounds(&only_shape(&fixture));
    approx(bounds.width(), 60.0);
    approx(bounds.height(), 40.0);
}

#[test]
fn a_display_none_participant_collapses_and_leaves_the_flow_from_a_riv_file() {
    let fixture = fixture("layout/display_none_participant.riv", None, true);
    let shapes = fixture.find::<Shape>();
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes.iter().filter(|shape| collapsed(shape)).count(), 1);
    let shown = shapes
        .iter()
        .find(|shape| !collapsed(shape))
        .expect("one shown Shape");
    let shown = bounds(shown);
    approx(shown.width(), 200.0);
    approx(shown.height(), 200.0);
}

#[test]
fn min_max_constraints_clamp_a_participant_slot_from_a_riv_file() {
    let fixture = fixture("layout/constrained_participant.riv", None, true);
    let bounds = bounds(&only_shape(&fixture));
    approx(bounds.width(), 50.0);
    approx(bounds.height(), 30.0);
}

#[test]
fn a_solos_active_child_index_helpers_work_from_a_riv_file() {
    let fixture = fixture("layout/solo_participant.riv", None, true);
    let solo = only_solo(&fixture);
    assert_eq!(
        solo.with_downcast_mut::<Solo, _>(Solo::get_active_child_index),
        Some(0)
    );
    solo.with_downcast_mut::<Solo, _>(|solo| solo.update_by_index(1))
        .expect("Solo");
    assert_eq!(
        solo.with_downcast_mut::<Solo, _>(Solo::get_active_child_index),
        Some(1)
    );
}

#[test]
fn a_participant_animates_its_slot_under_an_animated_layout() {
    let (fixture, shape, container) = animated_participant("layout/animated_participant.riv");
    pinned_approx(bounds(&shape).width(), 200.0);
    set_width(&container, 100.0);
    let mut mid = 200.0;
    for _ in 0..5 {
        fixture.advance(0.2);
        let width = bounds(&shape).width();
        if width > 100.0 && width < 200.0 {
            mid = width;
        }
    }
    assert!(mid < 200.0);
    assert!(mid > 100.0);
    for _ in 0..3 {
        fixture.advance(1.0);
    }
    pinned_approx(bounds(&shape).width(), 100.0);
}

#[test]
fn a_participant_retargets_an_in_flight_layout_animation() {
    let (fixture, shape, container) = animated_participant("layout/animated_participant.riv");
    set_width(&container, 100.0);
    let mut mid = 200.0;
    for _ in 0..8 {
        if mid < 200.0 {
            break;
        }
        fixture.advance(0.1);
        mid = bounds(&shape).width();
    }
    assert!(mid < 200.0);
    assert!(mid > 100.0);
    set_width(&container, 50.0);
    for _ in 0..8 {
        fixture.advance(1.0);
    }
    pinned_approx(bounds(&shape).width(), 50.0);
}

#[test]
fn disabling_a_layouts_interpolation_frees_participant_animation() {
    let (fixture, shape, container) = animated_participant("layout/animated_participant.riv");
    let style = container
        .with(|owner| owner.as_layout_component().unwrap().style_handle())
        .flatten()
        .expect("container style");
    assert!(CoreRegistry::set_double_handle(&style,
        i32::from(nuxie_runtime::source::generated::layout::layout_component_style_base::LayoutComponentStyleBase::INTERPOLATION_TIME_PROPERTY_KEY), 0.0));
    fixture.advance(0.0);
    set_width(&container, 100.0);
    fixture.advance(0.016);
    approx(bounds(&shape).width(), 100.0);
}

#[test]
fn participants_size_to_grid_cells_from_a_riv_file() {
    let fixture = fixture("layout/grid_participant.riv", None, true);
    let shapes = fixture.find::<Shape>();
    assert_eq!(shapes.len(), 2);
    let mut slots: Vec<_> = shapes.iter().map(bounds).collect();
    slots.sort_by(|left, right| left.left().total_cmp(&right.left()));
    approx(slots[0].width(), 100.0);
    approx(slots[0].height(), 200.0);
    approx(slots[1].width(), 100.0);
    approx(slots[1].height(), 50.0);
}

#[test]
fn a_participant_animates_its_slot_with_a_cubic_interpolator() {
    let (fixture, shape, container) = animated_participant("layout/animated_cubic_participant.riv");
    pinned_approx(bounds(&shape).width(), 200.0);
    set_width(&container, 100.0);
    let mut mid = 200.0;
    for _ in 0..8 {
        fixture.advance(0.15);
        let width = bounds(&shape).width();
        if width > 100.0 && width < 200.0 {
            mid = width;
        }
    }
    assert!(mid < 200.0);
    assert!(mid > 100.0);
    for _ in 0..3 {
        fixture.advance(1.0);
    }
    pinned_approx(bounds(&shape).width(), 100.0);
}

#[test]
fn a_participant_retargets_a_cubic_animation_while_smoothing() {
    let (fixture, shape, container) = animated_participant("layout/animated_cubic_participant.riv");
    set_width(&container, 100.0);
    let mut current = 200.0;
    for _ in 0..8 {
        if current < 200.0 {
            break;
        }
        fixture.advance(0.1);
        current = bounds(&shape).width();
    }
    assert!(current < 200.0);
    set_width(&container, 80.0);
    fixture.advance(0.1);
    set_width(&container, 50.0);
    for _ in 0..20 {
        fixture.advance(1.0);
    }
    approx(bounds(&shape).width(), 50.0);
}

#[test]
fn a_participant_inside_a_group_is_laid_out_through_it() {
    let fixture = fixture("layout/group_participant.riv", None, true);
    let shape = only_shape(&fixture);
    assert!(layout_node_provider::from_component(&parent(&shape).expect("group")).is_none());
    let bounds = bounds(&shape);
    assert_eq!(bounds.width(), 200.0);
    assert_eq!(bounds.height(), 200.0);
}

#[test]
fn participants_nested_in_groups_and_in_a_grouped_solo_are_laid_out() {
    let fixture = fixture("layout/nested_group_participant.riv", None, true);
    let solo = only_solo(&fixture);
    let active = active(&solo);
    let active_bounds = bounds(&active);
    assert_eq!(active_bounds.width(), 200.0);
    assert_eq!(active_bounds.height(), 200.0);
    let all_shapes = fixture.find::<Shape>();
    assert_eq!(all_shapes.len(), 3);
    let inactive = all_shapes
        .iter()
        .find(|shape| **shape != active && parent(shape).as_ref() == Some(&solo))
        .expect("inactive grouped Solo child");
    assert!(collapsed(inactive));
    // Unlike the removed facade's optional bounds, the pinned provider still
    // exists for this child. Its canonical slot is not sized by the stack.
    assert_ne!(bounds(inactive).width(), 200.0);
    let container = fixture
        .find::<LayoutComponent>()
        .into_iter()
        .find(|owner| !owner.is_type_of(<Artboard as CoreType>::TYPE_KEY))
        .expect("stack");
    let layout_members = LayoutComponent::layout_providers_occurrence(&container);
    assert_eq!(
        layout_members.len(),
        2,
        "inactive Solo sibling is excluded from the solve"
    );
    assert!(
        !layout_members
            .iter()
            .any(|(_, member)| *member == provider(inactive))
    );
    let deep = all_shapes
        .iter()
        .find(|shape| parent(shape).as_ref() != Some(&solo))
        .expect("participant two groups deep");
    let deep_bounds = bounds(deep);
    assert_eq!(deep_bounds.width(), 200.0);
    assert_eq!(deep_bounds.height(), 200.0);
}

#[test]
fn an_artboard_component_list_inside_a_group_stays_out_of_the_layout() {
    let fixture = fixture("clipping_and_draw_order.riv", None, true);
    let lists = fixture.find::<ArtboardComponentList>();
    assert_eq!(lists.len(), 1);
    assert_eq!(
        parent(&lists[0]).expect("group").core_type(),
        Some(NodeBase::TYPE_KEY)
    );
    assert!(
        fixture
            .artboard
            .with(|owner| owner.as_layout_component().unwrap().is_leaf())
            .unwrap()
    );
}

#[test]
fn a_flagged_artboard_component_list_joins_the_layout_through_a_group() {
    let fixture = fixture("layout/list_in_group_joins_layout.riv", None, true);
    let lists = fixture.find::<ArtboardComponentList>();
    assert_eq!(lists.len(), 1);
    assert_eq!(
        parent(&lists[0]).expect("group").core_type(),
        Some(NodeBase::TYPE_KEY)
    );
    assert!(
        !fixture
            .artboard
            .with(|owner| owner.as_layout_component().unwrap().is_leaf())
            .unwrap()
    );
}

#[test]
fn a_custom_path_participant_measures_before_its_paths_are_built() {
    let fixture = fixture(
        "layout_grid_stack.riv",
        Some("GridWithLayoutParticipants"),
        false,
    );
    let shapes = fixture.find::<Shape>();
    assert!(!shapes.is_empty());
    let mut custom_path_shapes = 0;
    for shape in shapes {
        let (intrinsic, paths) = shape
            .with_downcast::<Shape, _>(|shape| (shape.compute_intrinsic_bounds(), shape.paths()))
            .expect("Shape");
        assert!(intrinsic.width() >= 0.0);
        assert!(intrinsic.height() >= 0.0);
        if paths
            .iter()
            .any(|path| path.is_type_of(<PointsPath as CoreType>::TYPE_KEY) && !collapsed(path))
        {
            custom_path_shapes += 1;
            assert!(intrinsic.width() > 0.0);
            assert!(intrinsic.height() > 0.0);
        }
    }
    assert!(custom_path_shapes > 0);
}

#[test]
fn a_participant_with_an_empty_path_keeps_a_sane_world_transform() {
    let fixture = fixture(
        "layout_grid_stack.riv",
        Some("GridWithLayoutParticipants"),
        true,
    );
    let shapes = fixture.find::<Shape>();
    assert!(!shapes.is_empty());
    for shape in shapes {
        let (intrinsic, world) = shape
            .with_downcast::<Shape, _>(|shape| {
                (
                    shape.compute_intrinsic_bounds(),
                    *shape.base.world_transform(),
                )
            })
            .expect("Shape");
        assert!(intrinsic.width() >= 0.0);
        assert!(intrinsic.height() >= 0.0);
        assert!(world[4].abs() < 1.0e6);
        assert!(world[5].abs() < 1.0e6);
    }
}
