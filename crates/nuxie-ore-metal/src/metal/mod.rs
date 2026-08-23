//! Concrete ORE Metal resource payloads.

pub mod bind_group {
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_bind_group_metal_hpp::*;
}
pub mod buffer {
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_buffer_metal_hpp::*;
}
#[cfg(target_vendor = "apple")]
pub mod context {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_context_metal_hpp::*;
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_context_metal_mm::{
        MetalRenderCanvasBridge, MetalRenderCanvasHost, MetalRiveTextureBridge,
        MetalRiveTextureHost, MetalSubmissionCompletion, NativeTexture,
    };
}
pub mod pipeline {
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_pipeline_metal_hpp::*;
}
#[cfg(target_vendor = "apple")]
pub mod render_pass {
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_render_pass_metal_hpp::*;
}
pub mod sampler {
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_sampler_metal_hpp::*;
}
pub mod shader_module {
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_shader_module_metal_hpp::*;
}
pub mod texture {
    pub use crate::mechanical_port::source::renderer::src::ore::metal::ore_texture_metal_hpp::*;
}
