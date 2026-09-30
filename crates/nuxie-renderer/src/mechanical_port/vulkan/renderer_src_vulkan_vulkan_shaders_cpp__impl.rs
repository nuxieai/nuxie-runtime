//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/vulkan_shaders.cpp`.
//! Updated through upstream `c14cb2510071bd4cfa08d52ba5cd44d98c362237`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::vulkan_shaders_decl::ShaderSlot;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/vulkan_spirv_embedded.rs"));
}

macro_rules! embedded_shaders {
    ($($name:ident),+ $(,)?) => {
        $(pub(crate) static $name: ShaderSlot = ShaderSlot::embedded(embedded::$name);)+
    };
}

embedded_shaders!(
    color_ramp_vert,
    color_ramp_frag,
    tessellate_vert,
    tessellate_frag,
    render_atlas_vert,
    render_atlas_fill_frag,
    render_atlas_stroke_frag,
    draw_path_vert,
    draw_path_frag,
    draw_interior_triangles_vert,
    draw_interior_triangles_frag,
    draw_atlas_blit_vert,
    draw_atlas_blit_frag,
    draw_image_mesh_vert,
    draw_image_mesh_frag,
);

#[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
embedded_shaders!(
    atomic_draw_path_vert,
    atomic_draw_path_frag,
    atomic_draw_path_fixedcolor_frag,
    atomic_draw_interior_triangles_vert,
    atomic_draw_interior_triangles_frag,
    atomic_draw_interior_triangles_fixedcolor_frag,
    atomic_draw_atlas_blit_vert,
    atomic_draw_atlas_blit_frag,
    atomic_draw_atlas_blit_fixedcolor_frag,
    atomic_draw_image_rect_vert,
    atomic_draw_image_rect_frag,
    atomic_draw_image_rect_fixedcolor_frag,
    atomic_draw_image_mesh_vert,
    atomic_draw_image_mesh_frag,
    atomic_draw_image_mesh_fixedcolor_frag,
    atomic_resolve_vert,
    atomic_resolve_frag,
    atomic_resolve_fixedcolor_frag,
    atomic_resolve_coalesced_vert,
    atomic_resolve_coalesced_frag,
);

#[cfg(not(target_os = "android"))]
embedded_shaders!(
    draw_clockwise_path_vert,
    draw_clockwise_path_frag,
    draw_clockwise_path_fixedcolor_frag,
    draw_clockwise_clip_frag,
    draw_clockwise_clip_fixedcolor_frag,
    draw_clockwise_interior_triangles_vert,
    draw_clockwise_interior_triangles_frag,
    draw_clockwise_interior_triangles_fixedcolor_frag,
    draw_clockwise_clip_interior_triangles_frag,
    draw_clockwise_clip_interior_triangles_fixedcolor_frag,
    draw_clockwise_atlas_blit_vert,
    draw_clockwise_atlas_blit_frag,
    draw_clockwise_atlas_blit_fixedcolor_frag,
    draw_clockwise_image_mesh_vert,
    draw_clockwise_image_mesh_frag,
    draw_clockwise_image_mesh_fixedcolor_frag,
);

#[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
embedded_shaders!(
    draw_clockwise_atomic_path_vert,
    draw_clockwise_atomic_path_frag,
    draw_clockwise_atomic_path_fixedcolor_frag,
    draw_clockwise_atomic_borrowed_coverage_frag,
    draw_clockwise_atomic_clip_frag,
    draw_clockwise_atomic_clip_fixedcolor_frag,
    draw_clockwise_atomic_interior_triangles_vert,
    draw_clockwise_atomic_interior_triangles_frag,
    draw_clockwise_atomic_interior_triangles_fixedcolor_frag,
    draw_clockwise_atomic_borrowed_coverage_interior_triangles_frag,
    draw_clockwise_atomic_clip_interior_triangles_frag,
    draw_clockwise_atomic_clip_interior_triangles_fixedcolor_frag,
    clear_clockwise_atomic_clip_vert,
    clear_clockwise_atomic_clip_frag,
    clear_clockwise_atomic_clip_fixedcolor_frag,
    draw_clockwise_atomic_atlas_blit_vert,
    draw_clockwise_atomic_atlas_blit_frag,
    draw_clockwise_atomic_atlas_blit_fixedcolor_frag,
    draw_clockwise_atomic_image_mesh_vert,
    draw_clockwise_atomic_image_mesh_frag,
    draw_clockwise_atomic_image_mesh_fixedcolor_frag,
    init_clockwise_atomic_workaround_frag,
    init_clockwise_atomic_workaround_fixedcolor_frag,
);

