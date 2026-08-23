//! Exact generated-input translation of renderer/src/shaders/flush_uniforms.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/flush_uniforms.glsl";
pub const PINNED_SOURCE_SHA256: &str = "1c22659c0e40233b0b06515287e122e06b73a4428d8e78721ca71e3419db961e";
pub const OWNERSHIP_UNIT: &str = "shader:source:flush_uniforms";
pub const PINNED_SOURCE_LINE_COUNT: usize = 58;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2454;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_flush_uniforms_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
