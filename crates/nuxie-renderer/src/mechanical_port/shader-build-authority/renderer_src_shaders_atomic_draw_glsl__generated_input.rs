//! Exact generated-input translation of renderer/src/shaders/atomic_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "2579994c59cff57ac04d3a38401fa37ad1315425";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/atomic_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "6e55d8b08cc536b5a0c74e52332367d57507b753482543ceb61c6321fdfd6bc3";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 1173;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 40640;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_atomic_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
