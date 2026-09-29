//! Exact generated-input translation of renderer/src/shaders/glsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c18b32511bfeaeee6b7c54e35152aea3fdbb5964";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/glsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "95ed8ed5e9e35b0d34c30b4415ccc204f113406b83a833ac738d659b418a4cb6";
pub const OWNERSHIP_UNIT: &str = "shader:source:glsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 736;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 30772;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_glsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
