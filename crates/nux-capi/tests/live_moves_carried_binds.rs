//! A carried element keeps working on the Screen it lands on, through the C
//! API in the iOS SDK's order, after the Screen it came from goes to sleep.
//!
//! The fixture is Duo's rebuild (`fixtures/duo-twobindrel.md`). Its tap and
//! pose paths run through wiring that no runtime fixture builds: a unit's
//! click firing its model's trigger, the slot's two-way trigger channel, the
//! Screen's view-model listener, and a pose binding into a nested component.
//! Through the C API a host hears events and drawing, so these tests read
//! those.
use nux_capi::*;
use std::{cell::RefCell, collections::HashMap, ptr};

const DUO: &[u8] = include_bytes!("fixtures/duo-twobindrel.nux");

fn sv(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
unsafe fn text(view: NuxStringView) -> String {
    if view.data.is_null() || view.len == 0 {
        return String::new();
    }
    String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(view.data.cast::<u8>(), view.len) })
        .into_owned()
}

#[derive(Default)]
struct Catalog {
    schemas: Vec<String>,
    globals: Vec<bool>,
    // schema, name, kind, first enum label, enum label count
    props: Vec<(usize, String, u32, usize, usize)>,
    labels: Vec<String>,
}
unsafe fn catalog(file: *mut NuxFile) -> Catalog {
    unsafe {
        let mut handle = ptr::null_mut();
        assert_eq!(nux_file_view_model_catalog(file, &mut handle), NuxStatus::Ok);
        let mut info = NuxViewModelCatalogInfo::default();
        assert_eq!(nux_view_model_catalog_info(handle, &mut info), NuxStatus::Ok);
        let mut out = Catalog::default();
        for i in 0..info.schema_count {
            let mut schema = NuxViewModelSchemaView::default();
            assert_eq!(nux_view_model_catalog_schema(handle, i, &mut schema), NuxStatus::Ok);
            out.schemas.push(text(schema.name));
            out.globals.push(schema.is_global != 0);
        }
        for i in 0..info.property_count {
            let mut p = NuxViewModelPropertyView::default();
            assert_eq!(nux_view_model_catalog_property(handle, i, &mut p), NuxStatus::Ok);
            out.props.push((
                p.schema_index,
                text(p.name),
                p.kind,
                p.first_enum_label,
                p.enum_label_count,
            ));
        }
        for i in 0..info.enum_label_count {
            let mut label = NuxStringView::default();
            assert_eq!(nux_view_model_catalog_enum_label(handle, i, &mut label), NuxStatus::Ok);
            out.labels.push(text(label));
        }
        nux_view_model_catalog_free(handle);
        out
    }
}

/// The iOS SDK's value markers (NuxieNativeValuePolicy.markers(in:)).
fn sdk_markers(catalog: &Catalog) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (index, schema) in catalog.schemas.iter().enumerate() {
        let props: Vec<_> = catalog.props.iter().filter(|p| p.0 == index).collect();
        for value in &props {
            if ![
                NUX_VIEW_MODEL_VALUE_KIND_NUMBER,
                NUX_VIEW_MODEL_VALUE_KIND_BOOL,
                NUX_VIEW_MODEL_VALUE_KIND_COLOR,
                NUX_VIEW_MODEL_VALUE_KIND_ENUM,
            ]
            .contains(&value.2)
            {
                continue;
            }
            let marker = match value.1.strip_prefix("state:") {
                Some(rest) => format!("state:isset:{rest}"),
                None => format!("isset:{}", value.1),
            };
            if let Some(m) = props.iter().find(|p| p.1 == marker) {
                assert_eq!(m.2, NUX_VIEW_MODEL_VALUE_KIND_BOOL);
                out.push((schema.clone(), value.1.clone(), marker));
            }
        }
    }
    out
}

struct Frame {
    status: NuxStatus,
    message: String,
    events: Vec<String>,
}

struct Page {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    root: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
    catalog: Catalog,
    rows: Vec<(String, *mut NuxViewModelInstance)>,
    time: f32,
}

