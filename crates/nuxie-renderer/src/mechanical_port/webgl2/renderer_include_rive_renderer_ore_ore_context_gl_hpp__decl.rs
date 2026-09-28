//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/ore/ore_context_gl.hpp`.

#![allow(non_snake_case)]

use super::gles3_decl::{GLExecutionDomain, GLExecutionStamp, GLint};
use nuxie_ore_metal::context::{Context, ShaderTarget};
use nuxie_ore_metal::types::Features;
use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_ore_ore_context_gl.hpp");

/// Exact source `ContextGL::GLSavedState` value record.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GLSavedState {
    pub(crate) program: GLint,
    pub(crate) arrayBuffer: GLint,
    pub(crate) uniformBuffer: GLint,
    pub(crate) framebuffer: GLint,
    pub(crate) vertexArray: GLint,
}

/// Source scratch fields shared with unfinished passes instead of retaining
/// C++'s raw ContextGL pointer. The last owner performs generation-safe deletion.
#[derive(Default)]
pub(super) struct ScratchPassState {
    pub(super) m_scratchFBO: u32,
    pub(super) m_scratchVAO: u32,
    pub(super) m_scratchResolveFBO: u32,
    pub(super) m_scratchFBOLent: bool,
    pub(super) m_scratchVAOLent: bool,
    pub(super) m_scratchFBOColorCount: u32,
    pub(super) m_scratchFBODepthAttachment: u32,
}

pub(super) struct ScratchPassObjects {
    pub(super) state: RefCell<ScratchPassState>,
    pub(super) execution: GLExecutionStamp,
}

impl Drop for ScratchPassObjects {
    fn drop(&mut self) {
        super::ore_context_gl_impl::destroyScratch(self);
    }
}

/// The source scratch field group uses shared ownership for Rust pass lifetimes.
#[repr(C)]
pub(crate) struct ContextGL {
    pub(super) base: ManuallyDrop<Context>,
    pub(super) m_renderContextImpl: *mut std::ffi::c_void,
    pub(super) m_savedState: GLSavedState,
    pub(super) m_executionStamp: ManuallyDrop<GLExecutionStamp>,
    pub(super) rust_scratch: Option<Rc<ScratchPassObjects>>,
}

impl ContextGL {
    pub(crate) fn newBase(
        features: Features,
        executionStamp: GLExecutionStamp,
        renderContextImpl: *mut std::ffi::c_void,
    ) -> Self {
        let base = nuxie_ore_metal::new_context_backend_base_with_final_release_drain(
            features,
            None,
            executionStamp.domain().resourceFinalReleaseDrain(),
        );
        Self {
            base: ManuallyDrop::new(base),
            m_savedState: GLSavedState::default(),
            m_renderContextImpl: renderContextImpl,
            rust_scratch: Some(Rc::new(ScratchPassObjects {
                state: RefCell::new(ScratchPassState::default()),
                execution: executionStamp.clone(),
            })),
            m_executionStamp: ManuallyDrop::new(executionStamp),
        }
    }

    /// Source `Make()` plus the shared current-context execution authority.
    pub(crate) fn Make(
        executionStamp: GLExecutionStamp,
        renderContextImpl: *mut std::ffi::c_void,
    ) -> Option<Box<Self>> {
        super::ore_context_gl_impl::Make(executionStamp, renderContextImpl)
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.m_executionStamp
    }

    pub(crate) fn executionDomain(&self) -> &GLExecutionDomain {
        self.executionStamp().domain()
    }

    pub(crate) fn shaderTarget(&self) -> ShaderTarget {
        let execution = self.executionStamp().clone();
        execution.withCurrent(|| ShaderTarget::glsl)
    }
}

impl Drop for ContextGL {
    fn drop(&mut self) {
        super::ore_context_gl_impl::destroy(self);
        unsafe {
            ManuallyDrop::drop(&mut self.base);
        }
        // Releasing the source Context may enqueue retained resources. Drain
        // them while this creation generation is still current; on a stale
        // generation the posted retirement path releases Rust ownership and
        // suppresses the stale numeric-name deletes instead.
        let execution = (&*self.m_executionStamp).clone();
        let _ = execution.withDeleteCurrent(|| {});
        unsafe {
            // ContextGL releases only its clone; RenderContextGLImpl remains
            // the execution-domain lifetime root for resource finalization.
            ManuallyDrop::drop(&mut self.m_executionStamp);
        }
    }
}

impl Deref for ContextGL {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ContextGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub(crate) const SOURCE_PUBLIC_METHOD_COUNT: usize = 19;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 3;
pub(crate) const SOURCE_FIELD_LEDGER_COUNT: usize = 14;
pub(crate) const SOURCE_DELETED_COPY_OPERATION_COUNT: usize = 2;
pub(crate) const RUST_EXECUTION_SIDECAR_COUNT: usize = 1;
const _: [(); 4764] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    #[test]
    fn complete_header_and_field_denominators_are_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 120);
        assert_eq!(SOURCE_PUBLIC_METHOD_COUNT, 19);
        assert_eq!(SOURCE_FRIEND_COUNT, 3);
        assert_eq!(SOURCE_FIELD_LEDGER_COUNT, 14);
        assert_eq!(SOURCE_DELETED_COPY_OPERATION_COUNT, 2);
        assert_eq!(RUST_EXECUTION_SIDECAR_COUNT, 1);
        assert_eq!(std::mem::size_of::<GLSavedState>(), 20);
        assert_eq!(std::mem::align_of::<GLSavedState>(), 4);
        assert_eq!(offset_of!(ContextGL, base), 0);
        assert!(offset_of!(ContextGL, m_executionStamp) > offset_of!(ContextGL, m_savedState));
    }
}
