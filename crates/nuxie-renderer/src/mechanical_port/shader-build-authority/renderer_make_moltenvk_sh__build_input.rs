//! Complete source-owner translation of renderer/make_moltenvk.sh.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/make_moltenvk.sh";
pub const PINNED_SOURCE_SHA256: &str =
    "10e70a3f5f378fff23f09ffc11d853c48ddc044fd310f2b3fb71a79b5683f80b";
pub const PINNED_SOURCE_LINE_COUNT: usize = 22;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 569;
pub const PINNED_SOURCE: &str = include_str!("source/renderer_make_moltenvk.sh");
pub const REPOSITORY: &str = "https://github.com/rive-app/MoltenVK.git";
pub const SOURCE_BRANCH: &str = "origin/VK_EXT_rasterization_order_attachment_access";
pub const FROZEN_RESOLVED_REVISION: &str = "7de494443641fc4f81d8232fe379c336face30ab";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceStep {
    pub lines: &'static str,
    pub operation: &'static str,
    pub failure: &'static str,
}

pub const SOURCE_STEPS: &[SourceStep] = &[
    SourceStep { lines: "3", operation: "set -e", failure: "stop on first failing command" },
    SourceStep { lines: "5-13", operation: "create dependencies and clone rive-app/MoltenVK only when absent", failure: "stop without a dependency root" },
    SourceStep { lines: "15-18", operation: "enter MoltenVK and fetch macOS dependencies", failure: "stop before checkout/build" },
    SourceStep { lines: "20-21", operation: "checkout moving SOURCE_BRANCH; preparation resolves it to FROZEN_RESOLVED_REVISION", failure: "reject any revision other than the separately frozen source root" },
    SourceStep { lines: "22", operation: "xcodebuild MoltenVK Package (macOS only), Release", failure: "no successful Vulkan platform root" },
];

pub const XCODE_PROJECT: &str = "MoltenVKPackaging.xcodeproj";
pub const XCODE_SCHEME: &str = "MoltenVK Package (macOS only)";
pub const XCODE_CONFIGURATION: &str = "Release";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
const _: [(); 5] = [(); SOURCE_STEPS.len()];
