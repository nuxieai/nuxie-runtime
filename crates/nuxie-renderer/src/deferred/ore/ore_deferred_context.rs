//! renderer/ore/cmd/ore_deferred_context.hpp at 707c4f60.
#![allow(non_snake_case)]
use crate::deferred::cmd::{
    foreign_image_registry::ForeignImageRegistry, render_handle::CANVAS_HANDLE_MASK,
};
use nuxie_ore_metal::cmd::{
    id_allocator::{Allocation, IdAllocator},
    live_recorder_registry::{register_recorder, unregister_recorder},
};
use nuxie_ore_metal::ore_cmd::{
    ore_command_buffer::{OreCommandBuffer, SharedIdAllocator, SharedOreCommandBuffer},
    ore_commands::WrapCanvasViewMode,
    ore_deferred_resource::*,
    ore_handle::{INVALID_HANDLE, REAL_RESOURCE_FLAG, REAL_RESOURCE_MASK},
    ore_make_recording::*,
    ore_make_replay::OreResident,
    ore_render_pass_recording::RenderPassRecording,
    ore_replay::replayOreStream,
};
use nuxie_ore_metal::{
    context::{
        ActiveRenderPass, CanvasImageInfo, CanvasTextureInfo, Context, ContextApi, FrameDescriptor,
        ShaderTarget,
    },
    gpu_resource::{AnyResourceHandle, ResourceHandle},
    render_pass::RenderPassApi,
    types::*,
};
use nuxie_render_api::{OreContextHandle, RenderCanvasHandle, RenderImage};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    ffi::c_void,
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct RealResources {
    ids: HashMap<usize, u32>,
    objects: Vec<AnyResourceHandle>,
}
impl RealResources {
    fn handleFor(&mut self, r: &AnyResourceHandle) -> u32 {
        let key = r.allocation_identity();
        if let Some(h) = self.ids.get(&key) {
            return *h;
        }
        assert!(self.objects.len() <= REAL_RESOURCE_MASK as usize);
        let h = REAL_RESOURCE_FLAG | self.objects.len() as u32;
        self.objects.push(r.clone());
        self.ids.insert(key, h);
        h
    }
}
pub struct DeferredOreContext {
    base: Context,
    real: Option<Weak<RefCell<dyn ContextApi>>>,
    render: SharedOreCommandBuffer,
    ids: SharedIdAllocator,
    realResources: Rc<RefCell<RealResources>>,
    canvasIdProvider: Option<Box<dyn FnMut(RenderCanvasHandle) -> u32>>,
    canvasRegistry: Option<Rc<RefCell<ForeignImageRegistry>>>,
}
pub struct StreamBytes {
    pub commands: usize,
    pub blobs: usize,
}
impl StreamBytes {
    pub fn total(&self) -> usize {
        self.commands + self.blobs
    }
}
impl DeferredOreContext {
    #[cfg(test)]
    pub(super) fn assertExclusiveTeardownFixture(&self) {
        assert!(
            self.real.is_none() && self.canvasIdProvider.is_none() && self.canvasRegistry.is_none()
        );
        assert!(self.base.activeRenderPass().is_none());
        assert_eq!(Rc::strong_count(&self.render), 1);
        assert_eq!(Rc::weak_count(&self.render), 0);
        assert_eq!(Rc::strong_count(&self.realResources), 2);
        assert_eq!(Rc::weak_count(&self.realResources), 0);
        assert!(self.realResources.borrow().objects.is_empty());
        assert!(self.render.borrow().keepAlive().is_empty());
        let pending = self.base.pendingFrame();
        assert_eq!(Rc::strong_count(&pending), 2);
        assert_eq!(Rc::weak_count(&pending), 0);
    }
    pub fn new(real: Option<OreContextHandle>) -> Self {
        let realResources = Rc::new(RefCell::new(RealResources::default()));
        let resources = realResources.clone();
        let mut render = OreCommandBuffer::default();
        render.realHandleProvider = Some(Box::new(move |r| resources.borrow_mut().handleFor(r)));
        render.bindRecordingThread();
        let render = Rc::new(RefCell::new(render));
        let ids = Arc::new(Mutex::new(IdAllocator::default()));
        register_recorder(render.borrow().recorderIdentity());
        register_recorder(Arc::as_ptr(&ids) as usize);
        let out = Self {
            base: nuxie_ore_metal::new_context_backend_base(Features::default(), None),
            real: real.as_ref().map(Rc::downgrade),
            render,
            ids,
            realResources,
            canvasIdProvider: None,
            canvasRegistry: None,
        };
        out.adoptRealFeatures();
        out
    }
    fn real(&self) -> Option<OreContextHandle> {
        self.real.as_ref().and_then(Weak::upgrade)
    }
    fn alloc(&self) -> Allocation {
        self.ids
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .alloc()
    }
    fn adoptRealFeatures(&self) {
        if let Some(real) = self.real() {
            nuxie_ore_metal::context_backend_set_features(&self.base, real.borrow().features());
        }
    }
    pub fn bindReal(&mut self, real: Option<OreContextHandle>) {
        let late = self.real().is_none() && real.is_some();
        self.real = real.as_ref().map(Rc::downgrade);
        self.adoptRealFeatures();
        if late {
            let real = real.unwrap();
            let real = real.borrow();
            if real.shaderTarget() != ShaderTarget::glsl {
                eprintln!(
                    "rive deferred: TRIPWIRE late bound backend consumes shader target {}, but recording already loaded {}",
                    real.shaderTarget() as u8,
                    ShaderTarget::glsl as u8
                );
                debug_assert!(false, "late bind changed the recorded shader target");
            }
            if real.canvasTargetFormat() != TextureFormat::rgba8unorm {
                eprintln!(
                    "rive deferred: TRIPWIRE late bound backend allocates canvases as format {}, but recording already reserved canvas views as {}",
                    real.canvasTargetFormat() as u8,
                    TextureFormat::rgba8unorm as u8
                );
                debug_assert!(false, "late bind changed the recorded canvas format");
            }
        }
    }
    pub fn setCanvasIdProvider(
        &mut self,
        provider: Option<Box<dyn FnMut(RenderCanvasHandle) -> u32>>,
    ) {
        self.canvasIdProvider = provider;
    }
    pub fn setCanvasRegistry(&mut self, registry: Option<Rc<RefCell<ForeignImageRegistry>>>) {
        self.canvasRegistry = registry;
    }
    pub fn stream(&self) -> SharedOreCommandBuffer {
        self.render.clone()
    }
    pub fn streamBytes(&self) -> StreamBytes {
        let stream = self.render.borrow();
        StreamBytes {
            commands: stream.command_bytes().len(),
            blobs: stream.blob_bytes().len(),
        }
    }
    pub fn realResources(&self) -> Ref<'_, [AnyResourceHandle]> {
        Ref::map(self.realResources.borrow(), |r| r.objects.as_slice())
    }
    pub fn handleFor(&self, r: Option<&AnyResourceHandle>) -> u32 {
        let Some(r) = r else {
            return INVALID_HANDLE;
        };
        if let Some(d) = deferredResource(r) {
            if d.recordsInto(&self.render) {
                return d.clientHandle();
            }
        }
        self.realResources.borrow_mut().handleFor(r)
    }
    pub fn resetFrame(&mut self) {
        self.render.borrow_mut().reset();
        self.render.borrow_mut().drainDestroys();
        let mut real = self.realResources.borrow_mut();
        real.ids.clear();
        real.objects.clear();
    }
    pub fn replayFrame(
        &self,
        realCtx: &mut dyn ContextApi,
        table: &mut OreResident,
        canvasAt: &mut dyn FnMut(u32) -> Option<CanvasTextureInfo>,
    ) {
        let stream = self.render.borrow();
        replayOreStream(
            realCtx,
            stream.command_bytes(),
            stream.blob_bytes(),
            table,
            &mut |h| {
                self.realResources
                    .borrow()
                    .objects
                    .get((h & REAL_RESOURCE_MASK) as usize)
                    .cloned()
            },
            canvasAt,
            &mut |_| None,
        );
    }
    pub fn replay(&self, realCtx: &mut dyn ContextApi) {
        self.replayFrame(realCtx, &mut OreResident::default(), &mut |_| None);
    }
    pub fn makeReservedCanvasView(
        &self,
        id: u32,
        generation: u32,
        width: u32,
        height: u32,
    ) -> AnyResourceHandle {
        let desc = TextureDesc {
            width,
            height,
            format: self.real().map_or(TextureFormat::rgba8unorm, |r| {
                r.borrow().canvasTargetFormat()
            }),
            r#type: TextureType::texture2D,
            renderTarget: true,
            numMipmaps: 1,
            sampleCount: 1,
            ..Default::default()
        };
        let texture =
            ResourceHandle::new_texture(None, DeferredTexture::new(0, 0, None, None, &desc))
                .erase();
        let view = TextureViewDesc {
            texture: Some(&texture),
            dimension: TextureViewDimension::texture2D,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
            ..Default::default()
        };
        ResourceHandle::new(
            None,
            DeferredTextureView::new(
                id,
                generation,
                Some(&self.render),
                Some(&self.ids),
                Some(texture.clone()),
                &view,
            ),
        )
        .erase()
    }
}
impl Drop for DeferredOreContext {
    fn drop(&mut self) {
        unregister_recorder(self.render.borrow().recorderIdentity());
        unregister_recorder(Arc::as_ptr(&self.ids) as usize);
        self.render.borrow_mut().drainDestroys();
    }
}
impl ContextApi for DeferredOreContext {
    fn contextBase(&self) -> &Context {
        &self.base
    }
    fn isRecording(&self) -> bool {
        true
    }
    fn featuresKnown(&self) -> bool {
        self.real().is_some()
    }
    fn features(&self) -> Features {
        self.base.features()
    }
    fn lastError(&self) -> String {
        self.base.lastError()
    }
    fn activeRenderPass(&self) -> Option<Weak<dyn ActiveRenderPass>> {
        self.base.activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.base.setActiveRenderPass(pass);
    }
    fn finishActiveRenderPass(&self) {
        self.base.finishActiveRenderPass();
    }
    fn clearLastError(&self) {
        self.base.clearLastError();
    }
    fn setLastError(&self, message: &str) {
        self.base.setLastError(message);
    }
    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        let a = self.alloc();
        recordMakeBuffer(&mut self.render.borrow_mut(), a.id, a.generation, desc);
        Some(
            ResourceHandle::new_buffer(
                None,
                DeferredBuffer::new(
                    a.id,
                    a.generation,
                    Some(&self.render),
                    Some(&self.ids),
                    desc.size,
                    desc.usage,
                ),
            )
            .erase(),
        )
    }
    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        let a = self.alloc();
        recordMakeTexture(&mut self.render.borrow_mut(), a.id, a.generation, desc);
        Some(
            ResourceHandle::new_texture(
                None,
                DeferredTexture::new(
                    a.id,
                    a.generation,
                    Some(&self.render),
                    Some(&self.ids),
                    desc,
                ),
            )
            .erase(),
        )
    }
    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        let a = self.alloc();
        let texture = self.handleFor(desc.texture);
        recordMakeTextureView(
            &mut self.render.borrow_mut(),
            a.id,
            a.generation,
            desc,
            texture,
        );
        Some(
            ResourceHandle::new(
                None,
                DeferredTextureView::new(
                    a.id,
                    a.generation,
                    Some(&self.render),
                    Some(&self.ids),
                    desc.texture.cloned(),
                    desc,
                ),
            )
            .erase(),
        )
    }
    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        let a = self.alloc();
        recordMakeSampler(&mut self.render.borrow_mut(), a.id, a.generation, desc);
        Some(
            ResourceHandle::new(
                None,
                DeferredSampler::new(a.id, a.generation, Some(&self.render), Some(&self.ids)),
            )
            .erase(),
        )
    }
    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        let a = self.alloc();
        recordMakeShaderModule(&mut self.render.borrow_mut(), a.id, a.generation, desc);
        let mut obj =
            DeferredShaderModule::new(a.id, a.generation, Some(&self.render), Some(&self.ids));
        if desc.bindingMapBytes.is_some() && desc.bindingMapSize > 0 {
            obj.base.applyBindingMapFromDesc(desc);
        }
        Some(ResourceHandle::new(None, obj).erase())
    }
    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        let a = self.alloc();
        recordMakeBindGroupLayout(&mut self.render.borrow_mut(), a.id, a.generation, desc);
        Some(
            ResourceHandle::new(
                None,
                DeferredBindGroupLayout::new(
                    a.id,
                    a.generation,
                    Some(&self.render),
                    Some(&self.ids),
                    desc,
                ),
            )
            .erase(),
        )
    }
    fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        _outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        let bgls: Vec<_> = desc.bindGroupLayouts.unwrap_or(&[])
            [..desc.bindGroupLayoutCount as usize]
            .iter()
            .map(|r| self.handleFor(*r))
            .collect();
        let a = self.alloc();
        let vertex = self.handleFor(desc.vertexModule);
        let fragment = self.handleFor(desc.fragmentModule);
        recordMakePipeline(
            &mut self.render.borrow_mut(),
            a.id,
            a.generation,
            desc,
            vertex,
            fragment,
            &bgls,
        );
        Some(
            ResourceHandle::new(
                None,
                DeferredPipeline::new(
                    a.id,
                    a.generation,
                    Some(&self.render),
                    Some(&self.ids),
                    desc,
                )?,
            )
            .erase(),
        )
    }
    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        let ubos: Vec<_> = desc.ubos[..desc.uboCount as usize]
            .iter()
            .map(|e| self.handleFor(e.buffer))
            .collect();
        let texs: Vec<_> = desc.textures[..desc.textureCount as usize]
            .iter()
            .map(|e| self.handleFor(e.view))
            .collect();
        let samps: Vec<_> = desc.samplers[..desc.samplerCount as usize]
            .iter()
            .map(|e| self.handleFor(e.sampler))
            .collect();
        let a = self.alloc();
        let layout = self.handleFor(desc.layout);
        recordMakeBindGroup(
            &mut self.render.borrow_mut(),
            a.id,
            a.generation,
            desc,
            layout,
            &ubos,
            &texs,
            &samps,
        );
        Some(
            ResourceHandle::new(
                None,
                DeferredBindGroup::new(
                    a.id,
                    a.generation,
                    Some(&self.render),
                    Some(&self.ids),
                    desc,
                ),
            )
            .erase(),
        )
    }
    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        _outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        Some(Box::new(RenderPassRecording::new(
            Some(&self.base),
            self.render.clone(),
            desc,
        )))
    }
    fn beginFrame(&mut self, _: &FrameDescriptor) {}
    fn endFrame(&mut self) {}
    fn waitForGPU(&mut self) {}
    unsafe fn wrapCanvasTexture(&mut self, canvas: *mut c_void) -> Option<AnyResourceHandle> {
        assert!(
            self.canvasIdProvider.is_none(),
            "recording canvas requires its typed host owner"
        );
        let real = self
            .real()
            .expect("sessionless recorder needs real context");
        let texture = unsafe { real.borrow_mut().wrapCanvasTexture(canvas) };
        texture
    }
    unsafe fn wrapCanvasTextureInfo(
        &mut self,
        info: CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        let Some(provider) = &mut self.canvasIdProvider else {
            let real = self
                .real()
                .expect("sessionless recorder needs real context");
            return unsafe { real.borrow_mut().wrapCanvasTextureInfo(info) };
        };
        let canvas =
            nuxie_render_api::canvas_texture_owner(&info).expect("recording canvas host owner");
        let id = provider(canvas);
        let a = self.alloc();
        recordWrapCanvasView(
            &mut self.render.borrow_mut(),
            a.id,
            a.generation,
            id,
            WrapCanvasViewMode::colorView,
        );
        Some(self.makeReservedCanvasView(a.id, a.generation, info.width, info.height))
    }
    fn recordWrapCanvasImage(&mut self, info: CanvasImageInfo) -> Option<AnyResourceHandle> {
        let image = info
            .owner
            .downcast_ref::<Rc<dyn RenderImage>>()
            .expect("recording image host owner");
        let id = self
            .canvasRegistry
            .as_ref()
            .expect("canvas registry")
            .borrow_mut()
            .image_draw_id(image.as_ref())
            & CANVAS_HANDLE_MASK;
        let a = self.alloc();
        recordWrapCanvasView(
            &mut self.render.borrow_mut(),
            a.id,
            a.generation,
            id,
            WrapCanvasViewMode::sampleView,
        );
        Some(self.makeReservedCanvasView(a.id, a.generation, info.width, info.height))
    }
    fn recordWrapImageView(
        &mut self,
        imageId: u32,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        let a = self.alloc();
        recordWrapCanvasView(
            &mut self.render.borrow_mut(),
            a.id,
            a.generation,
            imageId,
            WrapCanvasViewMode::imageView,
        );
        Some(self.makeReservedCanvasView(a.id, a.generation, width, height))
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        eprintln!(
            "rive deferred: TRIPWIRE wrapRiveTexture hit immediately during recording (a script GPU op is not deferred)"
        );
        debug_assert!(false, "wrapRiveTexture must be deferred while recording");
        let real = self.real()?;
        let texture = unsafe { real.borrow_mut().wrapRiveTexture(texture, width, height) };
        texture
    }
    fn shaderTarget(&self) -> ShaderTarget {
        self.real()
            .map_or(ShaderTarget::glsl, |r| r.borrow().shaderTarget())
    }
}
