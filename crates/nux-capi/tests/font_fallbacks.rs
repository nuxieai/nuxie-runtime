//! Native drawing through the public C process hook and live text surfaces.
use nux_capi::*;
use nuxie_binary::BinaryWriter;
use std::{collections::HashMap, ffi::c_void, ptr};

static HOOK_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

const LATIN: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
const ARABIC: &[u8] = include_bytes!("../../../fixtures/fonts/ibm-plex-sans-arabic.ttf");

fn key(kind: &str, name: &str) -> u32 {
    let definition = nuxie_schema::definition_by_name(kind).unwrap();
    u32::from(
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|name| {
                nuxie_schema::definition_by_name(name)
                    .unwrap()
                    .properties
                    .iter()
            }))
            .find(|p| p.name == name)
            .unwrap()
            .key
            .int,
    )
}

fn object(writer: &mut BinaryWriter<'_>, kind: &str, fields: impl FnOnce(&mut BinaryWriter<'_>)) {
    writer.write_var_uint32(u32::from(
        nuxie_schema::definition_by_name(kind).unwrap().type_key.int,
    ));
    fields(writer);
    writer.write_var_uint32(0);
}

// Two independent authored font occurrences must both see the process hook.
fn scene(primary: &[u8], value: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut w = BinaryWriter::new(&mut bytes);
    w.write_bytes(b"RIVE");
    for value in [7, 0, 9641, 0] {
        w.write_var_uint32(value);
    }
    object(&mut w, "Backboard", |_| {});
    for _ in 0..2 {
        object(&mut w, "FontAsset", |_| {});
        object(&mut w, "FileAssetContents", |w| {
            w.write_var_uint32(key("FileAssetContents", "bytes"));
            w.write_string(primary);
        });
    }
    object(&mut w, "Artboard", |w| {
        for (name, value) in [("width", 500.0), ("height", 300.0)] {
            w.write_var_uint32(key("Artboard", name));
            w.write_f32(value);
        }
    });
    for row in 0..2 {
        let text = 1 + row * 5;
        object(&mut w, "Text", |w| {
            w.write_var_uint32(key("Text", "parentId"));
            w.write_var_uint32(0);
            w.write_var_uint32(key("Text", "y"));
            w.write_f32(row as f32 * 100.0);
        });
        object(&mut w, "TextStylePaint", |w| {
            w.write_var_uint32(key("TextStylePaint", "parentId"));
            w.write_var_uint32(text);
            w.write_var_uint32(key("TextStylePaint", "fontAssetId"));
            w.write_var_uint32(row);
            w.write_var_uint32(key("TextStylePaint", "fontSize"));
            w.write_f32(32.0);
        });
        object(&mut w, "Fill", |w| {
            w.write_var_uint32(key("Fill", "parentId"));
            w.write_var_uint32(text + 1);
        });
        object(&mut w, "SolidColor", |w| {
            w.write_var_uint32(key("SolidColor", "parentId"));
            w.write_var_uint32(text + 2);
            w.write_var_uint32(key("SolidColor", "colorValue"));
            w.write_u32(0xff000000);
        });
        object(&mut w, "TextValueRun", |w| {
            w.write_var_uint32(key("TextValueRun", "parentId"));
            w.write_var_uint32(text);
            w.write_var_uint32(key("TextValueRun", "styleId"));
            w.write_var_uint32(text + 1);
            w.write_var_uint32(key("TextValueRun", "name"));
            w.write_string(format!("row{row}").as_bytes());
            w.write_var_uint32(key("TextValueRun", "text"));
            w.write_string(value.as_bytes());
        });
    }
    drop(w);
    bytes
}

struct Recording {
    paths: HashMap<u64, Vec<(Vec<u8>, Vec<u32>)>>,
    drawn: Vec<Vec<(Vec<u8>, Vec<u32>)>>,
    drawn_transforms: Vec<[f32; 6]>,
    transform: [f32; 6],
    saved_transforms: Vec<[f32; 6]>,
}
impl Default for Recording {
    fn default() -> Self {
        Self {
            paths: Default::default(),
            drawn: Vec::new(),
            drawn_transforms: Vec::new(),
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            saved_transforms: Vec::new(),
        }
    }
}

