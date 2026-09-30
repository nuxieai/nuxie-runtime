//! `tests/unit_tests/runtime/serialized_replay_test.cpp` through 3d0d3f56.
use nuxie_render_api::serialized_replay::{SerializedReplayHooks, replay_serialized_commands};
use nuxie_render_api::*;
use std::{cell::RefCell, rc::Rc};

#[test]
fn serialized_2d_commands_replay_byte_identically() {
    let mut a = SerializingFactory::default();
    a.frame_size(256, 256);
    a.add_frame();
    let mut renderer_a = a.make_renderer();
    let mut paint = a.make_render_paint();
    paint.color(0xff112233);
    paint.style(RenderPaintStyle::Stroke);
    paint.thickness(3.5);
    paint.join(StrokeJoin::Round);
    paint.cap(StrokeCap::Square);
    paint.blend_mode(BlendMode::Multiply);
    paint.feather(2.0);
    paint.additiveness(0.375);

    let mut rp = RawPath::new();
    rp.move_to(0.0, 0.0);
    rp.line_to(10.0, 0.0);
    rp.cubic_to(10.0, 5.0, 5.0, 10.0, 0.0, 10.0);
    rp.close();
    let path = a.make_render_path(rp, FillRule::EvenOdd);
    let mut clip = a.make_empty_render_path();
    let mut cp = RawPath::new();
    cp.move_to(0.0, 0.0);
    cp.line_to(20.0, 0.0);
    cp.line_to(20.0, 20.0);
    cp.close();
    clip.add_raw_path(&cp);
    let grad = a.make_linear_gradient(
        0.0,
        0.0,
        100.0,
        100.0,
        &[0xffff0000, 0xff0000ff],
        &[0.0, 1.0],
    );
    let mut paint2 = a.make_render_paint();
    paint2.shader(Some(grad.as_ref()));
    let image_path = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/open_source.jpg");
    let image_bytes = std::fs::read(&image_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", image_path.display()));
    let image = a.decode_image(&image_bytes).expect("modulating image");
    let sampler = ImageSampler {
        filter: ImageFilter::Nearest,
        wrap_x: ImageWrap::Repeat,
        wrap_y: ImageWrap::Mirror,
    };
    paint2.modulated_image(
        Some(image.as_ref()),
        sampler,
        Mat2D([2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
    );
    let mut paint3 = a.make_render_paint();
    paint3.modulated_image(Some(image.as_ref()), sampler, Mat2D::IDENTITY);
    paint3.modulated_image(None, ImageSampler::LINEAR_CLAMP, Mat2D::IDENTITY);
    renderer_a.save();
    renderer_a.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 5.0, 7.0]));
    renderer_a.clip_path(clip.as_ref());
    renderer_a.modulate_opacity(0.5);
    renderer_a.draw_path(path.as_ref(), paint.as_ref());
    renderer_a.draw_path(path.as_ref(), paint2.as_ref());
    renderer_a.draw_path(path.as_ref(), paint3.as_ref());
    renderer_a.draw_image(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
    renderer_a.draw_image_with_additiveness(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
        0.625,
    );

    let verts = [0.0f32, 0.0, 10.0, 0.0, 10.0, 10.0, 0.0, 10.0];
    let uvs = [0.0f32, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
    let indices = [0u16, 1, 2, 0, 2, 3];
    let mut vertex_buffer = a.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::None,
        std::mem::size_of_val(&verts),
    );
    let mut uv_buffer = a.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::None,
        std::mem::size_of_val(&uvs),
    );
    let mut index_buffer = a.make_render_buffer(
        RenderBufferType::Index,
        RenderBufferFlags::None,
        std::mem::size_of_val(&indices),
    );
    vertex_buffer.map_mut().copy_from_slice(
        &verts
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect::<Vec<_>>(),
    );
    vertex_buffer.unmap();
    uv_buffer
        .map_mut()
        .copy_from_slice(&uvs.iter().flat_map(|v| v.to_ne_bytes()).collect::<Vec<_>>());
    uv_buffer.unmap();
    index_buffer.map_mut().copy_from_slice(
        &indices
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect::<Vec<_>>(),
    );
    index_buffer.unmap();
    renderer_a.draw_image_mesh(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        Some(vertex_buffer.as_ref()),
        Some(uv_buffer.as_ref()),
        Some(index_buffer.as_ref()),
        4,
        6,
        BlendMode::SrcOver,
        1.0,
    );
    renderer_a.draw_image_mesh_with_additiveness(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        Some(vertex_buffer.as_ref()),
        Some(uv_buffer.as_ref()),
        Some(index_buffer.as_ref()),
        4,
        6,
        BlendMode::SrcOver,
        1.0,
        0.625,
    );
    renderer_a.restore();

    let mut b = PersistentFactory::new(SerializingFactory::default());
    let mut renderer_b = b.borrow().make_renderer();
    let frame_factory = b.clone();
    let size_factory = b.clone();
    let mut hooks = SerializedReplayHooks {
        on_frame: Some(Box::new(move || frame_factory.borrow_mut().add_frame())),
        on_frame_size: Some(Box::new(move |w, h| {
            size_factory.borrow_mut().frame_size(w, h)
        })),
        ..Default::default()
    };
    assert!(replay_serialized_commands(
        &a.bytes(),
        &mut b,
        &mut renderer_b,
        &mut hooks
    ));
    let sa = a.bytes();
    let b = b.borrow();
    let sb = b.bytes();
    assert_eq!(sa.len(), sb.len());
    assert_eq!(&*sa, &*sb);
}

