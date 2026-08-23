//! Mechanical Rust port of Rive's ORE interface and Metal adapter.
//!
//! The source correspondence and translation queue are pinned in
//! `docs/metal-port-manifest.toml`. This crate deliberately remains separate
//! from the built-in renderer-platform implementation.

#[cfg(test)]
pub(crate) fn live_metal_test_unavailable(context: &str) {
    if std::env::var_os("NUXIE_REQUIRE_LIVE_METAL_TESTS").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        panic!("required live Metal test resource is unavailable: {context}");
    }
}

pub mod mechanical_port;

pub mod bind_group {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_hpp::*;
}
pub mod bind_group_layout {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_layout_hpp::*;
}
pub mod binding_map {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_binding_map_hpp::*;
}
pub mod buffer {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_buffer_hpp::*;
}
pub mod context {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_context_hpp::*;
}
pub mod gpu_resource {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::*;
}
pub mod metal;
pub mod pipeline {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_pipeline_hpp::*;
}
pub mod render_pass {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_render_pass_hpp::*;
}
pub mod rstb_entry_container {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_rstb_entry_container_hpp::*;
}
pub mod sampler {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_sampler_hpp::*;
}
pub mod shader_module {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_shader_module_hpp::*;
}
pub mod texture {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_texture_hpp::*;
}
pub mod types {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::*;
}
