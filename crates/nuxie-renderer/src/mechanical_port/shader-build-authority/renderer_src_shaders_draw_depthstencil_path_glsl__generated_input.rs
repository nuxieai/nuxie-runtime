//! Exact generated-input translation of renderer/src/shaders/draw_depthstencil_path.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c73593c3a868f87b5dcd328432255db33e3e2266";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_path.glsl";
pub const PINNED_SOURCE_SHA256: &str = "18bdf49f1132bf71eafd688763c54f75c636496a9c3de25d51f905714434a901";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_depthstencil_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 679;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 27091;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_depthstencil_path_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
