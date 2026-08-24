// Native-only crate root. Keep this surface limited to source-mechanical CPU
// support and the native Metal product graph; WebGPU/Naga are selected only
// by the explicit `rust-wgpu` feature.

#[cfg(test)]
pub(crate) fn live_metal_test_unavailable(context: &str) {
    if std::env::var_os("NUXIE_REQUIRE_LIVE_METAL_TESTS").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        panic!("required live Metal test resource is unavailable: {context}");
    }
}

mod draw;
#[cfg(test)]
mod feather_lut;
mod gpu;
mod gr_triangulator;
#[allow(dead_code)]
mod intersection_board;

#[cfg(all(
    feature = "native-metal-experimental",
    any(target_os = "ios", target_os = "macos")
))]
mod native_apple_surface;

#[cfg(any(
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "native-webgl2-experimental",
    all(
        feature = "native-metal-experimental",
        any(
            target_os = "ios",
            target_os = "macos",
            target_os = "tvos",
            target_os = "visionos"
        )
    )
))]
mod mechanical_port;

#[cfg(all(
    feature = "native-metal-experimental",
    any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "visionos"
    )
))]
mod native_metal;

#[cfg(all(
    feature = "native-metal-experimental",
    any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "visionos"
    )
))]
use mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm as mechanical_metal_implementation;

#[cfg(all(
    feature = "native-metal-experimental",
    any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "visionos"
    )
))]
pub use native_metal::{
    NativeMetalContextOptions, NativeMetalDrawableFrame, NativeMetalExecutionInventory,
    NativeMetalFactory, NativeMetalFrame, NativeMetalFrameOutput,
    NativeMetalSynthesizedFailureType, ShaderCompilationMode,
};

#[cfg(all(
    feature = "native-ore-metal-experimental",
    feature = "native-metal-experimental",
    any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "visionos"
    )
))]
pub use native_metal::NativeMetalRenderCanvas;
