mod render_callbacks;

pub use render_callbacks::{NuxImageSampler, NuxRawPathView, NuxRenderCallbacks};

use nuxie::{ArtboardInstance, ArtboardRenderCache, File, StateMachineInstance, ViewModelInstance};
use render_callbacks::{CallbackFactory, CallbackRenderer};
use std::ffi::{CStr, c_char};
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use std::slice;

/// Increment only for a breaking change to the exported C contract.
pub const NUX_CAPI_ABI_VERSION: u32 = 1;

const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");
const SOURCE_REVISION: &str = env!("NUX_RUNTIME_SOURCE_REVISION");

/// Panic firewall for the C ABI boundary.
///
/// Every `extern "C"` entry point runs its body through this guard so a Rust
/// panic is turned into `default` (a status or handle the caller already knows
/// how to handle) instead of unwinding across the FFI boundary, which is
/// undefined behaviour. The runtime ships as an SDK embedded in customer apps,
/// so a stray unwind into C is existential.
///
/// This is profile-independent by design. Release builds set `panic = "abort"`,
/// under which nothing ever unwinds and `catch_unwind` compiles down to a plain
/// call (free); debug builds of the `cdylib` *do* unwind, and there this guard
/// is what stops a panic from reaching C.
///
/// `body` captures raw pointers (and references derived from them), which are
/// not `UnwindSafe`. Asserting unwind safety is sound here: on a panic we drop
/// all locals and return a fixed error value without ever letting the caller
/// observe a half-updated Rust invariant across the boundary.
fn ffi_guard<R>(default: R, body: impl FnOnce() -> R) -> R {
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(_) => default,
    }
}

/// Debug-only tracking of which [`NuxFile`]s still have live artboard
/// instances borrowing them. See the module docs for the ownership contract
/// this guards; it turns the "free the file before its instances" use-after-free
/// footgun into a loud, deterministic abort in debug builds instead of silent
/// UB. It compiles to nothing in release (where `panic = "abort"` is set and
/// the real fix — shared ownership — is tracked as a follow-up).
#[cfg(debug_assertions)]
mod liveness {
    use super::NuxFile;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    // Maps a live `NuxFile` pointer to the number of outstanding artboard
    // instances that borrow it.
    static REGISTRY: OnceLock<Mutex<HashMap<usize, usize>>> = OnceLock::new();

    fn registry() -> &'static Mutex<HashMap<usize, usize>> {
        REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub(super) fn register_instance(file: *const NuxFile) {
        if file.is_null() {
            return;
        }
        let mut map = registry().lock().expect("liveness registry poisoned");
        *map.entry(file as usize).or_insert(0) += 1;
    }

