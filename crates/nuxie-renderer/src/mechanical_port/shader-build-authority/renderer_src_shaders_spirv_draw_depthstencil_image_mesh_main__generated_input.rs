//! Exact generated-input translation of renderer/src/shaders/spirv/draw_depthstencil_image_mesh.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/draw_depthstencil_image_mesh.main";
pub const PINNED_SOURCE_SHA256: &str = "33526daa726bfd6eea27edf0da68cdd046574fc12cf579b01edde54867e5a88b";
pub const OWNERSHIP_UNIT: &str = "shader:source:spirv_draw_depthstencil_image_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 15;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 532;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_draw_depthstencil_image_mesh_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
