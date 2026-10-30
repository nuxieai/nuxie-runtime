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
    /// Each decoded picture's id and pixel size, as import recorded them.
    pictures: Vec<(String, f32, f32)>,
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
        let pictures = factory
            .borrow()
            .stream()
            .lines()
            .filter_map(|line| {
                let mut fields = line.strip_prefix("decodeImage id=")?.split(' ');
                let id = fields.next()?.to_owned();
                let width = fields.next()?.strip_prefix("width=")?.parse().ok()?;
                let height = fields.next()?.strip_prefix("height=")?.parse().ok()?;
                Some((id, width, height))
            })
            .collect();
        Self {
            file,
            artboard,
            _factory: factory,
            pictures,
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

/// What gives the box, or its slot, its quarter turn.
#[derive(Clone, Copy, PartialEq, Debug)]
enum SlotTurn {
    /// Its own rotation.
    Own,
    /// A RotationConstraint copying a root node's turn; its own rotation
    /// stays 0.
    Constraint,
}

/// A root node at `node` turned a quarter turn, and a RotationConstraint on
/// `constrained` that copies its turn in world space at the default
/// strength: two objects.
fn quarter_turn_constraint(riv: &mut Riv, constrained: u64, node: u64) {
    use crate::source::generated::{
        constraints::{
            rotation_constraint_base::RotationConstraintBase,
            targeted_constraint_base::TargetedConstraintBase,
        },
        node_base::NodeBase,
        transform_component_base::TransformComponentBase,
    };
    riv.object(NodeBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .float(
            TransformComponentBase::ROTATION_PROPERTY_KEY,
            std::f32::consts::FRAC_PI_2,
        )
        .end();
    riv.object(RotationConstraintBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, constrained)
        .uint(TargetedConstraintBase::TARGET_ID_PROPERTY_KEY, node)
        .end();
}

/// A CSS Screen with a 100 by 60 box at (36, 20), turned a quarter turn
/// about its `pivot`, or a CSS slot of that size hosting a 200 by 120 CSS
/// artboard; each holds a 13 by 9 marker at (8, 5).
fn pivot_scene(hosted: bool, pivot: f32, turn: SlotTurn) -> Scene {
    use crate::source::generated::{
        component_origin_base::ComponentOriginBase,
        shapes::paint::{fill_base::FillBase, solid_color_base::SolidColorBase},
        transform_component_base::TransformComponentBase,
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    board(&mut riv, 400.0, 300.0, if hosted { 5 } else { 8 });
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end(); // 1
    riv.object(if hosted {
        NestedArtboardLayoutBase::TYPE_KEY
    } else {
        LayoutComponentBase::TYPE_KEY
    })
    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
    if turn == SlotTurn::Own {
        riv.float(
            TransformComponentBase::ROTATION_PROPERTY_KEY,
            std::f32::consts::FRAC_PI_2,
        );
    }
    if hosted {
        riv.uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1);
    } else {
        riv.uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 7);
    }
    riv.end(); // 2
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 100.0)
        .uint(HEIGHT_KIND, 1)
        .float(HEIGHT, 60.0)
        .float(MARGIN_TOP, 20.0)
        .float(MARGIN_TOP_KIND + 10, 36.0)
        .end(); // 3
    riv.object(ComponentOriginBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
        .float(ComponentOriginBase::ORIGIN_X_PROPERTY_KEY, pivot)
        .float(ComponentOriginBase::ORIGIN_Y_PROPERTY_KEY, pivot)
        .end(); // 4
    if hosted {
        style(&mut riv, 0); // 5
        if turn == SlotTurn::Constraint {
            quarter_turn_constraint(&mut riv, 2, 6); // 6, 7
        }
        board(&mut riv, 200.0, 120.0, 4);
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .end(); // 1
    }
    riv.object(FillBase::TYPE_KEY)
        .uint(
            ComponentBase::PARENT_ID_PROPERTY_KEY,
            if hosted { 0 } else { 2 },
        )
        .end();
    riv.object(SolidColorBase::TYPE_KEY)
        .uint(
            ComponentBase::PARENT_ID_PROPERTY_KEY,
            if hosted { 2 } else { 5 },
        )
        .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0xff123456)
        .end();
    style(&mut riv, if hosted { 0 } else { 2 });
    if !hosted {
        style(&mut riv, 0);
    }
    let marker = if hosted { 5 } else { 9 };
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(
            ComponentBase::PARENT_ID_PROPERTY_KEY,
            if hosted { 0 } else { 2 },
        )
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, marker + 4)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, marker)
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 13.0)
        .uint(HEIGHT_KIND, 1)
        .float(HEIGHT, 9.0)
        .float(MARGIN_TOP, 5.0)
        .float(MARGIN_TOP_KIND + 10, 8.0)
        .end();
    riv.object(FillBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, marker)
        .end();
    riv.object(SolidColorBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, marker + 2)
        .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, 0xffabcdef)
        .end();
    style(&mut riv, marker);
    if !hosted && turn == SlotTurn::Constraint {
        quarter_turn_constraint(&mut riv, 2, 14); // 14, 15
    }
    let scene = Scene::import(riv);
    scene.advance();
    scene
}

/// Compare final painted coordinates: artboard paths can bake in their origin.
fn painted_points(scene: &Scene) -> Vec<crate::source::math::vec2d::Vec2D> {
    let mut renderer = scene._factory.borrow().make_renderer();
    scene._factory.borrow_mut().clear();
    scene.artboard.draw(&mut renderer);
    let stream = scene._factory.borrow().stream();
    let mut matrix = Mat2D::identity();
    let mut stack = Vec::new();
    let mut draws = Vec::new();
    for line in stream.lines() {
        if line == "save" {
            stack.push(matrix);
        } else if line == "restore" {
            matrix = stack.pop().unwrap();
        } else if let Some(values) = line
            .strip_prefix("transform matrix=[")
            .and_then(|s| s.strip_suffix(']'))
        {
            let values: Vec<f32> = values.split(',').map(|s| s.parse().unwrap()).collect();
            let mut local = Mat2D::identity();
            for i in 0..6 {
                local[i] = values[i];
            }
            matrix *= local;
        } else if line.starts_with("drawPath ") {
            let points = line
                .split("points=[")
                .nth(1)
                .unwrap()
                .split(']')
                .next()
                .unwrap();
            let mut painted = Vec::new();
            for point in points.split("),(") {
                let (x, y) = point.trim_matches(['(', ')']).split_once(',').unwrap();
                painted.push(
                    matrix
                        * crate::source::math::vec2d::Vec2D::new(
                            x.parse().unwrap(),
                            y.parse().unwrap(),
                        ),
                );
            }
            // Root artboards and layout proxies may spell the same closed
            // rectangle with or without a repeated first vertex.
            if painted.len() > 1 && painted.first() == painted.last() {
                painted.pop();
            }
            draws.extend(painted);
        } else if let Some(image) = line.strip_prefix("drawImage image=") {
            // A picture paints its pixels' quad through the current transform.
            let id = image.split(' ').next().unwrap();
            let &(_, width, height) = scene
                .pictures
                .iter()
                .find(|(decoded, ..)| decoded == id)
                .expect("a decoded picture");
            draws.extend(
                [(0.0, 0.0), (width, 0.0), (width, height), (0.0, height)]
                    .map(|(x, y)| matrix * crate::source::math::vec2d::Vec2D::new(x, y)),
            );
        }
    }
    draws
}

