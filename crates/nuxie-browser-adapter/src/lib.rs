//! Browser-canvas presentation adapter.
//!
//! This package owns canvas attachment, resize ordering, bounded recovery,
//! direct presentation, and browser frame leases. It deliberately exposes no
//! `wgpu` device, queue, surface, or texture state.

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(any(test, target_arch = "wasm32"))]
mod browser_surface_lifecycle;

#[cfg(target_arch = "wasm32")]
pub use browser::{BrowserFactory, BrowserFrame, BrowserResizeError};
