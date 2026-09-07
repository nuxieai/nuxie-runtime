//! Exact generated-input translation of renderer/src/shaders/draw_path.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ad6fcf47526b033e5cbe16275e9219365551d76";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path.vert";
pub const PINNED_SOURCE_SHA256: &str = "61cb7a9875b263f101e1621d89c139404fbc2406ae570b1235cb36060dfe1650";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 549;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 18959;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
