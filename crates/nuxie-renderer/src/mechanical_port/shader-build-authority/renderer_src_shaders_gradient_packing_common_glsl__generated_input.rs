//! Exact generated-input translation of renderer/src/shaders/gradient_packing_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4672b88bf62eee54746854440a914ed20499fe8a";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/gradient_packing_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "63818d2b1ab524993424f52d8161f4ca11db2db1dccfa6f90ff94934c4cc3c83";
pub const OWNERSHIP_UNIT: &str = "shader:source:gradient_packing_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 167;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6996;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_gradient_packing_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