    pub(super) fn unregister_instance(file: *const NuxFile) {
        if file.is_null() {
            return;
        }
        let mut map = registry().lock().expect("liveness registry poisoned");
        if let Some(count) = map.get_mut(&(file as usize)) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                map.remove(&(file as usize));
            }
        }
    }

    pub(super) fn assert_no_live_instances(file: *const NuxFile) {
        if file.is_null() {
            return;
        }
        let count = registry()
            .lock()
            .expect("liveness registry poisoned")
            .get(&(file as usize))
            .copied()
            .unwrap_or(0);
        if count != 0 {
            // Not a panic: a panic here would be swallowed by the `ffi_guard`
            // around `nux_file_free`. Abort surfaces the misuse loudly.
            eprintln!(
                "nux-capi: use-after-free averted: nux_file_free({file:p}) called \
                 while {count} artboard instance(s) still borrow this file. Free every \
                 NuxArtboardInstance before its NuxFile. Aborting (debug build)."
            );
            std::process::abort();
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxStatus {
    Ok = 0,
    NullArgument = 1,
    ImportError = 2,
    NotFound = 3,
    RuntimeError = 4,
    InvalidArgument = 5,
    AbiMismatch = 6,
}

pub struct NuxFile {
    file: File,
}

/// Owned artboard instance. The [`NuxFile`] it was created from must stay
/// alive (not freed) for as long as this instance exists.
pub struct NuxArtboardInstance {
    instance: ArtboardInstance<'static>,
    /// Originating file pointer, tracked only in debug builds to detect the
    /// use-after-free footgun in [`liveness`].
    #[cfg(debug_assertions)]
    file: *const NuxFile,
}

/// Render resources retained across draws of one artboard instance.
pub struct NuxRenderCache {
    instance: *const NuxArtboardInstance,
    callbacks: NuxRenderCallbacks,
    cache: ArtboardRenderCache,
}

/// Owned state machine instance. Advance it through the
/// [`NuxArtboardInstance`] it was created from.
pub struct NuxStateMachineInstance {
    instance: StateMachineInstance,
}

/// Owned view-model context for driving an artboard's data binds.
///
/// Unlike [`NuxArtboardInstance`], this handle owns a private copy of the
/// view model's values and does **not** borrow the [`NuxFile`] it came from,
/// so it participates in no liveness ordering: it may be freed before or after
/// its originating file and artboard instance. It is only meaningful when bound
/// back (via `nux_artboard_instance_bind_view_model`) to the artboard instance
/// it was created from, which must still be alive at bind time.
pub struct NuxViewModelInstance {
    instance: ViewModelInstance,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxStringView {
    pub data: *const c_char,
    pub len: usize,
}

impl Default for NuxStringView {
    fn default() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
        }
    }
}

impl NuxStringView {
    fn from_static(value: &'static str) -> Self {
        Self {
            data: value.as_ptr().cast(),
            len: value.len(),
        }
    }
}

/// Immutable identity embedded into the shipped runtime binary.
///
/// Both strings have process-static lifetime and are not NUL-terminated; C
/// callers must respect their explicit lengths.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxRuntimeInfo {
    pub abi_version: u32,
    pub runtime_version: NuxStringView,
    pub source_revision: NuxStringView,
}

impl Default for NuxRuntimeInfo {
    fn default() -> Self {
        Self {
            abi_version: 0,
            runtime_version: NuxStringView::default(),
            source_revision: NuxStringView::default(),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_capi_abi_version() -> u32 {
    ffi_guard(0, || NUX_CAPI_ABI_VERSION)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_capi_require_abi(required_version: u32) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if required_version == NUX_CAPI_ABI_VERSION {
            NuxStatus::Ok
        } else {
            NuxStatus::AbiMismatch
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_capi_runtime_info(out_info: *mut NuxRuntimeInfo) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_info.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_info = NuxRuntimeInfo {
                abi_version: NUX_CAPI_ABI_VERSION,
                runtime_version: NuxStringView::from_static(RUNTIME_VERSION),
                source_revision: NuxStringView::from_static(SOURCE_REVISION),
            };
        }
        NuxStatus::Ok
    })
}

/// Pointer id reported to the runtime for the single-pointer C surface.
const DEFAULT_POINTER_ID: i32 = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_import(
    bytes: *const u8,
    len: usize,
    out_file: *mut *mut NuxFile,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_file.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_file = ptr::null_mut();
        }
        if bytes.is_null() && len != 0 {
            return NuxStatus::NullArgument;
        }

