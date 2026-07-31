/// Runtime boundary for pinned `TextStyleFeature::onAddedDirty`.
///
/// Feature children are deliberately rejected until their parent registration
/// and HarfBuzz feature application are ported. Keeping that tracked ceiling
/// in the matching owner file prevents an unrelated shaping helper from being
/// presented as `TextStyleFeature` correspondence (`text_style_feature.cpp:7`).
fn static_text_style_feature_is_unsupported(type_name: Option<&str>) -> bool {
    type_name == Some("TextStyleFeature")
}
