//! Exact generated-input translation of renderer/src/shaders/unreal/draw_depthstencil_atlas_blit.usf.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "625454e362a27bb3168f00cd87e9487f54d39338";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/unreal/draw_depthstencil_atlas_blit.usf";
pub const PINNED_SOURCE_SHA256: &str = "415388937a702426f1452fe805eaf85f94fafc4b888ce104b5fc128eb29ee047";
pub const OWNERSHIP_UNIT: &str = "shader:source:unreal_draw_depthstencil_atlas_blit";
pub const PINNED_SOURCE_LINE_COUNT: usize = 25;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 747;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_unreal_draw_depthstencil_atlas_blit_usf__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
