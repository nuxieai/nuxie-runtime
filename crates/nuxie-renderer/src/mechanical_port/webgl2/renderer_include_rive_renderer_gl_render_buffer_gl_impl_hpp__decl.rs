//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/gl/render_buffer_gl_impl.hpp`.

#![allow(non_snake_case)]

use super::gl_state_decl::GLState;
use super::gles3_decl::{GLExecutionStamp, GLenum, GLuint};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBufferContract, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    LiteRttiCastFrom, LiteRttiTypeId, CONST_ID,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RiveRenderBuffer;
use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_render_buffer_gl_impl.hpp");

pub(crate) type GLStateOwner = Rc<RefCell<GLState>>;

#[repr(C)]
pub(crate) struct RenderBufferGLImpl {
    pub(crate) base: ManuallyDrop<RiveRenderBuffer>,
    pub(crate) m_target: GLenum,
    pub(crate) m_bufferID: GLuint,
    pub(crate) m_fallbackMappedMemory: ManuallyDrop<Option<Vec<u8>>>,
    pub(crate) m_state: ManuallyDrop<Option<GLStateOwner>>,

    /// Rust-only creation identity after the complete source field prefix.
    pub(crate) rust_execution: ManuallyDrop<Option<GLExecutionStamp>>,
}
impl RenderBufferGLImpl {
    pub(crate) fn new(
        bufferType: RenderBufferType,
        flags: RenderBufferFlags,
        sizeInBytes: usize,
        state: GLStateOwner,
    ) -> Self {
        let mut owner = Self::newUninitialized(bufferType, flags, sizeInBytes);
        owner.init(state);
        owner
    }

    pub(crate) fn newUninitialized(
        bufferType: RenderBufferType,
        flags: RenderBufferFlags,
        sizeInBytes: usize,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(unsafe {
                RiveRenderBuffer::new_for_owner::<Self>(bufferType, flags, sizeInBytes)
            }),
            m_target: if bufferType == RenderBufferType::vertex {
                super::gles3_decl::GL_ARRAY_BUFFER
            } else {
                super::gles3_decl::GL_ELEMENT_ARRAY_BUFFER
            },
            m_bufferID: 0,
            m_fallbackMappedMemory: ManuallyDrop::new(None),
            m_state: ManuallyDrop::new(None),
            rust_execution: ManuallyDrop::new(None),
        }
    }

    pub(crate) fn init(&mut self, state: GLStateOwner) {
        super::render_buffer_gl_impl_impl::init(self, state)
    }

    pub(crate) const fn bufferID(&self) -> GLuint {
        self.m_bufferID
    }

    pub(crate) fn detachBuffer(&mut self) -> GLuint {
        std::mem::take(&mut self.m_bufferID)
    }

    pub(crate) fn state(&self) -> &GLStateOwner {
        self.m_state
            .as_ref()
            .expect("RenderBufferGLImpl source state is initialized")
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        self.rust_execution
            .as_ref()
            .expect("RenderBufferGLImpl execution stamp is initialized")
    }
}

impl Deref for RenderBufferGLImpl {
    type Target = RiveRenderBuffer;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for RenderBufferGLImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl LiteRttiTypeId for RenderBufferGLImpl {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("RenderBufferGLImpl");
}

impl LiteRttiCastFrom<crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer>
    for RenderBufferGLImpl
{
    unsafe fn from_base(
        base: *mut crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer,
    ) -> *mut Self {
        base.cast()
    }
}

impl RenderBufferContract for RenderBufferGLImpl {
    fn onMap(&mut self) -> *mut core::ffi::c_void {
        super::render_buffer_gl_impl_impl::onMap(self)
    }

    fn onUnmap(&mut self) {
        super::render_buffer_gl_impl_impl::onUnmap(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_header_denominator_and_base_layout_are_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 59);
        assert_eq!(std::mem::offset_of!(RenderBufferGLImpl, base), 0);
        assert!(
            std::mem::offset_of!(RenderBufferGLImpl, m_state)
                > std::mem::offset_of!(RenderBufferGLImpl, m_fallbackMappedMemory)
        );
        assert!(
            std::mem::offset_of!(RenderBufferGLImpl, rust_execution)
                > std::mem::offset_of!(RenderBufferGLImpl, m_state)
        );
    }
}
