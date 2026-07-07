use rive_capi::{
    RiveFile, RiveStatus, RiveStringView, rive_file_artboard_animation_count,
    rive_file_artboard_count, rive_file_artboard_name, rive_file_artboard_state_machine_count,
    rive_file_free, rive_file_import,
};
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
