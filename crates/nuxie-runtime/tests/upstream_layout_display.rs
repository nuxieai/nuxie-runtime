//! runtime/layout_display_test.cpp at 75a22f94.
#[path = "support/layout_955.rs"]
mod support;
use nuxie_runtime::source::generated::layout::layout_sizing_style_base::LayoutSizingStyleBase;
use support::*;

#[test]
fn a_layout_hidden_through_its_style_reads_as_collapsed_at_once() {
    let file = read_file("layout/layout_anim_bound.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    artboard.advance_default(0.0);
    let layout = artboard
        .with_artboard(|a| a.find_all_handles::<LayoutComponent>())
        .into_iter()
        .find(|candidate| {
            !candidate
                .is_type_of(nuxie_runtime::source::generated::artboard_base::ArtboardBase::TYPE_KEY)
                && candidate
                    .with(|o| o.as_layout_component().unwrap().style_handle().is_some())
                    .unwrap()
        })
        .expect("layout with style");
    let assert_visibility = |hidden| {
        layout
            .with(|component| {
                assert_eq!(component.component_is_collapsed(), hidden);
                assert_eq!(component.as_layout_component().unwrap().is_hidden(), hidden);
            })
            .unwrap();
    };
    assert_visibility(false);
    uint(
        &style(&layout),
        LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY,
        1,
    );
    assert_visibility(true);
    uint(
        &style(&layout),
        LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY,
        0,
    );
    assert_visibility(false);
}
