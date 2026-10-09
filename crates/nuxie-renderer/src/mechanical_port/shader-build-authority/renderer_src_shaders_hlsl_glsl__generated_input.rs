//! Exact generated-input translation of renderer/src/shaders/hlsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/hlsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "55a3fe453ea3052e71fec769d18d3a3be5b451eefdc89d20086c88a9c4924615";
pub const OWNERSHIP_UNIT: &str = "shader:source:hlsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 466;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 19113;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_hlsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
