//! Exact generated-input translation of renderer/src/shaders/draw_path_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "3a6e72e80eec81b2eb467134f62188e2a86f7debfb0798a8c4ed5873beb7e86e";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 914;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 39516;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
