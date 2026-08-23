//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/draw_shader_vulkan.cpp`.

#![allow(non_snake_case)]

use super::draw_shader_vulkan_decl::{DrawShaderVulkan, DrawShaderVulkanType};
use super::vulkan_context_decl::VulkanContext;
use super::vulkan_shaders_decl::{self as spirv, ShaderSlot};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawType, InterlockMode, ShaderFeatures, ShaderMiscFlags,
};
use ash::vk;
use std::sync::Arc;

#[derive(Clone, Copy)]
struct ShaderPair {
    vert: &'static ShaderSlot,
    frag: &'static ShaderSlot,
}

#[inline]
fn feature_is_set(features: ShaderFeatures, flag: ShaderFeatures) -> bool {
    (features & flag).0 != 0
}

#[inline]
fn misc_is_set(flags: ShaderMiscFlags, flag: ShaderMiscFlags) -> bool {
    (flags & flag).0 != 0
}

#[inline]
fn misc_any_set(flags: ShaderMiscFlags, flags_to_test: ShaderMiscFlags) -> bool {
    (flags & flags_to_test).0 != 0
}

#[cold]
#[track_caller]
fn source_unreachable() -> ! {
    panic!("RIVE_UNREACHABLE in pinned draw_shader_vulkan.cpp")
}

fn assert_msaa_color_output_configuration(
    shader_type: DrawShaderVulkanType,
    drawType: DrawType,
    shaderFeatures: ShaderFeatures,
    interlockMode: InterlockMode,
    shaderMiscFlags: ShaderMiscFlags,
) {
    if shader_type == DrawShaderVulkanType::fragment
        && interlockMode == InterlockMode::msaa
        && drawType != DrawType::renderPassInitialize
        && drawType != DrawType::renderPassResolve
    {
        // Fixed function output and advanced blend are mutually exclusive and
        // the source requires exactly one for fragment draws in MSAA mode.
        assert_ne!(
            misc_is_set(
                shaderMiscFlags,
                ShaderMiscFlags::fixedFunctionColorOutput,
            ),
            feature_is_set(shaderFeatures, ShaderFeatures::ENABLE_ADVANCED_BLEND)
        );
    }
}

