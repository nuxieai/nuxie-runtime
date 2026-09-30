//! Exact generated-input translation of renderer/src/shaders/draw_clockwise_path.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "b86b7ecb0256842cc37823f63c8699d5bffe081e";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_clockwise_path.frag";
pub const PINNED_SOURCE_SHA256: &str = "1a70223689c84e2c99a36858b6c6cc1784551c18e0522bef9226d4ac0f136396";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_clockwise_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 259;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 9716;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_clockwise_path_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
