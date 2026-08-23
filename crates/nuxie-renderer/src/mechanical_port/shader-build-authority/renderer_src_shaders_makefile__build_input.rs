//! Complete source-owner translation of renderer/src/shaders/Makefile.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/minify_py.rs"]
mod minify_py;
#[path = "../source/renderer/src/shaders/makefile.rs"]
pub mod executable_translation;

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/Makefile";
pub const PINNED_SOURCE_SHA256: &str =
    "ec5d0d98d78051e98cda80f92cd67858cb1fb70be64cddd8ad13bcd4ad5f50fc";
pub const PINNED_SOURCE_LINE_COUNT: usize = 502;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22_079;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_makefile.source");
pub const FROZEN_GENERATED_ARTIFACT_COUNT: usize = 520;
pub const EXECUTION_CONTRACT: &str = "Execute the independently compiled complete Rust translation or the pinned upstream Makefile with the frozen toolchain; accept output only when every retained artifact matches docs/backend-port-generated-artifacts.tsv.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