fn select_shader_pair(
    drawType: DrawType,
    shaderFeatures: ShaderFeatures,
    interlockMode: InterlockMode,
    shaderMiscFlags: ShaderMiscFlags,
) -> ShaderPair {
    let fixedFunctionColorOutput = misc_is_set(
        shaderMiscFlags,
        ShaderMiscFlags::fixedFunctionColorOutput,
    );

    match interlockMode {
        InterlockMode::rasterOrdering => match drawType {
            DrawType::midpointFanPatches
            | DrawType::midpointFanCenterAAPatches
            | DrawType::outerCurvePatches => ShaderPair {
                vert: &spirv::draw_path_vert,
                frag: &spirv::draw_path_frag,
            },
            DrawType::interiorTriangulation => ShaderPair {
                vert: &spirv::draw_interior_triangles_vert,
                frag: &spirv::draw_interior_triangles_frag,
            },
            DrawType::featherAtlasBlit => ShaderPair {
                vert: &spirv::draw_atlas_blit_vert,
                frag: &spirv::draw_atlas_blit_frag,
            },
            DrawType::imageMesh => ShaderPair {
                vert: &spirv::draw_image_mesh_vert,
                frag: &spirv::draw_image_mesh_frag,
            },
            DrawType::renderPassResolve => ShaderPair {
                vert: &spirv::draw_fullscreen_quad_vert,
                frag: &spirv::draw_input_attachment_frag,
            },
            DrawType::imageRect
            | DrawType::msaaStrokes
            | DrawType::msaaMidpointFanBorrowedCoverage
            | DrawType::msaaDynamicMidpointFans
            | DrawType::msaaMidpointFans
            | DrawType::msaaMidpointFanStencilReset
            | DrawType::msaaMidpointFanPathsStencil
            | DrawType::msaaMidpointFanPathsCover
            | DrawType::msaaOuterCubics
            | DrawType::clipReset
            | DrawType::renderPassInitialize => source_unreachable(),
        },

        InterlockMode::atomics => match drawType {
            DrawType::midpointFanPatches
            | DrawType::midpointFanCenterAAPatches
            | DrawType::outerCurvePatches => ShaderPair {
                vert: &spirv::atomic_draw_path_vert,
                frag: if fixedFunctionColorOutput {
                    &spirv::atomic_draw_path_fixedcolor_frag
                } else {
                    &spirv::atomic_draw_path_frag
                },
            },
            DrawType::interiorTriangulation => ShaderPair {
                vert: &spirv::atomic_draw_interior_triangles_vert,
                frag: if fixedFunctionColorOutput {
                    &spirv::atomic_draw_interior_triangles_fixedcolor_frag
                } else {
                    &spirv::atomic_draw_interior_triangles_frag
                },
            },
            DrawType::featherAtlasBlit => ShaderPair {
                vert: &spirv::atomic_draw_atlas_blit_vert,
                frag: if fixedFunctionColorOutput {
                    &spirv::atomic_draw_atlas_blit_fixedcolor_frag
                } else {
                    &spirv::atomic_draw_atlas_blit_frag
                },
            },
            DrawType::imageRect => ShaderPair {
                vert: &spirv::atomic_draw_image_rect_vert,
                frag: if fixedFunctionColorOutput {
                    &spirv::atomic_draw_image_rect_fixedcolor_frag
                } else {
                    &spirv::atomic_draw_image_rect_frag
                },
            },
            DrawType::imageMesh => ShaderPair {
                vert: &spirv::atomic_draw_image_mesh_vert,
                frag: if fixedFunctionColorOutput {
                    &spirv::atomic_draw_image_mesh_fixedcolor_frag
                } else {
                    &spirv::atomic_draw_image_mesh_frag
                },
            },
            DrawType::renderPassResolve => {
                if misc_is_set(
                    shaderMiscFlags,
                    ShaderMiscFlags::coalescedResolveAndTransfer,
                ) {
                    ShaderPair {
                        vert: &spirv::atomic_resolve_coalesced_vert,
                        frag: &spirv::atomic_resolve_coalesced_frag,
                    }
                } else {
                    ShaderPair {
                        vert: &spirv::atomic_resolve_vert,
                        frag: if fixedFunctionColorOutput {
                            &spirv::atomic_resolve_fixedcolor_frag
                        } else {
                            &spirv::atomic_resolve_frag
                        },
                    }
                }
            }
            DrawType::msaaStrokes
            | DrawType::msaaMidpointFanBorrowedCoverage
            | DrawType::msaaDynamicMidpointFans
            | DrawType::msaaMidpointFanStencilReset
            | DrawType::msaaMidpointFans
            | DrawType::msaaMidpointFanPathsStencil
            | DrawType::msaaMidpointFanPathsCover
            | DrawType::msaaOuterCubics
            | DrawType::clipReset
            | DrawType::renderPassInitialize => source_unreachable(),
        },

        InterlockMode::clockwise => {
            #[cfg(target_os = "android")]
            {
                let _ = (drawType, shaderFeatures, fixedFunctionColorOutput);
                source_unreachable()
            }
            #[cfg(not(target_os = "android"))]
            match drawType {
                DrawType::midpointFanPatches
                | DrawType::midpointFanCenterAAPatches
                | DrawType::outerCurvePatches => ShaderPair {
                    vert: &spirv::draw_clockwise_path_vert,
                    frag: if misc_is_set(shaderMiscFlags, ShaderMiscFlags::clipUpdateOnly) {
                        if fixedFunctionColorOutput {
                            &spirv::draw_clockwise_clip_fixedcolor_frag
                        } else {
                            &spirv::draw_clockwise_clip_frag
                        }
                    } else if fixedFunctionColorOutput {
                        &spirv::draw_clockwise_path_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_path_frag
                    },
                },
                DrawType::interiorTriangulation => ShaderPair {
                    vert: &spirv::draw_clockwise_interior_triangles_vert,
                    frag: if misc_is_set(shaderMiscFlags, ShaderMiscFlags::clipUpdateOnly) {
                        if fixedFunctionColorOutput {
                            &spirv::draw_clockwise_clip_interior_triangles_fixedcolor_frag
                        } else {
                            &spirv::draw_clockwise_clip_interior_triangles_frag
                        }
                    } else if fixedFunctionColorOutput {
                        &spirv::draw_clockwise_interior_triangles_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_interior_triangles_frag
                    },
                },
                DrawType::featherAtlasBlit => ShaderPair {
                    vert: &spirv::draw_clockwise_atlas_blit_vert,
                    frag: if fixedFunctionColorOutput {
                        &spirv::draw_clockwise_atlas_blit_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_atlas_blit_frag
                    },
                },
                DrawType::imageMesh => ShaderPair {
                    vert: &spirv::draw_clockwise_image_mesh_vert,
                    frag: if fixedFunctionColorOutput {
                        &spirv::draw_clockwise_image_mesh_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_image_mesh_frag
                    },
                },
                DrawType::imageRect
                | DrawType::msaaStrokes
                | DrawType::msaaMidpointFanBorrowedCoverage
                | DrawType::msaaDynamicMidpointFans
                | DrawType::msaaMidpointFanStencilReset
                | DrawType::msaaMidpointFans
                | DrawType::msaaMidpointFanPathsStencil
                | DrawType::msaaMidpointFanPathsCover
                | DrawType::msaaOuterCubics
                | DrawType::clipReset
                | DrawType::renderPassResolve
                | DrawType::renderPassInitialize => source_unreachable(),
            }
        }

        InterlockMode::clockwiseAtomic => {
            let drawUsesAdvancedBlend = feature_is_set(
                shaderFeatures,
                ShaderFeatures::ENABLE_ADVANCED_BLEND,
            );
            match drawType {
                DrawType::midpointFanPatches
                | DrawType::midpointFanCenterAAPatches
                | DrawType::outerCurvePatches => ShaderPair {
                    vert: &spirv::draw_clockwise_atomic_path_vert,
                    frag: if misc_is_set(
                        shaderMiscFlags,
                        ShaderMiscFlags::borrowedCoveragePass,
                    ) {
                        assert!(fixedFunctionColorOutput);
                        assert!(!misc_any_set(
                            shaderMiscFlags,
                            ShaderMiscFlags::clipUpdateOnly
                                | ShaderMiscFlags::nestedClipUpdateOnly,
                        ));
                        assert!(!drawUsesAdvancedBlend);
                        &spirv::draw_clockwise_atomic_borrowed_coverage_frag
                    } else if misc_any_set(
                        shaderMiscFlags,
                        ShaderMiscFlags::clipUpdateOnly | ShaderMiscFlags::nestedClipUpdateOnly,
                    ) {
                        if !drawUsesAdvancedBlend {
                            &spirv::draw_clockwise_atomic_clip_fixedcolor_frag
                        } else {
                            &spirv::draw_clockwise_atomic_clip_frag
                        }
                    } else if !drawUsesAdvancedBlend {
                        &spirv::draw_clockwise_atomic_path_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_atomic_path_frag
                    },
                },
                DrawType::interiorTriangulation => ShaderPair {
                    vert: &spirv::draw_clockwise_atomic_interior_triangles_vert,
                    frag: if misc_is_set(
                        shaderMiscFlags,
                        ShaderMiscFlags::borrowedCoveragePass,
                    ) {
                        assert!(fixedFunctionColorOutput);
                        assert!(!misc_any_set(
                            shaderMiscFlags,
                            ShaderMiscFlags::clipUpdateOnly
                                | ShaderMiscFlags::nestedClipUpdateOnly,
                        ));
                        assert!(!drawUsesAdvancedBlend);
                        &spirv::draw_clockwise_atomic_borrowed_coverage_interior_triangles_frag
                    } else if misc_any_set(
                        shaderMiscFlags,
                        ShaderMiscFlags::clipUpdateOnly | ShaderMiscFlags::nestedClipUpdateOnly,
                    ) {
                        if !drawUsesAdvancedBlend {
                            &spirv::draw_clockwise_atomic_clip_interior_triangles_fixedcolor_frag
                        } else {
                            &spirv::draw_clockwise_atomic_clip_interior_triangles_frag
                        }
                    } else if !drawUsesAdvancedBlend {
                        &spirv::draw_clockwise_atomic_interior_triangles_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_atomic_interior_triangles_frag
                    },
                },
                DrawType::featherAtlasBlit => ShaderPair {
                    vert: &spirv::draw_clockwise_atomic_atlas_blit_vert,
                    frag: if !drawUsesAdvancedBlend {
                        &spirv::draw_clockwise_atomic_atlas_blit_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_atomic_atlas_blit_frag
                    },
                },
                DrawType::imageMesh => ShaderPair {
                    vert: &spirv::draw_clockwise_atomic_image_mesh_vert,
                    frag: if !drawUsesAdvancedBlend {
                        &spirv::draw_clockwise_atomic_image_mesh_fixedcolor_frag
                    } else {
                        &spirv::draw_clockwise_atomic_image_mesh_frag
                    },
                },
                DrawType::clipReset => ShaderPair {
                    vert: &spirv::clear_clockwise_atomic_clip_vert,
                    frag: if !drawUsesAdvancedBlend {
                        &spirv::clear_clockwise_atomic_clip_fixedcolor_frag
                    } else {
                        &spirv::clear_clockwise_atomic_clip_frag
                    },
                },
                DrawType::renderPassInitialize => ShaderPair {
                    vert: &spirv::draw_fullscreen_quad_vert,
                    frag: if fixedFunctionColorOutput {
                        &spirv::init_clockwise_atomic_workaround_fixedcolor_frag
                    } else {
                        &spirv::init_clockwise_atomic_workaround_frag
                    },
                },
                DrawType::imageRect
                | DrawType::msaaStrokes
                | DrawType::msaaMidpointFanBorrowedCoverage
                | DrawType::msaaDynamicMidpointFans
                | DrawType::msaaMidpointFanStencilReset
                | DrawType::msaaMidpointFans
                | DrawType::msaaMidpointFanPathsStencil
                | DrawType::msaaMidpointFanPathsCover
                | DrawType::msaaOuterCubics
                | DrawType::renderPassResolve => source_unreachable(),
            }
        }

        InterlockMode::msaa => match drawType {
            DrawType::midpointFanPatches
            | DrawType::midpointFanCenterAAPatches
            | DrawType::outerCurvePatches
            | DrawType::interiorTriangulation
            | DrawType::imageRect => source_unreachable(),
            DrawType::msaaOuterCubics
            | DrawType::msaaStrokes
            | DrawType::msaaMidpointFanBorrowedCoverage
            | DrawType::msaaDynamicMidpointFans
            | DrawType::msaaMidpointFans
            | DrawType::msaaMidpointFanStencilReset
            | DrawType::msaaMidpointFanPathsStencil
            | DrawType::msaaMidpointFanPathsCover => ShaderPair {
                vert: if feature_is_set(shaderFeatures, ShaderFeatures::ENABLE_CLIP_RECT) {
                    &spirv::draw_msaa_path_vert
                } else {
                    &spirv::draw_msaa_path_noclipdistance_vert
                },
                frag: if fixedFunctionColorOutput {
                    &spirv::draw_msaa_path_fixedcolor_frag
                } else {
                    &spirv::draw_msaa_path_frag
                },
            },
            DrawType::clipReset => ShaderPair {
                vert: &spirv::draw_msaa_stencil_vert,
                frag: &spirv::draw_msaa_stencil_frag,
            },
            DrawType::featherAtlasBlit => ShaderPair {
                vert: if feature_is_set(shaderFeatures, ShaderFeatures::ENABLE_CLIP_RECT) {
                    &spirv::draw_msaa_atlas_blit_vert
                } else {
                    &spirv::draw_msaa_atlas_blit_noclipdistance_vert
                },
                frag: if fixedFunctionColorOutput {
                    &spirv::draw_msaa_atlas_blit_fixedcolor_frag
                } else {
                    &spirv::draw_msaa_atlas_blit_frag
                },
            },
            DrawType::imageMesh => ShaderPair {
                vert: if feature_is_set(shaderFeatures, ShaderFeatures::ENABLE_CLIP_RECT) {
                    &spirv::draw_msaa_image_mesh_vert
                } else {
                    &spirv::draw_msaa_image_mesh_noclipdistance_vert
                },
                frag: if fixedFunctionColorOutput {
                    &spirv::draw_msaa_image_mesh_fixedcolor_frag
                } else {
                    &spirv::draw_msaa_image_mesh_frag
                },
            },
            DrawType::renderPassInitialize => ShaderPair {
                vert: &spirv::draw_fullscreen_quad_vert,
                frag: &spirv::draw_msaa_color_seed_attachment_frag,
            },
            DrawType::renderPassResolve => ShaderPair {
                vert: &spirv::draw_fullscreen_quad_vert,
                frag: &spirv::draw_msaa_resolve_frag,
            },
        },
    }
}

