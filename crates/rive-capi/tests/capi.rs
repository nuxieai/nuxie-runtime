use rive_capi::{
    RiveArtboardInstance, RiveFile, RiveRenderCallbacks, RiveStateMachineInstance, RiveStatus,
    RiveStringView, rive_artboard_instance_advance, rive_artboard_instance_draw,
    rive_artboard_instance_free, rive_artboard_instance_new, rive_file_artboard_animation_count,
    rive_file_artboard_count, rive_file_artboard_name, rive_file_artboard_state_machine_count,
    rive_file_artboard_state_machine_name, rive_file_free, rive_file_import,
    rive_state_machine_instance_advance, rive_state_machine_instance_fire_trigger,
    rive_state_machine_instance_free, rive_state_machine_instance_new,
    rive_state_machine_instance_new_default, rive_state_machine_instance_set_bool,
    rive_state_machine_instance_set_number,
};
use std::ffi::{CString, c_void};
use std::path::PathBuf;

fn fixture_bytes(name: &str) -> Vec<u8> {
    let fixture = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
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

fn import_repo_fixture(relative: &str) -> *mut RiveFile {
    let bytes = repo_fixture_bytes(relative);
    let mut file: *mut RiveFile = std::ptr::null_mut();
    let status = unsafe { rive_file_import(bytes.as_ptr(), bytes.len(), &mut file) };
    assert_eq!(status, RiveStatus::Ok);
    assert!(!file.is_null());
    file
}

fn string_view_to_owned(view: RiveStringView) -> String {
    assert!(!view.data.is_null());
    let bytes = unsafe { std::slice::from_raw_parts(view.data.cast::<u8>(), view.len) };
    std::str::from_utf8(bytes).expect("utf8 name").to_owned()
}

#[test]
fn c_api_imports_file_and_exposes_artboard_metadata() {
    let bytes = fixture_bytes("shapetest.riv");
    let mut file: *mut RiveFile = std::ptr::null_mut();
    let status = unsafe { rive_file_import(bytes.as_ptr(), bytes.len(), &mut file) };
    assert_eq!(status, RiveStatus::Ok);
    assert!(!file.is_null());

    let artboard_count = unsafe { rive_file_artboard_count(file) };
    assert_eq!(artboard_count, 1);

    let mut name = RiveStringView {
        data: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { rive_file_artboard_name(file, 0, &mut name) };
    assert_eq!(status, RiveStatus::Ok);
    assert!(!name.data.is_null());
    let name_bytes = unsafe { std::slice::from_raw_parts(name.data.cast::<u8>(), name.len) };
    assert_eq!(std::str::from_utf8(name_bytes).unwrap(), "New Artboard");

    let mut animation_count = usize::MAX;
    let status = unsafe { rive_file_artboard_animation_count(file, 0, &mut animation_count) };
    assert_eq!(status, RiveStatus::Ok);
    assert_ne!(animation_count, usize::MAX);

    let mut state_machine_count = usize::MAX;
    let status =
        unsafe { rive_file_artboard_state_machine_count(file, 0, &mut state_machine_count) };
    assert_eq!(status, RiveStatus::Ok);
    assert_ne!(state_machine_count, usize::MAX);

    let missing = unsafe { rive_file_artboard_name(file, 99, &mut name) };
    assert_eq!(missing, RiveStatus::NotFound);

    unsafe {
        rive_file_free(file);
    }
}

#[test]
fn c_api_rejects_null_arguments_without_writing_handles() {
    let bytes = fixture_bytes("shapetest.riv");
    let status = unsafe { rive_file_import(bytes.as_ptr(), bytes.len(), std::ptr::null_mut()) };
    assert_eq!(status, RiveStatus::NullArgument);

    let mut file: *mut RiveFile = std::ptr::dangling_mut();
    let status = unsafe { rive_file_import(std::ptr::null(), bytes.len(), &mut file) };
    assert_eq!(status, RiveStatus::NullArgument);
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
        rive_file_artboard_state_machine_count(file, SMI_INPUT_ARTBOARD, &mut state_machine_count)
    };
    assert_eq!(status, RiveStatus::Ok);
    assert!(state_machine_count >= 1);

    let mut name = RiveStringView::default();
    let status =
        unsafe { rive_file_artboard_state_machine_name(file, SMI_INPUT_ARTBOARD, 0, &mut name) };
    assert_eq!(status, RiveStatus::Ok);
    assert_eq!(string_view_to_owned(name), "State Machine 1");
    let status =
        unsafe { rive_file_artboard_state_machine_name(file, SMI_INPUT_ARTBOARD, 99, &mut name) };
    assert_eq!(status, RiveStatus::NotFound);

    let mut instance: *mut RiveArtboardInstance = std::ptr::null_mut();
    let status = unsafe { rive_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, RiveStatus::Ok);
    assert!(!instance.is_null());

    let mut state_machine: *mut RiveStateMachineInstance = std::ptr::null_mut();
    let status = unsafe { rive_state_machine_instance_new_default(instance, &mut state_machine) };
    assert_eq!(status, RiveStatus::Ok);
    assert!(!state_machine.is_null());

    let bool_name = CString::new("bool").unwrap();
    let num_name = CString::new("num").unwrap();
    let trig_name = CString::new("trig").unwrap();
    let missing_name = CString::new("nope").unwrap();

    let status =
        unsafe { rive_state_machine_instance_set_bool(state_machine, bool_name.as_ptr(), true) };
    assert_eq!(status, RiveStatus::Ok);
    let status =
        unsafe { rive_state_machine_instance_set_number(state_machine, num_name.as_ptr(), 42.0) };
    assert_eq!(status, RiveStatus::Ok);
    let status =
        unsafe { rive_state_machine_instance_fire_trigger(state_machine, trig_name.as_ptr()) };
    assert_eq!(status, RiveStatus::Ok);

    // Missing input name vs. wrong input kind report distinct statuses.
    let status =
        unsafe { rive_state_machine_instance_set_bool(state_machine, missing_name.as_ptr(), true) };
    assert_eq!(status, RiveStatus::NotFound);
    let status =
        unsafe { rive_state_machine_instance_set_number(state_machine, bool_name.as_ptr(), 1.0) };
    assert_eq!(status, RiveStatus::InvalidArgument);

    let mut changed = false;
    let status = unsafe {
        rive_state_machine_instance_advance(instance, state_machine, 0.016, &mut changed)
    };
    assert_eq!(status, RiveStatus::Ok);
    // The advance out-param is optional.
    let status = unsafe {
        rive_state_machine_instance_advance(instance, state_machine, 0.016, std::ptr::null_mut())
    };
    assert_eq!(status, RiveStatus::Ok);

    unsafe {
        rive_state_machine_instance_free(state_machine);
        rive_artboard_instance_free(instance);
        rive_file_free(file);
    }
}

#[test]
fn c_api_state_machine_instance_by_index_and_missing_index() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut RiveArtboardInstance = std::ptr::null_mut();
    let status = unsafe { rive_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, RiveStatus::Ok);

    let mut state_machine: *mut RiveStateMachineInstance = std::ptr::null_mut();
    let status = unsafe { rive_state_machine_instance_new(instance, 0, &mut state_machine) };
    assert_eq!(status, RiveStatus::Ok);
    assert!(!state_machine.is_null());
    unsafe { rive_state_machine_instance_free(state_machine) };

    let status = unsafe { rive_state_machine_instance_new(instance, 99, &mut state_machine) };
    assert_eq!(status, RiveStatus::NotFound);
    assert!(state_machine.is_null());

    unsafe {
        rive_artboard_instance_free(instance);
        rive_file_free(file);
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
    path: *const rive_capi::RiveRawPathView,
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
    let mut instance: *mut RiveArtboardInstance = std::ptr::null_mut();
    let status = unsafe { rive_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, RiveStatus::Ok);
    let status = unsafe { rive_artboard_instance_advance(instance, 0.0, std::ptr::null_mut()) };
    assert_eq!(status, RiveStatus::Ok);

    let mut counters_data = RenderCounters::default();
    let callbacks = RiveRenderCallbacks {
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
        ..RiveRenderCallbacks::default()
    };

    let status = unsafe { rive_artboard_instance_draw(instance, &callbacks) };
    assert_eq!(status, RiveStatus::Ok);

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
        rive_artboard_instance_free(instance);
        rive_file_free(file);
    }
}

