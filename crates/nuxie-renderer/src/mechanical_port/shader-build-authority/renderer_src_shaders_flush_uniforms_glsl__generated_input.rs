//! Exact generated-input translation of renderer/src/shaders/flush_uniforms.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/flush_uniforms.glsl";
pub const PINNED_SOURCE_SHA256: &str = "a629fa5ad38939de27aa5d0851dbc58cb2672e14fe70b05ef27268bf2906f48d";
pub const OWNERSHIP_UNIT: &str = "shader:source:flush_uniforms";
pub const PINNED_SOURCE_LINE_COUNT: usize = 63;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2716;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_flush_uniforms_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
