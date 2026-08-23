//! Complete mechanical implementation translation of
//! `renderer/src/ore/gl/ore_shader_module_gl.cpp` for `ORE_BACKEND_GL`.

use super::gles3_decl::{GLCommand, recordGLCommand};
use super::ore_shader_module_gl_decl::ShaderModuleGL;
use std::mem::ManuallyDrop;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_shader_module_gl.cpp");

impl Drop for ShaderModuleGL {
    fn drop(&mut self) {
        if self.m_glShader != 0 {
            recordGLCommand(GLCommand::DeleteShader(self.m_glShader));
        }
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_implementation_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 23);
    }

    #[test]
    fn nonzero_shader_is_deleted_once() {
        super::super::gles3_decl::resetGLCommandStream();
        let mut shader = ShaderModuleGL::new();
        shader.m_glShader = 23;
        drop(shader);
        assert_eq!(
            super::super::gles3_decl::takeGLCommands(),
            vec![GLCommand::DeleteShader(23)]
        );
    }
}
