//! Exact generated-input translation of renderer/src/shaders/specialization.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/specialization.glsl";
pub const PINNED_SOURCE_SHA256: &str = "afd2d84e69cc09c49c66e524f89d0ad23435daf6e559b5142c0cef8a2de44add";
pub const OWNERSHIP_UNIT: &str = "shader:source:specialization";
pub const PINNED_SOURCE_LINE_COUNT: usize = 52;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2493;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_specialization_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
