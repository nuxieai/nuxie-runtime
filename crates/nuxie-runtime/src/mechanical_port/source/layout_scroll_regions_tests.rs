//! The scroll regions a host's capture carries (semantic_scroll_regions.rs).
//!
//! Oracle: CSS arithmetic for where each viewport's padding box lands, and
//! real drags through the state machine for where a scroll stands: a region
//! can move a way exactly when the content is not at that end of its range.
use super::*;
use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
use crate::source::generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase;
use crate::source::generated::core_registry::CoreRegistry;
use crate::source::semantic::semantic_provider::semantic_source_is_visible;
use crate::source::semantic::semantic_scroll_regions::{
    MAX_SCROLL_REGIONS, ScrollRegion, ScrollRegions, TooManyScrollRegions, can_scroll,
};

const VERTICAL: u64 = 1;
const HORIZONTAL: u64 = 0;

fn regions(f: &Scene) -> ScrollRegions {
    ScrollRegions::collect(&f.artboard.core_handle()).expect("within the limit")
}

/// A box's border box in root space, from its solved layout.
fn border_box(f: &Scene, name: &str) -> [f32; 4] {
    let [left, top, width, height] = f.bounds(name);
    [left, top, left + width, top + height]
}

fn region_box(region: &ScrollRegion) -> [f32; 4] {
    let b = region.bounds;
    [b.min_x, b.min_y, b.max_x, b.max_y]
}

/// A vertical CSS scroll box as the compiler writes one, 200 by 88 with a
/// 4 point border, so its padding box is 192 by 80, whose list holds
/// `rows`.
fn scroll_box(viewport: &'static str, list: &'static str, rows: Vec<B>) -> B {
    css(
        viewport,
        [
            [
                U(POSITION, 1),
                U(OVERFLOW_Y, 3),
                U(BOX_SIZING, 1),
                F(BORDER_TOP, 4.0),
                F(BORDER_RIGHT, 4.0),
                F(BORDER_BOTTOM, 4.0),
                F(BORDER_LEFT, 4.0),
            ]
            .as_slice(),
            &width(200.0),
            &height(88.0),
        ]
        .concat(),
        vec![B {
            scroll: Some(VERTICAL),
            scroll_threshold: 10.0,
            ..css(list, [], rows)
        }],
    )
}

/// A 40 point "Row" holding a 180 by 40 "Item", above a 200 point "Tail":
/// a list 240 tall, so a range of 160 in an 80 point padding box.
fn tall_rows() -> Vec<B> {
    vec![
        css(
            "Row",
            height(40.0),
            vec![css("Item", [width(180.0), height(40.0)].concat(), vec![])],
        ),
        css("Tail", height(200.0), vec![]),
    ]
}

/// A press at `from` moved by `dy` points in steps of 10, held.
fn drag_vertically(f: &Scene, from: (f32, f32), dy: f32) {
    let steps = (dy.abs() / 10.0).ceil() as usize;
    let path = (1..=steps)
        .map(|step| {
            let moved = (dy * step as f32 / steps as f32).round();
            (from.0, from.1 + moved)
        })
        .collect::<Vec<_>>();
    press_and_hold(&f.machine(), from, &path);
    f.advance(1);
}

/// A CSS scroll box at its top can move toward its bottom only; dragged
/// 30 points up, both ways; dragged to the end of its 160 point range,
/// toward its top only. Its box is its padding box, inside the border.
#[test]
fn css_scroll_box_region_moves_as_far_as_its_range() {
    let start = scene((400.0, 300.0), scroll_box("Viewport", "List", tall_rows()));
    start.machine();
    let found = regions(&start);
    assert_eq!(found.regions().len(), 1);
    let [left, top, right, bottom] = border_box(&start, "Viewport");
    assert_eq!(
        region_box(&found.regions()[0]),
        [left + 4.0, top + 4.0, right - 4.0, bottom - 4.0]
    );
    assert_eq!(found.regions()[0].parent, -1);
    assert_eq!(found.regions()[0].can_scroll, can_scroll::BOTTOM);

    let middle = scene((400.0, 300.0), scroll_box("Viewport", "List", tall_rows()));
    drag_vertically(&middle, (100.0, 60.0), -30.0);
    assert_eq!(middle.scroll_offset("List"), (0.0, -30.0));
    assert_eq!(
        regions(&middle).regions()[0].can_scroll,
        can_scroll::TOP | can_scroll::BOTTOM
    );

    let end = scene((400.0, 300.0), scroll_box("Viewport", "List", tall_rows()));
    drag_vertically(&end, (100.0, 70.0), -160.0);
    assert_eq!(end.scroll_offset("List"), (0.0, -160.0));
    let found = regions(&end);
    assert_eq!(found.regions()[0].can_scroll, can_scroll::TOP);
    // Scrolled out of view: the item is clipped away, which takes its
    // actions, and it still names the region it is in.
    let item = end.named("Item");
    assert!(!semantic_source_is_visible(&item));
    assert_eq!(found.region_of(&item), 0);
    // Scrolling does not move the viewport.
    assert_eq!(
        region_box(&found.regions()[0]),
        [left + 4.0, top + 4.0, right - 4.0, bottom - 4.0]
    );
}

