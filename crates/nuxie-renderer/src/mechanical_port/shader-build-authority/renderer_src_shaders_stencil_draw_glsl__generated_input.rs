//! Exact generated-input translation of renderer/src/shaders/stencil_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "57dddb3727306e284773ec20c653cf686c45abee";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/stencil_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "f05d05ee97d8bcb65284e92a9b102ea6fc8e494dee5575a73a0e733cb5684202";
pub const OWNERSHIP_UNIT: &str = "shader:source:stencil_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 31;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 772;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_stencil_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
