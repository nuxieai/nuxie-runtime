//! Exact generated-input translation of renderer/src/shaders/draw_path_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "3b615b829a58b67379f9304a161b3e87119bbf04";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "86f1f471765659bd2a7b0efca9f00bddac13f9ccd821329598b395f83fdcc893";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 929;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 40113;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
