/*
 * Complete source-owner translation of the pinned renderer/premake5.lua.
 *
 * The file is build and rooted-product authority, not runtime renderer
 * behavior. Every authored project branch and port-specific compile/link
 * effect is retained below without selecting or shipping a backend.
 */

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/premake5.lua";
pub const PINNED_SOURCE_SHA256: &str =
    "8ae326a887fd81dd76cdc56382497a3a3905e5c92bfe3976aed7e333eb1878a1";
pub const PINNED_SOURCE_LINE_COUNT: usize = 335;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 9_716;
pub const PINNED_SOURCE: &str = include_str!("source/renderer_premake5.lua");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityOccurrence {
    pub symbol: &'static str,
    pub count: usize,
    pub lines: &'static str,
}

pub const CONFIGURATION_AUTHORITIES: &[AuthorityOccurrence] = &[
    AuthorityOccurrence {
        symbol: "RIVE_RUNTIME_DIR",
        count: 12,
        lines: "4,5,6,10,21,23,24,84,85,86,89,195",
    },
    AuthorityOccurrence {
        symbol: "RIVE_SKIA",
        count: 1,
        lines: "88",
    },
    AuthorityOccurrence {
        symbol: "RIVE_WINDOWS",
        count: 2,
        lines: "96,227",
    },
    AuthorityOccurrence {
        symbol: "RIVE_WAGYU_PORT",
        count: 3,
        lines: "330,331,332",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRule {
    pub lines: &'static str,
    pub condition: &'static str,
    pub effects: &'static [&'static str],
}

pub const SOURCE_RULES: &[SourceRule] = &[
    SourceRule {
        lines: "1-10",
        condition: "root build inclusion",
        effects: &[
            "load rive_build_config.lua",
            "load premake5_pls_renderer.lua",
            "load runtime, decoders, and GLFW build authorities",
            "load Skia renderer authority only for with-skia",
        ],
    },
    SourceRule {
        lines: "13-67",
        condition: "!with-webgpu => project:path_fiddle",
        effects: &[
            "ConsoleApp depending on rive",
            "include exact renderer/hotload/GLFW/Yoga roots",
            "add exact path_fiddle and shader_hotload files",
            "link renderer, decoders, WebP, HarfBuzz, SheenBidi, Yoga, and Luau",
            "conditionally link PNG/JPEG families",
            "load Vulkan bootstrap project only for with_vulkan",
        ],
    },
    SourceRule {
        lines: "69-108",
        condition: "path_fiddle compiler/options",
        effects: &[
            "Xcode adds Yoga as -isystem",
            "non-MSVC enables -Wshorten-64-to-32",
            "with-skia adds exact includes/defines/libdir/links",
            "Windows forces x64, RIVE_WINDOWS, CRT define, GL/D3D links",
            "optional Optick includes and links",
            "non-Unreal Windows adds DirectX headers",
        ],
    },
    SourceRule {
        lines: "110-157",
        condition: "path_fiddle native platforms and Dawn",
        effects: &[
            "macOS adds ObjC++ context, ARC, GLFW and four frameworks",
            "Linux links GLFW",
            "Dawn adds exact includes, library roots, and five libraries",
            "Dawn Windows adds dxguid; Dawn macOS adds IOSurface",
        ],
    },
    SourceRule {
        lines: "159-185",
        condition: "path_fiddle Emscripten/layout/assets",
        effects: &[
            "emit .js",
            "link USE_GLFW=3 and WebGL minimum/maximum version 2",
            "preload exact zzzgold/rivs root",
            "add index.html",
            "conditionally add Yoga layout",
            "copy HTML inputs to target directory",
        ],
    },
    SourceRule {
        lines: "189-222",
        condition: "with-webgpu|with-dawn => project:webgpu_player",
        effects: &[
            "ConsoleApp with exact runtime/GL/GLFW includes",
            "add webgpu_player.cpp and index.html",
            "link renderer, all decoder families, HarfBuzz, SheenBidi, and Yoga",
        ],
    },
    SourceRule {
        lines: "224-276",
        condition: "webgpu_player native platforms and Dawn",
        effects: &[
            "Windows forces x64, platform defines, and GL/D3D links",
            "macOS adds Dawn helper ObjC++, ARC, GLFW, and four frameworks",
            "Dawn adds exact includes, helper source, library roots, and five libraries",
            "Dawn Windows adds dxguid; Dawn macOS adds IOSurface",
        ],
    },
    SourceRule {
        lines: "278-286",
        condition: "webgpu_player Emscripten base",
        effects: &[
            "emit .js",
            "export main/malloc/free",
            "export ccall/cwrap/HEAPU32 runtime methods",
            "restrict environment to web,shell",
        ],
    },
    SourceRule {
        lines: "288-297",
        condition: "Emscripten WebGPU v1 non-Wagyu",
        effects: &["link -sUSE_WEBGPU legacy library"],
    },
    SourceRule {
        lines: "299-310",
        condition: "Emscripten WebGPU v2 non-Wagyu",
        effects: &[
            "apply --use-port=emdawnwebgpu to compile options",
            "apply --use-port=emdawnwebgpu to link options",
        ],
    },
    SourceRule {
        lines: "312-319",
        condition: "with_rive_layout",
        effects: &["define YOGA_EXPORT=", "include Yoga", "link rive_yoga"],
    },
    SourceRule {
        lines: "321-326",
        condition: "webgpu_player HTML/RIV/JS assets",
        effects: &["copy each asset to the target directory with declared output"],
    },
    SourceRule {
        lines: "328-333",
        condition: "RIVE_WAGYU_PORT is set",
        effects: &[
            "apply identical Wagyu port to compile options",
            "apply identical Wagyu port to link options",
        ],
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootedPlayer {
    WebGl2PathFiddle,
    WebGpuV1Player,
    WebGpuV2Player,
    WebGpuWagyuV2Player,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactRootSelection {
    pub project: &'static str,
    pub emscripten_link_options: &'static [&'static str],
    pub emscripten_build_options: &'static [&'static str],
}

pub const fn exact_root_selection(root: RootedPlayer) -> ExactRootSelection {
    match root {
        RootedPlayer::WebGl2PathFiddle => ExactRootSelection {
            project: "path_fiddle",
            emscripten_link_options: &[
                "-sUSE_GLFW=3",
                "-sMIN_WEBGL_VERSION=2",
                "-sMAX_WEBGL_VERSION=2",
                "--preload-file <zzzgold>/rivs@/",
            ],
            emscripten_build_options: &[],
        },
        RootedPlayer::WebGpuV1Player => ExactRootSelection {
            project: "webgpu_player",
            emscripten_link_options: &[
                "-sEXPORTED_FUNCTIONS=_main,_malloc,_free",
                "-sEXPORTED_RUNTIME_METHODS=ccall,cwrap,HEAPU32",
                "-sENVIRONMENT=web,shell",
                "-sUSE_WEBGPU",
            ],
            emscripten_build_options: &[],
        },
        RootedPlayer::WebGpuV2Player => ExactRootSelection {
            project: "webgpu_player",
            emscripten_link_options: &[
                "-sEXPORTED_FUNCTIONS=_main,_malloc,_free",
                "-sEXPORTED_RUNTIME_METHODS=ccall,cwrap,HEAPU32",
                "-sENVIRONMENT=web,shell",
                "--use-port=emdawnwebgpu",
            ],
            emscripten_build_options: &["--use-port=emdawnwebgpu"],
        },
        RootedPlayer::WebGpuWagyuV2Player => ExactRootSelection {
            project: "webgpu_player",
            emscripten_link_options: &["<RIVE_WAGYU_PORT>"],
            emscripten_build_options: &["<RIVE_WAGYU_PORT>"],
        },
    }
}

const _: [(); 4] = [(); CONFIGURATION_AUTHORITIES.len()];
const _: [(); 13] = [(); SOURCE_RULES.len()];
const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
