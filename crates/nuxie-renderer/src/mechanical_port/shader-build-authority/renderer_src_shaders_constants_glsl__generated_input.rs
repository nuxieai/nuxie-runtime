//! Exact generated-input translation of renderer/src/shaders/constants.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "966499fffe2aadcbcd1fe4388160e4e7d5c0d967";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/constants.glsl";
pub const PINNED_SOURCE_SHA256: &str = "b0b59911b49c1105c635569ce476418ea62dc1d42c9ff55ce8bfb5df700ada5a";
pub const OWNERSHIP_UNIT: &str = "shader:source:constants";
pub const PINNED_SOURCE_LINE_COUNT: usize = 335;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 13960;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_constants_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
