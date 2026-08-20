// Mechanical translation of:
//   renderer/src/ore/metal/ore_bind_group_metal.hpp
//   renderer/src/ore/metal/ore_bind_group_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the pending context/render-pass unit constructs and consumes Metal bind groups"
    )
)]

use crate::bind_group::BindGroup;
use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::metal::buffer::BufferMetal;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::rc::Retained;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::runtime::ProtocolObject;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_metal::{MTLBuffer, MTLSamplerState, MTLTexture};

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct RetainedMetalTexture(Retained<ProtocolObject<dyn MTLTexture>>);

// SAFETY: MTLTexture is retained by Metal and accessed only through its
// immutable binding handle. CPU-side resource mutation is owned by the
// corresponding TextureMetal/TextureViewMetal resource and is not performed
// through this wrapper.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Send for RetainedMetalTexture {}
// SAFETY: Same immutable native-handle invariant as `Send` above.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Sync for RetainedMetalTexture {}

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct RetainedMetalSampler(Retained<ProtocolObject<dyn MTLSamplerState>>);

// SAFETY: MTLSamplerState is immutable after creation and supports concurrent
// retain/release and binding.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Send for RetainedMetalSampler {}
// SAFETY: Same immutable native-handle invariant as `Send` above.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Sync for RetainedMetalSampler {}

/// Metal buffer binding metadata.
///
/// C++ stores a non-owning `BufferMetal*` and keeps the corresponding
/// `rcp<Buffer>` in `BindGroup::m_retainedBuffers`. Rust keeps that one strong
/// owner in the base vector and stores its stable index here. At encode time
/// the index is checked and downcast to `BufferMetal`, then its current backing
/// is resolved; this preserves orphan-on-update behavior without a duplicate
/// logical retain or a dangling raw pointer.
pub struct MTLBufferBinding {
    src_index: usize,
    offset: u32,
    binding: u32,
    hasDynamicOffset: bool,
    vsSlot: u16,
    fsSlot: u16,
}

impl MTLBufferBinding {
    pub(crate) fn new(
        src_index: usize,
        offset: u32,
        binding: u32,
        has_dynamic_offset: bool,
        vs_slot: u16,
        fs_slot: u16,
    ) -> Self {
        Self {
            src_index,
            offset,
            binding,
            hasDynamicOffset: has_dynamic_offset,
            vsSlot: vs_slot,
            fsSlot: fs_slot,
        }
    }

    pub(crate) fn src_index(&self) -> usize {
        self.src_index
    }

    pub(crate) fn offset(&self) -> u32 {
        self.offset
    }

    pub(crate) fn binding(&self) -> u32 {
        self.binding
    }

    pub(crate) fn has_dynamic_offset(&self) -> bool {
        self.hasDynamicOffset
    }

    pub(crate) fn vs_slot(&self) -> u16 {
        self.vsSlot
    }

    pub(crate) fn fs_slot(&self) -> u16 {
        self.fsSlot
    }

    pub(crate) fn source<'a>(&self, group: &'a BindGroupMetal) -> Option<&'a BufferMetal> {
        group
            .base
            .retained_buffer(self.src_index)
            .and_then(|resource| resource.downcast_ref::<BufferMetal>())
    }

    /// Resolve the backing selected at encode time, after any prior update may
    /// have orphaned the previous native buffer.
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn current_buffer(
        &self,
        group: &BindGroupMetal,
    ) -> Option<Retained<ProtocolObject<dyn MTLBuffer>>> {
        self.source(group).map(BufferMetal::current_buffer)
    }
}

/// Metal texture binding metadata and its native handle retained for the
/// bind-group lifetime. The logical `TextureView` owner remains in the base
/// resource vector; this second retain is the native ARC handle required by
/// Metal's stage binding table, matching the C++ field exactly.
pub struct MTLTextureBinding {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    texture: Option<RetainedMetalTexture>,
    vsSlot: u16,
    fsSlot: u16,
}

