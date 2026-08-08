#![allow(
    clippy::arc_with_non_send_sync,
    reason = "OwnedArtboardInstance's native ownership API requires Arc<File>; C handles remain creator-thread affine"
)]
#![allow(
    clippy::missing_safety_doc,
    reason = "the centralized pointer/thread/lifetime safety contract is emitted in the public nux_capi.h umbrella header"
)]

mod render_callbacks;

pub use render_callbacks::{
    NUX_RENDER_CALLBACKS_V3_MIN_SIZE, NuxImageSampler, NuxRawPathView, NuxRenderCallbacks,
};

use nuxie::{
    File, LinearAnimationInstance, OwnedArtboardInstance, StateMachineInstance, ViewModelInstance,
};
use render_callbacks::{CallbackFactory, CallbackRenderer};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_void};
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use std::rc::Rc;
use std::slice;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, ThreadId};

/// Increment only for a breaking change to the exported C contract.
pub const NUX_CAPI_ABI_VERSION: u32 = 3;

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
/// This is profile-independent by design. The shipped profiles use
/// `panic = "unwind"` because the embedded Luau implementation has protected
/// error paths built on unwinding, so this guard remains active in production.
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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxStatus {
    Ok = 0,
    NullArgument = 1,
    ImportError = 2,
    NotFound = 3,
    RuntimeError = 4,
    InvalidArgument = 5,
    AbiMismatch = 6,
    WrongThread = 7,
    InvalidStructSize = 8,
    HandleMismatch = 9,
    ReentrantCall = 10,
}

pub struct NuxFile {
    file: Arc<File>,
    owner_thread: ThreadId,
}

struct ArtboardOccurrence {
    instance: RefCell<OwnedArtboardInstance>,
    renderer_domain: Cell<Option<RendererDomainBinding>>,
    active: Cell<bool>,
    poisoned: Cell<bool>,
}

/// Renderer-owned resources cached by an occurrence belong to exactly one
/// backend domain. Additional backend variants can be added here without
/// changing the public lifecycle model.
#[derive(Clone, Copy)]
enum RendererDomainBinding {
    Callbacks {
        descriptor: *const NuxRenderCallbacks,
        table: NuxRenderCallbacks,
    },
}

struct OccurrenceCallGuard<'a> {
    occurrence: &'a ArtboardOccurrence,
}

impl Drop for OccurrenceCallGuard<'_> {
    fn drop(&mut self) {
        self.occurrence.active.set(false);
        if thread::panicking() {
            self.occurrence.poisoned.set(true);
        }
    }
}

fn enter_occurrence(occurrence: &ArtboardOccurrence) -> Result<OccurrenceCallGuard<'_>, NuxStatus> {
    if occurrence.poisoned.get() {
        return Err(NuxStatus::RuntimeError);
    }
    if occurrence.active.replace(true) {
        return Err(NuxStatus::ReentrantCall);
    }
    Ok(OccurrenceCallGuard { occurrence })
}

/// Owned artboard occurrence. It retains the imported [`File`] through native
/// shared ownership and therefore remains valid after its [`NuxFile`] handle
/// is released.
pub struct NuxArtboardInstance {
    occurrence: Rc<ArtboardOccurrence>,
    owner_thread: ThreadId,
    provenance: Arc<()>,
}

/// Owned state machine instance. Advance it through the
/// [`NuxArtboardInstance`] it was created from.
pub struct NuxStateMachineInstance {
    instance: RefCell<StateMachineInstance>,
    owner_thread: ThreadId,
    provenance: Arc<()>,
}

/// Owned linear-animation occurrence selected from an artboard.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxPlayerKind {
    StaticArtboard = 0,
    StateMachine = 1,
    LinearAnimation = 2,
}

enum PlayerInstance {
    StaticArtboard,
    StateMachine(Box<StateMachineInstance>),
    LinearAnimation(Box<LinearAnimationInstance>),
}

enum DefaultSceneSelection<S, A> {
    StateMachine { index: usize, instance: S },
    LinearAnimation { index: usize, instance: A },
    StaticArtboard,
}

/// Pinned C++ `defaultScene` order. Keeping the decision in this small generic
/// helper makes every branch testable without teaching the C layer how to
/// synthesize binary Rive files.
fn select_default_scene<C, S, A>(
    context: &mut C,
    authored_default: Option<usize>,
    mut state_machine_at: impl FnMut(&mut C, usize) -> Option<S>,
    mut animation_at: impl FnMut(&mut C, usize) -> Option<A>,
) -> DefaultSceneSelection<S, A> {
    if let Some(index) = authored_default
        && let Some(instance) = state_machine_at(context, index)
    {
        return DefaultSceneSelection::StateMachine { index, instance };
    }
    if let Some(instance) = state_machine_at(context, 0) {
        return DefaultSceneSelection::StateMachine { index: 0, instance };
    }
    if let Some(instance) = animation_at(context, 0) {
        return DefaultSceneSelection::LinearAnimation { index: 0, instance };
    }
    DefaultSceneSelection::StaticArtboard
}

/// Product-neutral selected player. This surface establishes selection,
/// ownership, and metadata; playback operations are exposed separately.
pub struct NuxPlayer {
    instance: RefCell<PlayerInstance>,
    artboard: Rc<ArtboardOccurrence>,
    owner_thread: ThreadId,
    provenance: Arc<()>,
    selection_index: usize,
    selection_name: Box<[u8]>,
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
    instance: RefCell<ViewModelInstance>,
    owner_thread: ThreadId,
    provenance: Arc<()>,
}

fn require_owner_thread(owner_thread: ThreadId) -> Result<(), NuxStatus> {
    if thread::current().id() == owner_thread {
        Ok(())
    } else {
        Err(NuxStatus::WrongThread)
    }
}

fn struct_size_supports(caller_size: u32, minimum_size: usize) -> bool {
    usize::try_from(caller_size).is_ok_and(|caller_size| caller_size >= minimum_size)
}

