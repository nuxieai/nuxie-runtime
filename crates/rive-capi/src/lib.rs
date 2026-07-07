use rive::File;
use std::ffi::c_char;
use std::ptr;
use std::slice;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiveStatus {
    Ok = 0,
    NullArgument = 1,
    ImportError = 2,
    NotFound = 3,
}

pub struct RiveFile {
    file: File,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RiveStringView {
    pub data: *const c_char,
    pub len: usize,
}

impl Default for RiveStringView {
    fn default() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_file_import(
    bytes: *const u8,
    len: usize,
    out_file: *mut *mut RiveFile,
) -> RiveStatus {
    if out_file.is_null() {
        return RiveStatus::NullArgument;
    }
    unsafe {
        *out_file = ptr::null_mut();
    }
    if bytes.is_null() && len != 0 {
        return RiveStatus::NullArgument;
    }

    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(bytes, len) }
    };
    match File::import(bytes) {
        Ok(file) => {
            let handle = Box::new(RiveFile { file });
            unsafe {
                *out_file = Box::into_raw(handle);
            }
            RiveStatus::Ok
        }
        Err(_) => RiveStatus::ImportError,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_file_free(file: *mut RiveFile) {
    if file.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(file));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_file_artboard_count(file: *const RiveFile) -> usize {
    let Some(file) = (unsafe { file.as_ref() }) else {
        return 0;
    };
    file.file.artboard_count()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_file_artboard_name(
    file: *const RiveFile,
    index: usize,
    out_name: *mut RiveStringView,
) -> RiveStatus {
    if out_name.is_null() {
        return RiveStatus::NullArgument;
    }
    unsafe {
        *out_name = RiveStringView::default();
    }
    let Some(file) = (unsafe { file.as_ref() }) else {
        return RiveStatus::NullArgument;
    };
    let Some(artboard) = file.file.artboard(index) else {
        return RiveStatus::NotFound;
    };
    let Some(name) = artboard.name() else {
        return RiveStatus::NotFound;
    };
    let bytes = name.as_bytes();
    unsafe {
        *out_name = RiveStringView {
            data: bytes.as_ptr().cast::<c_char>(),
            len: bytes.len(),
        };
    }
    RiveStatus::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_file_artboard_animation_count(
    file: *const RiveFile,
    index: usize,
    out_count: *mut usize,
) -> RiveStatus {
    artboard_count_by(file, index, out_count, |artboard| {
        artboard.animation_count()
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_file_artboard_state_machine_count(
    file: *const RiveFile,
    index: usize,
    out_count: *mut usize,
) -> RiveStatus {
    artboard_count_by(file, index, out_count, |artboard| {
        artboard.state_machine_count()
    })
}

fn artboard_count_by(
    file: *const RiveFile,
    index: usize,
    out_count: *mut usize,
    count: impl FnOnce(rive::Artboard<'_>) -> usize,
) -> RiveStatus {
    if out_count.is_null() {
        return RiveStatus::NullArgument;
    }
    unsafe {
        *out_count = 0;
    }
    let Some(file) = (unsafe { file.as_ref() }) else {
        return RiveStatus::NullArgument;
    };
    let Some(artboard) = file.file.artboard(index) else {
        return RiveStatus::NotFound;
    };
    unsafe {
        *out_count = count(artboard);
    }
    RiveStatus::Ok
}
