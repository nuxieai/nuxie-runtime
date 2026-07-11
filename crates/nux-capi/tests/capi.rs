use nux_capi::{
    NuxArtboardInstance, NuxFile, NuxRenderCache, NuxRenderCallbacks, NuxStateMachineInstance,
    NuxStatus, NuxStringView, NuxViewModelInstance, nux_artboard_instance_advance,
    nux_artboard_instance_bind_view_model, nux_artboard_instance_draw,
    nux_artboard_instance_draw_cached, nux_artboard_instance_free, nux_artboard_instance_new,
    nux_file_artboard_animation_count, nux_file_artboard_count, nux_file_artboard_name,
    nux_file_artboard_state_machine_count, nux_file_artboard_state_machine_name, nux_file_free,
    nux_file_import, nux_render_cache_free, nux_render_cache_new,
    nux_state_machine_instance_advance, nux_state_machine_instance_fire_trigger,
    nux_state_machine_instance_free, nux_state_machine_instance_new,
    nux_state_machine_instance_new_default, nux_state_machine_instance_pointer_down,
    nux_state_machine_instance_pointer_move, nux_state_machine_instance_pointer_up,
    nux_state_machine_instance_set_bool, nux_state_machine_instance_set_number,
    nux_view_model_instance_free, nux_view_model_instance_new_default,
    nux_view_model_instance_new_instance, nux_view_model_instance_set_bool,
    nux_view_model_instance_set_number, nux_view_model_instance_set_string,
};
use std::ffi::{CString, c_void};
use std::path::PathBuf;

fn fixture_bytes(name: &str) -> Vec<u8> {
    let fixture = PathBuf::from(
        std::env::var_os("NUX_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    std::fs::read(&fixture).expect("read fixture")
}

fn repo_fixture_bytes(relative: &str) -> Vec<u8> {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read(&fixture).expect("read repo fixture")
}

fn import_repo_fixture(relative: &str) -> *mut NuxFile {
    let bytes = repo_fixture_bytes(relative);
    let mut file: *mut NuxFile = std::ptr::null_mut();
    let status = unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) };
    assert_eq!(status, NuxStatus::Ok);
    assert!(!file.is_null());
    file
}

fn string_view_to_owned(view: NuxStringView) -> String {
    assert!(!view.data.is_null());
    let bytes = unsafe { std::slice::from_raw_parts(view.data.cast::<u8>(), view.len) };
    std::str::from_utf8(bytes).expect("utf8 name").to_owned()
}

#[test]
fn c_api_imports_file_and_exposes_artboard_metadata() {
    let bytes = fixture_bytes("shapetest.riv");
    let mut file: *mut NuxFile = std::ptr::null_mut();
    let status = unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) };
    assert_eq!(status, NuxStatus::Ok);
    assert!(!file.is_null());

    let artboard_count = unsafe { nux_file_artboard_count(file) };
    assert_eq!(artboard_count, 1);

    let mut name = NuxStringView {
        data: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { nux_file_artboard_name(file, 0, &mut name) };
    assert_eq!(status, NuxStatus::Ok);
    assert!(!name.data.is_null());
    let name_bytes = unsafe { std::slice::from_raw_parts(name.data.cast::<u8>(), name.len) };
    assert_eq!(std::str::from_utf8(name_bytes).unwrap(), "New Artboard");

    let mut animation_count = usize::MAX;
    let status = unsafe { nux_file_artboard_animation_count(file, 0, &mut animation_count) };
    assert_eq!(status, NuxStatus::Ok);
    assert_ne!(animation_count, usize::MAX);

    let mut state_machine_count = usize::MAX;
    let status =
        unsafe { nux_file_artboard_state_machine_count(file, 0, &mut state_machine_count) };
    assert_eq!(status, NuxStatus::Ok);
    assert_ne!(state_machine_count, usize::MAX);

    let missing = unsafe { nux_file_artboard_name(file, 99, &mut name) };
    assert_eq!(missing, NuxStatus::NotFound);

    unsafe {
        nux_file_free(file);
    }
}

