//! Exact-source renderer implementations behind the `nuxie-render-api` trait boundary.
//!
//! Every product renderer is selected explicitly. This crate contains no
//! legacy Rust-WGPU renderer and does not provide automatic backend fallback.

use nuxie_render_api::authored_ore_shader;
pub mod deferred;
mod renderer_types;
mod stack_vector;
pub use renderer_types::{BackendWorkMetrics, RenderMode, RendererError};

#[cfg(any(
    feature = "native-vulkan-experimental",
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2",
    feature = "renderer-metal"
))]
mod exact_source_adapter;

#[cfg(any(
    feature = "native-ore-vulkan-experimental",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2"
))]
mod exact_gpu_canvas;

#[cfg(feature = "renderer-vulkan")]
mod native_vulkan;
#[cfg(feature = "renderer-vulkan")]
pub use native_vulkan::{NativeVulkanFactory, NativeVulkanFrame};
#[cfg(all(feature = "renderer-vulkan", target_os = "android"))]
pub use native_vulkan::{NativeVulkanPresentation, NativeVulkanSurfaceAdmission};

#[cfg(feature = "renderer-webgpu")]
mod native_webgpu;
#[cfg(all(
    feature = "renderer-webgpu",
    target_arch = "wasm32",
    target_os = "unknown"
))]
pub use native_webgpu::ExternalImageTextures;
#[cfg(feature = "renderer-webgpu")]
pub use native_webgpu::{NativeWebGpuFactory, NativeWebGpuFrame};

#[cfg(any(
    test,
    all(
        feature = "renderer-webgpu",
        target_arch = "wasm32",
        target_os = "unknown"
    )
))]
mod external_image;

#[cfg(all(
    feature = "renderer-webgl2",
    target_arch = "wasm32",
    target_os = "unknown"
))]
mod native_webgl2;
#[cfg(all(
    feature = "renderer-webgl2",
    target_arch = "wasm32",
    target_os = "unknown"
))]
pub use native_webgl2::{WebGl2Factory, WebGl2Frame};

#[cfg(all(
    feature = "webgl2-test-support",
    target_arch = "wasm32",
    target_os = "unknown"
))]
pub use mechanical_port::webgl2::scratch_pass_browser_tests::run_scratch_pass_browser_tests;

mod tessellation_relocation;
pub(crate) use tessellation_relocation::relocate_tessellation_logically;
#[cfg(test)]
pub(crate) use tessellation_relocation::relocate_tessellation_logically_with_scratch;

#[cfg(all(feature = "renderer-metal", test))]
mod feather_lut;

include!("native_root.rs");
