//! Mechanical Rust port of Rive's ORE interface and Metal adapter.
//!
//! The source correspondence and translation queue are pinned in
//! `docs/metal-port-manifest.toml`. This crate deliberately remains separate
//! from the built-in renderer-platform implementation.

pub mod binding_map;
pub mod gpu_resource;
pub mod rstb_entry_container;
pub mod types;
