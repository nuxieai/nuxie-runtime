//! Exact generated-input translation of renderer/src/shaders/blit_texture_as_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/blit_texture_as_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "c9d6ab3c8911900a246d22484ad4dbda0a050ba76d74353c9a514d3ca7da3515";
pub const OWNERSHIP_UNIT: &str = "shader:source:blit_texture_as_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 72;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 1976;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_blit_texture_as_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
