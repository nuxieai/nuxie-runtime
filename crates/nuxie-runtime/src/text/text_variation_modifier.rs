/// Runtime boundary for pinned `TextVariationModifier::modify` and
/// `axisValueChanged`. Live variation-modifier identity remains a tracked
/// unsupported graph variant; the direct owner records that ceiling instead
/// of presenting range interpolation as variation behavior
/// (`text_variation_modifier.cpp:7-24`).
fn static_text_variation_modifier_is_unsupported(type_name: Option<&str>) -> bool {
    type_name == Some("TextVariationModifier")
}
