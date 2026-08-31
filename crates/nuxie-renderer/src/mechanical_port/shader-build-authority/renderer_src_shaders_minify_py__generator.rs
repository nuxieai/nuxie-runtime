//! Complete source-owner translation of renderer/src/shaders/minify.py.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/minify_py.rs"]
pub mod executable_translation;

pub const PINNED_UPSTREAM_COMMIT: &str = "2b2203f45a67f813cb662272962192ecfdfd923e";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/minify.py";
pub const PINNED_SOURCE_SHA256: &str =
    "bc6f3cb877ff8af9c73177d06704ac067a5f6d9a1321fc5edd5ac429d33791b1";
pub const PINNED_SOURCE_LINE_COUNT: usize = 642;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 33051;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_minify_py.source");
pub const OUTPUT_STAGES: &[&str] = &[
    "minify-export",
    "minified-source:<root>.minified<ext>",
    "minify-header",
];
pub const EXECUTION_CONTRACT: &str = "The independently compiled complete Rust translation retains argument parsing, lexer, preprocessor, symbol renaming, source emission, and failure behavior; pinned Python output remains the generated-byte oracle.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