        let bytes = if len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(bytes, len) }
        };
        match File::import(bytes) {
            Ok(file) => {
                let handle = Box::new(NuxFile { file });
                unsafe {
                    *out_file = Box::into_raw(handle);
                }
                NuxStatus::Ok
            }
            Err(_) => NuxStatus::ImportError,
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_free(file: *mut NuxFile) {
    ffi_guard((), || {
        if file.is_null() {
            return;
        }
        // Debug-only: abort loudly if instances still borrow this file rather
        // than let the caller dangle them (silent UB otherwise).
        #[cfg(debug_assertions)]
        liveness::assert_no_live_instances(file);
        unsafe {
            drop(Box::from_raw(file));
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_artboard_count(file: *const NuxFile) -> usize {
    ffi_guard(0, || {
        let Some(file) = (unsafe { file.as_ref() }) else {
            return 0;
        };
        file.file.artboard_count()
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_artboard_name(
    file: *const NuxFile,
    index: usize,
    out_name: *mut NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_name.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_name = NuxStringView::default();
        }
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(artboard) = file.file.artboard(index) else {
            return NuxStatus::NotFound;
        };
        let Some(name) = artboard.name() else {
            return NuxStatus::NotFound;
        };
        let bytes = name.as_bytes();
        unsafe {
            *out_name = NuxStringView {
                data: bytes.as_ptr().cast::<c_char>(),
                len: bytes.len(),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_artboard_animation_count(
    file: *const NuxFile,
    index: usize,
    out_count: *mut usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        artboard_count_by(file, index, out_count, |artboard| {
            artboard.animation_count()
        })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_artboard_state_machine_count(
    file: *const NuxFile,
    index: usize,
    out_count: *mut usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        artboard_count_by(file, index, out_count, |artboard| {
            artboard.state_machine_count()
        })
    })
}

fn artboard_count_by(
    file: *const NuxFile,
    index: usize,
    out_count: *mut usize,
    count: impl FnOnce(nuxie::Artboard<'_>) -> usize,
) -> NuxStatus {
    if out_count.is_null() {
        return NuxStatus::NullArgument;
    }
    unsafe {
        *out_count = 0;
    }
    let Some(file) = (unsafe { file.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    let Some(artboard) = file.file.artboard(index) else {
        return NuxStatus::NotFound;
    };
    unsafe {
        *out_count = count(artboard);
    }
    NuxStatus::Ok
}

/// Name of one of an artboard's state machines. The returned view borrows the
/// file and is valid until the file is freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_artboard_state_machine_name(
    file: *const NuxFile,
    artboard_index: usize,
    state_machine_index: usize,
    out_name: *mut NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_name.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_name = NuxStringView::default();
        }
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(artboard) = file.file.artboard(artboard_index) else {
            return NuxStatus::NotFound;
        };
        let Some(name) = artboard.state_machine_name(state_machine_index) else {
            return NuxStatus::NotFound;
        };
        let bytes = name.as_bytes();
        unsafe {
            *out_name = NuxStringView {
                data: bytes.as_ptr().cast::<c_char>(),
                len: bytes.len(),
            };
        }
        NuxStatus::Ok
    })
}

/// Instantiate the artboard at `artboard_index`. The file must outlive the
/// returned instance; free it with `nux_artboard_instance_free` before
/// freeing the file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_new(
    file: *const NuxFile,
    artboard_index: usize,
    out_instance: *mut *mut NuxArtboardInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_instance = ptr::null_mut();
        }
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(artboard) = file.file.artboard(artboard_index) else {
            return NuxStatus::NotFound;
        };
        match artboard.instantiate() {
            Ok(instance) => {
                // SAFETY: the caller keeps the file alive for the whole lifetime
                // of the instance (documented ownership contract, enforced with a
                // debug-only liveness check in `nux_file_free`), so extending the
                // borrow to 'static never dangles.
                let instance = unsafe {
                    std::mem::transmute::<ArtboardInstance<'_>, ArtboardInstance<'static>>(instance)
                };
                #[cfg(debug_assertions)]
                liveness::register_instance(file);
                let handle = NuxArtboardInstance {
                    instance,
                    #[cfg(debug_assertions)]
                    file,
                };
                unsafe {
                    *out_instance = Box::into_raw(Box::new(handle));
                }
                NuxStatus::Ok
            }
            Err(_) => NuxStatus::RuntimeError,
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_free(instance: *mut NuxArtboardInstance) {
    ffi_guard((), || {
        if instance.is_null() {
            return;
        }
        let handle = unsafe { Box::from_raw(instance) };
        #[cfg(debug_assertions)]
        liveness::unregister_instance(handle.file);
        drop(handle);
    })
}

/// Advance the artboard timeline without a state machine. `out_changed` is
/// optional and reports whether anything changed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_advance(
    instance: *mut NuxArtboardInstance,
    elapsed_seconds: f32,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if let Some(out_changed) = unsafe { out_changed.as_mut() } {
            *out_changed = false;
        }
        let Some(instance) = (unsafe { instance.as_mut() }) else {
            return NuxStatus::NullArgument;
        };
        let changed = instance.instance.advance(elapsed_seconds);
        if let Some(out_changed) = unsafe { out_changed.as_mut() } {
            *out_changed = changed;
        }
        NuxStatus::Ok
    })
}

/// Draw the artboard through the caller-provided render vtable. See
/// `NuxRenderCallbacks` for the ownership and handle contract; the callbacks
/// only need to stay valid for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_draw(
    instance: *mut NuxArtboardInstance,
    callbacks: *const NuxRenderCallbacks,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(callbacks) = (unsafe { callbacks.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(instance) = (unsafe { instance.as_mut() }) else {
            return NuxStatus::NullArgument;
        };
        let mut factory = CallbackFactory::new(*callbacks);
        let mut renderer = CallbackRenderer::new(*callbacks);
        match instance.instance.draw(&mut factory, &mut renderer) {
            Ok(()) => NuxStatus::Ok,
            Err(_) => NuxStatus::RuntimeError,
        }
    })
}

/// Create a retained render cache for `instance` without invoking renderer callbacks.
/// Resource creation begins on the first cached draw, and a failed candidate is
/// discarded so the same cache can retry. The callback table and its `user_data`
/// must remain valid until `nux_render_cache_free` returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_render_cache_new(
    instance: *const NuxArtboardInstance,
    callbacks: *const NuxRenderCallbacks,
    out_cache: *mut *mut NuxRenderCache,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_cache.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_cache = ptr::null_mut();
        }
        let Some(instance_ref) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(callbacks) = (unsafe { callbacks.as_ref() }).copied() else {
            return NuxStatus::NullArgument;
        };
        let cache = instance_ref.instance.new_render_cache();
        unsafe {
            *out_cache = Box::into_raw(Box::new(NuxRenderCache {
                instance,
                callbacks,
                cache,
            }));
        }
        NuxStatus::Ok
    })
}

