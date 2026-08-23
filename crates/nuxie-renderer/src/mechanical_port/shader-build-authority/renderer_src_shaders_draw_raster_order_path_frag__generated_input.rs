//! Exact generated-input translation of renderer/src/shaders/draw_raster_order_path.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_raster_order_path.frag";
pub const PINNED_SOURCE_SHA256: &str = "f4b4f70790ff16aa39870f0fcd848afa69dc52bf4b45fbbed3c1dab645eeb67f";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_raster_order_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 234;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 8245;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_raster_order_path_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