impl Page {
    /// The page as the iOS SDK opens it: value markers, the env global with
    /// an iPhone 17 Pro's safe area, semantics and the layout size.
    unsafe fn open(bytes: &[u8], callbacks: &NuxRenderCallbacks) -> Self {
        unsafe {
            let mut file = ptr::null_mut();
            let mut imported = ptr::null_mut();
            let commands = NuxHostCommandImportConfig {
                module_name: sv("nuxie"),
                ..Default::default()
            };
            let status = nux_file_import_trusted_with_host_commands(
                bytes.as_ptr(),
                bytes.len(),
                callbacks,
                &commands,
                &mut file,
                &mut imported,
            );
            if !imported.is_null() {
                nux_capi_result_free(imported);
            }
            assert_eq!(status, NuxStatus::Ok, "import");
            let catalog = catalog(file);
            let markers = sdk_markers(&catalog);
            let entries: Vec<_> = markers
                .iter()
                .map(|(m, v, k)| NuxValueMarker {
                    model: sv(m),
                    value: sv(v),
                    marker: sv(k),
                })
                .collect();
            assert_eq!(
                nux_file_set_value_markers(file, entries.as_ptr(), entries.len()),
                NuxStatus::Ok
            );
            let mut artboard = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new_named(file, sv("$nuxie.page"), &mut artboard),
                NuxStatus::Ok
            );
            let mut root = ptr::null_mut();
            assert_eq!(nux_view_model_instance_new_default(artboard, &mut root), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_bind_view_model(artboard, root), NuxStatus::Ok);
            let mut player = ptr::null_mut();
            assert_eq!(
                nux_player_new_state_machine_named(artboard, sv("$nuxie.page"), &mut player),
                NuxStatus::Ok
            );
            let env = catalog
                .schemas
                .iter()
                .enumerate()
                .position(|(i, n)| n == "env" && catalog.globals[i])
                .expect("env global");
            let mut env_model = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_new_schema_default(file, env, &mut env_model),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_set_global_view_model(player, sv("env"), env_model),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            assert_eq!(nux_player_layout_size_set(player, 402.0, 874.0), NuxStatus::Ok);
            let page = Self {
                file,
                artboard,
                root,
                player,
                catalog,
                rows: Vec::new(),
                time: 0.0,
            };
            for (path, value) in [
                ("safeArea/top", 62.0f32),
                ("safeArea/bottom", 34.0),
                ("safeArea/left", 0.0),
                ("safeArea/right", 0.0),
                ("fontScale", 1.0),
            ] {
                assert_eq!(
                    page.mutate(&[NuxViewModelMutation {
                        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
                        instance: env_model,
                        path: sv(path),
                        number_value: value,
                        ..Default::default()
                    }]),
                    NuxStatus::Ok,
                    "env {path}"
                );
            }
            page
        }
    }
    unsafe fn mutate(&self, writes: &[NuxViewModelMutation]) -> NuxStatus {
        unsafe {
            let batch = NuxViewModelMutationBatch {
                mutations: writes.as_ptr(),
                mutation_count: writes.len(),
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            let status = nux_view_model_mutate(&batch, &mut result);
            if !result.is_null() {
                nux_view_model_mutation_result_free(result);
            }
            status
        }
    }
    /// The schema of a Screen's own model, read from its artboard's default.
    unsafe fn screen_schema(&self, screen: &str) -> usize {
        unsafe {
            let mut artboard = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new_named(self.file, sv(screen), &mut artboard),
                NuxStatus::Ok
            );
            let mut model = ptr::null_mut();
            assert_eq!(nux_view_model_instance_new_default(artboard, &mut model), NuxStatus::Ok);
            let mut snapshot = ptr::null_mut();
            assert_eq!(nux_view_model_instance_snapshot(model, &mut snapshot), NuxStatus::Ok);
            let mut info = NuxViewModelSnapshotInfo::default();
            assert_eq!(nux_view_model_snapshot_info(snapshot, &mut info), NuxStatus::Ok);
            let mut schema = None;
            for i in 0..info.instance_count {
                let mut instance = NuxViewModelSnapshotInstanceView::default();
                assert_eq!(
                    nux_view_model_snapshot_instance(snapshot, i, &mut instance),
                    NuxStatus::Ok
                );
                if instance.instance_id == info.root_instance_id {
                    schema = Some(instance.schema_index);
                }
            }
            nux_view_model_snapshot_free(snapshot);
            nux_view_model_instance_free(model);
            nux_artboard_instance_free(artboard);
            schema.expect("Screen root schema")
        }
    }
    /// ExperienceInteractivePage.prepareRow: a fresh model of the Screen's schema.
    unsafe fn prepare_row(&mut self, screen: &str) {
        unsafe {
            let schema = self.screen_schema(screen);
            let mut row = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_new_schema_default(self.file, schema, &mut row),
                NuxStatus::Ok
            );
            self.rows.push((screen.into(), row));
        }
    }
    fn row(&self, screen: &str) -> *mut NuxViewModelInstance {
        self.rows.iter().find(|r| r.0 == screen).unwrap().1
    }
    fn index(&self, screen: &str) -> u32 {
        self.rows.iter().position(|r| r.0 == screen).unwrap() as u32
    }
    unsafe fn insert(&self, screen: &str) {
        unsafe {
            assert_eq!(
                self.mutate(&[NuxViewModelMutation {
                    kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
                    instance: self.root,
                    path: sv("screens"),
                    related_instance: self.row(screen),
                    index: self.index(screen) as usize,
                    ..Default::default()
                }]),
                NuxStatus::Ok,
                "insert {screen}"
            );
        }
    }
    fn phase_label(&self, phase: &str) -> u64 {
        for p in &self.catalog.props {
            if p.1 == "phase" {
                for i in 0..p.4 {
                    if self.catalog.labels[p.3 + i] == phase {
                        return i as u64;
                    }
                }
            }
        }
        panic!("no phase label {phase}");
    }
    /// ExperienceScreenLifecycleSnapshot.stateCommand: phase, appearances, transition.
    unsafe fn phase(&self, screen: &str, phase: &str, appearances: f32, transition: &str) {
        unsafe {
            let row = self.row(screen);
            let status = self.mutate(&[
                NuxViewModelMutation {
                    kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_ENUM,
                    instance: row,
                    path: sv("screen/phase"),
                    integer_value: self.phase_label(phase),
                    ..Default::default()
                },
                NuxViewModelMutation {
                    kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
                    instance: row,
                    path: sv("screen/appearances"),
                    number_value: appearances,
                    ..Default::default()
                },
                NuxViewModelMutation {
                    kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
                    instance: row,
                    path: sv("screen/transition"),
                    bytes_value: NuxByteView {
                        data: transition.as_ptr(),
                        len: transition.len(),
                    },
                    ..Default::default()
                },
            ]);
            assert_eq!(status, NuxStatus::Ok, "{screen} {phase}");
        }
    }
    unsafe fn set_row_collapsed(&self, screen: &str, collapsed: bool) {
        unsafe {
            assert_eq!(
                nux_player_set_row_collapsed(
                    self.player,
                    sv("screens"),
                    self.index(screen),
                    u8::from(collapsed)
                ),
                NuxStatus::Ok,
                "{screen} collapsed={collapsed}"
            );
        }
    }
    unsafe fn step(&mut self, elapsed: f32, pointers: &[(u32, f32, f32)]) -> Frame {
        unsafe {
            self.time += elapsed;
            let events: Vec<_> = pointers
                .iter()
                .map(|&(kind, x, y)| NuxPlayerPointerEvent {
                    kind,
                    x,
                    y,
                    pointer_id: 0,
                    timestamp_seconds: self.time,
                })
                .collect();
            let step = NuxPlayerStep {
                pointers: if events.is_empty() {
                    ptr::null()
                } else {
                    events.as_ptr()
                },
                pointer_count: events.len(),
                elapsed_seconds: elapsed,
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            let status = nux_player_step(self.player, &step, &mut result);
            let mut frame = Frame {
                status,
                message: String::new(),
                events: Vec::new(),
            };
            if result.is_null() {
                return frame;
            }
            if status != NuxStatus::Ok {
                let mut diagnostic = NuxCapiDiagnosticView::default();
                if nux_player_step_result_diagnostic(result, &mut diagnostic) == NuxStatus::Ok {
                    frame.message =
                        format!("{}: {}", text(diagnostic.code), text(diagnostic.message));
                }
                nux_player_step_result_free(result);
                return frame;
            }
            let mut info = NuxPlayerStepInfo {
                struct_size: std::mem::size_of::<NuxPlayerStepInfo>() as u32,
                keep_going: false,
                pointer_result_count: 0,
                state_change_count: 0,
                event_count: 0,
                host_command_count: 0,
                view_model_change_count: 0,
                focus_input_result_count: 0,
                rule_report_count: 0,
            };
            assert_eq!(nux_player_step_result_info(result, &mut info), NuxStatus::Ok);
            for i in 0..info.event_count {
                let mut event = NuxPlayerEventView::default();
                assert_eq!(nux_player_step_result_event(result, i, &mut event), NuxStatus::Ok);
                frame.events.push(text(event.name));
            }
            nux_player_step_result_free(result);
            frame
        }
    }
    unsafe fn frame(&mut self, what: &str, elapsed: f32) -> Vec<String> {
        unsafe {
            let frame = self.step(elapsed, &[]);
            assert_eq!(frame.status, NuxStatus::Ok, "{what}: {}", frame.message);
            frame.events
        }
    }
    /// A tap and the two frames after it; returns the events they report.
    unsafe fn tap(&mut self, x: f32, y: f32) -> Vec<String> {
        unsafe {
            let mut heard = Vec::new();
            for pointers in [
                &[(NUX_PLAYER_POINTER_KIND_DOWN, x, y)][..],
                &[(NUX_PLAYER_POINTER_KIND_UP, x, y)][..],
                &[][..],
                &[][..],
            ] {
                let frame = self.step(1.0 / 60.0, pointers);
                assert_eq!(frame.status, NuxStatus::Ok, "tap: {}", frame.message);
                heard.extend(frame.events);
            }
            heard
        }
    }
}