// An unbacked RenderCanvas has an image identity before GPU allocation, just
// like the upstream tests' directly constructed gpu::RenderCanvas.
#[derive(Clone)]
struct CanvasImage {
    identity: Rc<()>,
    width: u32,
    height: u32,
}
impl RenderImage for CanvasImage {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.identity) as usize
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}
struct Canvas(CanvasImage);
impl RenderCanvas for Canvas {
    fn width(&self) -> u32 {
        self.0.width
    }
    fn height(&self) -> u32 {
        self.0.height
    }
    fn is_backed(&self) -> bool {
        false
    }
    fn render_image(&self) -> Rc<dyn RenderImage> {
        self.0.retain_image()
    }
    fn begin_frame(
        &mut self,
        _: ColorInt,
    ) -> Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
        Err(RenderCanvasError::unsupported())
    }
}
fn canvas(width: u32, height: u32) -> RenderCanvasHandle {
    Rc::new(RefCell::new(Box::new(Canvas(CanvasImage {
        identity: Rc::new(()),
        width,
        height,
    }))))
}

// Supplemental coverage of the explicit nullable branches in the source API.
#[test]
fn serializing_canvas_null_inputs_and_cache_disable_preserve_source_behavior() {
    let context = PersistentFactory::new(NullFactory::new());
    let mut factory = SerializingFactory::new();
    factory.enable_bitmap_cache(context.persistent_context());
    assert!(factory.render_context().is_some());
    assert!(factory.deferred_canvas_host().is_some());
    assert!(factory.canvas_content_host().is_some());
    factory.enable_bitmap_cache(None);
    assert!(factory.render_context().is_none());
    assert!(factory.deferred_canvas_host().is_none());
    assert!(factory.canvas_content_host().is_none());
    assert!(DeferredCanvasHost::make_content_canvas(&mut factory, 8, 8).is_none());
    let before = factory.bytes().to_vec();
    assert!(factory.begin_canvas_content(None, 0).is_none());
    factory.end_canvas_content(None);
    assert!(factory.content_canvas_image(None).is_none());
    assert_eq!(&*factory.bytes(), before.as_slice());
}

