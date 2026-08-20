#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_render_api::{Factory, FillRule, RawPath, Renderer};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use nuxie_renderer::NativeMetalFactory;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::rc::autoreleasepool;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_core_foundation::CGSize;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_metal::MTLPixelFormat;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_quartz_core::CAMetalLayer;

#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe extern "C" {
    fn pthread_main_np() -> std::ffi::c_int;
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn require_caller_drawable<T>(drawable: Option<T>) -> Result<T, &'static str> {
    drawable.ok_or("caller-owned CAMetalLayer returned no drawable")
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn main() {
    autoreleasepool(|_| {
        // SAFETY: `pthread_main_np` takes no arguments and only reports
        // whether this process entry point is executing on Darwin's main
        // thread; it does not retain or dereference Rust storage.
        assert_eq!(unsafe { pthread_main_np() }, 1);
        let mut factory = NativeMetalFactory::new(64, 64).expect("create native Metal tracer");
        let mut path = RawPath::new();
        path.move_to(8.0, 8.0);
        path.line_to(56.0, 8.0);
        path.line_to(56.0, 56.0);
        path.line_to(8.0, 56.0);
        path.close();
        let path = factory.make_render_path(path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.color(0xff00_ff00);

        // Root deterministic readback and its pixel oracle in the same final
        // executable that roots the caller-owned product presentation path.
        let mut frame = factory
            .begin_frame(0)
            .expect("acquire native Metal tracer command buffer");
        frame.draw_path(path.as_ref(), paint.as_ref());
        let pixels = frame.finish().expect("finish native Metal tracer");
        assert_eq!(pixels[(32 * 64 + 32) * 4..][..4], [0, 255, 0, 255]);

        factory
            .resize(32, 32)
            .expect("resize native Metal product target");
        let layer = CAMetalLayer::new();
        let device = factory.retained_metal_device();
        layer.setDevice(Some(&device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setFramebufferOnly(true);
        layer.setDrawableSize(CGSize::new(32.0, 32.0));
        layer.setMaximumDrawableCount(2);
        layer.setAllowsNextDrawableTimeout(true);

        let mut product_path = RawPath::new();
        product_path.move_to(4.0, 4.0);
        product_path.line_to(28.0, 4.0);
        product_path.line_to(28.0, 28.0);
        product_path.line_to(4.0, 28.0);
        product_path.close();
        let product_path = factory.make_render_path(product_path, FillRule::NonZero);

        assert_eq!(
            require_caller_drawable(None::<()>),
            Err("caller-owned CAMetalLayer returned no drawable")
        );
        let drawable = require_caller_drawable(layer.nextDrawable())
            .expect("the product-surface root requires a drawable");
        let mut product_frame = factory
            .begin_drawable_frame(&drawable, 0)
            .expect("explicitly select native Metal for the Apple drawable");
        product_frame.draw_path(product_path.as_ref(), paint.as_ref());
        product_frame
            .finish()
            .expect("finish and present native Metal product frame");
        drop(drawable);
        drop(
            layer
                .nextDrawable()
                .expect("completed product frame must leave the layer reusable"),
        );
    });
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn main() {
    panic!("native Metal product root requires an Apple target");
}
