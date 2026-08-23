//! Complete include-only implementation translation of
//! `renderer/src/ore/gl/ore_bind_group_gl.cpp`.

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_bind_group_gl.cpp");

pub(crate) use super::ore_bind_group_gl_decl::{
    BindGroupGL, GLSamplerBinding, GLTexBinding, GLUBOBinding,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_only_owner_invents_no_out_of_line_behavior() {
        assert_eq!(PINNED_SOURCE.lines().count(), 5);
        let _ = std::mem::size_of::<BindGroupGL>();
        let _ = std::mem::size_of::<GLUBOBinding>();
        let _ = std::mem::size_of::<GLTexBinding>();
        let _ = std::mem::size_of::<GLSamplerBinding>();
    }
}