/// Copy only the prefix that both this runtime and the caller provide. This is
/// what lets ABI-v3 append fields without an older caller being overwritten by
/// a newer runtime.
unsafe fn write_caller_struct<T>(
    out: *mut T,
    value: &T,
    minimum_size: usize,
) -> Result<(), NuxStatus> {
    if out.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let caller_size = unsafe { out.cast::<u32>().read() };
    if !struct_size_supports(caller_size, minimum_size) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let write_len = usize::try_from(caller_size)
        .unwrap_or(usize::MAX)
        .min(std::mem::size_of::<T>());
    unsafe {
        ptr::copy_nonoverlapping(
            (value as *const T).cast::<u8>(),
            out.cast::<u8>(),
            write_len,
        );
    }
    Ok(())
}

/// Read a versioned callback-table prefix without first forming a reference to
/// the runtime's possibly larger current struct.
unsafe fn read_render_callbacks(
    callbacks: *const NuxRenderCallbacks,
) -> Result<NuxRenderCallbacks, NuxStatus> {
    if callbacks.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let caller_size = unsafe { callbacks.cast::<u32>().read() };
    if !struct_size_supports(caller_size, NUX_RENDER_CALLBACKS_V3_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let mut value = NuxRenderCallbacks::default();
    let read_len = usize::try_from(caller_size)
        .unwrap_or(usize::MAX)
        .min(std::mem::size_of::<NuxRenderCallbacks>());
    unsafe {
        ptr::copy_nonoverlapping(
            callbacks.cast::<u8>(),
            (&mut value as *mut NuxRenderCallbacks).cast::<u8>(),
            read_len,
        );
    }
    Ok(value)
}

fn require_same_artboard(left: &Arc<()>, right: &Arc<()>) -> Result<(), NuxStatus> {
    if Arc::ptr_eq(left, right) {
        Ok(())
    } else {
        Err(NuxStatus::HandleMismatch)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HandleKind {
    File,
    Artboard,
    StateMachine,
    Player,
    ViewModel,
    Result,
}

struct HandleRecord {
    kind: HandleKind,
    owner_thread: ThreadId,
    active: bool,
    poisoned: bool,
}

static HANDLE_REGISTRY: OnceLock<Mutex<HashMap<usize, HandleRecord>>> = OnceLock::new();

fn handle_registry() -> &'static Mutex<HashMap<usize, HandleRecord>> {
    HANDLE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn registry_lock() -> std::sync::MutexGuard<'static, HashMap<usize, HandleRecord>> {
    handle_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn register_handle<T>(handle: *mut T, kind: HandleKind, owner_thread: ThreadId) {
    registry_lock().insert(
        handle as usize,
        HandleRecord {
            kind,
            owner_thread,
            active: false,
            poisoned: false,
        },
    );
}

struct HandleCallGuard {
    address: usize,
}

impl Drop for HandleCallGuard {
    fn drop(&mut self) {
        if let Some(record) = registry_lock().get_mut(&self.address) {
            record.active = false;
            record.poisoned |= thread::panicking();
        }
    }
}

fn enter_handle<T>(handle: *const T, kind: HandleKind) -> Result<HandleCallGuard, NuxStatus> {
    if handle.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let address = handle as usize;
    let mut registry = registry_lock();
    let Some(record) = registry.get_mut(&address) else {
        return Err(NuxStatus::HandleMismatch);
    };
    if record.kind != kind {
        return Err(NuxStatus::HandleMismatch);
    }
    if record.owner_thread != thread::current().id() {
        return Err(NuxStatus::WrongThread);
    }
    if record.active {
        return Err(NuxStatus::ReentrantCall);
    }
    if record.poisoned {
        return Err(NuxStatus::RuntimeError);
    }
    record.active = true;
    Ok(HandleCallGuard { address })
}

fn remove_handle<T>(handle: *mut T, kind: HandleKind) -> Result<(), NuxStatus> {
    if handle.is_null() {
        return Ok(());
    }
    let address = handle as usize;
    let mut registry = registry_lock();
    let Some(record) = registry.get(&address) else {
        return Err(NuxStatus::HandleMismatch);
    };
    if record.kind != kind {
        return Err(NuxStatus::HandleMismatch);
    }
    if record.owner_thread != thread::current().id() {
        return Err(NuxStatus::WrongThread);
    }
    if record.active {
        return Err(NuxStatus::ReentrantCall);
    }
    registry.remove(&address);
    Ok(())
}

macro_rules! enter_status_handle {
    ($handle:expr, $kind:expr) => {
        match enter_handle($handle, $kind) {
            Ok(guard) => guard,
            Err(status) => return status,
        }
    };
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

fn with_utf8_view<R>(view: NuxStringView, body: impl FnOnce(&str) -> R) -> Result<R, NuxStatus> {
    if view.data.is_null() && view.len != 0 {
        return Err(NuxStatus::NullArgument);
    }
    let bytes = if view.len == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(view.data.cast::<u8>(), view.len) }
    };
    let value = std::str::from_utf8(bytes).map_err(|_| NuxStatus::InvalidArgument)?;
    Ok(body(value))
}

/// Immutable identity embedded into the shipped runtime binary.
///
/// Both strings have process-static lifetime and are not NUL-terminated; C
/// callers must respect their explicit lengths.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxRuntimeInfo {
    /// Must be initialized to `sizeof(NuxRuntimeInfo)` by the caller.
    pub struct_size: u32,
    pub abi_version: u32,
    pub runtime_version: NuxStringView,
    pub source_revision: NuxStringView,
}

pub const NUX_RUNTIME_INFO_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxRuntimeInfo, source_revision) + std::mem::size_of::<NuxStringView>();

impl Default for NuxRuntimeInfo {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            abi_version: 0,
            runtime_version: NuxStringView::default(),
            source_revision: NuxStringView::default(),
        }
    }
}

