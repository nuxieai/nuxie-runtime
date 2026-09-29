//! Exact generated-input translation of renderer/src/shaders/draw_mesh.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "9463ff7b5b9a1452d0c32e41390a99cd39b6c946";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_mesh.frag";
pub const PINNED_SOURCE_SHA256: &str = "c5e85eb53d7d5885d9074e3c595ca9690f8b68290c637f3545ae1042f03b14ec";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 233;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 7162;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_mesh_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
