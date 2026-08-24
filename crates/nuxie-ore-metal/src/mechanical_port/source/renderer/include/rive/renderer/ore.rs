// Both exact headers expose a source-local `raw_abi` adapter module. C++ keeps
// those nested under their declaring types; this generated namespace
// aggregation intentionally leaves neither ambiguous spelling authoritative.
#![allow(ambiguous_glob_reexports)]

#[path = "ore/ore_bind_group_hpp.rs"]
pub mod ore_bind_group_hpp;
#[path = "ore/ore_bind_group_layout_hpp.rs"]
pub mod ore_bind_group_layout_hpp;
#[path = "ore/ore_binding_map_hpp.rs"]
pub mod ore_binding_map_hpp;
#[path = "ore/ore_buffer_hpp.rs"]
pub mod ore_buffer_hpp;
#[path = "ore/ore_context_hpp.rs"]
pub mod ore_context_hpp;
#[path = "ore/ore_context_metal_hpp.rs"]
#[cfg(target_vendor = "apple")]
pub mod ore_context_metal_hpp;
#[path = "ore/ore_pipeline_hpp.rs"]
pub mod ore_pipeline_hpp;
#[path = "ore/ore_render_pass_hpp.rs"]
pub mod ore_render_pass_hpp;
#[path = "ore/ore_rstb_entry_container_hpp.rs"]
pub mod ore_rstb_entry_container_hpp;
#[path = "ore/ore_sampler_hpp.rs"]
pub mod ore_sampler_hpp;
#[path = "ore/ore_shader_module_hpp.rs"]
pub mod ore_shader_module_hpp;
#[path = "ore/ore_texture_hpp.rs"]
pub mod ore_texture_hpp;
#[path = "ore/ore_types_hpp.rs"]
pub mod ore_types_hpp;

pub use ore_bind_group_hpp::*;
pub use ore_bind_group_layout_hpp::*;
pub use ore_binding_map_hpp::*;
pub use ore_buffer_hpp::*;
pub use ore_context_hpp::*;
#[cfg(target_vendor = "apple")]
pub use ore_context_metal_hpp::*;
pub use ore_pipeline_hpp::*;
pub use ore_render_pass_hpp::*;
pub use ore_rstb_entry_container_hpp::*;
pub use ore_sampler_hpp::*;
pub use ore_shader_module_hpp::*;
pub use ore_texture_hpp::*;
pub use ore_types_hpp::*;
