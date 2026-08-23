//! Compiled module tree for the pinned source-shaped renderer translation.

// This tree deliberately preserves source declarations, conditional branches,
// C++ naming, and translation-unit imports even when one Rust product
// configuration does not reference them. The campaign ledgers/checker, not
// Rust's reachability lints, are the completeness authority for these files.
#![allow(
    dead_code,
    irrefutable_let_patterns,
    non_upper_case_globals,
    unused_assignments,
    unused_imports,
    unused_macros,
    unused_mut,
    unused_parens,
    unused_unsafe,
    unused_variables
)]

#[doc(hidden)]
mod target_inventory;

pub(crate) mod source {
    pub(crate) mod decoders {
        pub(crate) mod include {
            pub(crate) mod rive {
                pub(crate) mod decoders {
                    pub(crate) mod astc_footprints_hpp;
                }
            }
        }
    }

    pub(crate) mod include {
        pub(crate) mod utils {
            pub(crate) mod lite_rtti_hpp;
        }

        pub(crate) mod rive {
            pub(crate) mod factory_hpp;
            pub(crate) mod gpu_texture_format_hpp;
            pub(crate) mod refcnt_hpp;
            pub(crate) mod renderer_hpp;
            pub(crate) mod rive_types_hpp;

            pub(crate) mod shapes {
                pub(crate) mod paint {
                    pub(crate) mod image_sampler_hpp;
                }
            }
        }
    }

    pub(crate) mod renderer {
        pub(crate) mod include {
            pub(crate) mod rive {
                pub(crate) mod renderer {
                    pub(crate) mod buffer_ring_hpp;
                    pub(crate) mod draw_hpp;
                    pub(crate) mod gpu_hpp;
                    pub(crate) mod render_canvas_hpp;
                    pub(crate) mod render_context_helper_impl_hpp;
                    pub(crate) mod render_context_hpp;
                    pub(crate) mod render_context_impl_hpp;
                    pub(crate) mod render_target_hpp;
                    pub(crate) mod rive_render_buffer_hpp;
                    pub(crate) mod rive_render_factory_hpp;
                    pub(crate) mod rive_render_image_hpp;
                    pub(crate) mod rive_renderer_hpp;
                    pub(crate) mod texture_hpp;

                    pub(crate) mod metal {
                        pub(crate) mod render_context_metal_impl_h;
                    }
                }
            }
        }

        pub(crate) mod src {
            pub(crate) mod draw_cpp;
            pub(crate) mod gpu_cpp;
            pub(crate) mod gradient_cpp;
            pub(crate) mod gradient_hpp;
            pub(crate) mod render_context_cpp;
            pub(crate) mod render_context_helper_impl_cpp;
            pub(crate) mod rive_render_factory_cpp;
            pub(crate) mod rive_render_image_cpp;
            pub(crate) mod rive_render_paint_cpp;
            pub(crate) mod rive_render_paint_hpp;
            pub(crate) mod rive_render_path_cpp;
            pub(crate) mod rive_render_path_hpp;
            pub(crate) mod rive_renderer_cpp;

            pub(crate) mod metal {
                pub(crate) mod background_shader_compiler_h;
                pub(crate) mod background_shader_compiler_mm;
                pub(crate) mod render_context_metal_impl_mm;
            }

            pub(crate) mod shaders {
                pub(crate) mod advanced_blend_glsl;
                pub(crate) mod atomic_draw_glsl;
                pub(crate) mod bezier_utils_glsl;
                pub(crate) mod blit_texture_as_draw_glsl;
                pub(crate) mod clear_clockwise_atomic_clip_glsl;
                pub(crate) mod color_ramp_glsl;
                pub(crate) mod common_glsl;
                pub(crate) mod constants_glsl;
                pub(crate) mod draw_clockwise_atomic_borrowed_coverage_frag;
                pub(crate) mod draw_clockwise_atomic_clip_frag;
                pub(crate) mod draw_clockwise_atomic_path_frag;
                pub(crate) mod draw_clockwise_clip_frag;
                pub(crate) mod draw_clockwise_path_frag;
                pub(crate) mod draw_fullscreen_quad_vert;
                pub(crate) mod draw_image_mesh_vert;
                pub(crate) mod draw_input_attachment_frag;
                pub(crate) mod draw_mesh_frag;
                pub(crate) mod draw_msaa_object_frag;
                pub(crate) mod draw_msaa_resolve_frag;
                pub(crate) mod draw_path_common_glsl;
                pub(crate) mod draw_path_vert;
                pub(crate) mod draw_raster_order_path_frag;
                pub(crate) mod flush_uniforms_glsl;
                pub(crate) mod glsl_glsl;
                pub(crate) mod hlsl_glsl;
                pub(crate) mod init_clockwise_atomic_workaround_frag;
                pub(crate) mod makefile;
                pub(crate) mod metal_glsl;
                pub(crate) mod minify_py;
                pub(crate) mod pls_load_store_ext_glsl;
                pub(crate) mod render_atlas_glsl;
                pub(crate) mod resolve_atlas_glsl;
                pub(crate) mod rhi_glsl;
                pub(crate) mod specialization_glsl;
                pub(crate) mod stencil_draw_glsl;
                pub(crate) mod tessellate_glsl;

                pub(crate) mod metal {
                    pub(crate) mod color_ramp_metal;
                    pub(crate) mod draw_metal;
                    pub(crate) mod generate_draw_combinations_py;
                    pub(crate) mod tessellate_metal;
                }
            }
        }
    }

    pub(crate) mod src {
        pub(crate) mod renderer_cpp;
    }
}
