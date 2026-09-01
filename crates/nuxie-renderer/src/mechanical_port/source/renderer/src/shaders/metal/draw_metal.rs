/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/metal/draw.metal.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal/draw.metal";
pub const PINNED_SOURCE_SHA256: &str =
    "1111713584059e5d2b6469d45200b5c11949de17d7dcb7ffe62529c96c6269bd";
pub const PINNED_SOURCE_LINE_COUNT: usize = 42;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 947;

/// Exact pinned upstream source bytes.
pub const PINNED_DRAW_METAL_SOURCE: &str = r###"#include <metal_stdlib>

// Add baseInstance to the instanceID for path draws.
#define ENABLE_INSTANCE_INDEX

#define FRAGMENT

#define VERTEX
#include "metal.minified.glsl"
#include "constants.minified.glsl"
#define DRAW_IMAGE

#include "flush_uniforms.minified.glsl"
#include "common.minified.glsl"
#undef DRAW_IMAGE
#define DRAW_PATH
#define DRAW_INTERIOR_TRIANGLES
#define ENABLE_FEATHER
#define FEATHER_ATLAS_BLIT
#include "draw_path_common.minified.glsl"

#define ATLAS_FEATHERED_FILL
#define ATLAS_FEATHERED_STROKE
#include "render_atlas.minified.glsl"
#undef ATLAS_FEATHERED_FILL
#undef ATLAS_FEATHERED_STROKE

#undef FEATHER_ATLAS_BLIT
#undef ENABLE_FEATHER
#undef DRAW_INTERIOR_TRIANGLES
#undef DRAW_PATH
#undef VERTEX

#define ENABLE_ADVANCED_BLEND 1
#define ENABLE_HSL_BLEND_MODES 1
#include "advanced_blend.minified.glsl"
#undef ENABLE_HSL_BLEND_MODES
#undef ENABLE_ADVANCED_BLEND

#undef FRAGMENT

#include "draw_combinations.metal"
"###;

/// Stable source aliases.
pub const PINNED_DRAW_SOURCE: &str = PINNED_DRAW_METAL_SOURCE;
pub const DRAW_METAL_SOURCE: &str = PINNED_DRAW_METAL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_METAL_SOURCE
}
