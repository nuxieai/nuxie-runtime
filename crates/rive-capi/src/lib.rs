mod render_callbacks;

pub use render_callbacks::{RiveImageSampler, RiveRawPathView, RiveRenderCallbacks};

use render_callbacks::{CallbackFactory, CallbackRenderer};
use rive::{ArtboardInstance, File, StateMachineInstance};
use std::ffi::{CStr, c_char};
use std::ptr;
use std::slice;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiveStatus {
    Ok = 0,
    NullArgument = 1,
    ImportError = 2,
    NotFound = 3,
    RuntimeError = 4,
    InvalidArgument = 5,
}

pub struct RiveFile {
    file: File,
}

/// Owned artboard instance. The [`RiveFile`] it was created from must stay
/// alive (not freed) for as long as this instance exists.
pub struct RiveArtboardInstance {
    instance: ArtboardInstance<'static>,
}

/// Owned state machine instance. Advance it through the
/// [`RiveArtboardInstance`] it was created from.
pub struct RiveStateMachineInstance {
    instance: StateMachineInstance,
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

/// Name of one of an artboard's state machines. The returned view borrows the
/// file and is valid until the file is freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_file_artboard_state_machine_name(
    file: *const RiveFile,
    artboard_index: usize,
    state_machine_index: usize,
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
    let Some(artboard) = file.file.artboard(artboard_index) else {
        return RiveStatus::NotFound;
    };
    let Some(name) = artboard.state_machine_name(state_machine_index) else {
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

/// Instantiate the artboard at `artboard_index`. The file must outlive the
/// returned instance; free it with `rive_artboard_instance_free` before
/// freeing the file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_artboard_instance_new(
    file: *const RiveFile,
    artboard_index: usize,
    out_instance: *mut *mut RiveArtboardInstance,
) -> RiveStatus {
    if out_instance.is_null() {
        return RiveStatus::NullArgument;
    }
    unsafe {
        *out_instance = ptr::null_mut();
    }
    let Some(file) = (unsafe { file.as_ref() }) else {
        return RiveStatus::NullArgument;
    };
    let Some(artboard) = file.file.artboard(artboard_index) else {
        return RiveStatus::NotFound;
    };
    match artboard.instantiate() {
        Ok(instance) => {
            // SAFETY: the caller keeps the file alive for the whole lifetime
            // of the instance (documented ownership contract), so extending
            // the borrow to 'static never dangles.
            let instance = unsafe {
                std::mem::transmute::<ArtboardInstance<'_>, ArtboardInstance<'static>>(instance)
            };
            unsafe {
                *out_instance = Box::into_raw(Box::new(RiveArtboardInstance { instance }));
            }
            RiveStatus::Ok
        }
        Err(_) => RiveStatus::RuntimeError,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_artboard_instance_free(instance: *mut RiveArtboardInstance) {
    if instance.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(instance));
    }
}

/// Advance the artboard timeline without a state machine. `out_changed` is
/// optional and reports whether anything changed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_artboard_instance_advance(
    instance: *mut RiveArtboardInstance,
    elapsed_seconds: f32,
    out_changed: *mut bool,
) -> RiveStatus {
    if let Some(out_changed) = unsafe { out_changed.as_mut() } {
        *out_changed = false;
    }
    let Some(instance) = (unsafe { instance.as_mut() }) else {
        return RiveStatus::NullArgument;
    };
    let changed = instance.instance.advance(elapsed_seconds);
    if let Some(out_changed) = unsafe { out_changed.as_mut() } {
        *out_changed = changed;
    }
    RiveStatus::Ok
}

/// Draw the artboard through the caller-provided render vtable. See
/// `RiveRenderCallbacks` for the ownership and handle contract; the callbacks
/// only need to stay valid for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_artboard_instance_draw(
    instance: *mut RiveArtboardInstance,
    callbacks: *const RiveRenderCallbacks,
) -> RiveStatus {
    let Some(callbacks) = (unsafe { callbacks.as_ref() }) else {
        return RiveStatus::NullArgument;
    };
    let Some(instance) = (unsafe { instance.as_mut() }) else {
        return RiveStatus::NullArgument;
    };
    let mut factory = CallbackFactory::new(*callbacks);
    let mut renderer = CallbackRenderer::new(*callbacks);
    match instance.instance.draw(&mut factory, &mut renderer) {
        Ok(()) => RiveStatus::Ok,
        Err(_) => RiveStatus::RuntimeError,
    }
}

