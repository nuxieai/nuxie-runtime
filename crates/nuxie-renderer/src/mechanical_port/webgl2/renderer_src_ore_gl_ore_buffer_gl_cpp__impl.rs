//! Complete mechanical implementation translation of
//! `renderer/src/ore/gl/ore_buffer_gl.cpp` for `ORE_BACKEND_GL`.

#![allow(non_snake_case)]

use super::gles3_decl::*;
use super::ore_buffer_gl_decl::BufferGL;
use nuxie_ore_metal::buffer::BufferUpdateError;
use nuxie_ore_metal::types::BufferUsage;
use std::mem::ManuallyDrop;

pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_ore_gl_ore_buffer_gl.cpp");

pub(crate) fn update(
    buffer: &BufferGL,
    data: &[u8],
    size: u32,
    offset: u32,
) -> Result<(), BufferUpdateError> {
    let execution = buffer.executionStamp().clone();
    execution.withCurrent(|| updateCurrent(buffer, data, size, offset))
}

fn updateCurrent(
    buffer: &BufferGL,
    data: &[u8],
    size: u32,
    offset: u32,
) -> Result<(), BufferUpdateError> {
    let end = offset
        .checked_add(size)
        .ok_or(BufferUpdateError::RangeOverflow)?;
    if end > buffer.base.size() {
        return Err(BufferUpdateError::RangeOutOfBounds);
    }
    let bytes = data
        .get(..size as usize)
        .ok_or(BufferUpdateError::SourceTooShort)?;
    assert!(buffer.m_glBuffer != 0);

    if buffer.base.usage() == BufferUsage::index {
        let previousBinding = buffer
            .executionStamp()
            .domain()
            .getInteger(GL_ELEMENT_ARRAY_BUFFER_BINDING);
        recordGLCommand(GLCommand::BindBuffer(
            GL_ELEMENT_ARRAY_BUFFER,
            buffer.m_glBuffer,
        ));
        recordGLCommand(GLCommand::BufferSubData {
            target: GL_ELEMENT_ARRAY_BUFFER,
            offset,
            data: bytes.to_vec(),
        });
        recordGLCommand(GLCommand::BindBuffer(
            GL_ELEMENT_ARRAY_BUFFER,
            previousBinding as GLuint,
        ));
    } else {
        recordGLCommand(GLCommand::BindBuffer(
            GL_COPY_WRITE_BUFFER,
            buffer.m_glBuffer,
        ));
        recordGLCommand(GLCommand::BufferSubData {
            target: GL_COPY_WRITE_BUFFER,
            offset,
            data: bytes.to_vec(),
        });
        recordGLCommand(GLCommand::BindBuffer(GL_COPY_WRITE_BUFFER, 0));
    }
    Ok(())
}

impl Drop for BufferGL {
    fn drop(&mut self) {
        let execution = self.executionStamp().clone();
        let _ = execution.withDeleteCurrent(|| {
            if self.m_glBuffer != 0 {
                recordGLCommand(GLCommand::DeleteBuffer(self.m_glBuffer));
            }
        });
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_implementation_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 53);
    }
}
