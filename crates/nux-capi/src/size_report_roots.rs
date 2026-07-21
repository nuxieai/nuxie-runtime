//! Measurement-only full renderer consumer.
//!
//! Fat LTO removes renderer methods that the portable C ABI does not call.
//! `tools/size-report.sh` links this one hidden root to model a Rust host that
//! consumes every public `WgpuFactory` / `WgpuFrame` entry point plus the
//! Darwin presentation surface. The report verifies the `root!` inventory
//! against both renderer source files and the committed audited inventory
//! before accepting a measurement.

use nuxie::{
    ApplePresentationCompletion, AppleSurface, BlendMode, Factory, FillRule, GpuCanvasPlan,
    GpuCanvasShader, GpuCanvasShaderStage, ImageFilter,
    ImageSampler, ImageWrap, Mat2D, RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType,
    RenderImage, RenderMode, RenderPaint, RenderPath, Renderer, WgpuFactory, WgpuFrame,
};
use std::ffi::c_void;
use std::future::Future;
use std::hint::black_box;
use std::ptr;
use std::task::{Context, Poll, Waker};

/// Opaque inputs make every consumer call observable to LTO. This type never
/// crosses the shipped C API and the retained root is never executed.
struct RootArgs {
    factory: *mut WgpuFactory,
    frame: *mut WgpuFrame,
    surface: *mut AppleSurface,
    completion: *mut ApplePresentationCompletion,
    drawable: *mut c_void,
    path: *const Box<dyn RenderPath>,
    paint: *const Box<dyn RenderPaint>,
    image: *const Box<dyn RenderImage>,
    vertices: *const Box<dyn RenderBuffer>,
    uv_coords: *const Box<dyn RenderBuffer>,
    indices: *const Box<dyn RenderBuffer>,
    bytes: *const u8,
    byte_len: usize,
    colors: *const u32,
    color_len: usize,
    stops: *const f32,
    stop_len: usize,
    width: u32,
    height: u32,
    count: u32,
    scalar: f32,
}

macro_rules! root {
    ($name:literal, $body:block) => {{
        black_box($name);
        $body
    }};
}

fn poll_once(future: impl Future) {
    let mut future = std::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let result = Future::poll(future.as_mut(), &mut context);
    black_box(matches!(result, Poll::Ready(_)));
}

fn render_mode(selector: u32) -> RenderMode {
    if selector & 1 == 0 {
        RenderMode::Msaa
    } else {
        RenderMode::ClockwiseAtomic
    }
}

fn buffer_type(selector: u32) -> RenderBufferType {
    if selector & 1 == 0 {
        RenderBufferType::Index
    } else {
        RenderBufferType::Vertex
    }
}

fn buffer_flags(selector: u32) -> RenderBufferFlags {
    if selector & 1 == 0 {
        RenderBufferFlags::None
    } else {
        RenderBufferFlags::MappedOnceAtInitialization
    }
}

fn fill_rule(selector: u32) -> FillRule {
    match selector % 3 {
        0 => FillRule::NonZero,
        1 => FillRule::EvenOdd,
        _ => FillRule::Clockwise,
    }
}

fn blend_mode(selector: u32) -> BlendMode {
    if selector & 1 == 0 {
        BlendMode::SrcOver
    } else {
        BlendMode::Screen
    }
}

fn sampler(selector: u32) -> ImageSampler {
    ImageSampler {
        wrap_x: if selector & 1 == 0 {
            ImageWrap::Clamp
        } else {
            ImageWrap::Repeat
        },
        wrap_y: if selector & 2 == 0 {
            ImageWrap::Clamp
        } else {
            ImageWrap::Mirror
        },
        filter: if selector & 4 == 0 {
            ImageFilter::Bilinear
        } else {
            ImageFilter::Nearest
        },
    }
}

unsafe fn boxed_ref<'a, T: ?Sized>(pointer: *const Box<T>) -> Option<&'a T> {
    unsafe { pointer.as_ref().map(Box::as_ref) }
}