/// A scroll box whose list fits is listed and can move no way.
#[test]
fn css_scroll_box_that_fits_is_listed_without_moves() {
    let f = scene(
        (400.0, 300.0),
        scroll_box("Viewport", "List", vec![css("Row", height(60.0), vec![])]),
    );
    f.machine();
    let found = regions(&f);
    assert_eq!(found.regions().len(), 1);
    assert_eq!(found.regions()[0].can_scroll, 0);
}

/// An inner scroll box in an outer one: two regions, the inner after the
/// outer and naming it; a box in the inner list names the inner region, a
/// box in the outer list the outer, and a box outside both neither.
#[test]
fn inner_scroll_box_names_the_outer_as_its_parent() {
    let f = scene(
        (400.0, 300.0),
        scroll_box(
            "Viewport",
            "List",
            vec![
                scroll_box("Inner viewport", "Inner list", tall_rows()),
                css("Outer tail", height(200.0), vec![]),
            ],
        ),
    );
    f.machine();
    let found = regions(&f);
    assert_eq!(found.regions().len(), 2);
    assert_eq!(found.regions()[0].parent, -1);
    assert_eq!(found.regions()[1].parent, 0);
    // The inner border box sits at the outer padding box's corner.
    let [left, top, _, _] = border_box(&f, "Viewport");
    assert_eq!(
        region_box(&found.regions()[1]),
        [left + 8.0, top + 8.0, left + 8.0 + 192.0, top + 8.0 + 80.0]
    );
    assert_eq!(found.region_of(&f.named("Item")), 1);
    assert_eq!(found.region_of(&f.named("Inner list")), 1);
    assert_eq!(found.region_of(&f.named("Inner viewport")), 0);
    assert_eq!(found.region_of(&f.named("Outer tail")), 0);
    assert_eq!(found.region_of(&f.named("Viewport")), -1);
    assert_eq!(found.region_of(&f.artboard.core_handle()), -1);
}

/// A translated (Rive-made) horizontal scroller inside a nested artboard,
/// inside a CSS scroll box: listed after the box and naming it, its box in
/// root space, so it moves up with the outer scroll. At its start it can
/// move toward its right only; dragged sideways, both ways.
#[test]
fn nested_translated_scroller_is_listed_in_root_space() {
    let holder = || B {
        nested_scroller: true,
        ..css("Holder", [width(200.0), height(40.0)].concat(), vec![])
    };
    let f = css_scroll_view(None, holder());
    f.machine();
    let found = regions(&f);
    assert_eq!(found.regions().len(), 2);
    let [left, top, _, _] = border_box(&f, "Viewport");
    let nested = found.regions()[1];
    assert_eq!(nested.parent, 0);
    assert_eq!(region_box(&nested), [left, top, left + 200.0, top + 40.0]);
    assert_eq!(nested.can_scroll, can_scroll::RIGHT);
    assert_eq!(found.regions()[0].can_scroll, can_scroll::BOTTOM);
    // The nested list, and the nested artboard itself, by their hosts.
    let nested_scroll = f.nested_scroll();
    let nested_list = nested_scroll
        .with(|object| object.component_parent_handle())
        .flatten()
        .expect("the nested list");
    assert_eq!(found.region_of(&nested_list), 1);
    let nested_artboard = nested_list
        .with(|object| object.component_artboard_handle())
        .flatten()
        .expect("the nested artboard");
    assert_eq!(found.region_of(&nested_artboard), 0);

    let f = css_scroll_view(None, holder());
    press_and_hold(
        &f.machine(),
        (100.0, 35.0),
        &[(100.0, 25.0), (100.0, 15.0), (100.0, 5.0)],
    );
    f.advance(1);
    assert_eq!(f.scroll_offset("List"), (0.0, -30.0));
    let moved = regions(&f).regions()[1];
    assert_eq!(
        region_box(&moved),
        [left, top - 30.0, left + 200.0, top + 10.0]
    );

    let f = css_scroll_view(None, holder());
    press_and_hold(&f.machine(), (100.0, 20.0), &[(60.0, 20.0), (20.0, 20.0)]);
    f.advance(1);
    assert!(f.nested_scroll_offset().0 < 0.0);
    assert_eq!(
        regions(&f).regions()[1].can_scroll,
        can_scroll::LEFT | can_scroll::RIGHT
    );
}

