//! Hosted CSS roots use live whole-box styles. Expected geometry matches Chrome.
use super::layout_measured_content_tests::Riv;
use crate::css_layout::keys::*;
use crate::source::{
    artboard::Artboard,
    artboard_component_list::ArtboardComponentList,
    generated::{
        artboard_base::ArtboardBase, artboard_component_list_base::ArtboardComponentListBase,
        backboard_base::BackboardBase, component_base::ComponentBase, core_registry::CoreRegistry,
        layout::layout_component_style_base::LayoutComponentStyleBase,
        layout_component_base::LayoutComponentBase, nested_artboard_base::NestedArtboardBase,
        nested_artboard_layout_base::NestedArtboardLayoutBase,
        nested_artboard_leaf_base::NestedArtboardLeafBase,
    },
    layout_component::LayoutComponent,
    math::mat2d::Mat2D,
    nested_artboard_layout::NestedArtboardLayout,
    nested_artboard_leaf::NestedArtboardLeaf,
    viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem,
};
use crate::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

const WIDTH: u16 = WIDTH_KIND + 1;
const WIDTH_PERCENT: u16 = WIDTH_KIND + 2;
const HEIGHT: u16 = HEIGHT_KIND + 1;
const MIN_HEIGHT: u16 = MIN_HEIGHT_KIND + 1;
const MARGIN_TOP: u16 = MARGIN_TOP_KIND + 1;
const PADDING_RIGHT: u16 = PADDING_TOP + 2;
const PADDING_BOTTOM: u16 = PADDING_TOP + 4;
const PADDING_LEFT: u16 = PADDING_TOP + 6;

struct Scene {
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    _factory: PersistentFactory<RecordingFactory>,
}
impl Scene {
    fn import(riv: Riv) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &riv.bytes(),
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            None,
        )
        .expect("hosted CSS fixture imports");
        let artboard = file
            .with_file(File::artboard_default)
            .expect("outer artboard");
        Self {
            file,
            artboard,
            _factory: factory,
        }
    }
    fn advance(&self) {
        for _ in 0..3 {
            self.artboard.advance_default(1.0 / 60.0);
        }
    }
}

fn board(riv: &mut Riv, width: f32, height: f32, style: u64) {
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, width)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, height)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, style)
        .end();
}
fn style(riv: &mut Riv, parent: u64) {
    riv.object(LayoutComponentStyleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
        .end();
}

fn list_scene(row_css: bool, percent_width: bool) -> Scene {
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    // Outer artboard 0, column record 1, list 2, style 3.
    board(&mut riv, 300.0, 300.0, 3);
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 2)
        .float(ROW_GAP, 12.0)
        .end();
    riv.object(ArtboardComponentListBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    style(&mut riv, 0);
    // Row artboard 0, optional record 1, style follows.
    board(&mut riv, 200.0, 46.0, if row_css { 2 } else { 1 });
    if row_css {
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(BOX_SIZING, 1)
            .uint(MIN_HEIGHT_KIND, 1)
            .float(MIN_HEIGHT, 80.0)
            .float(PADDING_TOP, 14.0)
            .float(PADDING_RIGHT, 14.0)
            .float(PADDING_BOTTOM, 14.0)
            .float(PADDING_LEFT, 14.0);
        if percent_width {
            riv.uint(WIDTH_KIND, 7).float(WIDTH_PERCENT, 50.0);
        }
        riv.end();
    }
    style(&mut riv, 0);
    let scene = Scene::import(riv);
    let list = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<ArtboardComponentList>()[0].clone());
    // Exercise the list's mount path without requiring a data-binding fixture.
    let arena = list.retain_arena().expect("list arena");
    let items: Vec<_> = (0..3)
        .map(|_| arena.insert(ViewModelInstanceListItem::default()))
        .collect();
    ArtboardComponentList::update_list_occurrence(&list, &items);
    let source = scene
        .file
        .with_file(|file| file.artboard_at_source(1))
        .expect("row source");
    for index in 0..3 {
        let row = Artboard::nested_instance_from_handle(&source).expect("row instance");
        ArtboardComponentList::add_artboard_at_occurrence(&list, row, index, true);
    }
    scene.advance();
    scene
}

fn assert_rows(scene: &Scene, width: f32, height: f32, stride: f32) {
    let list = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<ArtboardComponentList>()[0].clone());
    for index in 0..3 {
        let row = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index))
            .flatten()
            .expect("mounted row");
        let bounds = row.with_artboard(|row| row.layout_bounds());
        assert_eq!(
            (bounds.width(), bounds.height()),
            (width, height),
            "row {index}"
        );
        assert_eq!(
            row.with_artboard(|row| row.layout_y()),
            index as f32 * stride,
            "row {index}"
        );
    }
}

#[test]
fn css_list_rows_use_their_minimum_and_padding() {
    assert_rows(&list_scene(true, false), 300.0, 80.0, 92.0);
}

#[test]
fn css_list_rows_resolve_percentage_width_against_the_column() {
    assert_rows(&list_scene(true, true), 150.0, 80.0, 92.0);
}

#[test]
fn css_list_rows_without_records_keep_their_stored_sizes() {
    assert_rows(&list_scene(false, false), 200.0, 46.0, 58.0);
}

fn nested_scene(host_css: bool, root_css: bool) -> Scene {
    nested_copies_scene(host_css, root_css, 1)
}

fn nested_copies_scene(host_css: bool, root_css: bool, copies: u64) -> Scene {
    nested_display_scene(host_css, root_css, copies, false)
}

fn nested_display_scene(host_css: bool, root_css: bool, copies: u64, hidden: bool) -> Scene {
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    // The outer style follows all host and CSS record pairs.
    board(
        &mut riv,
        300.0,
        300.0,
        2 + copies * if host_css { 2 } else { 1 },
    );
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 2)
        .end();
    for index in 0..copies {
        let host = 2 + index * if host_css { 2 } else { 1 };
        riv.object(NestedArtboardLayoutBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
            .end();
        if host_css {
            riv.object(LAYOUT_CSS_STYLE)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
                .uint(DISPLAY, if hidden && index == 0 { 3 } else { 0 })
                .uint(WIDTH_KIND, 7)
                .float(WIDTH_PERCENT, 50.0)
                .float(MARGIN_TOP, 10.0)
                .uint(MIN_HEIGHT_KIND, 1)
                .float(MIN_HEIGHT, 40.0)
                .end();
        }
    }
    style(&mut riv, 0);
    // 0 component, optional CSS, child, child CSS, root style, child style.
    let child = if root_css { 2 } else { 1 };
    board(&mut riv, 200.0, 100.0, child + 2);
    if root_css {
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(MIN_HEIGHT_KIND, 1)
            .float(MIN_HEIGHT, 40.0)
            .end();
    }
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, child + 3)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, child)
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 10.0)
        .uint(HEIGHT_KIND, 1)
        .float(HEIGHT, 10.0)
        .end();
    style(&mut riv, 0);
    style(&mut riv, child);
    let scene = Scene::import(riv);
    scene.advance();
    scene
}

#[test]
fn css_nested_copy_uses_its_record_for_item_properties() {
    let scene = nested_scene(true, true);
    let host = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>()[0].clone());
    let bounds = host
        .with_downcast::<NestedArtboardLayout, _>(NestedArtboardLayout::layout_bounds)
        .unwrap();
    assert_eq!((bounds.width(), bounds.height()), (150.0, 40.0));
    let row = host
        .with_downcast::<NestedArtboardLayout, _>(|host| host.base.base.artboard_instance_handle(0))
        .flatten()
        .unwrap();
    assert_eq!(row.with_artboard(|row| row.layout_y()), 10.0);
}

#[test]
fn css_nested_copy_without_either_record_keeps_instance_sizing() {
    for (host_css, root_css) in [(false, true), (true, false), (false, false)] {
        let scene = nested_scene(host_css, root_css);
        let bounds = scene
            .artboard
            .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>()[0].clone())
            .with_downcast::<NestedArtboardLayout, _>(NestedArtboardLayout::layout_bounds)
            .unwrap();
        assert_eq!((bounds.width(), bounds.height()), (200.0, 100.0));
    }
}

