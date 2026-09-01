/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/metal/tessellate.metal.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal/tessellate.metal";
pub const PINNED_SOURCE_SHA256: &str =
    "e7bc6d35e1ad2f95e84b9595738c7035a0ff0bf13277681bec36ddf15cae3471";
pub const PINNED_SOURCE_LINE_COUNT: usize = 12;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 271;

/// Exact pinned upstream source bytes.
pub const PINNED_TESSELLATE_METAL_SOURCE: &str = r###"#include <metal_stdlib>

#define VERTEX
#define FRAGMENT

#include "metal.minified.glsl"
#include "constants.minified.glsl"

#include "flush_uniforms.minified.glsl"
#include "common.minified.glsl"
#include "bezier_utils.minified.glsl"
#include "tessellate.minified.glsl"
"###;

/// Stable source aliases.
pub const PINNED_TESSELLATE_SOURCE: &str = PINNED_TESSELLATE_METAL_SOURCE;
pub const TESSELLATE_METAL_SOURCE: &str = PINNED_TESSELLATE_METAL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_TESSELLATE_METAL_SOURCE
}
