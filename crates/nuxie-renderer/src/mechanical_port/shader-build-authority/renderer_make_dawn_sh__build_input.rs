//! Complete source-owner translation of renderer/make_dawn.sh.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/make_dawn.sh";
pub const PINNED_SOURCE_SHA256: &str =
    "d858f043c468c9c256e985b3a8a81f62dbd884d79c9ef7000cbcd7c884da7aa0";
pub const PINNED_SOURCE_LINE_COUNT: usize = 51;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 1_340;
pub const PINNED_SOURCE: &str = include_str!("source/renderer_make_dawn.sh");
pub const DAWN_REVISION: &str = "211333b2e3e429c3508f25c81c547f602adf448c";
pub const REPOSITORY: &str = "https://dawn.googlesource.com/dawn";
pub const GN_ARGS: &str = "is_debug=false dawn_complete_static_libs=true use_custom_libcxx=false dawn_use_swiftshader=false angle_enable_swiftshader=false";
pub const NINJA_TARGETS: &[&str] = &["webgpu_dawn_static", "cpp", "proc_static"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceStep {
    pub lines: &'static str,
    pub operation: &'static str,
    pub failure: &'static str,
}

pub const SOURCE_STEPS: &[SourceStep] = &[
    SourceStep { lines: "3", operation: "set -e", failure: "stop on first failing command" },
    SourceStep { lines: "5-15", operation: "create dependencies; clone Dawn if absent, otherwise fetch origin", failure: "leave prior checkout but stop" },
    SourceStep { lines: "17-24", operation: "enter Dawn; reset tracked worktree; checkout exact DAWN_REVISION; copy standalone.gclient; gclient sync -f -D", failure: "stop before patch/build" },
    SourceStep { lines: "26-43", operation: "apply exact inline build/config/compiler/BUILD.gn warning-suppression patch", failure: "stop before GN generation" },
    SourceStep { lines: "45-47", operation: "generate out/release using exact GN_ARGS", failure: "stop before compilation" },
    SourceStep { lines: "49-51", operation: "ninja -C out/release -j20 exact NINJA_TARGETS", failure: "no successful dependency root" },
];

pub const WARNING_PATCH_DEFINE: &str =
    "_SILENCE_CXX20_OLD_SHARED_PTR_ATOMIC_SUPPORT_DEPRECATION_WARNING";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
const _: [(); 6] = [(); SOURCE_STEPS.len()];
