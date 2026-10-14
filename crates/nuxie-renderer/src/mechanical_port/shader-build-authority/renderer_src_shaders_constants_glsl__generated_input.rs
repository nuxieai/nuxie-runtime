//! Exact generated-input translation of renderer/src/shaders/constants.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4672b88bf62eee54746854440a914ed20499fe8a";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/constants.glsl";
pub const PINNED_SOURCE_SHA256: &str = "32650e811721a020028be54dd85ab05736e18d0b95eff434ccdb18fe0075fcc2";
pub const OWNERSHIP_UNIT: &str = "shader:source:constants";
pub const PINNED_SOURCE_LINE_COUNT: usize = 420;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 18378;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_constants_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
