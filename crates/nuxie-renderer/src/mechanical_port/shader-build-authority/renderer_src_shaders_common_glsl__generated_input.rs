//! Exact generated-input translation of renderer/src/shaders/common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c18b32511bfeaeee6b7c54e35152aea3fdbb5964";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "7a6fff449a340673add5a68490524bb4682bf4a2583637dc462573aa80c07c6a";
pub const OWNERSHIP_UNIT: &str = "shader:source:common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 495;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 16623;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