unsafe fn recording<'a>(context: *mut c_void) -> &'a mut Recording {
    unsafe { &mut *context.cast() }
}
unsafe extern "C" fn empty_path(context: *mut c_void) -> u64 {
    let r = unsafe { recording(context) };
    let id = r.paths.len() as u64 + 1;
    r.paths.insert(id, Vec::new());
    id
}
unsafe extern "C" fn raw_path(context: *mut c_void, path: *const NuxRawPathView, _: u8) -> u64 {
    let id = unsafe { empty_path(context) };
    unsafe { add_raw(context, id, path) };
    id
}
unsafe extern "C" fn add_raw(context: *mut c_void, id: u64, path: *const NuxRawPathView) {
    let path = unsafe { &*path };
    if path.verb_count == 0 {
        return;
    }
    let verbs = unsafe { std::slice::from_raw_parts(path.verbs, path.verb_count) }.to_vec();
    let points = unsafe { std::slice::from_raw_parts(path.points, path.point_count * 2) }
        .iter()
        .map(|v| v.to_bits())
        .collect();
    unsafe { recording(context) }
        .paths
        .get_mut(&id)
        .unwrap()
        .push((verbs, points));
}
unsafe extern "C" fn rewind(context: *mut c_void, id: u64) {
    unsafe { recording(context) }
        .paths
        .get_mut(&id)
        .unwrap()
        .clear();
}
unsafe extern "C" fn add_path(context: *mut c_void, to: u64, from: u64, transform: *const f32) {
    let r = unsafe { recording(context) };
    let mut source = r.paths[&from].clone();
    let matrix = unsafe { std::slice::from_raw_parts(transform, 6) };
    for (_, points) in &mut source {
        for point in points.chunks_exact_mut(2) {
            let x = f32::from_bits(point[0]);
            let y = f32::from_bits(point[1]);
            point[0] = (matrix[0] * x + matrix[2] * y + matrix[4]).to_bits();
            point[1] = (matrix[1] * x + matrix[3] * y + matrix[5]).to_bits();
        }
    }
    r.paths.get_mut(&to).unwrap().extend(source);
}
unsafe extern "C" fn paint(_: *mut c_void) -> u64 {
    1
}
unsafe extern "C" fn draw(context: *mut c_void, path: u64, _: u64) {
    let r = unsafe { recording(context) };
    let path = r.paths[&path].clone();
    if !path.is_empty() {
        r.drawn.push(path);
        r.drawn_transforms.push(r.transform);
    }
}
unsafe extern "C" fn save(context: *mut c_void) {
    let r = unsafe { recording(context) };
    r.saved_transforms.push(r.transform);
}
unsafe extern "C" fn restore(context: *mut c_void) {
    let r = unsafe { recording(context) };
    r.transform = r.saved_transforms.pop().unwrap();
}
unsafe extern "C" fn transform(context: *mut c_void, incoming: *const f32) {
    let r = unsafe { recording(context) };
    let a = r.transform;
    let b = unsafe { std::slice::from_raw_parts(incoming, 6) };
    r.transform = [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ];
}

fn hook_test_lock() -> std::sync::MutexGuard<'static, ()> {
    HOOK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn render(primary: &[u8], value: &str, live_value: Option<&str>) -> Recording {
    render_retiring(primary, value, live_value, ptr::null_mut())
}