embedded_shaders!(
    draw_depthstencil_path_vert,
    draw_depthstencil_path_frag,
    draw_depthstencil_path_fixedcolor_frag,
    draw_depthstencil_path_msaa_frag,
    draw_depthstencil_fill_vert,
    draw_depthstencil_fill_noclipdistance_vert,
    draw_depthstencil_path_noclipdistance_vert,
    draw_depthstencil_triangles_nocolor_vert,
    draw_depthstencil_triangles_nocolor_frag,
    draw_depthstencil_triangles_nocolor_fixedcolor_frag,
    draw_depthstencil_atlas_blit_vert,
    draw_depthstencil_atlas_blit_frag,
    draw_depthstencil_atlas_blit_fixedcolor_frag,
    draw_depthstencil_atlas_blit_msaa_frag,
    draw_depthstencil_atlas_blit_noclipdistance_vert,
    draw_depthstencil_image_mesh_vert,
    draw_depthstencil_image_mesh_frag,
    draw_depthstencil_image_mesh_fixedcolor_frag,
    draw_depthstencil_image_mesh_msaa_frag,
    draw_depthstencil_image_mesh_noclipdistance_vert,
    draw_fullscreen_quad_vert,
    draw_input_attachment_frag,
    draw_msaa_color_seed_attachment_frag,
    draw_msaa_resolve_frag,
);

// Pinned header declaration with no definition/include in vulkan_shaders.cpp.
#[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
pub(crate) static init_clockwise_atomic_workaround_vert: ShaderSlot = ShaderSlot::undefined();