fn assert_painted_like_inline(inline: &Scene, hosted: &Scene) {
    let expected = painted_points(inline);
    let actual = painted_points(hosted);
    assert!(!expected.is_empty());
    assert_eq!(actual.len(), expected.len(), "{actual:?} != {expected:?}");
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual.x - expected.x).abs() < 0.001 && (actual.y - expected.y).abs() < 0.001,
            "{actual:?} != {expected:?}"
        );
    }
}

#[test]
fn css_hosted_box_pivots_draw_like_inline_layout_boxes() {
    use crate::source::generated::{
        component_origin_base::ComponentOriginBase,
        transform_component_base::TransformComponentBase,
    };
    for pivot in [0.5, 0.0] {
        let inline = pivot_scene(false, pivot, SlotTurn::Own);
        let hosted = pivot_scene(true, pivot, SlotTurn::Own);
        assert_painted_like_inline(&inline, &hosted);
        // Native origin invalidation also follows animation/binding setters,
        // percentage pivots after resizing, and nonuniform scales.
        for scene in [&inline, &hosted] {
            let objects = scene.artboard.with_artboard(|b| b.objects().to_vec());
            for (object, key, value) in [
                (2, TransformComponentBase::SCALE_X_PROPERTY_KEY, 1.5),
                (2, TransformComponentBase::SCALE_Y_PROPERTY_KEY, 0.75),
                (3, WIDTH, 140.0),
                (3, HEIGHT, 90.0),
                (4, ComponentOriginBase::ORIGIN_X_PROPERTY_KEY, 0.25),
                (4, ComponentOriginBase::ORIGIN_Y_PROPERTY_KEY, 0.75),
            ] {
                assert!(CoreRegistry::set_double_handle(
                    objects[object].as_ref().unwrap(),
                    key.into(),
                    value
                ));
            }
            scene.artboard.advance_default(0.0);
        }
        assert_painted_like_inline(&inline, &hosted);
    }
}

/// A slot a constraint turns turns about its pivot, as an inline box turns
/// about its ComponentOrigin, which is its local anchor
/// (`Constraint::compose_keeping_anchor`). The same RotationConstraint
/// turns each a quarter turn and leaves its own rotation at 0.
#[test]
fn a_constrained_css_slot_turns_about_its_pivot_like_an_inline_box() {
    use crate::source::generated::transform_component_base::TransformComponentBase;
    for pivot in [0.5, 0.0] {
        let inline = pivot_scene(false, pivot, SlotTurn::Constraint);
        let hosted = pivot_scene(true, pivot, SlotTurn::Constraint);
        for (name, scene) in [("inline", &inline), ("hosted", &hosted)] {
            let card = scene
                .artboard
                .with_artboard(|board| board.objects()[2].clone())
                .expect("the box or its slot");
            let rotation = CoreRegistry::get_double_handle(
                &card,
                TransformComponentBase::ROTATION_PROPERTY_KEY.into(),
            )
            .expect("its rotation");
            let world = card
                .with(|card| {
                    card.as_world_transform_component()
                        .map(|card| card.world_transform())
                })
                .flatten()
                .expect("its world");
            assert!(
                rotation == 0.0 && world[0].abs() < 1e-6 && (world[1] - 1.0).abs() < 1e-6,
                "{name} {pivot}: the constraint turns it a quarter turn: {rotation}, {world:?}"
            );
        }
        assert_painted_like_inline(&inline, &hosted);
    }
}

const MARGIN_LEFT: u16 = MARGIN_TOP_KIND + 10;
const CARD_COLOR: u32 = 0xffcc_3333;
const MARKER_COLOR: u32 = 0xffab_cdef;
const LATER_COLOR: u32 = 0xff33_33cc;
const ROW_COLOR: u32 = 0xff12_3456;

/// Where the frame under test sits.
#[derive(Clone, Copy, PartialEq, Debug)]
enum FrameHost {
    /// An inline CSS box with the same values and a `ComponentOrigin`: what
    /// CSS draws, and the oracle for a frame a CSS slot places.
    Inline,
    /// A `NestedArtboardLayout` with an item record, hosting a CSS artboard.
    CssSlot,
    /// A `NestedArtboardLayout` without an item record, hosting a CSS artboard.
    RecordFreeSlot,
    /// A `NestedArtboardLayout` with an item record, hosting an artboard
    /// without a record of its own.
    NonCssFrame,
}

/// The box's own values, written in the file, and its surroundings.
#[derive(Clone, Copy)]
struct FrameSpec {
    host: FrameHost,
    rotation: f32,
    scale: (f32, f32),
    origin: (f32, f32),
    opacity: f32,
    /// A track keys the box's turn at 0, which authors a CSS transform.
    keyed_turn: bool,
    /// The slot's own opacity (the box's, inline).
    slot_opacity: f32,
    /// A later flow sibling that overlaps the box's bottom 30 points.
    later: bool,
    /// What a CSS box holds besides its marker.
    content: FrameContent,
}

impl FrameSpec {
    fn new(host: FrameHost) -> Self {
        Self {
            host,
            rotation: 0.0,
            scale: (1.0, 1.0),
            origin: (0.0, 0.0),
            opacity: 1.0,
            keyed_turn: false,
            slot_opacity: 1.0,
            later: false,
            content: FrameContent::Marker,
        }
    }
}

/// What a CSS box holds beside its marker, a layout child: each a route
/// that places a child of the box.
#[derive(Clone, Copy, PartialEq, Debug)]
enum FrameContent {
    /// Nothing more.
    Marker,
    /// A picture participant, as an `<img>` compiles.
    Picture,
    /// A filled shape participant.
    ShapeParticipant,
    /// A text participant.
    Text,
    /// A filled shape that layout does not place.
    NonLayoutShape,
    /// A group holding a picture participant and a second marker.
    Group,
    /// A component host, a `NestedArtboardLayout` with an item record
    /// hosting a small CSS artboard, a layout child of the box. Not in
    /// `ALL`.
    ComponentHost,
    /// The same component host inside a plain group in the box. Not in
    /// `ALL`.
    GroupedComponentHost,
}

impl FrameContent {
    const ALL: [Self; 6] = [
        Self::Marker,
        Self::Picture,
        Self::ShapeParticipant,
        Self::Text,
        Self::NonLayoutShape,
        Self::Group,
    ];
    fn needs_assets(self) -> bool {
        matches!(self, Self::Picture | Self::Text | Self::Group)
    }
}

/// The picture asset's pixels and declared size.
const PICTURE: (u32, u32) = (30, 16);
const SHAPE_COLOR: u32 = 0xff22_8844;
const TEXT_COLOR: u32 = 0xff44_2288;
const COMPONENT_COLOR: u32 = 0xff66_5599;

/// The file's assets: a 30 by 16 picture (asset 0) and a font (asset 1).
fn assets(riv: &mut Riv) {
    use crate::source::generated::assets::{
        drawable_asset_base::DrawableAssetBase, file_asset_base::FileAssetBase,
        file_asset_contents_base::FileAssetContentsBase, font_asset_base::FontAssetBase,
        image_asset_base::ImageAssetBase,
    };
    // A PNG signature and header carry all a recording factory decodes.
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend_from_slice(&PICTURE.0.to_be_bytes());
    png.extend_from_slice(&PICTURE.1.to_be_bytes());
    riv.object(ImageAssetBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "picture")
        .uint(FileAssetBase::ASSET_ID_PROPERTY_KEY, 1)
        .float(DrawableAssetBase::WIDTH_PROPERTY_KEY, PICTURE.0 as f32)
        .float(DrawableAssetBase::HEIGHT_PROPERTY_KEY, PICTURE.1 as f32)
        .end();
    riv.object(FileAssetContentsBase::TYPE_KEY)
        .byte_field(FileAssetContentsBase::BYTES_PROPERTY_KEY, &png)
        .end();
    riv.object(FontAssetBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "font")
        .uint(FileAssetBase::ASSET_ID_PROPERTY_KEY, 2)
        .end();
    riv.object(FileAssetContentsBase::TYPE_KEY)
        .byte_field(
            FileAssetContentsBase::BYTES_PROPERTY_KEY,
            include_bytes!("../../../../../fixtures/fonts/roboto-input.ttf"),
        )
        .end();
}

