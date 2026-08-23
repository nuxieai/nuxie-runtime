//! Complete bridge declaration translation of
//! `renderer/include/rive/renderer/gl/load_gles_extensions.hpp`.

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_load_gles_extensions.hpp");

pub(crate) use super::gles3_decl::{GLCapabilities, GLenum, GLint, GLuint};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_is_exactly_the_platform_gl_type_seam() {
        assert_eq!(PINNED_SOURCE.lines().count(), 10);
        assert_eq!(std::mem::size_of::<GLenum>(), 4);
        assert_eq!(std::mem::size_of::<GLint>(), 4);
        assert_eq!(std::mem::size_of::<GLuint>(), 4);
        let _ = GLCapabilities::default();
    }
}
