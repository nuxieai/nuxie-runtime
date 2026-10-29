//! Exact generated-input translation of renderer/src/shaders/draw_path_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "6285c6fee237e464e21a19e75017501691cc3472";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "d9391635eaf18928bd40553eb218f0cb42d01ca3a17e1f7ea00391991b090610";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 972;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 37921;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
