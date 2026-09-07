//! Exact generated-input translation of renderer/src/shaders/glsl.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ad6fcf47526b033e5cbe16275e9219365551d76";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/glsl.glsl";
pub const PINNED_SOURCE_SHA256: &str = "980cefe2bdf70e2e9ee86be2c1e9fe806d78277a5d55033032b362b5c6461db5";
pub const OWNERSHIP_UNIT: &str = "shader:source:glsl";
pub const PINNED_SOURCE_LINE_COUNT: usize = 736;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 30718;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_glsl_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