#[test]
fn c_api_rejects_null_arguments_without_writing_handles() {
    let bytes = fixture_bytes("shapetest.riv");
    let status = unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), std::ptr::null_mut()) };
    assert_eq!(status, NuxStatus::NullArgument);

    let mut file: *mut NuxFile = std::ptr::dangling_mut();
    let status = unsafe { nux_file_import(std::ptr::null(), bytes.len(), &mut file) };
    assert_eq!(status, NuxStatus::NullArgument);
    assert!(file.is_null());
}

const SMI_FIXTURE: &str = "fixtures/animation/smi_test.riv";
/// Artboard index of "artboard to nest", whose default state machine has
/// bool/number/trigger inputs named "bool", "num", and "trig".
const SMI_INPUT_ARTBOARD: usize = 1;

#[test]
fn c_api_runs_embed_loop_with_state_machine_inputs() {
    let file = import_repo_fixture(SMI_FIXTURE);

    let mut state_machine_count = 0usize;
    let status = unsafe {
        nux_file_artboard_state_machine_count(file, SMI_INPUT_ARTBOARD, &mut state_machine_count)
    };
    assert_eq!(status, NuxStatus::Ok);
    assert!(state_machine_count >= 1);

    let mut name = NuxStringView::default();
    let status =
        unsafe { nux_file_artboard_state_machine_name(file, SMI_INPUT_ARTBOARD, 0, &mut name) };
    assert_eq!(status, NuxStatus::Ok);
    assert_eq!(string_view_to_owned(name), "State Machine 1");
    let status =
        unsafe { nux_file_artboard_state_machine_name(file, SMI_INPUT_ARTBOARD, 99, &mut name) };
    assert_eq!(status, NuxStatus::NotFound);

    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    let status = unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, NuxStatus::Ok);
    assert!(!instance.is_null());

    let mut state_machine: *mut NuxStateMachineInstance = std::ptr::null_mut();
    let status = unsafe { nux_state_machine_instance_new_default(instance, &mut state_machine) };
    assert_eq!(status, NuxStatus::Ok);
    assert!(!state_machine.is_null());

    let bool_name = CString::new("bool").unwrap();
    let num_name = CString::new("num").unwrap();
    let trig_name = CString::new("trig").unwrap();
    let missing_name = CString::new("nope").unwrap();

    let status =
        unsafe { nux_state_machine_instance_set_bool(state_machine, bool_name.as_ptr(), true) };
    assert_eq!(status, NuxStatus::Ok);
    let status =
        unsafe { nux_state_machine_instance_set_number(state_machine, num_name.as_ptr(), 42.0) };
    assert_eq!(status, NuxStatus::Ok);
    let status =
        unsafe { nux_state_machine_instance_fire_trigger(state_machine, trig_name.as_ptr()) };
    assert_eq!(status, NuxStatus::Ok);

    // Missing input name vs. wrong input kind report distinct statuses.
    let status =
        unsafe { nux_state_machine_instance_set_bool(state_machine, missing_name.as_ptr(), true) };
    assert_eq!(status, NuxStatus::NotFound);
    let status =
        unsafe { nux_state_machine_instance_set_number(state_machine, bool_name.as_ptr(), 1.0) };
    assert_eq!(status, NuxStatus::InvalidArgument);

    let mut changed = false;
    let status =
        unsafe { nux_state_machine_instance_advance(instance, state_machine, 0.016, &mut changed) };
    assert_eq!(status, NuxStatus::Ok);
    // The advance out-param is optional.
    let status = unsafe {
        nux_state_machine_instance_advance(instance, state_machine, 0.016, std::ptr::null_mut())
    };
    assert_eq!(status, NuxStatus::Ok);

    unsafe {
        nux_state_machine_instance_free(state_machine);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn c_api_state_machine_instance_by_index_and_missing_index() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    let status = unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, NuxStatus::Ok);

    let mut state_machine: *mut NuxStateMachineInstance = std::ptr::null_mut();
    let status = unsafe { nux_state_machine_instance_new(instance, 0, &mut state_machine) };
    assert_eq!(status, NuxStatus::Ok);
    assert!(!state_machine.is_null());
    unsafe { nux_state_machine_instance_free(state_machine) };

    let status = unsafe { nux_state_machine_instance_new(instance, 99, &mut state_machine) };
    assert_eq!(status, NuxStatus::NotFound);
    assert!(state_machine.is_null());

    unsafe {
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[derive(Default)]
struct RenderCounters {
    next_handle: u64,
    made: usize,
    released: usize,
    paint_colors: usize,
    draw_paths: usize,
    saves: usize,
    restores: usize,
    transforms: usize,
}

unsafe fn counters<'a>(user_data: *mut c_void) -> &'a mut RenderCounters {
    unsafe { &mut *user_data.cast::<RenderCounters>() }
}

unsafe extern "C" fn counting_make_render_path(
    user_data: *mut c_void,
    path: *const nux_capi::NuxRawPathView,
    _fill_rule: u8,
) -> u64 {
    assert!(!path.is_null());
    let counters = unsafe { counters(user_data) };
    counters.made += 1;
    counters.next_handle += 1;
    counters.next_handle
}

unsafe extern "C" fn counting_make_handle(user_data: *mut c_void) -> u64 {
    let counters = unsafe { counters(user_data) };
    counters.made += 1;
    counters.next_handle += 1;
    counters.next_handle
}

unsafe extern "C" fn counting_release(user_data: *mut c_void, handle: u64) {
    assert_ne!(handle, 0);
    let counters = unsafe { counters(user_data) };
    counters.released += 1;
}

unsafe extern "C" fn counting_paint_color(user_data: *mut c_void, paint: u64, _color: u32) {
    assert_ne!(paint, 0);
    let counters = unsafe { counters(user_data) };
    counters.paint_colors += 1;
}

unsafe extern "C" fn counting_draw_path(user_data: *mut c_void, path: u64, paint: u64) {
    assert_ne!(path, 0);
    assert_ne!(paint, 0);
    let counters = unsafe { counters(user_data) };
    counters.draw_paths += 1;
}

unsafe extern "C" fn counting_save(user_data: *mut c_void) {
    unsafe { counters(user_data) }.saves += 1;
}

unsafe extern "C" fn counting_restore(user_data: *mut c_void) {
    unsafe { counters(user_data) }.restores += 1;
}

unsafe extern "C" fn counting_transform(user_data: *mut c_void, transform: *const f32) {
    assert!(!transform.is_null());
    unsafe { counters(user_data) }.transforms += 1;
}

#[test]
fn c_api_draw_forwards_render_calls_to_vtable() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    let status = unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, NuxStatus::Ok);
    let status = unsafe { nux_artboard_instance_advance(instance, 0.0, std::ptr::null_mut()) };
    assert_eq!(status, NuxStatus::Ok);

    let mut counters_data = RenderCounters::default();
    let callbacks = NuxRenderCallbacks {
        user_data: (&mut counters_data as *mut RenderCounters).cast::<c_void>(),
        make_render_path: Some(counting_make_render_path),
        make_empty_render_path: Some(counting_make_handle),
        make_render_paint: Some(counting_make_handle),
        release_render_path: Some(counting_release),
        release_render_paint: Some(counting_release),
        release_render_shader: Some(counting_release),
        render_paint_color: Some(counting_paint_color),
        draw_path: Some(counting_draw_path),
        save: Some(counting_save),
        restore: Some(counting_restore),
        transform: Some(counting_transform),
        ..NuxRenderCallbacks::default()
    };

    let status = unsafe { nux_artboard_instance_draw(instance, &callbacks) };
    assert_eq!(status, NuxStatus::Ok);

    assert!(counters_data.draw_paths > 0, "expected draw_path calls");
    assert!(counters_data.paint_colors > 0, "expected paint color calls");
    assert!(counters_data.saves > 0);
    assert_eq!(counters_data.saves, counters_data.restores);
    assert!(counters_data.transforms > 0);
    assert!(counters_data.made > 0);
    assert_eq!(
        counters_data.made, counters_data.released,
        "every created render object must be released exactly once"
    );

    unsafe {
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn c_api_draw_with_empty_vtable_behaves_like_null_renderer() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    let status = unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, NuxStatus::Ok);

    let callbacks = NuxRenderCallbacks::default();
    let status = unsafe { nux_artboard_instance_draw(instance, &callbacks) };
    assert_eq!(status, NuxStatus::Ok);

    unsafe {
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn c_api_retained_draw_reuses_and_releases_render_handles() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) },
        NuxStatus::Ok
    );

    let mut counters_data = RenderCounters::default();
    let callbacks = NuxRenderCallbacks {
        user_data: (&mut counters_data as *mut RenderCounters).cast::<c_void>(),
        make_render_path: Some(counting_make_render_path),
        make_empty_render_path: Some(counting_make_handle),
        make_render_paint: Some(counting_make_handle),
        release_render_path: Some(counting_release),
        release_render_paint: Some(counting_release),
        release_render_shader: Some(counting_release),
        draw_path: Some(counting_draw_path),
        ..NuxRenderCallbacks::default()
    };
    let mut cache: *mut NuxRenderCache = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_render_cache_new(instance, &callbacks, &mut cache) },
        NuxStatus::Ok
    );
    assert!(!cache.is_null());

    assert_eq!(
        unsafe { nux_artboard_instance_draw_cached(instance, cache) },
        NuxStatus::Ok
    );
    let made_after_first_draw = counters_data.made;
    let released_after_first_draw = counters_data.released;
    assert!(made_after_first_draw > 0);
    assert_eq!(
        unsafe { nux_artboard_instance_draw_cached(instance, cache) },
        NuxStatus::Ok
    );
    assert_eq!(counters_data.made, made_after_first_draw);
    assert_eq!(counters_data.released, released_after_first_draw);

    unsafe { nux_render_cache_free(cache) };
    assert_eq!(counters_data.released, counters_data.made);
    unsafe {
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn c_api_instance_functions_reject_null_arguments() {
    let file = import_repo_fixture(SMI_FIXTURE);

    let status =
        unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, std::ptr::null_mut()) };
    assert_eq!(status, NuxStatus::NullArgument);

    let mut instance: *mut NuxArtboardInstance = std::ptr::dangling_mut();
    let status = unsafe { nux_artboard_instance_new(std::ptr::null(), 0, &mut instance) };
    assert_eq!(status, NuxStatus::NullArgument);
    assert!(instance.is_null());

    let status = unsafe { nux_artboard_instance_new(file, 99, &mut instance) };
    assert_eq!(status, NuxStatus::NotFound);
    assert!(instance.is_null());

    let status =
        unsafe { nux_artboard_instance_advance(std::ptr::null_mut(), 0.0, std::ptr::null_mut()) };
    assert_eq!(status, NuxStatus::NullArgument);

    let status = unsafe { nux_artboard_instance_draw(std::ptr::null_mut(), std::ptr::null()) };
    assert_eq!(status, NuxStatus::NullArgument);

    let mut state_machine: *mut NuxStateMachineInstance = std::ptr::dangling_mut();
    let status = unsafe { nux_state_machine_instance_new(std::ptr::null(), 0, &mut state_machine) };
    assert_eq!(status, NuxStatus::NullArgument);
    assert!(state_machine.is_null());

    let name = CString::new("bool").unwrap();
    let status =
        unsafe { nux_state_machine_instance_set_bool(std::ptr::null_mut(), name.as_ptr(), true) };
    assert_eq!(status, NuxStatus::NullArgument);

    unsafe {
        nux_file_free(file);
    }
}

