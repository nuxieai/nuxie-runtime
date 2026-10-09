//! Exact generated-input translation of renderer/src/shaders/draw_depthstencil_path.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_path.glsl";
pub const PINNED_SOURCE_SHA256: &str = "c8a758cbc337d9ef33cf22703bd41eabc5ab4a08323ff9ae3b65a610ef0951e4";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_depthstencil_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 563;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22736;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_depthstencil_path_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
