//! CSS sizing through real layout owners and the upstream text measurer.
use super::*;
use crate::source::{
    assets::font_asset::FontAsset,
    core::CoreArena,
    layout::{
        layout_component_style::LayoutComponentStyle,
        layout_enums::LayoutAlignmentType,
        layout_participant::LayoutParticipant,
        layout_style_applier::{YGFlexDirection, YGUnit},
    },
    text::{font_hb::HbFont, text::Text, text_style::TextStyle, text_value_run::TextValueRun},
};

thread_local! {
    static HOST_MEASURES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
pub(super) fn note_host_measure() {
    HOST_MEASURES.with(|count| count.set(count.get() + 1));
}
fn host_measure_count() -> usize {
    HOST_MEASURES.with(std::cell::Cell::get)
}

struct ParentContext<'a> {
    arena: &'a CoreArena,
    parent: CoreHandle,
}
impl CoreContext for ParentContext<'_> {
    fn core_arena(&self) -> &CoreArena {
        self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        (id == 1).then(|| self.parent.clone())
    }
}
fn attach(arena: &CoreArena, parent: &CoreHandle, child: &CoreHandle) {
    let mut context = ParentContext {
        arena,
        parent: parent.clone(),
    };
    assert_eq!(
        child.with_mut(|o| {
            let component = o.as_component_mut().unwrap();
            component.base.set_parent_id_value(1);
            component.on_added_dirty(&mut context)
        }),
        Some(StatusCode::Ok)
    );
}
/// A Text with one run of `content` in Inter Bold 28 on line height 36.
fn make_text(arena: &CoreArena, content: &str) -> (CoreHandle, CoreHandle) {
    let asset = arena.insert(FontAsset::default());
    let font_root = std::env::var_os("RIVE_RUNTIME_DIR")
        .expect("set RIVE_RUNTIME_DIR to the pinned upstream fixture checkout");
    let font = std::fs::read(
        std::path::PathBuf::from(font_root)
            .join("tests/unit_tests/assets/fonts/Inter_28pt-Bold.ttf"),
    )
    .unwrap();
    FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(&font).unwrap()));
    let mut style = TextStyle::default();
    style.base.set_font_size_value(28.0);
    style.base.set_line_height_value(36.0);
    let style = arena.insert(style);
    TextStyle::set_asset_occurrence(&style, Some(asset));
    let mut run = TextValueRun::default();
    run.base.set_text_value(content.into());
    run.set_style(style);
    let run = arena.insert(run);
    let mut text = Text::default();
    text.add_run(run.clone());
    let text = arena.insert(text);
    attach(&arena, &text, &run);
    (text, run)
}
fn layout(
    arena: &CoreArena,
    parent: Option<&CoreHandle>,
    width: Option<f32>,
    row: bool,
) -> CoreHandle {
    let owner = arena.insert(LayoutComponent::default());
    if let Some(parent) = parent {
        attach(arena, parent, &owner);
    }
    let mut style = LayoutComponentStyle::default();
    style
        .base
        .set_layout_width_scale_type_value(if width.is_some() {
            LayoutScaleType::Fixed
        } else {
            LayoutScaleType::Hug
        } as u8);
    style
        .base
        .set_layout_height_scale_type_value(LayoutScaleType::Hug as u8);
    style.base.set_width_units_value_value(if width.is_some() {
        YGUnit::Point
    } else {
        YGUnit::Auto
    } as u8);
    style.base.set_height_units_value_value(YGUnit::Auto as u8);
    style.base.set_flex_direction_value_value(if row {
        YGFlexDirection::Row
    } else {
        YGFlexDirection::Column
    } as u8);
    style
        .base
        .set_layout_alignment_type_value(LayoutAlignmentType::TopCenter as u8);
    style
        .base
        .set_intrinsically_sized_value_value(width.is_none());
    let style = arena.insert(style);
    owner
        .with_downcast_mut::<LayoutComponent, _>(|o| {
            o.base.set_width_value(width.unwrap_or(0.0));
            o.base.set_height_value(800.0);
            o.set_style(Some(style.clone()));
            o.add_layout_style_applier(owner.clone());
            o.add_layout_style_applier(style.clone());
        })
        .unwrap();
    owner
}
struct Fixture {
    arena: CoreArena,
    root: CoreHandle,
    label: CoreHandle,
    text: CoreHandle,
    run: CoreHandle,
}
impl Fixture {
    fn new(content: &str, offer: f32, row: bool) -> Self {
        Self::new_host(content, offer, row, false)
    }
    fn new_host(content: &str, offer: f32, row: bool, participant: bool) -> Self {
        let arena = CoreArena::default();
        let root = layout(&arena, None, Some(offer + 64.0), row);
        root.with_downcast_mut::<LayoutComponent, _>(|o| {
            o.with_style_mut(|s| {
                s.base.set_padding_left_value(32.0);
                s.base.set_padding_right_value(32.0);
                s.base
                    .set_padding_left_units_value_value(YGUnit::Point as u8);
                s.base
                    .set_padding_right_units_value_value(YGUnit::Point as u8);
            })
        })
        .unwrap();
        let (text, run) = make_text(&arena, content);
        let label = if participant {
            attach(&arena, &root, &text);
            let mut p = LayoutParticipant::default();
            p.base
                .set_layout_width_scale_type_value(LayoutScaleType::Hug as u8);
            p.base
                .set_layout_height_scale_type_value(LayoutScaleType::Hug as u8);
            p.base.set_width_units_value_value(YGUnit::Auto as u8);
            p.base.set_height_units_value_value(YGUnit::Auto as u8);
            let p = arena.insert(p);
            attach(&arena, &text, &p);
            let mut context = ParentContext {
                arena: &arena,
                parent: text.clone(),
            };
            assert_eq!(
                LayoutParticipant::on_added_clean_occurrence(&p, &mut context),
                StatusCode::Ok
            );
            p
        } else {
            let label = layout(&arena, Some(&root), None, false);
            attach(&arena, &label, &text);
            LayoutComponent::sync_style_occurrence(&label);
            LayoutComponent::sync_layout_children_occurrence(&label);
            label
        };
        LayoutComponent::sync_style_occurrence(&root);
        LayoutComponent::sync_layout_children_occurrence(&root);
        Self {
            arena,
            root,
            label,
            text,
            run,
        }
    }
    fn solve(&self) -> Layout {
        LayoutComponent::calculate_layout_occurrence(&self.root, f32::NAN, f32::NAN);
        self.label
            .with(|o| {
                if let Some(layout) = o.as_layout_component() {
                    layout.layout_data.solved_layout
                } else {
                    o.as_any()
                        .downcast_ref::<LayoutParticipant>()
                        .unwrap()
                        .native_layout_data()
                        .unwrap()
                        .solved_layout
                }
            })
            .unwrap()
    }
}
fn assert_width(f: &Fixture, expected: f32) {
    let bounds = f.solve();
    assert!(
        (bounds.width() - expected).abs() < 0.001,
        "{bounds:?} != width {expected}"
    );
}
const TITLE: &str = "Cross-plane experience is active";
// Pinned Inter_28pt-Bold at 28px and line-height 36. These literals were
// measured by calling Text::measure_layout directly with Undefined on both
// axes, outside the layout adapter: Unlock premium = 214.29297,
// experience (the longest word) = 145.4414.
#[test]
fn css_fit_content_title_uses_the_full_column_offer() {
    let f = Fixture::new(TITLE, 338.0, false);
    assert_width(&f, 338.0);
    assert_eq!(f.solve().left(), 32.0);
}
#[test]
fn css_fit_content_one_line_title_keeps_its_natural_width() {
    assert_width(&Fixture::new("Unlock premium", 338.0, false), 214.29297);
}
#[test]
fn css_fit_content_narrow_offer_floors_at_the_longest_word() {
    assert_width(&Fixture::new("experience", 338.0, false), 145.4414);
    assert_width(&Fixture::new(TITLE, 40.0, false), 145.4414);
}
/// A CSS box (a LayoutCssStyle record, so the style's Hug is ignored)
/// measures its text: `width: auto` in a centred column is fit-content, so
/// a short title keeps its natural width and a long one takes the offer and
/// wraps (docs/CSS_LAYOUT.md, 5).
#[test]
fn css_mode_box_sizes_to_its_text() {
    for (content, expected) in [("Unlock premium", 214.29297), (TITLE, 338.0)] {
        let f = Fixture::new(content, 338.0, false);
        f.label
            .with_downcast_mut::<LayoutComponent, _>(|o| {
                o.with_style_mut(|s| s.base.set_intrinsically_sized_value_value(false))
            })
            .unwrap();
        let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
        attach(&f.arena, &f.label, &record);
        f.label
            .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(record.clone()))
            .unwrap();
        LayoutComponent::sync_style_occurrence(&f.label);
        assert!(
            f.label
                .with_downcast::<LayoutComponent, _>(LayoutComponent::is_css_mode)
                .unwrap()
        );
        assert_width(&f, expected);
    }
}
/// A CSS box gives its text its content box, where Taffy measured it:
/// `width: 100px; padding: 10px` lays the text out 100 wide, not 120.
#[test]
fn css_mode_text_takes_the_content_box() {
    use crate::source::generated::core_registry::CoreRegistry;
    let f = Fixture::new("Unlock premium", 338.0, false);
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.label, &record);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(record.clone()))
        .unwrap();
    // width: 100px, and 10 points of padding on every side.
    assert!(CoreRegistry::set_uint_handle(&record, 61021, 1));
    assert!(CoreRegistry::set_double_handle(&record, 61022, 100.0));
    for key in [61063, 61065, 61067, 61069] {
        assert!(CoreRegistry::set_double_handle(&record, key, 10.0));
    }
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_eq!(f.solve().width(), 120.0);
    LayoutComponent::update_layout_bounds_occurrence(&f.root, false);
    let text = f
        .text
        .with_downcast::<Text, _>(|text| (text.effective_width(), text.effective_height()));
    let height = f.solve().height() - 20.0;
    assert_eq!(text, Some((100.0, height)));
    // And it is placed at the content box's origin, past the padding.
    let offset = f.text.with(|object| {
        let offset = object
            .as_transform_component()
            .expect("a transform")
            .layout_content_offset();
        (offset.x, offset.y)
    });
    assert_eq!(offset, Some((10.0, 10.0)));
}
/// A CSS box measures by its record alone: the Rive style's hugUnbounded
/// does not unbind its width, so the title still wraps at the offer.
#[test]
fn css_mode_ignores_a_legacy_hug_unbounded() {
    let f = Fixture::new(TITLE, 338.0, false);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| {
            o.with_style_mut(|s| s.base.set_hug_unbounded_value(true))
        })
        .unwrap();
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.label, &record);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(record.clone()))
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_width(&f, 338.0);
}
/// Padding written at run time narrows a CSS box's text, though the box
/// keeps its border-box size.
#[test]
fn css_mode_text_follows_padding_in_a_fixed_box() {
    use crate::source::generated::core_registry::CoreRegistry;
    let f = Fixture::new("Unlock premium", 338.0, false);
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.label, &record);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(record.clone()))
        .unwrap();
    // box-sizing: border-box; width: 200px.
    assert!(CoreRegistry::set_uint_handle(&record, 61001, 1));
    assert!(CoreRegistry::set_uint_handle(&record, 61021, 1));
    assert!(CoreRegistry::set_double_handle(&record, 61022, 200.0));
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_eq!(f.solve().width(), 200.0);
    LayoutComponent::update_layout_bounds_occurrence(&f.root, false);
    let width = || {
        f.text
            .with_downcast::<Text, _>(Text::effective_width)
            .unwrap()
    };
    assert_eq!(width(), 200.0);
    // padding-left: 30px.
    assert!(CoreRegistry::set_double_handle(&record, 61069, 30.0));
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_eq!(f.solve().width(), 200.0);
    LayoutComponent::update_layout_bounds_occurrence(&f.root, false);
    assert_eq!(width(), 170.0);
}
/// A participant whose host has a record measures by its CSS: the Rive
/// hugUnbounded does not unbind its width, so the title wraps at the offer.
#[test]
fn css_participant_ignores_a_legacy_hug_unbounded() {
    let f = Fixture::new_host(TITLE, 338.0, false, true);
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.text, &record);
    f.label
        .with_downcast_mut::<LayoutParticipant, _>(|p| {
            p.base.set_hug_unbounded_value(true);
            p.add_layout_style_applier(record.clone());
        })
        .unwrap();
    LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
    assert_width(&f, 338.0);
}

