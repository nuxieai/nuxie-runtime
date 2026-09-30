//! Exact generated-input translation of renderer/src/shaders/draw_path.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "57dddb3727306e284773ec20c653cf686c45abee";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path.vert";
pub const PINNED_SOURCE_SHA256: &str = "eb68ec1e79af90d098b6595101bdba833d4b8cd33a965c3fa02ea5563c8babeb";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 496;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 16763;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