/// The question Screen opens, then a Magic Move lands on the lesson, in the
/// iOS SDK's order (ExperienceScreenViewController.mountInteractiveScreen,
/// ExperiencePageStep.perform): insert, expand, a 0 s settle, both phases,
/// Begin, frames until both completion events, End, question hidden, question
/// asleep, lesson active.
unsafe fn land_on_lesson(callbacks: &NuxRenderCallbacks) -> Page {
    unsafe {
        let mut page = Page::open(DUO, callbacks);
        page.prepare_row("question");
        page.insert("question");
        page.phase("question", "entering", 0.0, "");
        for i in 0..3 {
            page.frame("question opens", if i == 0 { 0.0 } else { 1.0 / 60.0 });
        }
        page.phase("question", "active", 1.0, "");
        page.frame("question active", 1.0 / 60.0);
        page.prepare_row("lesson");
        page.insert("lesson");
        page.set_row_collapsed("lesson", false);
        page.frame("lesson settle", 0.0);
        page.phase("question", "exiting", 1.0, "step-1");
        page.phase("lesson", "entering", 0.0, "step-1");
        assert_eq!(
            nux_player_step_begin(page.player, sv("lesson"), &sv("step-1")),
            NuxStatus::Ok
        );
        let (mut exited, mut entered) = (false, false);
        for _ in 0..240 {
            for event in page.frame("flight", 1.0 / 60.0) {
                exited |= event.starts_with("nx_exit_done:");
                entered |= event.starts_with("nx_enter_done:");
            }
            if exited && entered {
                break;
            }
        }
        assert!(exited && entered, "the Magic Move completes");
        assert_eq!(
            nux_player_step_end(page.player, sv("lesson"), &sv("step-1")),
            NuxStatus::Ok
        );
        page.phase("question", "hidden", 1.0, "");
        page.set_row_collapsed("question", true);
        page.phase("lesson", "active", 1.0, "");
        page
    }
}

