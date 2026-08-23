//! Exact generated-input translation of renderer/src/shaders/spirv/draw_path.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/draw_path.main";
pub const PINNED_SOURCE_SHA256: &str = "7e32273a1aa44b8bb8e6295ae209c1337abb871f4c0c1f2574e4e1eebb70b3e1";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 15;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 539;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_draw_path_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
