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
