//! Exact generated-input translation of renderer/src/shaders/constants.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/constants.glsl";
pub const PINNED_SOURCE_SHA256: &str = "eefc0b9115beb4fe87b85431b0c683fdec73244c044278e97fdba0c96014bb56";
pub const OWNERSHIP_UNIT: &str = "shader:source:constants";
pub const PINNED_SOURCE_LINE_COUNT: usize = 322;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 13401;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_constants_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
