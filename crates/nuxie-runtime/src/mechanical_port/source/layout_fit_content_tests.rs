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
    println!("expected={expected} box={bounds:?}");
    assert!(
        (bounds.width() - expected).abs() < 0.001,
        "{bounds:?} != width {expected}"
    );
}
const TITLE: &str = "Cross-plane experience is active";
// Pinned Inter_28pt-Bold at 28px. Independent natural measurements:
// Unlock premium = 214.29297, experience (the longest word) = 145.4414.
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
fn css_fit_content_hug_in_a_row_still_does_not_shrink_beside_a_sibling() {
    let f = Fixture::new(TITLE, 338.0, true);
    let sibling = layout(&f.arena, Some(&f.root), Some(56.0), false);
    LayoutComponent::sync_style_occurrence(&sibling);
    LayoutComponent::sync_layout_children_occurrence(&f.root);
    // UNIV-4844: the preserved hug flex-shrink:0 gets 338, while the
    // browser default flex-shrink:1 would share the row and get 282.
    assert_eq!(
        f.label
            .with_downcast::<LayoutComponent, _>(|l| l.layout_style().flex_shrink),
        Some(0.0)
    );
    assert_width(&f, 338.0);
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
        println!("fresh-solve verification: zero mismatches for text changes and resizing");
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
    println!("max-width column: {bounds:?}");
    assert!((bounds.width() - 260.0).abs() < 0.001, "{bounds:?}");
    assert!((bounds.height() - 70.30176).abs() < 0.001, "{bounds:?}");
    assert!(
        sibling
            .with_downcast::<LayoutComponent, _>(|s| s.layout_data.solved_layout.top())
            .unwrap()
            >= bounds.top() + bounds.height()
    );
}
