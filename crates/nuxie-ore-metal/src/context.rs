// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_context.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

//! Backend-independent ORE context state.
//!
//! The C++ context keeps a non-owning `RenderPass*`. Rust represents the same
//! relationship with a weak trait-object token. A render pass owns the strong
//! token together with its encoder, so finishing a stale pass is safe without
//! making the context keep that pass alive. The printf-style error setter is
//! translated as an owned string sink; callers format before entering this
//! module, which removes the C variadic boundary without changing observable
//! error text or ordering.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::ffi::c_void;
use std::ptr::NonNull;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use std::sync::{Arc, Mutex, MutexGuard, Weak};

#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::gpu_resource::GpuResourceManager;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::types::Features;

/// RSTB shader variant selected by a concrete backend.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderTarget {
    wgsl = 0,
    glsl = 1,
    msl = 2,
    hlsl = 3,
    spirv = 5,
}

/// Frame numbers and optional externally supplied command buffer.
///
/// Metal ignores `externalCommandBuffer` in the pinned source and allocates a
/// command buffer from its queue. It remains in the portable descriptor
/// because other ORE backends consume it.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameDescriptor {
    pub externalCommandBuffer: Option<NonNull<c_void>>,
    pub safeFrameNumber: u64,
    pub currentFrameNumber: u64,
}

/// Narrow state owned by a live render pass and observed weakly by a context.
///
/// Keeping this interface here mirrors the two operations used by
/// `Context::finishActiveRenderPass`; draw and encoder behavior remain in the
/// render-pass translation.
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub(crate) trait ActiveRenderPass: Send + Sync {
    fn is_finished(&self) -> bool;
    fn finish(&self);
}

/// Cross-cutting state shared by a concrete context and its render passes.
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub(crate) struct ContextState {
    // Rust fields drop in declaration order. C++ destroys Context members in
    // reverse declaration order: manager, error, active pointer, features.
    manager: Option<GpuResourceManager>,
    last_error: Mutex<String>,
    active_render_pass: Mutex<Option<Weak<dyn ActiveRenderPass>>>,
    features: Features,
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
impl ContextState {
    pub(crate) fn new(features: Features, manager: Option<GpuResourceManager>) -> Arc<Self> {
        Arc::new(Self {
            manager,
            last_error: Mutex::new(String::new()),
            active_render_pass: Mutex::new(None),
            features,
        })
    }

    pub(crate) fn features(&self) -> &Features {
        &self.features
    }

    pub(crate) fn manager(&self) -> Option<GpuResourceManager> {
        self.manager.clone()
    }

    pub(crate) fn last_error(&self) -> String {
        self.lock_last_error().clone()
    }

    pub(crate) fn set_last_error(&self, message: impl Into<String>) {
        *self.lock_last_error() = message.into();
    }

    pub(crate) fn clear_last_error(&self) {
        self.lock_last_error().clear();
    }

    /// Register a non-owning token for Lua-style stale-pass auto-finish.
    pub(crate) fn set_active_render_pass(&self, pass: Arc<dyn ActiveRenderPass>) {
        *self.lock_active_render_pass() = Some(Arc::downgrade(&pass));
    }

    /// Finish a previous live pass but preserve token identity, matching C++.
    pub(crate) fn finish_active_render_pass(&self) {
        let active = self
            .lock_active_render_pass()
            .as_ref()
            .and_then(Weak::upgrade);
        if let Some(pass) = active
            && !pass.is_finished()
        {
            pass.finish();
        }
    }

    fn lock_last_error(&self) -> MutexGuard<'_, String> {
        self.last_error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn lock_active_render_pass(&self) -> MutexGuard<'_, Option<Weak<dyn ActiveRenderPass>>> {
        self.active_render_pass
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(all(test, any(target_os = "ios", target_os = "macos")))]
mod tests {
    use super::*;
    use crate::gpu_resource::GpuResourceManagerOwner;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct TestPass {
        finished: AtomicBool,
    }

    impl ActiveRenderPass for TestPass {
        fn is_finished(&self) -> bool {
            self.finished.load(Ordering::Relaxed)
        }

        fn finish(&self) {
            self.finished.store(true, Ordering::Relaxed);
        }
    }

    #[test]
    fn error_state_is_replaced_and_cleared_explicitly() {
        let owner = GpuResourceManagerOwner::new();
        let state = ContextState::new(Features::default(), Some(owner.manager()));
        state.set_last_error("previous failure");
        assert_eq!(state.last_error(), "previous failure");
        state.set_last_error("next failure");
        assert_eq!(state.last_error(), "next failure");
        state.clear_last_error();
        assert_eq!(state.last_error(), "");
        let manager = state.manager().expect("manager");
        assert_eq!(manager.safe_frame_number(), 0);
        assert_eq!(manager.current_frame_number(), 0);
    }

    #[test]
    fn active_pass_is_weakly_held_and_finished_at_most_once() {
        let state = ContextState::new(Features::default(), None);
        let pass = Arc::new(TestPass {
            finished: AtomicBool::new(false),
        });
        state.set_active_render_pass(pass.clone());

        state.finish_active_render_pass();
        assert!(pass.is_finished());
        state.finish_active_render_pass();
        assert!(pass.is_finished());

        let weak = Arc::downgrade(&pass);
        drop(pass);
        assert!(weak.upgrade().is_none(), "context must not own the pass");
        state.finish_active_render_pass();
    }

    #[test]
    fn shader_target_values_preserve_the_rstb_wire_format_gap() {
        assert_eq!(ShaderTarget::wgsl as u8, 0);
        assert_eq!(ShaderTarget::msl as u8, 2);
        assert_eq!(ShaderTarget::spirv as u8, 5);
    }
}