/// A CSS leaf artboard gives its text its content box, as a CSS box does.
#[test]
fn css_leaf_artboard_gives_its_text_the_content_box() {
    use crate::source::artboard::Artboard;
    let arena = CoreArena::default();
    let artboard = arena.insert(Artboard::default());
    let style = arena.insert(LayoutComponentStyle::default());
    let record = arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&arena, &artboard, &record);
    artboard
        .with_downcast_mut::<Artboard, _>(|a| {
            let layout = &mut a.base.base;
            layout.set_style(Some(style.clone()));
            layout.add_layout_style_applier(artboard.clone());
            layout.add_layout_style_applier(style.clone());
            layout.add_layout_style_applier(record.clone());
        })
        .unwrap();
    let (text, _run) = make_text(&arena, "Unlock premium");
    attach(&arena, &artboard, &text);
    artboard
        .with_downcast_mut::<Artboard, _>(|a| {
            let layout = &mut a.base.base;
            layout.layout = Layout::new(0.0, 0.0, 200.0, 100.0);
            layout.solved_padding = LayoutPadding::new(10.0, 0.0, 20.0, 0.0);
            layout.layout_padding = layout.solved_padding;
        })
        .unwrap();
    LayoutComponent::propagate_size_occurrence(&artboard);
    let width = text
        .with_downcast::<Text, _>(Text::effective_width)
        .unwrap();
    assert_eq!(width, 170.0);
}
/// A CSS box measures its text by CSS whatever its Rive style's sizing
/// modes: after layout gives the text its size, the next solve still
/// wraps the title at the offer, with the same height.
#[test]
fn css_mode_text_remeasures_by_css_after_its_first_layout() {
    let f = Fixture::new(TITLE, 338.0, false);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| {
            o.with_style_mut(|s| {
                s.base
                    .set_layout_width_scale_type_value(LayoutScaleType::Fixed as u8);
                s.base
                    .set_layout_height_scale_type_value(LayoutScaleType::Fixed as u8);
                s.base.set_intrinsically_sized_value_value(false);
            })
        })
        .unwrap();
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.label, &record);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(record.clone()))
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    let first = f.solve();
    assert_eq!(first.width(), 338.0);
    LayoutComponent::update_layout_bounds_occurrence(&f.root, false);
    // Lay out again from scratch.
    LayoutComponent::mark_layout_node_dirty_occurrence(&f.label, true);
    LayoutComponent::sync_style_occurrence(&f.label);
    let second = f.solve();
    assert_eq!(
        (second.width(), second.height()),
        (first.width(), first.height())
    );
}
/// A participant whose host has a record lays its text out at its CSS
/// width: AutoHeight text authored 20 wide, with `width: 338px` on its
/// host, wraps at 338 as a CSS box's text does.
#[test]
fn css_participant_text_wraps_at_its_css_width() {
    use crate::source::generated::core_registry::CoreRegistry;
    let f = Fixture::new_host(TITLE, 338.0, false, true);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.base.set_sizing_value_value(1);
            text.base.set_width_value(20.0);
        })
        .unwrap();
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.text, &record);
    assert!(CoreRegistry::set_uint_handle(&record, 61021, 1));
    assert!(CoreRegistry::set_double_handle(&record, 61022, 338.0));
    f.label
        .with_downcast_mut::<LayoutParticipant, _>(|p| p.add_layout_style_applier(record.clone()))
        .unwrap();
    LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
    let solved = f.solve();
    // Two lines at 338, as the CSS box's title above.
    assert_eq!((solved.width(), solved.height()), (338.0, 70.30176));
}

/// A CSS leaf's minimum is its contents' largest: text with hidden overflow
/// adds nothing, and the other text still holds the box at its longest
/// word in a narrow offer.
#[test]
fn css_leaf_minimum_skips_only_the_contents_without_one() {
    let f = Fixture::new("Unlock premium", 40.0, false);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.base.set_overflow_value_value(1);
        })
        .unwrap();
    let (second, _run) = make_text(&f.arena, TITLE);
    attach(&f.arena, &f.label, &second);
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.label, &record);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(record.clone()))
        .unwrap();
    LayoutComponent::sync_layout_children_occurrence(&f.label);
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_width(&f, 145.4414);
}
/// Text with hidden overflow adds nothing to a CSS box's minimum height
/// either: in a 50-point column the box shrinks past its wrapped title.
#[test]
fn css_leaf_minimum_height_skips_hidden_text() {
    use crate::source::generated::core_registry::CoreRegistry;
    let f = Fixture::new(TITLE, 338.0, false);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.base.set_overflow_value_value(1);
        })
        .unwrap();
    // The root: a CSS column 50 points tall; the label: a CSS box.
    let column = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.root, &column);
    assert!(CoreRegistry::set_uint_handle(&column, 61000, 1));
    assert!(CoreRegistry::set_uint_handle(&column, 61006, 2));
    assert!(CoreRegistry::set_uint_handle(&column, 61024, 1));
    assert!(CoreRegistry::set_double_handle(&column, 61025, 50.0));
    f.root
        .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(column.clone()))
        .unwrap();
    let record = f.arena.insert(crate::css_layout::LayoutCssStyle::default());
    attach(&f.arena, &f.label, &record);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|o| o.add_layout_style_applier(record.clone()))
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.root);
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_eq!(f.solve().height(), 50.0);
}
#[test]
fn css_fit_content_only_child_uses_the_row_offer() {
    assert_width(&Fixture::new(TITLE, 338.0, true), 338.0);
}
#[test]
fn css_fit_content_participant_uses_the_same_text_minimum_and_offer() {
    for (offer, expected) in [(338.0, 338.0), (40.0, 145.4414)] {
        assert_width(&Fixture::new_host(TITLE, offer, false, true), expected);
    }
}

#[test]
fn css_fit_content_retained_tree_tracks_text_changes_and_resizing() {
    if let Some(report) = std::env::var_os("NUXIE_LAYOUT_VERIFY") {
        assert!(
            cfg!(debug_assertions),
            "fresh-solve verification needs debug assertions"
        );
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(report)
            .expect("fresh-solve report must be writable");
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(std::env::var_os("NUXIE_LAYOUT_VERIFY").unwrap())
            .unwrap();
        writeln!(file, "verification enabled").unwrap();
    }
    for participant in [false, true] {
        let f = Fixture::new_host(TITLE, 338.0, false, participant);
        assert_width(&f, 338.0);
        // A settled second solve must retain the same result.
        assert_width(&f, 338.0);
        f.run
            .with_downcast_mut::<TextValueRun, _>(|run| {
                run.set_bound_text("Unlock premium".into());
            })
            .unwrap();
        assert_width(&f, 214.29297);
        f.run
            .with_downcast_mut::<TextValueRun, _>(|run| {
                run.set_bound_text(TITLE.into());
            })
            .unwrap();
        f.root
            .with_downcast_mut::<LayoutComponent, _>(|root| {
                root.base.set_width_value(264.0);
            })
            .unwrap();
        LayoutComponent::sync_style_occurrence(&f.root);
        assert_width(&f, 200.0);
        let expected_height = f
            .text
            .with_downcast_mut::<Text, _>(|text| {
                text.measure_layout(
                    200.0,
                    LayoutMeasureMode::Exactly,
                    f32::NAN,
                    LayoutMeasureMode::Undefined,
                )
                .y
            })
            .unwrap();
        assert!((f.solve().height() - expected_height).abs() < 0.001);
        f.root
            .with_downcast_mut::<LayoutComponent, _>(|root| {
                root.base.set_width_value(104.0);
            })
            .unwrap();
        LayoutComponent::sync_style_occurrence(&f.root);
        assert_width(&f, 145.4414);
    }
    if let Some(report) = std::env::var_os("NUXIE_LAYOUT_VERIFY") {
        let contents = std::fs::read_to_string(&report).unwrap();
        assert!(!contents.contains("mismatch:"), "{contents}");
        if let Some(outputs) = std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR") {
            let source = std::path::PathBuf::from(report);
            let destination = std::path::PathBuf::from(outputs).join("layout-verify.jsonl");
            if source.canonicalize().ok() != destination.canonicalize().ok() {
                std::fs::copy(source, destination).unwrap();
            }
        }
    }
}

#[test]
fn css_fit_content_does_not_change_wrapping_in_a_fixed_inline_slot() {
    let f = Fixture::new(TITLE, 338.0, false);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|label| {
            label.base.set_width_value(40.0);
            label.with_style_mut(|style| {
                style
                    .base
                    .set_layout_width_scale_type_value(LayoutScaleType::Fixed as u8);
                style.base.set_width_units_value_value(YGUnit::Point as u8);
            });
        })
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_width(&f, 40.0);
    // The fixed slot still uses upstream emergency word wrapping at 40px.
    assert!((f.solve().height() - 502.30176).abs() < 0.001);
}

