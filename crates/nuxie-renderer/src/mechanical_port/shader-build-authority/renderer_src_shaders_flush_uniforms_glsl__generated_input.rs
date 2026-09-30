//! Exact generated-input translation of renderer/src/shaders/flush_uniforms.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "7732f41ef93e4cb74286934ee596e1041d0a0ba7";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/flush_uniforms.glsl";
pub const PINNED_SOURCE_SHA256: &str = "d8ff851857239ab10ecf81cb55009e672097435c30b006bb1ccc667b874585ed";
pub const OWNERSHIP_UNIT: &str = "shader:source:flush_uniforms";
pub const PINNED_SOURCE_LINE_COUNT: usize = 60;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2573;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_flush_uniforms_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
