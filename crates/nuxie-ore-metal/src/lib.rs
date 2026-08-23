//! Mechanical Rust port of Rive's ORE interface and Metal adapter.
//!
//! The source correspondence and translation queue are pinned in
//! `docs/metal-port-manifest.toml`. This crate deliberately remains separate
//! from the built-in renderer-platform implementation.

#[cfg(test)]
pub(crate) fn live_metal_test_unavailable(context: &str) {
    if std::env::var_os("NUXIE_REQUIRE_LIVE_METAL_TESTS").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        panic!("required live Metal test resource is unavailable: {context}");
    }
}

pub mod mechanical_port;

pub mod bind_group {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_hpp::*;
}
pub mod bind_group_layout {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_layout_hpp::*;
}
pub mod binding_map {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_binding_map_hpp::*;
}
pub mod buffer {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_buffer_hpp::*;
}
pub mod context {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_context_hpp::*;
}
pub mod gpu_resource {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::*;
}
pub mod metal;
pub mod pipeline {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_pipeline_hpp::*;
}
pub mod render_pass {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_render_pass_hpp::*;
}
pub mod rstb_entry_container {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_rstb_entry_container_hpp::*;
}
pub mod sampler {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_sampler_hpp::*;
}
pub mod shader_module {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_shader_module_hpp::*;
}
pub mod texture {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_texture_hpp::*;
}
pub mod types {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::*;
}

/// Backend integration seam for exact concrete ORE sampler subclasses.
#[doc(hidden)]
pub fn new_sampler_backend_base() -> sampler::Sampler {
    sampler::Sampler::new()
}

/// Backend integration seam for exact concrete ORE shader-module subclasses.
#[doc(hidden)]
pub fn new_shader_module_backend_base() -> shader_module::ShaderModule {
    shader_module::ShaderModule::new()
}

/// Backend integration seam for exact concrete ORE bind-group-layout subclasses.
#[doc(hidden)]
pub fn new_bind_group_layout_backend_base() -> bind_group_layout::BindGroupLayout {
    bind_group_layout::BindGroupLayout::new()
}

/// Backend integration seam for exact concrete `GPUResource` subclasses.
#[doc(hidden)]
pub fn new_gpu_resource_backend_base() -> gpu_resource::GPUResource {
    gpu_resource::GPUResource::new(None)
}

/// Backend integration seam for exact concrete ORE buffer subclasses.
#[doc(hidden)]
pub fn new_buffer_backend_base(
    manager: gpu_resource::GPUResourceManager,
    size: u32,
    usage: types::BufferUsage,
) -> buffer::Buffer {
    use gpu_resource::GpuResourcePayload;

    let mut base = buffer::Buffer::new(size, usage);
    base.gpu_resource_mut().install_manager(Some(manager));
    base
}

/// Backend integration seam for an exact embedded `GPUResourcePool` base.
#[doc(hidden)]
pub fn new_gpu_resource_pool_backend_base(
    manager: gpu_resource::GPUResourceManager,
    max_pool_size: usize,
) -> gpu_resource::GPUResourcePool {
    let mut base = gpu_resource::GPUResource::new(None);
    base.install_manager(Some(manager));
    gpu_resource::GPUResourcePool {
        base: std::mem::ManuallyDrop::new(base),
        members: std::mem::ManuallyDrop::new(gpu_resource::GPUResourcePoolMembers {
            m_maxPoolCount: max_pool_size,
            m_pool: std::mem::ManuallyDrop::new(Default::default()),
        }),
    }
}
