/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/draw_msaa_resolve.frag.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_msaa_resolve.frag";
pub const PINNED_SOURCE_SHA256: &str =
    "93cac1c9b5a8f5a4c41100475b797ae1352fc803c81d619de1b1e81bdc0fb6a1";
pub const PINNED_SOURCE_LINE_COUNT: usize = 18;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 438;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_MSAA_RESOLVE_FRAG_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @FRAGMENT
layout(input_attachment_index = 0,
       binding = COLOR_PLANE_IDX,
       set = PLS_TEXTURE_BINDINGS_SET) uniform lowp subpassInputMS msaaColor;

layout(location = 0) out half4 outputColor;

void main()
{
    outputColor = (subpassLoad(msaaColor, 0) + subpassLoad(msaaColor, 1) +
                   subpassLoad(msaaColor, 2) + subpassLoad(msaaColor, 3)) *
                  .25;
}
#endif
"###;

/// Stable source aliases.
pub const PINNED_DRAW_MSAA_RESOLVE_SOURCE: &str = PINNED_DRAW_MSAA_RESOLVE_FRAG_SOURCE;
pub const DRAW_MSAA_RESOLVE_FRAG_SOURCE: &str = PINNED_DRAW_MSAA_RESOLVE_FRAG_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_MSAA_RESOLVE_FRAG_SOURCE
}