impl MTLTextureBinding {
    pub(crate) fn new(vs_slot: u16, fs_slot: u16) -> Self {
        Self {
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            texture: None,
            vsSlot: vs_slot,
            fsSlot: fs_slot,
        }
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn with_native(
        texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
        vs_slot: u16,
        fs_slot: u16,
    ) -> Self {
        Self {
            texture: texture.map(RetainedMetalTexture),
            vsSlot: vs_slot,
            fsSlot: fs_slot,
        }
    }

    pub(crate) fn vs_slot(&self) -> u16 {
        self.vsSlot
    }

    pub(crate) fn fs_slot(&self) -> u16 {
        self.fsSlot
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn texture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.texture.as_ref().map(|texture| texture.0.as_ref())
    }
}

/// Metal sampler binding metadata and its retained native state.
pub struct MTLSamplerBinding {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    sampler: Option<RetainedMetalSampler>,
    vsSlot: u16,
    fsSlot: u16,
}

impl MTLSamplerBinding {
    pub(crate) fn new(vs_slot: u16, fs_slot: u16) -> Self {
        Self {
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            sampler: None,
            vsSlot: vs_slot,
            fsSlot: fs_slot,
        }
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn with_native(
        sampler: Option<Retained<ProtocolObject<dyn MTLSamplerState>>>,
        vs_slot: u16,
        fs_slot: u16,
    ) -> Self {
        Self {
            sampler: sampler.map(RetainedMetalSampler),
            vsSlot: vs_slot,
            fsSlot: fs_slot,
        }
    }

    pub(crate) fn vs_slot(&self) -> u16 {
        self.vsSlot
    }

    pub(crate) fn fs_slot(&self) -> u16 {
        self.fsSlot
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn sampler(&self) -> Option<&ProtocolObject<dyn MTLSamplerState>> {
        self.sampler.as_ref().map(|sampler| sampler.0.as_ref())
    }
}

/// Concrete Metal bind group.
///
/// The source `.mm` translation unit is intentionally empty. Construction,
/// layout validation, resource downcasts, and error publication remain in the
/// pending `ContextMetal` unit. This leaf owns only the accepted payload and
/// exposes narrow encode-time accessors for the pending render-pass unit.
pub struct BindGroupMetal {
    // Rust field-drop order mirrors C++ reverse member destruction: samplers,
    // textures, then buffers release before the portable base owners.
    m_mtlSamplers: Vec<MTLSamplerBinding>,
    m_mtlTextures: Vec<MTLTextureBinding>,
    m_mtlBuffers: Vec<MTLBufferBinding>,
    base: BindGroup,
}

impl BindGroupMetal {
    /// Adopt the already validated records from `ContextMetal`.
    ///
    /// UBOs are sorted by WGSL binding before publication, so dynamic offset
    /// arrays and stage emission observe a deterministic record order.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the pending ContextMetal unit publishes accepted bindings"
        )
    )]
    pub(crate) fn from_parts(
        base: BindGroup,
        mut buffers: Vec<MTLBufferBinding>,
        textures: Vec<MTLTextureBinding>,
        samplers: Vec<MTLSamplerBinding>,
    ) -> Self {
        buffers.sort_by_key(MTLBufferBinding::binding);
        Self {
            m_mtlSamplers: samplers,
            m_mtlTextures: textures,
            m_mtlBuffers: buffers,
            base,
        }
    }

    pub fn base(&self) -> &BindGroup {
        &self.base
    }

