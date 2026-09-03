//! renderer/ore/cmd/ore_deferred_resource.hpp at e949498e.
#![allow(non_snake_case)]
use super::{
    ore_command_buffer::{
        OreCommandBuffer, PendingDestroy, SharedIdAllocator, SharedOreCommandBuffer,
    },
    ore_make_recording::{recordBufferUpdate, recordTextureUpload},
};
use crate::cmd::{id_allocator::IdAllocator, live_recorder_registry::recorder_registry};
use crate::{
    bind_group::BindGroup,
    bind_group_layout::BindGroupLayout,
    buffer::{Buffer, BufferApi, BufferUpdateError},
    gpu_resource::{AnyResourceHandle, GPUResource, GpuResourcePayload},
    pipeline::Pipeline,
    sampler::Sampler,
    shader_module::ShaderModule,
    texture::{Texture, TextureApi, TextureUploadError, TextureView},
    types::*,
};
use std::{
    cell::RefCell,
    mem::ManuallyDrop,
    ptr::NonNull,
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

pub struct DeferredResource {
    handle: u32,
    generation: u32,
    stream: Option<NonNull<RefCell<OreCommandBuffer>>>,
    streamIdentity: usize,
    queue: Option<Weak<Mutex<Vec<PendingDestroy>>>>,
    allocator: Option<Weak<Mutex<IdAllocator>>>,
    allocatorIdentity: usize,
}
impl DeferredResource {
    pub fn new(
        handle: u32,
        generation: u32,
        stream: Option<&SharedOreCommandBuffer>,
        allocator: Option<&SharedIdAllocator>,
    ) -> Self {
        Self {
            handle,
            generation,
            stream: stream.map(|s| NonNull::from(s.as_ref())),
            streamIdentity: stream.map_or(0, |s| s.borrow().recorderIdentity()),
            queue: stream.map(|s| s.borrow().destroyQueue()),
            allocator: allocator.map(Arc::downgrade),
            allocatorIdentity: allocator.map_or(0, |a| Arc::as_ptr(a) as usize),
        }
    }
    pub fn clientHandle(&self) -> u32 {
        self.handle
    }
    pub fn recordsInto(&self, stream: &SharedOreCommandBuffer) -> bool {
        self.stream
            .is_some_and(|p| p.as_ptr() == Rc::as_ptr(stream).cast_mut())
    }
    fn withStream(&self, f: impl FnOnce(&mut OreCommandBuffer)) {
        if let Some(stream) = self.stream {
            // Source update/upload are recording-thread operations. ResourceHandle's
            // dispatch enforces that thread; the owning recorder outlives calls.
            // This raw nonowner carries no Rc count into an arbitrary GC finalizer.
            let registry = recorder_registry();
            assert!(
                registry.contains(&self.streamIdentity),
                "update after recorder destruction"
            );
            unsafe {
                f(&mut stream.as_ref().borrow_mut());
            }
        }
    }
}
impl Drop for DeferredResource {
    fn drop(&mut self) {
        let registry = recorder_registry();
        if self.stream.is_some() {
            if !registry.contains(&self.streamIdentity) {
                return;
            }
            if let Some(queue) = self.queue.as_ref().and_then(Weak::upgrade) {
                queue
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(PendingDestroy {
                        handle: self.handle,
                        generation: self.generation,
                        allocator: self.allocator.clone(),
                    });
            }
            return;
        }
        if self.allocator.is_some() && !registry.contains(&self.allocatorIdentity) {
            return;
        }
        if let Some(allocator) = self.allocator.as_ref().and_then(Weak::upgrade) {
            allocator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .release(self.handle, self.generation);
        }
    }
}

macro_rules! deferred_owner {
    ($name:ident,$base:ty $(,$projection:ident)?)=>{
        #[repr(C)] pub struct $name {pub base:ManuallyDrop<$base>,pub deferred:ManuallyDrop<DeferredResource>}
        impl $name {
            pub fn fromBase(base:$base,handle:u32,generation:u32,stream:Option<&SharedOreCommandBuffer>,allocator:Option<&SharedIdAllocator>)->Self {
                Self {base:ManuallyDrop::new(base),deferred:ManuallyDrop::new(DeferredResource::new(handle,generation,stream,allocator))}
            }
            pub fn clientHandle(&self)->u32 {self.deferred.clientHandle()}
            pub fn recordsInto(&self,stream:&SharedOreCommandBuffer)->bool {self.deferred.recordsInto(stream)}
        }
        impl Drop for $name {fn drop(&mut self) {unsafe {ManuallyDrop::drop(&mut self.deferred);ManuallyDrop::drop(&mut self.base);}}}
        // The repr(C) base is the first field and itself starts with GPUResource.
        unsafe impl GpuResourcePayload for $name {
            fn gpu_resource(&self)->&GPUResource {self.base.gpu_resource()}
            fn gpu_resource_mut(&mut self)->&mut GPUResource {self.base.gpu_resource_mut()}
            $(fn $projection(&self)->Option<&$base> {Some(&self.base)})?
        }
    };
}
deferred_owner!(DeferredBuffer, Buffer);
deferred_owner!(DeferredTexture, Texture);
deferred_owner!(DeferredTextureView, TextureView, texture_view_base);
deferred_owner!(DeferredSampler, Sampler);
deferred_owner!(
    DeferredBindGroupLayout,
    BindGroupLayout,
    bind_group_layout_base
);
deferred_owner!(DeferredPipeline, Pipeline, pipeline_base);
deferred_owner!(DeferredBindGroup, BindGroup, bind_group_base);
#[repr(C)]
pub struct DeferredShaderModule {
    pub base: ManuallyDrop<ShaderModule>,
    pub deferred: ManuallyDrop<DeferredResource>,
}
impl DeferredShaderModule {
    pub fn new(
        handle: u32,
        generation: u32,
        stream: Option<&SharedOreCommandBuffer>,
        allocator: Option<&SharedIdAllocator>,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(ShaderModule::new()),
            deferred: ManuallyDrop::new(DeferredResource::new(
                handle, generation, stream, allocator,
            )),
        }
    }
    pub fn clientHandle(&self) -> u32 {
        self.deferred.clientHandle()
    }
    pub fn recordsInto(&self, s: &SharedOreCommandBuffer) -> bool {
        self.deferred.recordsInto(s)
    }
}
impl Drop for DeferredShaderModule {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.deferred);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}
unsafe impl GpuResourcePayload for DeferredShaderModule {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
    fn shader_module_base(&self) -> Option<&ShaderModule> {
        Some(&self.base)
    }
}
impl DeferredBuffer {
    pub fn new(
        h: u32,
        g: u32,
        s: Option<&SharedOreCommandBuffer>,
        a: Option<&SharedIdAllocator>,
        size: u32,
        usage: BufferUsage,
    ) -> Self {
        Self::fromBase(Buffer::new(size, usage), h, g, s, a)
    }
}
impl BufferApi for DeferredBuffer {
    fn size(&self) -> u32 {
        self.base.size()
    }
    fn usage(&self) -> BufferUsage {
        self.base.usage()
    }
    fn update(&self, data: &[u8], size: u32, offset: u32) -> Result<(), BufferUpdateError> {
        self.deferred
            .withStream(|s| recordBufferUpdate(s, self.clientHandle(), Some(data), size, offset));
        Ok(())
    }
}
impl DeferredTexture {
    pub fn new(
        h: u32,
        g: u32,
        s: Option<&SharedOreCommandBuffer>,
        a: Option<&SharedIdAllocator>,
        desc: &TextureDesc<'_>,
    ) -> Self {
        Self::fromBase(Texture::new(desc), h, g, s, a)
    }
}
impl TextureApi for DeferredTexture {
    fn width(&self) -> u32 {
        self.base.width()
    }
    fn height(&self) -> u32 {
        self.base.height()
    }
    fn depthOrArrayLayers(&self) -> u32 {
        self.base.depthOrArrayLayers()
    }
    fn format(&self) -> TextureFormat {
        self.base.format()
    }
    fn r#type(&self) -> TextureType {
        self.base.r#type()
    }
    fn numMipmaps(&self) -> u32 {
        self.base.numMipmaps()
    }
    fn sampleCount(&self) -> u32 {
        self.base.sampleCount()
    }
    fn isRenderTarget(&self) -> bool {
        self.base.isRenderTarget()
    }
    fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        self.deferred
            .withStream(|s| recordTextureUpload(s, self.clientHandle(), data));
        Ok(())
    }
}
impl DeferredTextureView {
    pub fn new(
        h: u32,
        g: u32,
        s: Option<&SharedOreCommandBuffer>,
        a: Option<&SharedIdAllocator>,
        texture: Option<AnyResourceHandle>,
        desc: &TextureViewDesc<'_>,
    ) -> Self {
        Self::fromBase(TextureView::new_nullable(texture, desc), h, g, s, a)
    }
}
macro_rules! default_base {
    ($name:ident,$base:ty) => {
        impl $name {
            pub fn new(
                h: u32,
                g: u32,
                s: Option<&SharedOreCommandBuffer>,
                a: Option<&SharedIdAllocator>,
            ) -> Self {
                Self::fromBase(<$base>::new(), h, g, s, a)
            }
        }
    };
}
default_base!(DeferredSampler, Sampler);
default_base!(DeferredBindGroupLayout, BindGroupLayout);
default_base!(DeferredBindGroup, BindGroup);
impl DeferredPipeline {
    pub fn new(
        h: u32,
        g: u32,
        s: Option<&SharedOreCommandBuffer>,
        a: Option<&SharedIdAllocator>,
        desc: &PipelineDesc<'_>,
    ) -> Option<Self> {
        Some(Self::fromBase(Pipeline::new(desc)?, h, g, s, a))
    }
}

pub fn deferredResource(resource: &AnyResourceHandle) -> Option<&DeferredResource> {
    macro_rules! downcast {($($t:ty),*)=>{$(if let Some(d)=resource.downcast_ref::<$t>() {return Some(&d.deferred);})*};}
    downcast!(
        DeferredBuffer,
        DeferredTexture,
        DeferredTextureView,
        DeferredSampler,
        DeferredShaderModule,
        DeferredBindGroupLayout,
        DeferredPipeline,
        DeferredBindGroup
    );
    None
}
