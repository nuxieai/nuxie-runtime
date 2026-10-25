//! Exact generated-input translation of renderer/src/shaders/constants.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "86594b7144ceeabe4261afeb0082fd9ee75bf9ca";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/constants.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "7b32bbf5e24222d6818aafcd07c2c6f77baa892a110ff14c3e923136a8aaece1";
pub const OWNERSHIP_UNIT: &str = "shader:source:constants";
pub const PINNED_SOURCE_LINE_COUNT: usize = 421;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 18400;
pub const PINNED_SOURCE: &[u8] =
    include_bytes!("source/renderer_src_shaders_constants_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