fn render_retiring(
    primary: &[u8],
    value: &str,
    live_value: Option<&str>,
    retire_font: *mut NuxFont,
) -> Recording {
    let bytes = scene(primary, value);
    let mut r = Recording::default();
    let callbacks = NuxRenderCallbacks {
        user_data: ptr::from_mut(&mut r).cast(),
        make_render_path: Some(raw_path),
        make_empty_render_path: Some(empty_path),
        make_render_paint: Some(paint),
        render_path_add_raw_path: Some(add_raw),
        render_path_rewind: Some(rewind),
        render_path_add_render_path: Some(add_path),
        draw_path: Some(draw),
        save: Some(save),
        restore: Some(restore),
        transform: Some(transform),
        ..Default::default()
    };
    unsafe {
        let mut file = ptr::null_mut();
        assert_eq!(
            nux_file_import(bytes.as_ptr(), bytes.len(), &callbacks, &mut file),
            NuxStatus::Ok
        );
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        if let Some(value) = live_value {
            let mutations = ["row0", "row1"].map(|name| NuxTextRunMutation {
                name: view(name),
                text: NuxByteView {
                    data: value.as_ptr(),
                    len: value.len(),
                },
            });
            let batch = NuxTextRunMutationBatch {
                mutations: mutations.as_ptr(),
                mutation_count: 2,
                ..Default::default()
            };
            assert_eq!(
                nux_artboard_instance_set_text_runs(artboard, &batch, ptr::null_mut()),
                NuxStatus::Ok
            );
        }
        // The occurrence retains its primary font after the file handle dies.
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
        assert_eq!(
            nux_artboard_instance_advance(artboard, 0.0, ptr::null_mut()),
            NuxStatus::Ok
        );
        if !retire_font.is_null() {
            assert_eq!(
                nux_font_set_fallback_proc(None, ptr::null_mut()),
                NuxStatus::Ok
            );
            assert_eq!(nux_font_free(retire_font), NuxStatus::Ok);
        }
        assert_eq!(nux_artboard_instance_draw(artboard), NuxStatus::Ok);
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
    }
    r
}

fn byte_font(bytes: &[u8]) -> *mut NuxFont {
    let source = NuxFontSource {
        bytes: NuxByteView {
            data: bytes.as_ptr(),
            len: bytes.len(),
        },
        ..Default::default()
    };
    let mut font = ptr::null_mut();
    unsafe {
        assert_eq!(nux_font_new(&source, &mut font), NuxStatus::Ok);
    }
    font
}

unsafe extern "C" fn pick_font(
    context: *mut c_void,
    missing: u32,
    index: u32,
    weight: u16,
) -> *const NuxFont {
    assert_eq!(weight, 400);
    if index != 0 {
        return ptr::null();
    }
    let font = context.cast::<NuxFont>();
    let mut has = false;
    unsafe {
        assert_eq!(nux_font_has_glyph(font, missing, &mut has), NuxStatus::Ok);
    }
    if has { font } else { ptr::null() }
}

#[test]
fn process_hook_draws_arabic_keeps_latin_and_clear_restores_missing_glyphs() {
    let _guard = hook_test_lock();
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
    }
    let covering = render(ARABIC, "مرحبا", None).drawn;
    assert_eq!(
        covering.len(),
        2,
        "independently authored covering rows draw"
    );
    assert!(render(LATIN, "مرحبا", None).drawn.is_empty());
    let latin = render(LATIN, "Ada", None).drawn;
    assert_eq!(latin.len(), 2);
    let fallback = byte_font(ARABIC);
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(Some(pick_font), fallback.cast()),
            NuxStatus::Ok
        );
    }
    let actual = render(LATIN, "مرحبا", None).drawn;
    // Clear before assertions so failed comparisons cannot leak global state.
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
    }
    assert_eq!(
        actual, covering,
        "the C hook must draw the covering font's Arabic outlines"
    );
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(Some(pick_font), fallback.cast()),
            NuxStatus::Ok
        );
    }
    assert_eq!(render(LATIN, "Ada", None).drawn, latin);
    assert_eq!(render(LATIN, "Ada", Some("مرحبا")).drawn, covering);
    // The same cache handle can answer on a different shaping thread.
    assert_eq!(
        std::thread::spawn(|| render(LATIN, "مرحبا", None).drawn)
            .join()
            .unwrap(),
        covering
    );
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
        assert_eq!(nux_font_free(fallback), NuxStatus::Ok);
    }
    assert!(render(LATIN, "مرحبا", None).drawn.is_empty());
    assert_eq!(render(LATIN, "Ada", None).drawn, latin);
    // Shape with a mapped font, then retire the handle and hook before drawing.
    // The shaped runs must retain their font and mapping independently.
    let path = std::path::Path::new(
        option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
    )
    .join("../../fixtures/fonts/ibm-plex-sans-arabic.ttf")
    .canonicalize()
    .unwrap();
    let source = NuxFontSource {
        immutable_path: view(path.to_str().unwrap()),
        ..Default::default()
    };
    let mut mapped = ptr::null_mut();
    unsafe {
        assert_eq!(nux_font_new(&source, &mut mapped), NuxStatus::Ok);
        assert_eq!(
            nux_font_set_fallback_proc(Some(pick_font), mapped.cast()),
            NuxStatus::Ok
        );
    }
    assert_eq!(
        render_retiring(LATIN, "مرحبا", None, mapped).drawn,
        covering
    );
}