/// Draw using render handles retained in `cache` from previous calls.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_draw_cached(
    instance: *mut NuxArtboardInstance,
    cache: *mut NuxRenderCache,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(instance_ref) = (unsafe { instance.as_mut() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(cache) = (unsafe { cache.as_mut() }) else {
            return NuxStatus::NullArgument;
        };
        if !std::ptr::eq(instance.cast_const(), cache.instance) {
            return NuxStatus::InvalidArgument;
        }
        let mut factory = CallbackFactory::new(cache.callbacks);
        let mut renderer = CallbackRenderer::new(cache.callbacks);
        match instance_ref.instance.draw_with_render_cache(
            &mut factory,
            &mut renderer,
            &mut cache.cache,
        ) {
            Ok(()) => NuxStatus::Ok,
            Err(_) => NuxStatus::RuntimeError,
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_render_cache_free(cache: *mut NuxRenderCache) {
    ffi_guard((), || {
        if !cache.is_null() {
            unsafe {
                drop(Box::from_raw(cache));
            }
        }
    })
}

/// Instantiate the state machine at `state_machine_index` on the instance's
/// artboard. Free with `nux_state_machine_instance_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_new(
    instance: *const NuxArtboardInstance,
    state_machine_index: usize,
    out_state_machine: *mut *mut NuxStateMachineInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_state_machine.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_state_machine = ptr::null_mut();
        }
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(state_machine) = instance
            .instance
            .state_machine_instance(state_machine_index)
        else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_state_machine = Box::into_raw(Box::new(NuxStateMachineInstance {
                instance: state_machine,
            }));
        }
        NuxStatus::Ok
    })
}

/// Instantiate the artboard's default state machine: the one flagged in the
/// source file when present, otherwise the first state machine. Returns
/// `NUX_STATUS_NOT_FOUND` when the artboard has no state machines.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_new_default(
    instance: *const NuxArtboardInstance,
    out_state_machine: *mut *mut NuxStateMachineInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_state_machine.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_state_machine = ptr::null_mut();
        }
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(state_machine) = instance.instance.default_state_machine_instance() else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_state_machine = Box::into_raw(Box::new(NuxStateMachineInstance {
                instance: state_machine,
            }));
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_free(
    state_machine: *mut NuxStateMachineInstance,
) {
    ffi_guard((), || {
        if state_machine.is_null() {
            return;
        }
        unsafe {
            drop(Box::from_raw(state_machine));
        }
    })
}