/// Versioned metadata for a selected runtime-native player.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxPlayerInfo {
    /// Must be initialized to `sizeof(NuxPlayerInfo)` by the caller.
    pub struct_size: u32,
    pub kind: NuxPlayerKind,
    /// Authored index within the selected kind, or `SIZE_MAX` for static.
    pub index: usize,
    /// Copied UTF-8 metadata owned by `NuxPlayer`; valid until player free.
    pub name: NuxStringView,
}

pub const NUX_PLAYER_INFO_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxPlayerInfo, name) + std::mem::size_of::<NuxStringView>();

/// Caller-sized view into one owned C-ABI result. `code` and `message` remain
/// valid until the owning `NuxCapiResult` is released.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxCapiDiagnosticView {
    pub struct_size: u32,
    pub status: NuxStatus,
    pub code: NuxStringView,
    pub message: NuxStringView,
}

pub const NUX_CAPI_DIAGNOSTIC_VIEW_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxCapiDiagnosticView, message) + std::mem::size_of::<NuxStringView>();

impl Default for NuxCapiDiagnosticView {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            status: NuxStatus::Ok,
            code: NuxStringView::default(),
            message: NuxStringView::default(),
        }
    }
}

/// Bounded, library-owned diagnostic/result storage.
pub struct NuxCapiResult {
    status: NuxStatus,
    code: Box<[u8]>,
    message: Box<[u8]>,
}

const MAX_DIAGNOSTIC_BYTES: usize = 4 * 1024;

fn bounded_diagnostic_bytes(value: impl AsRef<[u8]>) -> Box<[u8]> {
    let value = value.as_ref();
    let mut end = value.len().min(MAX_DIAGNOSTIC_BYTES);
    if let Ok(text) = std::str::from_utf8(value) {
        while !text.is_char_boundary(end) {
            end = end.saturating_sub(1);
        }
    }
    value
        .get(..end)
        .unwrap_or(value)
        .to_vec()
        .into_boxed_slice()
}

fn status_code(status: NuxStatus) -> &'static str {
    match status {
        NuxStatus::Ok => "nux_capi.ok",
        NuxStatus::NullArgument => "nux_capi.null_argument",
        NuxStatus::ImportError => "nux_capi.import_error",
        NuxStatus::NotFound => "nux_capi.not_found",
        NuxStatus::RuntimeError => "nux_capi.runtime_error",
        NuxStatus::InvalidArgument => "nux_capi.invalid_argument",
        NuxStatus::AbiMismatch => "nux_capi.abi_mismatch",
        NuxStatus::WrongThread => "nux_capi.wrong_thread",
        NuxStatus::InvalidStructSize => "nux_capi.invalid_struct_size",
        NuxStatus::HandleMismatch => "nux_capi.handle_mismatch",
        NuxStatus::ReentrantCall => "nux_capi.reentrant_call",
    }
}

fn publish_result(
    out_result: *mut *mut NuxCapiResult,
    status: NuxStatus,
    message: impl AsRef<[u8]>,
) {
    let result = Box::new(NuxCapiResult {
        status,
        code: bounded_diagnostic_bytes(status_code(status)),
        message: bounded_diagnostic_bytes(message),
    });
    let result = Box::into_raw(result);
    register_handle(result, HandleKind::Result, thread::current().id());
    unsafe { *out_result = result };
}

/// Panic firewall for the one API that publishes both a file handle and an
/// owned diagnostic. Any partially published handles are reclaimed before a
/// panic result is exposed to the caller.
fn ffi_guard_with_handle_result<T>(
    out_handle: *mut *mut T,
    out_result: *mut *mut NuxCapiResult,
    handle_kind: HandleKind,
    body: impl FnOnce() -> NuxStatus,
) -> NuxStatus {
    if !out_handle.is_null()
        && !out_result.is_null()
        && out_handle.cast::<c_void>() == out_result.cast::<c_void>()
    {
        // Both slots have pointer representation/alignment. Clear the shared
        // storage once and reject before either owned handle can be published.
        unsafe { *out_handle = ptr::null_mut() };
        return NuxStatus::InvalidArgument;
    }
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(status) => status,
        Err(_) => {
            if !out_handle.is_null() {
                let published_handle = unsafe { *out_handle };
                if !published_handle.is_null()
                    && remove_handle(published_handle, handle_kind).is_ok()
                {
                    unsafe { drop(Box::from_raw(published_handle)) };
                }
                unsafe { *out_handle = ptr::null_mut() };
            }
            if !out_result.is_null() {
                let published_result = unsafe { *out_result };
                if !published_result.is_null()
                    && remove_handle(published_result, HandleKind::Result).is_ok()
                {
                    unsafe { drop(Box::from_raw(published_result)) };
                }
                unsafe { *out_result = ptr::null_mut() };
                publish_result(
                    out_result,
                    NuxStatus::RuntimeError,
                    "Rust panic contained at nux-capi boundary",
                );
            }
            NuxStatus::RuntimeError
        }
    }
}

impl Default for NuxPlayerInfo {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            kind: NuxPlayerKind::StaticArtboard,
            index: usize::MAX,
            name: NuxStringView::default(),
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
        let expected_size =
            u32::try_from(std::mem::size_of::<NuxRuntimeInfo>()).unwrap_or(u32::MAX);
        let value = NuxRuntimeInfo {
            struct_size: expected_size,
            abi_version: NUX_CAPI_ABI_VERSION,
            runtime_version: NuxStringView::from_static(RUNTIME_VERSION),
            source_revision: NuxStringView::from_static(SOURCE_REVISION),
        };
        unsafe { write_caller_struct(out_info, &value, NUX_RUNTIME_INFO_V3_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
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
                let handle = Box::new(NuxFile {
                    file: Arc::new(file),
                    owner_thread: thread::current().id(),
                });
                unsafe {
                    let handle = Box::into_raw(handle);
                    register_handle(handle, HandleKind::File, thread::current().id());
                    *out_file = handle;
                }
                NuxStatus::Ok
            }
            Err(_) => NuxStatus::ImportError,
        }
    })
}