#[test]
fn mapped_probe_transfers_mapping_and_weight_copy_outlives_original_handles() {
    let path = std::path::Path::new(
        option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
    )
    .join("../../fixtures/fonts/ibm-plex-sans-arabic.ttf")
    .canonicalize()
    .unwrap();
    unsafe {
        let mut probe = ptr::null_mut();
        assert_eq!(
            nux_font_probe_file(view(path.to_str().unwrap()), 0, &mut probe),
            NuxStatus::Ok
        );
        let mut has = false;
        assert_eq!(
            nux_font_probe_has_glyph(probe, 'م' as u32, &mut has),
            NuxStatus::Ok
        );
        assert!(has);
        assert_eq!(
            nux_font_probe_has_glyph(probe, 0x10ffff, &mut has),
            NuxStatus::Ok
        );
        assert!(!has);
        let mut font = ptr::null_mut();
        assert_eq!(nux_font_probe_make_font(probe, &mut font), NuxStatus::Ok);
        assert_eq!(
            nux_font_probe_has_glyph(probe, 'م' as u32, &mut has),
            NuxStatus::Ok
        );
        assert!(!has, "promoted probes no longer own a face");
        let mut second = ptr::null_mut();
        assert_eq!(
            nux_font_probe_make_font(probe, &mut second),
            NuxStatus::NotFound
        );
        assert!(second.is_null());
        let mut weighted = ptr::null_mut();
        assert_eq!(
            nux_font_with_weight(font, 700, &mut weighted),
            NuxStatus::Ok
        );
        assert_eq!(nux_font_probe_free(probe), NuxStatus::Ok);
        assert_eq!(nux_font_free(font), NuxStatus::Ok);
        assert_eq!(
            nux_font_has_glyph(weighted, 'م' as u32, &mut has),
            NuxStatus::Ok
        );
        assert!(has);
        let address = weighted as usize;
        assert_eq!(
            std::thread::spawn(move || nux_font_free(address as *mut NuxFont))
                .join()
                .unwrap(),
            NuxStatus::Ok
        );
    }
}

#[test]
fn malformed_font_sources_publish_no_handle() {
    for (source, expected) in [
        (
            NuxFontSource {
                struct_size: 0,
                ..Default::default()
            },
            NuxStatus::InvalidStructSize,
        ),
        (NuxFontSource::default(), NuxStatus::InvalidArgument),
        (
            NuxFontSource {
                bytes: NuxByteView {
                    data: ptr::null(),
                    len: 1,
                },
                ..Default::default()
            },
            NuxStatus::NullArgument,
        ),
        (
            NuxFontSource {
                bytes: NuxByteView {
                    data: ptr::dangling(),
                    len: NUX_FONT_MAX_BYTES + 1,
                },
                ..Default::default()
            },
            NuxStatus::LimitExceeded,
        ),
        (
            NuxFontSource {
                bytes: NuxByteView {
                    data: ARABIC.as_ptr(),
                    len: ARABIC.len(),
                },
                face_index: 1,
                ..Default::default()
            },
            NuxStatus::ImportError,
        ),
        (
            NuxFontSource {
                immutable_path: view("relative.ttf"),
                ..Default::default()
            },
            NuxStatus::InvalidArgument,
        ),
    ] {
        let mut out = ptr::dangling_mut();
        unsafe {
            assert_eq!(nux_font_new(&source, &mut out), expected);
        }
        assert!(out.is_null());
    }
}