/// Set a bool input by name (NUL-terminated UTF-8). Returns
/// `NUX_STATUS_NOT_FOUND` when no input has that name and
/// `NUX_STATUS_INVALID_ARGUMENT` when the input is not a bool.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_set_bool(
    state_machine: *mut NuxStateMachineInstance,
    name: *const c_char,
    value: bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        state_machine_input_by_name(state_machine, name, |state_machine, index| {
            state_machine.set_bool(index, value)
        })
    })
}

/// Set a number input by name (NUL-terminated UTF-8). Returns
/// `NUX_STATUS_NOT_FOUND` when no input has that name and
/// `NUX_STATUS_INVALID_ARGUMENT` when the input is not a number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_set_number(
    state_machine: *mut NuxStateMachineInstance,
    name: *const c_char,
    value: f32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        state_machine_input_by_name(state_machine, name, |state_machine, index| {
            state_machine.set_number(index, value)
        })
    })
}

/// Fire a trigger input by name (NUL-terminated UTF-8). Returns
/// `NUX_STATUS_NOT_FOUND` when no input has that name and
/// `NUX_STATUS_INVALID_ARGUMENT` when the input is not a trigger.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_fire_trigger(
    state_machine: *mut NuxStateMachineInstance,
    name: *const c_char,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        state_machine_input_by_name(state_machine, name, |state_machine, index| {
            state_machine.fire_trigger(index)
        })
    })
}

fn state_machine_input_by_name(
    state_machine: *mut NuxStateMachineInstance,
    name: *const c_char,
    apply: impl FnOnce(&mut StateMachineInstance, usize) -> bool,
) -> NuxStatus {
    let Some(state_machine) = (unsafe { state_machine.as_mut() }) else {
        return NuxStatus::NullArgument;
    };
    if name.is_null() {
        return NuxStatus::NullArgument;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return NuxStatus::InvalidArgument;
    };
    let Some(index) = state_machine.instance.input_index_named(name) else {
        return NuxStatus::NotFound;
    };
    if apply(&mut state_machine.instance, index) {
        NuxStatus::Ok
    } else {
        NuxStatus::InvalidArgument
    }
}

/// Advance the artboard while driving `state_machine`. The state machine must
/// have been created from the same artboard instance. `out_changed` is
/// optional and reports whether anything changed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_advance(
    instance: *mut NuxArtboardInstance,
    state_machine: *mut NuxStateMachineInstance,
    elapsed_seconds: f32,
    out_changed: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if let Some(out_changed) = unsafe { out_changed.as_mut() } {
            *out_changed = false;
        }
        let Some(instance) = (unsafe { instance.as_mut() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(state_machine) = (unsafe { state_machine.as_mut() }) else {
            return NuxStatus::NullArgument;
        };
        let changed = instance
            .instance
            .advance_with_state_machine(&mut state_machine.instance, elapsed_seconds);
        if let Some(out_changed) = unsafe { out_changed.as_mut() } {
            *out_changed = changed;
        }
        NuxStatus::Ok
    })
}

/// Deliver a pointer-down at artboard coordinates `(x, y)` to `state_machine`,
/// which must have been created from `instance`. `out_hit` is optional and
/// reports whether the event landed on a listener.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_pointer_down(
    instance: *const NuxArtboardInstance,
    state_machine: *mut NuxStateMachineInstance,
    x: f32,
    y: f32,
    out_hit: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        state_machine_pointer_event(
            instance,
            state_machine,
            out_hit,
            |state_machine, artboard| {
                state_machine.pointer_down(artboard.instance.raw(), x, y, DEFAULT_POINTER_ID)
            },
        )
    })
}

