//! Exact generated-input translation of renderer/src/shaders/specialization.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/specialization.glsl";
pub const PINNED_SOURCE_SHA256: &str = "e510983192e84c1cc69d990a14f606cfa92af234290636593d5be3e3f4e07f72";
pub const OWNERSHIP_UNIT: &str = "shader:source:specialization";
pub const PINNED_SOURCE_LINE_COUNT: usize = 43;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2044;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_specialization_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