#[test]
fn c_api_pointer_events_dispatch_through_state_machine() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    let status = unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, NuxStatus::Ok);

    let mut state_machine: *mut NuxStateMachineInstance = std::ptr::null_mut();
    let status = unsafe { nux_state_machine_instance_new_default(instance, &mut state_machine) };
    assert_eq!(status, NuxStatus::Ok);

    // Settle the state machine before delivering input, mirroring an embed
    // loop's first frame.
    let status = unsafe {
        nux_state_machine_instance_advance(instance, state_machine, 0.016, std::ptr::null_mut())
    };
    assert_eq!(status, NuxStatus::Ok);

    // out_hit is optional and must always be (re)initialized when provided.
    let mut hit = true;
    let status = unsafe {
        nux_state_machine_instance_pointer_down(instance, state_machine, 10.0, 10.0, &mut hit)
    };
    assert_eq!(status, NuxStatus::Ok);
    // This artboard has no listeners, so nothing is hit.
    assert!(!hit);

    let status = unsafe {
        nux_state_machine_instance_pointer_move(
            instance,
            state_machine,
            12.0,
            12.0,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(status, NuxStatus::Ok);

    let mut hit = true;
    let status = unsafe {
        nux_state_machine_instance_pointer_up(instance, state_machine, 12.0, 12.0, &mut hit)
    };
    assert_eq!(status, NuxStatus::Ok);
    assert!(!hit);

    // The state machine still advances cleanly after pointer traffic.
    let status = unsafe {
        nux_state_machine_instance_advance(instance, state_machine, 0.016, std::ptr::null_mut())
    };
    assert_eq!(status, NuxStatus::Ok);

    unsafe {
        nux_state_machine_instance_free(state_machine);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn c_api_pointer_events_reject_null_arguments() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    let status = unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, NuxStatus::Ok);
    let mut state_machine: *mut NuxStateMachineInstance = std::ptr::null_mut();
    let status = unsafe { nux_state_machine_instance_new_default(instance, &mut state_machine) };
    assert_eq!(status, NuxStatus::Ok);

    let mut hit = true;
    let status = unsafe {
        nux_state_machine_instance_pointer_down(std::ptr::null(), state_machine, 0.0, 0.0, &mut hit)
    };
    assert_eq!(status, NuxStatus::NullArgument);
    assert!(!hit, "out_hit must be reset even on error");

    let status = unsafe {
        nux_state_machine_instance_pointer_move(
            instance,
            std::ptr::null_mut(),
            0.0,
            0.0,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(status, NuxStatus::NullArgument);

    let status = unsafe {
        nux_state_machine_instance_pointer_up(
            std::ptr::null(),
            std::ptr::null_mut(),
            0.0,
            0.0,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(status, NuxStatus::NullArgument);

    unsafe {
        nux_state_machine_instance_free(state_machine);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

/// Panic firewall coverage: every `extern "C"` entry point in the crate must
/// route its body through `ffi_guard` so a panic can never unwind into C.
/// This scans the source so a newly added export cannot silently skip the
/// firewall. (The behavioral half — a deliberately panicking path returning
/// the error status in debug — lives in the crate's `firewall_tests` module.)
#[test]
fn every_extern_c_export_is_panic_firewalled() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"),
    )
    .expect("read nux-capi src/lib.rs");

    let mut checked = 0usize;
    for (index, _) in source.match_indices("pub unsafe extern \"C\" fn ") {
        let rest = &source[index..];
        let body_start = rest.find('{').expect("extern fn has a body");
        let body = rest[body_start + 1..].trim_start();
        let name_end = rest.find('(').expect("extern fn has parameters");
        let name = &rest["pub unsafe extern \"C\" fn ".len()..name_end];
        assert!(
            body.starts_with("ffi_guard("),
            "extern \"C\" fn `{name}` does not open with the ffi_guard panic firewall"
        );
        checked += 1;
    }
    assert!(
        checked >= 18,
        "expected to scan all extern C exports, found only {checked}"
    );
}

/// External databind fixture whose artboard 0 binds a shape to the view model's
/// `num` number property, so a set changes the drawn geometry.
const VM_FIXTURE: &str = "data_binding_test_2.riv";

#[derive(Default)]
struct GeometryProbe {
    next_handle: u64,
    /// Position-weighted checksum of every geometry/paint float observed while
    /// drawing, so any change to the rendered output surfaces as a new value.
    checksum: f64,
    samples: u64,
    draw_paths: usize,
}

impl GeometryProbe {
    fn mix(&mut self, value: f64) {
        self.samples += 1;
        self.checksum += value * (self.samples as f64);
    }
}

unsafe fn probe<'a>(user_data: *mut c_void) -> &'a mut GeometryProbe {
    unsafe { &mut *user_data.cast::<GeometryProbe>() }
}

unsafe extern "C" fn probe_make_render_path(
    user_data: *mut c_void,
    path: *const nux_capi::NuxRawPathView,
    fill_rule: u8,
) -> u64 {
    let probe = unsafe { probe(user_data) };
    let view = unsafe { &*path };
    probe.mix(f64::from(fill_rule));
    if view.point_count != 0 {
        let points = unsafe { std::slice::from_raw_parts(view.points, view.point_count * 2) };
        for value in points {
            probe.mix(f64::from(*value));
        }
    }
    probe.next_handle += 1;
    probe.next_handle
}

unsafe extern "C" fn probe_make_handle(user_data: *mut c_void) -> u64 {
    let probe = unsafe { probe(user_data) };
    probe.next_handle += 1;
    probe.next_handle
}

unsafe extern "C" fn probe_release(_user_data: *mut c_void, _handle: u64) {}

unsafe extern "C" fn probe_move_to(user_data: *mut c_void, _path: u64, x: f32, y: f32) {
    let probe = unsafe { probe(user_data) };
    probe.mix(f64::from(x));
    probe.mix(f64::from(y));
}

unsafe extern "C" fn probe_line_to(user_data: *mut c_void, _path: u64, x: f32, y: f32) {
    let probe = unsafe { probe(user_data) };
    probe.mix(f64::from(x));
    probe.mix(f64::from(y));
}

unsafe extern "C" fn probe_cubic_to(
    user_data: *mut c_void,
    _path: u64,
    ox: f32,
    oy: f32,
    ix: f32,
    iy: f32,
    x: f32,
    y: f32,
) {
    let probe = unsafe { probe(user_data) };
    for value in [ox, oy, ix, iy, x, y] {
        probe.mix(f64::from(value));
    }
}

unsafe extern "C" fn probe_paint_color(user_data: *mut c_void, _paint: u64, color: u32) {
    unsafe { probe(user_data) }.mix(f64::from(color));
}

unsafe extern "C" fn probe_transform(user_data: *mut c_void, transform: *const f32) {
    let probe = unsafe { probe(user_data) };
    let values = unsafe { std::slice::from_raw_parts(transform, 6) };
    for value in values {
        probe.mix(f64::from(*value));
    }
}

unsafe extern "C" fn probe_draw_path(user_data: *mut c_void, _path: u64, _paint: u64) {
    unsafe { probe(user_data) }.draw_paths += 1;
}

fn probe_callbacks(probe: &mut GeometryProbe) -> NuxRenderCallbacks {
    NuxRenderCallbacks {
        user_data: (probe as *mut GeometryProbe).cast::<c_void>(),
        make_render_path: Some(probe_make_render_path),
        make_empty_render_path: Some(probe_make_handle),
        make_render_paint: Some(probe_make_handle),
        release_render_path: Some(probe_release),
        release_render_paint: Some(probe_release),
        release_render_shader: Some(probe_release),
        render_path_move_to: Some(probe_move_to),
        render_path_line_to: Some(probe_line_to),
        render_path_cubic_to: Some(probe_cubic_to),
        render_paint_color: Some(probe_paint_color),
        transform: Some(probe_transform),
        draw_path: Some(probe_draw_path),
        ..NuxRenderCallbacks::default()
    }
}

/// Import a fixture, instantiate artboard 0, optionally set `num`, bind, advance
/// and draw, returning the geometry checksum captured through the render vtable.
fn draw_geometry_checksum(number: Option<f32>) -> f64 {
    let bytes = fixture_bytes(VM_FIXTURE);
    let mut file: *mut NuxFile = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) },
        NuxStatus::Ok
    );

    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut instance) },
        NuxStatus::Ok
    );

    let mut view_model: *mut NuxViewModelInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(instance, &mut view_model) },
        NuxStatus::Ok
    );
    assert!(!view_model.is_null());

    if let Some(value) = number {
        let name = CString::new("num").unwrap();
        assert_eq!(
            unsafe { nux_view_model_instance_set_number(view_model, name.as_ptr(), value) },
            NuxStatus::Ok
        );
    }

    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(instance, view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_advance(instance, 0.0, std::ptr::null_mut()) },
        NuxStatus::Ok
    );

    let mut probe = GeometryProbe::default();
    let callbacks = probe_callbacks(&mut probe);
    assert_eq!(
        unsafe { nux_artboard_instance_draw(instance, &callbacks) },
        NuxStatus::Ok
    );
    assert!(probe.draw_paths > 0, "expected the artboard to draw paths");

    unsafe {
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
    probe.checksum
}

