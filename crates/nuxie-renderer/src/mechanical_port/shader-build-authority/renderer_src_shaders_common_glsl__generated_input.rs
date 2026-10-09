//! Exact generated-input translation of renderer/src/shaders/common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "ee9d8efa482164ab747f47cf5c1beb0d43fada40a13e4b99473b5e00a1ce3e53";
pub const OWNERSHIP_UNIT: &str = "shader:source:common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 503;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 17400;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
