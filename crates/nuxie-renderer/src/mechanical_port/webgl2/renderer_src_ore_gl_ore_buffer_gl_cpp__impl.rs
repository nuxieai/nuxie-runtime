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
        let previousBinding = allocateGLQuerySlot();
        recordGLCommand(GLCommand::GetInteger(
            GL_ELEMENT_ARRAY_BUFFER_BINDING,
            previousBinding,
        ));
        recordGLCommand(GLCommand::BindBuffer(
            GL_ELEMENT_ARRAY_BUFFER,
            buffer.m_glBuffer,
        ));
        recordGLCommand(GLCommand::BufferSubData {
            target: GL_ELEMENT_ARRAY_BUFFER,
            offset,
            data: bytes.to_vec(),
        });
        recordGLCommand(GLCommand::BindBufferFromQuery(
            GL_ELEMENT_ARRAY_BUFFER,
            previousBinding,
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
        if self.m_glBuffer != 0 {
            recordGLCommand(GLCommand::DeleteBuffer(self.m_glBuffer));
        }
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_ore_metal::buffer::BufferApi;

    #[test]
    fn complete_implementation_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 53);
    }

    #[test]
    fn index_update_preserves_the_host_vao_element_binding() {
        resetGLCommandStream();
        let mut buffer = BufferGL::new(8, BufferUsage::index);
        buffer.m_glBuffer = 17;
        buffer.update(&[1, 2, 3, 4], 4, 2).unwrap();
        let commands = takeGLCommands();
        let GLCommand::GetInteger(GL_ELEMENT_ARRAY_BUFFER_BINDING, slot) = commands[0] else {
            panic!("index update must first save the host EBO binding")
        };
        assert_eq!(
            commands,
            vec![
                GLCommand::GetInteger(GL_ELEMENT_ARRAY_BUFFER_BINDING, slot),
                GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 17),
                GLCommand::BufferSubData {
                    target: GL_ELEMENT_ARRAY_BUFFER,
                    offset: 2,
                    data: vec![1, 2, 3, 4],
                },
                GLCommand::BindBufferFromQuery(GL_ELEMENT_ARRAY_BUFFER, slot),
            ]
        );
        drop(buffer);
        assert_eq!(takeGLCommands(), vec![GLCommand::DeleteBuffer(17)]);
    }
}
