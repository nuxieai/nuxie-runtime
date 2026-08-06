//! Criterion mirrors of the pinned C++ renderer microbenchmarks.
//!
//! The opt-in support seam calls production-compiled algorithms without
//! bringing `cfg(test)` instrumentation into the timed binary.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::rc::Rc;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, NullFactory,
    RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, Renderer, StrokeCap, StrokeJoin,
};
use nuxie_renderer::upstream_microbenchmarks::{
    CapturedPathPaint, IntersectionBoardWorkload, IntersectionTileWorkload, NullFrameWorkload,
    Preparation,
};
use nuxie_runtime::ArtboardInstance;

const PAPER_BBOXES: &[u8] = include_bytes!("../../../benchmarks/data/paper_bboxes_6_copies.i32le");
const MARTY_BBOXES: &[u8] =
    include_bytes!("../../../benchmarks/data/marty_bboxes_187_copies.i32le");
const PAPER_RIV: &[u8] = include_bytes!("../../../benchmarks/data/paper.riv");

struct CapturePath {
    path: RawPath,
    fill_rule: FillRule,
}

impl RenderPath for CapturePath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.path.rewind();
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = path.as_any().downcast_ref::<Self>().expect("capture path");
        self.path.add_path(&path.path, transform);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = path.as_any().downcast_ref::<Self>().expect("capture path");
        self.path.add_path_backwards(&path.path, transform);
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.path.add_path(path, Mat2D::IDENTITY);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.path.cubic_to(ox, oy, ix, iy, x, y);
    }

    fn close(&mut self) {
        self.path.close();
    }
}

struct CapturePaint {
    style: RenderPaintStyle,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
}

impl Default for CapturePaint {
    fn default() -> Self {
        Self {
            style: RenderPaintStyle::Fill,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
        }
    }
}

impl RenderPaint for CapturePaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.style = style;
    }

    fn color(&mut self, _value: ColorInt) {}

    fn thickness(&mut self, value: f32) {
        self.thickness = value.abs();
    }

    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    fn feather(&mut self, value: f32) {
        self.feather = value.abs();
    }

    fn blend_mode(&mut self, _value: BlendMode) {}
    fn shader(&mut self, _shader: Option<&dyn RenderShader>) {}
    fn invalidate_stroke(&mut self) {}
}

struct CaptureFactory {
    null: NullFactory,
}

impl Default for CaptureFactory {
    fn default() -> Self {
        Self {
            null: NullFactory::new(),
        }
    }
}

impl Factory for CaptureFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.null
            .make_render_buffer(buffer_type, flags, size_in_bytes)
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.null
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.null
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        Box::new(CapturePath { path, fill_rule })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(CapturePath {
            path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(CapturePaint::default())
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.null.decode_image(data)
    }
}

struct CaptureRenderer {
    output: Rc<RefCell<Vec<CapturedPathPaint>>>,
}

impl Renderer for CaptureRenderer {
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _transform: Mat2D) {}

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let path = path
            .as_any()
            .downcast_ref::<CapturePath>()
            .expect("capture path");
        let paint = paint
            .as_any()
            .downcast_ref::<CapturePaint>()
            .expect("capture paint");
        self.output.borrow_mut().push(CapturedPathPaint {
            path: path.path.clone(),
            fill_rule: path.fill_rule,
            style: paint.style,
            thickness: paint.thickness,
            join: paint.join,
            cap: paint.cap,
            feather: paint.feather,
        });
    }

    fn clip_path(&mut self, _path: &dyn RenderPath) {}
    fn draw_image(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
    }
    fn draw_image_mesh(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _vertices: Option<&dyn RenderBuffer>,
        _uv_coords: Option<&dyn RenderBuffer>,
        _indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _opacity: f32) {}
}

fn paper_paths() -> Vec<CapturedPathPaint> {
    let runtime = read_runtime_file(PAPER_RIV).expect("pinned paper.riv imports");
    let graph = GraphFile::from_runtime_file(&runtime).expect("paper graph builds");
    let artboard = graph.artboards.first().expect("paper has an artboard");
    let mut instance =
        ArtboardInstance::from_graph_with_artboards(&runtime, artboard, &graph.artboards)
            .expect("paper artboard instantiates");
    instance
        .advance_frame_components(0.0)
        .expect("paper static scene advances");
    instance
        .update_pass_with_script_errors()
        .expect("paper settles");
    let mut factory = CaptureFactory::default();
    let output = Rc::new(RefCell::new(Vec::new()));
    let mut renderer = CaptureRenderer {
        output: Rc::clone(&output),
    };
    let external_images = BTreeMap::<u32, Arc<[u8]>>::new();
    instance
        .synchronize_artboard_renderer(
            &runtime,
            artboard,
            &graph.artboards,
            &external_images,
            &mut factory,
            None,
        )
        .expect("paper renderer synchronizes");
    instance
        .draw_artboard(
            &runtime,
            artboard,
            &graph.artboards,
            &mut factory,
            &mut renderer,
            &external_images,
            None,
            true,
        )
        .expect("paper draws into capture renderer");
    drop(renderer);
    Rc::try_unwrap(output)
        .expect("capture renderer released")
        .into_inner()
}

fn custom_paths(mut setup: impl FnMut(&mut RawPath)) -> Vec<CapturedPathPaint> {
    (0..1000)
        .map(|_| {
            let mut path = RawPath::new();
            setup(&mut path);
            CapturedPathPaint {
                path,
                fill_rule: FillRule::Clockwise,
                style: RenderPaintStyle::Stroke,
                thickness: 2.0,
                join: StrokeJoin::Miter,
                cap: StrokeCap::Butt,
                feather: 0.0,
            }
        })
        .collect()
}