/// Diagnostic import path for production consumers. `out_file` is published
/// only on success. `out_result` is always published and owns a bounded status
/// code/message until released with `nux_capi_result_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_import_with_result(
    bytes: *const u8,
    len: usize,
    out_file: *mut *mut NuxFile,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(out_file, out_result, HandleKind::File, || {
        if !out_file.is_null() {
            unsafe { *out_file = ptr::null_mut() };
        }
        if !out_result.is_null() {
            unsafe { *out_result = ptr::null_mut() };
        }
        if out_file.is_null() || out_result.is_null() {
            if !out_result.is_null() {
                publish_result(
                    out_result,
                    NuxStatus::NullArgument,
                    "an output pointer is null",
                );
            }
            return NuxStatus::NullArgument;
        }
        if bytes.is_null() && len != 0 {
            publish_result(out_result, NuxStatus::NullArgument, "bytes is null");
            return NuxStatus::NullArgument;
        }
        let bytes = if len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(bytes, len) }
        };
        match File::import(bytes) {
            Ok(file) => {
                let handle = Box::into_raw(Box::new(NuxFile {
                    file: Arc::new(file),
                    owner_thread: thread::current().id(),
                }));
                register_handle(handle, HandleKind::File, thread::current().id());
                unsafe { *out_file = handle };
                publish_result(out_result, NuxStatus::Ok, "");
                NuxStatus::Ok
            }
            Err(error) => {
                publish_result(out_result, NuxStatus::ImportError, error.to_string());
                NuxStatus::ImportError
            }
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_capi_result_status(
    result: *const NuxCapiResult,
    out_status: *mut NuxStatus,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_status.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_status = NuxStatus::RuntimeError };
        let _result_call = enter_status_handle!(result, HandleKind::Result);
        let Some(result) = (unsafe { result.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        unsafe { *out_status = result.status };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_capi_result_diagnostic(
    result: *const NuxCapiResult,
    out_diagnostic: *mut NuxCapiDiagnosticView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let expected_size =
            u32::try_from(std::mem::size_of::<NuxCapiDiagnosticView>()).unwrap_or(u32::MAX);
        let _result_call = enter_status_handle!(result, HandleKind::Result);
        let Some(result) = (unsafe { result.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let value = NuxCapiDiagnosticView {
            struct_size: expected_size,
            status: result.status,
            code: NuxStringView {
                data: result.code.as_ptr().cast(),
                len: result.code.len(),
            },
            message: NuxStringView {
                data: result.message.as_ptr().cast(),
                len: result.message.len(),
            },
        };
        unsafe { write_caller_struct(out_diagnostic, &value, NUX_CAPI_DIAGNOSTIC_VIEW_V3_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_capi_result_free(result: *mut NuxCapiResult) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if result.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(result, HandleKind::Result) {
            return status;
        }
        unsafe { drop(Box::from_raw(result)) };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_free(file: *mut NuxFile) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if file.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(file, HandleKind::File) {
            return status;
        }
        unsafe {
            drop(Box::from_raw(file));
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_artboard_count(
    file: *const NuxFile,
    out_count: *mut usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_count.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_count = 0 };
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        unsafe { *out_count = file.file.artboard_count() };
        NuxStatus::Ok
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
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(file.owner_thread) {
            return status;
        }
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

/// Name of one of an artboard's linear animations. The returned
/// length-delimited UTF-8 view borrows `file` and expires when the file handle
/// is freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_artboard_animation_name(
    file: *const NuxFile,
    artboard_index: usize,
    animation_index: usize,
    out_name: *mut NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_name.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_name = NuxStringView::default() };
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(artboard) = file.file.artboard(artboard_index) else {
            return NuxStatus::NotFound;
        };
        let Some(name) = artboard.animation_name(animation_index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_name = NuxStringView {
                data: name.as_ptr().cast(),
                len: name.len(),
            };
        }
        NuxStatus::Ok
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
    let _file_call = match enter_handle(file, HandleKind::File) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Some(file) = (unsafe { file.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    if let Err(status) = require_owner_thread(file.owner_thread) {
        return status;
    }
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
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(file.owner_thread) {
            return status;
        }
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

/// Instantiate the artboard at `artboard_index`. The returned occurrence
/// retains the imported file and may outlive the `NuxFile` handle.
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
        let _file_call = enter_status_handle!(file, HandleKind::File);
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(file.owner_thread) {
            return status;
        }
        match OwnedArtboardInstance::instantiate(Arc::clone(&file.file), artboard_index) {
            Ok(instance) => {
                let handle = NuxArtboardInstance {
                    occurrence: Rc::new(ArtboardOccurrence {
                        instance: RefCell::new(instance),
                        renderer_domain: Cell::new(None),
                        active: Cell::new(false),
                        poisoned: Cell::new(false),
                    }),
                    owner_thread: file.owner_thread,
                    provenance: Arc::new(()),
                };
                unsafe {
                    let handle = Box::into_raw(Box::new(handle));
                    register_handle(handle, HandleKind::Artboard, file.owner_thread);
                    *out_instance = handle;
                }
                NuxStatus::Ok
            }
            Err(_) if artboard_index >= file.file.artboard_count() => NuxStatus::NotFound,
            Err(_) => NuxStatus::RuntimeError,
        }
    })
}

/// Instantiate the first artboard whose authored name exactly matches the
/// length-delimited UTF-8 `name`. Matching is case-sensitive and has no
/// fallback; an empty name is a valid selector.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_new_named(
    file: *const NuxFile,
    name: NuxStringView,
    out_instance: *mut *mut NuxArtboardInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_instance = ptr::null_mut() };
        let index = match with_utf8_view(name, |name| -> Result<usize, NuxStatus> {
            let _file_call = enter_handle(file, HandleKind::File)?;
            let file_ref = unsafe { file.as_ref() }.ok_or(NuxStatus::NullArgument)?;
            file_ref
                .file
                .artboard_named(name)
                .map(|artboard| artboard.index())
                .ok_or(NuxStatus::NotFound)
        }) {
            Ok(Ok(index)) => index,
            Ok(Err(status)) => return status,
            Err(status) => return status,
        };
        unsafe { nux_artboard_instance_new(file, index, out_instance) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_free(
    instance: *mut NuxArtboardInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if instance.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(instance, HandleKind::Artboard) {
            return status;
        }
        unsafe { drop(Box::from_raw(instance)) };
        NuxStatus::Ok
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
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let changed = artboard.advance(elapsed_seconds);
        if let Some(out_changed) = unsafe { out_changed.as_mut() } {
            *out_changed = changed;
        }
        NuxStatus::Ok
    })
}

/// Draw the artboard through the caller-provided render vtable. See
/// `NuxRenderCallbacks` for the ownership and handle contract. The first
/// draw's renderer domain is retained by the Artboard occurrence, even if that
/// draw returns an error. A different descriptor/domain returns
/// `NUX_STATUS_HANDLE_MISMATCH`; a future explicit reset can make switching
/// domains safe. Callback functions and `user_data` must remain valid until
/// the last artboard/player retaining the occurrence is freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_draw(
    instance: *mut NuxArtboardInstance,
    callbacks: *const NuxRenderCallbacks,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let callbacks_source = callbacks;
        let callbacks = match unsafe { read_render_callbacks(callbacks) } {
            Ok(callbacks) => callbacks,
            Err(status) => return status,
        };
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let retained_callbacks = match instance.occurrence.renderer_domain.get() {
            Some(RendererDomainBinding::Callbacks { descriptor, table }) => {
                if descriptor != callbacks_source {
                    return NuxStatus::HandleMismatch;
                }
                table
            }
            None => {
                instance
                    .occurrence
                    .renderer_domain
                    .set(Some(RendererDomainBinding::Callbacks {
                        descriptor: callbacks_source,
                        table: callbacks,
                    }));
                callbacks
            }
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let mut factory = CallbackFactory::new(retained_callbacks);
        let mut renderer = CallbackRenderer::new(retained_callbacks);
        match artboard.draw(&mut factory, &mut renderer) {
            Ok(()) => NuxStatus::Ok,
            Err(_) => NuxStatus::RuntimeError,
        }
    })
}

/// Instantiate the state machine at `state_machine_index` on the instance's
/// artboard. Free with `nux_state_machine_instance_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_new(
    instance: *mut NuxArtboardInstance,
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
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let Some(state_machine) = artboard.state_machine_instance(state_machine_index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            let handle = Box::into_raw(Box::new(NuxStateMachineInstance {
                instance: RefCell::new(state_machine),
                owner_thread: instance.owner_thread,
                provenance: Arc::clone(&instance.provenance),
            }));
            register_handle(handle, HandleKind::StateMachine, instance.owner_thread);
            *out_state_machine = handle;
        }
        NuxStatus::Ok
    })
}