/// A participant and its host's CSS record, placed `left` and `top` into
/// the box's flow, for the host at `host`: two objects.
fn participant(riv: &mut Riv, host: u64, left: f32, top: f32) {
    riv.object(crate::source::layout::layout_participant::LayoutParticipant::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, host)
        .float(MARGIN_TOP, top)
        .float(MARGIN_LEFT, left)
        .end();
}

/// A picture participant inside `parent`, the picture at `index`: three
/// objects.
fn picture(riv: &mut Riv, parent: u64, index: u64) {
    use crate::source::generated::shapes::image_base::ImageBase;
    riv.object(ImageBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
        .uint(ImageBase::ASSET_ID_PROPERTY_KEY, 0)
        .float(ImageBase::ORIGIN_X_PROPERTY_KEY, 0.0)
        .float(ImageBase::ORIGIN_Y_PROPERTY_KEY, 0.0)
        .end();
    participant(riv, index, 40.0, 3.0);
}

/// A filled 20 by 10 rectangle shape at `index` inside `parent`, at its
/// authored `at`: four objects.
fn filled_shape(riv: &mut Riv, parent: u64, index: u64, at: (f32, f32)) {
    use crate::source::generated::{
        node_base::NodeBase,
        shapes::{
            parametric_path_base::ParametricPathBase, rectangle_base::RectangleBase,
            shape_base::ShapeBase,
        },
    };
    riv.object(ShapeBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
        .float(NodeBase::X_PROPERTY_KEY, at.0)
        .float(NodeBase::Y_PROPERTY_KEY, at.1)
        .end();
    riv.object(RectangleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, index)
        .float(ParametricPathBase::WIDTH_PROPERTY_KEY, 20.0)
        .float(ParametricPathBase::HEIGHT_PROPERTY_KEY, 10.0)
        .end();
    painted_fill(riv, index, index + 2, SHAPE_COLOR);
}

/// A component host named "Component" at `index` inside `parent`, hosting
/// the artboard at `component` in the file's artboard list, and its item
/// record: 30 by 20, 4 points down and 56 points in, below the marker in
/// the box's block flow. Two objects.
fn component_host(riv: &mut Riv, parent: u64, index: u64, component: u64) {
    riv.object(NestedArtboardLayoutBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Component")
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
        .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, component)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, index)
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 30.0)
        .uint(HEIGHT_KIND, 1)
        .float(HEIGHT, 20.0)
        .float(MARGIN_TOP, 4.0)
        .float(MARGIN_LEFT, 56.0)
        .end();
}

/// `content` inside the box `parent`, from `index` on, a component host
/// hosting the artboard at `component`; returns how many objects it wrote.
fn frame_content(
    riv: &mut Riv,
    parent: u64,
    index: u64,
    content: FrameContent,
    component: u64,
) -> u64 {
    use crate::source::generated::{
        node_base::NodeBase,
        text::{
            text_base::TextBase, text_style_base::TextStyleBase,
            text_style_paint_base::TextStylePaintBase, text_value_run_base::TextValueRunBase,
        },
    };
    match content {
        FrameContent::Marker => 0,
        FrameContent::Picture => {
            picture(riv, parent, index);
            3
        }
        FrameContent::ShapeParticipant => {
            filled_shape(riv, parent, index, (0.0, 0.0));
            participant(riv, index, 50.0, 3.0);
            6
        }
        FrameContent::Text => {
            // 0 text, 1 run, 2 its style, 3 and 4 the style's paint.
            riv.object(TextBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
                .end();
            riv.object(TextValueRunBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, index)
                .uint(TextValueRunBase::STYLE_ID_PROPERTY_KEY, index + 2)
                .string(TextValueRunBase::TEXT_PROPERTY_KEY, "Hi")
                .end();
            riv.object(TextStylePaintBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, index)
                .uint(TextStyleBase::FONT_ASSET_ID_PROPERTY_KEY, 1)
                .float(TextStyleBase::FONT_SIZE_PROPERTY_KEY, 10.0)
                .end();
            painted_fill(riv, index + 2, index + 3, TEXT_COLOR);
            participant(riv, index, 60.0, 3.0);
            7
        }
        FrameContent::NonLayoutShape => {
            filled_shape(riv, parent, index, (70.0, 40.0));
            4
        }
        FrameContent::Group => {
            riv.object(NodeBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
                .end();
            picture(riv, index, index + 1);
            marker(riv, index, index + 4);
            9
        }
        FrameContent::ComponentHost => {
            component_host(riv, parent, index, component);
            2
        }
        FrameContent::GroupedComponentHost => {
            riv.object(NodeBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
                .end();
            component_host(riv, index, index + 1, component);
            3
        }
    }
}

fn keyed_turn_at_zero(riv: &mut Riv, object: u64) {
    use crate::source::generated::animation::{
        interpolating_keyframe_base::InterpolatingKeyFrameBase, keyed_object_base::KeyedObjectBase,
        keyed_property_base::KeyedPropertyBase, keyframe_base::KeyFrameBase,
        keyframe_double_base::KeyFrameDoubleBase, linear_animation_base::LinearAnimationBase,
    };
    use crate::source::generated::transform_component_base::TransformComponentBase;
    riv.object(LinearAnimationBase::TYPE_KEY)
        .uint(LinearAnimationBase::FPS_PROPERTY_KEY, 60)
        .uint(LinearAnimationBase::DURATION_PROPERTY_KEY, 60)
        .end();
    riv.object(KeyedObjectBase::TYPE_KEY)
        .uint(KeyedObjectBase::OBJECT_ID_PROPERTY_KEY, object)
        .end();
    riv.object(KeyedPropertyBase::TYPE_KEY)
        .uint(
            KeyedPropertyBase::PROPERTY_KEY_PROPERTY_KEY,
            TransformComponentBase::ROTATION_PROPERTY_KEY.into(),
        )
        .end();
    riv.object(KeyFrameDoubleBase::TYPE_KEY)
        .uint(KeyFrameBase::FRAME_PROPERTY_KEY, 0)
        .float(KeyFrameDoubleBase::VALUE_PROPERTY_KEY, 0.0)
        .uint(
            InterpolatingKeyFrameBase::INTERPOLATION_TYPE_PROPERTY_KEY,
            1,
        )
        .end();
}

/// A Fill at `fill` on `parent`, and its SolidColor: two objects.
fn painted_fill(riv: &mut Riv, parent: u64, fill: u64, color: u32) {
    use crate::source::generated::shapes::paint::{
        fill_base::FillBase, solid_color_base::SolidColorBase,
    };
    riv.object(FillBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
        .end();
    riv.object(SolidColorBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, fill)
        .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, color)
        .end();
}

/// A 13 by 9 marker at (8, 5) inside `parent`, the box at `index`: five
/// objects.
fn marker(riv: &mut Riv, parent: u64, index: u64) {
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, index + 1)
        .end();
    style(riv, index);
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, index)
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 13.0)
        .uint(HEIGHT_KIND, 1)
        .float(HEIGHT, 9.0)
        .float(MARGIN_TOP, 5.0)
        .float(MARGIN_LEFT, 8.0)
        .end();
    painted_fill(riv, index, index + 3, MARKER_COLOR);
}

