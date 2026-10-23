//! Exact generated-input translation of renderer/src/shaders/stencil_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "ada892f37808e289e716d29010b7e79d0656f79d";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/stencil_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "32c231d28cf7a606755725c4187c29a4d8237da88ab1ee3a069bf329ac78be6c";
pub const OWNERSHIP_UNIT: &str = "shader:source:stencil_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 34;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 811;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_stencil_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