#[test]
fn css_nested_copies_overlap_in_a_translated_stack_like_translated_copies() {
    for css in [false, true] {
        let mut riv = Riv::default();
        riv.object(BackboardBase::TYPE_KEY).end();
        // The root's stack style follows two host and optional CSS record pairs.
        board(&mut riv, 300.0, 300.0, if css { 5 } else { 3 });
        for index in 0..2 {
            let host = 1 + index * if css { 2 } else { 1 };
            riv.object(NestedArtboardLayoutBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
                .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
                .end();
            if css {
                riv.object(LAYOUT_CSS_STYLE)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
                    .uint(WIDTH_KIND, 1)
                    .float(WIDTH, 200.0)
                    .uint(HEIGHT_KIND, 1)
                    .float(HEIGHT, 100.0)
                    .end();
            }
        }
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(LayoutComponentStyleBase::LAYOUT_TYPE_VALUE_PROPERTY_KEY, 2)
            .end();
        board(&mut riv, 200.0, 100.0, if css { 2 } else { 1 });
        if css {
            riv.object(LAYOUT_CSS_STYLE)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
                .end();
        }
        style(&mut riv, 0);
        let scene = Scene::import(riv);
        scene.advance();
        let hosts = scene
            .artboard
            .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>());
        assert_eq!(hosts.len(), 2);
        for (index, host) in hosts.iter().enumerate() {
            let row = host
                .with_downcast::<NestedArtboardLayout, _>(|host| {
                    host.base.base.artboard_instance_handle(0)
                })
                .flatten()
                .expect("stack copy");
            let geometry = row.with_artboard(|row| {
                let bounds = row.layout_bounds();
                (
                    row.layout_x(),
                    row.layout_y(),
                    bounds.width(),
                    bounds.height(),
                )
            });
            assert_eq!(
                geometry,
                (0.0, 0.0, 200.0, 100.0),
                "copy {index}, CSS {css}"
            );
        }
    }
}

fn leaf_scene(
    css: bool,
    fit_to_layout_parent: bool,
    padding: [f32; 4],
    border: f32,
    layout_sibling: bool,
) -> Scene {
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    // 0 outer artboard, 1 layout, 2 leaf, optional CSS and layout sibling, then styles.
    let sibling_id = 3 + u64::from(css);
    let first_style = sibling_id + u64::from(layout_sibling);
    board(&mut riv, 300.0, 300.0, first_style);
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Leaf sizing box")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, first_style + 1)
        .end();
    riv.object(NestedArtboardLeafBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
        .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
        .uint(NestedArtboardLeafBase::FIT_PROPERTY_KEY, 1)
        .bool(
            NestedArtboardLeafBase::FIT_TO_LAYOUT_PARENT_PROPERTY_KEY,
            fit_to_layout_parent,
        )
        .end();
    if css {
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
            .uint(BOX_SIZING, 1)
            .uint(WIDTH_KIND, 1)
            .float(WIDTH, 200.0)
            .uint(HEIGHT_KIND, 1)
            .float(HEIGHT, 100.0)
            .float(PADDING_TOP, padding[0])
            .float(PADDING_RIGHT, padding[1])
            .float(PADDING_BOTTOM, padding[2])
            .float(PADDING_LEFT, padding[3])
            .float(BORDER_TOP, border)
            .float(BORDER_TOP + 1, border)
            .float(BORDER_TOP + 2, border)
            .float(BORDER_TOP + 3, border)
            .end();
    }
    if layout_sibling {
        riv.object(LayoutComponentBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 1.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 1.0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, first_style + 2)
            .end();
    }
    style(&mut riv, 0);
    riv.object(LayoutComponentStyleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
        .float(LayoutComponentStyleBase::PADDING_TOP_PROPERTY_KEY, 10.0)
        .float(LayoutComponentStyleBase::PADDING_RIGHT_PROPERTY_KEY, 10.0)
        .float(LayoutComponentStyleBase::PADDING_BOTTOM_PROPERTY_KEY, 10.0)
        .float(LayoutComponentStyleBase::PADDING_LEFT_PROPERTY_KEY, 10.0)
        .end();
    if layout_sibling {
        style(&mut riv, sibling_id);
    }
    board(&mut riv, 100.0, 100.0, 1);
    style(&mut riv, 0);
    let scene = Scene::import(riv);
    scene.advance();
    scene
}

fn scene_leaf_transform(scene: &Scene) -> Mat2D {
    scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedArtboardLeaf>()[0].clone())
        .with(|leaf| {
            leaf.as_world_transform_component()
                .unwrap()
                .world_transform()
        })
        .unwrap()
}

fn leaf_transform(css: bool, fit_to_layout_parent: bool) -> Mat2D {
    scene_leaf_transform(&leaf_scene(
        css,
        fit_to_layout_parent,
        [10.0; 4],
        0.0,
        false,
    ))
}

#[test]
fn css_nested_leaf_contains_in_the_content_box() {
    for reach_parent in [false, true] {
        let world = leaf_transform(true, reach_parent);
        assert_eq!(
            (world[0], world[3], world[4], world[5]),
            (0.8, 0.8, 60.0, 10.0)
        );
    }
}

#[test]
fn css_block_nested_leaf_excludes_border_and_padding_from_fit() {
    let scene = leaf_scene(true, false, [10.0; 4], 2.0, false);
    let world = scene_leaf_transform(&scene);
    assert_eq!(
        (world[0], world[3], world[4], world[5]),
        (0.76, 0.76, 62.0, 12.0)
    );
}

#[test]
fn css_nested_leaf_refits_when_only_trailing_padding_changes() {
    for (key, final_padding, expected) in [
        (
            PADDING_RIGHT,
            [10.0, 110.0, 10.0, 10.0],
            (0.8, 0.8, 10.0, 10.0),
        ),
        (
            PADDING_BOTTOM,
            [10.0, 10.0, 40.0, 10.0],
            (0.5, 0.5, 75.0, 10.0),
        ),
    ] {
        let scene = leaf_scene(true, false, [10.0; 4], 0.0, false);
        let before = scene_leaf_transform(&scene);
        let layout = scene.artboard.with_artboard(|board| {
            board
                .find_handle::<LayoutComponent>("Leaf sizing box")
                .expect("sizing box")
        });
        let record = layout
            .with_downcast::<LayoutComponent, _>(LayoutComponent::css_record)
            .flatten()
            .expect("sizing box CSS");
        let final_value = if key == PADDING_RIGHT { 110.0 } else { 40.0 };
        CoreRegistry::set_double_handle(&record, key.into(), final_value);
        scene.advance();
        let world = scene_leaf_transform(&scene);
        let fresh = scene_leaf_transform(&leaf_scene(true, false, final_padding, 0.0, false));
        let geometry = |world: Mat2D| (world[0], world[3], world[4], world[5]);
        assert_eq!(geometry(world), expected, "padding key {key}");
        assert_eq!(
            geometry(world),
            geometry(fresh),
            "fresh import, padding key {key}"
        );
        assert_ne!(
            geometry(world),
            geometry(before),
            "fit changed, padding key {key}"
        );
    }
}

#[test]
fn css_nested_leaf_with_a_layout_sibling_fits_at_the_content_origin() {
    for reach_parent in [false, true] {
        for (border, expected) in [
            (0.0, (0.8, 0.8, 60.0, 10.0)),
            (2.0, (0.76, 0.76, 62.0, 12.0)),
        ] {
            let scene = leaf_scene(true, reach_parent, [10.0; 4], border, true);
            let world = scene_leaf_transform(&scene);
            assert_eq!((world[0], world[3], world[4], world[5]), expected);
        }
    }
}

#[test]
fn translated_nested_leaf_contains_in_the_whole_box() {
    let world = leaf_transform(false, false);
    assert_eq!(
        (world[0], world[3], world[4], world[5]),
        (1.0, 1.0, 50.0, 0.0)
    );
}

#[test]
fn css_nested_copies_keep_independent_live_item_styles() {
    let scene = nested_copies_scene(true, true, 2);
    let hosts = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>());
    let record = hosts[0]
        .with_downcast::<NestedArtboardLayout, _>(|host| {
            crate::css_layout::record::find_in(host.base.base.children())
        })
        .flatten()
        .expect("copy item record");
    let bounds = |index: usize| {
        hosts[index]
            .with_downcast::<NestedArtboardLayout, _>(NestedArtboardLayout::layout_bounds)
            .unwrap()
    };
    assert_eq!(bounds(0).width(), 150.0);
    assert_eq!(bounds(1).width(), 150.0);
    CoreRegistry::set_double_handle(&record, WIDTH_PERCENT.into(), 25.0);
    CoreRegistry::set_double_handle(&record, MIN_HEIGHT.into(), 60.0);
    // Rive's instance dimensions must not replace CSS, including live writes.
    CoreRegistry::set_double_handle(
        &hosts[0],
        NestedArtboardLayoutBase::INSTANCE_WIDTH_PROPERTY_KEY.into(),
        270.0,
    );
    CoreRegistry::set_double_handle(
        &hosts[0],
        NestedArtboardLayoutBase::INSTANCE_HEIGHT_PROPERTY_KEY.into(),
        220.0,
    );
    scene.advance();
    assert_eq!((bounds(0).width(), bounds(0).height()), (75.0, 60.0));
    assert_eq!((bounds(1).width(), bounds(1).height()), (150.0, 40.0));
}

