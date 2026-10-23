//! Hosted CSS roots use live item styles. Expected geometry matches Chrome.
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
use crate::{File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle};
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
