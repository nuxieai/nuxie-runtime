//! Exact generated-input translation of renderer/src/shaders/draw_raster_order_path.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "3ed35ee0ded0d58fb8d380930a156041a4624a2f";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_raster_order_path.frag";
pub const PINNED_SOURCE_SHA256: &str = "06a00d578f24dc6e0447172995f2358098bcbde350b1d0b6d2f9d4fabc454009";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_raster_order_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 240;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 8433;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_raster_order_path_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
