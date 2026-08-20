// Mechanical translation of:
// - renderer/include/rive/renderer/ore/ore_sampler.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

//! Backend-neutral ORE sampler resource.
//!
//! The C++ source makes `Sampler` a `GPUResource` base class. Rust represents
//! that base exactly once in [`crate::gpu_resource::ResourceHandle`], so the
//! sampler payload itself is intentionally empty.

/// The portable portion of `rive::ore::Sampler`.
///
/// There is intentionally no sampler descriptor or backend operation here.
/// Those belong to [`crate::types::Sampler`] and the concrete Metal owner.
pub struct Sampler;

impl Sampler {
    /// Translate the C++ constructor that creates an unmanaged resource.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_payload_has_no_duplicate_resource_owner() {
        let _sampler = Sampler::new();
        assert_eq!(std::mem::size_of::<Sampler>(), 0);
    }
}
