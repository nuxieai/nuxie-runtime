//! Immutable product view of capabilities derived by the canonical
//! `RenderContextMetal` source owner.

pub(crate) use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::AtomicBarrierType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MetalCapabilitySelection {
    pub(crate) max_texture_size: u32,
    pub(crate) supports_raster_ordering: bool,
    pub(crate) supports_atomic_mode: bool,
    pub(crate) path_id_granularity: u32,
    pub(crate) supports_texture_compression_etc2: bool,
    pub(crate) supports_texture_compression_astc: bool,
    pub(crate) supports_texture_compression_bc: bool,
    pub(crate) atomic_barrier_type: AtomicBarrierType,
}
