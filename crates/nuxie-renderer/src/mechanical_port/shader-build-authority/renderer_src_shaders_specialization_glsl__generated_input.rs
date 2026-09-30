//! Exact generated-input translation of renderer/src/shaders/specialization.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c14cb2510071bd4cfa08d52ba5cd44d98c362237";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/specialization.glsl";
pub const PINNED_SOURCE_SHA256: &str = "efbc2e65e159ae25f7cbd0f9859cd4809abafdc37429c87471041b29e2df3781";
pub const OWNERSHIP_UNIT: &str = "shader:source:specialization";
pub const PINNED_SOURCE_LINE_COUNT: usize = 46;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2213;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_specialization_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
