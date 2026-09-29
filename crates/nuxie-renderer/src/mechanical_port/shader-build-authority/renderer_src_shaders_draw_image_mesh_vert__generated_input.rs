//! Exact generated-input translation of renderer/src/shaders/draw_image_mesh.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c18b32511bfeaeee6b7c54e35152aea3fdbb5964";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_image_mesh.vert";
pub const PINNED_SOURCE_SHA256: &str = "3a9c838f13be3c5682f5ed6751ee2234a647b892eaa7b6fc90431c56b3f84f19";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_image_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 144;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 4597;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_image_mesh_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