#[test]
fn css_fit_content_column_max_width_controls_wrapped_height() {
    // Three 145.4414px words joined without a break make one 436.3242px run.
    // At max-width 260, BreakWord draws two 36px lines. Rive's unchanged
    // baseline metrics trim 1.69824px, giving 2 * 36 - 1.69824 = 70.30176.
    let f = Fixture::new("experienceexperienceexperience", 200.0, false);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|label| {
            label.with_style_mut(|style| {
                style.base.set_max_width_value(260.0);
                style
                    .base
                    .set_max_width_units_value_value(YGUnit::Point as u8);
            });
        })
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    let sibling = layout(&f.arena, Some(&f.root), Some(20.0), false);
    LayoutComponent::sync_style_occurrence(&sibling);
    LayoutComponent::sync_layout_children_occurrence(&f.root);
    let bounds = f.solve();
    assert!((bounds.width() - 260.0).abs() < 0.001, "{bounds:?}");
    assert!((bounds.height() - 70.30176).abs() < 0.001, "{bounds:?}");
    assert!(
        sibling
            .with_downcast::<LayoutComponent, _>(|s| s.layout_data.solved_layout.top())
            .unwrap()
            >= bounds.top() + bounds.height()
    );
}

#[test]
fn css_fit_content_participant_paint_frames_reuse_shaping() {
    use crate::source::component::ComponentDirt;
    for participant in [false, true] {
        let before = HbFont::test_shape_count();
        let f = Fixture::new_host(TITLE, 338.0, false, participant);
        assert_width(&f, 338.0);
        let initial = HbFont::test_shape_count() - before;
        f.text
            .with_downcast_mut::<Text, _>(|text| {
                text.update_after_transform_super(ComponentDirt::PATH);
            })
            .unwrap();
        f.solve();
        let settled = HbFont::test_shape_count();
        for _ in 0..4 {
            let measures = host_measure_count();
            f.text
                .with_downcast_mut::<Text, _>(|text| {
                    text.mark_paint_dirty();
                    text.update_after_transform_super(ComponentDirt::PAINT);
                })
                .unwrap();
            assert_width(&f, 338.0);
            // Upstream 5551696a: unchanged paint-only frames owe no layout
            // measurement. Size-affecting paragraph/origin setters now dirty
            // layout explicitly; unchanged glyph inputs still reuse shaping.
            assert_eq!(
                host_measure_count(),
                measures,
                "unchanged paint-only frames must not remeasure"
            );
        }
        let paint = HbFont::test_shape_count() - settled;
        assert_eq!(paint, 0, "paint-only frames must reuse shaping");
        assert_eq!(
            initial, 1,
            "min-content must reuse the unbounded measure's shape"
        );
    }
}

fn natural_size(f: &Fixture) -> Vec2D {
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.measure_layout(
                f32::NAN,
                LayoutMeasureMode::Undefined,
                f32::NAN,
                LayoutMeasureMode::Undefined,
            )
        })
        .unwrap()
}

#[test]
fn css_fit_content_participant_width_follows_text_changes() {
    let f = Fixture::new_host("Unlock premium", 338.0, false, true);
    assert_width(&f, 214.29297);
    for (content, width) in [
        (TITLE, 338.0),
        ("experience", 145.4414),
        ("Unlock premium", 214.29297),
    ] {
        f.run
            .with_downcast_mut::<TextValueRun, _>(|run| {
                run.set_bound_text(content.into());
            })
            .unwrap();
        assert_width(&f, width);
    }
}

#[test]
fn css_fit_content_break_all_uses_cluster_minimum() {
    let control = Fixture::new("x", 338.0, false);
    let cluster = natural_size(&control).x;
    let f = Fixture::new("xxx", 1.0, false);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.base.set_word_break_value_value(2); // BreakAll
        })
        .unwrap();
    assert_width(&f, cluster);
    assert!((f.solve().height() - 106.30176).abs() < 0.001);
}

#[test]
fn css_fit_content_no_wrap_keeps_max_content() {
    let f = Fixture::new(TITLE, 40.0, false);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.base.set_wrap_value_value(1); // NoWrap
        })
        .unwrap();
    let natural = natural_size(&f);
    assert!(natural.x > 338.0);
    assert_width(&f, natural.x);
    assert!((f.solve().height() - 34.301754).abs() < 0.001);
}

#[test]
fn css_fit_content_several_paragraphs_use_the_widest_intrinsic_run() {
    let f = Fixture::new("Unlock premium\nexperience", 0.0, false);
    assert_width(&f, 145.4414);
    // Three 36px lines share one initial metric trim: 108 - 1.69824.
    assert!((f.solve().height() - 106.30176).abs() < 0.001);
}

#[test]
fn css_fit_content_letter_spacing_is_included_in_the_minimum() {
    let f = Fixture::new("experience", 0.0, false);
    let style = f
        .run
        .with_downcast::<TextValueRun, _>(|run| run.style())
        .flatten()
        .unwrap();
    style
        .with_downcast_mut::<TextStyle, _>(|style| {
            style.base.set_letter_spacing_value(2.0);
        })
        .unwrap();
    // Ten glyph advances include 2px each: 145.4414 + 10 * 2.
    assert!((natural_size(&f).x - 165.4414).abs() < 0.001);
    assert_width(&f, 165.4414);
}

#[test]
fn css_fit_content_zero_offer_uses_word_minimum() {
    assert_width(&Fixture::new(TITLE, 0.0, false), 145.4414);
}

#[test]
fn css_fit_content_max_width_subtracts_leaf_padding_before_wrapping() {
    let f = Fixture::new("experienceexperienceexperience", 200.0, false);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|label| {
            label.with_style_mut(|style| {
                style.base.set_max_width_value(260.0);
                style
                    .base
                    .set_max_width_units_value_value(YGUnit::Point as u8);
                style.base.set_padding_left_value(30.0);
                style.base.set_padding_right_value(30.0);
                style.base.set_padding_top_value(4.0);
                style.base.set_padding_bottom_value(6.0);
                style
                    .base
                    .set_padding_left_units_value_value(YGUnit::Point as u8);
                style
                    .base
                    .set_padding_right_units_value_value(YGUnit::Point as u8);
                style
                    .base
                    .set_padding_top_units_value_value(YGUnit::Point as u8);
                style
                    .base
                    .set_padding_bottom_units_value_value(YGUnit::Point as u8);
            });
        })
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_width(&f, 260.0);
    // 260 - 60 padding leaves 200 for three lines, plus 10 vertical padding.
    assert!(
        (f.solve().height() - 116.30176).abs() < 0.001,
        "{:?}",
        f.solve()
    );
}

#[test]
fn css_fit_content_resolves_percentage_max_width() {
    let f = Fixture::new("experienceexperienceexperience", 200.0, false);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|label| {
            label.with_style_mut(|style| {
                style.base.set_max_width_value(75.0);
                style
                    .base
                    .set_max_width_units_value_value(YGUnit::Percent as u8);
            });
        })
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_width(&f, 150.0); // 75% of the 200px parent content box.
    assert!((f.solve().height() - 106.30176).abs() < 0.001);
}

#[test]
fn css_fit_content_min_width_wins_over_max_width() {
    let f = Fixture::new("experienceexperienceexperience", 200.0, false);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|label| {
            label.with_style_mut(|style| {
                style.base.set_max_width_value(260.0);
                style
                    .base
                    .set_max_width_units_value_value(YGUnit::Point as u8);
                style.base.set_min_width_value(500.0);
                style
                    .base
                    .set_min_width_units_value_value(YGUnit::Point as u8);
            });
        })
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    assert_width(&f, 500.0);
    assert!((f.solve().height() - 34.301754).abs() < 0.001);
}

#[test]
fn css_fit_content_keeps_direct_rive_measure_answers() {
    let f = Fixture::new(TITLE, 338.0, false);
    for mode in [LayoutMeasureMode::AtMost, LayoutMeasureMode::Exactly] {
        let measured = f
            .text
            .with_downcast_mut::<Text, _>(|text| {
                text.measure_layout(338.0, mode, f32::NAN, LayoutMeasureMode::Undefined)
            })
            .unwrap();
        assert!((measured.x - 313.26367).abs() < 0.001);
        assert!((measured.y - 70.30176).abs() < 0.001);
    }
    let zero = f
        .text
        .with_downcast_mut::<Text, _>(|text| {
            text.measure_layout(
                0.0,
                LayoutMeasureMode::AtMost,
                f32::NAN,
                LayoutMeasureMode::Undefined,
            )
        })
        .unwrap();
    assert_eq!(zero.x, 0.0);
}

// These properties publish PAINT dirt upstream, but both affect measured height.
#[test]
fn css_fit_content_participant_paragraph_spacing_publishes_height() {
    use crate::source::component::ComponentDirt;
    let f = Fixture::new_host("experience\nexperience", 338.0, false, true);
    let before = f.solve();
    assert!((before.height() - 70.30176).abs() < 0.001, "{before:?}");
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.update_after_transform_super(ComponentDirt::PATH);
        })
        .unwrap();
    f.solve();
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            assert!(text.base.set_paragraph_spacing_value(20.0));
            text.paragraph_spacing_changed();
            text.update_after_transform_super(ComponentDirt::PAINT);
        })
        .unwrap();
    let after = f.solve();
    // One inter-paragraph gap adds 20px to two 36px lines, minus the
    // same 1.69824px font metric trim as the initial measure.
    assert!((after.height() - 90.30176).abs() < 0.001, "{after:?}");
}

#[test]
fn css_fit_content_participant_baseline_origin_publishes_height() {
    use crate::source::component::ComponentDirt;
    let f = Fixture::new_host("experience\nexperience", 338.0, false, true);
    assert!((f.solve().height() - 70.30176).abs() < 0.001);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.update_after_transform_super(ComponentDirt::PATH);
        })
        .unwrap();
    f.solve();
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            assert!(text.base.set_origin_value_value(1));
            text.origin_value_changed();
            text.update_after_transform_super(ComponentDirt::PAINT);
        })
        .unwrap();
    // Direct Rive measurement supplies the unchanged Baseline metrics;
    // the regression concerns publishing them through the retained participant.
    let expected = natural_size(&f).y;
    // First baseline is 27.125px: 70.30176 - 27.125 = 43.176754.
    assert!((expected - 43.176754).abs() < 0.001);
    let after = f.solve();
    assert!(
        (after.height() - expected).abs() < 0.001,
        "{after:?} != {expected}"
    );
}

