//! Exact generated-input translation of renderer/src/shaders/draw_clockwise_path.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_path.frag";
pub const PINNED_SOURCE_SHA256: &str = "f033a35f69ad4d2802fc9afa21f0ca0e06f73bb516d9cd9099a378a553eaa377";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_clockwise_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 251;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 9698;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_clockwise_path_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
