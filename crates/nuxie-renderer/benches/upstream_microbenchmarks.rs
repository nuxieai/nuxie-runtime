//! Criterion mirrors of the pinned C++ renderer microbenchmarks.
//!
//! The private renderer modules are compiled directly into this bench target.
//! This exercises the shipping algorithms without widening the public API just
//! for measurement.

#![allow(dead_code, unused_imports)]

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
use nuxie_runtime::ArtboardInstance;

#[path = "../src/draw.rs"]
mod draw;
#[path = "../src/gpu.rs"]
mod gpu;
#[path = "../src/gr_triangulator.rs"]
mod gr_triangulator;
#[path = "../src/intersection_board.rs"]
mod intersection_board;

use intersection_board::{FindResult, GroupingType, IntersectionBoard, IntersectionTile, Rect};

const PAPER_BBOXES: &[u8] = include_bytes!("../../../benchmarks/data/paper_bboxes_6_copies.i32le");
const MARTY_BBOXES: &[u8] =
    include_bytes!("../../../benchmarks/data/marty_bboxes_187_copies.i32le");
const PAPER_RIV: &[u8] = include_bytes!("../../../benchmarks/data/paper.riv");

fn reset_cpp_rand() {
    // SAFETY: C's process-global PRNG accepts every unsigned seed. Criterion
    // executes these benchmark closures serially.
    unsafe { libc::srand(0) };
}

fn cpp_rand() -> i32 {
    // SAFETY: `rand` has no preconditions. Benchmark execution is serial.
    unsafe { libc::rand() }
}

fn bbox_rows(bytes: &[u8]) -> Vec<Rect> {
    bytes
        .chunks_exact(16)
        .map(|row| {
            let value = |offset| i32::from_le_bytes(row[offset..offset + 4].try_into().unwrap());
            Rect::new(value(0), value(4), value(8), value(12))
        })
        .collect()
}

fn run_intersection_tile(grouping_type: GroupingType, tile: &mut IntersectionTile) -> i16 {
    reset_cpp_rand();
    tile.reset(0, 0, 0, 0);
    let mut result = 0;
    for _ in 0..10_000 {
        let values = [cpp_rand(), cpp_rand(), cpp_rand(), cpp_rand()].map(|value| value & 0xff);
        let left = values[0].min(values[2]).min(254);
        let top = values[1].min(values[3]).min(254);
        let right = values[0].max(values[2]).min(254) + 1;
        let bottom = values[1].max(values[3]).min(254) + 1;
        let rect = Rect::new(left, top, right, bottom);
        let found =
            tile.find_max_intersecting_group_index(grouping_type, rect, FindResult::default());
        result = found.max_group_indices.into_iter().max().unwrap_or(0) + 1;
        tile.add_rectangle(grouping_type, rect, result, 0);
    }
    result
}

struct BoardWorkload {
    board: IntersectionBoard,
    boxes: Vec<Rect>,
}

impl BoardWorkload {
    fn new(boxes: &[u8]) -> Self {
        let mut workload = Self {
            board: IntersectionBoard::new(GroupingType::Disjoint),
            boxes: bbox_rows(boxes),
        };
        workload.run();
        workload
    }

    fn run(&mut self) -> i16 {
        self.board.resize_and_reset(3456, 2102);
        self.boxes.iter().fold(0, |maximum, &rect| {
            maximum.max(self.board.add_rectangle(rect, 1))
        })
    }
}

#[derive(Clone, Debug)]
struct CapturedPathPaint {
    path: RawPath,
    fill_rule: FillRule,
    style: RenderPaintStyle,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
}

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

#[derive(Clone, Copy)]
enum Preparation {
    Authored,
    Strokes(StrokeJoin),
    Feather(f32),
}

fn prepare_paths(paths: &[CapturedPathPaint], preparation: Preparation) -> usize {
    let mut result = 0usize;
    let mut scratch = draw::StrokePreparationScratch::default();
    for _ in 0..10 {
        for path in paths {
            let prepared = match preparation {
                Preparation::Authored if path.feather != 0.0 => draw::build_feather_tessellation(
                    &path.path,
                    Mat2D::IDENTITY,
                    path.feather,
                    (path.style == RenderPaintStyle::Stroke).then_some((
                        path.thickness,
                        path.join,
                        path.cap,
                    )),
                ),
                Preparation::Authored if path.style == RenderPaintStyle::Stroke => {
                    draw::build_stroke_tessellation_with_layout_using_scratch(
                        &path.path,
                        Mat2D::IDENTITY,
                        path.thickness,
                        path.join,
                        path.cap,
                        &mut scratch,
                    )
                    .map(|value| value.tessellation)
                }
                Preparation::Authored => draw::build_fill_tessellation(&path.path, Mat2D::IDENTITY),
                Preparation::Strokes(join) => {
                    draw::build_stroke_tessellation_with_layout_using_scratch(
                        &path.path,
                        Mat2D::IDENTITY,
                        2.0,
                        join,
                        StrokeCap::Butt,
                        &mut scratch,
                    )
                    .map(|value| value.tessellation)
                }
                Preparation::Feather(feather) => draw::build_feather_tessellation(
                    &path.path,
                    Mat2D::IDENTITY,
                    feather,
                    (path.style == RenderPaintStyle::Stroke).then_some((
                        path.thickness,
                        path.join,
                        path.cap,
                    )),
                ),
            };
            if let Some(prepared) = prepared {
                result = result.wrapping_add(prepared.spans.len());
            }
        }
    }
    result
}

