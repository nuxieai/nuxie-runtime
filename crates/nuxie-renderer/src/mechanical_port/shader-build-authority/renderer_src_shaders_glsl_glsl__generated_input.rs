//! Exact generated-input translation of renderer/src/shaders/glsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/glsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "d7e3b795badbe6e5108f268ddea4f7c0bb5af4ad1416e41c7304beca89a15523";
pub const OWNERSHIP_UNIT: &str = "shader:source:glsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 726;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 30330;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_glsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
