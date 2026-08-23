//! Exact generated-input translation of renderer/src/shaders/tessellate.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/tessellate.glsl";
pub const PINNED_SOURCE_SHA256: &str = "ecf7b979552b7bf7af6ff17a3fc8a0f5666942f7f7febda96fb4255640deee2f";
pub const OWNERSHIP_UNIT: &str = "shader:source:tessellate";
pub const PINNED_SOURCE_LINE_COUNT: usize = 560;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 24329;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_tessellate_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