#[test]
fn css_fit_content_fallback_font_keeps_the_full_offer() {
    use crate::source::text_engine::{Font, FontRef, fallback_calls, with_host_fallback_proc};
    thread_local! {
        static FONT: std::cell::RefCell<Option<FontRef>> = const { std::cell::RefCell::new(None) };
    }
    fn pick(missing: u32, index: u32, _: &dyn Font) -> Option<FontRef> {
        (index == 0 && missing == 0x3297)
            .then(|| FONT.with(|slot| slot.borrow().clone()))
            .flatten()
    }
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            FONT.with(|slot| slot.replace(None));
        }
    }
    let root = std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").unwrap());
    let primary = HbFont::decode(
        &std::fs::read(root.join("tests/unit_tests/assets/fonts/Inter_28pt-Bold.ttf")).unwrap(),
    )
    .unwrap();
    let fallback = HbFont::decode(
        &std::fs::read(root.join("tests/unit_tests/assets/TwemojiMozilla.subset.ttf")).unwrap(),
    )
    .unwrap();
    // U+3297 is a color glyph in the pinned emoji subset, absent from Inter.
    assert!(!primary.has_glyph(0x3297));
    assert!(fallback.has_glyph(0x3297));
    FONT.with(|slot| slot.replace(Some(fallback)));
    let _restore = Restore;
    with_host_fallback_proc(pick, || {
        for participant in [false, true] {
            let f = Fixture::new_host(
                "Cross-plane ㊗ experience is active",
                338.0,
                false,
                participant,
            );
            let before = fallback_calls();
            assert!(natural_size(&f).x > 338.0);
            assert!(
                fallback_calls() > before,
                "the title must exercise fallback shaping"
            );
            // The widest word is experience, 145.4414px, and max-content
            // exceeds 338px. CSS min(max-content, max(min-content, 338)) = 338.
            assert_width(&f, 338.0);
        }
    });
}

// UNIV-4844: CSS flex arithmetic, exercised through real text and layout owners.
fn row_icon(f: &Fixture, width: f32, minimum: f32, gap: f32) -> CoreHandle {
    let icon = layout(&f.arena, Some(&f.root), Some(width), false);
    // The icon has intrinsic content. A fixed child supplies its automatic
    // minimum without imposing an explicit minimum on the flex item itself.
    let content = layout(&f.arena, Some(&icon), Some(minimum), false);
    LayoutComponent::sync_style_occurrence(&content);
    LayoutComponent::sync_style_occurrence(&icon);
    LayoutComponent::sync_layout_children_occurrence(&icon);
    f.root
        .with_downcast_mut::<LayoutComponent, _>(|root| {
            root.with_style_mut(|s| {
                s.base.set_gap_horizontal_value(gap);
                s.base
                    .set_gap_horizontal_units_value_value(YGUnit::Point as u8);
            });
        })
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.root);
    LayoutComponent::sync_layout_children_occurrence(&f.root);
    icon
}
fn solved(owner: &CoreHandle) -> Layout {
    owner
        .with(|o| {
            if let Some(l) = o.as_layout_component() {
                l.layout_data.solved_layout
            } else {
                o.as_any()
                    .downcast_ref::<LayoutParticipant>()
                    .unwrap()
                    .native_layout_data()
                    .unwrap()
                    .solved_layout
            }
        })
        .unwrap()
}
fn add_label(f: &Fixture, content: &str, participant: bool) -> CoreHandle {
    let style = f
        .run
        .with_downcast::<TextValueRun, _>(|r| r.style())
        .flatten()
        .unwrap();
    let mut run = TextValueRun::default();
    run.base.set_text_value(content.into());
    run.set_style(style);
    let run = f.arena.insert(run);
    let mut text = Text::default();
    text.add_run(run.clone());
    let text = f.arena.insert(text);
    attach(&f.arena, &text, &run);
    let label = if participant {
        attach(&f.arena, &f.root, &text);
        let mut p = LayoutParticipant::default();
        p.base
            .set_layout_width_scale_type_value(LayoutScaleType::Hug as u8);
        p.base
            .set_layout_height_scale_type_value(LayoutScaleType::Hug as u8);
        p.base.set_width_units_value_value(YGUnit::Auto as u8);
        p.base.set_height_units_value_value(YGUnit::Auto as u8);
        let p = f.arena.insert(p);
        attach(&f.arena, &text, &p);
        let mut context = ParentContext {
            arena: &f.arena,
            parent: text,
        };
        assert_eq!(
            LayoutParticipant::on_added_clean_occurrence(&p, &mut context),
            StatusCode::Ok
        );
        p
    } else {
        let label = layout(&f.arena, Some(&f.root), None, false);
        attach(&f.arena, &label, &text);
        LayoutComponent::sync_style_occurrence(&label);
        LayoutComponent::sync_layout_children_occurrence(&label);
        label
    };
    LayoutComponent::sync_layout_children_occurrence(&f.root);
    label
}
macro_rules! row_hosts {
    ($component:ident, $participant:ident, $body:expr) => {
        #[test]
        fn $component() {
            ($body)(false);
        }
        #[test]
        fn $participant() {
            ($body)(true);
        }
    };
}
row_hosts!(
    css_row_shrink_label_beside_icon,
    css_row_shrink_participant_beside_icon,
    |participant| {
        let f = Fixture::new_host(TITLE, 338.0, true, participant);
        row_icon(&f, 40.0, 40.0, 16.0);
        // 402 screen - 64 padding - 40 icon - 16 gap = 282.
        assert_width(&f, 282.0);
        let bounds = f.solve();
        // Two 36px lines, minus the pinned 1.69824px metric trim.
        assert!((bounds.height() - 70.30176).abs() < 0.001, "{bounds:?}");
    }
);
row_hosts!(
    css_row_shrink_short_label,
    css_row_shrink_participant_short_label,
    |participant| {
        let f = Fixture::new_host("Unlock premium", 338.0, true, participant);
        row_icon(&f, 40.0, 40.0, 16.0);
        assert_width(&f, 214.29297);
        assert!((f.solve().height() - 34.301754).abs() < 0.001);
    }
);
row_hosts!(
    css_row_shrink_longest_word_floor,
    css_row_shrink_participant_longest_word_floor,
    |participant| {
        let f = Fixture::new_host(TITLE, 160.0, true, participant);
        let icon = row_icon(&f, 40.0, 40.0, 16.0);
        // 104px remains, but min-width:auto preserves the 145.4414px word.
        assert_width(&f, 145.4414);
        assert!((solved(&icon).width() - 40.0).abs() < 0.001);
        assert!(f.solve().width() + solved(&icon).width() + 16.0 > 160.0);
    }
);
row_hosts!(
    css_row_shrink_weighted_hugs,
    css_row_shrink_participant_weighted_hugs,
    |participant| {
        let f = Fixture::new_host("x x x", 120.0, true, participant);
        let second = add_label(&f, "x x x x x", participant);
        // Independent unbounded Rive measurements give the flex bases. CSS
        // distributes negative free space in proportion to shrink * base size.
        let first_base = natural_size(&f).x;
        let control = Fixture::new("x x x x x", 338.0, false);
        let second_base = natural_size(&control).x;
        let expected = 120.0 * first_base / (first_base + second_base);
        assert!(first_base + second_base > 120.0);
        assert_width(&f, expected);
        assert!(
            (solved(&second).width() - (120.0 - expected)).abs() < 0.001,
            "{:?}",
            solved(&second)
        );
    }
);
row_hosts!(
    css_row_shrink_fixed_box_to_content_floor,
    css_row_shrink_participant_fixed_box_to_content_floor,
    |participant| {
        let f = Fixture::new_host("experience", 200.0, true, participant);
        let fixed = row_icon(&f, 200.0, 80.0, 0.0);
        assert_width(&f, 145.4414);
        // Its width is a flex basis, not a no-shrink promise. The 80px
        // automatic minimum then freezes it, so the pair overflows 200px.
        assert!(
            (solved(&fixed).width() - 80.0).abs() < 0.001,
            "{:?}",
            solved(&fixed)
        );
    }
);
row_hosts!(
    css_row_shrink_fill_keeps_fraction,
    css_row_shrink_participant_fill_keeps_fraction,
    |participant| {
        let f = Fixture::new_host("experience", 338.0, true, participant);
        f.label
            .with_mut(|o| {
                if let Some(l) = o.as_layout_component_mut() {
                    l.base.set_fractional_width_value(2.0);
                    l.with_style_mut(|s| {
                        s.base
                            .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    });
                } else {
                    let p = o.as_any_mut().downcast_mut::<LayoutParticipant>().unwrap();
                    p.base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    p.base.set_fractional_width_value(2.0);
                }
            })
            .unwrap();
        if participant {
            LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
        } else {
            LayoutComponent::sync_style_occurrence(&f.label);
        }
        row_icon(&f, 40.0, 40.0, 16.0);
        assert_width(&f, 282.0);
    }
);
row_hosts!(
    css_row_shrink_column_height_floor,
    css_row_shrink_participant_column_height_floor,
    |participant| {
        let f = Fixture::new_host("experience", 338.0, false, participant);
        f.root
            .with_downcast_mut::<LayoutComponent, _>(|r| {
                r.base.set_height_value(100.0);
                r.with_style_mut(|s| {
                    s.base
                        .set_layout_height_scale_type_value(LayoutScaleType::Fixed as u8);
                    s.base.set_height_units_value_value(YGUnit::Point as u8);
                });
            })
            .unwrap();
        let fixed = layout(&f.arena, Some(&f.root), Some(40.0), false);
        fixed
            .with_downcast_mut::<LayoutComponent, _>(|l| {
                l.base.set_height_value(100.0);
                l.with_style_mut(|s| {
                    s.base
                        .set_layout_height_scale_type_value(LayoutScaleType::Fixed as u8);
                    s.base.set_height_units_value_value(YGUnit::Point as u8);
                });
            })
            .unwrap();
        LayoutComponent::sync_style_occurrence(&fixed);
        LayoutComponent::sync_style_occurrence(&f.root);
        LayoutComponent::sync_layout_children_occurrence(&f.root);
        // The text freezes at its one-line content height; the empty fixed
        // box shrinks from its 100px basis into the remaining height.
        assert!(
            (f.solve().height() - 34.301754).abs() < 0.001,
            "{:?}",
            f.solve()
        );
        assert!(
            (solved(&fixed).height() - 65.69824).abs() < 0.001,
            "{:?}",
            solved(&fixed)
        );
    }
);

row_hosts!(
    css_row_shrink_fixed_text_uses_content_minimum,
    css_row_shrink_fixed_participant_uses_content_minimum,
    |participant| {
        let f = Fixture::new_host(TITLE, 200.0, true, participant);
        f.label
            .with_mut(|o| {
                if let Some(l) = o.as_layout_component_mut() {
                    l.base.set_width_value(200.0);
                    l.with_style_mut(|s| {
                        s.base
                            .set_layout_width_scale_type_value(LayoutScaleType::Fixed as u8);
                        s.base.set_width_units_value_value(YGUnit::Point as u8);
                    });
                } else {
                    let p = o.as_any_mut().downcast_mut::<LayoutParticipant>().unwrap();
                    p.base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fixed as u8);
                    p.base.set_width_units_value_value(YGUnit::Point as u8);
                    p.base.set_width_value(200.0);
                }
            })
            .unwrap();
        if participant {
            LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
        } else {
            LayoutComponent::sync_style_occurrence(&f.label);
        }
        row_icon(&f, 80.0, 80.0, 0.0);
        // min(200px specified width, 145.4414px min-content) is the automatic
        // floor. A fixed width must neither disable shrink nor force min-content.
        assert_width(&f, 145.4414);
    }
);

