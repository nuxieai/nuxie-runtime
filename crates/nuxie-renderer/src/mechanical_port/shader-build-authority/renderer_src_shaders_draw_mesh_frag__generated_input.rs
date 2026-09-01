//! Exact generated-input translation of renderer/src/shaders/draw_mesh.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "3ed35ee0ded0d58fb8d380930a156041a4624a2f";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_mesh.frag";
pub const PINNED_SOURCE_SHA256: &str = "d6e7ec4585532526c9c225f5d49fe44c1e68a0acff94caa750a761212b5a3546";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 234;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 7187;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_mesh_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
