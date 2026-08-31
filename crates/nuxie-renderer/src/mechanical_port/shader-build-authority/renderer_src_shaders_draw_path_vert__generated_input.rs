//! Exact generated-input translation of renderer/src/shaders/draw_path.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "2b2203f45a67f813cb662272962192ecfdfd923e";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path.vert";
pub const PINNED_SOURCE_SHA256: &str = "fbf1a2dcc7674eaf044275476c402db700d7de3a4f74fc4ac475b051e451f326";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 520;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 18139;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_path_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
