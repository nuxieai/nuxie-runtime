//! tests/unit_tests/renderer/deferred_measure_test.cpp at e949498e.
//! Upstream hidden measurement cases; no thresholds are added.
use super::super::{
    command_stream::CommandReader,
    deferred_replayer::*,
    deferred_session::DeferredSession,
    render_command_buffer::RenderCommandBuffer,
    render_commands::*,
    render_replay::{replay_render_commands, ReplayHooks, ResourceTable},
};
use super::*;
use std::{path::PathBuf, time::Instant};
fn env_int(name: &str, fallback: i32) -> i32 {
    std::env::var(name)
        .ok()
        .filter(|v| !v.is_empty())
        .map_or(fallback, |v| {
            let v = v.trim_start();
            let sign = if v.starts_with('-') { -1i32 } else { 1 };
            let digits = v.strip_prefix(['-', '+']).unwrap_or(v);
            digits
                .bytes()
                .take_while(u8::is_ascii_digit)
                .fold(0i32, |n, d| {
                    n.wrapping_mul(10).wrapping_add(i32::from(d - b'0'))
                })
                .wrapping_mul(sign)
        })
}
fn corpus_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR"))
        .join("zzzgold/rivs")
}
fn corpus() -> Vec<String> {
    let spec = std::env::var("RIVE_MEASURE_RIVS").unwrap_or_default();
    if spec == "all" {
        let mut names: Vec<_> = std::fs::read_dir(corpus_dir())
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().into_string().ok()?;
                (n.len() > 4 && n.ends_with(".riv")).then_some(n)
            })
            .collect();
        names.sort();
        names
    } else if spec.is_empty() {
        [
            "Halloween_v3.riv",
            "UI_Swipe_left_to_delete.riv",
            "Tom_Morello.riv",
            "Knight_square_2.riv",
            "falling.riv",
            "popsicle_loader.riv",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    } else {
        spec.split(',')
            .filter(|n| !n.is_empty())
            .map(str::to_owned)
            .collect()
    }
}
#[derive(Default)]
struct MPath {
    self_adds: u32,
}
impl RenderPath for MPath {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn rewind(&mut self) {}
    fn fill_rule(&mut self, _: FillRule) {}
    fn add_render_path(&mut self, _: &dyn RenderPath, _: Mat2D) {}
    fn add_render_path_self(&mut self, _: Mat2D) {
        self.self_adds += 1;
    }
    fn add_render_path_backwards(&mut self, _: &dyn RenderPath, _: Mat2D) {}
    fn add_raw_path(&mut self, _: &RawPath) {}
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {}
    fn cubic_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
    fn close(&mut self) {}
}
struct MPaint;
impl RenderPaint for MPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn style(&mut self, _: RenderPaintStyle) {}
    fn color(&mut self, _: u32) {}
    fn thickness(&mut self, _: f32) {}
    fn join(&mut self, _: StrokeJoin) {}
    fn cap(&mut self, _: StrokeCap) {}
    fn feather(&mut self, _: f32) {}
    fn blend_mode(&mut self, _: BlendMode) {}
    fn shader(&mut self, _: Option<&dyn RenderShader>) {}
    fn invalidate_stroke(&mut self) {}
}
#[derive(Clone)]
struct MShader(Rc<()>);
impl RenderShader for MShader {
    fn shader_identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn retain_shader(&self) -> Rc<dyn RenderShader> {
        Rc::new(self.clone())
    }
}
#[derive(Clone)]
struct MImage(Rc<()>);
impl RenderImage for MImage {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn width(&self) -> u32 {
        0
    }
    fn height(&self) -> u32 {
        0
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}
#[derive(Default)]
struct MFactory {
    counts: [i64; 5],
}
impl Factory for MFactory {
    fn make_render_path(&mut self, _: RawPath, _: FillRule) -> Box<dyn RenderPath> {
        self.counts[0] += 1;
        Box::new(MPath::default())
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.counts[0] += 1;
        Box::new(MPath::default())
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.counts[1] += 1;
        Box::new(MPaint)
    }
    fn make_linear_gradient(
        &mut self,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: &[u32],
        _: &[f32],
    ) -> Box<dyn RenderShader> {
        self.counts[2] += 1;
        Box::new(MShader(Rc::new(())))
    }
    fn make_radial_gradient(
        &mut self,
        _: f32,
        _: f32,
        _: f32,
        _: &[u32],
        _: &[f32],
    ) -> Box<dyn RenderShader> {
        self.counts[2] += 1;
        Box::new(MShader(Rc::new(())))
    }
    fn make_render_buffer(
        &mut self,
        t: RenderBufferType,
        f: RenderBufferFlags,
        size: usize,
    ) -> Box<dyn RenderBuffer> {
        self.counts[3] += 1;
        NullFactory.make_render_buffer(t, f, size)
    }
    fn decode_image(&mut self, _: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.counts[4] += 1;
        Ok(Box::new(MImage(Rc::new(()))))
    }
}

#[test]
fn same_id_path_append_reaches_the_headless_measure_sink_without_aliasing() {
    let mut buffer = RenderCommandBuffer::default();
    buffer.append(
        RenderCmd::MakeEmptyPath,
        &MakeIdPod {
            id: 0,
            generation: 0,
        },
    );
    buffer.append(
        RenderCmd::PathAddRenderPath,
        &PathAddPathPod {
            path: 0,
            src: 0,
            xx: 2.0,
            xy: 0.0,
            yx: 0.0,
            yy: 3.0,
            tx: 4.0,
            ty: 5.0,
        },
    );

    let mut factory = MFactory::default();
    let mut table = ResourceTable::default();
    replay_render_commands(
        &mut factory,
        None,
        buffer.command_bytes(),
        buffer.blob_bytes(),
        &mut table,
        &mut ReplayHooks::default(),
    );

    let path = table.paths.get(0).expect("resident replay path");
    assert_eq!(
        path.borrow()
            .as_any()
            .downcast_ref::<MPath>()
            .expect("measure path")
            .self_adds,
        1
    );
}
struct MSink {
    factory: PersistentFactory<MFactory>,
}
impl Default for MSink {
    fn default() -> Self {
        Self {
            factory: PersistentFactory::new(MFactory::default()),
        }
    }
}
impl DeferredFrameSink for MSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        assert_eq!(target, 0);
        Some(Rc::new(RefCell::new(Box::new(NullRenderer))))
    }
}
struct Census {
    count: [u64; 33],
    geom_bytes: u64,
    command_bytes: u64,
    blob_bytes: u64,
    frames: u64,
    overrun: bool,
}
impl Default for Census {
    fn default() -> Self {
        Self {
            count: [0; 33],
            geom_bytes: 0,
            command_bytes: 0,
            blob_bytes: 0,
            frames: 0,
            overrun: false,
        }
    }
}
impl Census {
    fn add(&mut self, buffer: &RenderCommandBuffer) {
        self.frames += 1;
        self.command_bytes += buffer.command_bytes().len() as u64;
        self.blob_bytes += buffer.blob_bytes().len() as u64;
        let mut reader = CommandReader::new(buffer.command_bytes(), buffer.blob_bytes());
        while let Some(kind) = reader.next_u8() {
            let Some(command) = RenderCmd::from_byte(kind) else {
                self.overrun = true;
                break;
            };
            self.count[kind as usize] += 1;
            match command {
                RenderCmd::MakePath => {
                    let p: MakePathPod = reader.read();
                    self.geom_bytes += u64::from(p.verb_count) + u64::from(p.point_count) * 8;
                }
                RenderCmd::PathAddRawPath => {
                    let p: PathRawPod = reader.read();
                    self.geom_bytes += u64::from(p.verb_count) + u64::from(p.point_count) * 8;
                }
                _ => reader.skip(payload_size_of(command)),
            }
        }
        self.overrun |= reader.overrun();
    }
}
#[derive(Default)]
struct Phase {
    advance_us: f64,
    record_us: f64,
    snapshot_us: f64,
    replay_us: f64,
    frames: u64,
    census: Census,
    sink_counts: [i64; 5],
}
fn row(riv: &str, phase: &str, metric: &str, value: f64) {
    println!("MEASURE,{riv},{phase},{metric},{value:.6}");
}
const COMMAND_NAMES: [&str; 33] = [
    "makePath",
    "makeEmptyPath",
    "makePaint",
    "makeLinearGradient",
    "makeRadialGradient",
    "decodeImage",
    "makeBuffer",
    "bufferData",
    "destroyResource",
    "pathRewind",
    "pathFillRule",
    "pathAddRawPath",
    "pathAddRenderPath",
    "paintStyle",
    "paintColor",
    "paintThickness",
    "paintJoin",
    "paintCap",
    "paintFeather",
    "paintBlendMode",
    "paintShader",
    "paintInvalidateStroke",
    "save",
    "restore",
    "transform",
    "drawPath",
    "clipPath",
    "drawImage",
    "drawImageMesh",
    "modulateOpacity",
    "canvasContentBegin",
    "canvasContentEnd",
    "resourceNewVersion",
];
fn emit(riv: &str, name: &str, p: &Phase) {
    if p.frames == 0 {
        return;
    }
    let n = p.frames as f64;
    row(riv, name, "frames", n);
    for (key, value) in [
        ("advance_us", p.advance_us),
        ("record_us", p.record_us),
        ("snapshot_us", p.snapshot_us),
        ("replay_us", p.replay_us),
        ("cmd_bytes", p.census.command_bytes as f64),
        ("blob_bytes", p.census.blob_bytes as f64),
        (
            "stream_bytes",
            (p.census.command_bytes + p.census.blob_bytes) as f64,
        ),
        ("geom_bytes", p.census.geom_bytes as f64),
    ] {
        row(riv, name, key, value / n);
    }
    for (index, key) in [
        "sink_paths",
        "sink_paints",
        "sink_shaders",
        "sink_buffers",
        "sink_images",
    ]
    .into_iter()
    .enumerate()
    {
        row(riv, name, key, p.sink_counts[index] as f64 / n);
    }
    for (index, &count) in p.census.count.iter().enumerate() {
        if count != 0 {
            row(
                riv,
                name,
                &format!("op_{}", COMMAND_NAMES[index]),
                count as f64 / n,
            );
        }
    }
}
fn resident_counts(t: &ResourceTable) -> (usize, usize, usize) {
    let sizes = [
        t.paths.objects.len(),
        t.paints.objects.len(),
        t.shaders.objects.len(),
        t.images.objects.len(),
        t.buffers.objects.len(),
    ];
    let live = t.paths.objects.iter().filter(|o| o.is_some()).count()
        + t.paints.objects.iter().filter(|o| o.is_some()).count()
        + t.shaders.objects.iter().filter(|o| o.is_some()).count()
        + t.images.objects.iter().filter(|o| o.is_some()).count()
        + t.buffers.objects.iter().filter(|o| o.is_some()).count();
    let slots = sizes.iter().sum();
    let bytes = sizes[0] * (std::mem::size_of::<super::super::render_replay::PathOwner>() + 8)
        + sizes[1] * (std::mem::size_of::<super::super::render_replay::PaintOwner>() + 8)
        + sizes[2] * (std::mem::size_of::<Rc<dyn RenderShader>>() + 8)
        + sizes[3] * (std::mem::size_of::<Rc<dyn RenderImage>>() + 8)
        + sizes[4] * (std::mem::size_of::<super::super::render_replay::BufferOwner>() + 8);
    (slots, live, bytes)
}
fn measure_riv(name: &str, frames: i32, warmup: i32) {
    let Ok(bytes) = std::fs::read(corpus_dir().join(name)) else {
        println!("MEASURE_SKIP,{name},missing");
        return;
    };
    let mut factory = PersistentFactory::new(DeferredSession::new(None));
    let mut case = match RuntimeCase::import_result(&bytes, &mut factory) {
        Ok(case) => case,
        Err(reason) => {
            println!("MEASURE_SKIP,{name},{reason}");
            return;
        }
    };
    let mut sink = MSink::default();
    let mut replayer = DeferredReplayer::default();
    let mut phases: [Phase; 3] = std::array::from_fn(|_| Phase::default());
    let mut retained = -3;
    let mut dropped = 0u32;
    for frame in 0..frames {
        let p = &mut phases[if frame == 0 {
            0
        } else if frame < warmup {
            1
        } else {
            2
        }];
        let t0 = Instant::now();
        case.advance(if frame == 0 { 0.0 } else { 1.0 / 60.0 });
        let t1 = Instant::now();
        let renderer = factory.borrow().screen_renderer(0);
        case.draw(renderer.borrow_mut().as_mut());
        let t2 = Instant::now();
        p.census
            .add(&factory.borrow().command_buffer().lock().unwrap());
        let snapshot = snapshot_frame(&mut factory.borrow_mut());
        factory.borrow_mut().reset_frame();
        let t3 = Instant::now();
        let before = sink.factory.borrow().counts;
        replayer.replay_frame(&snapshot, &mut sink);
        let t4 = Instant::now();
        dropped = dropped.wrapping_add(replayer.dropped_draws());
        p.advance_us += (t1 - t0).as_secs_f64() * 1e6;
        p.record_us += (t2 - t1).as_secs_f64() * 1e6;
        p.snapshot_us += (t3 - t2).as_secs_f64() * 1e6;
        p.replay_us += (t4 - t3).as_secs_f64() * 1e6;
        p.frames += 1;
        for i in 0..5 {
            p.sink_counts[i] += sink.factory.borrow().counts[i] - before[i];
        }
        if frame == frames - 1 {
            retained = -2;
        }
    }
    for (namep, p) in ["first", "transient", "steady"].into_iter().zip(&phases) {
        emit(name, namep, p);
    }
    row(name, "run", "retained_geometry_bytes", retained as f64);
    row(name, "run", "dropped_draws", dropped as f64);
    row(
        name,
        "run",
        "stream_overrun",
        if phases.iter().any(|p| p.census.overrun) {
            1.0
        } else {
            0.0
        },
    );
    let t = &replayer.table;
    for (key, value) in [
        ("path_slots", t.paths.objects.len()),
        ("path_live", t.paths.objects.iter().flatten().count()),
        ("paint_slots", t.paints.objects.len()),
        ("paint_live", t.paints.objects.iter().flatten().count()),
        ("shader_slots", t.shaders.objects.len()),
        ("shader_live", t.shaders.objects.iter().flatten().count()),
        ("image_slots", t.images.objects.len()),
        ("buffer_slots", t.buffers.objects.len()),
        ("buffer_live", t.buffers.objects.iter().flatten().count()),
    ] {
        row(name, "resident", key, value as f64);
    }
    row(
        name,
        "resident",
        "slot_vector_bytes",
        resident_counts(t).2 as f64,
    );
}
#[test]
#[ignore = "upstream hidden deferred_measure"]
fn deferred_measure() {
    let frames = env_int("RIVE_MEASURE_FRAMES", 3000);
    let warmup = env_int("RIVE_MEASURE_WARMUP", 300);
    println!("MEASURE_CONFIG,frames,{frames},warmup,{warmup}");
    println!("MEASURE_CONFIG,retained_instrumented,0");
    for name in corpus() {
        measure_riv(&name, frames, warmup);
    }
}
#[test]
#[ignore = "upstream hidden deferred_measure concurrent sessions"]
fn concurrent_sessions() {
    let sessions = env_int("RIVE_MEASURE_SESSIONS", 8);
    let frames = (env_int("RIVE_MEASURE_FRAMES", 3000) / 100).max(4);
    let names = corpus();
    if names.is_empty() {
        return;
    }
    struct Live {
        factory: PersistentFactory<DeferredSession>,
        case: RuntimeCase,
        sink: MSink,
        replayer: DeferredReplayer,
    }
    let mut live = Vec::new();
    for i in 0..sessions {
        let Ok(bytes) = std::fs::read(corpus_dir().join(&names[i as usize % names.len()])) else {
            continue;
        };
        let mut factory = PersistentFactory::new(DeferredSession::new(None));
        let Some(case) = RuntimeCase::import(&bytes, &mut factory) else {
            continue;
        };
        live.push(Live {
            factory,
            case,
            sink: MSink::default(),
            replayer: DeferredReplayer::default(),
        });
    }
    println!(
        "MEASURE_CONFIG,concurrent_sessions,{},frames,{frames}",
        live.len()
    );
    for frame in 0..frames {
        for l in &mut live {
            l.case.advance(if frame == 0 { 0.0 } else { 1.0 / 60.0 });
            let renderer = l.factory.borrow().screen_renderer(0);
            l.case.draw(renderer.borrow_mut().as_mut());
            let snapshot = snapshot_frame(&mut l.factory.borrow_mut());
            l.factory.borrow_mut().reset_frame();
            l.replayer.replay_frame(&snapshot, &mut l.sink);
        }
    }
    let mut total_slots = 0;
    let mut total_live = 0;
    let mut total_bytes = 0;
    for (i, l) in live.iter().enumerate() {
        let (slots, used, _) = resident_counts(&l.replayer.table);
        let bytes = slots * (std::mem::size_of::<super::super::render_replay::PathOwner>() + 8);
        println!("MEASURE,session_{i},resident,slots,{slots}");
        println!("MEASURE,session_{i},resident,live,{used}");
        total_slots += slots;
        total_live += used;
        total_bytes += bytes;
    }
    println!("MEASURE,all_sessions,resident,slots,{total_slots}");
    println!("MEASURE,all_sessions,resident,live,{total_live}");
    println!("MEASURE,all_sessions,resident,slot_vector_bytes,{total_bytes}");
}
