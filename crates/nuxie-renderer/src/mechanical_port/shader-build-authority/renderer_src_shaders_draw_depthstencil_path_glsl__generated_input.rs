//! Exact generated-input translation of renderer/src/shaders/draw_depthstencil_path.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "625454e362a27bb3168f00cd87e9487f54d39338";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_path.glsl";
pub const PINNED_SOURCE_SHA256: &str = "a625e0df52f09fed2531ee047b85bea26f494c5af896a6a239e31b0a2dbe677d";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_depthstencil_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 644;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 25621;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_depthstencil_path_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
