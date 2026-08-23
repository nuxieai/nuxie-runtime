//! Complete source-owner translation of renderer/make_swiftshader.sh.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/make_swiftshader.sh";
pub const PINNED_SOURCE_SHA256: &str =
    "c046e447bd6d0753829b69e0824679b07ed4f0329bad7c651b2b917edf1aabe2";
pub const PINNED_SOURCE_LINE_COUNT: usize = 19;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 376;
pub const PINNED_SOURCE: &str = include_str!("source/renderer_make_swiftshader.sh");
pub const REPOSITORY: &str = "https://github.com/google/swiftshader.git";
pub const SOURCE_REVISION: &str = "origin/main";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceStep {
    pub lines: &'static str,
    pub operation: &'static str,
    pub failure: &'static str,
}

pub const SOURCE_STEPS: &[SourceStep] = &[
    SourceStep { lines: "3", operation: "set -e", failure: "stop on first failing command" },
    SourceStep { lines: "5-15", operation: "create dependencies; clone SwiftShader if absent, otherwise fetch origin and checkout moving origin/main", failure: "stop without a dependency root" },
    SourceStep { lines: "17-18", operation: "enter pre-existing swiftshader/build and configure parent with cmake", failure: "stop before build" },
    SourceStep { lines: "19", operation: "cmake --build . --parallel", failure: "no successful software Vulkan root" },
];

pub const SOURCE_PIN_STATUS: &str = "moving-source-authority; diagnostic-only until a separate consumer freezes the resolved revision";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
const _: [(); 4] = [(); SOURCE_STEPS.len()];
