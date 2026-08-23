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
#[cfg(all(
    feature = "native-metal-experimental",
    any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "visionos"
    )
))]
mod target_inventory;

#[doc(hidden)]
mod backend_shader_authority_inventory;

#[path = "mechanical_port/shader-build-authority/renderer_premake5_pls_renderer_lua__build_input.rs"]
pub(crate) mod backend_port_build_pls_renderer;

#[path = "mechanical_port/shader-build-authority/renderer_premake5_lua__build_input.rs"]
pub(crate) mod backend_port_build_renderer;

#[path = "mechanical_port/shader-build-authority/renderer_make_dawn_sh__build_input.rs"]
pub(crate) mod backend_port_dependency_dawn;

#[path = "mechanical_port/shader-build-authority/renderer_make_moltenvk_sh__build_input.rs"]
pub(crate) mod backend_port_dependency_moltenvk;

#[path = "mechanical_port/shader-build-authority/renderer_make_swiftshader_sh__build_input.rs"]
pub(crate) mod backend_port_dependency_swiftshader;

#[path = "mechanical_port/shader-build-authority/renderer_src_shaders_makefile__build_input.rs"]
pub(crate) mod backend_port_shader_build_graph;

#[path = "mechanical_port/shader-build-authority/renderer_src_shaders_minify_py__generator.rs"]
pub(crate) mod backend_port_shader_minifier;

#[path = "mechanical_port/shader-build-authority/renderer_src_shaders_spirv_binary_to_header_py__generator.rs"]
pub(crate) mod backend_port_spirv_header_generator;

#[path = "mechanical_port/shader-build-authority/renderer_src_shaders_wgsl_to_header_py__generator.rs"]
pub(crate) mod backend_port_wgsl_header_generator;

#[cfg(feature = "native-webgpu-experimental")]
pub(crate) mod webgpu {
    #[path = "renderer_src_ore_wgpu_ore_bind_group_layout_wgpu_hpp__decl.rs"]
    pub(crate) mod ore_bind_group_layout_wgpu_decl;
    #[path = "renderer_src_webgpu_wagyu_port_include_webgpu_webgpu_h__decl.rs"]
    pub(crate) mod webgpu_decl;
    #[path = "renderer_src_webgpu_wagyu_port_include_webgpu_webgpu_wagyu_h__decl.rs"]
    pub(crate) mod webgpu_wagyu_decl;
    #[path = "renderer_src_webgpu_wagyu_port_src_webgpu_c__impl.rs"]
    pub(crate) mod webgpu_impl;
    #[path = "renderer_src_webgpu_wagyu_port_include_webgpu_webgpu_cpp_chained_struct_h__decl.rs"]
    pub(crate) mod webgpu_cpp_chained_struct_decl;
    #[path = "renderer_src_webgpu_wagyu_port_include_webgpu_webgpu_cpp_h__decl.rs"]
    pub(crate) mod webgpu_cpp_decl;
    #[path = "renderer_src_webgpu_wagyu_port_include_webgpu_webgpu_enum_class_bitmasks_h__decl.rs"]
    pub(crate) mod webgpu_enum_class_bitmasks_decl;
    #[path = "renderer_src_webgpu_webgpu_compat_h__decl.rs"]
    pub(crate) mod webgpu_compat_decl;
    #[path = "renderer_src_webgpu_wagyu_port_src_library_webgpu_stubs_js__compat_build_input.rs"]
    pub(crate) mod library_webgpu_stubs_build_input;
    #[path = "renderer_src_webgpu_wagyu_port_src_library_webgpu_wagyu_stubs_js__compat_build_input.rs"]
    pub(crate) mod library_webgpu_wagyu_stubs_build_input;
    #[path = "renderer_src_webgpu_wagyu_port_webgpu_port_py__generator.rs"]
    pub(crate) mod webgpu_port_generator;
}