#[test]
fn css_nested_copy_applies_limits_ratio_and_absolute_insets() {
    let scene = nested_scene(true, true);
    let host = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>()[0].clone());
    let record = host
        .with_downcast::<NestedArtboardLayout, _>(|host| {
            crate::css_layout::record::find_in(host.base.base.children())
        })
        .flatten()
        .unwrap();
    CoreRegistry::set_uint_handle(&record, MAX_WIDTH_KIND.into(), 1);
    CoreRegistry::set_double_handle(&record, i32::from(MAX_WIDTH_KIND + 1), 120.0);
    CoreRegistry::set_double_handle(&record, MIN_HEIGHT.into(), 0.0);
    CoreRegistry::set_double_handle(&record, ASPECT_RATIO.into(), 2.0);
    CoreRegistry::set_uint_handle(&record, POSITION.into(), 2);
    CoreRegistry::set_double_handle(&record, MARGIN_TOP.into(), 0.0);
    CoreRegistry::set_uint_handle(&record, INSET_TOP_KIND.into(), 1);
    CoreRegistry::set_double_handle(&record, i32::from(INSET_TOP_KIND + 1), 7.0);
    CoreRegistry::set_uint_handle(&record, i32::from(INSET_TOP_KIND + 9), 1);
    CoreRegistry::set_double_handle(&record, i32::from(INSET_TOP_KIND + 10), 9.0);
    scene.advance();
    let row = host
        .with_downcast::<NestedArtboardLayout, _>(|host| host.base.base.artboard_instance_handle(0))
        .flatten()
        .unwrap();
    let geometry = row.with_artboard(|row| {
        let bounds = row.layout_bounds();
        (
            bounds.width(),
            bounds.height(),
            row.layout_x(),
            row.layout_y(),
        )
    });
    assert_eq!(geometry, (120.0, 60.0, 9.0, 7.0));
}

#[test]
fn css_row_record_on_a_top_level_artboard_keeps_the_root_size() {
    let scene = list_scene(true, true);
    let artboard = scene
        .file
        .with_file(|file| file.artboard_at(1))
        .expect("top-level row artboard");
    for _ in 0..3 {
        artboard.advance_default(1.0 / 60.0);
    }
    let bounds = artboard.with_artboard(|artboard| artboard.layout_bounds());
    assert_eq!((bounds.width(), bounds.height()), (200.0, 46.0));
}

#[test]
fn css_nested_copy_uses_flex_basis_grow_and_self_alignment() {
    let scene = nested_copies_scene(true, true, 2);
    let hosts = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>());
    let record = hosts[0]
        .with_downcast::<NestedArtboardLayout, _>(|host| {
            crate::css_layout::record::find_in(host.base.base.children())
        })
        .flatten()
        .unwrap();
    CoreRegistry::set_uint_handle(&record, FLEX_BASIS_KIND.into(), 1);
    CoreRegistry::set_double_handle(&record, i32::from(FLEX_BASIS_KIND + 1), 60.0);
    CoreRegistry::set_double_handle(&record, FLEX_GROW.into(), 1.0);
    CoreRegistry::set_uint_handle(&record, ALIGN_SELF.into(), 3);
    scene.advance();
    let row = hosts[0]
        .with_downcast::<NestedArtboardLayout, _>(|host| host.base.base.artboard_instance_handle(0))
        .flatten()
        .unwrap();
    let geometry = row.with_artboard(|row| {
        let bounds = row.layout_bounds();
        (
            bounds.width(),
            bounds.height(),
            row.layout_x(),
            row.layout_y(),
        )
    });
    assert_eq!(geometry, (150.0, 240.0, 150.0, 10.0));
}

