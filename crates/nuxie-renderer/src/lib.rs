//! Exact-source renderer implementations behind the `nuxie-render-api` trait boundary.
//!
//! Every product renderer is selected explicitly. This crate contains no
//! legacy Rust-WGPU renderer and does not provide automatic backend fallback.

mod renderer_types;
pub use renderer_types::{BackendWorkMetrics, RenderMode, RendererError};

#[cfg(any(
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2"
))]
mod exact_source_adapter;

#[cfg(feature = "renderer-vulkan")]
mod native_vulkan;
#[cfg(feature = "renderer-vulkan")]
pub use native_vulkan::{NativeVulkanFactory, NativeVulkanFrame};

#[cfg(feature = "renderer-webgpu")]
mod native_webgpu;
#[cfg(feature = "renderer-webgpu")]
pub use native_webgpu::{NativeWebGpuFactory, NativeWebGpuFrame};

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

mod tessellation_relocation;
pub(crate) use tessellation_relocation::relocate_tessellation_logically;
#[cfg(test)]
pub(crate) use tessellation_relocation::relocate_tessellation_logically_with_scratch;

#[cfg(all(feature = "renderer-metal", test))]
mod feather_lut;

include!("native_root.rs");
