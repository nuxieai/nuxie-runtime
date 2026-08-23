//! Exact generated-input translation of renderer/src/shaders/draw_image_mesh.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_image_mesh.vert";
pub const PINNED_SOURCE_SHA256: &str = "f8c9d0c3a50cd3d42af1e67f8acb4258ac8c05833210d0b4556c95dff3312166";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_image_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 144;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 4552;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_image_mesh_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