#[test]
fn css_nested_item_display_none_removes_only_its_copy_from_flow() {
    let scene = nested_display_scene(true, true, 2, true);
    let hosts = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>());
    let row = |index: usize| {
        hosts[index]
            .with_downcast::<NestedArtboardLayout, _>(|host| {
                host.base.base.artboard_instance_handle(0)
            })
            .flatten()
            .unwrap()
    };
    assert_eq!(row(1).with_artboard(|board| board.layout_y()), 10.0);
    assert_eq!(
        row(0).with_artboard(|board| board.layout_bounds().height()),
        0.0
    );
    assert_eq!(
        row(1).with_artboard(|board| board.layout_bounds().height()),
        40.0
    );
}
// Two copies of one painted, clickable CSS component. Only the first copy
// optionally owns an item record; the second always participates in CSS sizing.
fn interactive_display_scene(item_record: bool, root_css: bool, bound: bool) -> Scene {
    use crate::source::generated::{
        animation::{
            any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
            exit_state_base::ExitStateBase, listener_fire_event_base::ListenerFireEventBase,
            listener_types::listener_input_type_base::ListenerInputTypeBase,
            nested_state_machine_base::NestedStateMachineBase,
            state_machine_base::StateMachineBase, state_machine_layer_base::StateMachineLayerBase,
            state_machine_listener_base::StateMachineListenerBase,
        },
        data_bind::{data_bind_base::DataBindBase, data_bind_context_base::DataBindContextBase},
        event_base::EventBase,
        nested_animation_base::NestedAnimationBase,
        shapes::paint::{fill_base::FillBase, solid_color_base::SolidColorBase},
        viewmodel::{
            viewmodel_base::ViewModelBase,
            viewmodel_property_number_base::ViewModelPropertyNumberBase,
        },
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    if bound {
        riv.object(ViewModelBase::TYPE_KEY).end();
        riv.object(ViewModelPropertyNumberBase::TYPE_KEY).end();
    }
    // Root 0, CSS 1, copies (host, optional record, nested machine), root style.
    let first_slots = if item_record { 3 } else { 2 };
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 300.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 300.0)
        .uint(
            LayoutComponentBase::STYLE_ID_PROPERTY_KEY,
            2 + first_slots + 3,
        );
    if bound {
        riv.uint(ArtboardBase::VIEW_MODEL_ID_PROPERTY_KEY, 0);
    }
    riv.end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 2)
        .end();
    for index in 0..2 {
        let host = if index == 0 { 2 } else { 2 + first_slots };
        riv.object(NestedArtboardLayoutBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
            .end();
        if index == 1 || item_record {
            riv.object(LAYOUT_CSS_STYLE)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
                .uint(DISPLAY, if index == 0 { 3 } else { 0 })
                .uint(WIDTH_KIND, 1)
                .float(WIDTH, 100.0)
                .uint(HEIGHT_KIND, 1)
                .float(HEIGHT, 40.0)
                .end();
            if index == 0 && bound {
                riv.object(DataBindContextBase::TYPE_KEY)
                    .uint(DataBindBase::PROPERTY_KEY_PROPERTY_KEY, DISPLAY.into())
                    .byte_field(DataBindContextBase::SOURCE_PATH_IDS_PROPERTY_KEY, &[0, 0])
                    .end();
            }
        }
        riv.object(NestedStateMachineBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
            .uint(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY, 0)
            .end();
    }
    style(&mut riv, 0);
    riv.object(StateMachineBase::TYPE_KEY).end();
    riv.object(StateMachineLayerBase::TYPE_KEY).end();
    for state in [
        EntryStateBase::TYPE_KEY,
        AnyStateBase::TYPE_KEY,
        ExitStateBase::TYPE_KEY,
    ] {
        riv.object(state).end();
    }
    // Component root, optional CSS record, fill, solid, event, style.
    let fill = if root_css { 2 } else { 1 };
    board(&mut riv, 100.0, 40.0, fill + 7);
    if root_css {
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            // The fallback container is a flex row; each copy can replace it.
            .uint(DISPLAY, 1)
            .end();
    }
    riv.object(FillBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(SolidColorBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, fill)
        .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0xff112233)
        .end();
    riv.object(EventBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .string(ComponentBase::NAME_PROPERTY_KEY, "tap")
        .end();
    for index in 0..2 {
        let child = fill + 3 + index * 2;
        riv.object(LayoutComponentBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, fill + 8 + index)
            .end();
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, child)
            .uint(WIDTH_KIND, 1)
            .float(WIDTH, 10.0)
            .uint(HEIGHT_KIND, 1)
            .float(HEIGHT, 10.0)
            .end();
    }
    style(&mut riv, 0);
    style(&mut riv, fill + 3);
    style(&mut riv, fill + 5);
    riv.object(StateMachineBase::TYPE_KEY).end();
    riv.object(StateMachineListenerBase::TYPE_KEY)
        .uint(StateMachineListenerBase::TARGET_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(ListenerInputTypeBase::TYPE_KEY)
        .uint(
            ListenerInputTypeBase::LISTENER_TYPE_VALUE_PROPERTY_KEY,
            crate::source::listener_type::ListenerType::Click as u64,
        )
        .end();
    riv.object(ListenerFireEventBase::TYPE_KEY)
        .uint(ListenerFireEventBase::EVENT_ID_PROPERTY_KEY, fill + 2)
        .end();
    riv.object(StateMachineLayerBase::TYPE_KEY).end();
    for state in [
        EntryStateBase::TYPE_KEY,
        AnyStateBase::TYPE_KEY,
        ExitStateBase::TYPE_KEY,
    ] {
        riv.object(state).end();
    }
    let scene = Scene::import(riv);
    scene.advance();
    scene
}

fn display_copies(scene: &Scene) -> Vec<RuntimeArtboardInstanceHandle> {
    scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>())
        .iter()
        .map(|host| {
            host.with_downcast::<NestedArtboardLayout, _>(|host| {
                host.base.base.artboard_instance_handle(0)
            })
            .flatten()
            .unwrap()
        })
        .collect()
}

fn display_draw_count(scene: &Scene) -> usize {
    let before = scene._factory.borrow().stream().lines().count();
    let mut renderer = scene._factory.borrow().make_renderer();
    scene.artboard.draw(&mut renderer);
    scene
        ._factory
        .borrow()
        .stream()
        .lines()
        .skip(before)
        .filter(|line| line.starts_with("drawPath") && line.contains("color=0xff112233"))
        .count()
}

// Pointer delivery goes through the host, but each nested machine reports
// the events fired by its own listeners in its own queue.
fn display_click(
    scene: &Scene,
    machine: &crate::RuntimeStateMachineInstanceHandle,
    y: f32,
) -> (bool, Vec<CoreHandle>) {
    use crate::source::{
        animation::nested_state_machine::NestedStateMachine, math::vec2d::Vec2D,
        pointer_button::PointerButton,
    };
    let nested: Vec<_> = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<NestedStateMachine>())
        .iter()
        .map(|animation| {
            animation
                .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
                .flatten()
                .expect("copy state machine")
        })
        .collect();
    let before: Vec<_> = nested
        .iter()
        .map(|machine| machine.with_instance(|machine| machine.reported_event_count()))
        .collect();
    let listens = machine.with_instance_mut(|machine| {
        let point = Vec2D::new(10.0, y);
        let listens = machine.listens_to_button_at(point, PointerButton::Primary);
        machine.pointer_down(point, 0, PointerButton::Primary);
        machine.pointer_up(point, 0, PointerButton::Primary);
        listens
    });
    let targets = nested
        .iter()
        .zip(before)
        .flat_map(|(machine, before)| {
            machine.with_instance(|machine| {
                (before..machine.reported_event_count())
                    .filter_map(|index| machine.reported_event_at(index).event)
                    .map(|event| {
                        event
                            .with(|event| event.component_artboard_handle())
                            .flatten()
                            .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    (listens, targets)
}

#[test]
fn css_nested_item_display_none_hides_paint_and_taps() {
    let scene = interactive_display_scene(true, true, false);
    let machine = scene.artboard.state_machine_at(0).unwrap();
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    scene.advance();
    assert_eq!(display_draw_count(&scene), 1);
    let copies = display_copies(&scene);
    assert_eq!(
        display_click(&scene, &machine, 10.0),
        (true, vec![copies[1].core_handle()]),
        "visible sibling receives taps"
    );
    assert_eq!(
        display_click(&scene, &machine, 50.0),
        (false, vec![]),
        "hidden copy has no tap target"
    );
}

#[test]
fn css_nested_bound_item_display_toggles_and_relayouts() {
    use crate::source::{
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
        viewmodel::viewmodel_instance::ViewModelInstance,
    };
    let scene = interactive_display_scene(true, true, true);
    let instance = scene
        .file
        .with_file(|file| {
            file.create_view_model_instance_for_artboard(scene.artboard.core_handle())
        })
        .unwrap();
    let number = instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_values()[0].clone())
        .unwrap();
    scene.artboard.bind_view_model_instance(Some(instance));
    let machine = scene.artboard.state_machine_at(0).unwrap();
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    let copies = display_copies(&scene);
    for display in [3, 0, 1, 2, 3, 0] {
        CoreRegistry::set_double_handle(
            &number,
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            display as f32,
        );
        scene.advance();
        machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        let hidden = display == 3;
        assert_eq!(
            copies[1].with_artboard(|board| board.layout_y()),
            if hidden { 0.0 } else { 40.0 },
            "display {display}"
        );
        assert_eq!(
            display_draw_count(&scene),
            if hidden { 1 } else { 2 },
            "display {display}"
        );
        assert_eq!(
            display_click(&scene, &machine, 50.0),
            (
                !hidden,
                if hidden {
                    vec![]
                } else {
                    vec![copies[1].core_handle()]
                }
            ),
            "display {display}"
        );
        assert_eq!(
            display_click(&scene, &machine, 10.0),
            (true, vec![copies[usize::from(hidden)].core_handle()]),
            "tap reaches the visible copy, display {display}"
        );
        if !hidden {
            let children =
                copies[0].with_artboard(|board| board.find_all_handles::<LayoutComponent>());
            let second = children
                .last()
                .unwrap()
                .with_downcast::<LayoutComponent, _>(|child| (child.layout_x(), child.layout_y()))
                .unwrap();
            assert_eq!(
                second,
                match display {
                    1 => (10.0, 0.0),
                    2 => (0.0, 20.0),
                    _ => (0.0, 10.0),
                },
                "the copy's display controls its container, display {display}"
            );
        }
        assert_eq!(
            copies[1].with_artboard(|board| board.layout_bounds().height()),
            40.0
        );
    }
}

#[test]
fn css_nested_display_without_item_record_keeps_existing_behavior() {
    let scene = interactive_display_scene(false, true, false);
    let copies = display_copies(&scene);
    assert_eq!(copies[1].with_artboard(|board| board.layout_y()), 40.0);
    assert_eq!(display_draw_count(&scene), 2);
    let machine = scene.artboard.state_machine_at(0).unwrap();
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    scene.advance();
    assert_eq!(
        display_click(&scene, &machine, 10.0),
        (true, vec![copies[0].core_handle()])
    );
    assert_eq!(
        display_click(&scene, &machine, 50.0),
        (true, vec![copies[1].core_handle()])
    );
}

#[test]
fn css_nested_item_display_on_a_translated_artboard_keeps_existing_behavior() {
    let scene = interactive_display_scene(true, false, false);
    let copies = display_copies(&scene);
    assert_eq!(
        copies[0].with_artboard(|board| board.layout_bounds().height()),
        40.0
    );
    assert_eq!(copies[1].with_artboard(|board| board.layout_y()), 40.0);
}

// The first copy has a whole-box record; the second uses the component's
// own column with 4-point padding and a 3-point gap.
fn whole_box_scene(padding: f32, bound_padding: bool) -> Scene {
    use crate::source::generated::{
        data_bind::{data_bind_base::DataBindBase, data_bind_context_base::DataBindContextBase},
        viewmodel::{
            viewmodel_base::ViewModelBase,
            viewmodel_property_number_base::ViewModelPropertyNumberBase,
        },
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    if bound_padding {
        riv.object(ViewModelBase::TYPE_KEY).end();
        riv.object(ViewModelPropertyNumberBase::TYPE_KEY).end();
    }
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 300.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 500.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 5);
    if bound_padding {
        riv.uint(ArtboardBase::VIEW_MODEL_ID_PROPERTY_KEY, 0);
    }
    riv.end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 2)
        .end();
    riv.object(NestedArtboardLayoutBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 0)
        .float(COLUMN_GAP, 8.0)
        .uint(BOX_SIZING, 0)
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 100.0)
        .float(PADDING_TOP, padding)
        .float(PADDING_RIGHT, padding)
        .float(PADDING_BOTTOM, padding)
        .float(PADDING_LEFT, padding)
        .end();
    if bound_padding {
        riv.object(DataBindContextBase::TYPE_KEY)
            .uint(DataBindBase::PROPERTY_KEY_PROPERTY_KEY, PADDING_TOP.into())
            .byte_field(DataBindContextBase::SOURCE_PATH_IDS_PROPERTY_KEY, &[0, 0])
            .end();
    }
    riv.object(NestedArtboardLayoutBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
        .end();
    style(&mut riv, 0);
    board(&mut riv, 200.0, 100.0, 6);
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 2)
        .float(ROW_GAP, 3.0)
        .float(PADDING_TOP, 4.0)
        .float(PADDING_RIGHT, 4.0)
        .float(PADDING_BOTTOM, 4.0)
        .float(PADDING_LEFT, 4.0)
        .end();
    for index in 0..2 {
        let child = 2 + index * 2;
        riv.object(LayoutComponentBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 7 + index)
            .end();
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, child)
            .uint(WIDTH_KIND, 1)
            .float(WIDTH, 10.0)
            .uint(HEIGHT_KIND, 1)
            .float(HEIGHT, 10.0)
            .end();
    }
    style(&mut riv, 0);
    style(&mut riv, 2);
    style(&mut riv, 4);
    let scene = Scene::import(riv);
    scene.advance();
    scene
}

fn copy_children(copy: &RuntimeArtboardInstanceHandle) -> Vec<(f32, f32)> {
    copy.with_artboard(|board| board.find_all_handles::<LayoutComponent>())
        .iter()
        .filter(|child| !child.is_type_of(ArtboardBase::TYPE_KEY))
        .map(|child| {
            child
                .with_downcast::<LayoutComponent, _>(|child| (child.layout_x(), child.layout_y()))
                .unwrap()
        })
        .collect()
}

#[test]
fn css_nested_whole_box_record_controls_only_its_copies_container() {
    let scene = whole_box_scene(20.0, false);
    let copies = display_copies(&scene);
    assert_eq!(copy_children(&copies[0]), [(20.0, 20.0), (38.0, 20.0)]);
    assert_eq!(copy_children(&copies[1]), [(4.0, 4.0), (4.0, 17.0)]);
}

#[test]
fn css_nested_whole_box_content_width_adds_its_padding() {
    let scene = whole_box_scene(10.0, false);
    let copies = display_copies(&scene);
    assert_eq!(
        copies[0].with_artboard(|board| board.layout_bounds().width()),
        120.0
    );
    assert_eq!(
        copies[1].with_artboard(|board| board.layout_bounds().width()),
        200.0
    );
}

#[test]
fn css_nested_whole_box_without_copy_record_keeps_its_artboard_record() {
    let scene = whole_box_scene(20.0, false);
    let copies = display_copies(&scene);
    assert_eq!(copy_children(&copies[1]), [(4.0, 4.0), (4.0, 17.0)]);
    assert_eq!(
        copies[1].with_artboard(|board| (
            board.layout_bounds().width(),
            board.layout_bounds().height()
        )),
        (200.0, 100.0)
    );
}

#[test]
fn css_nested_whole_box_bound_padding_updates_only_its_copy() {
    use crate::source::{
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
        viewmodel::viewmodel_instance::ViewModelInstance,
    };
    let scene = whole_box_scene(20.0, true);
    let instance = scene
        .file
        .with_file(|file| {
            file.create_view_model_instance_for_artboard(scene.artboard.core_handle())
        })
        .unwrap();
    let number = instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_values()[0].clone())
        .unwrap();
    scene.artboard.bind_view_model_instance(Some(instance));
    let copies = display_copies(&scene);
    for padding in [20.0, 30.0, 0.0] {
        CoreRegistry::set_double_handle(
            &number,
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            padding,
        );
        scene.advance();
        assert_eq!(
            copy_children(&copies[0]),
            [(20.0, padding), (38.0, padding)]
        );
        assert_eq!(copy_children(&copies[1]), [(4.0, 4.0), (4.0, 17.0)]);
        assert_eq!(
            copies[1].with_artboard(|board| board.layout_y()),
            padding + 30.0
        );
    }
}

#[test]
fn css_nested_whole_box_ignores_its_replaced_artboard_record() {
    let scene = whole_box_scene(20.0, false);
    let copies = display_copies(&scene);
    let own = copies[0]
        .core_handle()
        .with(|object| {
            crate::css_layout::record::find_in(object.as_container_component().unwrap().children())
        })
        .flatten()
        .unwrap();
    CoreRegistry::set_uint_handle(&own, DISPLAY.into(), 3);
    CoreRegistry::set_uint_handle(&own, DIRECTION.into(), 2);
    CoreRegistry::set_double_handle(&own, PADDING_TOP.into(), 77.0);
    scene.advance();
    assert_eq!(copy_children(&copies[0]), [(20.0, 20.0), (38.0, 20.0)]);
    assert_eq!(
        copies[0].with_artboard(|board| (
            board.layout_bounds().width(),
            board.layout_bounds().height()
        )),
        (140.0, 50.0)
    );
    assert_eq!(copy_children(&copies[1]), [(4.0, 4.0), (4.0, 17.0)]);
}

#[test]
fn css_nested_whole_box_live_direction_border_and_overflow_use_the_copy_record() {
    let scene = whole_box_scene(20.0, false);
    let copies = display_copies(&scene);
    let record = copies[0].with_artboard(|board| board.css_record()).unwrap();
    CoreRegistry::set_uint_handle(&record, DIRECTION.into(), 2);
    CoreRegistry::set_uint_handle(&record, OVERFLOW_X.into(), 2);
    for key in BORDER_TOP..BORDER_TOP + 4 {
        CoreRegistry::set_double_handle(&record, key.into(), 5.0);
    }
    scene.advance();
    assert_eq!(copy_children(&copies[0]), [(115.0, 25.0), (97.0, 25.0)]);
    assert_eq!(
        copies[0].with_artboard(|board| board.layout_bounds().width()),
        150.0
    );
    assert!(copies[0].with_artboard(|board| board.clips()));
    assert_eq!(copy_children(&copies[1]), [(4.0, 4.0), (4.0, 17.0)]);
    assert!(!copies[1].with_artboard(|board| board.clips()));
}

fn nested_copy_container_scene(bound_key: Option<u16>, record_first: bool) -> Scene {
    use crate::source::generated::{
        data_bind::{data_bind_base::DataBindBase, data_bind_context_base::DataBindContextBase},
        viewmodel::{
            viewmodel_base::ViewModelBase,
            viewmodel_property_number_base::ViewModelPropertyNumberBase,
        },
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    if bound_key.is_some() {
        riv.object(ViewModelBase::TYPE_KEY).end();
        riv.object(ViewModelPropertyNumberBase::TYPE_KEY).end();
    }
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 300.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 300.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 4);
    if bound_key.is_some() {
        riv.uint(ArtboardBase::VIEW_MODEL_ID_PROPERTY_KEY, 0);
    }
    riv.end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 2)
        .end();
    let host = if record_first { 3 } else { 2 };
    for record in if record_first {
        [true, false]
    } else {
        [false, true]
    } {
        if record {
            riv.object(LAYOUT_CSS_STYLE)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
                .uint(
                    DISPLAY,
                    if bound_key == Some(JUSTIFY_ITEMS) {
                        2
                    } else {
                        1
                    },
                )
                .float(COLUMN_GAP, 8.0)
                .uint(WIDTH_KIND, 1)
                .float(WIDTH, 100.0)
                .uint(HEIGHT_KIND, 1)
                .float(HEIGHT, 60.0)
                .end();
            if let Some(key) = bound_key {
                riv.object(DataBindContextBase::TYPE_KEY)
                    .uint(DataBindBase::PROPERTY_KEY_PROPERTY_KEY, key.into())
                    .byte_field(DataBindContextBase::SOURCE_PATH_IDS_PROPERTY_KEY, &[0, 0])
                    .end();
            }
        } else {
            riv.object(NestedArtboardLayoutBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
                .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
                .end();
        }
    }
    style(&mut riv, 0);
    // The component itself contains two copies of a third artboard.
    board(&mut riv, 100.0, 60.0, 6);
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .end();
    for child in [2, 4] {
        riv.object(NestedArtboardLayoutBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 2)
            .end();
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, child)
            .uint(WIDTH_KIND, 1)
            .float(WIDTH, 10.0)
            .uint(HEIGHT_KIND, 1)
            .float(HEIGHT, 10.0)
            .end();
    }
    style(&mut riv, 0);
    board(&mut riv, 10.0, 10.0, 2);
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    style(&mut riv, 0);
    let scene = Scene::import(riv);
    scene.advance();
    scene
}

