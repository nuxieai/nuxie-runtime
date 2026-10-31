//! Exact generated-input translation of renderer/src/shaders/metal.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "67946b1396fad83f4c140a38daa43d33e9fd6763";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal.glsl";
pub const PINNED_SOURCE_SHA256: &str = "9d340e6626fbd8f821d20ea443ed9df266e268572cd511ac0f1372f69bd49d63";
pub const OWNERSHIP_UNIT: &str = "shader:source:metal";
pub const PINNED_SOURCE_LINE_COUNT: usize = 643;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 33473;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_metal_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
