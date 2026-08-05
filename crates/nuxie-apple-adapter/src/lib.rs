//! Apple drawable-presentation adapter.
//!
//! The adapter exposes only the existing high-level surface lifecycle. It has
//! no direct dependency on Objective-C, Metal, `wgpu`, device, queue, surface,
//! or texture types. UNIV-1626 moves the implementation and image-admission
//! policy here; `nuxie-renderer` keeps temporary compatibility exports until
//! then.

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use nuxie_renderer::{
    ApplePresentationCompletion, AppleSurface, SurfaceDisposition, SurfaceError,
};

#[cfg(all(test, any(target_os = "ios", target_os = "macos")))]
mod tests {
    use super::AppleSurface;

    #[test]
    fn apple_contract_keeps_legacy_type_identity() {
        fn accepts_legacy(value: nuxie_renderer::AppleSurface) -> AppleSurface {
            value
        }

        let _ = accepts_legacy;
    }
}
