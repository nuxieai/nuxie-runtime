//! Renderer resource tests translated from pls_render_context_test.cpp at 2210ed87.

use crate::mechanical_port::source::include::rive::{
    gpu_texture_format_hpp::GPUTextureFormat,
    refcnt_hpp::{make_rcp, rcp},
    renderer_hpp::{
        RenderBuffer, RenderBufferContract, RenderBufferFlags, RenderBufferType, RendererContract,
    },
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{LiteRttiTypeId, CONST_ID};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::ResourceAllocationCounts;
use crate::mechanical_port::source::renderer::include::rive::renderer::{
    buffer_ring_hpp::{BufferRingContract, HeapBufferRing},
    gpu_hpp::{FlushDescriptor, StorageBufferStructure},
    render_context_hpp::{FlushResources, FrameDescriptor, RenderContext},
    render_context_impl_hpp::{RenderContextImpl, RenderContextImplContract},
    render_target_hpp::RenderTarget,
    rive_renderer_hpp::RiveRenderer,
    texture_hpp::Texture,
};
use std::{cell::Cell, ffi::c_void, pin::Pin, rc::Rc};

#[test]
fn resource_allocation_counts() {
    let mut allocs = ResourceAllocationCounts {
        pathBufferCount: 1,
        contourBufferCount: 2,
        gradSpanBufferCount: 4,
        tessSpanBufferCount: 5,
        triangleVertexBufferCount: 6,
        imageRectInstanceBufferCount: 7,
        imageMeshInstanceBufferCount: 8,
        gradTextureHeight: 9,
        tessTextureHeight: 10,
        ..Default::default()
    };
    allocs = ResourceAllocationCounts::FromVec(&allocs.toVec().map(|v| v * 2));
    assert_eq!(allocs.pathBufferCount, 2);
    assert_eq!(allocs.contourBufferCount, 4);
    assert_eq!(allocs.gradSpanBufferCount, 8);
    assert_eq!(allocs.tessSpanBufferCount, 10);
    assert_eq!(allocs.triangleVertexBufferCount, 12);
    assert_eq!(allocs.imageRectInstanceBufferCount, 14);
    assert_eq!(allocs.imageMeshInstanceBufferCount, 16);
    assert_eq!(allocs.gradTextureHeight, 18);
    assert_eq!(allocs.tessTextureHeight, 20);

    let test_allocs = ResourceAllocationCounts {
        pathBufferCount: 18,
        contourBufferCount: 16,
        gradSpanBufferCount: 14,
        tessSpanBufferCount: 12,
        triangleVertexBufferCount: 10,
        imageRectInstanceBufferCount: 8,
        imageMeshInstanceBufferCount: 6,
        gradTextureHeight: 4,
        tessTextureHeight: 2,
        ..Default::default()
    };
    // Each lane is the same simd::if_then_else selection used upstream.
    let a = allocs.toVec();
    let b = test_allocs.toVec();
    allocs = ResourceAllocationCounts::FromVec(&std::array::from_fn(|i| {
        if a[i] <= b[i] {
            b[i]
        } else {
            a[i] * 5
        }
    }));
    assert_eq!(allocs.pathBufferCount, 18);
    assert_eq!(allocs.contourBufferCount, 16);
    assert_eq!(allocs.gradSpanBufferCount, 14);
    assert_eq!(allocs.tessSpanBufferCount, 12);
    assert_eq!(allocs.triangleVertexBufferCount, 12 * 5);
    assert_eq!(allocs.imageRectInstanceBufferCount, 14 * 5);
    assert_eq!(allocs.imageMeshInstanceBufferCount, 16 * 5);
    assert_eq!(allocs.gradTextureHeight, 18 * 5);
    assert_eq!(allocs.tessTextureHeight, 20 * 5);

    let a = allocs.toVec();
    allocs = ResourceAllocationCounts::FromVec(&std::array::from_fn(|i| {
        if b[i] * 2 <= a[i] {
            a[i] / 2
        } else {
            a[i]
        }
    }));
    assert_eq!(allocs.pathBufferCount, 18);
    assert_eq!(allocs.contourBufferCount, 16);
    assert_eq!(allocs.gradSpanBufferCount, 14);
    assert_eq!(allocs.tessSpanBufferCount, 12);
    assert_eq!(allocs.triangleVertexBufferCount, 6 * 5);
    assert_eq!(allocs.imageRectInstanceBufferCount, 7 * 5);
    assert_eq!(allocs.imageMeshInstanceBufferCount, 8 * 5);
    assert_eq!(allocs.gradTextureHeight, 9 * 5);
    assert_eq!(allocs.tessTextureHeight, 10 * 5);
}

const MAP_COUNT: usize = 10;

// The existing heap ring is the null GPU's shadow buffer owner. GPU submission
// is a no-op, but allocation, map failure, and every draw/flush/unwind are real.
struct MapFailureContext {
    base: RenderContextImpl,
    buffers: [HeapBufferRing; MAP_COUNT],
    map_counts: [usize; MAP_COUNT],
    fail_index: usize,
    total: Rc<Cell<usize>>,
}

impl MapFailureContext {
    fn new(fail_index: usize, total: Rc<Cell<usize>>) -> Self {
        let mut base = RenderContextImpl::default();
        base.m_platformFeatures.supportsRasterOrderingMode = true;
        base.m_platformFeatures.supportsAtomicMode = true;
        base.m_platformFeatures.supportsClockwiseMode = true;
        base.m_platformFeatures.supportsClockwiseFixedFunctionMode = true;
        base.m_platformFeatures.supportsClockwiseAtomicMode = true;
        Self {
            base,
            buffers: std::array::from_fn(|_| HeapBufferRing::new(0)),
            map_counts: [0; MAP_COUNT],
            fail_index,
            total,
        }
    }
    fn resize(&mut self, index: usize, bytes: usize) {
        assert_eq!(self.map_counts[index], 0);
        self.buffers[index] = HeapBufferRing::new(bytes);
    }
    fn map(&mut self, index: usize, bytes: usize) -> *mut c_void {
        assert_eq!(self.map_counts[index], 0);
        if index == self.fail_index {
            return std::ptr::null_mut();
        }
        self.map_counts[index] = bytes;
        self.total.set(self.total.get() + bytes);
        self.buffers[index].mapBuffer(bytes)
    }
    fn unmap(&mut self, index: usize, bytes: usize) {
        if self.map_counts[index] != 0 {
            assert_eq!(self.map_counts[index], bytes);
            self.buffers[index].unmapAndSubmitBuffer();
            self.map_counts[index] = 0;
            self.total.set(self.total.get() - bytes);
        }
    }
}

#[repr(C)]
struct DataRenderBuffer {
    base: RenderBuffer,
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

macro_rules! map_unmap {
    ($index:expr, $map:ident, $unmap:ident) => {
        fn $map(&mut self, bytes: usize) -> *mut c_void {
            const { assert!($index < MAP_COUNT) };
            self.map($index, bytes)
        }
        fn $unmap(&mut self, bytes: usize) {
            self.unmap($index, bytes);
        }
    };
}
macro_rules! resize_buffer {
    ($index:expr, $method:ident) => {
        fn $method(&mut self, bytes: usize) {
            self.resize($index, bytes);
        }
    };
    ($index:expr, $method:ident, storage) => {
        fn $method(&mut self, bytes: usize, _: StorageBufferStructure) {
            self.resize($index, bytes);
        }
    };
}

impl RenderContextImplContract for MapFailureContext {
    fn renderContextImpl(&self) -> &RenderContextImpl {
        &self.base
    }
    fn renderContextImplMut(&mut self) -> &mut RenderContextImpl {
        &mut self.base
    }

    fn makeRenderBuffer(
        &mut self,
        ty: RenderBufferType,
        flags: RenderBufferFlags,
        size: usize,
    ) -> rcp<RenderBuffer> {
        let buffer = Box::new(DataRenderBuffer {
            base: unsafe { RenderBuffer::new_for_owner::<DataRenderBuffer>(ty, flags, size) },
            bytes: vec![0; size],
        });
        unsafe { rcp::from_ptr(Box::into_raw(buffer).cast::<RenderBuffer>()) }
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
        feature = "native-webgpu-experimental",
        feature = "ore-gl"
    ))]
    fn makeOreContext(&mut self) -> Option<Box<crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::OreContext>>{
        None
    }

    resize_buffer!(0, resizeFlushUniformBuffer);
    resize_buffer!(1, resizePathBuffer, storage);
    resize_buffer!(2, resizePaintBuffer, storage);
    resize_buffer!(3, resizePaintAuxBuffer, storage);
    resize_buffer!(4, resizeContourBuffer, storage);
    resize_buffer!(5, resizeGradSpanBuffer);
    resize_buffer!(6, resizeTessVertexSpanBuffer);
    resize_buffer!(7, resizeTriangleVertexBuffer);
    resize_buffer!(8, resizeImageRectInstanceBuffer);
    resize_buffer!(9, resizeImageMeshInstanceBuffer);
    map_unmap!(0, mapFlushUniformBuffer, unmapFlushUniformBuffer);
    map_unmap!(1, mapPathBuffer, unmapPathBuffer);
    map_unmap!(2, mapPaintBuffer, unmapPaintBuffer);
    map_unmap!(3, mapPaintAuxBuffer, unmapPaintAuxBuffer);
    map_unmap!(4, mapContourBuffer, unmapContourBuffer);
    map_unmap!(5, mapGradSpanBuffer, unmapGradSpanBuffer);
    map_unmap!(6, mapTessVertexSpanBuffer, unmapTessVertexSpanBuffer);
    map_unmap!(7, mapTriangleVertexBuffer, unmapTriangleVertexBuffer);
    map_unmap!(8, mapImageRectInstanceBuffer, unmapImageRectInstanceBuffer);
    map_unmap!(9, mapImageMeshInstanceBuffer, unmapImageMeshInstanceBuffer);
    fn resizeGradientTexture(&mut self, _: u32, _: u32) {}
    fn resizeTessellationTexture(&mut self, _: u32, _: u32) {}
    fn resizeFeatherAtlasTexture(&mut self, _: u32, _: u32) {}
    fn resizeCoverageBuffer(&mut self, _: usize) {}
    unsafe fn flush(&mut self, _: &FlushDescriptor) {}
    fn secondsNow(&self) -> f64 {
        7.0
    }
}

#[test]
fn map_failure_unwind() {
    for fail_index in 0..MAP_COUNT {
        let total = Rc::new(Cell::new(0));
        let mut context =
            RenderContext::from_impl(Box::new(MapFailureContext::new(fail_index, total.clone())));
        // The context remains pinned and outlives the renderer and all draws.
        let context = unsafe { Pin::get_unchecked_mut(context.as_mut()) };
        let mut renderer = unsafe { RiveRenderer::new(context) };
        let path = context.makeEmptyRenderPath();
        unsafe {
            (&mut *path.get()).base.addRect(0.0, 0.0, 100.0, 100.0);
            (&mut *path.get()).base.addRect(20.0, 20.0, 80.0, 80.0);
        }
        let paint = context.makeRenderPaint();
        let target = make_rcp(|| RenderTarget::new(200, 200));
        context.beginFrameExecutable(&FrameDescriptor {
            renderTargetWidth: 200,
            renderTargetHeight: 200,
            ..Default::default()
        });
        unsafe {
            renderer.drawPath(path.get(), paint.get());
            context.flushExecutable(&FlushResources {
                renderTarget: target.get(),
                ..Default::default()
            });
        }
        assert_eq!(total.get(), 0, "failing map index {fail_index}");
    }
}