/// Linker root for the exact public renderer inventory.
///
/// The name deliberately does not begin with `nux_`, so it cannot enter the
/// shipped C export inventory. `size-report.sh` retains it by its exact symbol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __nuxie_size_report_renderer_roots(
    selector: u32,
    opaque_args: *mut c_void,
) -> usize {
    let Some(args) = (unsafe { opaque_args.cast::<RootArgs>().as_mut() }) else {
        return 0;
    };
    let bytes = if args.bytes.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args.bytes, args.byte_len) }
    };
    let colors = if args.colors.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args.colors, args.color_len) }
    };
    let stops = if args.stops.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args.stops, args.stop_len) }
    };

    match selector % 43 {
        0 => root!("inherent WgpuFactory::validate_image_bytes", {
            black_box(WgpuFactory::validate_image_bytes(bytes).is_ok());
            0
        }),
        1 => root!("inherent WgpuFactory::new", {
            black_box(WgpuFactory::new(args.width, args.height).is_ok());
            0
        }),
        2 => root!("inherent WgpuFactory::new_with_mode", {
            black_box(
                WgpuFactory::new_with_mode(args.width, args.height, render_mode(selector)).is_ok(),
            );
            0
        }),
        3 => root!("inherent WgpuFactory::new_async", {
            poll_once(WgpuFactory::new_async(args.width, args.height));
            0
        }),
        4 => root!("inherent WgpuFactory::new_async_with_mode", {
            poll_once(WgpuFactory::new_async_with_mode(
                args.width,
                args.height,
                render_mode(selector),
            ));
            0
        }),
        5 => root!("inherent WgpuFactory::begin_frame", {
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(factory.begin_frame(args.count));
            0
        }),
        6 => root!("inherent WgpuFactory::begin_frame_for_benchmark", {
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(factory.begin_frame_for_benchmark(args.count, selector & 1 != 0));
            0
        }),
        7 => root!("inherent WgpuFactory::adapter_info", {
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(factory.adapter_info());
            0
        }),
        8 => root!("inherent WgpuFactory::dimensions", {
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(factory.dimensions());
            0
        }),
        9 => root!("inherent WgpuFactory::device_is_lost", {
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(factory.device_is_lost());
            0
        }),
        10 => root!("inherent WgpuFactory::resize", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(factory.resize(args.width, args.height).is_ok());
            0
        }),
        11 => root!("inherent WgpuFactory::new_session_factory", {
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(
                factory
                    .new_session_factory(args.width, args.height, render_mode(selector))
                    .is_ok(),
            );
            0
        }),
        12 => root!("inherent WgpuFrame::finish", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            black_box(unsafe { ptr::read(frame) }.finish().is_ok());
            0
        }),
        13 => root!("inherent WgpuFrame::finish_async", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            poll_once(unsafe { ptr::read(frame) }.finish_async());
            0
        }),
        14 => root!("inherent WgpuFrame::finish_for_benchmark", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            black_box(unsafe { ptr::read(frame) }.finish_for_benchmark().is_ok());
            0
        }),
        15 => root!("inherent WgpuFrame::finish_for_benchmark_async", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            poll_once(unsafe { ptr::read(frame) }.finish_for_benchmark_async());
            0
        }),
        16 => root!("trait Factory::make_render_buffer", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(Factory::make_render_buffer(
                factory,
                buffer_type(selector),
                buffer_flags(selector),
                args.byte_len,
            ));
            0
        }),
        17 => root!("trait Factory::make_linear_gradient", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(Factory::make_linear_gradient(
                factory,
                args.scalar,
                args.scalar,
                args.scalar,
                args.scalar,
                colors,
                stops,
            ));
            0
        }),
        18 => root!("trait Factory::make_radial_gradient", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(Factory::make_radial_gradient(
                factory,
                args.scalar,
                args.scalar,
                args.scalar,
                colors,
                stops,
            ));
            0
        }),
        19 => root!("trait Factory::make_render_path", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(Factory::make_render_path(
                factory,
                RawPath::new(),
                fill_rule(selector),
            ));
            0
        }),
        20 => root!("trait Factory::make_empty_render_path", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(Factory::make_empty_render_path(factory));
            0
        }),
        21 => root!("trait Factory::make_render_paint", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(Factory::make_render_paint(factory));
            0
        }),
        22 => root!("trait Factory::decode_image", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(Factory::decode_image(factory, bytes).is_ok());
            0
        }),
        23 => root!("trait Factory::make_gpu_canvas_image", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            let stage = GpuCanvasShaderStage {
                source: String::new(),
                logical_entry_point: String::new(),
                physical_entry_point: String::new(),
            };
            let shader = GpuCanvasShader {
                vertex: stage.clone(),
                fragment: stage,
            };
            let plan = GpuCanvasPlan {
                width: args.width,
                height: args.height,
                clear_color: [0.0; 4],
                vertex_count: args.count,
                instance_count: 1,
                first_vertex: 0,
                first_instance: 0,
                uniform_buffers: Vec::new(),
                vertex_layouts: Vec::new(),
                vertex_buffers: Vec::new(),
            };
            black_box(Factory::make_gpu_canvas_image(factory, &shader, &plan).is_ok());
            0
        }),
        24 => root!("trait Renderer::save", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            Renderer::save(frame);
            0
        }),
        25 => root!("trait Renderer::restore", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            Renderer::restore(frame);
            0
        }),
        26 => root!("trait Renderer::transform", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            Renderer::transform(frame, Mat2D([args.scalar; 6]));
            0
        }),
        27 => root!("trait Renderer::draw_path", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            let Some(path) = (unsafe { boxed_ref(args.path) }) else {
                return 0;
            };
            let Some(paint) = (unsafe { boxed_ref(args.paint) }) else {
                return 0;
            };
            Renderer::draw_path(frame, path, paint);
            0
        }),
        28 => root!("trait Renderer::clip_path", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            let Some(path) = (unsafe { boxed_ref(args.path) }) else {
                return 0;
            };
            Renderer::clip_path(frame, path);
            0
        }),
        29 => root!("trait Renderer::draw_image", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            Renderer::draw_image(
                frame,
                unsafe { boxed_ref(args.image) },
                sampler(selector),
                blend_mode(selector),
                args.scalar,
            );
            0
        }),
        30 => root!("trait Renderer::draw_image_mesh", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            Renderer::draw_image_mesh(
                frame,
                unsafe { boxed_ref(args.image) },
                sampler(selector),
                unsafe { boxed_ref(args.vertices) },
                unsafe { boxed_ref(args.uv_coords) },
                unsafe { boxed_ref(args.indices) },
                args.count,
                args.count,
                blend_mode(selector),
                args.scalar,
            );
            0
        }),
        31 => root!("trait Renderer::modulate_opacity", {
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            Renderer::modulate_opacity(frame, args.scalar);
            0
        }),
        32 => root!("inherent ApplePresentationCompletion::new", {
            black_box(ApplePresentationCompletion::new(|| {}));
            0
        }),
        33 => root!("inherent AppleSurface::attach_with_factory", {
            black_box(
                AppleSurface::attach_with_factory(args.width, args.height, render_mode(selector))
                    .is_ok(),
            );
            0
        }),
        34 => root!("inherent AppleSurface::attach", {
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(AppleSurface::attach(factory, args.width, args.height).is_ok());
            0
        }),
        35 => root!("inherent AppleSurface::dimensions", {
            let Some(surface) = (unsafe { args.surface.as_ref() }) else {
                return 0;
            };
            black_box(surface.dimensions());
            0
        }),
        36 => root!("inherent AppleSurface::is_attached", {
            let Some(surface) = (unsafe { args.surface.as_ref() }) else {
                return 0;
            };
            black_box(surface.is_attached());
            0
        }),
        37 => root!("inherent AppleSurface::resize", {
            let Some(surface) = (unsafe { args.surface.as_mut() }) else {
                return 0;
            };
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(surface.resize(factory, args.width, args.height).is_ok());
            0
        }),
        38 => root!("inherent AppleSurface::detach", {
            let Some(surface) = (unsafe { args.surface.as_mut() }) else {
                return 0;
            };
            surface.detach();
            0
        }),
        39 => root!("inherent AppleSurface::reattach", {
            let Some(surface) = (unsafe { args.surface.as_mut() }) else {
                return 0;
            };
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            black_box(surface.reattach(factory, args.width, args.height).is_ok());
            0
        }),
        40 => root!("inherent AppleSurface::copy_metal_device", {
            let Some(surface) = (unsafe { args.surface.as_ref() }) else {
                return 0;
            };
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(surface.copy_metal_device(factory).is_ok());
            0
        }),
        41 => root!("inherent AppleSurface::preflight_present", {
            let Some(surface) = (unsafe { args.surface.as_ref() }) else {
                return 0;
            };
            let Some(factory) = (unsafe { args.factory.as_ref() }) else {
                return 0;
            };
            black_box(
                surface
                    .preflight_present(factory, selector & 1 != 0)
                    .is_ok(),
            );
            0
        }),
        _ => root!("inherent AppleSurface::present", {
            let Some(surface) = (unsafe { args.surface.as_mut() }) else {
                return 0;
            };
            let Some(factory) = (unsafe { args.factory.as_mut() }) else {
                return 0;
            };
            let Some(frame) = (unsafe { args.frame.as_mut() }) else {
                return 0;
            };
            let completion = unsafe {
                args.completion
                    .as_mut()
                    .map(|completion| ptr::read(completion))
            };
            black_box(
                unsafe { surface.present(factory, ptr::read(frame), args.drawable, completion) }
                    .is_ok(),
            );
            0
        }),
    }
}