row_hosts!(
    css_row_shrink_fill_word_floor,
    css_row_shrink_participant_fill_word_floor,
    |participant| {
        let f = Fixture::new_host(TITLE, 160.0, true, participant);
        f.label
            .with_mut(|o| {
                if let Some(l) = o.as_layout_component_mut() {
                    l.base.set_fractional_width_value(2.0);
                    l.with_style_mut(|s| {
                        s.base
                            .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    });
                } else {
                    let p = o.as_any_mut().downcast_mut::<LayoutParticipant>().unwrap();
                    p.base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    p.base.set_fractional_width_value(2.0);
                }
            })
            .unwrap();
        if participant {
            LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
        } else {
            LayoutComponent::sync_style_occurrence(&f.label);
        }
        row_icon(&f, 40.0, 40.0, 16.0);
        assert_width(&f, 145.4414);
    }
);

#[test]
fn css_row_contract_width_owned_paragraph_wraps_for_every_authored_sizing() {
    for authored in [1, 2] {
        let f = Fixture::new_host(TITLE, 338.0, true, true);
        f.text
            .with_downcast_mut::<Text, _>(|text| {
                text.base.set_sizing_value_value(authored);
                text.base.set_width_value(1000.0);
                text.base.set_height_value(72.0);
            })
            .unwrap();
        f.label
            .with_downcast_mut::<LayoutParticipant, _>(|p| {
                p.base
                    .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
            })
            .unwrap();
        LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
        row_icon(&f, 40.0, 40.0, 16.0);
        assert_width(&f, 282.0);
        let expected_height = if authored == 1 { 70.30176 } else { 72.0 };
        assert!(
            (f.solve().height() - expected_height).abs() < 0.001,
            "authored sizing {authored}: {:?}",
            f.solve()
        );
    }
}

row_hosts!(
    css_row_contract_fitted_leaf_keeps_zero_minimum,
    css_row_contract_fitted_participant_keeps_zero_minimum,
    |participant| {
        use crate::source::text_engine::TextOverflow;
        let f = Fixture::new_host(TITLE, 160.0, true, participant);
        f.text
            .with_downcast_mut::<Text, _>(|text| {
                text.base.set_overflow_value_value(5);
                assert_eq!(text.overflow(), TextOverflow::FitFontSize);
            })
            .unwrap();
        row_icon(&f, 40.0, 40.0, 16.0);
        // Fitted fonts have no intrinsic minimum contract yet. Keep main's
        // zero minimum, so the label takes the 104px remaining in the row.
        assert_width(&f, 104.0);
    }
);

#[test]
fn css_scroll_viewport_keeps_its_slot_and_scroll_range() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for (overflow, infer_scroll) in [(1, false), (2, false), (0, true)] {
        let arena = CoreArena::default();
        let root = layout(&arena, None, Some(200.0), true);
        let viewport = layout(&arena, Some(&root), Some(200.0), false);
        viewport
            .with_downcast_mut::<LayoutComponent, _>(|v| {
                v.with_style_mut(|s| {
                    s.base.set_overflow_value_value(overflow);
                    s.base
                        .set_layout_alignment_type_value(LayoutAlignmentType::TopLeft as u8);
                });
            })
            .unwrap();
        let content = layout(&arena, Some(&viewport), Some(500.0), false);
        let scroll = arena.insert(ScrollConstraint::default());
        if infer_scroll {
            attach(&arena, &content, &scroll);
        }
        let icon = layout(&arena, Some(&root), Some(40.0), false);
        let icon_content = layout(&arena, Some(&icon), Some(40.0), false);
        for node in [&icon_content, &icon, &content, &viewport, &root] {
            LayoutComponent::sync_style_occurrence(node);
            LayoutComponent::sync_layout_children_occurrence(node);
        }
        LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
        LayoutComponent::update_layout_bounds_occurrence(&root, false);
        assert!(
            (solved(&viewport).width() - 160.0).abs() < 0.001,
            "overflow {overflow}, inferred {infer_scroll}: {:?}",
            solved(&viewport)
        );
        assert_eq!(solved(&content).width(), 500.0);
        if !infer_scroll {
            attach(&arena, &content, &scroll);
        }
        scroll
            .with_downcast::<ScrollConstraint, _>(|s| {
                assert_eq!(s.viewport_width(), 160.0);
                assert_eq!(s.content_width(), 500.0);
                assert_eq!(s.max_offset_x(), -340.0);
            })
            .unwrap();
    }
}

#[test]
fn css_draw_clip_keeps_the_word_minimum() {
    let f = Fixture::new("experience", 160.0, true);
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|l| {
            l.base.set_clip_value(true);
        })
        .unwrap();
    LayoutComponent::sync_style_occurrence(&f.label);
    row_icon(&f, 40.0, 40.0, 16.0);
    // Rive's draw clip is CSS overflow:clip, not a scroll container.
    assert_width(&f, 145.4414);
}

fn image_transfer_fixture(participant: bool, row: bool) -> (CoreArena, CoreHandle, CoreHandle) {
    use crate::source::shapes::image::Image;
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(200.0), row);
    root.with_downcast_mut::<LayoutComponent, _>(|r| {
        r.base.set_height_value(120.0);
        r.with_style_mut(|s| {
            s.base
                .set_layout_height_scale_type_value(LayoutScaleType::Fixed as u8);
            s.base.set_height_units_value_value(YGUnit::Point as u8);
        });
    })
    .unwrap();
    let mut image = Image::default();
    image.set_runtime_size(442.0, 412.0);
    let image = arena.insert(image);
    let subject = if participant {
        attach(&arena, &root, &image);
        let mut p = LayoutParticipant::default();
        p.base
            .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
        p.base
            .set_layout_height_scale_type_value(LayoutScaleType::Fill as u8);
        let p = arena.insert(p);
        attach(&arena, &image, &p);
        let mut context = ParentContext {
            arena: &arena,
            parent: image.clone(),
        };
        assert_eq!(
            LayoutParticipant::on_added_clean_occurrence(&p, &mut context),
            StatusCode::Ok
        );
        p
    } else {
        let subject = layout(&arena, Some(&root), None, false);
        attach(&arena, &subject, &image);
        subject
            .with_downcast_mut::<LayoutComponent, _>(|l| {
                l.with_style_mut(|s| {
                    s.base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    s.base
                        .set_layout_height_scale_type_value(LayoutScaleType::Fill as u8);
                });
            })
            .unwrap();
        LayoutComponent::sync_style_occurrence(&subject);
        LayoutComponent::sync_layout_children_occurrence(&subject);
        subject
    };
    let sibling = layout(&arena, Some(&root), Some(60.0), false);
    let content = layout(&arena, Some(&sibling), Some(60.0), false);
    if !row {
        for node in [&sibling, &content] {
            node.with_downcast_mut::<LayoutComponent, _>(|l| {
                l.base.set_height_value(40.0);
                l.with_style_mut(|s| {
                    s.base
                        .set_layout_height_scale_type_value(LayoutScaleType::Fixed as u8);
                    s.base.set_height_units_value_value(YGUnit::Point as u8);
                });
            })
            .unwrap();
        }
    }
    for node in [&content, &sibling, &root] {
        LayoutComponent::sync_style_occurrence(node);
        LayoutComponent::sync_layout_children_occurrence(node);
    }
    (arena, root, subject)
}
row_hosts!(
    css_image_transfers_the_known_cross_size,
    css_image_participant_transfers_the_known_cross_size,
    |participant| {
        for row in [true, false] {
            let (_arena, root, image) = image_transfer_fixture(participant, row);
            LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
            let bounds = solved(&image);
            if row {
                // min(442, 120 * 442 / 412) = 128.73787. The 140px
                // remaining after the 60px sibling is above that minimum.
                assert!((bounds.width() - 140.0).abs() < 0.001, "{bounds:?}");
                assert_eq!(bounds.height(), 120.0);
            } else {
                // min(412, 200 * 412 / 442) = 186.42534. This exceeds
                // the 80px remaining in the column and must overflow.
                assert!((bounds.height() - 186.42534).abs() < 0.001, "{bounds:?}");
                assert_eq!(bounds.width(), 200.0);
            }
        }
    }
);

row_hosts!(
    css_row_contract_cache_tracks_font_fitting,
    css_row_contract_participant_cache_tracks_font_fitting,
    |participant| {
        let f = Fixture::new_host(TITLE, 160.0, true, participant);
        row_icon(&f, 40.0, 40.0, 16.0);
        assert_width(&f, 145.4414);
        for (overflow, width) in [(5, 104.0), (0, 145.4414)] {
            f.text
                .with_downcast_mut::<Text, _>(|text| {
                    text.base.set_overflow_value_value(overflow);
                    text.overflow_value_changed();
                })
                .unwrap();
            assert_width(&f, width);
        }
    }
);

row_hosts!(
    css_row_truncating_text_has_zero_automatic_minimum,
    css_row_truncating_participant_has_zero_automatic_minimum,
    |participant| {
        let mut widths = Vec::new();
        for overflow in 1..=5 {
            let f = Fixture::new_host(TITLE, 160.0, true, participant);
            f.text
                .with_downcast_mut::<Text, _>(|text| {
                    text.base.set_overflow_value_value(overflow);
                    if overflow == 3 {
                        text.base.set_wrap_value_value(1);
                    }
                })
                .unwrap();
            f.label
                .with_mut(|object| {
                    if let Some(layout) = object.as_layout_component_mut() {
                        layout.with_style_mut(|style| {
                            style
                                .base
                                .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                        });
                    } else {
                        object
                            .as_any_mut()
                            .downcast_mut::<LayoutParticipant>()
                            .unwrap()
                            .base
                            .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    }
                })
                .unwrap();
            if participant {
                LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
            } else {
                LayoutComponent::sync_style_occurrence(&f.label);
            }
            row_icon(&f, 40.0, 40.0, 16.0);
            widths.push(f.solve().width());
        }
        // Hidden, Clipped, Ellipsis, Fit and FitFontSize all have the CSS
        // hidden-overflow zero minimum. The icon and gap leave 104 points.
        assert_eq!(widths, vec![104.0; 5]);
    }
);

