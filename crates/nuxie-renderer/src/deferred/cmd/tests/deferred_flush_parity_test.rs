//! tests/unit_tests/renderer/deferred_flush_parity_test.cpp at e949498e.
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::render_context_null::*;
use super::*;
use std::path::{Path, PathBuf};
const FRAMES: usize = 30;
const FIRST_STEADY: usize = 2;
fn immediate(path: &Path) -> (Vec<FlushStats>, u32) {
    let (mut factory, stats, features) = observing_factory(1, 1);
    let mut case = RuntimeCase::import(&std::fs::read(path).unwrap(), &mut factory)
        .expect("parity fixture imports");
    let (width, height) = case
        .artboard
        .with_artboard(|a| (a.width().ceil() as u32, a.height().ceil() as u32));
    factory.borrow().resize(width, height).unwrap();
    let mut frames = Vec::new();
    for frame in 0..FRAMES {
        case.advance(if frame == 0 { 0.0 } else { 1.0 / 60.0 });
        let before = *stats.borrow();
        let mut renderer = factory
            .borrow()
            .begin_frame(0, crate::RenderMode::RasterOrdering)
            .unwrap();
        case.draw(&mut renderer);
        renderer.finish_without_readback().unwrap();
        frames.push(*stats.borrow() - before);
    }
    (frames, features.get())
}
struct NullContextSink {
    factory: ObservingFactory,
    renderer: Option<RendererOwner>,
}
impl DeferredFrameSink for NullContextSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        assert_eq!(target, 0);
        let renderer: RendererOwner = Rc::new(RefCell::new(Box::new(
            self.factory
                .borrow()
                .begin_frame(0, crate::RenderMode::RasterOrdering)
                .unwrap(),
        )));
        self.renderer = Some(renderer.clone());
        Some(renderer)
    }
}
fn deferred(path: &Path) -> Vec<FlushStats> {
    let mut factory = PersistentFactory::new(DeferredSession::new(None));
    let mut case = RuntimeCase::import(&std::fs::read(path).unwrap(), &mut factory)
        .expect("parity fixture imports");
    let (width, height) = case
        .artboard
        .with_artboard(|a| (a.width().ceil() as u32, a.height().ceil() as u32));
    let (real, stats, _) = observing_factory(width, height);
    let mut sink = NullContextSink {
        factory: real,
        renderer: None,
    };
    let mut replayer = DeferredReplayer::default();
    let mut frames = Vec::new();
    for frame in 0..FRAMES {
        case.advance(if frame == 0 { 0.0 } else { 1.0 / 60.0 });
        let renderer = factory.borrow().screen_renderer(0);
        case.draw(renderer.borrow_mut().as_mut());
        let snapshot = snapshot_frame(&mut factory.borrow_mut());
        factory.borrow_mut().reset_frame();
        let before = *stats.borrow();
        replayer.replay_frame(&snapshot, &mut sink);
        assert_eq!(replayer.dropped_draws(), 0);
        if sink.renderer.is_some() {
            sink.factory
                .borrow()
                .with_backend_mut_for_test(NullBackend::flush);
            sink.renderer = None;
        }
        frames.push(*stats.borrow() - before);
    }
    frames
}
fn check_parity(name: &str, immediate: &[FlushStats], deferred: &[FlushStats]) {
    for i in FIRST_STEADY..immediate.len() {
        assert_eq!(
            immediate[i].flushes, deferred[i].flushes,
            "{name} frame{i} flushes"
        );
        assert_eq!(
            immediate[i].tess_vertex_spans, deferred[i].tess_vertex_spans,
            "{name} frame{i} tess spans"
        );
        assert_eq!(
            immediate[i].atlas_content_area, deferred[i].atlas_content_area,
            "{name} frame{i} atlas area"
        );
        assert_eq!(
            immediate[i].grad_data_height, deferred[i].grad_data_height,
            "{name} frame{i} gradient height"
        );
    }
}
fn print_parity(name: &str, a: &[FlushStats], b: &[FlushStats]) {
    println!(
        "\n== {name} flush parity (steady state, per frame) ==\n                     immediate     deferred"
    );
    let rows: [(&str, fn(&FlushStats) -> u64); 10] = [
        ("flushes", |s| s.flushes),
        ("paths", |s| s.path_count),
        ("contours", |s| s.contour_count),
        ("tessSpans", |s| s.tess_vertex_spans),
        ("tessDataHeight", |s| s.tess_data_height),
        ("gradSpans", |s| s.grad_spans),
        ("gradDataHeight", |s| s.grad_data_height),
        ("atlasFillBatches", |s| s.atlas_fill_batches),
        ("atlasStrokeBatches", |s| s.atlas_stroke_batches),
        ("atlasContentArea", |s| s.atlas_content_area),
    ];
    for (name, pick) in rows {
        let avg = |v: &[FlushStats]| {
            v[FIRST_STEADY..]
                .iter()
                .map(|s| pick(s) as f64)
                .sum::<f64>()
                / (v.len() - FIRST_STEADY) as f64
        };
        println!("  {name:18} {:12.1} {:12.1}", avg(a), avg(b));
    }
}
fn parity(name: &str) {
    let root =
        std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR pinned fixture checkout");
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets/parity")
        .join(name);
    assert!(
        path.is_file(),
        "flush parity riv missing: {}",
        path.display()
    );
    let (a, features) = immediate(&path);
    let b = deferred(&path);
    print_parity(name, &a, &b);
    let names = [
        "CLIPPING",
        "CLIP_RECT",
        "ADVANCED_BLEND",
        "FEATHER",
        "EVEN_ODD",
        "NESTED_CLIPPING",
        "HSL_BLEND_MODES",
        "DITHER",
    ];
    println!(
        "  shader features: {}",
        names
            .into_iter()
            .enumerate()
            .filter_map(|(i, name)| (features & (1 << i) != 0).then_some(name))
            .collect::<Vec<_>>()
            .join(" ")
    );
    check_parity(name, &a, &b);
}
#[test]
fn regressing_rivs() {
    for name in [
        "Halloween_v3.riv",
        "UI_Swipe_left_to_delete.riv",
        "Tom_Morello.riv",
    ] {
        parity(name);
    }
}
#[test]
fn parity_rivs() {
    for name in ["Knight_square_2.riv", "falling.riv", "popsicle_loader.riv"] {
        parity(name);
    }
}
#[test]
#[ignore = "upstream hidden corpus_parity: full zzzgold LFS corpus"]
fn whole_corpus() {
    let root = PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR"));
    let dir = root.join("zzzgold/rivs");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().into_string().ok()?;
            (n.len() > 4 && n.ends_with(".riv")).then_some(n)
        })
        .collect();
    names.sort();
    println!("corpus flush parity over {} rivs", names.len());
    for name in names {
        let path = dir.join(&name);
        if !path.is_file() {
            continue;
        }
        let (a, _) = immediate(&path);
        let b = deferred(&path);
        check_parity(&name, &a, &b);
    }
}