#[cfg(feature = "native-vulkan-experimental")]
pub(crate) mod vulkan {
    #[path = "renderer_src_vulkan_common_layouts_hpp__decl.rs"]
    pub(crate) mod common_layouts_decl;
    #[path = "renderer_src_vulkan_draw_pipeline_layout_vulkan_hpp__decl.rs"]
    pub(crate) mod draw_pipeline_layout_vulkan_decl;
    #[path = "renderer_src_vulkan_draw_pipeline_layout_vulkan_cpp__impl.rs"]
    pub(crate) mod draw_pipeline_layout_vulkan_impl;
    #[path = "renderer_src_vulkan_draw_pipeline_vulkan_hpp__decl.rs"]
    pub(crate) mod draw_pipeline_vulkan_decl;
    #[path = "renderer_src_vulkan_draw_pipeline_vulkan_cpp__impl.rs"]
    pub(crate) mod draw_pipeline_vulkan_impl;
    #[path = "renderer_src_vulkan_draw_shader_vulkan_hpp__decl.rs"]
    pub(crate) mod draw_shader_vulkan_decl;
    #[path = "renderer_src_vulkan_draw_shader_vulkan_cpp__impl.rs"]
    pub(crate) mod draw_shader_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_bind_group_layout_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_bind_group_layout_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_bind_group_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_bind_group_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_bind_group_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_bind_group_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_buffer_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_buffer_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_buffer_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_buffer_vulkan_impl;
    #[path = "renderer_include_rive_renderer_ore_ore_context_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_context_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_context_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_context_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_pipeline_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_pipeline_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_pipeline_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_pipeline_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_render_pass_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_render_pass_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_render_pass_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_render_pass_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_sampler_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_sampler_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_sampler_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_sampler_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_shader_module_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_shader_module_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_shader_module_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_shader_module_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_texture_vulkan_hpp__decl.rs"]
    pub(crate) mod ore_texture_vulkan_decl;
    #[path = "renderer_src_ore_vulkan_ore_texture_vulkan_cpp__impl.rs"]
    pub(crate) mod ore_texture_vulkan_impl;
    #[path = "renderer_src_ore_vulkan_ore_vulkan_dsl_hpp__decl.rs"]
    pub(crate) mod ore_vulkan_dsl;
    #[path = "renderer_src_vulkan_pipeline_manager_vulkan_hpp__decl.rs"]
    pub(crate) mod pipeline_manager_vulkan_decl;
    #[path = "renderer_src_vulkan_pipeline_manager_vulkan_cpp__impl.rs"]
    pub(crate) mod pipeline_manager_vulkan_impl;
    #[path = "renderer_include_rive_renderer_vulkan_render_context_vulkan_impl_hpp__decl.rs"]
    pub(crate) mod render_context_vulkan_decl;
    #[path = "renderer_src_vulkan_render_context_vulkan_impl_cpp__impl.rs"]
    pub(crate) mod render_context_vulkan_impl;
    #[path = "renderer_src_vulkan_render_pass_vulkan_hpp__decl.rs"]
    pub(crate) mod render_pass_vulkan_decl;
    #[path = "renderer_src_vulkan_render_pass_vulkan_cpp__impl.rs"]
    pub(crate) mod render_pass_vulkan_impl;
    #[path = "renderer_include_rive_renderer_vulkan_render_target_vulkan_hpp__decl.rs"]
    pub(crate) mod render_target_vulkan_decl;
    #[path = "renderer_src_vulkan_render_target_vulkan_cpp__impl.rs"]
    pub(crate) mod render_target_vulkan_impl;
    #[path = "renderer_include_rive_renderer_vulkan_vkutil_hpp__decl.rs"]
    pub(crate) mod vkutil_decl;
    #[path = "renderer_src_vulkan_vkutil_cpp__impl.rs"]
    pub(crate) mod vkutil_impl;
    #[path = "renderer_include_rive_renderer_vulkan_vulkan_context_hpp__decl.rs"]
    pub(crate) mod vulkan_context_decl;
    #[path = "renderer_src_vulkan_vulkan_context_cpp__impl.rs"]
    pub(crate) mod vulkan_context_impl;
    #[path = "renderer_src_vulkan_vulkan_memory_allocator_cpp__impl.rs"]
    pub(crate) mod vulkan_memory_allocator_impl;
    #[path = "renderer_src_vulkan_vulkan_shaders_hpp__decl.rs"]
    pub(crate) mod vulkan_shaders_decl;
    #[path = "renderer_src_vulkan_vulkan_shaders_cpp__impl.rs"]
    pub(crate) mod vulkan_shaders_impl;
}

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

                    #[cfg(all(
                        feature = "native-metal-experimental",
                        any(
                            target_os = "ios",
                            target_os = "macos",
                            target_os = "tvos",
                            target_os = "visionos"
                        )
                    ))]
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

            #[cfg(all(
                feature = "native-metal-experimental",
                any(
                    target_os = "ios",
                    target_os = "macos",
                    target_os = "tvos",
                    target_os = "visionos"
                )
            ))]
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

                #[cfg(all(
                    feature = "native-metal-experimental",
                    any(
                        target_os = "ios",
                        target_os = "macos",
                        target_os = "tvos",
                        target_os = "visionos"
                    )
                ))]
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
