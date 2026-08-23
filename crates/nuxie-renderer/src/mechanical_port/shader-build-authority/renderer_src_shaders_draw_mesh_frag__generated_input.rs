//! Exact generated-input translation of renderer/src/shaders/draw_mesh.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_mesh.frag";
pub const PINNED_SOURCE_SHA256: &str = "d3a060c05d66e187ca2a0edab03788e7d885364ba833237e2e89f39a2b5e9c1f";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 227;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6989;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_mesh_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
