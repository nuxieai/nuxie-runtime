//! Mechanical implementation translation of
//! `renderer/src/gl/load_store_actions_ext.cpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::load_store_actions_ext_decl::LoadStoreActionsEXT;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    FlushDescriptor, LoadAction, ShaderFeatures,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_gl_load_store_actions_ext.cpp");

const GLSL_CLEAR_COLOR: &str = "QE";
const GLSL_LOAD_COLOR: &str = "SE";
const GLSL_STORE_COLOR: &str = "ZD";
const GLSL_CLEAR_COVERAGE: &str = "AE";
const GLSL_CLEAR_CLIP: &str = "QF";
const GLSL_PLS_LOAD_STORE_EXT: &str =
    include_str!("../webgpu/source/generated_glsl/pls_load_store_ext.minified.glsl");

pub(crate) fn BuildLoadActionsEXT(
    desc: &FlushDescriptor,
    clearColor4f: &mut [f32; 4],
) -> LoadStoreActionsEXT {
    let mut actions = LoadStoreActionsEXT::clearCoverage;
    if desc.colorLoadAction == LoadAction::clear {
        let color = desc.colorClearValue;
        let inverse255 = 1.0f32 / 255.0f32;
        let alpha = ((color >> 24) & 0xff) as f32 * inverse255;
        *clearColor4f = [
            ((color >> 16) & 0xff) as f32 * inverse255 * alpha,
            ((color >> 8) & 0xff) as f32 * inverse255 * alpha,
            (color & 0xff) as f32 * inverse255 * alpha,
            alpha,
        ];
        actions |= LoadStoreActionsEXT::clearColor;
    } else if desc.colorLoadAction == LoadAction::preserveRenderTarget {
        actions |= LoadStoreActionsEXT::loadColor;
    }
    if desc.combinedShaderFeatures.0 & ShaderFeatures::ENABLE_CLIPPING.0 != 0 {
        actions |= LoadStoreActionsEXT::clearClip;
    }
    actions
}

pub(crate) fn BuildLoadStoreEXTGLSL<'a>(
    shader: &'a mut String,
    actions: LoadStoreActionsEXT,
) -> &'a mut String {
    for (action, name) in [
        (LoadStoreActionsEXT::clearColor, GLSL_CLEAR_COLOR),
        (LoadStoreActionsEXT::loadColor, GLSL_LOAD_COLOR),
        (LoadStoreActionsEXT::storeColor, GLSL_STORE_COLOR),
        (LoadStoreActionsEXT::clearCoverage, GLSL_CLEAR_COVERAGE),
        (LoadStoreActionsEXT::clearClip, GLSL_CLEAR_CLIP),
    ] {
        if actions.has(action) {
            shader.push_str("#define ");
            shader.push_str(name);
            shader.push('\n');
        }
    }
    shader.push_str(GLSL_PLS_LOAD_STORE_EXT);
    shader
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_source_action_emits_its_exact_minified_define_in_order() {
        let mut shader = String::new();
        let all = LoadStoreActionsEXT(
            LoadStoreActionsEXT::clearColor.0
                | LoadStoreActionsEXT::loadColor.0
                | LoadStoreActionsEXT::storeColor.0
                | LoadStoreActionsEXT::clearCoverage.0
                | LoadStoreActionsEXT::clearClip.0,
        );
        BuildLoadStoreEXTGLSL(&mut shader, all);
        assert!(shader.starts_with("#define QE\n#define SE\n#define ZD\n#define AE\n#define QF\n"));
        assert!(shader.ends_with(GLSL_PLS_LOAD_STORE_EXT));
    }
}