/// A CSS Screen holding a 100 by 60 box at (36, 20), as the spec places it,
/// with a 13 by 9 marker and the spec's content inside it when the box is
/// CSS, and optionally a later 200 by 100 flow sibling 30 points up into it.
fn frame_scene(spec: FrameSpec) -> Scene {
    use crate::source::generated::{
        artboard_base::ArtboardBase, component_origin_base::ComponentOriginBase,
        transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    };
    let inline = spec.host == FrameHost::Inline;
    let transform = |riv: &mut Riv, opacity: f32| {
        riv.float(TransformComponentBase::ROTATION_PROPERTY_KEY, spec.rotation)
            .float(TransformComponentBase::SCALE_X_PROPERTY_KEY, spec.scale.0)
            .float(TransformComponentBase::SCALE_Y_PROPERTY_KEY, spec.scale.1)
            .float(WorldTransformComponentBase::OPACITY_PROPERTY_KEY, opacity);
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    if spec.content.needs_assets() {
        assets(&mut riv);
    }
    // The Screen: 0 artboard, 1 record, 2 style, 3 the box or its slot.
    board(&mut riv, 400.0, 300.0, 2);
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    style(&mut riv, 0);
    let mut next = 4;
    if inline {
        riv.object(LayoutComponentBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Card")
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 5);
        transform(&mut riv, spec.opacity * spec.slot_opacity);
        riv.end(); // 3
    } else {
        riv.object(NestedArtboardLayoutBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Card")
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
            .float(
                WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
                spec.slot_opacity,
            );
        if spec.host == FrameHost::RecordFreeSlot {
            riv.float(NestedArtboardLayoutBase::INSTANCE_WIDTH_PROPERTY_KEY, 100.0)
                .float(NestedArtboardLayoutBase::INSTANCE_HEIGHT_PROPERTY_KEY, 60.0);
        }
        riv.end(); // 3
    }
    if spec.host != FrameHost::RecordFreeSlot {
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 3)
            .uint(WIDTH_KIND, 1)
            .float(WIDTH, 100.0)
            .uint(HEIGHT_KIND, 1)
            .float(HEIGHT, 60.0)
            .float(MARGIN_TOP, 20.0)
            .float(MARGIN_LEFT, 36.0)
            .end(); // 4
        next = 5;
    }
    if inline {
        style(&mut riv, 3); // 5
        riv.object(ComponentOriginBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 3)
            .float(ComponentOriginBase::ORIGIN_X_PROPERTY_KEY, spec.origin.0)
            .float(ComponentOriginBase::ORIGIN_Y_PROPERTY_KEY, spec.origin.1)
            .end(); // 6
        painted_fill(&mut riv, 3, 7, CARD_COLOR); // 7, 8
        marker(&mut riv, 3, 9); // 9 to 13
        // Screen 0, component 1.
        next = 14 + frame_content(&mut riv, 3, 14, spec.content, 1);
    }
    if spec.later {
        riv.object(LayoutComponentBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Later")
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, next + 1)
            .end();
        style(&mut riv, next);
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, next)
            .uint(WIDTH_KIND, 1)
            .float(WIDTH, 200.0)
            .uint(HEIGHT_KIND, 1)
            .float(HEIGHT, 100.0)
            .float(MARGIN_TOP, -30.0)
            .end();
        painted_fill(&mut riv, next, next + 3, LATER_COLOR);
    }
    if inline && spec.keyed_turn {
        keyed_turn_at_zero(&mut riv, 3);
    }
    if !inline {
        // The frame: 0 artboard, its record when CSS, its style, its paint.
        let css = spec.host != FrameHost::NonCssFrame;
        let style_index = if css { 2 } else { 1 };
        riv.object(ArtboardBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Unit")
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 60.0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, style_index)
            .float(ArtboardBase::ORIGIN_X_PROPERTY_KEY, spec.origin.0)
            .float(ArtboardBase::ORIGIN_Y_PROPERTY_KEY, spec.origin.1);
        transform(&mut riv, spec.opacity);
        riv.end();
        if css {
            riv.object(LAYOUT_CSS_STYLE)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
                .end();
        }
        style(&mut riv, 0);
        painted_fill(&mut riv, 0, style_index + 1, CARD_COLOR);
        if css {
            marker(&mut riv, 0, style_index + 3);
            // Screen 0, Unit 1, component 2.
            frame_content(&mut riv, 0, style_index + 8, spec.content, 2);
        }
        if spec.keyed_turn {
            keyed_turn_at_zero(&mut riv, 0);
        }
    }
    if matches!(
        spec.content,
        FrameContent::ComponentHost | FrameContent::GroupedComponentHost
    ) {
        // The component, last, as a track keys the artboard written
        // before it: 0 artboard, 1 record, 2 style, 3 and 4 its paint. Its
        // stored 64 by 48 is not the host record's 30 by 20, and its
        // origin stays (0, 0).
        board(&mut riv, 64.0, 48.0, 2);
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .end();
        style(&mut riv, 0);
        painted_fill(&mut riv, 0, 3, COMPONENT_COLOR);
    }
    let scene = Scene::import(riv);
    scene.advance();
    scene
}

/// The box, or its slot.
fn card(scene: &Scene) -> CoreHandle {
    scene
        .artboard
        .with_artboard(|board| {
            board
                .find_handle::<NestedArtboardLayout>("Card")
                .or_else(|| board.find_handle::<LayoutComponent>("Card"))
        })
        .expect("the box")
}

/// The box or the frame its slot hosts.
fn framed(scene: &Scene) -> CoreHandle {
    let card = card(scene);
    if card.is_type_of(NestedArtboardLayoutBase::TYPE_KEY) {
        card.with_downcast::<NestedArtboardLayout, _>(|host| {
            host.base.base.artboard_instance_handle(0)
        })
        .flatten()
        .expect("the hosted frame")
        .core_handle()
    } else {
        card
    }
}

fn write(object: &CoreHandle, values: &[(u16, f32)]) {
    for &(key, value) in values {
        assert!(CoreRegistry::set_double_handle(object, key.into(), value));
    }
}

/// Each drawPath's color and painted points, in draw order.
fn painted_draws(scene: &Scene) -> Vec<(u32, Vec<crate::source::math::vec2d::Vec2D>)> {
    let mut renderer = scene._factory.borrow().make_renderer();
    scene._factory.borrow_mut().clear();
    scene.artboard.draw(&mut renderer);
    let stream = scene._factory.borrow().stream();
    let mut matrix = Mat2D::identity();
    let mut stack = Vec::new();
    let mut draws = Vec::new();
    for line in stream.lines() {
        if line == "save" {
            stack.push(matrix);
        } else if line == "restore" {
            matrix = stack.pop().unwrap();
        } else if let Some(values) = line
            .strip_prefix("transform matrix=[")
            .and_then(|s| s.strip_suffix(']'))
        {
            let mut local = Mat2D::identity();
            for (i, value) in values.split(',').enumerate() {
                local[i] = value.parse().unwrap();
            }
            matrix *= local;
        } else if line.starts_with("drawPath ") {
            let color = line.split("color=0x").nth(1).unwrap();
            let color = u32::from_str_radix(&color[..8], 16).unwrap();
            let mut points = Vec::new();
            for point in line
                .split("points=[")
                .nth(1)
                .unwrap()
                .split(']')
                .next()
                .unwrap()
                .split("),(")
            {
                let (x, y) = point.trim_matches(['(', ')']).split_once(',').unwrap();
                points.push(
                    matrix
                        * crate::source::math::vec2d::Vec2D::new(
                            x.parse().unwrap(),
                            y.parse().unwrap(),
                        ),
                );
            }
            if points.len() > 1 && points.first() == points.last() {
                points.pop();
            }
            draws.push((color, points));
        }
    }
    draws
}