#[test]
fn c_api_view_model_number_set_changes_drawn_geometry() {
    let baseline = draw_geometry_checksum(None);
    let mutated = draw_geometry_checksum(Some(137.0));
    assert_ne!(
        baseline, mutated,
        "setting a bound number property must change the drawn geometry"
    );
}

#[test]
fn c_api_view_model_setters_report_status_codes() {
    let bytes = fixture_bytes(VM_FIXTURE);
    let mut file: *mut NuxFile = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) },
        NuxStatus::Ok
    );
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut instance) },
        NuxStatus::Ok
    );

    let mut view_model: *mut NuxViewModelInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(instance, &mut view_model) },
        NuxStatus::Ok
    );
    // Instance-by-index selection also works for a fixture with a source
    // instance, and out-of-range indices report NOT_FOUND.
    let mut by_index: *mut NuxViewModelInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_instance(instance, 0, &mut by_index) },
        NuxStatus::Ok
    );
    unsafe { nux_view_model_instance_free(by_index) };
    let mut missing_index: *mut NuxViewModelInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_instance(instance, 9_999, &mut missing_index) },
        NuxStatus::NotFound
    );
    assert!(missing_index.is_null());

    let num = CString::new("num").unwrap();
    let missing = CString::new("does-not-exist").unwrap();
    let value = CString::new("hello").unwrap();

    // A real number property sets OK; a wrong-kind or missing path is NOT_FOUND.
    assert_eq!(
        unsafe { nux_view_model_instance_set_number(view_model, num.as_ptr(), 5.0) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_set_number(view_model, missing.as_ptr(), 5.0) },
        NuxStatus::NotFound
    );
    assert_eq!(
        unsafe { nux_view_model_instance_set_bool(view_model, num.as_ptr(), true) },
        NuxStatus::NotFound
    );
    assert_eq!(
        unsafe { nux_view_model_instance_set_string(view_model, num.as_ptr(), value.as_ptr()) },
        NuxStatus::NotFound
    );

    // Null-argument handling on the ABI boundary.
    assert_eq!(
        unsafe { nux_view_model_instance_set_number(std::ptr::null_mut(), num.as_ptr(), 1.0) },
        NuxStatus::NullArgument
    );
    assert_eq!(
        unsafe { nux_view_model_instance_set_number(view_model, std::ptr::null(), 1.0) },
        NuxStatus::NullArgument
    );
    assert_eq!(
        unsafe { nux_view_model_instance_set_string(view_model, num.as_ptr(), std::ptr::null()) },
        NuxStatus::NullArgument
    );

    unsafe {
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn c_api_view_model_absent_reports_not_found() {
    // smi_test.riv artboards carry the -1 "no view model" sentinel.
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut NuxArtboardInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) },
        NuxStatus::Ok
    );
    let mut view_model: *mut NuxViewModelInstance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(instance, &mut view_model) },
        NuxStatus::NotFound
    );
    assert!(view_model.is_null());
    // Null out-pointer is rejected before anything else.
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(instance, std::ptr::null_mut()) },
        NuxStatus::NullArgument
    );

    unsafe {
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}
