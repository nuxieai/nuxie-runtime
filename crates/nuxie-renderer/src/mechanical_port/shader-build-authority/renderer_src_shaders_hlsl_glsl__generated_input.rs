//! Exact generated-input translation of renderer/src/shaders/hlsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/hlsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "ccdbdadea1add6c67088c2b36e4a25975d01150412f7d0554a8933bd91cb337d";
pub const OWNERSHIP_UNIT: &str = "shader:source:hlsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 458;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 18857;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_hlsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
