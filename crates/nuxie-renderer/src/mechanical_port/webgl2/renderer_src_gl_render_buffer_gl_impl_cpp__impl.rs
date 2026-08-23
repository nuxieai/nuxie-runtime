//! Complete mechanical implementation translation of
//! `renderer/src/gl/render_buffer_gl_impl.cpp` for `RIVE_WEBGL`.

#![allow(non_snake_case)]

use super::gles3_decl::*;
use super::render_buffer_gl_impl_decl::{GLStateOwner, RenderBufferGLImpl};
use crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags;
use std::mem::ManuallyDrop;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_gl_render_buffer_gl_impl.cpp");

pub(crate) fn init(buffer: &mut RenderBufferGLImpl, state: GLStateOwner) {
    assert!(buffer.m_state.is_none());
    assert_eq!(buffer.m_bufferID, 0);
    *buffer.m_state = Some(state);
    buffer.m_bufferID = allocateGLName();
    recordGLCommand(GLCommand::GenerateBuffer(buffer.m_bufferID));
    let mut state = buffer.state().borrow_mut();
    state.bindVAO(0);
    state.bindBuffer(buffer.m_target, buffer.m_bufferID);
    recordGLCommand(GLCommand::BufferData {
        target: buffer.m_target,
        size: buffer.base.sizeInBytes(),
        usage: if buffer.base.flags() as u8 & RenderBufferFlags::mappedOnceAtInitialization as u8
            != 0
        {
            GL_STATIC_DRAW
        } else {
            GL_DYNAMIC_DRAW
        },
    });
}

pub(crate) fn onMap(buffer: &mut RenderBufferGLImpl) -> *mut core::ffi::c_void {
    if buffer.m_fallbackMappedMemory.is_none() {
        *buffer.m_fallbackMappedMemory = Some(vec![0; buffer.base.sizeInBytes()]);
    }
    buffer
        .m_fallbackMappedMemory
        .as_mut()
        .expect("WebGL fallback mapping was allocated")
        .as_mut_ptr()
        .cast()
}

pub(crate) fn onUnmap(buffer: &mut RenderBufferGLImpl) {
    let bytes = buffer
        .m_fallbackMappedMemory
        .as_ref()
        .expect("RenderBufferGLImpl must be mapped before unmap")
        .clone();
    {
        let mut state = buffer.state().borrow_mut();
        state.bindVAO(0);
        state.bindBuffer(buffer.m_target, buffer.m_bufferID);
    }
    recordGLCommand(GLCommand::BufferSubData {
        target: buffer.m_target,
        offset: 0,
        data: bytes,
    });
    if buffer.base.flags() as u8 & RenderBufferFlags::mappedOnceAtInitialization as u8 != 0 {
        *buffer.m_fallbackMappedMemory = None;
    }
}

impl Drop for RenderBufferGLImpl {
    fn drop(&mut self) {
        if self.m_bufferID != 0 {
            self.state().borrow_mut().deleteBuffer(self.m_bufferID);
        }
        unsafe {
            // C++ destroys derived members in reverse declaration order, then
            // its RiveRenderBuffer base.
            ManuallyDrop::drop(&mut self.m_state);
            ManuallyDrop::drop(&mut self.m_fallbackMappedMemory);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::gl_state_decl::GLState;
    use super::*;
    use crate::mechanical_port::source::include::rive::renderer_hpp::{
        RenderBufferFlags, RenderBufferType,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn complete_webgl_implementation_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 120);
    }

    #[test]
    fn initialization_and_one_time_map_lower_exactly() {
        resetGLCommandStream();
        let state = Rc::new(RefCell::new(GLState::new(GLCapabilities::default())));
        takeGLCommands();
        let mut buffer = RenderBufferGLImpl::new(
            RenderBufferType::vertex,
            RenderBufferFlags::mappedOnceAtInitialization,
            4,
            state,
        );
        assert!(matches!(
            takeGLCommands().as_slice(),
            [
                GLCommand::GenerateBuffer(_),
                GLCommand::BindVertexArray(0),
                GLCommand::BindBuffer(GL_ARRAY_BUFFER, _),
                GLCommand::BufferData {
                    target: GL_ARRAY_BUFFER,
                    size: 4,
                    usage: GL_STATIC_DRAW
                }
            ]
        ));
        let mapped = onMap(&mut buffer).cast::<u8>();
        unsafe { std::ptr::copy_nonoverlapping([1_u8, 2, 3, 4].as_ptr(), mapped, 4) };
        onUnmap(&mut buffer);
        assert!(buffer.m_fallbackMappedMemory.is_none());
        assert!(matches!(
            takeGLCommands().last(),
            Some(GLCommand::BufferSubData { data, .. }) if data == &[1, 2, 3, 4]
        ));
    }
}