/// Deliver a pointer-move at artboard coordinates `(x, y)` to `state_machine`,
/// which must have been created from `instance`. `out_hit` is optional and
/// reports whether the event landed on a listener.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_pointer_move(
    instance: *const NuxArtboardInstance,
    state_machine: *mut NuxStateMachineInstance,
    x: f32,
    y: f32,
    out_hit: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        state_machine_pointer_event(
            instance,
            state_machine,
            out_hit,
            |state_machine, artboard| {
                state_machine.pointer_move(artboard.instance.raw(), x, y, 0.0, DEFAULT_POINTER_ID)
            },
        )
    })
}

/// Deliver a pointer-up at artboard coordinates `(x, y)` to `state_machine`,
/// which must have been created from `instance`. `out_hit` is optional and
/// reports whether the event landed on a listener.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_pointer_up(
    instance: *const NuxArtboardInstance,
    state_machine: *mut NuxStateMachineInstance,
    x: f32,
    y: f32,
    out_hit: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        state_machine_pointer_event(
            instance,
            state_machine,
            out_hit,
            |state_machine, artboard| {
                state_machine.pointer_up(artboard.instance.raw(), x, y, DEFAULT_POINTER_ID)
            },
        )
    })
}

fn state_machine_pointer_event(
    instance: *const NuxArtboardInstance,
    state_machine: *mut NuxStateMachineInstance,
    out_hit: *mut bool,
    dispatch: impl FnOnce(&mut StateMachineInstance, &NuxArtboardInstance) -> bool,
) -> NuxStatus {
    if let Some(out_hit) = unsafe { out_hit.as_mut() } {
        *out_hit = false;
    }
    let Some(instance) = (unsafe { instance.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    let Some(state_machine) = (unsafe { state_machine.as_mut() }) else {
        return NuxStatus::NullArgument;
    };
    let hit = dispatch(&mut state_machine.instance, instance);
    if let Some(out_hit) = unsafe { out_hit.as_mut() } {
        *out_hit = hit;
    }
    NuxStatus::Ok
}

/// Instantiate the artboard's view model with generated defaults (mirrors
/// `createDefaultViewModelInstance`). Returns `NUX_STATUS_NOT_FOUND` when the
/// artboard declares no view model. Free with `nux_view_model_instance_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_new_default(
    instance: *const NuxArtboardInstance,
    out_view_model: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        view_model_instance_new(out_view_model, || {
            let instance = unsafe { instance.as_ref() }?;
            instance.instance.instantiate_view_model()
        })
    })
}

/// Instantiate the artboard's view model from the source instance at
/// `instance_index` (the order the instances appear in the file). Returns
/// `NUX_STATUS_NOT_FOUND` when the artboard declares no view model or the
/// index is out of range. Free with `nux_view_model_instance_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_new_instance(
    instance: *const NuxArtboardInstance,
    instance_index: usize,
    out_view_model: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        view_model_instance_new(out_view_model, || {
            let instance = unsafe { instance.as_ref() }?;
            instance
                .instance
                .instantiate_view_model_instance(instance_index)
        })
    })
}

fn view_model_instance_new(
    out_view_model: *mut *mut NuxViewModelInstance,
    build: impl FnOnce() -> Option<ViewModelInstance>,
) -> NuxStatus {
    if out_view_model.is_null() {
        return NuxStatus::NullArgument;
    }
    unsafe {
        *out_view_model = ptr::null_mut();
    }
    let Some(view_model) = build() else {
        return NuxStatus::NotFound;
    };
    unsafe {
        *out_view_model = Box::into_raw(Box::new(NuxViewModelInstance {
            instance: view_model,
        }));
    }
    NuxStatus::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_free(view_model: *mut NuxViewModelInstance) {
    ffi_guard((), || {
        if view_model.is_null() {
            return;
        }
        unsafe {
            drop(Box::from_raw(view_model));
        }
    })
}

/// Set a number property by NUL-terminated UTF-8 name path (`/`-separated for
/// nested view models). Returns `NUX_STATUS_NOT_FOUND` when no settable number
/// property matches the path.
///
/// Note: for the mutation to reach the artboard, call
/// `nux_artboard_instance_bind_view_model` after setting and before advancing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_set_number(
    view_model: *mut NuxViewModelInstance,
    name_path: *const c_char,
    value: f32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        view_model_set(view_model, name_path, |view_model, name| {
            let changed = view_model.instance.set_number(name, value);
            changed
                || view_model
                    .instance
                    .raw()
                    .number_source_handle_by_property_name_path(name)
                    .is_some()
        })
    })
}

