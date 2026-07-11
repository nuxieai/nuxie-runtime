//! Experimental Rust wrapper for the C++ Rive Renderer.
//!
//! This crate is the M1 production-renderer seam. By default it is inert so
//! `cargo test --workspace` stays independent from local C++ renderer builds.
//! Enable the `native` feature after building `librive_pls_renderer.a` in the
//! reference runtime to exercise the real C++ `RiveRenderer`.

#[cfg(feature = "native")]
mod native;

#[cfg(feature = "native")]
pub use native::{FfiFactory, FfiFrame, FfiRenderMode, NativeRendererError};

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

        assert_eq!(frame.end(), 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn metal_context_produces_non_empty_pixels() {
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
}
