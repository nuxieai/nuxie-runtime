//! Exact generated-input translation of renderer/src/shaders/gradient_packing_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "9463ff7b5b9a1452d0c32e41390a99cd39b6c946";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/gradient_packing_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "2fd7fe601521a8ee83c8c6ff8bc35537607dae26dfc64143578ef130e525c68d";
pub const OWNERSHIP_UNIT: &str = "shader:source:gradient_packing_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 69;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2230;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_gradient_packing_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