#[test]
fn c_api_draw_with_empty_vtable_behaves_like_null_renderer() {
    let file = import_repo_fixture(SMI_FIXTURE);
    let mut instance: *mut RiveArtboardInstance = std::ptr::null_mut();
    let status = unsafe { rive_artboard_instance_new(file, SMI_INPUT_ARTBOARD, &mut instance) };
    assert_eq!(status, RiveStatus::Ok);

    let callbacks = RiveRenderCallbacks::default();
    let status = unsafe { rive_artboard_instance_draw(instance, &callbacks) };
    assert_eq!(status, RiveStatus::Ok);

    unsafe {
        rive_artboard_instance_free(instance);
        rive_file_free(file);
    }
}

#[test]
fn c_api_instance_functions_reject_null_arguments() {
    let file = import_repo_fixture(SMI_FIXTURE);

    let status =
        unsafe { rive_artboard_instance_new(file, SMI_INPUT_ARTBOARD, std::ptr::null_mut()) };
    assert_eq!(status, RiveStatus::NullArgument);

    let mut instance: *mut RiveArtboardInstance = std::ptr::dangling_mut();
    let status = unsafe { rive_artboard_instance_new(std::ptr::null(), 0, &mut instance) };
    assert_eq!(status, RiveStatus::NullArgument);
    assert!(instance.is_null());

    let status = unsafe { rive_artboard_instance_new(file, 99, &mut instance) };
    assert_eq!(status, RiveStatus::NotFound);
    assert!(instance.is_null());

    let status =
        unsafe { rive_artboard_instance_advance(std::ptr::null_mut(), 0.0, std::ptr::null_mut()) };
    assert_eq!(status, RiveStatus::NullArgument);

    let status = unsafe { rive_artboard_instance_draw(std::ptr::null_mut(), std::ptr::null()) };
    assert_eq!(status, RiveStatus::NullArgument);

    let mut state_machine: *mut RiveStateMachineInstance = std::ptr::dangling_mut();
    let status =
        unsafe { rive_state_machine_instance_new(std::ptr::null(), 0, &mut state_machine) };
    assert_eq!(status, RiveStatus::NullArgument);
    assert!(state_machine.is_null());

    let name = CString::new("bool").unwrap();
    let status =
        unsafe { rive_state_machine_instance_set_bool(std::ptr::null_mut(), name.as_ptr(), true) };
    assert_eq!(status, RiveStatus::NullArgument);

    unsafe {
        rive_file_free(file);
    }
}
