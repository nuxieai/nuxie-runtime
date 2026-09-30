//! Exact generated-input translation of renderer/src/shaders/atomic_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "1988fdd490cc7a0b88992bd7bc9b27f7c567ba62";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/atomic_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "44a730ff51d45fcfb679eca3b8de48cad4a0559e11d8d30b46de5156e4651451";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 1190;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 41436;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_atomic_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
