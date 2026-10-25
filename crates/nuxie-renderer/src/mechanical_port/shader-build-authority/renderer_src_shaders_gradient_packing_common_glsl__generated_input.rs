//! Exact generated-input translation of renderer/src/shaders/gradient_packing_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "41a2120bb572e8168ba08fd2ea514c9ddf25d270";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/gradient_packing_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "5cabd01586f69442dfac1f658bae8b940d08c89bfb0529c385028a7d8e5cccef";
pub const OWNERSHIP_UNIT: &str = "shader:source:gradient_packing_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 170;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 6372;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_gradient_packing_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