fn renderer_benches(criterion: &mut Criterion) {
    let mut tile = IntersectionTile::default();
    run_intersection_tile(GroupingType::Disjoint, &mut tile);
    criterion.bench_function("IntersectionTileBench", |bench| {
        bench.iter(|| black_box(run_intersection_tile(GroupingType::Disjoint, &mut tile)));
    });

    let mut overlap_tile = IntersectionTile::default();
    run_intersection_tile(GroupingType::OverlapAllowed, &mut overlap_tile);
    criterion.bench_function("IntersectionTileBenchWithOverlap", |bench| {
        bench.iter(|| {
            black_box(run_intersection_tile(
                GroupingType::OverlapAllowed,
                &mut overlap_tile,
            ))
        });
    });

    let mut paper_board = BoardWorkload::new(PAPER_BBOXES);
    criterion.bench_function("IntersectionBoardBench_paper", |bench| {
        bench.iter(|| black_box(paper_board.run()));
    });

    let mut marty_board = BoardWorkload::new(MARTY_BBOXES);
    criterion.bench_function("IntersectionBoardBench_marty", |bench| {
        bench.iter(|| black_box(marty_board.run()));
    });

    let paper = paper_paths();
    criterion.bench_function("DrawRiveRenderPaths", |bench| {
        bench.iter(|| black_box(prepare_paths(&paper, Preparation::Authored)));
    });
    criterion.bench_function("DrawRiveRenderPathsAsStrokes", |bench| {
        bench.iter(|| {
            black_box(prepare_paths(
                &paper,
                Preparation::Strokes(StrokeJoin::Bevel),
            ))
        });
    });
    criterion.bench_function("DrawRiveRenderPathsAsRoundJoinStrokes", |bench| {
        bench.iter(|| {
            black_box(prepare_paths(
                &paper,
                Preparation::Strokes(StrokeJoin::Round),
            ))
        });
    });
    criterion.bench_function("DrawFeatheredPaths_paper", |bench| {
        bench.iter(|| black_box(prepare_paths(&paper, Preparation::Feather(100.0))));
    });

    let zero_chop = custom_paths(|path| {
        path.move_to(199.0, 1225.0);
        for _ in 0..50 {
            path.cubic_to(197.0, 943.0, 349.0, 607.0, 549.0, 427.0);
            path.cubic_to(349.0, 607.0, 197.0, 943.0, 199.0, 1225.0);
        }
    });
    criterion.bench_function("DrawZeroChopStrokes", |bench| {
        bench.iter(|| black_box(prepare_paths(&zero_chop, Preparation::Authored)));
    });

    let one_chop = custom_paths(|path| {
        for _ in 0..50 {
            path.cubic_to(100.0, 0.0, 50.0, 100.0, 100.0, 100.0);
            path.cubic_to(0.0, -100.0, 200.0, 100.0, 0.0, 0.0);
        }
    });
    criterion.bench_function("DrawOneChopStrokes", |bench| {
        bench.iter(|| black_box(prepare_paths(&one_chop, Preparation::Authored)));
    });

    let two_chop = custom_paths(|path| {
        path.move_to(460.0, 1060.0);
        for _ in 0..50 {
            path.cubic_to(403.0, -320.0, 60.0, 660.0, 1181.0, 634.0);
            path.cubic_to(60.0, 660.0, 403.0, -320.0, 460.0, 1060.0);
        }
    });
    criterion.bench_function("DrawTwoChopStrokes", |bench| {
        bench.iter(|| black_box(prepare_paths(&two_chop, Preparation::Authored)));
    });

    let one_cusp = custom_paths(|path| {
        for _ in 0..50 {
            path.cubic_to(100.0, 100.0, 100.0, 0.0, 0.0, 100.0);
            path.cubic_to(100.0, 0.0, 100.0, 100.0, 0.0, 0.0);
        }
    });
    criterion.bench_function("DrawOneCuspStrokes", |bench| {
        bench.iter(|| black_box(prepare_paths(&one_cusp, Preparation::Authored)));
    });

    let two_cusp = custom_paths(|path| {
        for _ in 0..50 {
            path.cubic_to(100.0, 0.0, 50.0, 0.0, 150.0, 0.0);
            path.cubic_to(50.0, 0.0, 100.0, 0.0, 0.0, 0.0);
        }
    });
    criterion.bench_function("DrawTwoCuspStrokes", |bench| {
        bench.iter(|| black_box(prepare_paths(&two_cusp, Preparation::Authored)));
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
    criterion.bench_function("DrawCustomFeathers", |bench| {
        bench.iter(|| black_box(prepare_paths(&custom_feathers, Preparation::Authored)));
    });
}

criterion_group!(benches, renderer_benches);
criterion_main!(benches);
