//! Exact generated-input translation of renderer/src/shaders/glsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c14cb2510071bd4cfa08d52ba5cd44d98c362237";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/glsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "c24401654da284230ffbf34a71e4546592b184aee27709f88a9fadbd174081db";
pub const OWNERSHIP_UNIT: &str = "shader:source:glsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 750;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 31545;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_glsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