/// Instantiate the first state machine with an exact, case-sensitive authored
/// name. There is no default or cross-kind fallback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_new_named(
    instance: *mut NuxArtboardInstance,
    name: NuxStringView,
    out_state_machine: *mut *mut NuxStateMachineInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_state_machine.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_state_machine = ptr::null_mut() };
        let index = match with_utf8_view(name, |name| -> Result<usize, NuxStatus> {
            let _instance_call = enter_handle(instance, HandleKind::Artboard)?;
            let artboard_handle = unsafe { instance.as_ref() }.ok_or(NuxStatus::NullArgument)?;
            let _occurrence_call = enter_occurrence(&artboard_handle.occurrence)?;
            let artboard = artboard_handle
                .occurrence
                .instance
                .try_borrow()
                .map_err(|_| NuxStatus::ReentrantCall)?;
            artboard
                .artboard()
                .state_machine_index_named(name)
                .ok_or(NuxStatus::NotFound)
        }) {
            Ok(Ok(index)) => index,
            Ok(Err(status)) => return status,
            Err(status) => return status,
        };
        unsafe { nux_state_machine_instance_new(instance, index, out_state_machine) }
    })
}

/// Instantiate only the artboard's authored, valid default state machine.
/// Returns `NUX_STATUS_NOT_FOUND` when no valid default was authored; this
/// function does not fall back to state-machine zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_new_default(
    instance: *mut NuxArtboardInstance,
    out_state_machine: *mut *mut NuxStateMachineInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_state_machine.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_state_machine = ptr::null_mut();
        }
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let Some(default_index) = artboard.artboard().default_state_machine_index() else {
            return NuxStatus::NotFound;
        };
        let Some(state_machine) = artboard.state_machine_instance(default_index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            let handle = Box::into_raw(Box::new(NuxStateMachineInstance {
                instance: RefCell::new(state_machine),
                owner_thread: instance.owner_thread,
                provenance: Arc::clone(&instance.provenance),
            }));
            register_handle(handle, HandleKind::StateMachine, instance.owner_thread);
            *out_state_machine = handle;
        }
        NuxStatus::Ok
    })
}

fn publish_player(
    artboard: &NuxArtboardInstance,
    player: PlayerInstance,
    selection_index: usize,
    selection_name: &str,
    out_player: *mut *mut NuxPlayer,
) -> NuxStatus {
    unsafe {
        let handle = Box::into_raw(Box::new(NuxPlayer {
            instance: RefCell::new(player),
            artboard: Rc::clone(&artboard.occurrence),
            owner_thread: artboard.owner_thread,
            provenance: Arc::clone(&artboard.provenance),
            selection_index,
            selection_name: selection_name.as_bytes().to_vec().into_boxed_slice(),
        }));
        register_handle(handle, HandleKind::Player, artboard.owner_thread);
        *out_player = handle;
    }
    NuxStatus::Ok
}

