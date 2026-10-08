//! Exact generated-input translation of renderer/src/shaders/flush_uniforms.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "625454e362a27bb3168f00cd87e9487f54d39338";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/flush_uniforms.glsl";
pub const PINNED_SOURCE_SHA256: &str = "e4d61497e4e01f3ee262a4eec38d3d2968248def95d3abf0a55a0f14af4010c5";
pub const OWNERSHIP_UNIT: &str = "shader:source:flush_uniforms";
pub const PINNED_SOURCE_LINE_COUNT: usize = 65;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2831;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_flush_uniforms_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
