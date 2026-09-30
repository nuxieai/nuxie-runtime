//! Exact generated-input translation of renderer/src/shaders/draw_path_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "5705446d6aeb0dad34a63d8ddadbb79fbe327a37";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "38700d5ab6cc12b12f55cf694017e4e4cecc39fa4d17e5029eee44b2ae4b0da4";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 916;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 39624;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
