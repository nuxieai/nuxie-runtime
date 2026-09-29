//! Exact generated-input translation of renderer/src/shaders/spirv/draw_depthstencil_path.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c18b32511bfeaeee6b7c54e35152aea3fdbb5964";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/draw_depthstencil_path.main";
pub const PINNED_SOURCE_SHA256: &str = "a5ef3d8519a759f6c3b57ada19420c1a67c1006073e0b6e3d7cfaaa2c69d46e7";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_depthstencil_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 16;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 575;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_draw_depthstencil_path_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
