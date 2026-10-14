//! tests/unit_tests/renderer/hairline_batch_test.cpp at 9166e9b6.
//!
//! Count actual flushed batches: correct pixels alone cannot detect a broken
//! ordering of the hairlines that borrow their fill's tessellation.

use crate::mechanical_port::source::include::rive::{
    gpu_texture_format_hpp::GPUTextureFormat,
    refcnt_hpp::{make_rcp, rcp},
    renderer_hpp::{
        RenderBuffer, RenderBufferContract, RenderBufferFlags, RenderBufferType, RendererContract,
    },
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{LiteRttiTypeId, CONST_ID};
use crate::mechanical_port::source::renderer::include::rive::renderer::{
    buffer_ring_hpp::{BufferRingContract, HeapBufferRing},
    gpu_hpp::{DrawType, FlushDescriptor, InterlockMode, StorageBufferStructure},
    render_context_helper_impl_hpp::{
        RenderContextHelperBackendContract, RenderContextHelperBufferFactoryContract,
        RenderContextHelperImpl, RenderContextHelperImplAccess,
    },
    render_context_hpp::{FlushResources, FrameDescriptor, RenderContext},
    render_context_impl_hpp::RenderContextImpl,
    render_target_hpp::RenderTarget,
    rive_renderer_hpp::RiveRenderer,
    texture_hpp::Texture,
};
use crate::mechanical_port::source::renderer::src::rive_render_paint_hpp::RiveRenderPaint;
use std::{cell::RefCell, collections::BTreeMap, ffi::c_void, pin::Pin, rc::Rc};

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

type BatchCounts = Rc<RefCell<BTreeMap<u8, usize>>>;

// The NULL device uses real heap buffer rings, while capturing the real flush
// draw list instead of submitting it to a GPU.
struct CapturingRenderContext {
    base: RenderContextHelperImpl,
    counts: BatchCounts,
}
impl CapturingRenderContext {
    fn new(supports_pipeline_dynamic_state: bool, counts: BatchCounts) -> Self {
        let mut base = RenderContextImpl::default();
        base.m_platformFeatures.supportsRasterOrderingMode = true;
        base.m_platformFeatures.supportsAtomicMode = true;
        base.m_platformFeatures.supportsClockwiseMode = true;
        base.m_platformFeatures.supportsClockwiseFixedFunctionMode = true;
        base.m_platformFeatures.supportsClockwiseAtomicMode = true;
        base.m_platformFeatures.supportsPipelineDynamicState = supports_pipeline_dynamic_state;
        Self {
            base: RenderContextHelperImpl::new(base),
            counts,
        }
    }
}
impl RenderContextHelperImplAccess for CapturingRenderContext {
    fn renderContextHelperImpl(&self) -> &RenderContextHelperImpl {
        &self.base
    }
    fn renderContextHelperImplMut(&mut self) -> &mut RenderContextHelperImpl {
        &mut self.base
    }
}
impl RenderContextHelperBufferFactoryContract for CapturingRenderContext {
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
impl RenderContextHelperBackendContract for CapturingRenderContext {
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
    fn makeOreContext(
        &mut self,
    ) -> Option<Box<crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::OreContext>>{
        None
    }
    fn resizeGradientTexture(&mut self, _: u32, _: u32) {}
    fn resizeTessellationTexture(&mut self, _: u32, _: u32) {}
    fn resizeFeatherAtlasTexture(&mut self, _: u32, _: u32) {}
    fn resizeCoverageBuffer(&mut self, _: usize) {}
    unsafe fn flush(&mut self, desc: &FlushDescriptor) {
        let Some(draw_list) = desc.drawList else {
            return;
        };
        let mut counts = self.counts.borrow_mut();
        // SAFETY: the source flush owns the list for the duration of this call.
        for batch in unsafe { draw_list.as_ref() }.iter() {
            *counts.entry(batch.drawType as u8).or_default() += 1;
        }
    }
}

const TARGET_SIZE: u32 = 256;
const GRID_SIZE: usize = 6;
const CELL: f32 = 40.0;
const SQUARE: f32 = 30.0;
const PATH_COUNT: usize = GRID_SIZE * GRID_SIZE + (GRID_SIZE - 1) * (GRID_SIZE - 1);

fn make_rects() -> Vec<[f32; 4]> {
    let mut rects = Vec::new();
    for layer in 0..2 {
        let n = GRID_SIZE - layer;
        let offset = 4.0 + layer as f32 * CELL / 2.0;
        for y in 0..n {
            for x in 0..n {
                let l = offset + x as f32 * CELL;
                let t = offset + y as f32 * CELL;
                rects.push([l, t, l + SQUARE, t + SQUARE]);
            }
        }
    }
    assert_eq!(rects.len(), PATH_COUNT);
    rects
}

fn draw_rects(supports_pipeline_dynamic_state: bool) -> (Pin<Box<RenderContext>>, BatchCounts) {
    let counts = Rc::new(RefCell::new(BTreeMap::new()));
    let mut context = RenderContext::from_impl(Box::new(CapturingRenderContext::new(
        supports_pipeline_dynamic_state,
        counts.clone(),
    )));
    // The context stays pinned and outlives the renderer and every draw.
    {
        let ctx = unsafe { Pin::get_unchecked_mut(context.as_mut()) };
        ctx.beginFrameExecutable(&FrameDescriptor {
            renderTargetWidth: TARGET_SIZE,
            renderTargetHeight: TARGET_SIZE,
            msaaSampleCount: 1,
            clockwiseFillOverride: true,
            inkbleedOverride: true,
            ..Default::default()
        });
        assert_eq!(ctx.frameInterlockMode(), InterlockMode::depthStencil);
        let paint = ctx.makeRenderPaint();
        // RenderContext's factory creates a RiveRenderPaint with the base at
        // offset zero; use its concrete color implementation.
        unsafe { (&mut *paint.get().cast::<RiveRenderPaint>()).color(0xff336699) };
        let mut renderer = unsafe { RiveRenderer::new(ctx) };
        for r in make_rects() {
            let path = ctx.makeEmptyRenderPath();
            unsafe {
                let p = &mut (*path.get()).base;
                p.moveTo(r[0], r[1]);
                p.lineTo(r[2], r[1]);
                p.lineTo(r[2], r[3]);
                p.lineTo(r[0], r[3]);
                p.close();
                renderer.drawPath(path.get(), paint.get());
            }
        }
        let target = make_rcp(|| RenderTarget::new(TARGET_SIZE, TARGET_SIZE));
        unsafe {
            ctx.flushExecutable(&FlushResources {
                renderTarget: target.get(),
                ..Default::default()
            });
        }
    }
    (context, counts)
}

fn count(counts: &BatchCounts, draw_type: DrawType) -> usize {
    counts
        .borrow()
        .get(&(draw_type as u8))
        .copied()
        .unwrap_or(0)
}

#[test]
fn opaque_hairlines_batch_like_their_fills_dynamic_state() {
    let (_ctx, counts) = draw_rects(true);
    let fills = count(&counts, DrawType::stencilDynamicMidpointFans);
    let hairlines = count(&counts, DrawType::depthAAOuterHairline);
    assert!(fills > 1);
    assert!(fills < PATH_COUNT);
    assert!(hairlines < PATH_COUNT);
    assert_eq!(hairlines, fills);
}

#[test]
fn opaque_hairlines_batch_like_their_fills_no_dynamic_state() {
    let (_ctx, counts) = draw_rects(false);
    let borrowed_coverage = count(&counts, DrawType::stencilMidpointFanBorrowedCoverage);
    let fans = count(&counts, DrawType::stencilMidpointFans);
    let resets = count(&counts, DrawType::stencilMidpointFanReset);
    let hairlines = count(&counts, DrawType::depthAAOuterHairline);
    assert!(fans > 1);
    assert!(fans < PATH_COUNT);
    assert_eq!(borrowed_coverage, fans);
    assert_eq!(resets, fans);
    assert!(hairlines < PATH_COUNT);
    assert_eq!(hairlines, fans);
}