#[test]
fn fallback_hook_reentry_returns_reentrant_call() {
    const CHILD: &str = "NUX_FONT_REENTRY_TEST_CHILD";
    if std::env::var_os(CHILD).is_some() {
        unsafe extern "C" fn reenter(_: *mut c_void, _: u32, _: u32, _: u16) -> *const NuxFont {
            assert_eq!(
                unsafe { nux_font_set_fallback_proc(None, ptr::null_mut()) },
                NuxStatus::ReentrantCall
            );
            let mut file = ptr::null_mut();
            assert_eq!(
                unsafe { nux_file_import(ptr::null(), 0, ptr::null(), &mut file) },
                NuxStatus::ReentrantCall
            );
            ptr::null()
        }
        unsafe {
            assert_eq!(
                nux_font_set_fallback_proc(Some(reenter), ptr::null_mut()),
                NuxStatus::Ok
            );
        }
        render(LATIN, "مرحبا", None);
        return;
    }
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "fallback_hook_reentry_returns_reentrant_call"])
        .env(CHILD, "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "the callback must receive REENTRANT_CALL");
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("replacing the hook from its callback deadlocked");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn repeated_font_handle_ends_the_c_fallback_walk() {
    let _guard = hook_test_lock();
    struct Context {
        font: *mut NuxFont,
        indices: std::sync::Mutex<Vec<u32>>,
    }
    unsafe extern "C" fn same(context: *mut c_void, _: u32, index: u32, _: u16) -> *const NuxFont {
        let context = unsafe { &*context.cast::<Context>() };
        context.indices.lock().unwrap().push(index);
        // Bound a broken implementation so the assertion fails instead of recursing forever.
        if index > 4 {
            return ptr::null();
        }
        context.font
    }
    let mut context = Context {
        font: byte_font(LATIN),
        indices: Default::default(),
    };
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(Some(same), ptr::from_mut(&mut context).cast()),
            NuxStatus::Ok
        );
    }
    render(LATIN, "م", None);
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
        assert_eq!(nux_font_free(context.font), NuxStatus::Ok);
    }
    let indices = context.indices.into_inner().unwrap();
    assert!(!indices.is_empty());
    assert!(indices.chunks_exact(2).remainder().is_empty());
    assert!(
        indices.chunks_exact(2).all(|pair| pair == [0, 1]),
        "one fallback followed by the same-identity retry must stop: {indices:?}"
    );
}

#[test]
fn replacing_the_hook_waits_for_its_in_flight_callback() {
    use std::sync::{
        Barrier,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    let _guard = hook_test_lock();
    struct Context {
        entered: Barrier,
        release: Barrier,
        first: AtomicBool,
    }
    unsafe extern "C" fn blocked(context: *mut c_void, _: u32, _: u32, _: u16) -> *const NuxFont {
        let context = unsafe { &*context.cast::<Context>() };
        if context.first.swap(false, Ordering::SeqCst) {
            context.entered.wait();
            context.release.wait();
        }
        ptr::null()
    }
    let context = Box::new(Context {
        entered: Barrier::new(2),
        release: Barrier::new(2),
        first: AtomicBool::new(true),
    });
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(
                Some(blocked),
                ptr::from_ref(context.as_ref()).cast_mut().cast()
            ),
            NuxStatus::Ok
        );
    }
    let shaping = std::thread::spawn(|| render(LATIN, "م", None));
    context.entered.wait();
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let replacing = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        let status = unsafe { nux_font_set_fallback_proc(None, ptr::null_mut()) };
        done_tx.send(status).unwrap();
    });
    started_rx.recv().unwrap();
    let premature = done_rx.recv_timeout(std::time::Duration::from_millis(100));
    context.release.wait();
    shaping.join().unwrap();
    replacing.join().unwrap();
    assert!(
        matches!(premature, Err(mpsc::RecvTimeoutError::Timeout)),
        "replacement returned while the old context was still borrowed"
    );
    assert_eq!(
        done_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        NuxStatus::Ok
    );
}