#[derive(Clone, Default)]
struct CountingRenderer(Rc<RefCell<(usize, usize)>>);
impl Renderer for CountingRenderer {
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {
        self.0.borrow_mut().0 += 1;
    }
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {
        self.0.borrow_mut().1 += 1;
    }
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}
fn record_cached_stream(factory: &mut SerializingFactory, screen: &mut dyn Renderer) {
    let mut paint = factory.make_render_paint();
    paint.color(0xff445566);
    let mut raw = RawPath::new();
    raw.move_to(0.0, 0.0);
    raw.line_to(10.0, 0.0);
    raw.line_to(10.0, 10.0);
    raw.close();
    let path = factory.make_render_path(raw, FillRule::NonZero);
    screen.draw_path(path.as_ref(), paint.as_ref());
    let canvas = canvas(64, 48);
    let mut content = factory
        .begin_canvas_content(Some(canvas.clone()), 0xff204060)
        .unwrap();
    content.save();
    content.scale(2.0, 2.0);
    content.draw_path(path.as_ref(), paint.as_ref());
    content.restore();
    factory.end_canvas_content(Some(&canvas));
    screen.save();
    screen.draw_image(
        SerializingFactory::content_canvas_image(factory, Some(&canvas)).as_deref(),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
    screen.restore();
}
#[test]
fn serialized_canvas_content_replays_byte_identically() {
    let mut a = SerializingFactory::default();
    a.frame_size(256, 256);
    a.add_frame();
    let mut screen = a.make_renderer();
    record_cached_stream(&mut a, &mut screen);
    let mut b = PersistentFactory::new(SerializingFactory::default());
    let mut screen = b.borrow().make_renderer();
    let frame = b.clone();
    let size = b.clone();
    let begin = b.clone();
    let end = b.clone();
    let canvases = Rc::new(RefCell::new(Vec::<RenderCanvasHandle>::new()));
    let begin_canvases = canvases.clone();
    let end_canvases = canvases.clone();
    let counts = Rc::new(RefCell::new((0, 0)));
    let begin_counts = counts.clone();
    let end_counts = counts.clone();
    let mut hooks = SerializedReplayHooks {
        on_frame: Some(Box::new(move || frame.borrow_mut().add_frame())),
        on_frame_size: Some(Box::new(move |w, h| size.borrow_mut().frame_size(w, h))),
        on_canvas_content_begin: Some(Box::new(move |_, w, h, clear, image| {
            begin_counts.borrow_mut().0 += 1;
            let canvas = canvas(w, h);
            let renderer = begin
                .borrow_mut()
                .begin_canvas_content(Some(canvas.clone()), clear);
            *image = begin.borrow().content_canvas_image(Some(&canvas));
            begin_canvases.borrow_mut().push(canvas);
            renderer
        })),
        on_canvas_content_end: Some(Box::new(move |_| {
            end_counts.borrow_mut().1 += 1;
            end.borrow_mut()
                .end_canvas_content(Some(end_canvases.borrow().last().unwrap()));
        })),
    };
    assert!(replay_serialized_commands(
        &a.bytes(),
        &mut b,
        &mut screen,
        &mut hooks
    ));
    assert_eq!(*counts.borrow(), (1, 1));
    assert_eq!(canvases.borrow().len(), 1);
    assert_eq!(canvases.borrow()[0].borrow().width(), 64);
    assert_eq!(canvases.borrow()[0].borrow().height(), 48);
    assert_eq!(&*a.bytes(), &*b.borrow().bytes());
}
#[test]
fn serialized_canvas_content_without_target_is_dropped() {
    let mut a = SerializingFactory::default();
    a.frame_size(256, 256);
    a.add_frame();
    let mut screen = a.make_renderer();
    record_cached_stream(&mut a, &mut screen);
    let mut b = SerializingFactory::default();
    let mut screen = CountingRenderer::default();
    assert!(replay_serialized_commands(
        &a.bytes(),
        &mut b,
        &mut screen,
        &mut SerializedReplayHooks::default()
    ));
    assert_eq!(*screen.0.borrow(), (1, 0));
}
fn malformed_canvas_stream(wrong_end: bool) -> Vec<u8> {
    let mut a = SerializingFactory::default();
    a.frame_size(256, 256);
    a.add_frame();
    let paint = a.make_render_paint();
    let mut raw = RawPath::new();
    raw.move_to(0.0, 0.0);
    raw.line_to(10.0, 10.0);
    let path = a.make_render_path(raw, FillRule::NonZero);
    let other = canvas(4, 4);
    if wrong_end {
        a.begin_canvas_content(Some(other.clone()), 0)
            .unwrap()
            .draw_path(path.as_ref(), paint.as_ref());
        a.end_canvas_content(Some(&other));
    }
    a.begin_canvas_content(Some(canvas(8, 8)), 0)
        .unwrap()
        .draw_path(path.as_ref(), paint.as_ref());
    if wrong_end {
        a.end_canvas_content(Some(&other));
    }
    a.bytes().to_vec()
}
#[test]
fn serialized_canvas_content_rejects_wrong_end() {
    let mut b = SerializingFactory::default();
    let mut screen = CountingRenderer::default();
    let proxy = screen.clone();
    let ends = Rc::new(RefCell::new(0));
    let end_count = ends.clone();
    let mut hooks = SerializedReplayHooks {
        on_canvas_content_begin: Some(Box::new(move |_, _, _, _, _| Some(Box::new(proxy.clone())))),
        on_canvas_content_end: Some(Box::new(move |_| *end_count.borrow_mut() += 1)),
        ..Default::default()
    };
    assert!(!replay_serialized_commands(
        &malformed_canvas_stream(true),
        &mut b,
        &mut screen,
        &mut hooks
    ));
    assert_eq!(*ends.borrow(), 1);
}
#[test]
fn serialized_canvas_content_rejects_unclosed_content() {
    let mut b = SerializingFactory::default();
    let mut screen = CountingRenderer::default();
    let proxy = screen.clone();
    let mut hooks = SerializedReplayHooks {
        on_canvas_content_begin: Some(Box::new(move |_, _, _, _, _| Some(Box::new(proxy.clone())))),
        ..Default::default()
    };
    assert!(!replay_serialized_commands(
        &malformed_canvas_stream(false),
        &mut b,
        &mut screen,
        &mut hooks
    ));
}

#[test]
fn serialized_replay_rejects_a_bad_header() {
    let garbage = [b'X', b'X', b'X', b'X', 1, 0, 0, 0];
    let mut b = SerializingFactory::default();
    let mut renderer = b.make_renderer();
    assert!(!replay_serialized_commands(
        &garbage,
        &mut b,
        &mut renderer,
        &mut SerializedReplayHooks::default()
    ));
}
