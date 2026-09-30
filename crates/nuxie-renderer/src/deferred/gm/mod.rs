//! The eight deferred GM source owners added by upstream e949498e.
//! These use the live Metal backend, not a command-only stand-in. The paired
//! GMs use gmmain.cpp's non-atomic comparison (zero channel difference).
mod additive_advanced_blend;
mod additive_blend;
mod bitmap_cache_pixel;
mod layer_mask_pixel;
mod clipstrokes;
mod image_paint;
mod mesh_instanced;
mod ore_deferred_context;
mod ore_deferred_multipass;
mod ore_deferred_replay;
mod ore_deferred_resource;
mod ore_depth_write_always;
mod ore_depth_sample;
mod ore_mip_render_target;
mod ore_buffer_offsets;
mod ore_pipeline_switch;
mod ore_vertex_buffer_before_pipeline;
mod ore_gm_helper;
mod ore_layout_intern;
mod ore_nested_pass;
mod ore_render_deferred_canvas;
mod render_canvas;
mod render_canvas_dag;
mod render_deferred_2d;
mod runtime_deferred_import;
mod serialized_replay_2d;
mod strokes;
#[cfg(feature = "with-rive-tools")]
mod uber_gm_helper;
#[cfg(feature = "with-rive-tools")]
mod uber_parity;
#[cfg(feature = "with-rive-tools")]
mod uber_parity_srcover;