/// Select the artboard's default scene using the pinned C++ order: authored
/// valid default state machine, state machine zero, animation zero, then a
/// static artboard when it has no playable animation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_default(
    instance: *mut NuxArtboardInstance,
    out_player: *mut *mut NuxPlayer,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_player.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_player = ptr::null_mut() };
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let authored_default = artboard.artboard().default_state_machine_index();
        let selection = select_default_scene(
            &mut *artboard,
            authored_default,
            |artboard, index| artboard.state_machine_instance(index),
            |artboard, index| artboard.linear_animation_instance(index),
        );
        let (player, index, name) = match selection {
            DefaultSceneSelection::StateMachine { index, instance } => {
                let name = artboard
                    .artboard()
                    .state_machine_name(index)
                    .unwrap_or("")
                    .to_owned();
                (
                    PlayerInstance::StateMachine(Box::new(instance)),
                    index,
                    name,
                )
            }
            DefaultSceneSelection::LinearAnimation { index, instance } => {
                let name = artboard
                    .artboard()
                    .animation_name(index)
                    .unwrap_or("")
                    .to_owned();
                (
                    PlayerInstance::LinearAnimation(Box::new(instance)),
                    index,
                    name,
                )
            }
            DefaultSceneSelection::StaticArtboard => {
                (PlayerInstance::StaticArtboard, usize::MAX, String::new())
            }
        };
        drop(artboard);
        publish_player(instance, player, index, &name, out_player)
    })
}

/// Explicitly select a static artboard even when it also contains authored
/// state machines or linear animations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_static(
    instance: *mut NuxArtboardInstance,
    out_player: *mut *mut NuxPlayer,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_player.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_player = ptr::null_mut() };
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        publish_player(
            instance,
            PlayerInstance::StaticArtboard,
            usize::MAX,
            "",
            out_player,
        )
    })
}

/// Select a state machine by exact, case-sensitive, length-delimited UTF-8
/// name. No default, index-zero, or animation fallback is performed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_state_machine_named(
    instance: *mut NuxArtboardInstance,
    name: NuxStringView,
    out_player: *mut *mut NuxPlayer,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_player.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_player = ptr::null_mut() };
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        match with_utf8_view(name, |name| {
            let _occurrence_call = match enter_occurrence(&instance.occurrence) {
                Ok(guard) => guard,
                Err(status) => return status,
            };
            let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
                return NuxStatus::ReentrantCall;
            };
            let Some(index) = artboard.artboard().state_machine_index_named(name) else {
                return NuxStatus::NotFound;
            };
            let Some(machine) = artboard.state_machine_instance(index) else {
                return NuxStatus::NotFound;
            };
            drop(artboard);
            publish_player(
                instance,
                PlayerInstance::StateMachine(Box::new(machine)),
                index,
                name,
                out_player,
            )
        }) {
            Ok(status) => status,
            Err(status) => status,
        }
    })
}

/// Select a linear animation by exact, case-sensitive, length-delimited UTF-8
/// name. No state-machine or index fallback is performed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_linear_animation_named(
    instance: *mut NuxArtboardInstance,
    name: NuxStringView,
    out_player: *mut *mut NuxPlayer,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_player.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_player = ptr::null_mut() };
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        match with_utf8_view(name, |name| {
            let _occurrence_call = match enter_occurrence(&instance.occurrence) {
                Ok(guard) => guard,
                Err(status) => return status,
            };
            let Ok(artboard) = instance.occurrence.instance.try_borrow() else {
                return NuxStatus::ReentrantCall;
            };
            let Some(index) = artboard.artboard().animation_index_named(name) else {
                return NuxStatus::NotFound;
            };
            let Some(animation) = artboard.linear_animation_instance(index) else {
                return NuxStatus::NotFound;
            };
            drop(artboard);
            publish_player(
                instance,
                PlayerInstance::LinearAnimation(Box::new(animation)),
                index,
                name,
                out_player,
            )
        }) {
            Ok(status) => status,
            Err(status) => status,
        }
    })
}

/// Read selected-player metadata into a versioned caller-owned struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_info(
    player: *const NuxPlayer,
    out_info: *mut NuxPlayerInfo,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let expected_size = u32::try_from(std::mem::size_of::<NuxPlayerInfo>()).unwrap_or(u32::MAX);
        let _player_call = enter_status_handle!(player, HandleKind::Player);
        let Some(player) = (unsafe { player.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(player.owner_thread) {
            return status;
        }
        let Ok(player_instance) = player.instance.try_borrow() else {
            return NuxStatus::ReentrantCall;
        };
        let kind = match &*player_instance {
            PlayerInstance::StaticArtboard => NuxPlayerKind::StaticArtboard,
            PlayerInstance::StateMachine(_machine) => NuxPlayerKind::StateMachine,
            PlayerInstance::LinearAnimation(_animation) => NuxPlayerKind::LinearAnimation,
        };
        // These are deliberately retained by the player for future playback
        // operations and occurrence-lineage validation.
        let _retained_occurrence = &player.artboard;
        let _retained_lineage = &player.provenance;
        let value = NuxPlayerInfo {
            struct_size: expected_size,
            kind,
            index: player.selection_index,
            name: NuxStringView {
                data: player.selection_name.as_ptr().cast(),
                len: player.selection_name.len(),
            },
        };
        unsafe { write_caller_struct(out_info, &value, NUX_PLAYER_INFO_V3_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_free(player: *mut NuxPlayer) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if player.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(player, HandleKind::Player) {
            return status;
        }
        unsafe { drop(Box::from_raw(player)) };
        NuxStatus::Ok
    })
}

fn publish_operation_result(
    out_result: *mut *mut NuxCapiResult,
    status: NuxStatus,
    operation: &str,
) -> NuxStatus {
    if out_result.is_null() {
        return NuxStatus::NullArgument;
    }
    let message = if status == NuxStatus::Ok {
        String::new()
    } else {
        format!("{operation} failed: {}", status_code(status))
    };
    publish_result(out_result, status, message);
    status
}

/// Result-bearing exact artboard selection for consumers that need an owned
/// diagnostic on every outcome.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_artboard_instance_new_named_with_result(
    file: *const NuxFile,
    name: NuxStringView,
    out_instance: *mut *mut NuxArtboardInstance,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(out_instance, out_result, HandleKind::Artboard, || {
        if !out_instance.is_null() {
            unsafe { *out_instance = ptr::null_mut() };
        }
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_result = ptr::null_mut() };
        let status = unsafe { nux_artboard_instance_new_named(file, name, out_instance) };
        publish_operation_result(out_result, status, "named artboard selection")
    })
}