fn nested_copy_children(
    copy: &RuntimeArtboardInstanceHandle,
) -> Vec<RuntimeArtboardInstanceHandle> {
    copy.with_artboard(|board| board.find_all_handles::<NestedArtboardLayout>())
        .iter()
        .map(|host| {
            host.with_downcast::<NestedArtboardLayout, _>(|host| {
                host.base.base.artboard_instance_handle(0)
            })
            .flatten()
            .unwrap()
        })
        .collect()
}

fn assert_bound_nested_copy_flow(key: u16, cases: &[(f32, [(f32, f32); 2])]) {
    use crate::source::{
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
        viewmodel::viewmodel_instance::ViewModelInstance,
    };
    let scene = nested_copy_container_scene(Some(key), false);
    let model = scene
        .file
        .with_file(|file| {
            file.create_view_model_instance_for_artboard(scene.artboard.core_handle())
        })
        .unwrap();
    let number = model
        .with_downcast::<ViewModelInstance, _>(|model| model.property_values()[0].clone())
        .unwrap();
    scene.artboard.bind_view_model_instance(Some(model));
    let copy = &display_copies(&scene)[0];
    let children = nested_copy_children(copy);
    for (value, positions) in cases {
        CoreRegistry::set_double_handle(
            &number,
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            *value,
        );
        scene.advance();
        let hidden = key == DISPLAY && *value == 3.0;
        assert_eq!(
            copy.with_artboard(|board| (
                board.layout_bounds().width(),
                board.layout_bounds().height()
            )),
            if hidden { (0.0, 0.0) } else { (100.0, 60.0) }
        );
        if !hidden {
            let actual: Vec<_> = children
                .iter()
                .map(|child| child.with_artboard(|board| (board.layout_x(), board.layout_y())))
                .collect();
            assert_eq!(&actual, positions, "key {key}, value {value}");
        }
    }
}

