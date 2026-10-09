//! Exact generated-input translation of renderer/src/shaders/metal.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal.glsl";
pub const PINNED_SOURCE_SHA256: &str = "130728353bd3d3a2e015a8b412cca76cf421d7e3f6a03da36a34062669dde726";
pub const OWNERSHIP_UNIT: &str = "shader:source:metal";
pub const PINNED_SOURCE_LINE_COUNT: usize = 540;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 27528;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_metal_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