fn centroid(points: &[crate::source::math::vec2d::Vec2D]) -> crate::source::math::vec2d::Vec2D {
    let sum = points.iter().fold(
        crate::source::math::vec2d::Vec2D::new(0.0, 0.0),
        |sum, p| sum + *p,
    );
    crate::source::math::vec2d::Vec2D::new(sum.x / points.len() as f32, sum.y / points.len() as f32)
}

/// Whether the host's geometry query finds anything painted at `point`.
fn hits_at(scene: &Scene, point: crate::source::math::vec2d::Vec2D) -> bool {
    !crate::ArtboardInstance::from_native_handle(scene.file.clone(), 0, scene.artboard.clone())
        .hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(point.x, point.y))
        .is_empty()
}

/// Every child of a frame draws where the same child of the inline box
/// draws: its layout children, its participants (a picture, a shape, a
/// text), its content beside a layout child, and a group's children
/// (docs/CSS_LAYOUT.md, 5).
#[test]
fn css_slot_frame_slides_turns_and_scales_like_an_inline_box() {
    // Each case runs on its own, so a failure names every case it holds for.
    let mut failed = Vec::new();
    for content in FrameContent::ALL {
        for origin in [(0.5, 0.5), (0.0, 0.0), (0.25, 0.75)] {
            let case = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                frame_slides_turns_and_scales_like_an_inline_box(content, origin)
            }));
            if case.is_err() {
                failed.push((content, origin));
            }
        }
    }
    assert!(failed.is_empty(), "drawn unlike the inline box: {failed:?}");
}

fn frame_slides_turns_and_scales_like_an_inline_box(content: FrameContent, origin: (f32, f32)) {
    use crate::source::generated::node_base::NodeBase;
    let spec = |host| FrameSpec {
        rotation: 0.35,
        scale: (1.5, 0.75),
        origin,
        content,
        ..FrameSpec::new(host)
    };
    let inline = frame_scene(spec(FrameHost::Inline));
    let hosted = frame_scene(spec(FrameHost::CssSlot));
    // The frame turns about its origin with its content in its top-left
    // frame.
    assert_painted_like_inline(&inline, &hosted);
    // Tracks, bindings and scripts write the offset after load.
    for scene in [&inline, &hosted] {
        write(
            &framed(scene),
            &[
                (NodeBase::X_PROPERTY_KEY, 10.0),
                (NodeBase::Y_PROPERTY_KEY, 6.0),
            ],
        );
        scene.artboard.advance_default(0.0);
    }
    assert_painted_like_inline(&inline, &hosted);
    // Hit testing reads the same transform: the moved box is hit where
    // the unmoved box never was.
    let before = centroid(&painted_draws(&inline)[0].1);
    for scene in [&inline, &hosted] {
        write(&framed(scene), &[(NodeBase::Y_PROPERTY_KEY, 150.0)]);
        scene.artboard.advance_default(0.0);
    }
    assert_painted_like_inline(&inline, &hosted);
    let after = centroid(&painted_draws(&inline)[0].1);
    for (name, scene) in [("inline", &inline), ("hosted", &hosted)] {
        assert!(
            hits_at(scene, after),
            "{name} {content:?} {origin:?}: hit where it moved"
        );
        assert!(
            !hits_at(scene, before),
            "{name} {content:?} {origin:?}: no hit where it was"
        );
    }
}

/// The cases `run` panics for. Each runs on its own, so a failure names
/// every case it holds for.
fn failing_cases<T: Copy>(cases: impl IntoIterator<Item = T>, run: impl Fn(T)) -> Vec<T> {
    cases
        .into_iter()
        .filter(|&case| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(case))).is_err()
        })
        .collect()
}

/// A component host, a `NestedArtboardLayout` with an item record hosting
/// a CSS artboard, draws inside a turned and scaled frame a CSS slot
/// places where the same host draws inside the inline box: directly in
/// the frame, and inside a group in it, at each origin.
#[test]
fn a_component_host_in_a_css_slot_frame_draws_like_one_in_an_inline_box() {
    let contents = [
        FrameContent::ComponentHost,
        FrameContent::GroupedComponentHost,
    ];
    let origins = [(0.5, 0.5), (0.0, 0.0), (0.25, 0.75)];
    // The controls run first and outside catch_unwind, so a fixture that
    // does not hold its case fails with the control's own message.
    for origin in origins {
        let [direct, grouped] = contents.map(|content| component_host_controls(content, origin));
        assert_eq!(
            direct, grouped,
            "at {origin:?}: the group is transparent to layout"
        );
    }
    let cases = contents
        .into_iter()
        .flat_map(|content| origins.map(|origin| (content, origin)));
    let failed = failing_cases(cases, |(content, origin)| {
        frame_slides_turns_and_scales_like_an_inline_box(content, origin)
    });
    assert!(failed.is_empty(), "drawn unlike the inline box: {failed:?}");
}

/// The "Component" host: in the Screen for the inline box, inside the Unit
/// instance for a slot's frame, reached as `framed` reaches it.
fn component_host_in(scene: &Scene) -> CoreHandle {
    let card = card(scene);
    let board = if card.is_type_of(NestedArtboardLayoutBase::TYPE_KEY) {
        card.with_downcast::<NestedArtboardLayout, _>(|host| {
            host.base.base.artboard_instance_handle(0)
        })
        .flatten()
        .expect("the hosted frame")
    } else {
        scene.artboard.clone()
    };
    board
        .with_artboard(|board| board.find_handle::<NestedArtboardLayout>("Component"))
        .expect("the component host")
}

