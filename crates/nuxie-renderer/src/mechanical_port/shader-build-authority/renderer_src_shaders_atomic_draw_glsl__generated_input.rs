//! Exact generated-input translation of renderer/src/shaders/atomic_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "0aadd4c65084a38dbeae3bd05814ead3f743ed77";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/atomic_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "54e6f127d211a3d95e3fd30ee50f8859569ae91f5f8d6201b0a7517d712d1f6b";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 1180;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 41111;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_atomic_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