/// The nested artboard's horizontal scroller (200 by 40 over a 400 point
/// list) inside a page padded by 150, its host turned by `rotation` and
/// scaled by `scale_x`.
fn hosted_scroller(rotation: f32, scale_x: f32) -> Scene {
    use crate::source::generated::transform_component_base::TransformComponentBase;
    hosted_scroller_with(|host| {
        for (key, value) in [
            (TransformComponentBase::ROTATION_PROPERTY_KEY, rotation),
            (TransformComponentBase::SCALE_X_PROPERTY_KEY, scale_x),
        ] {
            assert!(CoreRegistry::set_double_handle(host, key.into(), value));
        }
    })
}

/// The same scene, its nested host changed by `edit` before the first
/// frame.
fn hosted_scroller_with(edit: impl Fn(&CoreHandle)) -> Scene {
    use crate::source::nested_artboard::NestedArtboard;
    let f = scene(
        (500.0, 500.0),
        css(
            "Page",
            [padding(150.0).as_slice(), &width(500.0), &height(500.0)].concat(),
            vec![B {
                nested_scroller: true,
                ..css("Holder", [width(200.0), height(40.0)].concat(), vec![])
            }],
        ),
    );
    let host = f
        .artboard
        .with_artboard(|artboard| artboard.find_handle::<NestedArtboard>("Nested"))
        .expect("the nested host");
    edit(&host);
    f.machine();
    f.advance(1);
    f
}

/// A press at the region's center moved by `(dx, dy)` in four steps, held.
fn drag_region(f: &Scene, dx: f32, dy: f32) {
    let [left, top, right, bottom] = region_box(&regions(f).regions()[0]);
    let (x, y) = ((left + right) / 2.0, (top + bottom) / 2.0);
    let moves: Vec<_> = (1..=4)
        .map(|step| {
            let step = step as f32 / 4.0;
            (x + dx * step, y + dy * step)
        })
        .collect();
    press_and_hold(&f.machine(), (x, y), &moves);
    f.advance(1);
}

/// A host that turns or mirrors a scroller: its bits name the way its
/// content moves on screen, and a drag that way moves it. Unturned, the
/// horizontal scroller at its start can show content to its right; a
/// quarter turn makes that its bottom, and mirroring it on x its left.
#[test]
fn turned_and_mirrored_hosts_name_screen_directions() {
    use std::f32::consts::FRAC_PI_2;
    for (name, rotation, scale_x, size, at_start, drag) in [
        (
            "plain",
            0.0,
            1.0,
            (200.0, 40.0),
            can_scroll::RIGHT,
            (-40.0, 0.0),
        ),
        (
            "quarter turn",
            FRAC_PI_2,
            1.0,
            (40.0, 200.0),
            can_scroll::BOTTOM,
            (0.0, -40.0),
        ),
        (
            "mirrored",
            0.0,
            -1.0,
            (200.0, 40.0),
            can_scroll::LEFT,
            (40.0, 0.0),
        ),
    ] {
        let f = hosted_scroller(rotation, scale_x);
        let found = regions(&f);
        assert_eq!(found.regions().len(), 1, "{name}");
        let [left, top, right, bottom] = region_box(&found.regions()[0]);
        assert!(
            ((right - left) - size.0).abs() < 0.01 && ((bottom - top) - size.1).abs() < 0.01,
            "{name}: {:?}",
            [left, top, right, bottom]
        );
        assert_eq!(found.regions()[0].can_scroll, at_start, "{name}");

        let f = hosted_scroller(rotation, scale_x);
        drag_region(&f, drag.0, drag.1);
        assert!(
            f.nested_scroll_offset().0 < 0.0,
            "{name}: a drag the way the bit names moves the content: {:?}",
            f.nested_scroll_offset()
        );
        let opposite = match at_start {
            can_scroll::RIGHT => can_scroll::LEFT,
            can_scroll::LEFT => can_scroll::RIGHT,
            can_scroll::BOTTOM => can_scroll::TOP,
            _ => can_scroll::BOTTOM,
        };
        assert_eq!(
            regions(&f).regions()[0].can_scroll,
            at_start | opposite,
            "{name}"
        );
    }
}