/// Result-bearing default-scene selection.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_default_with_result(
    instance: *mut NuxArtboardInstance,
    out_player: *mut *mut NuxPlayer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(out_player, out_result, HandleKind::Player, || {
        if !out_player.is_null() {
            unsafe { *out_player = ptr::null_mut() };
        }
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_result = ptr::null_mut() };
        let status = unsafe { nux_player_new_default(instance, out_player) };
        publish_operation_result(out_result, status, "default player selection")
    })
}

/// Result-bearing explicit static-artboard selection.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_static_with_result(
    instance: *mut NuxArtboardInstance,
    out_player: *mut *mut NuxPlayer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(out_player, out_result, HandleKind::Player, || {
        if !out_player.is_null() {
            unsafe { *out_player = ptr::null_mut() };
        }
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_result = ptr::null_mut() };
        let status = unsafe { nux_player_new_static(instance, out_player) };
        publish_operation_result(out_result, status, "static player selection")
    })
}

/// Result-bearing exact state-machine selection.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_state_machine_named_with_result(
    instance: *mut NuxArtboardInstance,
    name: NuxStringView,
    out_player: *mut *mut NuxPlayer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(out_player, out_result, HandleKind::Player, || {
        if !out_player.is_null() {
            unsafe { *out_player = ptr::null_mut() };
        }
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_result = ptr::null_mut() };
        let status = unsafe { nux_player_new_state_machine_named(instance, name, out_player) };
        publish_operation_result(out_result, status, "named state-machine selection")
    })
}

/// Result-bearing exact linear-animation selection.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_new_linear_animation_named_with_result(
    instance: *mut NuxArtboardInstance,
    name: NuxStringView,
    out_player: *mut *mut NuxPlayer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(out_player, out_result, HandleKind::Player, || {
        if !out_player.is_null() {
            unsafe { *out_player = ptr::null_mut() };
        }
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_result = ptr::null_mut() };
        let status = unsafe { nux_player_new_linear_animation_named(instance, name, out_player) };
        publish_operation_result(out_result, status, "named linear-animation selection")
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_free(
    state_machine: *mut NuxStateMachineInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if state_machine.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(state_machine, HandleKind::StateMachine) {
            return status;
        }
        unsafe {
            drop(Box::from_raw(state_machine));
        }
        NuxStatus::Ok
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
    let _state_machine_call = match enter_handle(state_machine, HandleKind::StateMachine) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Some(state_machine) = (unsafe { state_machine.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    if let Err(status) = require_owner_thread(state_machine.owner_thread) {
        return status;
    }
    if name.is_null() {
        return NuxStatus::NullArgument;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return NuxStatus::InvalidArgument;
    };
    let Ok(mut machine) = state_machine.instance.try_borrow_mut() else {
        return NuxStatus::ReentrantCall;
    };
    let Some(index) = machine.input_index_named(name) else {
        return NuxStatus::NotFound;
    };
    if apply(&mut machine, index) {
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
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let _state_machine_call = enter_status_handle!(state_machine, HandleKind::StateMachine);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(state_machine) = (unsafe { state_machine.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        if let Err(status) = require_owner_thread(state_machine.owner_thread) {
            return status;
        }
        if let Err(status) = require_same_artboard(&instance.provenance, &state_machine.provenance)
        {
            return status;
        }
        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let Ok(mut machine) = state_machine.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let changed = artboard.advance_with_state_machine(&mut machine, elapsed_seconds);
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
    instance: *mut NuxArtboardInstance,
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
                state_machine.pointer_down(artboard.raw_mut(), x, y, DEFAULT_POINTER_ID)
            },
        )
    })
}

/// Deliver a pointer-move at artboard coordinates `(x, y)` to `state_machine`,
/// which must have been created from `instance`. `out_hit` is optional and
/// reports whether the event landed on a listener.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_pointer_move(
    instance: *mut NuxArtboardInstance,
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
                state_machine.pointer_move(artboard.raw_mut(), x, y, 0.0, DEFAULT_POINTER_ID)
            },
        )
    })
}

/// Deliver a pointer-up at artboard coordinates `(x, y)` to `state_machine`,
/// which must have been created from `instance`. `out_hit` is optional and
/// reports whether the event landed on a listener.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_state_machine_instance_pointer_up(
    instance: *mut NuxArtboardInstance,
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
                state_machine.pointer_up(artboard.raw_mut(), x, y, DEFAULT_POINTER_ID)
            },
        )
    })
}