fn readNextBytecodeSpan(spirvData: &'static [u32], spirvIndex: &mut usize) -> &'static [u32] {
    let insnCount = spirvData[*spirvIndex] as usize;
    *spirvIndex += 1;
    let end = spirvIndex
        .checked_add(insnCount)
        .expect("hotload SPIR-V word count overflow");
    let insnData = &spirvData[*spirvIndex..end];
    *spirvIndex = end;
    insnData
}

fn visit_hotload_shaders(
    spirvData: &'static [u32],
    mut assign: impl FnMut(&'static ShaderSlot, &'static [u32]),
) {
    let mut spirvIndex = 0;
    macro_rules! read {
        ($slot:ident) => {
            assign(&$slot, readNextBytecodeSpan(spirvData, &mut spirvIndex))
        };
    }

    read!(color_ramp_vert);
    read!(color_ramp_frag);
    read!(tessellate_vert);
    read!(tessellate_frag);
    read!(render_atlas_vert);
    read!(render_atlas_fill_frag);
    read!(render_atlas_stroke_frag);
    read!(draw_path_vert);
    read!(draw_path_frag);
    read!(draw_interior_triangles_vert);
    read!(draw_interior_triangles_frag);
    read!(draw_atlas_blit_vert);
    read!(draw_atlas_blit_frag);
    read!(draw_image_mesh_vert);
    read!(draw_image_mesh_frag);

    #[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
    {
        read!(atomic_draw_path_vert);
        read!(atomic_draw_path_frag);
        read!(atomic_draw_path_fixedcolor_frag);
        read!(atomic_draw_interior_triangles_vert);
        read!(atomic_draw_interior_triangles_frag);
        read!(atomic_draw_interior_triangles_fixedcolor_frag);
        read!(atomic_draw_atlas_blit_vert);
        read!(atomic_draw_atlas_blit_frag);
        read!(atomic_draw_atlas_blit_fixedcolor_frag);
        read!(atomic_draw_image_rect_vert);
        read!(atomic_draw_image_rect_frag);
        read!(atomic_draw_image_rect_fixedcolor_frag);
        read!(atomic_draw_image_mesh_vert);
        read!(atomic_draw_image_mesh_frag);
        read!(atomic_draw_image_mesh_fixedcolor_frag);
        read!(atomic_resolve_vert);
        read!(atomic_resolve_frag);
        read!(atomic_resolve_fixedcolor_frag);
        read!(atomic_resolve_coalesced_vert);
        read!(atomic_resolve_coalesced_frag);
    }

    #[cfg(not(target_os = "android"))]
    {
        read!(draw_clockwise_path_vert);
        read!(draw_clockwise_path_frag);
        read!(draw_clockwise_path_fixedcolor_frag);
        read!(draw_clockwise_clip_frag);
        read!(draw_clockwise_clip_fixedcolor_frag);
        read!(draw_clockwise_interior_triangles_vert);
        read!(draw_clockwise_interior_triangles_frag);
        read!(draw_clockwise_interior_triangles_fixedcolor_frag);
        read!(draw_clockwise_clip_interior_triangles_frag);
        read!(draw_clockwise_clip_interior_triangles_fixedcolor_frag);
        read!(draw_clockwise_atlas_blit_vert);
        read!(draw_clockwise_atlas_blit_frag);
        read!(draw_clockwise_atlas_blit_fixedcolor_frag);
        read!(draw_clockwise_image_mesh_vert);
        read!(draw_clockwise_image_mesh_frag);
        read!(draw_clockwise_image_mesh_fixedcolor_frag);
    }

    #[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
    {
        read!(draw_clockwise_atomic_path_vert);
        read!(draw_clockwise_atomic_path_frag);
        read!(draw_clockwise_atomic_path_fixedcolor_frag);
        read!(draw_clockwise_atomic_clip_frag);
        read!(draw_clockwise_atomic_clip_fixedcolor_frag);
        read!(draw_clockwise_atomic_borrowed_coverage_frag);
        read!(draw_clockwise_atomic_interior_triangles_vert);
        read!(draw_clockwise_atomic_interior_triangles_frag);
        read!(draw_clockwise_atomic_interior_triangles_fixedcolor_frag);
        read!(draw_clockwise_atomic_clip_interior_triangles_frag);
        read!(draw_clockwise_atomic_clip_interior_triangles_fixedcolor_frag);
        read!(draw_clockwise_atomic_borrowed_coverage_interior_triangles_frag);

        read!(clear_clockwise_atomic_clip_vert);
        read!(clear_clockwise_atomic_clip_frag);
        read!(clear_clockwise_atomic_clip_fixedcolor_frag);
        read!(draw_clockwise_atomic_atlas_blit_vert);
        read!(draw_clockwise_atomic_atlas_blit_frag);
        read!(draw_clockwise_atomic_atlas_blit_fixedcolor_frag);
        read!(draw_clockwise_atomic_image_mesh_vert);
        read!(draw_clockwise_atomic_image_mesh_frag);
        read!(draw_clockwise_atomic_image_mesh_fixedcolor_frag);
        read!(init_clockwise_atomic_workaround_frag);
        read!(init_clockwise_atomic_workaround_fixedcolor_frag);
    }

    read!(draw_depthstencil_path_vert);
    read!(draw_depthstencil_path_noclipdistance_vert);
    read!(draw_depthstencil_path_frag);
    read!(draw_depthstencil_path_fixedcolor_frag);
    read!(draw_depthstencil_path_msaa_frag);
    read!(draw_depthstencil_fill_vert);
    read!(draw_depthstencil_fill_noclipdistance_vert);
    read!(draw_depthstencil_triangles_nocolor_vert);
    read!(draw_depthstencil_triangles_nocolor_frag);
    read!(draw_depthstencil_triangles_nocolor_fixedcolor_frag);
    read!(draw_depthstencil_atlas_blit_vert);
    read!(draw_depthstencil_atlas_blit_noclipdistance_vert);
    read!(draw_depthstencil_atlas_blit_frag);
    read!(draw_depthstencil_atlas_blit_fixedcolor_frag);
    read!(draw_depthstencil_atlas_blit_msaa_frag);
    read!(draw_depthstencil_image_mesh_vert);
    read!(draw_depthstencil_image_mesh_noclipdistance_vert);
    read!(draw_depthstencil_image_mesh_frag);
    read!(draw_depthstencil_image_mesh_fixedcolor_frag);
    read!(draw_depthstencil_image_mesh_msaa_frag);
    read!(draw_fullscreen_quad_vert);
    read!(draw_input_attachment_frag);
    read!(draw_msaa_color_seed_attachment_frag);
    read!(draw_msaa_resolve_frag);
}

// void hotload_shaders(rive::Span<const uint32_t> spirvData)
pub(crate) fn hotload_shaders(spirvData: &'static [u32]) {
    visit_hotload_shaders(spirvData, ShaderSlot::replace);
}

#[cfg(test)]
mod tests {
    use super::super::vulkan_shaders_decl::{
        DECLARED_SHADER_SYMBOL_COUNT, ShaderUnavailable, TARGET_SHADER_SYMBOL_COUNT,
    };
    use super::*;

    #[test]
    fn embedded_and_declared_shader_denominators_are_exact() {
        assert_eq!(DECLARED_SHADER_SYMBOL_COUNT, 99);
        #[cfg(not(target_os = "android"))]
        assert_eq!(TARGET_SHADER_SYMBOL_COUNT, 99);
        #[cfg(all(target_os = "android", feature = "android-vulkan-atomics"))]
        assert_eq!(TARGET_SHADER_SYMBOL_COUNT, 83);
        #[cfg(all(target_os = "android", not(feature = "android-vulkan-atomics")))]
        assert_eq!(TARGET_SHADER_SYMBOL_COUNT, 39);
        let color_ramp = color_ramp_vert.read().expect("embedded shader");
        assert_eq!(color_ramp.first(), Some(&0x0723_0203));
        assert!(color_ramp.len() > 4);
        drop(color_ramp);
        #[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
        assert!(matches!(
            init_clockwise_atomic_workaround_vert.read(),
            Err(ShaderUnavailable::UndefinedSourceSymbol)
        ));

        // The declaration-only workaround vertex has no embedded or hotload entry.
        let serialized_shader_count = TARGET_SHADER_SYMBOL_COUNT
            - usize::from(cfg!(any(
                not(target_os = "android"),
                feature = "android-vulkan-atomics"
            )));
        let mut hotload = Vec::with_capacity(serialized_shader_count * 2);
        for index in 0..serialized_shader_count {
            hotload.push(1);
            hotload.push(0x1000 + index as u32);
        }
        let hotload = Box::leak(hotload.into_boxed_slice());
        let mut assignments = Vec::with_capacity(serialized_shader_count);
        visit_hotload_shaders(hotload, |slot, words| assignments.push((slot, words)));
        assert_eq!(assignments.len(), serialized_shader_count);
        assert!(std::ptr::eq(assignments[0].0, &color_ramp_vert));
        assert_eq!(assignments[0].1, &[0x1000]);
        #[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
        {
            let atlas_blit_assignments = assignments
                .iter()
                .filter(|(slot, _)| std::ptr::eq(*slot, &draw_clockwise_atomic_atlas_blit_vert))
                .collect::<Vec<_>>();
            assert_eq!(atlas_blit_assignments.len(), 1);
            #[cfg(not(target_os = "android"))]
            assert_eq!(atlas_blit_assignments[0].1, &[0x1042]);
            #[cfg(target_os = "android")]
            assert_eq!(atlas_blit_assignments[0].1, &[0x1032]);
        }
        #[cfg(any(not(target_os = "android"), feature = "android-vulkan-atomics"))]
        assert!(matches!(
            init_clockwise_atomic_workaround_vert.read(),
            Err(ShaderUnavailable::UndefinedSourceSymbol)
        ));
        #[cfg(not(target_os = "android"))]
        assert!(std::ptr::eq(
            assignments.last().expect("last shader assignment").0,
            &draw_msaa_resolve_frag,
        ));
        #[cfg(target_os = "android")]
        assert!(std::ptr::eq(
            assignments.last().expect("last shader assignment").0,
            &draw_msaa_resolve_frag,
        ));
        #[cfg(not(target_os = "android"))]
        assert_eq!(assignments.last().unwrap().1, &[0x1061]);
        #[cfg(all(target_os = "android", feature = "android-vulkan-atomics"))]
        assert_eq!(assignments.last().unwrap().1, &[0x1051]);
        #[cfg(all(target_os = "android", not(feature = "android-vulkan-atomics")))]
        assert_eq!(assignments.last().unwrap().1, &[0x1026]);
    }
}