/// Instantiate the state machine at `state_machine_index` on the instance's
/// artboard. Free with `rive_state_machine_instance_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_state_machine_instance_new(
    instance: *const RiveArtboardInstance,
    state_machine_index: usize,
    out_state_machine: *mut *mut RiveStateMachineInstance,
) -> RiveStatus {
    if out_state_machine.is_null() {
        return RiveStatus::NullArgument;
    }
    unsafe {
        *out_state_machine = ptr::null_mut();
    }
    let Some(instance) = (unsafe { instance.as_ref() }) else {
        return RiveStatus::NullArgument;
    };
    let Some(state_machine) = instance
        .instance
        .state_machine_instance(state_machine_index)
    else {
        return RiveStatus::NotFound;
    };
    unsafe {
        *out_state_machine = Box::into_raw(Box::new(RiveStateMachineInstance {
            instance: state_machine,
        }));
    }
    RiveStatus::Ok
}

/// Instantiate the artboard's default state machine: the one flagged in the
/// source file when present, otherwise the first state machine. Returns
/// `RIVE_STATUS_NOT_FOUND` when the artboard has no state machines.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_state_machine_instance_new_default(
    instance: *const RiveArtboardInstance,
    out_state_machine: *mut *mut RiveStateMachineInstance,
) -> RiveStatus {
    if out_state_machine.is_null() {
        return RiveStatus::NullArgument;
    }
    unsafe {
        *out_state_machine = ptr::null_mut();
    }
    let Some(instance) = (unsafe { instance.as_ref() }) else {
        return RiveStatus::NullArgument;
    };
    let Some(state_machine) = instance.instance.default_state_machine_instance() else {
        return RiveStatus::NotFound;
    };
    unsafe {
        *out_state_machine = Box::into_raw(Box::new(RiveStateMachineInstance {
            instance: state_machine,
        }));
    }
    RiveStatus::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_state_machine_instance_free(
    state_machine: *mut RiveStateMachineInstance,
) {
    if state_machine.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(state_machine));
    }
}

/// Set a bool input by name (NUL-terminated UTF-8). Returns
/// `RIVE_STATUS_NOT_FOUND` when no input has that name and
/// `RIVE_STATUS_INVALID_ARGUMENT` when the input is not a bool.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_state_machine_instance_set_bool(
    state_machine: *mut RiveStateMachineInstance,
    name: *const c_char,
    value: bool,
) -> RiveStatus {
    state_machine_input_by_name(state_machine, name, |state_machine, index| {
        state_machine.set_bool(index, value)
    })
}

/// Set a number input by name (NUL-terminated UTF-8). Returns
/// `RIVE_STATUS_NOT_FOUND` when no input has that name and
/// `RIVE_STATUS_INVALID_ARGUMENT` when the input is not a number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_state_machine_instance_set_number(
    state_machine: *mut RiveStateMachineInstance,
    name: *const c_char,
    value: f32,
) -> RiveStatus {
    state_machine_input_by_name(state_machine, name, |state_machine, index| {
        state_machine.set_number(index, value)
    })
}

/// Fire a trigger input by name (NUL-terminated UTF-8). Returns
/// `RIVE_STATUS_NOT_FOUND` when no input has that name and
/// `RIVE_STATUS_INVALID_ARGUMENT` when the input is not a trigger.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_state_machine_instance_fire_trigger(
    state_machine: *mut RiveStateMachineInstance,
    name: *const c_char,
) -> RiveStatus {
    state_machine_input_by_name(state_machine, name, |state_machine, index| {
        state_machine.fire_trigger(index)
    })
}

fn state_machine_input_by_name(
    state_machine: *mut RiveStateMachineInstance,
    name: *const c_char,
    apply: impl FnOnce(&mut StateMachineInstance, usize) -> bool,
) -> RiveStatus {
    let Some(state_machine) = (unsafe { state_machine.as_mut() }) else {
        return RiveStatus::NullArgument;
    };
    if name.is_null() {
        return RiveStatus::NullArgument;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return RiveStatus::InvalidArgument;
    };
    let Some(index) = state_machine.instance.input_index_named(name) else {
        return RiveStatus::NotFound;
    };
    if apply(&mut state_machine.instance, index) {
        RiveStatus::Ok
    } else {
        RiveStatus::InvalidArgument
    }
}

/// Advance the artboard while driving `state_machine`. The state machine must
/// have been created from the same artboard instance. `out_changed` is
/// optional and reports whether anything changed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rive_state_machine_instance_advance(
    instance: *mut RiveArtboardInstance,
    state_machine: *mut RiveStateMachineInstance,
    elapsed_seconds: f32,
    out_changed: *mut bool,
) -> RiveStatus {
    if let Some(out_changed) = unsafe { out_changed.as_mut() } {
        *out_changed = false;
    }
    let Some(instance) = (unsafe { instance.as_mut() }) else {
        return RiveStatus::NullArgument;
    };
    let Some(state_machine) = (unsafe { state_machine.as_mut() }) else {
        return RiveStatus::NullArgument;
    };
    let changed = instance
        .instance
        .advance_with_state_machine(&mut state_machine.instance, elapsed_seconds);
    if let Some(out_changed) = unsafe { out_changed.as_mut() } {
        *out_changed = changed;
    }
    RiveStatus::Ok
}
