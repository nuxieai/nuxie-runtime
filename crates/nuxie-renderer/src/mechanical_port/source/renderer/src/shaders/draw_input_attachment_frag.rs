/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_input_attachment.frag.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_input_attachment.frag";
pub const PINNED_SOURCE_SHA256: &str =
    "8af2574495c71e8282116b3e598daf5e1c705d0b50c39d2439ad18e6be1e8694";
pub const PINNED_SOURCE_LINE_COUNT: usize = 18;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 392;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_INPUT_ATTACHMENT_FRAG_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @FRAGMENT
layout(input_attachment_index = 0,
#ifdef @INPUT_ATTACHMENT_BINDING
       binding = @INPUT_ATTACHMENT_BINDING,
#else
       binding = 0,
#endif
       set = PLS_TEXTURE_BINDINGS_SET) uniform lowp subpassInput
    inputAttachment;

layout(location = 0) out half4 outputColor;

void main() { outputColor = subpassLoad(inputAttachment); }
#endif
"###;

/// Stable source aliases.
pub const PINNED_DRAW_INPUT_ATTACHMENT_SOURCE: &str = PINNED_DRAW_INPUT_ATTACHMENT_FRAG_SOURCE;
pub const DRAW_INPUT_ATTACHMENT_FRAG_SOURCE: &str = PINNED_DRAW_INPUT_ATTACHMENT_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_INPUT_ATTACHMENT_FRAG_SOURCE
}