impl DrawShaderVulkan {
    pub(crate) fn new(
        shader_type: DrawShaderVulkanType,
        vk: &Arc<VulkanContext>,
        drawType: DrawType,
        shaderFeatures: ShaderFeatures,
        interlockMode: InterlockMode,
        shaderMiscFlags: ShaderMiscFlags,
    ) -> Self {
        assert_msaa_color_output_configuration(
            shader_type,
            drawType,
            shaderFeatures,
            interlockMode,
            shaderMiscFlags,
        );

        let pair = select_shader_pair(
            drawType,
            shaderFeatures,
            interlockMode,
            shaderMiscFlags,
        );
        let code = match shader_type {
            DrawShaderVulkanType::vertex => pair.vert.read(),
            DrawShaderVulkanType::fragment => pair.frag.read(),
        }
        .expect("selected shader symbol is undefined in pinned source");
        assert!(!code.is_empty());
        let createInfo = vk::ShaderModuleCreateInfo::default().code(&code);
        let module = unsafe { vk.m_ashDevice.create_shader_module(&createInfo, None) }
            .unwrap_or(vk::ShaderModule::null());

        Self {
            m_vk: Arc::clone(vk),
            m_module: module,
        }
    }
}

impl Drop for DrawShaderVulkan {
    fn drop(&mut self) {
        unsafe {
            self.m_vk
                .m_ashDevice
                .destroy_shader_module(self.m_module, None)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_pair(
        pair: ShaderPair,
        vert: &'static ShaderSlot,
        frag: &'static ShaderSlot,
    ) {
        assert!(std::ptr::eq(pair.vert, vert));
        assert!(std::ptr::eq(pair.frag, frag));
    }

    #[test]
    fn source_draw_and_interlock_selection_matrix_is_preserved() {
        let none = ShaderMiscFlags::none;
        let no_features = ShaderFeatures::NONE;
        assert_pair(
            select_shader_pair(
                DrawType::midpointFanPatches,
                no_features,
                InterlockMode::rasterOrdering,
                none,
            ),
            &spirv::draw_path_vert,
            &spirv::draw_path_frag,
        );
        assert_pair(
            select_shader_pair(
                DrawType::imageRect,
                no_features,
                InterlockMode::atomics,
                ShaderMiscFlags::fixedFunctionColorOutput,
            ),
            &spirv::atomic_draw_image_rect_vert,
            &spirv::atomic_draw_image_rect_fixedcolor_frag,
        );
        assert_pair(
            select_shader_pair(
                DrawType::renderPassResolve,
                no_features,
                InterlockMode::atomics,
                ShaderMiscFlags::coalescedResolveAndTransfer,
            ),
            &spirv::atomic_resolve_coalesced_vert,
            &spirv::atomic_resolve_coalesced_frag,
        );
        #[cfg(not(target_os = "android"))]
        assert_pair(
            select_shader_pair(
                DrawType::interiorTriangulation,
                no_features,
                InterlockMode::clockwise,
                ShaderMiscFlags::clipUpdateOnly
                    | ShaderMiscFlags::fixedFunctionColorOutput,
            ),
            &spirv::draw_clockwise_interior_triangles_vert,
            &spirv::draw_clockwise_clip_interior_triangles_fixedcolor_frag,
        );
        assert_pair(
            select_shader_pair(
                DrawType::midpointFanPatches,
                no_features,
                InterlockMode::clockwiseAtomic,
                ShaderMiscFlags::borrowedCoveragePass
                    | ShaderMiscFlags::fixedFunctionColorOutput,
            ),
            &spirv::draw_clockwise_atomic_path_vert,
            &spirv::draw_clockwise_atomic_borrowed_coverage_frag,
        );
        assert_pair(
            select_shader_pair(
                DrawType::featherAtlasBlit,
                ShaderFeatures::ENABLE_CLIP_RECT,
                InterlockMode::msaa,
                ShaderMiscFlags::fixedFunctionColorOutput,
            ),
            &spirv::draw_msaa_atlas_blit_vert,
            &spirv::draw_msaa_atlas_blit_fixedcolor_frag,
        );
        assert_pair(
            select_shader_pair(
                DrawType::renderPassResolve,
                no_features,
                InterlockMode::msaa,
                none,
            ),
            &spirv::draw_fullscreen_quad_vert,
            &spirv::draw_msaa_resolve_frag,
        );
    }

    #[test]
    #[should_panic(expected = "RIVE_UNREACHABLE")]
    fn source_unreachable_combination_remains_unreachable() {
        select_shader_pair(
            DrawType::imageRect,
            ShaderFeatures::NONE,
            InterlockMode::rasterOrdering,
            ShaderMiscFlags::none,
        );
    }

    #[test]
    #[should_panic]
    fn borrowed_coverage_invariants_remain_asserted() {
        select_shader_pair(
            DrawType::midpointFanPatches,
            ShaderFeatures::NONE,
            InterlockMode::clockwiseAtomic,
            ShaderMiscFlags::borrowedCoveragePass,
        );
    }

    #[test]
    #[should_panic]
    fn msaa_fragment_requires_exactly_one_color_output_path() {
        assert_msaa_color_output_configuration(
            DrawShaderVulkanType::fragment,
            DrawType::imageMesh,
            ShaderFeatures::NONE,
            InterlockMode::msaa,
            ShaderMiscFlags::none,
        );
    }
}
