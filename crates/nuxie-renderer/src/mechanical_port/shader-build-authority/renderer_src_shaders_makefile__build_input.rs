//! Complete source-owner translation of renderer/src/shaders/Makefile.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/makefile.rs"]
pub mod executable_translation;
#[path = "../source/renderer/src/shaders/minify_py.rs"]
mod minify_py;

pub const PINNED_UPSTREAM_COMMIT: &str = "625454e362a27bb3168f00cd87e9487f54d39338";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/Makefile";
pub const PINNED_SOURCE_SHA256: &str =
    "d273d70ff57b93e38dd35e898640cf3a008ad4693d00467bc418d5b22d735345";
pub const PINNED_SOURCE_LINE_COUNT: usize = 515;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22748;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_makefile.source");
// Historical complete artifact capture; this sync regenerates supported shader families.
pub const FROZEN_GENERATED_ARTIFACT_COUNT: usize = 526;
pub const EXECUTION_CONTRACT: &str = "Execute the independently compiled complete Rust translation or the pinned upstream Makefile with the frozen toolchain; compare emitted bytes with the retained shader inputs and backend import checksums before replacing artifacts.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
