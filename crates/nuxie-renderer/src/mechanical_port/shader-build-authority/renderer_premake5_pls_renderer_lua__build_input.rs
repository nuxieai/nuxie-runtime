/*
 * Complete source-owner translation of the pinned
 * renderer/premake5_pls_renderer.lua build authority.
 *
 * This translation preserves every authored option, configuration/capability
 * symbol, dependency pin, shader-generation decision, source-family rule, and
 * tool/link effect as immutable source-shaped data. It deliberately does not
 * make a shipping-backend choice; product selection remains a later queue.
 */

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "ea15876025689f4d7a8b20b03ab636a78c250763";
pub const PINNED_SOURCE_PATH: &str = "renderer/premake5_pls_renderer.lua";
pub const PINNED_SOURCE_SHA256: &str =
    "914f76343bd26e0c70dd837711b90206bb8e3387d177714eb98473984dff8565";
pub const PINNED_SOURCE_LINE_COUNT: usize = 648;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 21_491;
pub const PINNED_SOURCE: &str = include_str!("source/renderer_premake5_pls_renderer.lua");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityOccurrence {
    pub symbol: &'static str,
    pub count: usize,
    pub lines: &'static str,
}

pub const CONFIGURATION_AUTHORITIES: &[AuthorityOccurrence] = &[
    AuthorityOccurrence { symbol: "WITH_VULKAN_ATOMICS", count: 1, lines: "37" },
    AuthorityOccurrence { symbol: "RIVE_RUNTIME_DIR", count: 2, lines: "3,209" },
    AuthorityOccurrence { symbol: "RIVE_VULKAN", count: 1, lines: "29" },
    AuthorityOccurrence { symbol: "VK_NO_PROTOTYPES", count: 1, lines: "30" },
    AuthorityOccurrence { symbol: "RIVE_DESKTOP_GL", count: 3, lines: "47,48,50" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_GL", count: 6, lines: "55,74,77,106,130,174" },
    AuthorityOccurrence { symbol: "RIVE_ORE", count: 12, lines: "55,60,64,82,87,96,106,113,130,139,169,174" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_METAL", count: 1, lines: "64" },
    AuthorityOccurrence { symbol: "RIVE_OBJC_EXCEPTIONS", count: 1, lines: "71" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_D3D11", count: 1, lines: "82" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_D3D12", count: 1, lines: "82" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_RHI", count: 1, lines: "87" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_WGPU", count: 5, lines: "96,110,113,166,169" },
    AuthorityOccurrence { symbol: "RIVE_ANDROID", count: 1, lines: "101" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_VK", count: 4, lines: "122,130,135,139" },
    AuthorityOccurrence { symbol: "RIVE_DAWN", count: 1, lines: "157" },
    AuthorityOccurrence { symbol: "RIVE_WEBGL", count: 1, lines: "162" },
    AuthorityOccurrence { symbol: "RIVE_WEBGPU", count: 1, lines: "189" },
    AuthorityOccurrence { symbol: "RIVE_WAGYU", count: 1, lines: "202" },
    AuthorityOccurrence { symbol: "RIVE_WAGYU_PORT", count: 4, lines: "208,644,645,646" },
    AuthorityOccurrence { symbol: "RIVE_BUILD_OUT", count: 1, lines: "220" },
    AuthorityOccurrence { symbol: "RIVE_RAW_SHADERS", count: 1, lines: "251" },
    AuthorityOccurrence { symbol: "RIVE_OPTICK_URL", count: 1, lines: "326" },
    AuthorityOccurrence { symbol: "RIVE_OPTICK_VERSION", count: 1, lines: "326" },
    AuthorityOccurrence { symbol: "RIVE_MICROPROFILE_URL", count: 1, lines: "330" },
    AuthorityOccurrence { symbol: "RIVE_MICROPROFILE_VERSION", count: 1, lines: "330" },
    AuthorityOccurrence { symbol: "RIVE_DECODERS", count: 1, lines: "585" },
    AuthorityOccurrence { symbol: "RIVE_KTX2", count: 1, lines: "590" },
    AuthorityOccurrence { symbol: "RIVE_BC_DECODER", count: 1, lines: "598" },
    AuthorityOccurrence { symbol: "RIVE_ASTC_DECODER", count: 1, lines: "603" },
    AuthorityOccurrence { symbol: "RIVE_ETC_DECODER", count: 1, lines: "608" },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRule {
    pub lines: &'static str,
    pub condition: &'static str,
    pub effects: &'static [&'static str],
}

pub const SOURCE_RULES: &[SourceRule] = &[
    SourceRule { lines: "6-39", condition: "option:with_vulkan", effects: &["pin Vulkan-Headers vulkan-sdk-1.4.321", "pin VulkanMemoryAllocator v3.3.0", "define RIVE_VULKAN", "define VK_NO_PROTOTYPES", "define VMA_STATIC_VULKAN_FUNCTIONS=0", "define VMA_DYNAMIC_VULKAN_FUNCTIONS=1"] },
    SourceRule { lines: "34-38", condition: "option:with_vulkan && (system:!android || option:with_android_vulkan_atomics)", effects: &["define WITH_VULKAN_ATOMICS", "embed atomic and clockwiseAtomic SPIR-V"] },
    SourceRule { lines: "41-43", condition: "system:windows && !for_unreal", effects: &["pin DirectX-Headers v1.615.0"] },
    SourceRule { lines: "45-50", condition: "system:windows|macosx|linux", effects: &["define RIVE_DESKTOP_GL"] },
    SourceRule { lines: "53-139", condition: "platform/canvas backend matrix", effects: &["define exact ORE_BACKEND_GL/METAL/D3D11/D3D12/RHI/WGPU/VK set", "define RIVE_ORE iff authored backend rule does", "define RIVE_ANDROID on Android", "define RIVE_OBJC_EXCEPTIONS=1 only on Apple option"] },
    SourceRule { lines: "143-157", condition: "option:with-dawn", effects: &["define RIVE_DAWN"] },
    SourceRule { lines: "160-174", condition: "system:emscripten", effects: &["define RIVE_WEBGL", "select ORE_BACKEND_WGPU for Wagyu canvas", "select ORE_BACKEND_GL for non-no_gl canvas"] },
    SourceRule { lines: "177-190", condition: "option:with-webgpu", effects: &["define RIVE_WEBGPU=<webgpu-version 1|2>"] },
    SourceRule { lines: "194-211", condition: "option:with_wagyu", effects: &["reject webgpu-version < 2", "define RIVE_WAGYU", "add Wagyu include for compile database", "otherwise construct exact --use-port webgpu-port.py:wagyu=true"] },
    SourceRule { lines: "214-263", condition: "shader build bootstrap", effects: &["declare no_gl and raw_shaders", "pin dabeaz/ply 3.11", "derive host CPU count", "construct make -C src/shaders -j<N> OUT=<generated>", "append --human-readable and RIVE_RAW_SHADERS for raw", "append --msvc for raw MSVC"] },
    SourceRule { lines: "265-295", condition: "shader output matrix", effects: &["select exact Apple metallib target by OS/variant", "select d3d on Windows", "select spirv for Vulkan/Dawn/WebGPU", "select wgsl for Dawn/WebGPU", "pass WGSL_FLAGS=--raw for raw WGSL"] },
    SourceRule { lines: "298-306", condition: "shader execution", effects: &["print command", "execute once", "fail closed on nonzero result"] },
    SourceRule { lines: "308-330", condition: "project optional dependencies", effects: &["declare nop-obj-c/no-rive-decoders/universal-release/no_ffp_contract", "pin optional Optick and MicroProfile from configured URL/version"] },
    SourceRule { lines: "333-350", condition: "project:rive_pls_renderer", effects: &["StaticLib", "include include/glad/src/../include/generated", "fatal warnings", "add generic renderer, shader, header, ore binding-map, and ore bind-group-layout sources"] },
    SourceRule { lines: "352-367", condition: "optional tools and Vulkan", effects: &["include Optick/MicroProfile", "include Vulkan/VMA", "add src/vulkan/*.cpp"] },
    SourceRule { lines: "368-386", condition: "compiler/platform options", effects: &["include DirectX headers on non-Unreal Windows", "-Wshorten-64-to-32 outside MSVC", "non-Windows floating-point and psABI flags unless no_ffp_contract"] },
    SourceRule { lines: "390-404", condition: "non-iOS GL", effects: &["add exact six shared GL implementation files", "add src/ore/gl/*.cpp for canvas non-Unreal"] },
    SourceRule { lines: "407-423", condition: "desktop/Android GL implementation", effects: &["desktop adds WebGL PLS, RW-texture PLS, EGL/GLES/glad loader", "Android adds GLES extension loader and native EXT PLS"] },
    SourceRule { lines: "426-479", condition: "Apple ObjC and canvas matrix", effects: &["enable ARC", "add Metal renderer and ORE sources", "enable ObjC exceptions twice exactly under identical file filter", "add macOS ORE GL ObjC wrappers"] },
    SourceRule { lines: "483-510", condition: "D3D/WebGPU/Wagyu ORE", effects: &["add D3D11/D3D12 ORE on Windows", "add WGPU ORE for WebGPU/Dawn", "add GL or WGPU ORE on authored Android/Emscripten branches"] },
    SourceRule { lines: "514-549", condition: "Vulkan ORE coexistence", effects: &["add Vulkan ORE on Android", "add GL alongside Android Vulkan when enabled", "add Vulkan+GL ORE on Linux", "add Vulkan ORE on macOS/Windows", "add GL ORE on Emscripten non-Wagyu"] },
    SourceRule { lines: "552-567", condition: "WebGPU renderer", effects: &["include Dawn generated headers for with-dawn", "add src/webgpu/*.cpp and GL load-store actions for WebGPU/Dawn"] },
    SourceRule { lines: "570-590", condition: "fallback/decoder base", effects: &["add metal_nop for nop-obj-c", "always include decoders/include", "define RIVE_DECODERS unless disabled", "define RIVE_KTX2 unless decoder or KTX2 disabled"] },
    SourceRule { lines: "594-620", condition: "decoder families and Windows", effects: &["mirror BC/ASTC/ETC decoder defines", "force Windows x64", "add D3D/D3D11/D3D12 renderer sources outside Unreal"] },
    SourceRule { lines: "623-639", condition: "Emscripten", effects: &["always add pls_impl_webgl.cpp", "add --use-port=emdawnwebgpu only for WebGPU v2 non-Wagyu"] },
    SourceRule { lines: "642-647", condition: "RIVE_WAGYU_PORT is set", effects: &["apply identical Wagyu port to compile and link options"] },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortBuild {
    Vulkan {
        platform: NativeVulkanPlatform,
        no_gl: bool,
        with_android_vulkan_atomics: bool,
    },
    WebGpuDawnV2,
    WebGpuWagyuV2,
    WebGl2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeVulkanPlatform {
    Android,
    Linux,
    MacOS,
    Windows,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactPortSelection {
    pub defines: &'static [&'static str],
    pub shader_targets: &'static [&'static str],
    pub backend_sources: &'static [&'static str],
    pub ore_sources: &'static [&'static str],
    pub required_port: Option<&'static str>,
}

pub const fn exact_port_selection(port: PortBuild) -> ExactPortSelection {
    match port {
        PortBuild::Vulkan { platform, no_gl, with_android_vulkan_atomics } => match (platform, no_gl) {
            (NativeVulkanPlatform::Android, true) => ExactPortSelection {
                defines: if with_android_vulkan_atomics { &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "RIVE_ANDROID", "ORE_BACKEND_VK", "WITH_VULKAN_ATOMICS"] } else { &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "RIVE_ANDROID", "ORE_BACKEND_VK"] },
                shader_targets: &["spirv"],
                backend_sources: &["src/vulkan/*.cpp"],
                ore_sources: &["src/ore/*.cpp", "src/ore/vulkan/*.cpp"],
                required_port: None,
            },
            (NativeVulkanPlatform::Android, false) => ExactPortSelection {
                defines: if with_android_vulkan_atomics { &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "RIVE_ANDROID", "ORE_BACKEND_VK", "ORE_BACKEND_GL", "RIVE_ORE", "WITH_VULKAN_ATOMICS"] } else { &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "RIVE_ANDROID", "ORE_BACKEND_VK", "ORE_BACKEND_GL", "RIVE_ORE"] },
                shader_targets: &["spirv"],
                backend_sources: &["src/vulkan/*.cpp"],
                ore_sources: &["src/ore/*.cpp", "src/ore/vulkan/*.cpp", "src/ore/gl/*.cpp"],
                required_port: None,
            },
            (NativeVulkanPlatform::Linux, _) => ExactPortSelection {
                defines: &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "ORE_BACKEND_VK", "ORE_BACKEND_GL", "RIVE_ORE", "WITH_VULKAN_ATOMICS"],
                shader_targets: &["spirv"],
                backend_sources: &["src/vulkan/*.cpp"],
                ore_sources: &["src/ore/*.cpp", "src/ore/vulkan/*.cpp", "src/ore/gl/*.cpp"],
                required_port: None,
            },
            (NativeVulkanPlatform::MacOS | NativeVulkanPlatform::Windows, _) => ExactPortSelection {
                defines: &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "ORE_BACKEND_VK", "RIVE_ORE", "WITH_VULKAN_ATOMICS"],
                shader_targets: &["spirv"],
                backend_sources: &["src/vulkan/*.cpp"],
                ore_sources: &["src/ore/*.cpp", "src/ore/vulkan/*.cpp"],
                required_port: None,
            },
        },
        PortBuild::WebGpuDawnV2 => ExactPortSelection {
            defines: &["RIVE_DAWN", "ORE_BACKEND_WGPU", "RIVE_ORE"],
            shader_targets: &["spirv", "wgsl"],
            backend_sources: &["src/webgpu/*.cpp", "src/gl/load_store_actions_ext.cpp"],
            ore_sources: &["src/ore/*.cpp", "src/ore/wgpu/*.cpp"],
            required_port: None,
        },
        PortBuild::WebGpuWagyuV2 => ExactPortSelection {
            defines: &["RIVE_WEBGPU=2", "RIVE_WAGYU", "ORE_BACKEND_WGPU", "RIVE_ORE"],
            shader_targets: &["spirv", "wgsl"],
            backend_sources: &["src/webgpu/*.cpp", "src/gl/load_store_actions_ext.cpp"],
            ore_sources: &["src/ore/*.cpp", "src/ore/wgpu/*.cpp"],
            required_port: Some("--use-port=<runtime>/renderer/src/webgpu/wagyu-port/webgpu-port.py:wagyu=true"),
        },
        PortBuild::WebGl2 => ExactPortSelection {
            defines: &["RIVE_WEBGL", "ORE_BACKEND_GL", "RIVE_ORE"],
            shader_targets: &[],
            backend_sources: &["src/gl/gl_state.cpp", "src/gl/gl_utils.cpp", "src/gl/load_store_actions_ext.cpp", "src/gl/render_buffer_gl_impl.cpp", "src/gl/render_context_gl_impl.cpp", "src/gl/render_target_gl.cpp", "src/gl/pls_impl_webgl.cpp"],
            ore_sources: &["src/ore/*.cpp", "src/ore/gl/*.cpp"],
            required_port: None,
        },
    }
}

const _: [(); 31] = [(); CONFIGURATION_AUTHORITIES.len()];
const _: [(); 26] = [(); SOURCE_RULES.len()];
const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
