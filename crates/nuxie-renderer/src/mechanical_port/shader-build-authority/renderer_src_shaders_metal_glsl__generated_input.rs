//! Exact generated-input translation of renderer/src/shaders/metal.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "b86b7ecb0256842cc37823f63c8699d5bffe081e";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal.glsl";
pub const PINNED_SOURCE_SHA256: &str = "20116330a891582022b640d3c1b20e33f450dd91a31cce9871e6825966829dd5";
pub const OWNERSHIP_UNIT: &str = "shader:source:metal";
pub const PINNED_SOURCE_LINE_COUNT: usize = 536;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 27236;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_metal_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
