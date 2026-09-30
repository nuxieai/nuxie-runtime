//! Complete source-owner translation of renderer/src/shaders/minify.py.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/minify_py.rs"]
pub mod executable_translation;

pub const PINNED_UPSTREAM_COMMIT: &str = "696345630f860112d0496e0d1844e9d7087250d8";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/minify.py";
pub const PINNED_SOURCE_SHA256: &str =
    "1fe1a51e6abea47954a6e41b9fb0b9d2e6b9b676acacc9d352915b7298613664";
pub const PINNED_SOURCE_LINE_COUNT: usize = 569;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 24839;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_minify_py.source");
pub const OUTPUT_STAGES: &[&str] = &[
    "minify-export",
    "minified-source:<root>.minified<ext>",
    "minify-header",
];
pub const EXECUTION_CONTRACT: &str = "The independently compiled complete Rust translation retains argument parsing, lexer, preprocessor, symbol renaming, source emission, and failure behavior; pinned Python output remains the generated-byte oracle.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