row_hosts!(
    css_grid_keeps_unknown_leaf_contributions_through_flex,
    css_grid_keeps_unknown_participant_contributions_through_flex,
    |participant| {
        let f = Fixture::new_host("experience", 160.0, true, participant);
        f.text
            .with_downcast_mut::<Text, _>(|text| {
                text.base.set_overflow_value_value(5);
            })
            .unwrap();
        let grid = layout(&f.arena, None, Some(100.0), false);
        grid.with_downcast_mut::<LayoutComponent, _>(|layout| {
            layout.with_style_mut(|style| {
                style.base.set_layout_type_value_value(1);
            });
        })
        .unwrap();
        attach(&f.arena, &grid, &f.root);
        f.root
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.base.set_width_value(0.0);
                layout.with_style_mut(|style| {
                    style
                        .base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    style.base.set_width_units_value_value(YGUnit::Auto as u8);
                });
            })
            .unwrap();
        f.label
            .with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    layout.with_style_mut(|style| {
                        style
                            .base
                            .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    });
                } else {
                    object
                        .as_any_mut()
                        .downcast_mut::<LayoutParticipant>()
                        .unwrap()
                        .base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                }
            })
            .unwrap();
        if participant {
            LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
        } else {
            LayoutComponent::sync_style_occurrence(&f.label);
        }
        for node in [&f.root, &grid] {
            LayoutComponent::sync_style_occurrence(node);
            LayoutComponent::sync_layout_children_occurrence(node);
        }
        LayoutComponent::calculate_layout_occurrence(&grid, f32::NAN, f32::NAN);
        // The unknown fitted-text fallback stays max-content for grid track
        // contributions, even below a flex box. Include its 64-point padding.
        assert!(
            (solved(&f.root).width() - 209.4414).abs() < 0.001,
            "flex {:?}, text {:?}",
            solved(&f.root),
            solved(&f.label)
        );
        assert!((solved(&f.label).width() - 145.4414).abs() < 0.001);
    }
);

row_hosts!(
    css_image_minimum_uses_cross_limits_before_the_cross_size_is_known,
    css_image_participant_minimum_uses_cross_limits_before_the_cross_size_is_known,
    |participant| {
        use crate::source::generated::core_registry::CoreField as Field;
        let mut widths = Vec::new();
        for (minimum, maximum) in [(None, Some(120.0)), (Some(600.0), None)] {
            let (_arena, root, image) = image_transfer_fixture(participant, true);
            root.with_downcast_mut::<LayoutComponent, _>(|root| {
                root.base.set_height_value(0.0);
                root.with_style_mut(|style| {
                    style
                        .base
                        .set_layout_height_scale_type_value(LayoutScaleType::Hug as u8);
                    style.base.set_height_units_value_value(YGUnit::Auto as u8);
                });
            })
            .unwrap();
            let applier = if participant {
                image.clone()
            } else {
                image
                    .with(|object| object.as_layout_component().unwrap().style_handle())
                    .flatten()
                    .unwrap()
            };
            applier
                .with_mut(|object| {
                    object.set_uint(
                        Field::LayoutSizingStyleMinHeightUnitsValue,
                        if minimum.is_some() {
                            YGUnit::Point
                        } else {
                            YGUnit::Auto
                        } as u32,
                    );
                    object.set_double(Field::LayoutSizingStyleMinHeight, minimum.unwrap_or(0.0));
                    object.set_uint(
                        Field::LayoutSizingStyleMaxHeightUnitsValue,
                        if maximum.is_some() {
                            YGUnit::Point
                        } else {
                            YGUnit::Auto
                        } as u32,
                    );
                    object.set_double(Field::LayoutSizingStyleMaxHeight, maximum.unwrap_or(0.0));
                })
                .unwrap();
            if participant {
                LayoutParticipant::sync_style_changes_occurrence(&image, None);
            } else {
                LayoutComponent::sync_style_occurrence(&image);
            }
            LayoutComponent::sync_style_occurrence(&root);
            LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
            widths.push(solved(&image).width());
        }
        // max-height:120 transfers a 128.73787 minimum, below the 140 slot.
        // min-height:600 transfers 643.68933, which must overflow the row.
        for (actual, expected) in widths.iter().zip([140.0, 643.68933]) {
            assert!((actual - expected).abs() < 0.001, "{widths:?}");
        }
    }
);

row_hosts!(
    css_visible_authored_text_boxes_keep_their_width_minimum,
    css_visible_authored_text_participants_keep_their_width_minimum,
    |participant| {
        let mut widths = Vec::new();
        for authored in [1, 2] {
            for fill in [false, true] {
                let f = Fixture::new_host(TITLE, 60.0, true, participant);
                f.text
                    .with_downcast_mut::<Text, _>(|text| {
                        text.base.set_sizing_value_value(authored);
                        text.base.set_wrap_value_value(1);
                        text.base.set_width_value(80.0);
                        text.base.set_height_value(72.0);
                    })
                    .unwrap();
                if fill {
                    f.label
                        .with_mut(|object| {
                            if let Some(layout) = object.as_layout_component_mut() {
                                layout.with_style_mut(|style| {
                                    style.base.set_layout_width_scale_type_value(
                                        LayoutScaleType::Fill as u8,
                                    );
                                });
                            } else {
                                object
                                    .as_any_mut()
                                    .downcast_mut::<LayoutParticipant>()
                                    .unwrap()
                                    .base
                                    .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                            }
                        })
                        .unwrap();
                    if participant {
                        LayoutParticipant::sync_style_changes_occurrence(&f.label, None);
                    } else {
                        LayoutComponent::sync_style_occurrence(&f.label);
                    }
                }
                row_icon(&f, 20.0, 20.0, 0.0);
                widths.push(f.solve().width());
            }
        }
        // An unowned authored text box stays 80 wide. When a Fill participant
        // owns its width, NoWrap instead floors at the whole line, 427.35547.
        let expected = if participant {
            vec![80.0, 427.35547, 80.0, 427.35547]
        } else {
            vec![80.0; 4]
        };
        assert_eq!(widths, expected);
    }
);

// UNIV-4844: a leaf that does not hug is measured only for its automatic
// minimum. Fill keeps upstream's zero content basis, so equal Fill fractions
// still split the row equally whatever text each leaf holds.
#[test]
fn css_fill_text_leaves_keep_their_zero_basis() {
    let mut widths = Vec::new();
    for overflow in [0, 5] {
        let f = Fixture::new("x", 300.0, true);
        let mut leaves = vec![f.label.clone()];
        for content in ["x x x", "Cross plane"] {
            let label = add_label(&f, content, false);
            leaves.push(label);
        }
        for leaf in &leaves {
            leaf.with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.with_style_mut(|style| {
                    style
                        .base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
                    style.base.set_intrinsically_sized_value_value(false);
                });
                for child in layout.base.children().to_vec() {
                    child.with_downcast_mut::<Text, _>(|text| {
                        text.base.set_overflow_value_value(overflow);
                    });
                }
            })
            .unwrap();
            LayoutComponent::sync_style_occurrence(leaf);
        }
        LayoutComponent::sync_style_occurrence(&f.root);
        LayoutComponent::sync_layout_children_occurrence(&f.root);
        f.solve();
        widths.push(
            leaves
                .iter()
                .map(|leaf| solved(leaf).width())
                .collect::<Vec<_>>(),
        );
    }
    // 364 - 64 padding = 300 shared by three Fill leaves. Each automatic
    // minimum (one word, or zero for fitted text) is below 100.
    for row in &widths {
        for width in row {
            assert!((width - 100.0).abs() < 0.001, "{widths:?}");
        }
    }
}

// UNIV-4844 review: natural sizes of drawing content follow authored geometry.
#[derive(Clone, Copy, PartialEq)]
enum ShapeHost {
    HugBox,
    GroupedHugBox,
    Participant,
}
struct ShapeFixture {
    _arena: CoreArena,
    root: CoreHandle,
    subject: CoreHandle,
    path: CoreHandle,
}
fn shape_fixture(ellipse: bool, host: ShapeHost, rotation: f32) -> ShapeFixture {
    use crate::source::generated::core_registry::CoreField;
    use crate::source::math::mat2d::Mat2D;
    use crate::source::shapes::{ellipse::Ellipse, rectangle::Rectangle, shape::Shape};
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(300.0), true);
    let shape = arena.insert(Shape::default());
    let path = if ellipse {
        arena.insert(Ellipse::default())
    } else {
        arena.insert(Rectangle::default())
    };
    path.with_mut(|o| {
        o.set_double(CoreField::ParametricPathWidth, 80.0);
        o.set_double(CoreField::ParametricPathHeight, 80.0);
    })
    .unwrap();
    attach(&arena, &shape, &path);
    shape
        .with_downcast_mut::<Shape, _>(|s| s.add_path(path.clone()))
        .unwrap();
    // Stand in for the update pass that composes the authored rotation.
    path.with_mut(|o| {
        o.as_transform_component_mut()
            .unwrap()
            .with_transform_mut(|t| *t = Mat2D::from_rotation(rotation))
    })
    .unwrap();
    let subject = match host {
        ShapeHost::Participant => {
            attach(&arena, &root, &shape);
            let mut p = LayoutParticipant::default();
            p.base
                .set_layout_width_scale_type_value(LayoutScaleType::Hug as u8);
            p.base
                .set_layout_height_scale_type_value(LayoutScaleType::Hug as u8);
            p.base.set_width_units_value_value(YGUnit::Auto as u8);
            p.base.set_height_units_value_value(YGUnit::Auto as u8);
            let p = arena.insert(p);
            attach(&arena, &shape, &p);
            let mut context = ParentContext {
                arena: &arena,
                parent: shape.clone(),
            };
            assert_eq!(
                LayoutParticipant::on_added_clean_occurrence(&p, &mut context),
                StatusCode::Ok
            );
            p
        }
        ShapeHost::HugBox | ShapeHost::GroupedHugBox => {
            let subject = layout(&arena, Some(&root), None, false);
            if host == ShapeHost::GroupedHugBox {
                let group = arena.insert(crate::source::node::Node::default());
                attach(&arena, &subject, &group);
                attach(&arena, &group, &shape);
            } else {
                attach(&arena, &subject, &shape);
            }
            LayoutComponent::sync_style_occurrence(&subject);
            LayoutComponent::sync_layout_children_occurrence(&subject);
            subject
        }
    };
    LayoutComponent::sync_style_occurrence(&root);
    LayoutComponent::sync_layout_children_occurrence(&root);
    ShapeFixture {
        _arena: arena,
        root,
        subject,
        path,
    }
}
impl ShapeFixture {
    /// Solve, then publish bounds so layout controls the path's size.
    fn width(&self) -> f32 {
        LayoutComponent::calculate_layout_occurrence(&self.root, f32::NAN, f32::NAN);
        LayoutComponent::update_layout_bounds_occurrence(&self.root, false);
        solved(&self.subject).width()
    }
}