#[test]
fn process_hook_and_host_outline_draw_a_known_square() {
    let _guard = hook_test_lock();
    unsafe extern "C" fn square(
        _: *mut c_void,
        _: u16,
        _: *const NuxFontAxisCoord,
        _: usize,
        sink: NuxFontOutlineSink,
        context: *mut c_void,
    ) {
        unsafe {
            sink(context, 0, [0.0, 0.0].as_ptr());
            sink(context, 1, [1.0, 0.0].as_ptr());
            sink(context, 1, [1.0, 1.0].as_ptr());
            sink(context, 1, [0.0, 1.0].as_ptr());
            sink(context, 4, ptr::null());
        }
    }
    assert!(
        render(
            include_bytes!("../../../fixtures/fonts/roboto-a.ttf"),
            " ",
            None
        )
        .drawn
        .is_empty(),
        "the primary font must not draw the square without the hook"
    );
    let source = NuxFontSource {
        bytes: NuxByteView {
            data: LATIN.as_ptr(),
            len: LATIN.len(),
        },
        ..Default::default()
    };
    let callbacks = NuxFontOutlineCallbacks {
        struct_size: std::mem::size_of::<NuxFontOutlineCallbacks>() as u32,
        user_data: ptr::null_mut(),
        get_path: Some(square),
        release: None,
    };
    let mut font = ptr::null_mut();
    unsafe {
        assert_eq!(
            nux_font_new_with_outline(&source, &callbacks, &mut font),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_font_set_fallback_proc(Some(pick_font), font.cast()),
            NuxStatus::Ok
        );
    }
    // The primary has only 'a'; the fallback maps space but has no outline for it.
    let actual = render(
        include_bytes!("../../../fixtures/fonts/roboto-a.ttf"),
        " ",
        None,
    );
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
        assert_eq!(nux_font_free(font), NuxStatus::Ok);
    }
    assert_eq!(
        actual.drawn.len(),
        2,
        "both authored rows must draw the host shape"
    );
    let mut origins = Vec::new();
    for (row, matrix) in actual.drawn.iter().zip(&actual.drawn_transforms) {
        assert_eq!(row.len(), 1);
        assert_eq!(row[0].0, [0, 1, 1, 1, 5]);
        let points: Vec<_> = row[0].1.iter().map(|bits| f32::from_bits(*bits)).collect();
        let transformed: Vec<_> = points
            .chunks_exact(2)
            .map(|point| {
                [
                    matrix[0] * point[0] + matrix[2] * point[1] + matrix[4],
                    matrix[1] * point[0] + matrix[3] * point[1] + matrix[5],
                ]
            })
            .collect();
        let origin = transformed[0];
        origins.push(origin);
        let relative: Vec<_> = transformed
            .iter()
            .map(|point| [point[0] - origin[0], point[1] - origin[1]])
            .collect();
        assert_eq!(
            relative,
            [[0.0, 0.0], [32.0, 0.0], [32.0, 32.0], [0.0, 32.0]]
        );
    }
    let y0 = origins[0][1];
    let y1 = origins[1][1];
    assert_eq!(
        (y1 - y0).abs(),
        100.0,
        "the authored row origins must stay 100 units apart regardless of draw order"
    );
}
