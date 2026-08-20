//! Mechanical Rust port of Rive's ORE interface and Metal adapter.
//!
//! The source correspondence and translation queue are pinned in
//! `docs/metal-port-manifest.toml`. This crate deliberately remains separate
//! from the built-in renderer-platform implementation.

pub mod bind_group_layout;
pub mod binding_map;
pub mod gpu_resource;
pub mod metal;
pub mod rstb_entry_container;
pub mod sampler;
pub mod shader_module;
pub mod texture;
pub mod types;
