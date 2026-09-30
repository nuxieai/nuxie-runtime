//! Exact generated-input translation of renderer/src/shaders/glsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "57dddb3727306e284773ec20c653cf686c45abee";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/glsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "4ea69005b08d721c305d9addc94de8033f58ac0b881f8a24afe023d6954c3dc5";
pub const OWNERSHIP_UNIT: &str = "shader:source:glsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 751;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 31560;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_glsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
