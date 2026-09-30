//! Exact generated-input translation of renderer/src/shaders/draw_path.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "2579994c59cff57ac04d3a38401fa37ad1315425";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path.vert";
pub const PINNED_SOURCE_SHA256: &str = "5cbed9fefcde2a5a05bc711d5095f814c7a0989df955b5ad876e226219987f61";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 486;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 16226;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