#[test]
fn a_carried_buttons_taps_reach_the_screen_they_landed_on() {
    // Control first: the same landing with the question woken again.
    for question_awake in [true, false] {
        unsafe {
            let mut page = land_on_lesson(&NuxRenderCallbacks::default());
            if question_awake {
                page.set_row_collapsed("question", false);
            }
            for _ in 0..30 {
                page.frame("landed", 1.0 / 60.0);
            }
            for (label, (x, y), expected) in [
                ("Continue", (201.0, 798.0), "continue"),
                ("Back", (30.0, 88.0), "back"),
            ] {
                let heard = page.tap(x, y);
                assert!(
                    heard.iter().any(|e| e == expected),
                    "question awake={question_awake}: the lesson's {label} reports {heard:?}"
                );
            }
        }
    }
}

// Drawing, recorded through the render callbacks: each drawn path's paint
// color, the opacity it was drawn under, and its local bounds.
#[derive(Default)]
struct Drawing {
    next: u64,
    paths: HashMap<u64, [f32; 4]>,
    colors: HashMap<u64, u32>,
    opacity: Vec<f32>,
    drawn: Vec<(u32, f32, [f32; 4])>,
}
thread_local! {
    static DRAWING: RefCell<Drawing> = RefCell::new(Drawing::default());
}
const EMPTY: [f32; 4] = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
fn grow(bounds: &mut [f32; 4], x: f32, y: f32) {
    bounds[0] = bounds[0].min(x);
    bounds[1] = bounds[1].min(y);
    bounds[2] = bounds[2].max(x);
    bounds[3] = bounds[3].max(y);
}
unsafe fn raw_points(raw: *const NuxRawPathView) -> Vec<(f32, f32)> {
    let Some(raw) = (unsafe { raw.as_ref() }) else {
        return Vec::new();
    };
    if raw.points.is_null() {
        return Vec::new();
    }
    let points = unsafe { std::slice::from_raw_parts(raw.points, raw.point_count * 2) };
    points.chunks(2).map(|p| (p[0], p[1])).collect()
}
fn new_handle(d: &mut Drawing) -> u64 {
    d.next += 1;
    d.next
}
unsafe extern "C" fn make_path(_: *mut std::ffi::c_void, raw: *const NuxRawPathView, _: u8) -> u64 {
    let points = unsafe { raw_points(raw) };
    DRAWING.with_borrow_mut(|d| {
        let handle = new_handle(d);
        let mut bounds = EMPTY;
        for (x, y) in points {
            grow(&mut bounds, x, y);
        }
        d.paths.insert(handle, bounds);
        handle
    })
}
unsafe extern "C" fn make_empty_path(_: *mut std::ffi::c_void) -> u64 {
    DRAWING.with_borrow_mut(|d| {
        let handle = new_handle(d);
        d.paths.insert(handle, EMPTY);
        handle
    })
}
unsafe extern "C" fn make_handle(_: *mut std::ffi::c_void) -> u64 {
    DRAWING.with_borrow_mut(new_handle)
}
unsafe extern "C" fn make_gradient_linear(
    _: *mut std::ffi::c_void,
    _: f32,
    _: f32,
    _: f32,
    _: f32,
    _: *const u32,
    _: *const f32,
    _: usize,
) -> u64 {
    DRAWING.with_borrow_mut(new_handle)
}
unsafe extern "C" fn make_gradient_radial(
    _: *mut std::ffi::c_void,
    _: f32,
    _: f32,
    _: f32,
    _: *const u32,
    _: *const f32,
    _: usize,
) -> u64 {
    DRAWING.with_borrow_mut(new_handle)
}
unsafe extern "C" fn release(_: *mut std::ffi::c_void, _: u64) {}
unsafe extern "C" fn rewind(_: *mut std::ffi::c_void, path: u64) {
    DRAWING.with_borrow_mut(|d| {
        d.paths.insert(path, EMPTY);
    });
}
unsafe extern "C" fn point(_: *mut std::ffi::c_void, path: u64, x: f32, y: f32) {
    DRAWING.with_borrow_mut(|d| grow(d.paths.entry(path).or_insert(EMPTY), x, y));
}
#[allow(clippy::too_many_arguments)]
unsafe extern "C" fn cubic(
    _: *mut std::ffi::c_void,
    path: u64,
    ox: f32,
    oy: f32,
    ix: f32,
    iy: f32,
    x: f32,
    y: f32,
) {
    DRAWING.with_borrow_mut(|d| {
        let bounds = d.paths.entry(path).or_insert(EMPTY);
        for (x, y) in [(ox, oy), (ix, iy), (x, y)] {
            grow(bounds, x, y);
        }
    });
}
unsafe extern "C" fn add_raw(_: *mut std::ffi::c_void, path: u64, raw: *const NuxRawPathView) {
    let points = unsafe { raw_points(raw) };
    DRAWING.with_borrow_mut(|d| {
        let bounds = d.paths.entry(path).or_insert(EMPTY);
        for (x, y) in points {
            grow(bounds, x, y);
        }
    });
}
unsafe extern "C" fn add_path(_: *mut std::ffi::c_void, path: u64, source: u64, m: *const f32) {
    let m = if m.is_null() {
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    } else {
        unsafe { *m.cast::<[f32; 6]>() }
    };
    DRAWING.with_borrow_mut(|d| {
        let s = d.paths.get(&source).copied().unwrap_or(EMPTY);
        if s[0] > s[2] {
            return;
        }
        let bounds = d.paths.entry(path).or_insert(EMPTY);
        for (x, y) in [(s[0], s[1]), (s[2], s[1]), (s[0], s[3]), (s[2], s[3])] {
            grow(
                bounds,
                m[0] * x + m[2] * y + m[4],
                m[1] * x + m[3] * y + m[5],
            );
        }
    });
}
unsafe extern "C" fn paint_color(_: *mut std::ffi::c_void, paint: u64, color: u32) {
    DRAWING.with_borrow_mut(|d| {
        d.colors.insert(paint, color);
    });
}
unsafe extern "C" fn save(_: *mut std::ffi::c_void) {
    DRAWING.with_borrow_mut(|d| {
        let top = d.opacity.last().copied().unwrap_or(1.0);
        d.opacity.push(top);
    });
}
unsafe extern "C" fn restore(_: *mut std::ffi::c_void) {
    DRAWING.with_borrow_mut(|d| {
        d.opacity.pop();
    });
}
unsafe extern "C" fn modulate_opacity(_: *mut std::ffi::c_void, opacity: f32) {
    DRAWING.with_borrow_mut(|d| {
        if d.opacity.is_empty() {
            d.opacity.push(1.0);
        }
        *d.opacity.last_mut().unwrap() *= opacity;
    });
}
unsafe extern "C" fn draw_path(_: *mut std::ffi::c_void, path: u64, paint: u64) {
    DRAWING.with_borrow_mut(|d| {
        let color = d.colors.get(&paint).copied().unwrap_or(0);
        let opacity = d.opacity.last().copied().unwrap_or(1.0);
        let bounds = d.paths.get(&path).copied().unwrap_or(EMPTY);
        d.drawn.push((color, opacity, bounds));
    });
}
fn recording_callbacks() -> NuxRenderCallbacks {
    NuxRenderCallbacks {
        make_render_path: Some(make_path),
        make_empty_render_path: Some(make_empty_path),
        make_render_paint: Some(make_handle),
        make_linear_gradient: Some(make_gradient_linear),
        make_radial_gradient: Some(make_gradient_radial),
        release_render_path: Some(release),
        release_render_paint: Some(release),
        release_render_shader: Some(release),
        release_render_buffer: Some(release),
        release_render_image: Some(release),
        render_path_rewind: Some(rewind),
        render_path_move_to: Some(point),
        render_path_line_to: Some(point),
        render_path_cubic_to: Some(cubic),
        render_path_add_raw_path: Some(add_raw),
        render_path_add_render_path: Some(add_path),
        render_path_add_render_path_backwards: Some(add_path),
        render_paint_color: Some(paint_color),
        save: Some(save),
        restore: Some(restore),
        modulate_opacity: Some(modulate_opacity),
        draw_path: Some(draw_path),
        ..NuxRenderCallbacks::default()
    }
}

