//! Exact generated-input translation of renderer/src/shaders/common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "37d9f72c2ec84a9a24b42d8798c56c77e396c7b57a39f24edece8c95fe8b3881";
pub const OWNERSHIP_UNIT: &str = "shader:source:common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 494;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 16550;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
