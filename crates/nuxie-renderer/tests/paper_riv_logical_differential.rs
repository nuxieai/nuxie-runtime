//! Differential coverage for the four `paper.riv` workloads in the pinned
//! upstream `draw_pls_path.cpp` microbenchmark.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, NullFactory,
    RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, Renderer, StrokeCap, StrokeJoin,
};
use nuxie_renderer::{
    LogicalFrameConfig, LogicalFrameReport, LogicalPathPaint, NullLogicalRenderer, RenderMode,
    WgpuFactory,
};
use nuxie_runtime::ArtboardInstance;
use sha2::{Digest, Sha256};

const PAPER_RIV: &[u8] = include_bytes!("../../../benchmarks/data/paper.riv");
const PAPER_RIV_LEN: usize = 2_403_906;
const PAPER_PATH_COUNT: usize = 3_861;
const PAPER_RIV_SHA256: &str = "79b9fef67b397ad0eb5895a7857d38ca9b7e1ea51bde8ad9a50964ab674c4ee0";

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

fn capture_paper_paths() -> Vec<CapturedPathPaint> {
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

#[derive(Clone, Copy, Debug)]
enum PaperWorkload {
    Authored,
    BevelStrokes,
    RoundJoinStrokes,
    Feathered,
}

impl PaperWorkload {
    const ALL: [Self; 4] = [
        Self::Authored,
        Self::BevelStrokes,
        Self::RoundJoinStrokes,
        Self::Feathered,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Authored => "DrawRiveRenderPaths",
            Self::BevelStrokes => "DrawRiveRenderPathsAsStrokes",
            Self::RoundJoinStrokes => "DrawRiveRenderPathsAsRoundJoinStrokes",
            Self::Feathered => "DrawFeatheredPaths_paper",
        }
    }

    // Exact SniffPathsRenderer mutations from rive-runtime
    // tests/bench/draw_pls_path.cpp at 4ac7b32798da0482e441ef09304dc3b480ed3ee5.
    fn prepare(self, authored: &[CapturedPathPaint]) -> Vec<CapturedPathPaint> {
        authored
            .iter()
            .cloned()
            .map(|mut draw| {
                match self {
                    Self::Authored => {}
                    Self::BevelStrokes | Self::RoundJoinStrokes => {
                        draw.style = RenderPaintStyle::Stroke;
                        draw.thickness = 2.0;
                        draw.join = if matches!(self, Self::RoundJoinStrokes) {
                            StrokeJoin::Round
                        } else {
                            StrokeJoin::Bevel
                        };
                    }
                    Self::Feathered => {
                        draw.feather = 100.0;
                        draw.fill_rule = FillRule::Clockwise;
                    }
                }
                draw
            })
            .collect()
    }
}

fn logical_paint(draw: &CapturedPathPaint) -> LogicalPathPaint {
    LogicalPathPaint {
        style: draw.style,
        thickness: draw.thickness,
        join: draw.join,
        cap: draw.cap,
        feather: draw.feather,
        ..LogicalPathPaint::default()
    }
}

fn null_report(
    renderer: &mut NullLogicalRenderer,
    draws: &[CapturedPathPaint],
) -> LogicalFrameReport {
    let paths = draws
        .iter()
        .map(|draw| renderer.prepare_path(&draw.path, draw.fill_rule))
        .collect::<Vec<_>>();
    renderer.begin_frame();
    for (draw, path) in draws.iter().zip(&paths) {
        renderer
            .draw_path(path, logical_paint(draw))
            .expect("paper draw is supported by Null");
    }
    renderer
        .flush_with_diagnostics()
        .expect("paper Null frame flushes")
}

fn wgpu_report(factory: &mut WgpuFactory, draws: &[CapturedPathPaint]) -> LogicalFrameReport {
    let resources = draws
        .iter()
        .map(|draw| {
            let path = factory.make_render_path(draw.path.clone(), draw.fill_rule);
            let mut paint = factory.make_render_paint();
            paint.style(draw.style);
            paint.thickness(draw.thickness);
            paint.join(draw.join);
            paint.cap(draw.cap);
            paint.feather(draw.feather);
            (path, paint)
        })
        .collect::<Vec<_>>();
    let mut frame = factory.begin_frame(0);
    for (path, paint) in &resources {
        frame.draw_path(path.as_ref(), paint.as_ref());
    }
    frame
        .finish_logical_frame_for_differential()
        .expect("paper Wgpu logical frame finishes its production CPU boundary")
}

