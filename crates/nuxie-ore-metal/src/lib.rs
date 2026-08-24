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

// Raw source-shaped modules are the shared translation's member/friend zone.
// Consumers use the controlled public modules below and cannot assemble or
// split intrusive bases, retained arrays, manager links, or ManuallyDrop owner
// graphs directly.
mod mechanical_port;

pub mod bind_group {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_hpp::*;
}
pub mod bind_group_layout {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_layout_hpp::*;
    #[doc(hidden)]
    pub use crate::mechanical_port::source::renderer::src::ore::ore_bind_group_layout_cpp::{
        validateColorRequiresFragment, validateLayoutBasesAgainstBindingMap,
        validateLayoutsAgainstBindingMap,
    };
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
#[cfg(target_vendor = "apple")]
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

/// Backend integration seam for exact concrete ORE pipeline subclasses.
#[doc(hidden)]
pub fn new_pipeline_backend_base(
    manager: gpu_resource::GPUResourceManager,
    desc: &types::PipelineDesc<'_>,
) -> Option<pipeline::Pipeline> {
    use gpu_resource::GpuResourcePayload;

    let mut base = pipeline::Pipeline::new(desc)?;
    base.gpu_resource_mut().install_manager(Some(manager));
    Some(base)
}

/// Backend integration seam for concrete ORE pipeline subclasses whose source
/// constructor explicitly selects the null-manager base overload.
#[doc(hidden)]
pub fn new_pipeline_backend_base_without_manager(
    desc: &types::PipelineDesc<'_>,
) -> Option<pipeline::Pipeline> {
    pipeline::Pipeline::new(desc)
}

/// Exposes the protected source `Context::m_manager` to concrete backend
/// subclasses implemented in a sibling crate.
#[doc(hidden)]
pub fn context_backend_manager(
    context: &context::Context,
) -> Option<gpu_resource::GPUResourceManager> {
    context.state.manager()
}

#[doc(hidden)]
pub fn context_backend_domain(context: &context::Context) -> gpu_resource::ResourceDomain {
    context.state.resourceDomain()
}

/// Clones the Rust safety-sidecar destruction drain into a concrete backend's
/// execution authority. The handle may outlive the ORE Context and must be
/// drained on its owner thread before the backend API context/device teardown.
#[doc(hidden)]
pub fn context_resource_final_release_drain(
    context: &context::Context,
) -> gpu_resource::ResourceFinalReleaseDrain {
    context.state.resourceFinalReleaseDrain()
}

/// Constructs the protected backend-independent Context base for a concrete
/// backend subclass implemented in a sibling crate.
#[doc(hidden)]
pub fn new_context_backend_base(
    features: types::Features,
    manager: Option<gpu_resource::GPUResourceManager>,
) -> context::Context {
    context::Context::new(features, manager)
}

/// Constructs a backend Context whose resources route into a final-release
/// drain already owned by the concrete execution authority.
#[doc(hidden)]
pub fn new_context_backend_base_with_final_release_drain(
    features: types::Features,
    manager: Option<gpu_resource::GPUResourceManager>,
    drain: gpu_resource::ResourceFinalReleaseDrain,
) -> context::Context {
    context::Context::newWithFinalReleaseDrain(features, manager, drain)
}

/// Backend integration seam for exact concrete ORE texture subclasses.
#[doc(hidden)]
pub fn new_texture_backend_base(
    manager: gpu_resource::GPUResourceManager,
    desc: &types::TextureDesc<'_>,
) -> texture::Texture {
    use gpu_resource::GpuResourcePayload;

    let mut base = texture::Texture::new(desc);
    base.gpu_resource_mut().install_manager(Some(manager));
    base
}

/// Backend integration seam for concrete ORE texture subclasses whose source
/// constructor explicitly selects the null-manager base overload.
#[doc(hidden)]
pub fn new_texture_backend_base_without_manager(desc: &types::TextureDesc<'_>) -> texture::Texture {
    texture::Texture::new(desc)
}

/// Backend integration seam for exact concrete ORE texture-view subclasses.
#[doc(hidden)]
pub fn new_texture_view_backend_base(
    manager: gpu_resource::GPUResourceManager,
    texture: gpu_resource::AnyResourceHandle,
    desc: &types::TextureViewDesc<'_>,
) -> texture::TextureView {
    use gpu_resource::GpuResourcePayload;

    let mut base = texture::TextureView::new(texture, desc);
    base.gpu_resource_mut().install_manager(Some(manager));
    base
}

/// Backend integration seam for concrete ORE texture-view subclasses whose
/// source constructor explicitly selects the null-manager base overload.
#[doc(hidden)]
pub fn new_texture_view_backend_base_without_manager(
    texture: gpu_resource::AnyResourceHandle,
    desc: &types::TextureViewDesc<'_>,
) -> texture::TextureView {
    texture::TextureView::new(texture, desc)
}

/// Backend integration seam for exact concrete ORE bind-group subclasses.
#[doc(hidden)]
pub fn new_bind_group_backend_base(
    manager: gpu_resource::GPUResourceManager,
) -> bind_group::BindGroup {
    use gpu_resource::GpuResourcePayload;

    let mut base = bind_group::BindGroup::new();
    base.gpu_resource_mut().install_manager(Some(manager));
    base
}

/// Backend integration seam for concrete ORE bind-group subclasses whose
/// source constructor explicitly selects the null-manager base overload.
#[doc(hidden)]
pub fn new_bind_group_backend_base_without_manager() -> bind_group::BindGroup {
    bind_group::BindGroup::new()
}

/// Installs the source-owned resource graph captured by a concrete context.
#[doc(hidden)]
pub fn install_bind_group_backend_parts(
    base: &mut bind_group::BindGroup,
    dynamic_offset_count: u32,
    layout: Option<gpu_resource::AnyResourceHandle>,
    retained_buffers: Vec<gpu_resource::AnyResourceHandle>,
    retained_views: Vec<gpu_resource::AnyResourceHandle>,
    retained_samplers: Vec<gpu_resource::AnyResourceHandle>,
) {
    base.m_dynamicOffsetCount = dynamic_offset_count;
    base.m_layoutRef = layout;
    base.m_retainedBuffers = retained_buffers;
    base.m_retainedViews = retained_views;
    base.m_retainedSamplers = retained_samplers;
}

/// Installs the source non-owning `BindGroup::m_context` back-pointer.
#[doc(hidden)]
pub fn install_bind_group_backend_context(
    base: &mut bind_group::BindGroup,
    context: &context::Context,
) {
    base.m_context = std::sync::Arc::downgrade(&context.state);
}

/// Installs the protected source members authored by
/// `Context::makeBindGroupLayout`.
#[doc(hidden)]
pub fn install_bind_group_layout_backend_parts(
    base: &mut bind_group_layout::BindGroupLayout,
    context: &context::Context,
    group_index: u32,
    entries: Vec<types::BindGroupLayoutEntry>,
) {
    base.m_groupIndex = group_index;
    base.m_entries = entries;
    base.m_context = std::sync::Arc::downgrade(&context.state);
}

/// Backend integration seam for exact concrete ORE render-pass subclasses.
#[doc(hidden)]
pub fn new_render_pass_backend_base(context: &context::Context) -> render_pass::RenderPass {
    render_pass::RenderPass::new(std::sync::Arc::downgrade(&context.state))
}

/// Backend integration seam for the source default `RenderPass` constructor,
/// whose context back-pointer is null.
#[doc(hidden)]
pub fn new_render_pass_backend_base_without_context() -> render_pass::RenderPass {
    render_pass::RenderPass::new(std::sync::Weak::new())
}

/// Protected RenderPass seams used by concrete backends implemented in a
/// sibling crate. These are direct field/method projections, not alternate
/// behavior.
#[doc(hidden)]
pub fn render_pass_check_pipeline_compat(
    pass: &render_pass::RenderPass,
    pipeline: &pipeline::Pipeline,
) -> bool {
    pass.checkPipelineCompat(Some(pipeline))
}

#[doc(hidden)]
pub fn render_pass_is_finished(pass: &render_pass::RenderPass) -> bool {
    pass.m_finished
}

#[doc(hidden)]
pub fn render_pass_set_finished(pass: &mut render_pass::RenderPass, finished: bool) {
    pass.m_finished = finished;
}

#[doc(hidden)]
pub fn render_pass_has_context(pass: &render_pass::RenderPass) -> bool {
    pass.m_context.upgrade().is_some()
}

#[doc(hidden)]
pub fn render_pass_clear_bound_groups(pass: &mut render_pass::RenderPass) {
    for group in &mut pass.m_boundGroups {
        *group = None;
    }
}

#[doc(hidden)]
pub fn render_pass_clear_context(pass: &mut render_pass::RenderPass) {
    pass.m_context = std::sync::Weak::new();
}

#[doc(hidden)]
pub fn render_pass_depth_format(pass: &render_pass::RenderPass) -> types::TextureFormat {
    pass.m_depthFormat
}

#[doc(hidden)]
pub fn render_pass_retain_bound_group(
    pass: &mut render_pass::RenderPass,
    group_index: u32,
    group: gpu_resource::AnyResourceHandle,
) {
    pass.m_boundGroups[group_index as usize] = Some(group);
}

#[doc(hidden)]
pub fn render_pass_install_attachment_metadata(
    pass: &mut render_pass::RenderPass,
    color_formats: [types::TextureFormat; 4],
    color_count: u32,
    depth_format: types::TextureFormat,
    has_depth: bool,
    sample_count: u32,
) {
    pass.m_colorFormats = color_formats;
    pass.m_colorCount = color_count;
    pass.m_depthFormat = depth_format;
    pass.m_hasDepth = has_depth;
    pass.m_sampleCount = sample_count;
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

/// Backend integration seam for concrete ORE buffer subclasses whose source
/// constructor explicitly selects the null-manager base overload.
#[doc(hidden)]
pub fn new_buffer_backend_base_without_manager(
    size: u32,
    usage: types::BufferUsage,
) -> buffer::Buffer {
    buffer::Buffer::new(size, usage)
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