fn renderer_benches(criterion: &mut Criterion) {
    let mut tile = IntersectionTileWorkload::disjoint();
    criterion.bench_function("IntersectionTileBench", |bench| {
        bench.iter(|| black_box(tile.run()));
    });

    let mut overlap_tile = IntersectionTileWorkload::overlap_allowed();
    criterion.bench_function("IntersectionTileBenchWithOverlap", |bench| {
        bench.iter(|| black_box(overlap_tile.run()));
    });

    let mut paper_board = IntersectionBoardWorkload::from_i32le(PAPER_BBOXES);
    criterion.bench_function("IntersectionBoardBench_paper", |bench| {
        bench.iter(|| black_box(paper_board.run()));
    });

    let mut marty_board = IntersectionBoardWorkload::from_i32le(MARTY_BBOXES);
    criterion.bench_function("IntersectionBoardBench_marty", |bench| {
        bench.iter(|| black_box(marty_board.run()));
    });

    let paper = paper_paths();
    let mut authored_paper = NullFrameWorkload::new(paper.clone(), Preparation::Authored);
    criterion.bench_function("DrawRiveRenderPaths", |bench| {
        bench.iter(|| black_box(authored_paper.run()));
    });
    let mut stroke_paper =
        NullFrameWorkload::new(paper.clone(), Preparation::Strokes(StrokeJoin::Bevel));
    criterion.bench_function("DrawRiveRenderPathsAsStrokes", |bench| {
        bench.iter(|| black_box(stroke_paper.run()));
    });
    let mut round_stroke_paper =
        NullFrameWorkload::new(paper.clone(), Preparation::Strokes(StrokeJoin::Round));
    criterion.bench_function("DrawRiveRenderPathsAsRoundJoinStrokes", |bench| {
        bench.iter(|| black_box(round_stroke_paper.run()));
    });
    let mut feathered_paper = NullFrameWorkload::new(paper, Preparation::Feather(100.0));
    criterion.bench_function("DrawFeatheredPaths_paper", |bench| {
        bench.iter(|| black_box(feathered_paper.run()));
    });

    let zero_chop = custom_paths(|path| {
        path.move_to(199.0, 1225.0);
        for _ in 0..50 {
            path.cubic_to(197.0, 943.0, 349.0, 607.0, 549.0, 427.0);
            path.cubic_to(349.0, 607.0, 197.0, 943.0, 199.0, 1225.0);
        }
    });
    let mut zero_chop = NullFrameWorkload::new(zero_chop, Preparation::Authored);
    criterion.bench_function("DrawZeroChopStrokes", |bench| {
        bench.iter(|| black_box(zero_chop.run()));
    });

    let one_chop = custom_paths(|path| {
        for _ in 0..50 {
            path.cubic_to(100.0, 0.0, 50.0, 100.0, 100.0, 100.0);
            path.cubic_to(0.0, -100.0, 200.0, 100.0, 0.0, 0.0);
        }
    });
    let mut one_chop = NullFrameWorkload::new(one_chop, Preparation::Authored);
    criterion.bench_function("DrawOneChopStrokes", |bench| {
        bench.iter(|| black_box(one_chop.run()));
    });

    let two_chop = custom_paths(|path| {
        path.move_to(460.0, 1060.0);
        for _ in 0..50 {
            path.cubic_to(403.0, -320.0, 60.0, 660.0, 1181.0, 634.0);
            path.cubic_to(60.0, 660.0, 403.0, -320.0, 460.0, 1060.0);
        }
    });
    let mut two_chop = NullFrameWorkload::new(two_chop, Preparation::Authored);
    criterion.bench_function("DrawTwoChopStrokes", |bench| {
        bench.iter(|| black_box(two_chop.run()));
    });

    let one_cusp = custom_paths(|path| {
        for _ in 0..50 {
            path.cubic_to(100.0, 100.0, 100.0, 0.0, 0.0, 100.0);
            path.cubic_to(100.0, 0.0, 100.0, 100.0, 0.0, 0.0);
        }
    });
    let mut one_cusp = NullFrameWorkload::new(one_cusp, Preparation::Authored);
    criterion.bench_function("DrawOneCuspStrokes", |bench| {
        bench.iter(|| black_box(one_cusp.run()));
    });

    let two_cusp = custom_paths(|path| {
        for _ in 0..50 {
            path.cubic_to(100.0, 0.0, 50.0, 0.0, 150.0, 0.0);
            path.cubic_to(50.0, 0.0, 100.0, 0.0, 0.0, 0.0);
        }
    });
    let mut two_cusp = NullFrameWorkload::new(two_cusp, Preparation::Authored);
    criterion.bench_function("DrawTwoCuspStrokes", |bench| {
        bench.iter(|| black_box(two_cusp.run()));
    });

    let mut custom_feathers = custom_paths(|path| {
        for _ in 0..50 {
            path.cubic_to(-800.0, 1600.0, 2400.0, 1600.0, 1600.0, 0.0);
        }
    });
    for path in &mut custom_feathers {
        path.style = RenderPaintStyle::Fill;
        path.feather = 85.0;
    }
    let mut custom_feathers = NullFrameWorkload::new(custom_feathers, Preparation::Authored);
    criterion.bench_function("DrawCustomFeathers", |bench| {
        bench.iter(|| black_box(custom_feathers.run()));
    });
}

criterion_group!(benches, renderer_benches);
criterion_main!(benches);
