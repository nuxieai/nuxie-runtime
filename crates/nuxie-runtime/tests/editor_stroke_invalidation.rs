#[cfg(feature = "editor")]
use nuxie_runtime::source::component_dirt::ComponentDirt;
use nuxie_runtime::source::shapes::paint::stroke::Stroke;

#[cfg(feature = "editor")]
#[test]
fn colorless_stroke_invalidation_preserves_clean_dirt() {
    let mut stroke = Stroke::default();
    stroke.base.set_dirt(ComponentDirt::NONE);
    stroke.invalidate_rendering();
    assert_eq!(stroke.base.dirt(), ComponentDirt::NONE);
    stroke.base.invalidate_rendering();
    assert_eq!(stroke.base.dirt(), ComponentDirt::NONE);
}

#[cfg(not(feature = "editor"))]
#[test]
#[should_panic(expected = "initialized Stroke render paint")]
fn runtime_stroke_invalidation_requires_initialized_paint() {
    Stroke::default().invalidate_rendering();
}
