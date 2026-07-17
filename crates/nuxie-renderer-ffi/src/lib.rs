//! Experimental Rust wrapper for the C++ Rive Renderer.
//!
//! This crate is the M1 production-renderer seam. By default it is inert so
//! `cargo test --workspace` stays independent from local C++ renderer builds.
//! Enable the `native` feature after building `librive_pls_renderer.a` in the
//! reference runtime to exercise the real C++ `RiveRenderer`.

#[cfg(feature = "native")]
mod native;

#[cfg(feature = "native")]
pub use native::{FfiFactory, FfiFrame, FfiFrameMetrics, FfiRenderMode, NativeRendererError};

#[cfg(all(feature = "native", target_os = "macos"))]
pub use native::{MetalAdapterIdentity, metal_adapter_identity};

#[cfg(feature = "decode-oracle")]
#[doc(hidden)]
pub use native::{DecodedBitmapRgba, decode_bitmap_rgba};

/// Describes the state of the native bridge in default workspace builds.
pub const NATIVE_FEATURE_STATUS: &str =
    "enable the `native` feature and build the C++ PLS renderer to use nuxie-renderer-ffi";

#[cfg(test)]
mod tests {
    use super::NATIVE_FEATURE_STATUS;

    #[test]
    fn default_build_keeps_native_renderer_optional() {
        assert!(NATIVE_FEATURE_STATUS.contains("native"));
    }
}

#[cfg(all(test, feature = "native"))]
mod native_tests {
    use super::FfiFactory;
    use nuxie_render_api::{Factory, FillRule, RawPath, RenderPaintStyle, Renderer};

    #[test]
    fn null_context_counts_drawn_path() {
        let mut factory = FfiFactory::new_null(64, 64).expect("native context");
        let mut raw_path = RawPath::new();
        raw_path.move_to(4.0, 4.0);
        raw_path.line_to(60.0, 4.0);
        raw_path.line_to(60.0, 60.0);
        raw_path.close();
        let path = factory.make_render_path(raw_path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.style(RenderPaintStyle::Fill);
        paint.color(0xff00ff00);

        let mut frame = factory.begin_frame(0x00000000).expect("native frame");
        frame.draw_path(path.as_ref(), paint.as_ref());

        let metrics = frame.end_with_metrics().expect("complete native frame");
        assert_eq!(metrics.draw_calls, 1);
        assert_eq!(metrics.logical_flushes, 1);
        assert_eq!(metrics.atomic_strategy_partitions, 0);
    }

    #[test]
    #[should_panic(expected = "rive_ffi_decode_image returned null")]
    fn invalid_image_bytes_are_not_wrapped_as_a_renderable_image() {
        let mut factory = FfiFactory::new_null(64, 64).expect("native context");
        drop(factory.decode_image(b"not an encoded image"));
    }

    #[test]
    fn null_context_exposes_clockwise_atomic_strategy_count() {
        let mut factory = FfiFactory::new_null(64, 64).expect("native context");
        let frame = factory
            .begin_frame_with_mode(0x00000000, super::FfiRenderMode::ClockwiseAtomic)
            .expect("native frame");
        let metrics = frame.end_with_metrics().expect("complete native frame");
        assert_eq!(metrics.logical_flushes, 1);
        assert_eq!(metrics.atomic_strategy_partitions, 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn metal_context_produces_non_empty_pixels() {
        let identity = super::metal_adapter_identity().expect("Metal adapter identity");
        assert!(!identity.name.is_empty());
        assert!(!identity.vendor.is_empty());
        assert!(!identity.device.is_empty());
        assert!(!identity.driver.is_empty());
        let Ok(mut factory) = FfiFactory::new_metal(64, 64) else {
            eprintln!("skipping Metal pixel test because the native Metal context is unavailable");
            return;
        };
        let mut raw_path = RawPath::new();
        raw_path.move_to(4.0, 4.0);
        raw_path.line_to(60.0, 4.0);
        raw_path.line_to(60.0, 60.0);
        raw_path.close();
        let path = factory.make_render_path(raw_path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.style(RenderPaintStyle::Fill);
        paint.color(0xff00ff00);

        let mut frame = factory.begin_frame(0x00000000).expect("native frame");
        frame.draw_path(path.as_ref(), paint.as_ref());
        assert_eq!(frame.end(), 1);

        let pixels = factory.read_pixels().expect("Metal pixel readback");
        assert!(pixels.iter().any(|byte| *byte != 0));
    }

    #[cfg(all(feature = "dawn", target_os = "macos"))]
    #[test]
    fn dawn_context_produces_non_empty_pixels() {
        let mut factory = FfiFactory::new_dawn(64, 64).expect("Dawn context");
        let mut raw_path = RawPath::new();
        raw_path.move_to(4.0, 4.0);
        raw_path.line_to(60.0, 4.0);
        raw_path.line_to(60.0, 60.0);
        raw_path.close();
        let path = factory.make_render_path(raw_path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.style(RenderPaintStyle::Fill);
        paint.color(0xff00ff00);

        let mut frame = factory.begin_frame(0x00000000).expect("Dawn frame");
        frame.draw_path(path.as_ref(), paint.as_ref());
        assert_eq!(frame.end(), 1);

        let pixels = factory.read_pixels().expect("Dawn pixel readback");
        assert!(pixels.iter().any(|byte| *byte != 0));
    }
}