/// Draws the page and reports whether Duo's cheering pose shows. Only that
/// pose draws the raised wing's yellow dot (`#ffc800`, a circle 10 units
/// across); the standing pose has no yellow, and the question's yellow is a
/// 22 by 24 icon.
unsafe fn cheer_shows(page: &Page) -> (bool, Vec<String>) {
    DRAWING.with_borrow_mut(|d| {
        d.drawn.clear();
        d.opacity.clear();
    });
    assert_eq!(unsafe { nux_artboard_instance_draw(page.artboard) }, NuxStatus::Ok);
    DRAWING.with_borrow(|d| {
        let yellow: Vec<_> = d
            .drawn
            .iter()
            .filter(|(color, _, _)| color & 0x00ff_ffff == 0x00ff_c800)
            .collect();
        let shows = yellow.iter().any(|&&(color, opacity, b)| {
            let (width, height) = (b[2] - b[0], b[3] - b[1]);
            (color >> 24) as f32 / 255.0 * opacity > 0.5
                && (8.0..=14.0).contains(&width)
                && (8.0..=14.0).contains(&height)
        });
        let seen = yellow
            .iter()
            .map(|(color, opacity, b)| {
                format!(
                    "{color:#010x} at opacity {opacity}, {} by {}",
                    b[2] - b[0],
                    b[3] - b[1]
                )
            })
            .collect();
        (shows, seen)
    })
}

#[test]
fn a_carried_mascot_takes_its_new_screens_pose() {
    // Control first: the same landing with the question woken again.
    for question_awake in [true, false] {
        unsafe {
            let mut page = land_on_lesson(&recording_callbacks());
            if question_awake {
                page.set_row_collapsed("question", false);
            }
            // The lesson's appear sets its pose to cheer; Duo holds 100 ms
            // in the standing pose, then cheers.
            for _ in 0..20 {
                page.frame("after the lesson's appear", 1.0 / 60.0);
            }
            let (shows, seen) = cheer_shows(&page);
            assert!(
                shows,
                "question awake={question_awake}: Duo cheers 20 frames after the lesson's appear; yellow drawn: {seen:?}"
            );
        }
    }
}
