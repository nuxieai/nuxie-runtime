//! Exact generated-input translation of renderer/src/shaders/draw_msaa_resolve.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_msaa_resolve.frag";
pub const PINNED_SOURCE_SHA256: &str = "93cac1c9b5a8f5a4c41100475b797ae1352fc803c81d619de1b1e81bdc0fb6a1";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_msaa_resolve";
pub const PINNED_SOURCE_LINE_COUNT: usize = 18;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 438;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_msaa_resolve_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
