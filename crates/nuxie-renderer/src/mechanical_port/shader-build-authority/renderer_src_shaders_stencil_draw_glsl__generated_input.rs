//! Exact generated-input translation of renderer/src/shaders/stencil_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "67946b1396fad83f4c140a38daa43d33e9fd6763";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/stencil_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "b31296a77d5767a2666c05070833aed74f7900a4391e962d02dfa9e7eef9b406";
pub const OWNERSHIP_UNIT: &str = "shader:source:stencil_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 34;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 804;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_stencil_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