/// Set a boolean property by NUL-terminated UTF-8 name path (`/`-separated for
/// nested view models). Returns `NUX_STATUS_NOT_FOUND` when no settable
/// boolean property matches the path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_set_bool(
    view_model: *mut NuxViewModelInstance,
    name_path: *const c_char,
    value: bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        view_model_set(view_model, name_path, |view_model, name| {
            let changed = view_model.instance.set_bool(name, value);
            changed
                || view_model
                    .instance
                    .raw()
                    .boolean_source_handle_by_property_name_path(name)
                    .is_some()
        })
    })
}

/// Set a string property by NUL-terminated UTF-8 name path (`/`-separated for
/// nested view models). `value` is a NUL-terminated UTF-8 string. Returns
/// `NUX_STATUS_NOT_FOUND` when no settable string property matches the path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_set_string(
    view_model: *mut NuxViewModelInstance,
    name_path: *const c_char,
    value: *const c_char,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if value.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(value) = (unsafe { CStr::from_ptr(value) }).to_str() else {
            return NuxStatus::InvalidArgument;
        };
        view_model_set(view_model, name_path, |view_model, name| {
            let changed = view_model.instance.set_string(name, value);
            changed
                || view_model
                    .instance
                    .raw()
                    .string_source_handle_by_property_name_path(name)
                    .is_some()
        })
    })
}

fn view_model_set(
    view_model: *mut NuxViewModelInstance,
    name_path: *const c_char,
    apply: impl FnOnce(&mut NuxViewModelInstance, &str) -> bool,
) -> NuxStatus {
    let Some(view_model) = (unsafe { view_model.as_mut() }) else {
        return NuxStatus::NullArgument;
    };
    if name_path.is_null() {
        return NuxStatus::NullArgument;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name_path) }).to_str() else {
        return NuxStatus::InvalidArgument;
    };
    if apply(view_model, name) {
        NuxStatus::Ok
    } else {
        NuxStatus::NotFound
    }
}

/// Bind `view_model` to `instance`'s own data binds and nested-artboard
/// contexts (mirrors `artboard->bindViewModelInstance(...)`). The context is
/// copied in, so call this again after mutating `view_model` to propagate the
/// change on the next advance. `view_model` must have been created from
/// `instance`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_bind_view_model(
    instance: *mut NuxArtboardInstance,
    view_model: *const NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(instance) = (unsafe { instance.as_mut() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(view_model) = (unsafe { view_model.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        instance.instance.bind_view_model(&view_model.instance);
        NuxStatus::Ok
    })
}

#[cfg(test)]
mod firewall_tests {
    use super::*;

    // A deliberately-panicking internal path must surface as the function's
    // error value instead of unwinding across the C ABI boundary. This runs in
    // the dev profile (`debug_assertions`, unwinding enabled), which is exactly
    // the build where the firewall does real work.
    #[test]
    fn ffi_guard_converts_panic_to_error_status() {
        let status = ffi_guard(NuxStatus::RuntimeError, || -> NuxStatus {
            panic!("deliberate panic on an internal path");
        });
        assert_eq!(status, NuxStatus::RuntimeError);
    }

    #[test]
    fn ffi_guard_converts_panic_for_void_return() {
        // Must not propagate the unwind (would abort the test process if it did).
        ffi_guard((), || {
            panic!("deliberate panic on a void internal path");
        });
    }

    #[test]
    fn ffi_guard_passes_through_success_value() {
        let status = ffi_guard(NuxStatus::RuntimeError, || NuxStatus::Ok);
        assert_eq!(status, NuxStatus::Ok);
    }
}
