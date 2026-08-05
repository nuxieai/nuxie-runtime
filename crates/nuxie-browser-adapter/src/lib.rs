//! Browser-canvas presentation adapter.
//!
//! The adapter exposes only the existing high-level browser factory/frame
//! interface. It deliberately has no direct dependency on `wgpu`, `web-sys`,
//! device, queue, surface, or texture types. UNIV-1625 moves the implementation
//! here; `nuxie-renderer` keeps temporary compatibility exports until then.

#[cfg(target_arch = "wasm32")]
pub use nuxie_renderer::{BrowserFactory, BrowserFrame, BrowserResizeError};

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::BrowserFrame;

    #[test]
    fn browser_contract_keeps_legacy_type_identity() {
        fn accepts_legacy(value: nuxie_renderer::BrowserFrame) -> BrowserFrame {
            value
        }

        let _ = accepts_legacy;
    }
}
