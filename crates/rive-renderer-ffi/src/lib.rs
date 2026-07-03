//! Experimental Rust wrapper for the C++ Rive Renderer.
//!
//! This crate is the M1 production-renderer seam. By default it is inert so
//! `cargo test --workspace` stays independent from local C++ renderer builds.
//! Enable the `native` feature after building `librive_pls_renderer.a` in the
//! reference runtime to exercise the real C++ `RiveRenderer`.

#[cfg(feature = "native")]
mod native;

#[cfg(feature = "native")]
pub use native::{FfiFactory, FfiFrame, NativeRendererError};

/// Describes the state of the native bridge in default workspace builds.
pub const NATIVE_FEATURE_STATUS: &str =
    "enable the `native` feature and build the C++ PLS renderer to use rive-renderer-ffi";

#[cfg(test)]
mod tests {
    use super::NATIVE_FEATURE_STATUS;

    #[test]
    fn default_build_keeps_native_renderer_optional() {
        assert!(NATIVE_FEATURE_STATUS.contains("native"));
    }
}
