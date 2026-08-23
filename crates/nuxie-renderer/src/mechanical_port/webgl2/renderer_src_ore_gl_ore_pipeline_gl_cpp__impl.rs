//! Complete mechanical implementation translation of
//! `renderer/src/ore/gl/ore_pipeline_gl.cpp` for `ORE_BACKEND_GL`.

use super::gles3_decl::{recordGLCommand, GLCommand};
use super::ore_pipeline_gl_decl::PipelineGL;
use std::mem::ManuallyDrop;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_pipeline_gl.cpp");

impl Drop for PipelineGL {
    fn drop(&mut self) {
        let execution = self.executionStamp().clone();
        let _ = execution.withDeleteCurrent(|| {
            if self.m_glProgram != 0 {
                recordGLCommand(GLCommand::DeleteProgram(self.m_glProgram));
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
        assert_eq!(PINNED_SOURCE.lines().count(), 23);
    }
}
