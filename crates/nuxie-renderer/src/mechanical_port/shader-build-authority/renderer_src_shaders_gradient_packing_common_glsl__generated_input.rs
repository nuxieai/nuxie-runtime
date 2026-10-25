//! Exact generated-input translation of renderer/src/shaders/gradient_packing_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "74e5d3f635c3c463fe10e4327261705549ee5919";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/gradient_packing_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "51d678f536e41ff62ffe745e88ddbe37af73317cf700d0cf39612c4b30b8043f";
pub const OWNERSHIP_UNIT: &str = "shader:source:gradient_packing_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 170;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6400;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_gradient_packing_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
