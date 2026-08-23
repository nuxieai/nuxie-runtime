//! Exact generated-input translation of renderer/src/shaders/draw_fullscreen_quad.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_fullscreen_quad.vert";
pub const PINNED_SOURCE_SHA256: &str = "6a9842803e8472ab8f756a191c6a6d60a7c28db5587ee22c5e9bddb000c49cc2";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_fullscreen_quad";
pub const PINNED_SOURCE_LINE_COUNT: usize = 15;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 335;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_fullscreen_quad_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