#[test]
fn css_rotated_curve_natural_size_follows_its_geometry() {
    let quarter = std::f32::consts::FRAC_PI_4;
    let mut widths = Vec::new();
    for host in [
        ShapeHost::HugBox,
        ShapeHost::GroupedHugBox,
        ShapeHost::Participant,
    ] {
        widths.push(shape_fixture(true, host, quarter).width());
    }
    // An 80-point circle rotated 45 degrees still spans 80 points, not the
    // 113 of its rotated bounding box. Rive draws the circle as four cubics,
    // which stray up to about 0.03 percent from the true circle.
    for width in &widths {
        assert!((width - 80.0).abs() < 0.05, "{widths:?}");
    }
    // A rotated square measures as upstream does: a hug box asks the path's
    // authored width, a participant its precise transformed geometry.
    let square = [ShapeHost::HugBox, ShapeHost::Participant]
        .map(|host| shape_fixture(false, host, quarter).width());
    assert!((square[0] - 80.0).abs() < 0.001, "{square:?}");
    assert!(
        (square[1] - 80.0 * std::f32::consts::SQRT_2).abs() < 0.001,
        "{square:?}"
    );
}

#[test]
fn css_authored_write_equal_to_a_controlled_size_is_kept() {
    use crate::source::generated::core_registry::CoreField;
    let f = shape_fixture(false, ShapeHost::HugBox, 0.0);
    let cap = |maximum: Option<f32>| {
        f.subject
            .with_downcast_mut::<LayoutComponent, _>(|l| {
                l.with_style_mut(|s| {
                    s.base.set_max_width_value(maximum.unwrap_or(0.0));
                    s.base
                        .set_max_width_units_value_value(if maximum.is_some() {
                            YGUnit::Point
                        } else {
                            YGUnit::Auto
                        } as u8);
                });
            })
            .unwrap();
        LayoutComponent::sync_style_occurrence(&f.subject);
    };
    let mut widths = vec![f.width()];
    cap(Some(40.0));
    widths.push(f.width());
    // Layout wrote 40 into the path; an authored write of the same 40 must
    // still become the natural size once the cap is gone.
    f.path
        .with_mut(|o| o.set_double(CoreField::ParametricPathWidth, 40.0))
        .unwrap();
    cap(None);
    widths.push(f.width());
    assert_eq!(widths, [80.0, 40.0, 40.0]);
}

#[test]
fn css_grouped_hosts_keep_their_origins() {
    use crate::source::generated::core_registry::CoreField;
    // Two 100-point hosts in one group, at origin x 0 and 1: one spans
    // [0, 100] and the other [-100, 0], so the group is 200 points wide.
    let widths = [false, true].map(|joystick| {
        let arena = CoreArena::default();
        let root = layout(&arena, None, Some(1000.0), true);
        let subject = layout(&arena, Some(&root), None, false);
        let group = arena.insert(crate::source::node::Node::default());
        attach(&arena, &subject, &group);
        for origin in [0.0, 1.0] {
            let host = if joystick {
                let host = arena.insert(crate::source::joystick::Joystick::default());
                host.with_mut(|o| {
                    o.set_double(CoreField::JoystickWidth, 100.0);
                    o.set_double(CoreField::JoystickHeight, 30.0);
                    o.set_double(CoreField::JoystickOriginX, origin);
                })
                .unwrap();
                host
            } else {
                let host = arena.insert(crate::source::shapes::image::Image::default());
                host.with_mut(|o| o.set_double(CoreField::ImageOriginX, origin))
                    .unwrap();
                host.with_downcast_mut::<crate::source::shapes::image::Image, _>(|image| {
                    image.set_runtime_size(100.0, 30.0)
                })
                .unwrap();
                host
            };
            attach(&arena, &group, &host);
        }
        for node in [&subject, &root] {
            LayoutComponent::sync_style_occurrence(node);
            LayoutComponent::sync_layout_children_occurrence(node);
        }
        LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
        LayoutComponent::update_layout_bounds_occurrence(&root, false);
        solved(&subject).width()
    });
    assert_eq!(widths, [200.0, 200.0]);
}

#[test]
fn css_grouped_joysticks_keep_their_positions() {
    use crate::source::generated::core_registry::CoreField;
    // A joystick has no transform: it draws at its position. Two 100-point
    // joysticks at x 0 and 100 make a 200-point group.
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(1000.0), true);
    let subject = layout(&arena, Some(&root), None, false);
    let group = arena.insert(crate::source::node::Node::default());
    attach(&arena, &subject, &group);
    for x in [0.0, 100.0] {
        let host = arena.insert(crate::source::joystick::Joystick::default());
        host.with_mut(|o| {
            o.set_double(CoreField::JoystickWidth, 100.0);
            o.set_double(CoreField::JoystickHeight, 30.0);
            o.set_double(CoreField::JoystickOriginX, 0.0);
            o.set_double(CoreField::JoystickPosX, x);
        })
        .unwrap();
        attach(&arena, &group, &host);
    }
    for node in [&subject, &root] {
        LayoutComponent::sync_style_occurrence(node);
        LayoutComponent::sync_layout_children_occurrence(node);
    }
    LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
    LayoutComponent::update_layout_bounds_occurrence(&root, false);
    assert_eq!(solved(&subject).width(), 200.0);
}

#[test]
fn css_empty_shape_adds_no_bounds_to_a_group() {
    use crate::source::generated::core_registry::CoreField;
    use crate::source::math::mat2d::Mat2D;
    use crate::source::shapes::{rectangle::Rectangle, shape::Shape};
    // An 80-point shape placed away from the group origin, beside a shape
    // with no paths: the group is 80 points wide, not 80 plus the gap.
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(1000.0), true);
    let subject = layout(&arena, Some(&root), None, false);
    let group = arena.insert(crate::source::node::Node::default());
    attach(&arena, &subject, &group);
    let shape = arena.insert(Shape::default());
    let path = arena.insert(Rectangle::default());
    path.with_mut(|o| {
        o.set_double(CoreField::ParametricPathWidth, 80.0);
        o.set_double(CoreField::ParametricPathHeight, 30.0);
    })
    .unwrap();
    attach(&arena, &shape, &path);
    shape
        .with_downcast_mut::<Shape, _>(|s| s.add_path(path.clone()))
        .unwrap();
    // Stand in for the update pass that composes the shape's position.
    shape
        .with_mut(|o| {
            o.as_transform_component_mut()
                .unwrap()
                .with_transform_mut(|t| *t = Mat2D::from_translate(140.0, 0.0))
        })
        .unwrap();
    attach(&arena, &group, &shape);
    let empty = arena.insert(Shape::default());
    attach(&arena, &group, &empty);
    for node in [&subject, &root] {
        LayoutComponent::sync_style_occurrence(node);
        LayoutComponent::sync_layout_children_occurrence(node);
    }
    LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
    LayoutComponent::update_layout_bounds_occurrence(&root, false);
    assert_eq!(solved(&subject).width(), 80.0);
}

#[test]
fn css_grouped_shape_paths_keep_intermediate_transforms() {
    use crate::source::generated::core_registry::CoreField;
    use crate::source::math::mat2d::Mat2D;
    use crate::source::shapes::{rectangle::Rectangle, shape::Shape};
    // A shape's path may sit under a node inside the shape. An 80-point
    // rectangle below a node scaled 2 on x draws 160 points wide.
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(1000.0), true);
    let subject = layout(&arena, Some(&root), None, false);
    let group = arena.insert(crate::source::node::Node::default());
    attach(&arena, &subject, &group);
    let shape = arena.insert(Shape::default());
    attach(&arena, &group, &shape);
    let node = arena.insert(crate::source::node::Node::default());
    attach(&arena, &shape, &node);
    // Stand in for the update pass that composes the node's scale.
    node.with_mut(|o| {
        o.as_transform_component_mut()
            .unwrap()
            .with_transform_mut(|t| {
                *t = Mat2D::identity();
                t.scale_by_values(2.0, 1.0);
            })
    })
    .unwrap();
    let path = arena.insert(Rectangle::default());
    path.with_mut(|o| {
        o.set_double(CoreField::ParametricPathWidth, 80.0);
        o.set_double(CoreField::ParametricPathHeight, 30.0);
    })
    .unwrap();
    attach(&arena, &node, &path);
    shape
        .with_downcast_mut::<Shape, _>(|s| s.add_path(path.clone()))
        .unwrap();
    for layout in [&subject, &root] {
        LayoutComponent::sync_style_occurrence(layout);
        LayoutComponent::sync_layout_children_occurrence(layout);
    }
    LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
    LayoutComponent::update_layout_bounds_occurrence(&root, false);
    assert_eq!(solved(&subject).width(), 160.0);
}

// Chrome's line boxes in the bundled Roboto font, through a real leaf owner.
fn line_box_fixture(css: bool, runs: &[(&str, f32, f32)]) -> Fixture {
    line_box_fixture_with_font(
        css,
        runs,
        include_bytes!("../../../../../fixtures/fonts/roboto-input.ttf"),
    )
}

fn line_box_fixture_with_font(css: bool, runs: &[(&str, f32, f32)], bytes: &[u8]) -> Fixture {
    use crate::source::text::text_style_paint::TextStylePaint;
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(400.0), false);
    let label = layout(&arena, Some(&root), None, false);
    let asset = arena.insert(FontAsset::default());
    let font = HbFont::decode(bytes).unwrap();
    FontAsset::set_font_occurrence(&asset, Some(font));
    let text = arena.insert(Text::default());
    attach(&arena, &label, &text);
    let mut first_run = None;
    for &(content, size, line_height) in runs {
        let style = arena.insert(TextStylePaint::default());
        style.with_mut(|object| {
            let style = object.as_text_style_mut().unwrap();
            style.base.set_font_size_value(size);
            style.base.set_line_height_value(line_height);
        });
        TextStyle::set_asset_occurrence(&style, Some(asset.clone()));
        attach(&arena, &text, &style);
        let mut run = TextValueRun::default();
        run.base.set_text_value(content.into());
        run.set_style(style);
        let run = arena.insert(run);
        attach(&arena, &text, &run);
        text.with_downcast_mut::<Text, _>(|text| text.add_run(run.clone()));
        first_run.get_or_insert(run);
    }
    if css {
        let record = arena.insert(crate::css_layout::LayoutCssStyle::default());
        attach(&arena, &label, &record);
        label.with_downcast_mut::<LayoutComponent, _>(|owner| {
            owner.add_layout_style_applier(record);
        });
    }
    for owner in [&label, &root] {
        LayoutComponent::sync_style_occurrence(owner);
        LayoutComponent::sync_layout_children_occurrence(owner);
    }
    Fixture {
        arena,
        root,
        label,
        text,
        run: first_run.unwrap(),
    }
}

