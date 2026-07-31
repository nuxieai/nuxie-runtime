/// Runtime boundary for pinned `TextTargetModifier::onAddedDirty` and
/// `textComponent`. Target modifiers are still a tracked unsupported graph
/// variant, so the direct owner records that ceiling instead of borrowing the
/// modifier-group opacity implementation (`text_target_modifier.cpp:9-31`).
fn static_text_target_modifier_is_unsupported(type_name: Option<&str>) -> bool {
    type_name == Some("TextTargetModifier")
}
