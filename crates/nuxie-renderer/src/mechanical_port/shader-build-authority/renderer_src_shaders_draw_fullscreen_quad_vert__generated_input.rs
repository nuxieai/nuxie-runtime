//! Exact generated-input translation of renderer/src/shaders/draw_fullscreen_quad.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_fullscreen_quad.vert";
pub const PINNED_SOURCE_SHA256: &str = "44e538fa18b814c1643e046b3d3c2a0e2e54bb74521667f989bdbe7a0978253e";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_fullscreen_quad";
pub const PINNED_SOURCE_LINE_COUNT: usize = 21;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 492;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_fullscreen_quad_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
