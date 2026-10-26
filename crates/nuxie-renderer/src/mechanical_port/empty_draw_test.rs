//! Direct port of empty_draw_test.cpp at f5bd36c1.

use crate::mechanical_port::source::include::rive::{
    gpu_texture_format_hpp::GPUTextureFormat,
    refcnt_hpp::{make_rcp, rcp},
    renderer_hpp::{
        RenderBuffer, RenderBufferContract, RenderBufferFlags, RenderBufferType, RendererContract,
    },
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{CONST_ID, LiteRttiTypeId};
use crate::mechanical_port::source::renderer::include::rive::renderer::{
    buffer_ring_hpp::{BufferRingContract, HeapBufferRing},
    gpu_hpp::{BarrierFlags, DrawType, FlushDescriptor, InterlockMode, StorageBufferStructure},
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
            if batch.barriers.0 & BarrierFlags::dstBlend.0 != 0
                && batch.drawType != DrawType::renderPassInitialize
            {
                assert!(batch.dstReadList.is_some());
            }
        }
    }
}

const TARGET_SIZE: u32 = 256;

fn make_path(
    ctx: &mut RenderContext,
    rect: Option<[f32; 4]>,
) -> rcp<crate::mechanical_port::source::include::rive::renderer_hpp::RenderPath> {
    let path = ctx.makeEmptyRenderPath();
    unsafe {
        let p = &mut (*path.get()).base;
        if let Some([l, t, r, b]) = rect {
            p.moveTo(l, t);
            p.lineTo(r, t);
            p.lineTo(r, b);
            p.lineTo(l, b);
            p.close();
        } else {
            p.moveTo(20.0, 20.0);
            p.close();
            p.moveTo(200.0, 200.0);
            p.close();
        }
    }
    path
}

fn run_empty_case(case: u8) -> usize {
    let counts = Rc::new(RefCell::new(BTreeMap::new()));
    let mut context =
        RenderContext::from_impl(Box::new(CapturingRenderContext::new(false, counts.clone())));
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
    unsafe { (&mut *paint.get().cast::<RiveRenderPaint>()).color(0xff336699) };
    let mut renderer = unsafe { RiveRenderer::new(ctx) };
    let empty = make_path(ctx, None);
    unsafe {
        match case {
            0 => renderer.drawPath(empty.get(), paint.get()),
            1 => {
                renderer.save();
                renderer.clipPath(empty.get());
                let rect = make_path(ctx, Some([20.0, 20.0, 200.0, 200.0]));
                renderer.drawPath(rect.get(), paint.get());
                renderer.restore();
            }
            2 => {
                let multiply = ctx.makeRenderPaint();
                let p = &mut *multiply.get().cast::<RiveRenderPaint>();
                p.color(0xff996633);
                p.blendMode(nuxie_render_api::BlendMode::Multiply);
                let first = make_path(ctx, Some([20.0, 20.0, 120.0, 120.0]));
                renderer.drawPath(first.get(), paint.get());
                renderer.drawPath(empty.get(), multiply.get());
                let last = make_path(ctx, Some([140.0, 140.0, 240.0, 240.0]));
                renderer.drawPath(last.get(), paint.get());
            }
            _ => unreachable!(),
        }
        let target = make_rcp(|| RenderTarget::new(TARGET_SIZE, TARGET_SIZE));
        ctx.flushExecutable(&FlushResources {
            renderTarget: target.get(),
            ..Default::default()
        });
    }
    let total = counts
        .borrow()
        .iter()
        .filter(|(ty, _)| {
            **ty != DrawType::renderPassInitialize as u8
                && **ty != DrawType::renderPassResolve as u8
        })
        .map(|(_, n)| *n)
        .sum();
    total
}

#[test]
fn empty_paths_are_dropped_depth_stencil() {
    assert_eq!(run_empty_case(0), 0);
}
#[test]
fn empty_clip_clips_out_everything_depth_stencil() {
    assert_eq!(run_empty_case(1), 0);
}
#[test]
fn empty_advanced_blend_does_not_schedule_dst_blend_barrier() {
    assert!(run_empty_case(2) > 0);
}
