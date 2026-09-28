/*
 * Complete source-owner translation of the pinned renderer/premake5.lua.
 *
 * The file is build and rooted-product authority, not runtime renderer
 * behavior. Every authored project branch and port-specific compile/link
 * effect is retained below without selecting or shipping a backend.
 */

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "5ab9af03eb23e978abccc2dbd047bcde30e3cdbb";
pub const PINNED_SOURCE_PATH: &str = "renderer/premake5.lua";
pub const PINNED_SOURCE_SHA256: &str =
    "ae90147da2d05cbfc559077106ddaaa80d646815a1645b3a266e188c468eefa9";
pub const PINNED_SOURCE_LINE_COUNT: usize = 201;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 5_905;
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
        count: 11,
        lines: "4,5,6,10,21,23,24,84,85,86,89",
    },
    AuthorityOccurrence {
        symbol: "RIVE_SKIA",
        count: 1,
        lines: "88",
    },
    AuthorityOccurrence {
        symbol: "RIVE_WINDOWS",
        count: 1,
        lines: "96",
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
        lines: "69-122",
        condition: "path_fiddle compiler/options",
        effects: &[
            "Xcode adds Yoga as -isystem",
            "non-MSVC enables -Wshorten-64-to-32",
            "with-skia adds exact includes/defines/libdir/links",
            "Windows forces x64, RIVE_WINDOWS, CRT define, GL/D3D links and explicit gdi32/shell32/user32 for lld-link",
            "optional Optick includes and links",
            "non-Unreal Windows adds DirectX headers",
        ],
    },
    SourceRule {
        lines: "124-171",
        condition: "path_fiddle native platforms and Dawn",
        effects: &[
            "macOS adds ObjC++ context, ARC, GLFW and four frameworks",
            "Linux links GLFW",
            "Dawn adds exact includes, library roots, and five libraries",
            "Dawn Windows adds dxguid; Dawn macOS adds IOSurface",
        ],
    },
    SourceRule {
        lines: "173-199",
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
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootedPlayer {
    WebGl2PathFiddle,
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
    }
}

const _: [(); 3] = [(); CONFIGURATION_AUTHORITIES.len()];
const _: [(); 5] = [(); SOURCE_RULES.len()];
const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