fn assert_report_is_meaningful(
    workload: PaperWorkload,
    mode: RenderMode,
    report: &LogicalFrameReport,
) {
    assert!(
        report.draw_count > 0,
        "{} had no admitted draws",
        workload.name()
    );
    assert_eq!(
        report.resource_planning_passes,
        report.draw_count,
        "{} did not plan each admitted draw exactly once in {mode:?}",
        workload.name()
    );
    assert_eq!(
        report.plan_finalization_passes,
        1,
        "{} finalized more than once in {mode:?}",
        workload.name()
    );
    assert!(
        report.written.path_records > 0 && report.written.paint_records > 0,
        "{} wrote no path/paint resources in {mode:?}",
        workload.name()
    );
    assert!(
        report.written_bytes > 0 && report.shadow_fingerprint != 0,
        "{} produced empty diagnostic resources in {mode:?}",
        workload.name()
    );
    assert!(
        report.production_typed_output_consumed,
        "{} did not feed shared typed output to the production encoder in {mode:?}",
        workload.name()
    );
}

#[test]
fn pinned_paper_workloads_match_between_wgpu_and_null_logical_frames() {
    assert_eq!(PAPER_RIV.len(), PAPER_RIV_LEN, "paper.riv size drifted");
    assert_eq!(
        format!("{:x}", Sha256::digest(PAPER_RIV)),
        PAPER_RIV_SHA256,
        "paper.riv provenance drifted"
    );

    let authored = capture_paper_paths();
    assert_eq!(
        authored.len(),
        PAPER_PATH_COUNT,
        "paper path cardinality drifted"
    );
    assert!(
        authored
            .iter()
            .any(|draw| draw.style == RenderPaintStyle::Fill),
        "paper lost authored fills"
    );
    assert!(
        authored
            .iter()
            .any(|draw| draw.style == RenderPaintStyle::Stroke),
        "paper lost authored strokes"
    );

    for workload in PaperWorkload::ALL {
        let draws = workload.prepare(&authored);
        assert_eq!(
            draws.len(),
            authored.len(),
            "{} cardinality drifted",
            workload.name()
        );
        match workload {
            PaperWorkload::Authored => assert_eq!(draws.len(), authored.len()),
            PaperWorkload::BevelStrokes => assert!(draws.iter().all(|draw| {
                draw.style == RenderPaintStyle::Stroke
                    && draw.thickness == 2.0
                    && draw.join == StrokeJoin::Bevel
            })),
            PaperWorkload::RoundJoinStrokes => assert!(draws.iter().all(|draw| {
                draw.style == RenderPaintStyle::Stroke
                    && draw.thickness == 2.0
                    && draw.join == StrokeJoin::Round
            })),
            PaperWorkload::Feathered => assert!(draws
                .iter()
                .all(|draw| draw.feather == 100.0 && draw.fill_rule == FillRule::Clockwise)),
        }
    }

    for mode in [RenderMode::ClockwiseAtomic, RenderMode::Msaa] {
        let config = LogicalFrameConfig {
            width: 1_600,
            height: 1_600,
            mode,
            max_texture_dimension_2d: 8_192,
            msaa_atlas_supports_clip_rect: true,
        };
        let mut factory = WgpuFactory::new_with_mode(config.width, config.height, mode)
            .expect("test WebGPU adapter is available");
        let mut null_renderer = NullLogicalRenderer::new(config);
        for workload in PaperWorkload::ALL {
            let draws = workload.prepare(&authored);
            let wgpu = wgpu_report(&mut factory, &draws);
            let null = null_report(&mut null_renderer, &draws);
            assert_report_is_meaningful(workload, mode, &wgpu);
            assert_eq!(null, wgpu, "{} diverged in {mode:?}", workload.name());
        }
    }
}
