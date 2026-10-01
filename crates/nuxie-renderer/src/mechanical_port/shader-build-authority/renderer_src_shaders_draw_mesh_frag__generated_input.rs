//! Exact generated-input translation of renderer/src/shaders/draw_mesh.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "0aadd4c65084a38dbeae3bd05814ead3f743ed77";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_mesh.frag";
pub const PINNED_SOURCE_SHA256: &str = "f1d04c96e07fba636aa6aa3ef31fec31aafbe1c2dc7790be812492fe1a31e381";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 210;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6229;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_mesh_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
