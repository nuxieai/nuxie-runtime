# `TextModifier` paired audit

Upstream owner: `src/text/text_modifier.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owner: `crates/nuxie-runtime/src/text/text_modifier.rs`, with the parent
collection in `text_modifier_group.rs` and graph validation in `text.rs`.

Verdict: adapted and behaviorally equivalent.

The pinned C++ file owns one behavior: `onAddedDirty` requires a direct
`TextModifierGroup` parent and registers the modifier with that group. Rust
validates the same parent relationship before constructing a static text
slice, then `StaticTextModifier::from_group_child` registers every concrete
modifier in authored child order. Shape and follow-path indexes are derived by
the group after that registration; they do not change the base owner contract.

Verification covers authored ordering, generic/shape modifier registration,
and rejection of a modifier with the wrong parent through the text owner tests
and the converted `upstream_text_modifier_structure_body_is_ported` test.
