//! Exact generated-input translation of renderer/src/shaders/constants.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "1cc2396f0d0d3f6d9c0b16809904e85265f617eb";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/constants.glsl";
pub const PINNED_SOURCE_SHA256: &str = "177a54d495e6a9084b87ad3b2ef42366f058aedce92e09da789d3b701c6680f7";
pub const OWNERSHIP_UNIT: &str = "shader:source:constants";
pub const PINNED_SOURCE_LINE_COUNT: usize = 346;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 14450;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_constants_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
