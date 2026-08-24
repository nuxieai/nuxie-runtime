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

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/premake5_pls_renderer.lua";
pub const PINNED_SOURCE_SHA256: &str =
    "e00b0483a6c09608ef3ad8c61f1b30b3ef00e78722bc3e1228db7ee741d22c84";
pub const PINNED_SOURCE_LINE_COUNT: usize = 638;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 21_056;
pub const PINNED_SOURCE: &str = include_str!("source/renderer_premake5_pls_renderer.lua");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityOccurrence {
    pub symbol: &'static str,
    pub count: usize,
    pub lines: &'static str,
}

pub const CONFIGURATION_AUTHORITIES: &[AuthorityOccurrence] = &[
    AuthorityOccurrence { symbol: "RIVE_RUNTIME_DIR", count: 2, lines: "3,199" },
    AuthorityOccurrence { symbol: "RIVE_VULKAN", count: 1, lines: "24" },
    AuthorityOccurrence { symbol: "VK_NO_PROTOTYPES", count: 1, lines: "25" },
    AuthorityOccurrence { symbol: "RIVE_DESKTOP_GL", count: 3, lines: "37,38,40" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_GL", count: 6, lines: "45,64,67,96,120,164" },
    AuthorityOccurrence { symbol: "RIVE_ORE", count: 12, lines: "45,50,54,72,77,86,96,103,120,129,159,164" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_METAL", count: 1, lines: "54" },
    AuthorityOccurrence { symbol: "RIVE_OBJC_EXCEPTIONS", count: 1, lines: "61" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_D3D11", count: 1, lines: "72" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_D3D12", count: 1, lines: "72" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_RHI", count: 1, lines: "77" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_WGPU", count: 5, lines: "86,100,103,156,159" },
    AuthorityOccurrence { symbol: "RIVE_ANDROID", count: 1, lines: "91" },
    AuthorityOccurrence { symbol: "ORE_BACKEND_VK", count: 4, lines: "112,120,125,129" },
    AuthorityOccurrence { symbol: "RIVE_DAWN", count: 1, lines: "147" },
    AuthorityOccurrence { symbol: "RIVE_WEBGL", count: 1, lines: "152" },
    AuthorityOccurrence { symbol: "RIVE_WEBGPU", count: 1, lines: "179" },
    AuthorityOccurrence { symbol: "RIVE_WAGYU", count: 1, lines: "192" },
    AuthorityOccurrence { symbol: "RIVE_WAGYU_PORT", count: 4, lines: "198,634,635,636" },
    AuthorityOccurrence { symbol: "RIVE_BUILD_OUT", count: 1, lines: "210" },
    AuthorityOccurrence { symbol: "RIVE_RAW_SHADERS", count: 1, lines: "241" },
    AuthorityOccurrence { symbol: "RIVE_OPTICK_URL", count: 1, lines: "316" },
    AuthorityOccurrence { symbol: "RIVE_OPTICK_VERSION", count: 1, lines: "316" },
    AuthorityOccurrence { symbol: "RIVE_MICROPROFILE_URL", count: 1, lines: "320" },
    AuthorityOccurrence { symbol: "RIVE_MICROPROFILE_VERSION", count: 1, lines: "320" },
    AuthorityOccurrence { symbol: "RIVE_DECODERS", count: 1, lines: "575" },
    AuthorityOccurrence { symbol: "RIVE_KTX2", count: 1, lines: "580" },
    AuthorityOccurrence { symbol: "RIVE_BC_DECODER", count: 1, lines: "588" },
    AuthorityOccurrence { symbol: "RIVE_ASTC_DECODER", count: 1, lines: "593" },
    AuthorityOccurrence { symbol: "RIVE_ETC_DECODER", count: 1, lines: "598" },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRule {
    pub lines: &'static str,
    pub condition: &'static str,
    pub effects: &'static [&'static str],
}

pub const SOURCE_RULES: &[SourceRule] = &[
    SourceRule { lines: "6-29", condition: "option:with_vulkan", effects: &["pin Vulkan-Headers vulkan-sdk-1.4.321", "pin VulkanMemoryAllocator v3.3.0", "define RIVE_VULKAN", "define VK_NO_PROTOTYPES", "define VMA_STATIC_VULKAN_FUNCTIONS=0", "define VMA_DYNAMIC_VULKAN_FUNCTIONS=1"] },
    SourceRule { lines: "31-33", condition: "system:windows && !for_unreal", effects: &["pin DirectX-Headers v1.615.0"] },
    SourceRule { lines: "35-40", condition: "system:windows|macosx|linux", effects: &["define RIVE_DESKTOP_GL"] },
    SourceRule { lines: "43-129", condition: "platform/canvas backend matrix", effects: &["define exact ORE_BACKEND_GL/METAL/D3D11/D3D12/RHI/WGPU/VK set", "define RIVE_ORE iff authored backend rule does", "define RIVE_ANDROID on Android", "define RIVE_OBJC_EXCEPTIONS=1 only on Apple option"] },
    SourceRule { lines: "133-147", condition: "option:with-dawn", effects: &["define RIVE_DAWN"] },
    SourceRule { lines: "150-164", condition: "system:emscripten", effects: &["define RIVE_WEBGL", "select ORE_BACKEND_WGPU for Wagyu canvas", "select ORE_BACKEND_GL for non-no_gl canvas"] },
    SourceRule { lines: "167-180", condition: "option:with-webgpu", effects: &["define RIVE_WEBGPU=<webgpu-version 1|2>"] },
    SourceRule { lines: "184-201", condition: "option:with_wagyu", effects: &["reject webgpu-version < 2", "define RIVE_WAGYU", "add Wagyu include for compile database", "otherwise construct exact --use-port webgpu-port.py:wagyu=true"] },
    SourceRule { lines: "204-253", condition: "shader build bootstrap", effects: &["declare no_gl and raw_shaders", "pin dabeaz/ply 3.11", "derive host CPU count", "construct make -C src/shaders -j<N> OUT=<generated>", "append --human-readable and RIVE_RAW_SHADERS for raw", "append --msvc for raw MSVC"] },
    SourceRule { lines: "255-285", condition: "shader output matrix", effects: &["select exact Apple metallib target by OS/variant", "select d3d on Windows", "select spirv for Vulkan/Dawn/WebGPU", "select wgsl for Dawn/WebGPU", "pass WGSL_FLAGS=--raw for raw WGSL"] },
    SourceRule { lines: "288-296", condition: "shader execution", effects: &["print command", "execute once", "fail closed on nonzero result"] },
    SourceRule { lines: "298-320", condition: "project optional dependencies", effects: &["declare nop-obj-c/no-rive-decoders/universal-release/no_ffp_contract", "pin optional Optick and MicroProfile from configured URL/version"] },
    SourceRule { lines: "323-340", condition: "project:rive_pls_renderer", effects: &["StaticLib", "include include/glad/src/../include/generated", "fatal warnings", "add generic renderer, shader, header, ore binding-map, and ore bind-group-layout sources"] },
    SourceRule { lines: "342-357", condition: "optional tools and Vulkan", effects: &["include Optick/MicroProfile", "include Vulkan/VMA", "add src/vulkan/*.cpp"] },
    SourceRule { lines: "358-376", condition: "compiler/platform options", effects: &["include DirectX headers on non-Unreal Windows", "-Wshorten-64-to-32 outside MSVC", "non-Windows floating-point and psABI flags unless no_ffp_contract"] },
    SourceRule { lines: "380-394", condition: "non-iOS GL", effects: &["add exact six shared GL implementation files", "add src/ore/gl/*.cpp for canvas non-Unreal"] },
    SourceRule { lines: "397-413", condition: "desktop/Android GL implementation", effects: &["desktop adds WebGL PLS, RW-texture PLS, EGL/GLES/glad loader", "Android adds GLES extension loader and native EXT PLS"] },
    SourceRule { lines: "416-469", condition: "Apple ObjC and canvas matrix", effects: &["enable ARC", "add Metal renderer and ORE sources", "enable ObjC exceptions twice exactly under identical file filter", "add macOS ORE GL ObjC wrappers"] },
    SourceRule { lines: "473-500", condition: "D3D/WebGPU/Wagyu ORE", effects: &["add D3D11/D3D12 ORE on Windows", "add WGPU ORE for WebGPU/Dawn", "add GL or WGPU ORE on authored Android/Emscripten branches"] },
    SourceRule { lines: "504-539", condition: "Vulkan ORE coexistence", effects: &["add Vulkan ORE on Android", "add GL alongside Android Vulkan when enabled", "add Vulkan+GL ORE on Linux", "add Vulkan ORE on macOS/Windows", "add GL ORE on Emscripten non-Wagyu"] },
    SourceRule { lines: "542-557", condition: "WebGPU renderer", effects: &["include Dawn generated headers for with-dawn", "add src/webgpu/*.cpp and GL load-store actions for WebGPU/Dawn"] },
    SourceRule { lines: "560-580", condition: "fallback/decoder base", effects: &["add metal_nop for nop-obj-c", "always include decoders/include", "define RIVE_DECODERS unless disabled", "define RIVE_KTX2 unless decoder or KTX2 disabled"] },
    SourceRule { lines: "584-610", condition: "decoder families and Windows", effects: &["mirror BC/ASTC/ETC decoder defines", "force Windows x64", "add D3D/D3D11/D3D12 renderer sources outside Unreal"] },
    SourceRule { lines: "613-629", condition: "Emscripten", effects: &["always add pls_impl_webgl.cpp", "add --use-port=emdawnwebgpu only for WebGPU v2 non-Wagyu"] },
    SourceRule { lines: "632-637", condition: "RIVE_WAGYU_PORT is set", effects: &["apply identical Wagyu port to compile and link options"] },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortBuild {
    Vulkan {
        platform: NativeVulkanPlatform,
        no_gl: bool,
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
        PortBuild::Vulkan { platform, no_gl } => match (platform, no_gl) {
            (NativeVulkanPlatform::Android, true) => ExactPortSelection {
                defines: &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "RIVE_ANDROID", "ORE_BACKEND_VK"],
                shader_targets: &["spirv"],
                backend_sources: &["src/vulkan/*.cpp"],
                ore_sources: &["src/ore/*.cpp", "src/ore/vulkan/*.cpp"],
                required_port: None,
            },
            (NativeVulkanPlatform::Android, false) => ExactPortSelection {
                defines: &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "RIVE_ANDROID", "ORE_BACKEND_VK", "ORE_BACKEND_GL", "RIVE_ORE"],
                shader_targets: &["spirv"],
                backend_sources: &["src/vulkan/*.cpp"],
                ore_sources: &["src/ore/*.cpp", "src/ore/vulkan/*.cpp", "src/ore/gl/*.cpp"],
                required_port: None,
            },
            (NativeVulkanPlatform::Linux, _) => ExactPortSelection {
                defines: &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "ORE_BACKEND_VK", "ORE_BACKEND_GL", "RIVE_ORE"],
                shader_targets: &["spirv"],
                backend_sources: &["src/vulkan/*.cpp"],
                ore_sources: &["src/ore/*.cpp", "src/ore/vulkan/*.cpp", "src/ore/gl/*.cpp"],
                required_port: None,
            },
            (NativeVulkanPlatform::MacOS | NativeVulkanPlatform::Windows, _) => ExactPortSelection {
                defines: &["RIVE_VULKAN", "VK_NO_PROTOTYPES", "VMA_STATIC_VULKAN_FUNCTIONS=0", "VMA_DYNAMIC_VULKAN_FUNCTIONS=1", "ORE_BACKEND_VK", "RIVE_ORE"],
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

const _: [(); 30] = [(); CONFIGURATION_AUTHORITIES.len()];
const _: [(); 25] = [(); SOURCE_RULES.len()];
const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
