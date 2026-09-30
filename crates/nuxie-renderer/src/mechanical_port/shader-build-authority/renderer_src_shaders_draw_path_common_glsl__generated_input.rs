//! Exact generated-input translation of renderer/src/shaders/draw_path_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "69f43681303ec4e17c131c9f8d584c19f1c34290";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "a9ccb65a1fc87b9b17740b5795a81ad2c70b4c877753f37c682bbbfd9d6c5d1b";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 918;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 39826;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