/// A component host case's positive controls, for the inline and the
/// hosted scene built as `frame_slides_turns_and_scales_like_an_inline_box`
/// builds them. Each reads file properties and the layout solver only. The
/// host is the case claimed, it joined the box's layout, its component
/// draws once and is CSS, and the origin, turn and scale are the case's.
/// Returns the component's layout position, the same in both scenes.
fn component_host_controls(content: FrameContent, origin: (f32, f32)) -> (f32, f32) {
    use crate::source::generated::{
        component_origin_base::ComponentOriginBase, node_base::NodeBase,
        transform_component_base::TransformComponentBase,
    };
    let spec = |host| FrameSpec {
        rotation: 0.35,
        scale: (1.5, 0.75),
        origin,
        content,
        ..FrameSpec::new(host)
    };
    let mut slots = Vec::new();
    for (name, host) in [
        ("inline", FrameHost::Inline),
        ("hosted", FrameHost::CssSlot),
    ] {
        let scene = frame_scene(spec(host));
        let label = format!("{name} {content:?} {origin:?}");
        // The case is the one claimed: the host's parent is the box, or a
        // group whose parent is the box.
        let parent_of = |object: &CoreHandle| {
            object
                .with(|object| {
                    object
                        .as_component()
                        .and_then(|component| component.parent_handle())
                })
                .flatten()
                .unwrap_or_else(|| panic!("{label}: a parent"))
        };
        let component_host = component_host_in(&scene);
        let the_box = framed(&scene);
        let parent = parent_of(&component_host);
        let in_box = match content {
            FrameContent::ComponentHost => parent,
            FrameContent::GroupedComponentHost => {
                assert_eq!(
                    parent.core_type(),
                    Some(NodeBase::TYPE_KEY),
                    "{label}: the host's parent is a group"
                );
                parent_of(&parent)
            }
            _ => unreachable!("{label}: not a component host case"),
        };
        assert_eq!(in_box, the_box, "{label}: the host sits in the box");
        if host == FrameHost::Inline {
            assert_eq!(
                in_box.core_type(),
                Some(LayoutComponentBase::TYPE_KEY),
                "{label}: the box is the Card"
            );
        } else {
            assert!(
                in_box.is_type_of(ArtboardBase::TYPE_KEY),
                "{label}: the box is the Unit"
            );
        }
        // The host joined the box's layout: its record sizes it, and its
        // slot is not zero.
        let bounds = component_host
            .with_downcast::<NestedArtboardLayout, _>(NestedArtboardLayout::layout_bounds)
            .unwrap_or_else(|| panic!("{label}: a NestedArtboardLayout"));
        assert_eq!(
            (bounds.width(), bounds.height()),
            (30.0, 20.0),
            "{label}: the host's record sizes it"
        );
        let component = component_host
            .with_downcast::<NestedArtboardLayout, _>(|host| {
                host.base.base.artboard_instance_handle(0)
            })
            .flatten()
            .unwrap_or_else(|| panic!("{label}: the mounted component"));
        let slot =
            component.with_artboard(|component| (component.layout_x(), component.layout_y()));
        assert_ne!(slot, (0.0, 0.0), "{label}: the host's slot is not zero");
        slots.push(slot);
        // The component draws, once.
        let draws: Vec<_> = painted_draws(&scene)
            .into_iter()
            .filter(|(color, _)| *color == COMPONENT_COLOR)
            .collect();
        assert!(
            draws.len() == 1 && draws[0].1.len() == 4,
            "{label}: the component draws once, four points: {draws:?}"
        );
        // The component is CSS.
        assert!(
            component.with_artboard(|component| component.is_css_mode()),
            "{label}: the component is CSS"
        );
        // The origin, turn and scale are the case's.
        let read =
            |object: &CoreHandle, key: u16| CoreRegistry::get_double_handle(object, key.into());
        let (holder, x, y) = if host == FrameHost::Inline {
            (
                component_origin(&scene),
                ComponentOriginBase::ORIGIN_X_PROPERTY_KEY,
                ComponentOriginBase::ORIGIN_Y_PROPERTY_KEY,
            )
        } else {
            (
                the_box.clone(),
                ArtboardBase::ORIGIN_X_PROPERTY_KEY,
                ArtboardBase::ORIGIN_Y_PROPERTY_KEY,
            )
        };
        assert_eq!(
            (read(&holder, x), read(&holder, y)),
            (Some(origin.0), Some(origin.1)),
            "{label}: the origin"
        );
        assert_eq!(
            [
                TransformComponentBase::ROTATION_PROPERTY_KEY,
                TransformComponentBase::SCALE_X_PROPERTY_KEY,
                TransformComponentBase::SCALE_Y_PROPERTY_KEY,
            ]
            .map(|key| read(&the_box, key)),
            [Some(0.35), Some(1.5), Some(0.75)],
            "{label}: the turn and scale"
        );
    }
    assert_eq!(
        slots[0], slots[1],
        "{content:?} {origin:?}: the component's slot, inline and hosted"
    );
    slots[0]
}