fn state_machine_pointer_event(
    instance: *mut NuxArtboardInstance,
    state_machine: *mut NuxStateMachineInstance,
    out_hit: *mut bool,
    dispatch: impl FnOnce(&mut StateMachineInstance, &mut OwnedArtboardInstance) -> bool,
) -> NuxStatus {
    if let Some(out_hit) = unsafe { out_hit.as_mut() } {
        *out_hit = false;
    }
    let _instance_call = match enter_handle(instance, HandleKind::Artboard) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let _state_machine_call = match enter_handle(state_machine, HandleKind::StateMachine) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Some(instance) = (unsafe { instance.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    let Some(state_machine) = (unsafe { state_machine.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    if let Err(status) = require_owner_thread(instance.owner_thread) {
        return status;
    }
    if let Err(status) = require_owner_thread(state_machine.owner_thread) {
        return status;
    }
    if let Err(status) = require_same_artboard(&instance.provenance, &state_machine.provenance) {
        return status;
    }
    let _occurrence_call = match enter_occurrence(&instance.occurrence) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
        return NuxStatus::ReentrantCall;
    };
    let Ok(mut machine) = state_machine.instance.try_borrow_mut() else {
        return NuxStatus::ReentrantCall;
    };
    let hit = dispatch(&mut machine, &mut artboard);
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
        view_model_instance_new(instance, out_view_model, |artboard| {
            artboard.instantiate_view_model()
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
        view_model_instance_new(instance, out_view_model, |artboard| {
            artboard.instantiate_view_model_instance(instance_index)
        })
    })
}

fn view_model_instance_new(
    artboard: *const NuxArtboardInstance,
    out_view_model: *mut *mut NuxViewModelInstance,
    build: impl FnOnce(&OwnedArtboardInstance) -> Option<ViewModelInstance>,
) -> NuxStatus {
    if out_view_model.is_null() {
        return NuxStatus::NullArgument;
    }
    unsafe {
        *out_view_model = ptr::null_mut();
    }
    let _artboard_call = match enter_handle(artboard, HandleKind::Artboard) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Some(artboard) = (unsafe { artboard.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    if let Err(status) = require_owner_thread(artboard.owner_thread) {
        return status;
    }
    let _occurrence_call = match enter_occurrence(&artboard.occurrence) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Ok(artboard_instance) = artboard.occurrence.instance.try_borrow() else {
        return NuxStatus::ReentrantCall;
    };
    let Some(view_model) = build(&artboard_instance) else {
        return NuxStatus::NotFound;
    };
    unsafe {
        let handle = Box::into_raw(Box::new(NuxViewModelInstance {
            instance: RefCell::new(view_model),
            owner_thread: artboard.owner_thread,
            provenance: Arc::clone(&artboard.provenance),
        }));
        register_handle(handle, HandleKind::ViewModel, artboard.owner_thread);
        *out_view_model = handle;
    }
    NuxStatus::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_view_model_instance_free(
    view_model: *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if view_model.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(view_model, HandleKind::ViewModel) {
            return status;
        }
        unsafe {
            drop(Box::from_raw(view_model));
        }
        NuxStatus::Ok
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
            let changed = view_model.set_number(name, value);
            changed
                || view_model
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
            let changed = view_model.set_bool(name, value);
            changed
                || view_model
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
            let changed = view_model.set_string(name, value);
            changed
                || view_model
                    .raw()
                    .string_source_handle_by_property_name_path(name)
                    .is_some()
        })
    })
}

fn view_model_set(
    view_model: *mut NuxViewModelInstance,
    name_path: *const c_char,
    apply: impl FnOnce(&mut ViewModelInstance, &str) -> bool,
) -> NuxStatus {
    let _view_model_call = match enter_handle(view_model, HandleKind::ViewModel) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Some(view_model) = (unsafe { view_model.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    if let Err(status) = require_owner_thread(view_model.owner_thread) {
        return status;
    }
    if name_path.is_null() {
        return NuxStatus::NullArgument;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name_path) }).to_str() else {
        return NuxStatus::InvalidArgument;
    };
    let Ok(mut view_model_instance) = view_model.instance.try_borrow_mut() else {
        return NuxStatus::ReentrantCall;
    };
    if apply(&mut view_model_instance, name) {
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
        let _instance_call = enter_status_handle!(instance, HandleKind::Artboard);
        let _view_model_call = enter_status_handle!(view_model, HandleKind::ViewModel);
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(view_model) = (unsafe { view_model.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if let Err(status) = require_owner_thread(instance.owner_thread) {
            return status;
        }
        if let Err(status) = require_owner_thread(view_model.owner_thread) {
            return status;
        }
        if let Err(status) = require_same_artboard(&instance.provenance, &view_model.provenance) {
            return status;
        }
        let _occurrence_call = match enter_occurrence(&instance.occurrence) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Ok(mut artboard) = instance.occurrence.instance.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        let Ok(view_model_instance) = view_model.instance.try_borrow() else {
            return NuxStatus::ReentrantCall;
        };
        artboard.bind_view_model(&view_model_instance);
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

    #[test]
    fn panic_poisoned_handle_rejects_later_calls_but_can_be_freed() {
        let handle = Box::into_raw(Box::new(7_u8));
        register_handle(handle, HandleKind::File, thread::current().id());
        let caught = panic::catch_unwind(AssertUnwindSafe(|| {
            let _call = match enter_handle(handle, HandleKind::File) {
                Ok(call) => call,
                Err(status) => panic!("unexpected enter failure: {status:?}"),
            };
            panic!("poison this handle");
        }));
        assert!(caught.is_err());
        assert!(matches!(
            enter_handle(handle, HandleKind::File),
            Err(NuxStatus::RuntimeError)
        ));
        assert_eq!(remove_handle(handle, HandleKind::File), Ok(()));
        unsafe { drop(Box::from_raw(handle)) };
    }

    #[test]
    fn typed_result_firewall_reclaims_partially_published_handle() {
        let mut handle: *mut u8 = ptr::null_mut();
        let mut result: *mut NuxCapiResult = ptr::null_mut();
        let status = ffi_guard_with_handle_result(
            &mut handle,
            &mut result,
            HandleKind::File,
            || -> NuxStatus {
                handle = Box::into_raw(Box::new(9_u8));
                register_handle(handle, HandleKind::File, thread::current().id());
                publish_result(&mut result, NuxStatus::Ok, "");
                panic!("panic after both outputs were published");
            },
        );
        assert_eq!(status, NuxStatus::RuntimeError);
        assert!(handle.is_null());
        assert!(!result.is_null());
        let mut result_status = NuxStatus::Ok;
        assert_eq!(
            unsafe { nux_capi_result_status(result, &mut result_status) },
            NuxStatus::Ok
        );
        assert_eq!(result_status, NuxStatus::RuntimeError);
        assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
    }

    #[test]
    fn default_scene_uses_state_machine_zero_after_invalid_authored_default() {
        let mut context = ();
        let selection = select_default_scene(
            &mut context,
            Some(7),
            |(), index| (index == 0).then_some("state-machine-zero"),
            |(), index| (index == 0).then_some("animation-zero"),
        );
        assert!(matches!(
            selection,
            DefaultSceneSelection::StateMachine {
                index: 0,
                instance: "state-machine-zero"
            }
        ));
    }
}
