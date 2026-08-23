//! Caller-supplied Metal drawable validation, rendering, and presentation.
//!
//! This adapts the external target-texture seam in
//! `renderer/src/metal/render_context_metal_impl.mm:735-781` and the product
//! ordering oracle in
//! `renderer/path_fiddle/fiddle_context_metal.mm:65-84,100-120,186-191`, all
//! pinned at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The platform caller
//! retains ownership of acquisition, actor scheduling, and layer policy.

use super::{mechanical_render_context::MechanicalRenderContext, NativeMetalFrame};
use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLDrawable, MTLTexture};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// One renderer frame borrowing the platform caller's presentation owner.
pub struct NativeMetalDrawableFrame<'a> {
    frame: NativeMetalFrame,
    drawable: &'a ProtocolObject<dyn MTLDrawable>,
    mechanical: Rc<RefCell<MechanicalRenderContext>>,
    restore_texture: Retained<ProtocolObject<dyn MTLTexture>>,
    restore_width: u32,
    restore_height: u32,
}

impl<'a> NativeMetalDrawableFrame<'a> {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(super) fn new(
        mechanical: Rc<RefCell<MechanicalRenderContext>>,
        drawable: &'a ProtocolObject<dyn MTLDrawable>,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        restore_texture: Retained<ProtocolObject<dyn MTLTexture>>,
        restore_width: u32,
        restore_height: u32,
        clear_color: u32,
    ) -> Result<Self, RendererError> {
        let (renderer, frame_number, resource_domain) = {
            let mut mechanical_context = mechanical.borrow_mut();
            let width = u32::try_from(texture.width())
                .map_err(|_| RendererError::NativeMetal("drawable width exceeds UInt32".into()))?;
            let height = u32::try_from(texture.height())
                .map_err(|_| RendererError::NativeMetal("drawable height exceeds UInt32".into()))?;
            mechanical_context.replace_target(texture, width, height)?;
            mechanical_context.begin_frame(clear_color)?;
            let context = unsafe {
                std::pin::Pin::get_unchecked_mut(mechanical_context.render_context_mut())
            };
            let renderer = unsafe {
                crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer::new_from_context(context)
            };
            (
                renderer,
                mechanical_context.current_frame_number(),
                mechanical_context.resource_domain(),
            )
        };
        let frame = NativeMetalFrame {
            mechanical: Rc::clone(&mechanical),
            renderer,
            resource_domain,
            collect_work_metrics: false,
            frame_number,
        };
        Ok(Self {
            frame,
            drawable,
            mechanical,
            restore_texture,
            restore_width,
            restore_height,
        })
    }

    /// Commits renderer work, then presents the borrowed drawable on the next
    /// command buffer from the same queue, matching the pinned product oracle.
    pub fn finish(mut self) -> Result<(), RendererError> {
        self.frame.finish_present(self.drawable)?;
        self.mechanical.borrow_mut().replace_target(
            self.restore_texture.clone(),
            self.restore_width,
            self.restore_height,
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

impl Drop for NativeMetalDrawableFrame<'_> {
    fn drop(&mut self) {
        if self.mechanical.borrow().is_active_frame() {
            self.mechanical.borrow_mut().abandon_frame();
        }
        if !self.mechanical.borrow().target_matches(
            &self.restore_texture,
            self.restore_width,
            self.restore_height,
        ) {
            let _ = self.mechanical.borrow_mut().replace_target(
                self.restore_texture.clone(),
                self.restore_width,
                self.restore_height,
            );
        }
    }
}