#[test]
fn css_nested_copy_bound_display_updates_nested_children() {
    assert_bound_nested_copy_flow(
        DISPLAY,
        &[
            (1.0, [(0.0, 0.0), (18.0, 0.0)]),
            (3.0, [(0.0, 0.0); 2]),
            (1.0, [(0.0, 0.0), (18.0, 0.0)]),
        ],
    );
}

#[test]
fn css_nested_copy_bound_flex_direction_relayouts_nested_children() {
    assert_bound_nested_copy_flow(
        FLEX_DIRECTION,
        &[
            (0.0, [(0.0, 0.0), (18.0, 0.0)]),
            (2.0, [(0.0, 0.0), (0.0, 10.0)]),
            (1.0, [(90.0, 0.0), (72.0, 0.0)]),
        ],
    );
}

#[test]
fn css_nested_copy_bound_justify_items_relayouts_nested_children() {
    assert_bound_nested_copy_flow(
        JUSTIFY_ITEMS,
        &[
            (0.0, [(0.0, 0.0), (0.0, 30.0)]),
            (3.0, [(90.0, 0.0), (90.0, 30.0)]),
            (10.0, [(0.0, 0.0), (0.0, 30.0)]),
        ],
    );
}

#[test]
fn css_nested_copy_record_can_precede_its_parent_in_the_object_table() {
    let scene = nested_copy_container_scene(None, true);
    let copy = &display_copies(&scene)[0];
    let positions: Vec<_> = nested_copy_children(copy)
        .iter()
        .map(|child| child.with_artboard(|board| (board.layout_x(), board.layout_y())))
        .collect();
    assert_eq!(positions, [(0.0, 0.0), (18.0, 0.0)]);
    assert_eq!(
        copy.with_artboard(|board| (
            board.layout_bounds().width(),
            board.layout_bounds().height()
        )),
        (100.0, 60.0)
    );
}

#[test]
fn css_nested_copy_record_accepts_batched_registry_writes() {
    let scene = nested_copy_container_scene(None, false);
    let copy = &display_copies(&scene)[0];
    let record = copy.with_artboard(|board| board.css_record()).unwrap();
    record.with_mut(|object| {
        CoreRegistry::set_uint(object, FLEX_DIRECTION.into(), 2);
        CoreRegistry::set_double(object, PADDING_TOP.into(), 7.0);
    });
    scene.advance();
    let positions: Vec<_> = nested_copy_children(copy)
        .iter()
        .map(|child| child.with_artboard(|board| (board.layout_x(), board.layout_y())))
        .collect();
    assert_eq!(positions, [(0.0, 7.0), (0.0, 17.0)]);
    assert_eq!(
        copy.with_artboard(|board| (
            board.layout_bounds().width(),
            board.layout_bounds().height()
        )),
        (100.0, 67.0)
    );
}

// A compiler-generated binding row deliberately has different box declarations
// from its copy. Only the copy is an item when generatedListRow is set.
fn contents_list_scene(flagged: bool, percent: bool, copies: u64) -> Scene {
    contents_list_scene_with_virtualization(flagged, percent, copies, false)
}

