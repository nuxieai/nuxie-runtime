//! Exact generated-input translation of renderer/src/shaders/draw_path.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "5d7ab77e6a0fc9f91e69fd08c8b470c7d072d555";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path.vert";
pub const PINNED_SOURCE_SHA256: &str = "2d9be965fbfbfdc5a57313509af4d38a00328194054a6974ed8b8226fdf8659e";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 515;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 17447;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