/// One draw's calls as the recording logs them: every save and restore,
/// every transform, every path point and every paint, with resource ids
/// renamed in order of use.
fn drawn_calls(scene: &Scene) -> Vec<String> {
    let mut renderer = scene._factory.borrow().make_renderer();
    scene._factory.borrow_mut().clear();
    scene.artboard.draw(&mut renderer);
    scene
        ._factory
        .borrow()
        .canonical_recording()
        .stream()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The two draws log the same calls, compared exactly.
fn assert_draws_alike(expected: &[String], actual: &[String], label: &str) {
    let first = (0..expected.len().max(actual.len()))
        .find(|&index| expected.get(index) != actual.get(index));
    if let Some(index) = first {
        panic!(
            "{label}: draw call {index} differs\n  expected {:?}\n  actual   {:?}",
            expected.get(index),
            actual.get(index)
        );
    }
}

/// CSS's `transform-origin` is a pivot only (docs/CSS_LAYOUT.md, 2.1): a
/// frame a CSS slot places, with no turn or scale, hands the renderer the
/// numbers it hands it at origin (0, 0), every transform and every path
/// point, whatever its origin.
#[test]
fn an_unturned_css_slot_frame_draws_the_same_numbers_at_any_origin() {
    let at = |content, origin| {
        frame_scene(FrameSpec {
            origin,
            content,
            ..FrameSpec::new(FrameHost::CssSlot)
        })
    };
    let cases = FrameContent::ALL
        .into_iter()
        .flat_map(|content| [(content, (0.5, 0.5)), (content, (0.25, 0.75))]);
    let failed = failing_cases(cases, |(content, origin)| {
        assert_draws_alike(
            &drawn_calls(&at(content, (0.0, 0.0))),
            &drawn_calls(&at(content, origin)),
            &format!("{content:?} at {origin:?}"),
        );
    });
    assert!(failed.is_empty(), "drawn unlike origin (0, 0): {failed:?}");
}

/// An origin written at run time, alone, after a frame.
#[derive(Clone, Copy, Debug)]
enum OriginWrite {
    /// An unturned frame's own `originX` and `originY`, against the same
    /// frame unwritten.
    UnturnedFrame(FrameContent, (f32, f32)),
    /// A frame its own turn and scale turn, its `originX` and `originY`,
    /// against the inline box with the same `ComponentOrigin` written.
    TurnedFrame((f32, f32)),
    /// A slot turned as in `pivot_scene` at this pivot, its
    /// `ComponentOrigin` written, against the inline box with the same
    /// write.
    TurnedSlot(f32),
}

/// An origin written at run time moves only the pivot. An unturned frame
/// draws as it did; a turned frame and a turned slot turn about the new
/// pivot, as the inline box does. The turned slot's pivot sits in its
/// host's world, which only the host's transform update recomputes.
#[test]
fn an_origin_written_at_run_time_moves_only_the_pivot() {
    let mut cases = Vec::new();
    for origin in [(0.5, 0.5), (0.25, 0.75)] {
        for content in FrameContent::ALL {
            cases.push(OriginWrite::UnturnedFrame(content, origin));
        }
        cases.push(OriginWrite::TurnedFrame(origin));
    }
    for pivot in [0.5, 0.0] {
        cases.push(OriginWrite::TurnedSlot(pivot));
    }
    let failed = failing_cases(cases, origin_written_at_run_time);
    assert!(
        failed.is_empty(),
        "an origin write moved more than the pivot: {failed:?}"
    );
}

/// The inline box's `ComponentOrigin`.
fn component_origin(scene: &Scene) -> CoreHandle {
    use crate::source::generated::component_origin_base::ComponentOriginBase;
    scene
        .artboard
        .with_artboard(|board| board.objects().to_vec())
        .into_iter()
        .flatten()
        .find(|object| object.is_type_of(ComponentOriginBase::TYPE_KEY))
        .expect("a ComponentOrigin")
}

fn origin_written_at_run_time(case: OriginWrite) {
    use crate::source::generated::component_origin_base::ComponentOriginBase;
    let frame_origin = |origin: (f32, f32)| {
        [
            (ArtboardBase::ORIGIN_X_PROPERTY_KEY, origin.0),
            (ArtboardBase::ORIGIN_Y_PROPERTY_KEY, origin.1),
        ]
    };
    let box_origin = |origin: (f32, f32)| {
        [
            (ComponentOriginBase::ORIGIN_X_PROPERTY_KEY, origin.0),
            (ComponentOriginBase::ORIGIN_Y_PROPERTY_KEY, origin.1),
        ]
    };
    // Each scene has run its first frames; the write is the only change
    // before the next one.
    match case {
        OriginWrite::UnturnedFrame(content, origin) => {
            let spec = FrameSpec {
                content,
                ..FrameSpec::new(FrameHost::CssSlot)
            };
            let unwritten = frame_scene(spec);
            let written = frame_scene(spec);
            write(&framed(&written), &frame_origin(origin));
            for scene in [&unwritten, &written] {
                scene.artboard.advance_default(0.0);
            }
            assert_draws_alike(
                &drawn_calls(&unwritten),
                &drawn_calls(&written),
                &format!("{case:?}"),
            );
        }
        OriginWrite::TurnedFrame(origin) => {
            let spec = |host| FrameSpec {
                rotation: 0.35,
                scale: (1.5, 0.75),
                ..FrameSpec::new(host)
            };
            let inline = frame_scene(spec(FrameHost::Inline));
            let hosted = frame_scene(spec(FrameHost::CssSlot));
            write(&component_origin(&inline), &box_origin(origin));
            write(&framed(&hosted), &frame_origin(origin));
            for scene in [&inline, &hosted] {
                scene.artboard.advance_default(0.0);
            }
            assert_painted_like_inline(&inline, &hosted);
        }
        OriginWrite::TurnedSlot(pivot) => {
            let inline = pivot_scene(false, pivot, SlotTurn::Own);
            let hosted = pivot_scene(true, pivot, SlotTurn::Own);
            for scene in [&inline, &hosted] {
                write(&component_origin(scene), &box_origin((0.25, 0.75)));
                scene.artboard.advance_default(0.0);
            }
            assert_painted_like_inline(&inline, &hosted);
        }
    }
}

/// Rive-made content keeps upstream's placement: a participant without a
/// CSS record sits at its slot in its parent's space, whatever the
/// artboard's origin (src/shapes/image.cpp, `Image::composeWorldTransform`).
#[test]
fn a_participant_without_a_css_record_keeps_upstreams_placement() {
    use crate::source::{
        generated::layout::layout_sizing_style_base::LayoutSizingStyleBase,
        layout::{
            layout_enums::LayoutScaleType, layout_participant::LayoutParticipant,
            layout_style_applier::YGUnit,
        },
        shapes::image::Image,
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    assets(&mut riv);
    // 0 artboard, 1 picture, 2 its participant, 3 the artboard's style.
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 400.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 300.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 3)
        .float(ArtboardBase::ORIGIN_X_PROPERTY_KEY, 0.5)
        .float(ArtboardBase::ORIGIN_Y_PROPERTY_KEY, 0.5)
        .end();
    {
        use crate::source::generated::shapes::image_base::ImageBase;
        riv.object(ImageBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(ImageBase::ASSET_ID_PROPERTY_KEY, 0)
            .float(ImageBase::ORIGIN_X_PROPERTY_KEY, 0.0)
            .float(ImageBase::ORIGIN_Y_PROPERTY_KEY, 0.0)
            .end();
    }
    riv.object(LayoutParticipant::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
        .uint(
            LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
            LayoutScaleType::Hug as u64,
        )
        .uint(
            LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
            LayoutScaleType::Hug as u64,
        )
        .end();
    riv.object(LayoutComponentStyleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(
            LayoutComponentStyleBase::PADDING_LEFT_UNITS_VALUE_PROPERTY_KEY,
            YGUnit::Point as u64,
        )
        .float(LayoutComponentStyleBase::PADDING_LEFT_PROPERTY_KEY, 10.0)
        .uint(
            LayoutComponentStyleBase::PADDING_TOP_UNITS_VALUE_PROPERTY_KEY,
            YGUnit::Point as u64,
        )
        .float(LayoutComponentStyleBase::PADDING_TOP_PROPERTY_KEY, 12.0)
        .end();
    let scene = Scene::import(riv);
    scene.advance();
    let image = scene
        .artboard
        .with_artboard(|board| board.find_all_handles::<Image>()[0].clone());
    let participant = image
        .with_downcast::<Image, _>(Image::layout_participant)
        .flatten()
        .expect("the picture's participant");
    let (left, top, participating, record) = participant
        .with_downcast::<LayoutParticipant, _>(|participant| {
            (
                participant.resolved_left(),
                participant.resolved_top(),
                participant.is_participating_in_layout(),
                participant.css_record(),
            )
        })
        .expect("live participant");
    assert!(participating && record.is_none());
    // The artboard's padding places it; an origin step-back would not.
    assert!(left > 0.0 && top > 0.0, "slot ({left}, {top})");
    let parent = image
        .with(|image| image.component_parent_handle())
        .flatten()
        .expect("the artboard");
    let parent_world = parent
        .with(|parent| {
            parent
                .as_world_transform_component()
                .map(|parent| parent.world_transform())
        })
        .flatten()
        .expect("the artboard's world");
    let (world, transform) = image
        .with_downcast::<Image, _>(|image| (image.base.world_transform(), image.base.transform()))
        .expect("live picture");
    // m_WorldTransform = parent.worldTransform() * base * m_Transform.
    let expected = parent_world
        * Mat2D::from_translation(crate::source::math::vec2d::Vec2D::new(left, top))
        * transform;
    for i in 0..6 {
        assert!(
            (world[i] - expected[i]).abs() < 1e-4,
            "{world:?} != {expected:?}"
        );
    }
}

/// Upstream's path: load zeroes an artboard's x and y and a change only
/// tells its host, an artboard composes no offset, and it turns about its
/// own origin (src/artboard.cpp:2050-2060, :3941-3951, :2986-2995;
/// src/layout_component.cpp:280-284).
#[test]
fn top_level_frames_outside_a_css_slot_keep_upstreams_transform() {
    use crate::source::generated::{
        artboard_base::ArtboardBase, node_base::NodeBase,
        transform_component_base::TransformComponentBase,
    };
    let origin = (0.25, 0.75);
    let turn = 0.35;
    // Each case: the scene, the frame, and the artboard that hosts it.
    let row_scene = || {
        let mut riv = Riv::default();
        riv.object(BackboardBase::TYPE_KEY).end();
        // Page 0, column record 1, list 2, style 3.
        board(&mut riv, 400.0, 300.0, 3);
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(DISPLAY, 1)
            .uint(FLEX_DIRECTION, 2)
            .end();
        riv.object(ArtboardComponentListBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .end();
        style(&mut riv, 0);
        // A CSS Screen: 0 artboard, 1 record, 2 and 3 its paint, 4 style.
        riv.object(ArtboardBase::TYPE_KEY)
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 46.0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 4)
            .float(ArtboardBase::ORIGIN_X_PROPERTY_KEY, origin.0)
            .float(ArtboardBase::ORIGIN_Y_PROPERTY_KEY, origin.1)
            .end();
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .end();
        painted_fill(&mut riv, 0, 2, ROW_COLOR);
        style(&mut riv, 0);
        let scene = Scene::import(riv);
        let list = scene
            .artboard
            .with_artboard(|board| board.find_all_handles::<ArtboardComponentList>()[0].clone());
        let arena = list.retain_arena().expect("list arena");
        let item = arena.insert(ViewModelInstanceListItem::default());
        ArtboardComponentList::update_list_occurrence(&list, &[item]);
        let source = scene
            .file
            .with_file(|file| file.artboard_at_source(1))
            .expect("row source");
        let row = Artboard::nested_instance_from_handle(&source).expect("row instance");
        ArtboardComponentList::add_artboard_at_occurrence(&list, row, 0, true);
        scene.advance();
        let frame = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(0))
            .flatten()
            .expect("mounted row")
            .core_handle();
        (scene, frame)
    };
    let slot_scene = |host| {
        let scene = frame_scene(FrameSpec {
            origin,
            ..FrameSpec::new(host)
        });
        let frame = framed(&scene);
        (scene, frame)
    };
    for (label, (scene, frame)) in [
        (
            "a CSS component in a host without an item record",
            slot_scene(FrameHost::RecordFreeSlot),
        ),
        (
            "a non-CSS artboard in a CSS slot",
            slot_scene(FrameHost::NonCssFrame),
        ),
        ("a CSS Screen as a list row", row_scene()),
    ] {
        let sorts = || {
            scene
                .artboard
                .with_artboard(|board| board.dirty_handle().draw_order_change_counter())
        };
        let at_rest = painted_draws(&scene);
        let start = sorts();
        // It turns about its own origin, as upstream draws it.
        write(
            &frame,
            &[(TransformComponentBase::ROTATION_PROPERTY_KEY, turn)],
        );
        scene.artboard.advance_default(0.0);
        let turned = painted_draws(&scene);
        let own = &at_rest[0].1;
        let (min, max) = own.iter().fold(
            (
                crate::source::math::vec2d::Vec2D::new(f32::INFINITY, f32::INFINITY),
                crate::source::math::vec2d::Vec2D::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
            ),
            |(min, max), p| {
                (
                    crate::source::math::vec2d::Vec2D::new(min.x.min(p.x), min.y.min(p.y)),
                    crate::source::math::vec2d::Vec2D::new(max.x.max(p.x), max.y.max(p.y)),
                )
            },
        );
        let pivot = crate::source::math::vec2d::Vec2D::new(
            min.x + origin.0 * (max.x - min.x),
            min.y + origin.1 * (max.y - min.y),
        );
        let rotate = Mat2D::from_rotation(turn);
        assert_eq!(turned.len(), at_rest.len(), "{label}");
        for ((_, rest), (_, turned)) in at_rest.iter().zip(&turned) {
            assert_eq!(rest.len(), turned.len(), "{label}");
            for (rest, turned) in rest.iter().zip(turned) {
                let expected = pivot + rotate * (*rest - pivot);
                assert!(
                    (turned.x - expected.x).abs() < 0.01 && (turned.y - expected.y).abs() < 0.01,
                    "{label}: turned {turned:?}, expected {expected:?}"
                );
            }
        }
        // Its own x and y draw nothing.
        write(
            &frame,
            &[
                (NodeBase::X_PROPERTY_KEY, 10.0),
                (NodeBase::Y_PROPERTY_KEY, 6.0),
            ],
        );
        scene.artboard.advance_default(0.0);
        let moved = painted_draws(&scene);
        assert_eq!(moved.len(), turned.len(), "{label}");
        for ((_, turned), (_, moved)) in turned.iter().zip(&moved) {
            for (turned, moved) in turned.iter().zip(moved) {
                assert!(
                    (turned.x - moved.x).abs() < 0.001 && (turned.y - moved.y).abs() < 0.001,
                    "{label}: moved {moved:?}, expected {turned:?}"
                );
            }
        }
        // And neither re-sorts its host.
        assert_eq!(sorts(), start, "{label}: the host keeps its draw order");
    }
}

/// The order `colors` paint in, bottom first, matched on their color
/// channels: opacity changes a draw's alpha.
fn paint_order(scene: &Scene, colors: &[u32]) -> Vec<u32> {
    let draws = painted_draws(scene);
    let mut found: Vec<_> = colors
        .iter()
        .filter_map(|&color| {
            draws
                .iter()
                .position(|(drawn, _)| drawn & 0x00ff_ffff == color & 0x00ff_ffff)
                .map(|at| (at, color))
        })
        .collect();
    found.sort();
    found.into_iter().map(|(_, color)| color).collect()
}

/// CSS 2.1 Appendix E: a box with its own opacity below 1 or a transform
/// is a stacking context, painted with positioned boxes above later flow
/// boxes. A frame a CSS slot places is such a box, so its slot takes the
/// layer that box takes, as the same Screen with an inline box does.
#[test]
fn css_stacking_slot_takes_its_frames_layer() {
    use crate::source::generated::world_transform_component_base::WorldTransformComponentBase;
    let below = vec![CARD_COLOR, LATER_COLOR];
    let above = vec![LATER_COLOR, CARD_COLOR];
    let pair = |spec: FrameSpec| {
        (
            frame_scene(FrameSpec {
                host: FrameHost::Inline,
                ..spec
            }),
            frame_scene(spec),
        )
    };
    let assert_order = |inline: &Scene, hosted: &Scene, expected: &[u32], label: &str| {
        let colors = [CARD_COLOR, LATER_COLOR];
        assert_eq!(paint_order(inline, &colors), expected, "{label}: inline");
        assert_eq!(paint_order(hosted, &colors), expected, "{label}: slot");
    };
    let slot = FrameSpec {
        later: true,
        origin: (0.5, 0.5),
        ..FrameSpec::new(FrameHost::CssSlot)
    };
    let (inline, hosted) = pair(slot);
    assert_order(&inline, &hosted, &below, "at rest");

    // 1. A frame turned at rest.
    let (inline, hosted) = pair(FrameSpec {
        rotation: 0.35,
        ..slot
    });
    assert_order(&inline, &hosted, &above, "turned");

    // 2. A frame's opacity re-sorts its Screen exactly when it crosses 1.
    let (inline, hosted) = pair(slot);
    let sorts = || {
        hosted
            .artboard
            .with_artboard(|board| board.dirty_handle().draw_order_change_counter())
    };
    let start = sorts();
    for (opacity, expected, resorts) in [
        (0.5, &above, 1u8),
        (0.75, &above, 1),
        (1.0, &below, 2),
        (1.0, &below, 2),
    ] {
        for scene in [&inline, &hosted] {
            write(
                &framed(scene),
                &[(WorldTransformComponentBase::OPACITY_PROPERTY_KEY, opacity)],
            );
            scene.artboard.advance_default(0.0);
        }
        assert_order(&inline, &hosted, expected, &format!("opacity {opacity}"));
        assert_eq!(sorts(), start.wrapping_add(resorts), "opacity {opacity}");
    }

    // 3. A track on its turn at 0 authors a transform: a context at identity.
    let (inline, hosted) = pair(FrameSpec {
        keyed_turn: true,
        ..slot
    });
    assert_order(&inline, &hosted, &above, "keyed turn at 0");

    // 4. The slot's own fade.
    let (inline, hosted) = pair(slot);
    for (opacity, expected) in [(0.0, None), (0.5, Some(&above)), (1.0, Some(&below))] {
        write(
            &framed(&inline),
            &[(WorldTransformComponentBase::OPACITY_PROPERTY_KEY, opacity)],
        );
        write(
            &card(&hosted),
            &[(WorldTransformComponentBase::OPACITY_PROPERTY_KEY, opacity)],
        );
        for scene in [&inline, &hosted] {
            scene.artboard.advance_default(0.0);
        }
        match expected {
            Some(expected) => {
                assert_order(&inline, &hosted, expected, &format!("slot fade {opacity}"))
            }
            None => assert_eq!(
                paint_order(&hosted, &[CARD_COLOR, LATER_COLOR]),
                [LATER_COLOR],
                "a slot faded out paints nothing"
            ),
        }
    }

    // A slot without an item record, or with a frame that has no record of
    // its own, keeps its layer.
    for host in [FrameHost::RecordFreeSlot, FrameHost::NonCssFrame] {
        let scene = frame_scene(FrameSpec {
            host,
            rotation: 0.35,
            opacity: 0.5,
            ..slot
        });
        assert_eq!(
            paint_order(&scene, &[CARD_COLOR, LATER_COLOR]),
            below,
            "{host:?}"
        );
    }
}