fn contents_list_scene_with_virtualization(
    flagged: bool,
    percent: bool,
    copies: u64,
    virtualized: bool,
) -> Scene {
    use crate::source::generated::{
        animation::nested_state_machine_base::NestedStateMachineBase,
        constraints::{
            draggable_constraint_base::DraggableConstraintBase,
            scrolling::scroll_constraint_base::ScrollConstraintBase,
        },
        nested_animation_base::NestedAnimationBase,
        viewmodel::viewmodel_base::ViewModelBase,
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    riv.object(ViewModelBase::TYPE_KEY).end();
    board(
        &mut riv,
        300.0,
        if virtualized { 100.0 } else { 300.0 },
        if virtualized { 6 } else { 3 },
    );
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(DISPLAY, 1)
        .uint(FLEX_DIRECTION, 2)
        .float(PADDING_LEFT, 20.0)
        .float(PADDING_RIGHT, 20.0)
        .float(PADDING_TOP, 7.0)
        .float(ROW_GAP, 8.0)
        .end();
    if virtualized {
        riv.object(LayoutComponentBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 7)
            .end();
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
            .uint(DISPLAY, 1)
            .uint(FLEX_DIRECTION, 2)
            .uint(WIDTH_KIND, 7)
            .float(WIDTH_PERCENT, 100.0)
            .uint(HEIGHT_KIND, 7)
            .float(HEIGHT_KIND + 2, 100.0)
            .float(FLEX_SHRINK, 0.0)
            .float(ROW_GAP, 8.0)
            .end();
    }
    riv.object(ArtboardComponentListBase::TYPE_KEY)
        .uint(
            ComponentBase::PARENT_ID_PROPERTY_KEY,
            if virtualized { 2 } else { 0 },
        )
        .end();
    if virtualized {
        riv.object(ScrollConstraintBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
            .uint(DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY, 1)
            .bool(ScrollConstraintBase::VIRTUALIZE_PROPERTY_KEY, true)
            .end();
    }
    style(&mut riv, 0);
    if virtualized {
        style(&mut riv, 2);
    }
    contents_empty_machine(&mut riv);
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 60.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 20.0)
        .uint(ArtboardBase::VIEW_MODEL_ID_PROPERTY_KEY, 0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 2 + copies * 3)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        // Wire key kept literal so the fixture can run as a red test before
        // the runtime knows about generatedListRow.
        .uint(61106, u64::from(flagged))
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 60.0)
        .uint(HEIGHT_KIND, 1)
        .float(HEIGHT, 20.0)
        .float(PADDING_LEFT, 10.0)
        .float(PADDING_RIGHT, 10.0)
        .float(PADDING_TOP, 10.0)
        .float(PADDING_BOTTOM, 10.0)
        .end();
    for index in 0..copies {
        let host = 2 + index * 3;
        riv.object(NestedArtboardLayoutBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 2)
            .end();
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
            .uint(WIDTH_KIND, if percent { 7 } else { 1 })
            .float(
                if percent { WIDTH_PERCENT } else { WIDTH },
                if percent { 50.0 } else { 100.0 },
            )
            .uint(HEIGHT_KIND, 1)
            .float(HEIGHT, 20.0)
            .float(PADDING_LEFT, 10.0)
            .float(PADDING_RIGHT, 10.0)
            .float(PADDING_TOP, 10.0)
            .float(PADDING_BOTTOM, 10.0)
            // Native virtual scrolling estimates a long, non-shrinking list.
            .float(FLEX_SHRINK, 0.0)
            .end();
        riv.object(NestedStateMachineBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
            .uint(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY, 0)
            .end();
    }
    style(&mut riv, 0);
    contents_empty_machine(&mut riv);
    contents_interactive_component(&mut riv);
    let scene = Scene::import(riv);
    let list = contents_list(&scene);
    let arena = list.retain_arena().unwrap();
    let source = scene
        .file
        .with_file(|file| file.artboard_at_source(1))
        .unwrap();
    let items: Vec<_> = (0..if virtualized { 40 } else { 3 })
        .map(|_| {
            let model = scene
                .file
                .with_file(|file| file.create_view_model_instance_for_artboard(source.clone()))
                .unwrap();
            let mut item = ViewModelInstanceListItem::default();
            item.set_view_model_instance(Some(model));
            arena.insert(item)
        })
        .collect();
    ArtboardComponentList::update_list_occurrence(&list, &items);
    for _ in 0..4 {
        scene.advance();
    }
    scene
}

fn contents_empty_machine(riv: &mut Riv) {
    use crate::source::generated::animation::{
        any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
        exit_state_base::ExitStateBase, state_machine_base::StateMachineBase,
        state_machine_layer_base::StateMachineLayerBase,
    };
    riv.object(StateMachineBase::TYPE_KEY).end();
    riv.object(StateMachineLayerBase::TYPE_KEY).end();
    for state in [
        EntryStateBase::TYPE_KEY,
        AnyStateBase::TYPE_KEY,
        ExitStateBase::TYPE_KEY,
    ] {
        riv.object(state).end();
    }
}

fn contents_interactive_component(riv: &mut Riv) {
    use crate::source::generated::{
        animation::{
            any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
            exit_state_base::ExitStateBase, listener_fire_event_base::ListenerFireEventBase,
            listener_types::listener_input_type_base::ListenerInputTypeBase,
            state_machine_base::StateMachineBase, state_machine_layer_base::StateMachineLayerBase,
            state_machine_listener_base::StateMachineListenerBase,
        },
        event_base::EventBase,
        semantic::semantic_data_base::SemanticDataBase,
        shapes::paint::{fill_base::FillBase, solid_color_base::SolidColorBase},
    };
    board(riv, 10.0, 10.0, 6);
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(FillBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(SolidColorBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
        .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0xff112233)
        .end();
    riv.object(EventBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(SemanticDataBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(
            SemanticDataBase::ROLE_PROPERTY_KEY,
            crate::source::semantic::semantic_role::SemanticRole::Button as u64,
        )
        .string(SemanticDataBase::LABEL_PROPERTY_KEY, "copy")
        .end();
    style(riv, 0);
    riv.object(StateMachineBase::TYPE_KEY).end();
    riv.object(StateMachineListenerBase::TYPE_KEY)
        .uint(StateMachineListenerBase::TARGET_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(ListenerInputTypeBase::TYPE_KEY)
        .uint(
            ListenerInputTypeBase::LISTENER_TYPE_VALUE_PROPERTY_KEY,
            crate::source::listener_type::ListenerType::Click as u64,
        )
        .end();
    riv.object(ListenerFireEventBase::TYPE_KEY)
        .uint(ListenerFireEventBase::EVENT_ID_PROPERTY_KEY, 4)
        .end();
    riv.object(StateMachineLayerBase::TYPE_KEY).end();
    for state in [
        EntryStateBase::TYPE_KEY,
        AnyStateBase::TYPE_KEY,
        ExitStateBase::TYPE_KEY,
    ] {
        riv.object(state).end();
    }
}

fn contents_list(scene: &Scene) -> CoreHandle {
    scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<ArtboardComponentList>()[0].clone())
}
fn contents_rows(scene: &Scene) -> Vec<RuntimeArtboardInstanceHandle> {
    contents_list(scene)
        .with_downcast::<ArtboardComponentList, _>(|list| {
            (0..3).map(|i| list.artboard_instance(i).unwrap()).collect()
        })
        .unwrap()
}
fn contents_copy(row: &RuntimeArtboardInstanceHandle) -> RuntimeArtboardInstanceHandle {
    nested_copy_children(row).remove(0)
}

#[test]
fn css_list_contents_rows_use_the_copys_content_box_size_and_padding() {
    let scene = contents_list_scene(true, false, 1);
    for (i, row) in contents_rows(&scene).iter().enumerate() {
        let copy = contents_copy(row);
        assert_eq!(
            row.with_artboard(|board| board.layout_bounds()),
            crate::source::math::aabb::Aabb::new(
                20.0,
                7.0 + i as f32 * 48.0,
                140.0,
                47.0 + i as f32 * 48.0
            )
        );
        assert_eq!(
            copy.with_artboard(|board| (
                board.layout_bounds().width(),
                board.layout_bounds().height()
            )),
            (120.0, 40.0)
        );
    }
}

#[test]
fn css_list_contents_copy_percentages_use_the_lists_content_box() {
    let scene = contents_list_scene(true, true, 1);
    for row in contents_rows(&scene) {
        assert_eq!(
            row.with_artboard(|board| board.layout_bounds().width()),
            150.0
        );
        assert_eq!(
            contents_copy(&row).with_artboard(|board| board.layout_bounds().width()),
            150.0
        );
    }
}

#[test]
fn css_list_contents_unflagged_and_multiple_copy_rows_keep_their_boxes() {
    for (flag, copies) in [(false, 1), (true, 2)] {
        let scene = contents_list_scene(flag, false, copies);
        for row in contents_rows(&scene) {
            assert_eq!(
                row.with_artboard(|board| (
                    board.layout_bounds().width(),
                    board.layout_bounds().height()
                )),
                (80.0, 40.0)
            );
        }
    }
}

#[test]
fn css_list_contents_hidden_copy_collapses_the_row_and_relayouts_siblings() {
    let scene = contents_list_scene(true, false, 1);
    let rows = contents_rows(&scene);
    let record = contents_copy(&rows[0])
        .with_artboard(|board| board.css_record())
        .unwrap();
    CoreRegistry::set_uint_handle(&record, DISPLAY.into(), 3);
    scene.advance();
    assert_eq!(
        rows[0].with_artboard(|board| (
            board.layout_bounds().width(),
            board.layout_bounds().height()
        )),
        (0.0, 0.0)
    );
    assert_eq!(rows[1].with_artboard(|board| board.layout_y()), 7.0);
    CoreRegistry::set_uint_handle(&record, DISPLAY.into(), 0);
    scene.advance();
    assert_eq!(rows[1].with_artboard(|board| board.layout_y()), 55.0);
}

#[test]
fn css_list_contents_taps_and_semantics_follow_the_copy_box() {
    use crate::source::{
        animation::nested_state_machine::NestedStateMachine, math::vec2d::Vec2D,
        pointer_button::PointerButton,
    };
    let scene = contents_list_scene(true, false, 1);
    let machine = scene.artboard.state_machine_instance_handle(0).unwrap();
    machine.with_instance_mut(|machine| machine.enable_semantics());
    for _ in 0..3 {
        machine.advance_and_apply(1.0 / 60.0);
    }
    let rows = contents_rows(&scene);
    let copies: Vec<_> = rows.iter().map(contents_copy).collect();
    let nested_machines: Vec<_> = rows
        .iter()
        .map(|row| {
            let host = row
                .with_artboard(|board| board.find_all_handles::<NestedStateMachine>()[0].clone());
            host.with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
                .flatten()
                .unwrap()
        })
        .collect();
    let manager = machine
        .with_instance(|machine| machine.semantic_manager())
        .unwrap();
    let nodes = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    let mut buttons: Vec<_> = nodes.iter().filter(|node| node.label == "copy").collect();
    buttons.sort_by(|a, b| a.min_y.total_cmp(&b.min_y));
    assert_eq!(buttons.len(), 3);
    for (i, node) in buttons.iter().enumerate() {
        assert_eq!(
            (node.min_x, node.min_y, node.max_x, node.max_y),
            (20.0, 7.0 + i as f32 * 48.0, 140.0, 47.0 + i as f32 * 48.0)
        );
    }
    let click = || {
        let before: Vec<_> = nested_machines
            .iter()
            .map(|machine| machine.with_instance(|machine| machine.reported_event_count()))
            .collect();
        machine.with_instance_mut(|machine| {
            let point = Vec2D::new(25.0, 60.0);
            assert!(machine.listens_to_button_at(point, PointerButton::Primary));
            machine.pointer_down(point, 0, PointerButton::Primary);
            machine.pointer_up(point, 0, PointerButton::Primary);
        });
        nested_machines
            .iter()
            .zip(before)
            .flat_map(|(machine, before)| {
                machine.with_instance(|machine| {
                    (before..machine.reported_event_count())
                        .filter_map(|i| machine.reported_event_at(i).event)
                        .filter_map(|event| {
                            event
                                .with(|event| event.component_artboard_handle())
                                .flatten()
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(click(), vec![copies[1].core_handle()]);
    assert_eq!(display_draw_count(&scene), 3);
    let record = copies[1].with_artboard(|board| board.css_record()).unwrap();
    CoreRegistry::set_uint_handle(&record, DISPLAY.into(), 3);
    for _ in 0..3 {
        machine.advance_and_apply(1.0 / 60.0);
    }
    assert_eq!(display_draw_count(&scene), 2);
    let nodes = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    assert_eq!(nodes.iter().filter(|node| node.label == "copy").count(), 2);
    assert_eq!(click(), vec![copies[2].core_handle()]);
}

#[test]
fn css_list_contents_virtualized_rows_realize_measure_and_recycle() {
    use crate::source::{
        constraints::scrolling::scroll_constraint::ScrollConstraint,
        generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
    };
    let scene = contents_list_scene_with_virtualization(true, false, 1, true);
    let list = contents_list(&scene);
    let visible = || {
        list.with_downcast::<ArtboardComponentList, _>(|list| {
            assert!(list.virtualization_enabled());
            let mut realized = Vec::new();
            list.realized_indices(&mut realized);
            assert!(!realized.is_empty());
            assert!(
                realized.len() < 40,
                "virtualization must not realize the entire list"
            );
            let mut visible = Vec::new();
            for index in realized {
                let row = list.artboard_instance(index).unwrap();
                if list.is_within_visible_window(index) {
                    assert_eq!(
                        list.item_size(index),
                        crate::source::math::vec2d::Vec2D::new(120.0, 40.0)
                    );
                    assert_eq!(
                        row.with_artboard(|board| (
                            board.layout_bounds().width(),
                            board.layout_bounds().height()
                        )),
                        (120.0, 40.0)
                    );
                    visible.push(index);
                }
            }
            assert!(!visible.is_empty());
            visible
        })
        .unwrap()
    };
    let before = visible();
    let scroll = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<ScrollConstraint>()[0].clone());
    CoreRegistry::set_double_handle(
        &scroll,
        ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY.into(),
        -300.0,
    );
    for _ in 0..5 {
        scene.advance();
    }
    let after = visible();
    assert!(
        after[0] > before[0],
        "scrolling must realize a later set of rows"
    );
}

#[test]
fn css_list_contents_margins_flex_and_live_parent_axis_apply_to_the_copy() {
    let scene = contents_list_scene(true, false, 1);
    let rows = contents_rows(&scene);
    for row in &rows {
        let record = contents_copy(row)
            .with_artboard(|board| board.css_record())
            .unwrap();
        CoreRegistry::set_double_handle(&record, FLEX_GROW.into(), 1.0);
        CoreRegistry::set_double_handle(&record, MARGIN_TOP.into(), 3.0);
    }
    scene.advance();
    // 293 content height - 16 gaps - 9 margins = 268, split three ways.
    for row in &rows {
        assert!(
            (row.with_artboard(|board| board.layout_bounds().height()) - 268.0 / 3.0).abs() < 0.001
        );
    }
    assert_eq!(rows[0].with_artboard(|board| board.layout_y()), 10.0);
    let parent = scene
        .artboard
        .with_artboard(|board| board.css_record())
        .unwrap();
    CoreRegistry::set_uint_handle(&parent, FLEX_DIRECTION.into(), 0);
    // Fixed widths must share the parent's available inline space after the
    // parent changes from a column to a row, not the wrapper's 60px box.
    for row in &rows {
        let record = contents_copy(row)
            .with_artboard(|board| board.css_record())
            .unwrap();
        CoreRegistry::set_uint_handle(&record, WIDTH_KIND.into(), 0);
        CoreRegistry::set_double_handle(&record, MARGIN_TOP.into(), 0.0);
    }
    scene.advance();
    for row in &rows {
        assert!(
            (row.with_artboard(|board| board.layout_bounds().width()) - 260.0 / 3.0).abs() < 0.001
        );
        assert_eq!(
            row.with_artboard(|board| board.layout_bounds().height()),
            40.0
        );
    }
}