    pub fn dynamic_offset_count(&self) -> u32 {
        self.base.dynamic_offset_count()
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn dynamicOffsetCount(&self) -> u32 {
        self.dynamic_offset_count()
    }

    pub fn group_index(&self) -> u32 {
        self.base.group_index()
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn groupIndex(&self) -> u32 {
        self.group_index()
    }

    pub(crate) fn buffers(&self) -> &[MTLBufferBinding] {
        &self.m_mtlBuffers
    }

    pub(crate) fn textures(&self) -> &[MTLTextureBinding] {
        &self.m_mtlTextures
    }

    pub(crate) fn samplers(&self) -> &[MTLSamplerBinding] {
        &self.m_mtlSamplers
    }

    /// Adopt this payload into the translated `GPUResource` lifetime owner.
    pub(crate) fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_map::BindingMap;
    use crate::gpu_resource::ResourceHandle;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn buffer_records_sort_by_binding_and_preserve_absent_stage_slots() {
        let base = BindGroup::from_parts(1, None, vec![], vec![], vec![]);
        let group = BindGroupMetal::from_parts(
            base,
            vec![
                MTLBufferBinding::new(1, 8, 9, false, BindingMap::kAbsent, 3),
                MTLBufferBinding::new(0, 0, 2, true, 4, BindingMap::kAbsent),
            ],
            vec![MTLTextureBinding::new(BindingMap::kAbsent, 5)],
            vec![MTLSamplerBinding::new(7, BindingMap::kAbsent)],
        );

        assert_eq!(group.buffers()[0].binding(), 2);
        assert_eq!(group.buffers()[1].binding(), 9);
        assert_eq!(group.buffers()[0].src_index(), 0);
        assert!(group.buffers()[0].has_dynamic_offset());
        assert_eq!(group.buffers()[0].vs_slot(), 4);
        assert_eq!(group.buffers()[0].fs_slot(), BindingMap::kAbsent);
        assert_eq!(group.textures()[0].vs_slot(), BindingMap::kAbsent);
        assert_eq!(group.textures()[0].fs_slot(), 5);
        assert_eq!(group.samplers()[0].vs_slot(), 7);
        assert_eq!(group.samplers()[0].fs_slot(), BindingMap::kAbsent);
    }

    #[test]
    fn buffer_source_index_does_not_claim_an_unrelated_resource_as_metal() {
        let unrelated = ResourceHandle::new(None, 123_u32).erase();
        let base = BindGroup::from_parts(0, None, vec![unrelated], vec![], vec![]);
        let group = BindGroupMetal::from_parts(
            base,
            vec![MTLBufferBinding::new(0, 0, 0, false, 1, 1)],
            vec![],
            vec![],
        );

        assert!(group.buffers()[0].source(&group).is_none());
    }

    #[test]
    fn bind_group_owner_graph_is_thread_safe() {
        assert_send_sync::<BindGroup>();
        assert_send_sync::<BindGroupMetal>();
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn buffer_record_resolves_the_live_backing_without_a_second_logical_owner() {
        use crate::metal::buffer::BufferMetalContextState;
        use crate::types::{Buffer as _, BufferUsage};
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice, MTLResourceOptions};

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            return;
        };
        let initial = device
            .newBufferWithLength_options(8, MTLResourceOptions::StorageModeShared)
            .expect("allocate Metal buffer");
        let buffer = ResourceHandle::new(
            None,
            BufferMetal::with_native_buffer(
                8,
                BufferUsage::uniform,
                device,
                initial,
                BufferMetalContextState::new(),
                None,
            ),
        );
        let base = BindGroup::from_parts(1, None, vec![buffer.clone().erase()], vec![], vec![]);
        let group = BindGroupMetal::from_parts(
            base,
            vec![MTLBufferBinding::new(0, 4, 7, true, 2, 3)],
            vec![],
            vec![],
        );
        let binding = &group.buffers()[0];

        assert_eq!(binding.offset(), 4);
        assert_eq!(buffer.debugging_ref_count(), 2);
        assert!(std::ptr::eq(binding.source(&group).unwrap(), &*buffer));
        let first = binding.current_buffer(&group).expect("first backing");

        buffer.context_state().set_current_serial(1);
        buffer.mark_bound();
        buffer.update(&[1, 2], 3).expect("orphan bound buffer");
        let second = binding.current_buffer(&group).expect("second backing");

        assert_ne!(Retained::as_ptr(&first), Retained::as_ptr(&second));
        assert_eq!(buffer.debugging_ref_count(), 2);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn native_texture_and_sampler_records_retain_exact_binding_handles() {
        use objc2::rc::Weak;
        use objc2_metal::{
            MTLCreateSystemDefaultDevice, MTLDevice, MTLPixelFormat, MTLSamplerDescriptor,
            MTLTextureDescriptor,
        };

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            return;
        };
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        // SAFETY: the texture is two-dimensional with non-zero extents and
        // one mip level, satisfying each descriptor setter's precondition.
        unsafe {
            descriptor.setWidth(1);
            descriptor.setHeight(1);
            descriptor.setMipmapLevelCount(1);
        }
        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .expect("allocate Metal texture");
        let sampler = device
            .newSamplerStateWithDescriptor(&MTLSamplerDescriptor::new())
            .expect("allocate Metal sampler");
        let texture_pointer = Retained::as_ptr(&texture);
        let sampler_pointer = Retained::as_ptr(&sampler);
        let texture_owner = Weak::new(&*texture);
        let sampler_owner = Weak::new(&*sampler);

        let group = BindGroupMetal::from_parts(
            BindGroup::from_parts(
                0,
                None,
                vec![],
                vec![ResourceHandle::new(None, 1_u8).erase()],
                vec![ResourceHandle::new(None, 2_u8).erase()],
            ),
            vec![],
            vec![MTLTextureBinding::with_native(Some(texture.clone()), 4, 5)],
            vec![MTLSamplerBinding::with_native(Some(sampler.clone()), 6, 7)],
        );
        drop(texture);
        drop(sampler);

        assert!(texture_owner.load().is_some());
        assert!(sampler_owner.load().is_some());
        assert_eq!(
            std::ptr::from_ref(group.textures()[0].texture().unwrap()),
            texture_pointer
        );
        assert_eq!(
            std::ptr::from_ref(group.samplers()[0].sampler().unwrap()),
            sampler_pointer
        );
        assert_eq!(group.base().retained_view_count(), 1);
        assert_eq!(group.base().retained_sampler_count(), 1);

        let group = group.into_resource(None);
        assert_eq!(group.debugging_ref_count(), 1);
    }
}
