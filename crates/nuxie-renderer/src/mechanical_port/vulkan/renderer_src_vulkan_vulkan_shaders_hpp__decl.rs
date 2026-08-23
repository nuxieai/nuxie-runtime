//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/vulkan_shaders.hpp`.

#![allow(non_camel_case_types)]

use std::ops::Deref;
use std::sync::{RwLock, RwLockReadGuard};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShaderUnavailable {
    UndefinedSourceSymbol,
}

/// Mutable source `Span<const uint32_t>` global with process lifetime.
pub(crate) struct ShaderSlot {
    words: RwLock<Option<&'static [u32]>>,
}

impl ShaderSlot {
    pub(crate) const fn embedded(words: &'static [u32]) -> Self {
        Self {
            words: RwLock::new(Some(words)),
        }
    }

    /// Models the pinned header's declaration-only
    /// `init_clockwise_atomic_workaround_vert` symbol. The implementation has
    /// no embedded definition and hotload never assigns it.
    pub(crate) const fn undefined() -> Self {
        Self {
            words: RwLock::new(None),
        }
    }

    pub(crate) fn read(&self) -> Result<ShaderRead<'_>, ShaderUnavailable> {
        let guard = self
            .words
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.is_none() {
            return Err(ShaderUnavailable::UndefinedSourceSymbol);
        }
        Ok(ShaderRead { guard })
    }

    pub(super) fn replace(&self, words: &'static [u32]) {
        *self
            .words
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(words);
    }
}

pub(crate) struct ShaderRead<'a> {
    guard: RwLockReadGuard<'a, Option<&'static [u32]>>,
}

impl Deref for ShaderRead<'_> {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        self.guard
            .as_ref()
            .expect("ShaderRead is only constructed for a defined source symbol")
    }
}

// Exact source extern denominator, re-exported from the implementation owner.
pub(crate) use super::vulkan_shaders_impl::{
    atomic_draw_atlas_blit_fixedcolor_frag, atomic_draw_atlas_blit_frag,
    atomic_draw_atlas_blit_vert, atomic_draw_image_mesh_fixedcolor_frag,
    atomic_draw_image_mesh_frag, atomic_draw_image_mesh_vert,
    atomic_draw_image_rect_fixedcolor_frag, atomic_draw_image_rect_frag,
    atomic_draw_image_rect_vert, atomic_draw_interior_triangles_fixedcolor_frag,
    atomic_draw_interior_triangles_frag, atomic_draw_interior_triangles_vert,
    atomic_draw_path_fixedcolor_frag, atomic_draw_path_frag, atomic_draw_path_vert,
    atomic_resolve_coalesced_frag, atomic_resolve_coalesced_vert,
    atomic_resolve_fixedcolor_frag, atomic_resolve_frag, atomic_resolve_vert,
    clear_clockwise_atomic_clip_fixedcolor_frag, clear_clockwise_atomic_clip_frag,
    clear_clockwise_atomic_clip_vert, color_ramp_frag, color_ramp_vert,
    draw_atlas_blit_frag, draw_atlas_blit_vert,
    draw_clockwise_atomic_atlas_blit_fixedcolor_frag,
    draw_clockwise_atomic_atlas_blit_frag, draw_clockwise_atomic_atlas_blit_vert,
    draw_clockwise_atomic_borrowed_coverage_frag,
    draw_clockwise_atomic_borrowed_coverage_interior_triangles_frag,
    draw_clockwise_atomic_clip_fixedcolor_frag, draw_clockwise_atomic_clip_frag,
    draw_clockwise_atomic_clip_interior_triangles_fixedcolor_frag,
    draw_clockwise_atomic_clip_interior_triangles_frag,
    draw_clockwise_atomic_image_mesh_fixedcolor_frag,
    draw_clockwise_atomic_image_mesh_frag, draw_clockwise_atomic_image_mesh_vert,
    draw_clockwise_atomic_interior_triangles_fixedcolor_frag,
    draw_clockwise_atomic_interior_triangles_frag,
    draw_clockwise_atomic_interior_triangles_vert,
    draw_clockwise_atomic_path_fixedcolor_frag, draw_clockwise_atomic_path_frag,
    draw_clockwise_atomic_path_vert, draw_fullscreen_quad_vert, draw_image_mesh_frag,
    draw_image_mesh_vert, draw_input_attachment_frag, draw_interior_triangles_frag,
    draw_interior_triangles_vert, draw_msaa_atlas_blit_fixedcolor_frag,
    draw_msaa_atlas_blit_frag, draw_msaa_atlas_blit_noclipdistance_vert,
    draw_msaa_atlas_blit_vert, draw_msaa_color_seed_attachment_frag,
    draw_msaa_image_mesh_fixedcolor_frag, draw_msaa_image_mesh_frag,
    draw_msaa_image_mesh_noclipdistance_vert, draw_msaa_image_mesh_vert,
    draw_msaa_path_fixedcolor_frag, draw_msaa_path_frag,
    draw_msaa_path_noclipdistance_vert, draw_msaa_path_vert, draw_msaa_resolve_frag,
    draw_msaa_stencil_fixedcolor_frag, draw_msaa_stencil_frag, draw_msaa_stencil_vert,
    draw_path_frag, draw_path_vert, hotload_shaders,
    init_clockwise_atomic_workaround_fixedcolor_frag,
    init_clockwise_atomic_workaround_frag, init_clockwise_atomic_workaround_vert,
    render_atlas_fill_frag, render_atlas_stroke_frag, render_atlas_vert, tessellate_frag,
    tessellate_vert,
};

#[cfg(not(target_os = "android"))]
pub(crate) use super::vulkan_shaders_impl::{
    draw_clockwise_atlas_blit_fixedcolor_frag, draw_clockwise_atlas_blit_frag,
    draw_clockwise_atlas_blit_vert, draw_clockwise_clip_fixedcolor_frag,
    draw_clockwise_clip_frag, draw_clockwise_clip_interior_triangles_fixedcolor_frag,
    draw_clockwise_clip_interior_triangles_frag,
    draw_clockwise_image_mesh_fixedcolor_frag, draw_clockwise_image_mesh_frag,
    draw_clockwise_image_mesh_vert, draw_clockwise_interior_triangles_fixedcolor_frag,
    draw_clockwise_interior_triangles_frag, draw_clockwise_interior_triangles_vert,
    draw_clockwise_path_fixedcolor_frag, draw_clockwise_path_frag, draw_clockwise_path_vert,
};

pub(crate) const DECLARED_SHADER_SYMBOL_COUNT: usize = 94;
#[cfg(target_os = "android")]
pub(crate) const TARGET_SHADER_SYMBOL_COUNT: usize = DECLARED_SHADER_SYMBOL_COUNT - 16;
#[cfg(not(target_os = "android"))]
pub(crate) const TARGET_SHADER_SYMBOL_COUNT: usize = DECLARED_SHADER_SYMBOL_COUNT;
