//! Exact generated-input translation of renderer/src/shaders/spirv/atomic_base.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/atomic_base.glsl";
pub const PINNED_SOURCE_SHA256: &str = "4f929858680a03c74e88b85b676d8020c7a588146367c7189eec791b2412365a";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_base";
pub const PINNED_SOURCE_LINE_COUNT: usize = 19;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 658;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_atomic_base_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
