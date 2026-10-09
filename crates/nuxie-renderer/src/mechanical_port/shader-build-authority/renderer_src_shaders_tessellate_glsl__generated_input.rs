//! Exact generated-input translation of renderer/src/shaders/tessellate.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/tessellate.glsl";
pub const PINNED_SOURCE_SHA256: &str = "df9c6087f6dac668a3e274862ed34fb1df19c9a88bea87f4ccd93410cee7dd2b";
pub const OWNERSHIP_UNIT: &str = "shader:source:tessellate";
pub const PINNED_SOURCE_LINE_COUNT: usize = 617;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 27235;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_tessellate_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