/// A host the pointer path turns away lists no region, and a drag where
/// its scroller would be moves nothing: hidden, paused, or scaled to 0 on
/// one axis (its transform cannot be inverted). Unchanged, the same drag
/// moves the scroller (turned_and_mirrored_hosts_name_screen_directions).
#[test]
fn hosts_that_take_no_drag_list_no_region() {
    use crate::source::drawable_flag::DrawableFlag;
    use crate::source::generated::{
        drawable_base::DrawableBase, nested_artboard_base::NestedArtboardBase,
        transform_component_base::TransformComponentBase,
    };
    let plain = hosted_scroller(0.0, 1.0);
    let [left, top, right, bottom] = region_box(&regions(&plain).regions()[0]);
    let (x, y) = ((left + right) / 2.0, (top + bottom) / 2.0);
    let gates: [(&str, fn(&CoreHandle)); 3] = [
        ("hidden", |host| {
            assert!(CoreRegistry::set_uint_handle(
                host,
                DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY.into(),
                u32::from(DrawableFlag::HIDDEN.0)
            ))
        }),
        ("paused", |host| {
            assert!(CoreRegistry::set_bool_handle(
                host,
                NestedArtboardBase::IS_PAUSED_PROPERTY_KEY.into(),
                true
            ))
        }),
        ("no height", |host| {
            assert!(CoreRegistry::set_double_handle(
                host,
                TransformComponentBase::SCALE_Y_PROPERTY_KEY.into(),
                0.0
            ))
        }),
    ];
    for (name, gate) in gates {
        let f = hosted_scroller_with(gate);
        assert!(regions(&f).regions().is_empty(), "{name}");
        press_and_hold(&f.machine(), (x, y), &[(x - 20.0, y), (x - 40.0, y)]);
        f.advance(1);
        assert_eq!(f.nested_scroll_offset(), (0.0, 0.0), "{name}");
    }
}

/// An inner scroller scrolled wholly out of its outer one's padding box is
/// clipped away: it is not listed, and a drag where it was moves the outer
/// scroller, not it.
#[test]
fn a_scroller_clipped_away_is_not_listed() {
    let f = scene(
        (400.0, 300.0),
        scroll_box(
            "Viewport",
            "List",
            vec![
                scroll_box("Inner viewport", "Inner list", tall_rows()),
                css("Outer tail", height(200.0), vec![]),
            ],
        ),
    );
    f.machine();
    let [left, top, right, bottom] = region_box(&regions(&f).regions()[1]);
    let (x, y) = ((left + right) / 2.0, (top + bottom) / 2.0);
    // The outer list is 288 tall in an 80 point padding box: at its end
    // the inner box, 88 tall at its top, is 120 points above the viewport.
    let outer = f
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
        .into_iter()
        .find(|scroll| {
            scroll
                .with(|object| object.component_parent_handle())
                .flatten()
                == Some(f.named("List"))
        })
        .expect("the outer scroll");
    assert!(CoreRegistry::set_double_handle(
        &outer,
        ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY.into(),
        -208.0
    ));
    f.advance(2);
    assert_eq!(f.scroll_offset("List"), (0.0, -208.0));
    let found = regions(&f);
    assert_eq!(found.regions().len(), 1);
    assert_eq!(found.regions()[0].can_scroll, can_scroll::TOP);
    drag_vertically(&f, (x, y), 30.0);
    assert!(
        f.scroll_offset("List").1 > -208.0,
        "the outer scroller takes it"
    );
    assert_eq!(f.scroll_offset("Inner list"), (0.0, 0.0));
}

