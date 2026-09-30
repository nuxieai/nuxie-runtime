//! Exact generated-input translation of renderer/src/shaders/draw_mesh.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "2579994c59cff57ac04d3a38401fa37ad1315425";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_mesh.frag";
pub const PINNED_SOURCE_SHA256: &str = "6171a7ee41ed358ae9915a35c17e97d7fb694597e6aca2045f3023dbd66df721";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 221;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6615;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_mesh_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
