//! Exact generated-input translation of renderer/src/shaders/spirv/draw_path.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "9463ff7b5b9a1452d0c32e41390a99cd39b6c946";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/draw_path.main";
pub const PINNED_SOURCE_SHA256: &str = "e81f62d9c9e5d3c786261fe4424284626be83ee550e1ebd12469278ee0d38154";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 16;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 588;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_draw_path_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
