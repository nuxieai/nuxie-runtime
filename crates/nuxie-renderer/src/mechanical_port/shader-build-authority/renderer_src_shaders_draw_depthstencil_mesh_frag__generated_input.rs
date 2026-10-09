//! Exact generated-input translation of renderer/src/shaders/draw_depthstencil_mesh.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_depthstencil_mesh.frag";
pub const PINNED_SOURCE_SHA256: &str = "e62f45bc3e811a504e1bf844ae579fb16173e742e530bffbd459eeff0c8c11d6";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_depthstencil_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 101;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3017;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_depthstencil_mesh_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
