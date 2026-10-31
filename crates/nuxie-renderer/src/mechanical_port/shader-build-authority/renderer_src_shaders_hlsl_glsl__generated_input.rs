//! Exact generated-input translation of renderer/src/shaders/hlsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "67946b1396fad83f4c140a38daa43d33e9fd6763";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/hlsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "c5bbfef680613d8ce9d88ba1d07d0481445a8a2355a500606a6a855dada567ce";
pub const OWNERSHIP_UNIT: &str = "shader:source:hlsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 518;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22112;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_hlsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
