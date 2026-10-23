//! Exact generated-input translation of renderer/src/shaders/hlsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "ada892f37808e289e716d29010b7e79d0656f79d";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/hlsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "bb1d273560f3db276d69440d1e4431b7771651c7fca71db0fd85fa418157046d";
pub const OWNERSHIP_UNIT: &str = "shader:source:hlsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 510;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 21518;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_hlsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