fn assert_line_box(f: &Fixture, height: f32, baselines: &[f32]) {
    use crate::source::component_dirt::ComponentDirt;
    let measured = f.solve();
    assert!(
        (measured.height() - height).abs() < 0.01,
        "{measured:?} != {height}"
    );
    LayoutComponent::update_layout_bounds_occurrence(&f.root, false);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.update_after_transform_super(ComponentDirt::PATH);
            assert!((text.local_bounds().height() - height).abs() < 0.01);
            assert!((text.first_baseline().unwrap() - baselines[0]).abs() < 0.01);
            let lines = text.ordered_lines();
            assert_eq!(lines.len(), baselines.len());
            let mut previous_bottom = 0.0;
            for (line, expected) in lines.iter().zip(baselines) {
                let geometry = line.glyph_line();
                let top = line.y() - geometry.baseline + geometry.top;
                assert!((top - previous_bottom).abs() < 0.01);
                previous_bottom = line.bottom();
                assert!(
                    (line.y() - expected).abs() < 0.01,
                    "{} != {expected}",
                    line.y()
                );
            }
            assert!((previous_bottom - height).abs() < 0.01);
        })
        .unwrap();
}

#[test]
fn css_text_line_boxes_match_chrome() {
    for (size, line_height, height, baseline) in [
        (12.0, -1.0, 14.0, 11.0),
        (14.0, -1.0, 16.0, 13.0),
        (16.0, -1.0, 19.0, 15.0),
        (17.0, -1.0, 20.0, 16.0),
        (20.0, -1.0, 24.0, 19.0),
        (24.0, -1.0, 28.0, 22.0),
        (28.0, -1.0, 33.0, 26.0),
        (32.0, -1.0, 38.0, 30.0),
        (34.0, -1.0, 40.0, 32.0),
        (12.0, 16.0, 16.0, 12.0),
        (14.0, 20.0, 20.0, 15.0),
        (16.0, 24.0, 24.0, 17.0),
        (17.0, 22.0, 22.0, 17.0),
        (20.0, 28.0, 28.0, 21.0),
        (24.0, 32.0, 32.0, 24.0),
        (28.0, 36.0, 36.0, 27.0),
        (32.0, 40.0, 40.0, 31.0),
        (34.0, 41.0, 41.0, 32.0),
        (28.0, 28.0, 28.0, 23.0),
        (16.0, 16.0, 16.0, 13.0),
        // The compiler resolves a unitless 1.5 to 30 points at 20px.
        (20.0, 30.0, 30.0, 22.0),
    ] {
        assert_line_box(
            &line_box_fixture(true, &[("Aa", size, line_height)]),
            height,
            &[baseline],
        );
        assert_line_box(
            &line_box_fixture(true, &[("Aa\nAa", size, line_height)]),
            height * 2.0,
            &[baseline, baseline + height],
        );
    }
}

#[test]
fn css_text_mixed_runs_union_with_the_paragraph_strut() {
    // The element's empty base run still supplies its strut. The large
    // run alone has a 24-point box; its union with the strut is 28 tall.
    assert_line_box(
        &line_box_fixture(true, &[("", 16.0, 24.0), ("Aa", 28.0, 24.0)]),
        28.0,
        &[21.0],
    );
    for (line_height, baselines) in [(24.0, [21.0, 45.0]), (-1.0, [26.0, 48.0])] {
        assert_line_box(
            &line_box_fixture(
                true,
                &[
                    ("Aa", 16.0, line_height),
                    ("Aa", 28.0, line_height),
                    ("\nAa", 16.0, line_height),
                ],
            ),
            52.0,
            &baselines,
        );
    }
}

#[test]
fn translated_text_keeps_rive_line_spacing() {
    // Rive starts at 1900/2048 * size, then adds the authored descent
    // 500/2400 * line-height. Later baselines are one line-height apart.
    for (size, line_height, height, baseline) in [
        (16.0, 24.0, 19.84375, 14.84375),
        (28.0, 36.0, 33.4765625, 25.9765625),
        (16.0, -1.0, 18.75, 14.84375),
    ] {
        assert_line_box(
            &line_box_fixture(false, &[("Aa", size, line_height)]),
            height,
            &[baseline],
        );
        let step = if line_height < 0.0 {
            height
        } else {
            line_height
        };
        assert_line_box(
            &line_box_fixture(false, &[("Aa\nAa", size, line_height)]),
            height + step,
            &[baseline, baseline + step],
        );
    }
}

#[test]
fn css_text_drawn_glyph_baseline_matches_measurement() {
    use crate::source::text::text_style_paint::TextStylePaint;
    for (line_height, height, baseline) in [(24.0, 24.0, 17.0), (-1.0, 19.0, 15.0)] {
        let f = line_box_fixture(true, &[("A", 16.0, line_height)]);
        assert_line_box(&f, height, &[baseline]);
        let style: CoreHandle = f
            .run
            .with_downcast::<TextValueRun, _>(|run| run.style())
            .flatten()
            .unwrap();
        let glyph_top: f32 = f
            .text
            .with_downcast::<Text, _>(|text| {
                let run = &text.shape()[0].runs[0];
                let glyph = run.font.as_ref().unwrap().get_path(run.glyphs[0]);
                glyph.bounds().min_y * run.size + run.offsets[0].y
            })
            .unwrap();
        let drawn_top: f32 = style
            .with_downcast_mut::<TextStylePaint, _>(|style| {
                style.local_path().raw_path().bounds().min_y
            })
            .unwrap();
        assert!((drawn_top - glyph_top - baseline).abs() < 0.01);
    }
}

#[test]
fn css_text_fixed_leaf_gets_line_boxes_without_measurement() {
    use crate::source::component_dirt::ComponentDirt;
    let f = line_box_fixture(true, &[("Aa", 16.0, 24.0)]);
    let before = host_measure_count();
    // A definite leaf can skip Taffy's measure callback entirely.
    f.label.with_downcast_mut::<LayoutComponent, _>(|owner| {
        owner.set_solved_layout(
            Layout::new(0.0, 0.0, 200.0, 100.0),
            LayoutPadding::default(),
        );
    });
    LayoutComponent::update_layout_bounds_occurrence(&f.label, false);
    assert_eq!(host_measure_count(), before);
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.update_after_transform_super(ComponentDirt::PATH);
            assert_eq!(text.first_baseline(), Some(17.0));
            assert_eq!(text.ordered_lines()[0].glyph_line().bottom, 24.0);
            // Changing ownership at the same size must still invalidate shaping.
            text.base.set_dirt(ComponentDirt::NONE);
            text.set_owned_by_css_layout(false);
            assert!(text.base.has_dirt(ComponentDirt::PATH));
        })
        .unwrap();
}

#[test]
fn css_text_selection_and_carets_use_the_line_box() {
    use crate::source::text::{
        cursor::{Cursor, CursorPosition},
        glyph_lookup::GlyphLookup,
        text_layout_view::TextLayoutView,
    };
    for (css, height, baseline, selection_bottom) in
        [(true, 24.0, 17.0, 24.0), (false, 19.84375, 14.84375, 18.75)]
    {
        let f = line_box_fixture(css, &[("Aa", 16.0, 24.0)]);
        assert_line_box(&f, height, &[baseline]);
        f.text
            .with_downcast::<Text, _>(|text| {
                let mut lookup = GlyphLookup::default();
                lookup.compute(text.unichars(), text.shape());
                let lines = vec![vec![text.ordered_lines()[0].glyph_line().clone()]];
                let view =
                    TextLayoutView::new(text.shape(), &lines, text.ordered_lines(), &lookup, 2);
                for position in [CursorPosition::zero(), CursorPosition::new(0, 2)] {
                    let caret = position.visual_position(&view);
                    assert!(caret.found());
                    assert!(caret.top().abs() < 0.01);
                    assert!((caret.bottom() - selection_bottom).abs() < 0.01);
                }
                let mut rects = Vec::new();
                Cursor::new(CursorPosition::zero(), CursorPosition::new(0, 2))
                    .selection_rects(&mut rects, &view);
                assert!(!rects.is_empty());
                for rect in rects {
                    assert!(rect.min_y.abs() < 0.01);
                    assert!((rect.max_y - selection_bottom).abs() < 0.01);
                }
            })
            .unwrap();
    }
}

#[test]
fn css_text_normal_uses_hhea_and_rounded_line_gap_despite_typo_flag() {
    let mut bytes = include_bytes!("../../../../../fixtures/fonts/roboto-input.ttf").to_vec();
    let table_offset = |tag: &[u8]| {
        let count = u16::from_be_bytes(bytes[4..6].try_into().unwrap()) as usize;
        let record = bytes[12..]
            .chunks_exact(16)
            .take(count)
            .find(|record| &record[..4] == tag)
            .unwrap();
        u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize
    };
    let os2 = table_offset(b"OS/2");
    let hhea = table_offset(b"hhea");
    let selection = u16::from_be_bytes(bytes[os2 + 62..os2 + 64].try_into().unwrap());
    bytes[os2 + 62..os2 + 64].copy_from_slice(&(selection | 0x80).to_be_bytes());
    for (offset, value) in [(68, 1200i16), (70, -300), (72, 512)] {
        bytes[os2 + offset..os2 + offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    // At 16px, 320/2048 em rounds from 2.5 to 3 whole points.
    bytes[hhea + 8..hhea + 10].copy_from_slice(&320i16.to_be_bytes());
    assert_line_box(
        &line_box_fixture_with_font(true, &[("Aa\nAa", 16.0, -1.0)], &bytes),
        44.0,
        &[16.0, 38.0],
    );
    // Upstream still reads the requested typo extents and ignores their gap.
    assert_line_box(
        &line_box_fixture_with_font(false, &[("Aa\nAa", 16.0, -1.0)], &bytes),
        23.4375,
        &[9.375, 21.09375],
    );
}

#[test]
fn css_text_wrapped_lines_keep_the_full_line_height() {
    use crate::source::generated::core_registry::CoreRegistry;
    for (line_height, height, baseline) in [(24.0, 24.0, 17.0), (-1.0, 19.0, 15.0)] {
        let f = line_box_fixture(true, &[("Aa Aa", 16.0, line_height)]);
        let record = f
            .label
            .with_downcast::<LayoutComponent, _>(|owner| owner.css_record().unwrap())
            .unwrap();
        assert!(CoreRegistry::set_uint_handle(&record, 61021, 1));
        assert!(CoreRegistry::set_double_handle(&record, 61022, 30.0));
        LayoutComponent::sync_style_occurrence(&f.label);
        assert_line_box(&f, height * 2.0, &[baseline, baseline + height]);
    }
}
