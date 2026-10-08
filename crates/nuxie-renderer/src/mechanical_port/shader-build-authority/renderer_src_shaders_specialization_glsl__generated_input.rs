//! Exact generated-input translation of renderer/src/shaders/specialization.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "625454e362a27bb3168f00cd87e9487f54d39338";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/specialization.glsl";
pub const PINNED_SOURCE_SHA256: &str = "18528e3c245b10ef0680673d97530e6504fc565396fcf2792b9a2bccbe2e4543";
pub const OWNERSHIP_UNIT: &str = "shader:source:specialization";
pub const PINNED_SOURCE_LINE_COUNT: usize = 48;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2321;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_specialization_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
