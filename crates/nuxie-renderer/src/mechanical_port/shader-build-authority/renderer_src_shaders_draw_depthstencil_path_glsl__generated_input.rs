//! Exact generated-input translation of renderer/src/shaders/draw_depthstencil_path.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "1c75867de17587c541ad2d7ca88b10577b033bd2";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_path.glsl";
pub const PINNED_SOURCE_SHA256: &str = "c6cf5c8514aec2f3c3f40c5f03268ba2224db766ce4617d8e1152716ec5b4f11";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_depthstencil_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 575;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 23047;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_depthstencil_path_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
