//! Exact generated-input translation of renderer/src/shaders/draw_clockwise_path.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "3ed35ee0ded0d58fb8d380930a156041a4624a2f";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_path.frag";
pub const PINNED_SOURCE_SHA256: &str = "ea0dda57a43955db747aad0a802dc8f3cc41e42613083122674cd596addcecba";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_clockwise_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 258;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 9896;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_clockwise_path_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
