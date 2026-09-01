/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/metal/color_ramp.metal.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal/color_ramp.metal";
pub const PINNED_SOURCE_SHA256: &str =
    "eb525a31f81466aa9b54828f38dd68c3dd62b2728a9fcf7f674304806fc56970";
pub const PINNED_SOURCE_LINE_COUNT: usize = 11;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 233;

/// Exact pinned upstream source bytes.
pub const PINNED_COLOR_RAMP_METAL_SOURCE: &str = r###"#include <metal_stdlib>

#define VERTEX
#define FRAGMENT

#include "metal.minified.glsl"
#include "constants.minified.glsl"

#include "flush_uniforms.minified.glsl"
#include "common.minified.glsl"
#include "color_ramp.minified.glsl"
"###;

/// Stable source aliases.
pub const PINNED_COLOR_RAMP_SOURCE: &str = PINNED_COLOR_RAMP_METAL_SOURCE;
pub const COLOR_RAMP_METAL_SOURCE: &str = PINNED_COLOR_RAMP_METAL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_COLOR_RAMP_METAL_SOURCE
}
