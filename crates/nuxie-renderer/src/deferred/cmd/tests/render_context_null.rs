//! tests/common/render_context_null.hpp/.cpp at e949498e, with the
//! deferred_flush_parity_test.cpp FlushObservingNULL derived observation.
#![allow(non_snake_case)]
use super::*;
use crate::exact_source_adapter::{ExactSourceBackend, ExactSourceFactoryCore};
use crate::mechanical_port::source::{
    include::{
        rive::{
            gpu_texture_format_hpp::GPUTextureFormat,
            refcnt_hpp::{make_rcp, rcp},
            renderer_hpp::{
                RenderBuffer as SourceBuffer, RenderBufferContract,
                RenderBufferFlags as SourceBufferFlags, RenderBufferType as SourceBufferType,
            },
        },
        utils::lite_rtti_hpp::{LiteRttiTypeId, CONST_ID},
    },
    renderer::include::rive::renderer::{
        buffer_ring_hpp::{BufferRingContract, HeapBufferRing},
        gpu_hpp::{FlushDescriptor, StorageBufferStructure},
        render_context_helper_impl_hpp::{
            RenderContextHelperBackendContract, RenderContextHelperBufferFactoryContract,
            RenderContextHelperImpl, RenderContextHelperImplAccess,
        },
        render_context_hpp::{FlushResources, FrameDescriptor, RenderContext},
        render_context_impl_hpp::RenderContextImpl,
        render_target_hpp::RenderTarget,
        texture_hpp::Texture,
    },
};
use std::{ffi::c_void, pin::Pin};
#[derive(Clone, Copy, Default)]
pub struct FlushStats {
    pub flushes: u64,
    pub path_count: u64,
    pub contour_count: u64,
    pub tess_vertex_spans: u64,
    pub grad_spans: u64,
    pub grad_data_height: u64,
    pub tess_data_height: u64,
    pub atlas_fill_batches: u64,
    pub atlas_stroke_batches: u64,
    pub atlas_content_area: u64,
}
impl std::ops::Sub for FlushStats {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self {
            flushes: self.flushes - b.flushes,
            path_count: self.path_count - b.path_count,
            contour_count: self.contour_count - b.contour_count,
            tess_vertex_spans: self.tess_vertex_spans - b.tess_vertex_spans,
            grad_spans: self.grad_spans - b.grad_spans,
            grad_data_height: self.grad_data_height - b.grad_data_height,
            tess_data_height: self.tess_data_height - b.tess_data_height,
            atlas_fill_batches: self.atlas_fill_batches - b.atlas_fill_batches,
            atlas_stroke_batches: self.atlas_stroke_batches - b.atlas_stroke_batches,
            atlas_content_area: self.atlas_content_area - b.atlas_content_area,
        }
    }
}
#[repr(C)]
struct DataRenderBuffer {
    base: SourceBuffer,
    bytes: Vec<u8>,
}
impl LiteRttiTypeId for DataRenderBuffer {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("DataRenderBuffer");
}
impl RenderBufferContract for DataRenderBuffer {
    fn onMap(&mut self) -> *mut c_void {
        self.bytes.as_mut_ptr().cast()
    }
    fn onUnmap(&mut self) {}
}
fn data_buffer(t: SourceBufferType, f: SourceBufferFlags, size: usize) -> rcp<SourceBuffer> {
    let buffer = Box::new(DataRenderBuffer {
        base: unsafe { SourceBuffer::new_for_owner::<DataRenderBuffer>(t, f, size) },
        bytes: vec![0; size],
    });
    unsafe { rcp::from_ptr(Box::into_raw(buffer).cast::<SourceBuffer>()) }
}
struct RenderContextNull {
    base: RenderContextHelperImpl,
    stats: Rc<RefCell<FlushStats>>,
    features: Rc<Cell<u32>>,
}
impl RenderContextNull {
    fn new(stats: Rc<RefCell<FlushStats>>, features: Rc<Cell<u32>>) -> Self {
        let mut base = RenderContextImpl::default();
        base.m_platformFeatures.supportsRasterOrderingMode = true;
        base.m_platformFeatures.supportsAtomicMode = true;
        base.m_platformFeatures.supportsClockwiseMode = true;
        base.m_platformFeatures.supportsClockwiseFixedFunctionMode = true;
        base.m_platformFeatures.supportsClockwiseAtomicMode = true;
        Self {
            base: RenderContextHelperImpl::new(base),
            stats,
            features,
        }
    }
}
impl RenderContextHelperImplAccess for RenderContextNull {
    fn renderContextHelperImpl(&self) -> &RenderContextHelperImpl {
        &self.base
    }
    fn renderContextHelperImplMut(&mut self) -> &mut RenderContextHelperImpl {
        &mut self.base
    }
}
impl RenderContextHelperBufferFactoryContract for RenderContextNull {
    fn makeUniformBufferRing(&mut self, size: usize) -> Option<Box<dyn BufferRingContract>> {
        Some(Box::new(HeapBufferRing::new(size)))
    }
    fn makeStorageBufferRing(
        &mut self,
        size: usize,
        _: StorageBufferStructure,
    ) -> Option<Box<dyn BufferRingContract>> {
        Some(Box::new(HeapBufferRing::new(size)))
    }
    fn makeVertexBufferRing(&mut self, size: usize) -> Option<Box<dyn BufferRingContract>> {
        Some(Box::new(HeapBufferRing::new(size)))
    }
}
impl RenderContextHelperBackendContract for RenderContextNull {
    fn makeRenderBuffer(
        &mut self,
        t: SourceBufferType,
        f: SourceBufferFlags,
        size: usize,
    ) -> rcp<SourceBuffer> {
        data_buffer(t, f, size)
    }
    fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        _: u32,
        _: GPUTextureFormat,
        _: &[u8],
        _: u8,
        _: u8,
        _: bool,
        _: bool,
    ) -> rcp<Texture> {
        make_rcp(|| Texture::new(width, height))
    }
    #[cfg(any(
        feature = "native-ore-metal-experimental",
        feature = "native-ore-vulkan-experimental",
        feature = "ore-gl"
    ))]
    fn makeOreContext(&mut self)->Option<Box<crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::OreContext>>{
        None
    }
    fn resizeGradientTexture(&mut self, _: u32, _: u32) {}
    fn resizeTessellationTexture(&mut self, _: u32, _: u32) {}
    fn resizeFeatherAtlasTexture(&mut self, _: u32, _: u32) {}
    fn resizeCoverageBuffer(&mut self, _: usize) {}
    unsafe fn flush(&mut self, d: &FlushDescriptor) {
        self.features
            .set(self.features.get() | d.combinedShaderFeatures.0);
        let mut s = self.stats.borrow_mut();
        s.flushes += 1;
        s.path_count += u64::from(d.pathCount);
        s.contour_count += u64::from(d.contourCount);
        s.tess_vertex_spans += u64::from(d.tessVertexSpanCount);
        s.grad_spans += u64::from(d.gradSpanCount);
        s.grad_data_height += u64::from(d.gradDataHeight);
        s.tess_data_height += u64::from(d.tessDataHeight);
        s.atlas_fill_batches += d.featherAtlasFillBatchCount as u64;
        s.atlas_stroke_batches += d.featherAtlasStrokeBatchCount as u64;
        s.atlas_content_area +=
            u64::from(d.featherAtlasContentWidth) * u64::from(d.featherAtlasContentHeight);
    }
}
pub struct NullBackend {
    context: Pin<Box<RenderContext>>,
    target: rcp<RenderTarget>,
    width: u32,
    height: u32,
}
impl NullBackend {
    pub fn new(
        width: u32,
        height: u32,
        stats: Rc<RefCell<FlushStats>>,
        features: Rc<Cell<u32>>,
    ) -> Self {
        let mut context =
            RenderContext::from_impl(Box::new(RenderContextNull::new(stats, features)));
        crate::exact_source_adapter::install_bitmap_decoder(context.as_mut());
        Self {
            context,
            target: make_rcp(|| RenderTarget::new(width, height)),
            width,
            height,
        }
    }
    pub fn flush(&mut self) {
        unsafe {
            self.context
                .as_mut()
                .get_unchecked_mut()
                .flushExecutable(&FlushResources {
                    renderTarget: self.target.get(),
                    ..Default::default()
                });
        }
    }
}
impl ExactSourceBackend for NullBackend {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), crate::RendererError> {
        self.width = width;
        self.height = height;
        self.target = make_rcp(|| RenderTarget::new(width, height));
        Ok(())
    }
    fn context_mut(&mut self) -> Pin<&mut RenderContext> {
        self.context.as_mut()
    }
    fn begin_frame(&mut self, _: u32, _: crate::RenderMode) -> Result<u64, crate::RendererError> {
        unsafe {
            self.context
                .as_mut()
                .get_unchecked_mut()
                .beginFrameExecutable(&FrameDescriptor {
                    renderTargetWidth: self.width,
                    renderTargetHeight: self.height,
                    ..Default::default()
                });
        }
        Ok(0)
    }
    fn finish_frame(&mut self, _: u64) -> Result<Vec<u8>, crate::RendererError> {
        self.flush();
        Ok(Vec::new())
    }
    fn abort_frame(&mut self) {}
}
pub type ObservingFactory = PersistentFactory<ExactSourceFactoryCore<NullBackend>>;
pub fn observing_factory(
    width: u32,
    height: u32,
) -> (ObservingFactory, Rc<RefCell<FlushStats>>, Rc<Cell<u32>>) {
    let stats = Rc::new(RefCell::new(FlushStats::default()));
    let features = Rc::new(Cell::new(0));
    let f = PersistentFactory::new(ExactSourceFactoryCore::new(NullBackend::new(
        width,
        height,
        stats.clone(),
        features.clone(),
    )));
    (f, stats, features)
}
