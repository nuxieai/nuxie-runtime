/// Runtime boundary for pinned `TextModifier::onAddedDirty`. Abstract base
/// modifier children remain a tracked unsupported graph variant; concrete
/// modifier-group and follow-path behavior lives with those direct owners
/// (`text_modifier.cpp:7-21`).
fn static_text_modifier_is_unsupported(type_name: Option<&str>) -> bool {
    type_name == Some("TextModifier")
}
