//! Caller-supplied Metal drawable validation, rendering, and presentation.
//!
//! This adapts the external target-texture seam in
//! `renderer/src/metal/render_context_metal_impl.mm:735-781` and the product
//! ordering oracle in
//! `renderer/path_fiddle/fiddle_context_metal.mm:65-84,100-120,186-191`, all
//! pinned at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The platform caller
//! retains ownership of acquisition, actor scheduling, and layer policy.

use super::{NativeMetalContext, NativeMetalFrame, NativeMetalRenderState, RenderTargetMetal};
use crate::{RenderMode, RendererError};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLDevice, MTLDrawable, MTLPixelFormat, MTLResource, MTLTexture};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

/// One renderer frame borrowing the platform caller's presentation owner.
pub struct NativeMetalDrawableFrame<'a> {
    frame: NativeMetalFrame,
    drawable: &'a ProtocolObject<dyn MTLDrawable>,
}

impl<'a> NativeMetalDrawableFrame<'a> {
    pub(crate) fn new(
        context: Arc<NativeMetalContext>,
        mode: RenderMode,
        drawable: &'a ProtocolObject<dyn MTLDrawable>,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        expected_width: u32,
        expected_height: u32,
        clear_color: u32,
    ) -> Result<Self, RendererError> {
        let (width, height) =
            validate_drawable_texture(&texture, context.device(), expected_width, expected_height)?;
        let mut target = RenderTargetMetal::new(
            context.retained_device(),
            MTLPixelFormat::BGRA8Unorm,
            width,
            height,
            context.capabilities(),
        )?;
        target.set_target_texture(Some(texture))?;
        let command_buffer = context.make_command_buffer()?;
        let frame = NativeMetalFrame {
            context,
            target: Rc::new(RefCell::new(target)),
            mode,
            command_buffer,
            clear_color,
            state: NativeMetalRenderState::default(),
            state_stack: Vec::new(),
            atomic_logical_state: super::LogicalDrawState::default(),
            solid_draws: Vec::new(),
            atomic_path_inputs: Vec::new(),
            gradient_draws: Vec::new(),
            atlas_requests: Vec::new(),
            resource_lease: None,
            collect_work_metrics: false,
            backend_work: crate::BackendWorkMetrics::default(),
            atomic_draw_count: 0,
            atomic_draw_group_count: 0,
            atomic_barrier_count: 0,
            atomic_memory_barrier_count: 0,
            atomic_render_pass_break_count: 0,
            atomic_uses_clipping: false,
            atomic_uses_clip_rects: false,
            atomic_uses_advanced_blend: false,
            atomic_uses_hsl_blend_modes: false,
            unsupported: None,
        };
        Ok(Self { frame, drawable })
    }

    /// Commits renderer work, then presents the borrowed drawable on the next
    /// command buffer from the same queue, matching the pinned product oracle.
    pub fn finish(mut self) -> Result<(), RendererError> {
        self.frame.encode()?;
        let upload_completion = self.frame.transfer_upload_ownership()?;
        self.frame.context.commit_and_present(
            &self.frame.command_buffer,
            self.drawable,
            upload_completion,
        )
    }
}

impl Deref for NativeMetalDrawableFrame<'_> {
    type Target = NativeMetalFrame;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl DerefMut for NativeMetalDrawableFrame<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}

fn validate_drawable_texture(
    texture: &ProtocolObject<dyn MTLTexture>,
    context_device: &ProtocolObject<dyn MTLDevice>,
    expected_width: u32,
    expected_height: u32,
) -> Result<(u32, u32), RendererError> {
    if texture.pixelFormat() != MTLPixelFormat::BGRA8Unorm {
        return Err(RendererError::NativeMetal(
            "drawable texture is not BGRA8Unorm".to_owned(),
        ));
    }
    let texture_device = texture.device();
    if Retained::as_ptr(&texture_device) != std::ptr::from_ref(context_device) {
        return Err(RendererError::NativeMetal(
            "drawable texture belongs to a different MTLDevice".to_owned(),
        ));
    }
    let width = u32::try_from(texture.width())
        .map_err(|_| RendererError::NativeMetal("drawable width exceeds UInt32".to_owned()))?;
    let height = u32::try_from(texture.height())
        .map_err(|_| RendererError::NativeMetal("drawable height exceeds UInt32".to_owned()))?;
    if (width, height) != (expected_width, expected_height) {
        return Err(RendererError::NativeMetal(format!(
            "drawable texture is {width}x{height}, expected {expected_width}x{expected_height}"
        )));
    }
    Ok((width, height))
}
