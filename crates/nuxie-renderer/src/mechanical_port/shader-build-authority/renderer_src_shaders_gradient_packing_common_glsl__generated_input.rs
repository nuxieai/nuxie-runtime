//! Exact generated-input translation of renderer/src/shaders/gradient_packing_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/gradient_packing_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "31421d257f54a59a5088efd15c9212140ca7ba210eda63626cbbe8c984486f7b";
pub const OWNERSHIP_UNIT: &str = "shader:source:gradient_packing_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 147;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6102;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_gradient_packing_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