/// A translated scroll view on its own: one region, its viewport's box.
#[test]
fn translated_scroll_view_is_listed() {
    let f = translated_scroll_view(css("Item", [width(180.0), height(40.0)].concat(), vec![]));
    f.machine();
    let found = regions(&f);
    assert_eq!(found.regions().len(), 1);
    assert_eq!(region_box(&found.regions()[0]), border_box(&f, "Viewport"));
    assert_eq!(found.regions()[0].can_scroll, can_scroll::BOTTOM);
}

/// Right to left, the start is the viewport's right edge: at its start a
/// region can move toward its left, showing the content to the left, and
/// at its end toward its right. The bits name screen directions.
#[test]
fn rtl_region_names_screen_directions() {
    const RTL: u64 = 2;
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [
                [U(DISPLAY, 1), U(DIRECTION, RTL), U(OVERFLOW_X, 3)].as_slice(),
                &width(100.0),
                &height(20.0),
            ]
            .concat(),
            vec![B {
                scroll: Some(HORIZONTAL),
                ..css(
                    "Content",
                    [
                        [F(FLEX_SHRINK, 0.0)].as_slice(),
                        &width(300.0),
                        &height(20.0),
                    ]
                    .concat(),
                    vec![css("Item", [width(300.0), height(20.0)].concat(), vec![])],
                )
            }],
        ),
    );
    f.advance(1);
    assert_eq!(f.world_origin("Item").0, -200.0, "the right edge shows");
    assert_eq!(regions(&f).regions()[0].can_scroll, can_scroll::LEFT);
    f.artboard
        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
        .into_iter()
        .next()
        .expect("a scroll constraint")
        .with_downcast_mut::<ScrollConstraint, _>(|scroll| {
            scroll.drag_view(
                crate::source::math::vec2d::Vec2D::new(500.0, 0.0),
                0.0,
                false,
            )
        })
        .unwrap();
    f.advance(2);
    assert_eq!(f.world_origin("Item").0, 0.0, "the left edge shows");
    assert_eq!(regions(&f).regions()[0].can_scroll, can_scroll::RIGHT);
}

/// A constraint that is not interactive is not a region, and a scroller
/// inside a collapsed box is not walked into, so it is not one either.
#[test]
fn inactive_and_collapsed_scrollers_are_not_listed() {
    let f = scene((400.0, 300.0), scroll_box("Viewport", "List", tall_rows()));
    f.machine();
    let scroll = f
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
        .into_iter()
        .next()
        .expect("a scroll constraint");
    assert!(CoreRegistry::set_bool_handle(
        &scroll,
        ScrollConstraintBase::INTERACTIVE_PROPERTY_KEY.into(),
        false
    ));
    assert!(regions(&f).regions().is_empty());
    assert_eq!(regions(&f).region_of(&f.named("Item")), -1);

    // `display: none` around the scroller collapses its subtree.
    let f = scene(
        (400.0, 300.0),
        css(
            "Hidden",
            [U(DISPLAY, NONE as u64)],
            vec![scroll_box("Viewport", "List", tall_rows())],
        ),
    );
    f.machine();
    assert!(
        f.named("Hidden")
            .with_downcast::<LayoutComponent, _>(LayoutComponent::is_collapsed)
            .unwrap()
    );
    assert!(regions(&f).regions().is_empty());
    assert_eq!(regions(&f).region_of(&f.named("Item")), -1);
}

/// The walk stops past its limit and fails as a whole.
#[test]
fn regions_past_the_limit_fail_the_collection() {
    let f = scene(
        (400.0, 300.0),
        scroll_box(
            "Viewport",
            "List",
            vec![scroll_box("Inner viewport", "Inner list", tall_rows())],
        ),
    );
    f.machine();
    let root = f.artboard.core_handle();
    assert!(matches!(
        ScrollRegions::collect_at_most(&root, 1),
        Err(TooManyScrollRegions)
    ));
    assert_eq!(
        ScrollRegions::collect_at_most(&root, 2)
            .map(|found| found.regions().len())
            .ok(),
        Some(2)
    );
    assert_eq!(MAX_SCROLL_REGIONS, 256);
}
