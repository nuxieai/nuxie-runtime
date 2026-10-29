//! Exact generated-input translation of renderer/src/shaders/render_atlas.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "6285c6fee237e464e21a19e75017501691cc3472";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/render_atlas.glsl";
pub const PINNED_SOURCE_SHA256: &str = "71c6278a2dd1bee4353c7a9b34b2a187ce1a5429242a240b2d5305b28def4793";
pub const OWNERSHIP_UNIT: &str = "shader:source:render_atlas";
pub const PINNED_SOURCE_LINE_COUNT: usize = 257;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 7462;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_render_atlas_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
