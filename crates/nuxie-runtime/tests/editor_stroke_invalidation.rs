#![cfg(feature = "editor")]

use nuxie_runtime::source::{component_dirt::ComponentDirt, shapes::paint::stroke::Stroke};

#[test]
fn colorless_stroke_invalidation_preserves_clean_dirt() {
    let mut stroke = Stroke::default();
    stroke.base.set_dirt(ComponentDirt::NONE);
    stroke.invalidate_rendering();
    assert_eq!(stroke.base.dirt(), ComponentDirt::NONE);
    stroke.base.invalidate_rendering();
    assert_eq!(stroke.base.dirt(), ComponentDirt::NONE);
}
