//! Complete source-owner translation of renderer/src/shaders/Makefile.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/minify_py.rs"]
mod minify_py;
#[path = "../source/renderer/src/shaders/makefile.rs"]
pub mod executable_translation;

pub const PINNED_UPSTREAM_COMMIT: &str = "1cc2396f0d0d3f6d9c0b16809904e85265f617eb";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/Makefile";
pub const PINNED_SOURCE_SHA256: &str =
    "14756892cfe57c9d64fd2c69a678d0a639d136ecfc46ba4ba4c36d4ab8586620";
pub const PINNED_SOURCE_LINE_COUNT: usize = 515;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22748;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_makefile.source");
pub const FROZEN_GENERATED_ARTIFACT_COUNT: usize = 526;
pub const EXECUTION_CONTRACT: &str = "Execute the independently compiled complete Rust translation or the pinned upstream Makefile with the frozen toolchain; compare emitted bytes with the retained shader inputs and backend import checksums before replacing artifacts.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
