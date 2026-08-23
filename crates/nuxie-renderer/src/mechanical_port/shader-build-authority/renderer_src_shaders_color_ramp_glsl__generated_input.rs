//! Exact generated-input translation of renderer/src/shaders/color_ramp.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/color_ramp.glsl";
pub const PINNED_SOURCE_SHA256: &str = "65d0e8610193de1a7bf02722bcc153f04474e0d57644a35fd56291333ee8fde1";
pub const OWNERSHIP_UNIT: &str = "shader:source:color_ramp";
pub const PINNED_SOURCE_LINE_COUNT: usize = 107;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3167;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_color_ramp_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
