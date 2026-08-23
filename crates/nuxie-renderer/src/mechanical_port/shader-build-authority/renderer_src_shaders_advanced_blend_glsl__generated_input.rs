//! Exact generated-input translation of renderer/src/shaders/advanced_blend.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/advanced_blend.glsl";
pub const PINNED_SOURCE_SHA256: &str = "d7f8d9cec8e095c7e6d331a9f3ba48cdb18ea63f961d9223e3dfc509bcd8794b";
pub const OWNERSHIP_UNIT: &str = "shader:source:advanced_blend";
pub const PINNED_SOURCE_LINE_COUNT: usize = 330;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 13219;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_advanced_blend_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
