//! Exact generated-input translation of renderer/src/shaders/draw_depthstencil_path.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "ce891f526a3297f9498a3e524fce6385cf0f2f25";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_path.glsl";
pub const PINNED_SOURCE_SHA256: &str = "77d59b98f60e3f17b2eb2b8d32c18dff38d52a251cd4d888961eb540138d9959";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_depthstencil_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 574;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22971;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_depthstencil_path_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
