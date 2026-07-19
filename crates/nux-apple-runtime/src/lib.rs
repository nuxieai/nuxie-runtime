//! Product C ABI for the Nuxie Apple flow runtime.

#[cfg(all(feature = "apple-product", panic = "abort"))]
compile_error!(
    "nux-apple-runtime's apple-product feature requires panic=unwind; use the release-apple profile"
);

use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use std::slice;

#[cfg(feature = "apple-product")]
use nuxie::{
    ApplePresentationCompletion, AppleSurface, ArtboardRenderCache, File, Mat2D,
    OwnedArtboardInstance, RenderMode, Renderer, StateMachineInstance, SurfaceDisposition,
    WgpuFactory,
};
#[cfg(feature = "apple-product")]
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle, ThreadId},
};

pub const NUX_RUNTIME_ABI_MAJOR: u16 = 1;
pub const NUX_RUNTIME_ABI_MINOR: u16 = 0;

const MAX_ARTIFACT_BYTE_LENGTH: usize = 67_108_864;
const MAX_SELECTOR_BYTE_LENGTH: usize = 4_096;
const PANIC_DIAGNOSTIC: &str = "runtime panicked; the affected flow session is terminated";
const BUILD_PROVENANCE: &str = env!("NUX_RUNTIME_BUILD_PROVENANCE");

fn ffi_guard<R>(fallback: R, body: impl FnOnce() -> R) -> R {
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(_) => fallback,
    }
}

fn ffi_guard_with_result(
    out_result: *mut *mut NuxOperationResult,
    on_panic: impl FnOnce(),
    body: impl FnOnce() -> NuxStatus,
) -> NuxStatus {
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(status) => status,
        Err(_) => {
            let _ = panic::catch_unwind(AssertUnwindSafe(on_panic));
            replace_result(
                out_result,
                NuxOperationResult::failure(NuxStatus::RuntimeError, PANIC_DIAGNOSTIC),
            );
            NuxStatus::RuntimeError
        }
    }
}

/// Stable-width C status code. Named constants are exported separately so
/// Swift imports one unambiguous `UInt32`-backed type instead of a C enum tag
/// that competes with its typedef.
pub type NuxStatus = u32;

pub const NUX_STATUS_OK: NuxStatus = 0 as NuxStatus;
pub const NUX_STATUS_NULL_ARGUMENT: NuxStatus = 1 as NuxStatus;
pub const NUX_STATUS_IMPORT_ERROR: NuxStatus = 2 as NuxStatus;
pub const NUX_STATUS_NOT_FOUND: NuxStatus = 3 as NuxStatus;
pub const NUX_STATUS_RUNTIME_ERROR: NuxStatus = 4 as NuxStatus;
pub const NUX_STATUS_INVALID_ARGUMENT: NuxStatus = 5 as NuxStatus;
pub const NUX_STATUS_ABI_MISMATCH: NuxStatus = 6 as NuxStatus;
pub const NUX_STATUS_SURFACE_ERROR: NuxStatus = 7 as NuxStatus;

/// Stable-width C presentation outcome.
pub type NuxSurfaceDisposition = u32;

pub const NUX_SURFACE_DISPOSITION_NONE: NuxSurfaceDisposition = 0 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_PRESENTED: NuxSurfaceDisposition = 1 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_SKIPPED_ZERO_SIZE: NuxSurfaceDisposition =
    2 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_SKIPPED_TIMEOUT: NuxSurfaceDisposition =
    3 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_SKIPPED_OCCLUDED: NuxSurfaceDisposition =
    4 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_RECONFIGURED: NuxSurfaceDisposition = 5 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_RECREATED: NuxSurfaceDisposition = 6 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_DEVICE_LOST: NuxSurfaceDisposition = 7 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_OUT_OF_MEMORY: NuxSurfaceDisposition = 8 as NuxSurfaceDisposition;
pub const NUX_SURFACE_DISPOSITION_FATAL: NuxSurfaceDisposition = 9 as NuxSurfaceDisposition;

// Keep the internal implementation readable while the public C surface uses
// fixed-width aliases and exported constants.
#[allow(dead_code, non_upper_case_globals)]
trait NuxStatusConstants {
    const Ok: NuxStatus = NUX_STATUS_OK;
    const NullArgument: NuxStatus = NUX_STATUS_NULL_ARGUMENT;
    const ImportError: NuxStatus = NUX_STATUS_IMPORT_ERROR;
    const NotFound: NuxStatus = NUX_STATUS_NOT_FOUND;
    const RuntimeError: NuxStatus = NUX_STATUS_RUNTIME_ERROR;
    const InvalidArgument: NuxStatus = NUX_STATUS_INVALID_ARGUMENT;
    const AbiMismatch: NuxStatus = NUX_STATUS_ABI_MISMATCH;
    const SurfaceError: NuxStatus = NUX_STATUS_SURFACE_ERROR;
}

impl NuxStatusConstants for u32 {}

#[allow(dead_code, non_upper_case_globals)]
trait NuxSurfaceDispositionConstants {
    const None: NuxSurfaceDisposition = NUX_SURFACE_DISPOSITION_NONE;
    const Presented: NuxSurfaceDisposition = NUX_SURFACE_DISPOSITION_PRESENTED;
    const SkippedTimeout: NuxSurfaceDisposition = NUX_SURFACE_DISPOSITION_SKIPPED_TIMEOUT;
    const Recreated: NuxSurfaceDisposition = NUX_SURFACE_DISPOSITION_RECREATED;
    const Fatal: NuxSurfaceDisposition = NUX_SURFACE_DISPOSITION_FATAL;
}

impl NuxSurfaceDispositionConstants for u32 {}

#[cfg(feature = "apple-product")]
fn surface_disposition(value: SurfaceDisposition) -> NuxSurfaceDisposition {
    match value {
        SurfaceDisposition::None => NUX_SURFACE_DISPOSITION_NONE,
        SurfaceDisposition::Presented => NUX_SURFACE_DISPOSITION_PRESENTED,
        SurfaceDisposition::SkippedZeroSize => NUX_SURFACE_DISPOSITION_SKIPPED_ZERO_SIZE,
        SurfaceDisposition::SkippedTimeout => NUX_SURFACE_DISPOSITION_SKIPPED_TIMEOUT,
        SurfaceDisposition::SkippedOccluded => NUX_SURFACE_DISPOSITION_SKIPPED_OCCLUDED,
        SurfaceDisposition::Reconfigured => NUX_SURFACE_DISPOSITION_RECONFIGURED,
        SurfaceDisposition::Recreated => NUX_SURFACE_DISPOSITION_RECREATED,
        SurfaceDisposition::DeviceLost => NUX_SURFACE_DISPOSITION_DEVICE_LOST,
        SurfaceDisposition::OutOfMemory => NUX_SURFACE_DISPOSITION_OUT_OF_MEMORY,
        SurfaceDisposition::Fatal => NUX_SURFACE_DISPOSITION_FATAL,
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxByteView {
    pub data: *const u8,
    pub len: u64,
}

impl Default for NuxByteView {
    fn default() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
        }
    }
}

impl NuxByteView {
    fn from_static(value: &'static str) -> Self {
        Self {
            data: value.as_ptr(),
            len: u64::try_from(value.len()).unwrap_or(u64::MAX),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowImportRequest {
    pub struct_size: u32,
    /// Exact verified visual-runtime bytes. The field is container-neutral so
    /// the current RIV adapter can later be replaced without changing sessions.
    pub artifact_bytes: NuxByteView,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowSessionDescriptor {
    pub struct_size: u32,
    /// UTF-8 authored artboard name. A null view selects the default artboard.
    pub artboard_name: NuxByteView,
    /// UTF-8 authored state-machine name. A null view selects the authored
    /// default, falling back to state-machine zero. Slice 1 advances the base
    /// artboard update when no state machine exists; linear-animation fallback
    /// is added with the product-complete player selection operation.
    pub state_machine_name: NuxByteView,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxAppleSurfaceDescriptor {
    pub struct_size: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
}

/// Called exactly once when Metal has finished using a submitted drawable, or
/// before `nux_flow_render_session_advance` returns when it cannot be submitted.
pub type NuxFrameCompletionCallback = unsafe extern "C" fn(context: *mut c_void);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFrameOperation {
    pub struct_size: u32,
    pub elapsed_seconds: f32,
    pub render: bool,
    /// A live `id<CAMetalDrawable>` acquired by Swift on the main actor.
    /// The runtime borrows it only for the synchronous advance call.
    pub apple_drawable: *mut c_void,
    /// Caller-owned context consumed by `completion_callback`. Both completion
    /// fields must be null or non-null together.
    pub completion_context: *mut c_void,
    /// Optional one-shot GPU completion callback. The callback may run on an
    /// arbitrary Metal or runtime thread and must not call UIKit.
    pub completion_callback: Option<unsafe extern "C" fn(context: *mut c_void)>,
}

#[cfg(feature = "apple-product")]
struct PendingFrameCompletion {
    callback: Option<NuxFrameCompletionCallback>,
    context_identity: usize,
}

#[cfg(feature = "apple-product")]
impl PendingFrameCompletion {
    fn from_operation(operation: &NuxFrameOperation) -> Result<Self, &'static str> {
        if operation.completion_callback.is_some() == operation.completion_context.is_null() {
            return Err("frame completion callback and context must be supplied together");
        }
        Ok(Self {
            callback: operation.completion_callback,
            context_identity: operation.completion_context.expose_provenance(),
        })
    }

    fn into_renderer_completion(mut self) -> Option<ApplePresentationCompletion> {
        let callback = self.callback.take()?;
        let context_identity = self.context_identity;
        Some(ApplePresentationCompletion::new(move || unsafe {
            callback(ptr::with_exposed_provenance_mut(context_identity));
        }))
    }
}

#[cfg(feature = "apple-product")]
impl Drop for PendingFrameCompletion {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            unsafe {
                callback(ptr::with_exposed_provenance_mut(self.context_identity));
            }
        }
    }
}

/// Opaque C handle. Its storage is private and retained by child handles.
pub struct NuxFlowRuntimeContext {
    _private: [u8; 0],
}

/// Opaque C handle. It retains its runtime context.
pub struct NuxFlowRenderSession {
    _private: [u8; 0],
}

/// Opaque C handle. It retains the logical render session across detach.
pub struct NuxAppleSurface {
    _private: [u8; 0],
}

#[cfg(feature = "apple-product")]
type SessionId = u64;

#[cfg(feature = "apple-product")]
type SurfaceId = u64;

#[cfg(feature = "apple-product")]
struct RuntimeWorker {
    sender: Sender<WorkerMessage>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
    thread_id: ThreadId,
}

#[cfg(feature = "apple-product")]
struct FlowRuntimeContextHandle {
    worker: Arc<RuntimeWorker>,
}

#[cfg(feature = "apple-product")]
struct SessionToken {
    worker: Arc<RuntimeWorker>,
    id: SessionId,
}

#[cfg(feature = "apple-product")]
struct FlowRenderSessionHandle {
    token: Arc<SessionToken>,
}

#[cfg(feature = "apple-product")]
struct SurfaceToken {
    session: Arc<SessionToken>,
    id: SurfaceId,
}

#[cfg(feature = "apple-product")]
struct AppleSurfaceHandle {
    token: Arc<SurfaceToken>,
}

#[cfg(feature = "apple-product")]
struct WorkerState {
    owner_thread_id: ThreadId,
    file: Arc<File>,
    shared_gpu_factory: Option<WgpuFactory>,
    sessions: HashMap<SessionId, SessionState>,
    next_session_id: SessionId,
    next_surface_id: SurfaceId,
}

#[cfg(feature = "apple-product")]
struct SessionState {
    is_fatal: bool,
    instance: OwnedArtboardInstance,
    state_machine: Option<StateMachineInstance>,
    render_cache: ArtboardRenderCache,
    attachment: Option<SurfaceState>,
}

#[cfg(feature = "apple-product")]
struct SurfaceState {
    id: SurfaceId,
    factory: WgpuFactory,
    surface: AppleSurface,
}

#[cfg(feature = "apple-product")]
struct WorkerJob {
    session_id: Option<SessionId>,
    execute: Box<dyn FnOnce(&mut WorkerState) + Send + 'static>,
    on_panic: Box<dyn FnOnce() + Send + 'static>,
}

#[cfg(feature = "apple-product")]
enum WorkerMessage {
    Run(WorkerJob),
    Shutdown,
}

#[cfg(feature = "apple-product")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerCallError {
    Panicked,
    Unavailable,
}

#[cfg(feature = "apple-product")]
enum WorkerStartError {
    Import(String),
    Runtime(String),
    Panicked,
}

#[cfg(feature = "apple-product")]
#[derive(Debug)]
struct RuntimeFailure {
    status: NuxStatus,
    diagnostic: String,
}

#[cfg(feature = "apple-product")]
impl RuntimeFailure {
    fn new(status: NuxStatus, diagnostic: impl Into<String>) -> Self {
        Self {
            status,
            diagnostic: diagnostic.into(),
        }
    }

    fn runtime(diagnostic: impl Into<String>) -> Self {
        Self::new(NuxStatus::RuntimeError, diagnostic)
    }

    fn surface(diagnostic: impl Into<String>) -> Self {
        Self::new(NuxStatus::SurfaceError, diagnostic)
    }
}

#[cfg(feature = "apple-product")]
impl WorkerState {
    fn new(file: File) -> Self {
        Self {
            owner_thread_id: thread::current().id(),
            file: Arc::new(file),
            shared_gpu_factory: None,
            sessions: HashMap::new(),
            next_session_id: 1,
            next_surface_id: 1,
        }
    }

    fn allocate_session_id(&mut self) -> Result<SessionId, RuntimeFailure> {
        let id = self.next_session_id;
        self.next_session_id = self
            .next_session_id
            .checked_add(1)
            .ok_or_else(|| RuntimeFailure::runtime("session identifier space is exhausted"))?;
        Ok(id)
    }

    fn allocate_surface_id(&mut self) -> Result<SurfaceId, RuntimeFailure> {
        let id = self.next_surface_id;
        self.next_surface_id = self
            .next_surface_id
            .checked_add(1)
            .ok_or_else(|| RuntimeFailure::runtime("surface identifier space is exhausted"))?;
        Ok(id)
    }

    fn create_session(
        &mut self,
        artboard_name: Option<String>,
        state_machine_name: Option<String>,
    ) -> Result<SessionId, RuntimeFailure> {
        let artboard_index = match artboard_name {
            Some(name) => self
                .file
                .artboard_named(&name)
                .map(|artboard| artboard.index())
                .ok_or_else(|| {
                    RuntimeFailure::new(
                        NuxStatus::NotFound,
                        format!("artboard `{name}` was not found"),
                    )
                })?,
            None => self
                .file
                .default_artboard()
                .map(|artboard| artboard.index())
                .ok_or_else(|| {
                    RuntimeFailure::new(NuxStatus::NotFound, "artifact has no default artboard")
                })?,
        };
        let instance =
            OwnedArtboardInstance::instantiate(Arc::clone(&self.file), artboard_index)
                .map_err(|error| RuntimeFailure::new(NuxStatus::NotFound, error.to_string()))?;
        let state_machine = match state_machine_name {
            Some(name) => {
                let artboard = instance.artboard();
                let index = (0..artboard.state_machine_count())
                    .find(|index| artboard.state_machine_name(*index) == Some(name.as_str()));
                index
                    .and_then(|index| instance.state_machine_instance(index))
                    .map(Some)
                    .ok_or_else(|| {
                        RuntimeFailure::new(
                            NuxStatus::NotFound,
                            format!("state machine `{name}` was not found"),
                        )
                    })?
            }
            None => instance.default_state_machine_instance(),
        };
        let render_cache = instance.new_render_cache();
        let id = self.allocate_session_id()?;
        self.sessions.insert(
            id,
            SessionState {
                is_fatal: false,
                instance,
                state_machine,
                render_cache,
                attachment: None,
            },
        );
        Ok(id)
    }

    fn session(&self, id: SessionId) -> Result<&SessionState, RuntimeFailure> {
        self.sessions
            .get(&id)
            .ok_or_else(|| RuntimeFailure::runtime("render session is unavailable"))
    }

    fn session_mut(&mut self, id: SessionId) -> Result<&mut SessionState, RuntimeFailure> {
        self.sessions
            .get_mut(&id)
            .ok_or_else(|| RuntimeFailure::runtime("render session is unavailable"))
    }

    fn require_live_session(&self, id: SessionId) -> Result<(), RuntimeFailure> {
        if self.session(id)?.is_fatal {
            Err(RuntimeFailure::runtime(PANIC_DIAGNOSTIC))
        } else {
            Ok(())
        }
    }

    fn make_session_surface(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(WgpuFactory, AppleSurface), RuntimeFailure> {
        let dimensions = (width.max(1), height.max(1));
        if let Some(shared) = self.shared_gpu_factory.as_ref() {
            let mut factory = shared
                .new_session_factory(dimensions.0, dimensions.1, RenderMode::Msaa)
                .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
            let surface = AppleSurface::attach(&mut factory, width, height)
                .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
            return Ok((factory, surface));
        }

        let (factory, surface) = AppleSurface::attach_with_factory(width, height, RenderMode::Msaa)
            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
        self.shared_gpu_factory = Some(
            factory
                .new_session_factory(1, 1, RenderMode::Msaa)
                .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?,
        );
        Ok((factory, surface))
    }

    fn attach_surface(
        &mut self,
        session_id: SessionId,
        width: u32,
        height: u32,
    ) -> Result<SurfaceId, RuntimeFailure> {
        self.require_live_session(session_id)?;
        if self.session(session_id)?.attachment.is_some() {
            return Err(RuntimeFailure::new(
                NuxStatus::InvalidArgument,
                "session already has an attached surface",
            ));
        }
        let (factory, surface) = self.make_session_surface(width, height)?;
        let id = self.allocate_surface_id()?;
        self.session_mut(session_id)?.attachment = Some(SurfaceState {
            id,
            factory,
            surface,
        });
        Ok(id)
    }

    fn surface_mut(
        &mut self,
        session_id: SessionId,
        surface_id: SurfaceId,
    ) -> Result<&mut SurfaceState, RuntimeFailure> {
        self.require_live_session(session_id)?;
        self.session_mut(session_id)?
            .attachment
            .as_mut()
            .filter(|attachment| attachment.id == surface_id)
            .ok_or_else(|| RuntimeFailure::surface("surface is detached"))
    }

    fn remove_surface(&mut self, session_id: SessionId, surface_id: SurfaceId) {
        let Some(session) = self.sessions.get_mut(&session_id) else {
            return;
        };
        let is_current = session
            .attachment
            .as_ref()
            .is_some_and(|attachment| attachment.id == surface_id);
        if !is_current {
            return;
        }
        if let Some(mut attachment) = session.attachment.take() {
            attachment.surface.detach();
        }
    }

    fn remove_session(&mut self, session_id: SessionId) {
        if let Some(mut session) = self.sessions.remove(&session_id)
            && let Some(mut attachment) = session.attachment.take()
        {
            attachment.surface.detach();
        }
    }

    fn poison_session(&mut self, session_id: SessionId) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.is_fatal = true;
        }
    }
}

#[cfg(feature = "apple-product")]
fn centered_contain_transform(
    artboard_width: f32,
    artboard_height: f32,
    viewport_width: u32,
    viewport_height: u32,
) -> Result<Mat2D, RuntimeFailure> {
    if !artboard_width.is_finite()
        || !artboard_height.is_finite()
        || artboard_width <= 0.0
        || artboard_height <= 0.0
        || viewport_width == 0
        || viewport_height == 0
    {
        return Err(RuntimeFailure::runtime(
            "artboard and viewport dimensions must be finite and positive",
        ));
    }
    let viewport_width = viewport_width as f32;
    let viewport_height = viewport_height as f32;
    let scale = (viewport_width / artboard_width).min(viewport_height / artboard_height);
    let offset_x = (viewport_width - artboard_width * scale) * 0.5;
    let offset_y = (viewport_height - artboard_height * scale) * 0.5;
    if !scale.is_finite() || !offset_x.is_finite() || !offset_y.is_finite() || scale <= 0.0 {
        return Err(RuntimeFailure::runtime(
            "centered contain transform is not finite",
        ));
    }
    Ok(Mat2D([scale, 0.0, 0.0, scale, offset_x, offset_y]))
}

#[cfg(feature = "apple-product")]
impl Drop for WorkerState {
    fn drop(&mut self) {
        debug_assert_eq!(thread::current().id(), self.owner_thread_id);
    }
}

#[cfg(feature = "apple-product")]
impl RuntimeWorker {
    fn spawn(artifact_bytes: Vec<u8>) -> Result<Arc<Self>, WorkerStartError> {
        let (sender, receiver) = mpsc::channel();
        let (initialization_sender, initialization_receiver) = mpsc::sync_channel(1);
        let join_handle = thread::Builder::new()
            .name("nuxie-flow-runtime".to_owned())
            .spawn(move || {
                let state = panic::catch_unwind(AssertUnwindSafe(|| {
                    File::import(&artifact_bytes)
                        .map(WorkerState::new)
                        .map_err(|error| error.to_string())
                }));
                let state = match state {
                    Ok(Ok(state)) => state,
                    Ok(Err(diagnostic)) => {
                        let _ =
                            initialization_sender.send(Err(WorkerStartError::Import(diagnostic)));
                        return;
                    }
                    Err(_) => {
                        let _ = initialization_sender.send(Err(WorkerStartError::Panicked));
                        return;
                    }
                };
                let _ = initialization_sender.send(Ok(thread::current().id()));
                worker_loop(state, receiver);
            })
            .map_err(|error| WorkerStartError::Runtime(error.to_string()))?;

        let initialization = initialization_receiver.recv().map_err(|_| {
            WorkerStartError::Runtime("runtime worker stopped during initialization".to_owned())
        });
        let thread_id = match initialization {
            Ok(Ok(thread_id)) => thread_id,
            Ok(Err(error)) => {
                let _ = join_handle.join();
                return Err(error);
            }
            Err(error) => {
                let _ = join_handle.join();
                return Err(error);
            }
        };
        Ok(Arc::new(Self {
            sender,
            join_handle: Mutex::new(Some(join_handle)),
            thread_id,
        }))
    }

    fn call<R: Send + 'static>(
        &self,
        session_id: Option<SessionId>,
        body: impl FnOnce(&mut WorkerState) -> R + Send + 'static,
    ) -> Result<R, WorkerCallError> {
        if thread::current().id() == self.thread_id {
            return Err(WorkerCallError::Unavailable);
        }
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let panic_sender = response_sender.clone();
        let job = WorkerJob {
            session_id,
            execute: Box::new(move |state| {
                let response = body(state);
                let _ = response_sender.send(Ok(response));
            }),
            on_panic: Box::new(move || {
                let _ = panic_sender.send(Err(WorkerCallError::Panicked));
            }),
        };
        self.sender
            .send(WorkerMessage::Run(job))
            .map_err(|_| WorkerCallError::Unavailable)?;
        response_receiver
            .recv()
            .map_err(|_| WorkerCallError::Unavailable)?
    }

    fn poison_session(&self, session_id: SessionId) {
        let _ = self.call(None, move |state| state.poison_session(session_id));
    }

    #[cfg(test)]
    fn probe_thread_id(&self) -> Result<ThreadId, WorkerCallError> {
        self.call(None, |_| thread::current().id())
    }
}

#[cfg(feature = "apple-product")]
impl Drop for RuntimeWorker {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        let join_handle = match self.join_handle.get_mut() {
            Ok(join_handle) => join_handle.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some(join_handle) = join_handle
            && thread::current().id() != self.thread_id
        {
            let _ = join_handle.join();
        }
    }
}

#[cfg(feature = "apple-product")]
impl Drop for SessionToken {
    fn drop(&mut self) {
        let session_id = self.id;
        // Swift enqueues C destruction away from the main actor. Complete the
        // worker-side teardown before returning so GPU resources are gone
        // before the final Swift owner is released.
        let _ = self
            .worker
            .call(None, move |state| state.remove_session(session_id));
    }
}

#[cfg(feature = "apple-product")]
impl Drop for SurfaceToken {
    fn drop(&mut self) {
        let session_id = self.session.id;
        let surface_id = self.id;
        let _ = self.session.worker.call(Some(session_id), move |state| {
            state.remove_surface(session_id, surface_id);
        });
    }
}

#[cfg(feature = "apple-product")]
fn worker_loop(mut state: WorkerState, receiver: Receiver<WorkerMessage>) {
    while let Ok(message) = receiver.recv() {
        let WorkerMessage::Run(job) = message else {
            break;
        };
        debug_assert_eq!(thread::current().id(), state.owner_thread_id);
        let WorkerJob {
            session_id,
            execute,
            on_panic,
        } = job;
        if panic::catch_unwind(AssertUnwindSafe(|| execute(&mut state))).is_err() {
            if let Some(session_id) = session_id {
                state.poison_session(session_id);
            }
            let _ = panic::catch_unwind(AssertUnwindSafe(on_panic));
        }
    }
}

#[cfg(feature = "apple-product")]
#[allow(dead_code)]
fn assert_opaque_handle_storage_is_send_and_sync() {
    fn assert_send_and_sync<T: Send + Sync>() {}
    assert_send_and_sync::<FlowRuntimeContextHandle>();
    assert_send_and_sync::<FlowRenderSessionHandle>();
    assert_send_and_sync::<AppleSurfaceHandle>();
}

pub struct NuxOperationResult {
    status: NuxStatus,
    surface_disposition: NuxSurfaceDisposition,
    changed: bool,
    diagnostic: Vec<u8>,
}

impl NuxOperationResult {
    fn success(surface_disposition: NuxSurfaceDisposition, changed: bool) -> Self {
        Self {
            status: NuxStatus::Ok,
            surface_disposition,
            changed,
            diagnostic: Vec::new(),
        }
    }

    fn failure(status: NuxStatus, diagnostic: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            surface_disposition: NuxSurfaceDisposition::Fatal,
            changed: false,
            diagnostic: diagnostic.into(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nux_runtime_abi_major() -> u16 {
    ffi_guard(0, || NUX_RUNTIME_ABI_MAJOR)
}

#[unsafe(no_mangle)]
pub extern "C" fn nux_runtime_abi_minor() -> u16 {
    ffi_guard(0, || NUX_RUNTIME_ABI_MINOR)
}

#[unsafe(no_mangle)]
pub extern "C" fn nux_runtime_require_abi(required_major: u16, minimum_minor: u16) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if required_major == NUX_RUNTIME_ABI_MAJOR
            && (0..=NUX_RUNTIME_ABI_MINOR).contains(&minimum_minor)
        {
            NuxStatus::Ok
        } else {
            NuxStatus::AbiMismatch
        }
    })
}

#[unsafe(no_mangle)]
/// Writes a process-static UTF-8 JSON view to `out_provenance`.
///
/// # Safety
///
/// `out_provenance` must point to writable, properly aligned storage for one
/// [`NuxByteView`].
pub unsafe extern "C" fn nux_runtime_build_provenance(
    out_provenance: *mut NuxByteView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_provenance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_provenance = NuxByteView::from_static(BUILD_PROVENANCE);
        }
        NuxStatus::Ok
    })
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Imports one visual artifact into a retained runtime context.
///
/// # Safety
///
/// Non-null pointers must be properly aligned and valid for this call.
/// `request.artifact_bytes` must be readable for its declared length. Output
/// pointers must address writable handle storage.
pub unsafe extern "C" fn nux_flow_runtime_context_create(
    request: *const NuxFlowImportRequest,
    out_context: *mut *mut NuxFlowRuntimeContext,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || {},
        || {
            reset_out_handle(out_context);
            reset_out_handle(out_result);
            if request.is_null() || out_context.is_null() {
                return NuxStatus::NullArgument;
            }
            let struct_size = unsafe { read_struct_size(request) };
            if struct_size < size_u32::<NuxFlowImportRequest>() {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "flow import request is smaller than this ABI requires",
                );
            }
            let request = unsafe { *request };
            let bytes = match byte_vec(request.artifact_bytes, MAX_ARTIFACT_BYTE_LENGTH) {
                Ok(bytes) => bytes,
                Err(status) => {
                    return write_failure(out_result, status, "artifact byte view is invalid");
                }
            };
            match RuntimeWorker::spawn(bytes) {
                Ok(worker) => {
                    let context = Box::new(FlowRuntimeContextHandle { worker });
                    unsafe {
                        *out_context = Box::into_raw(context).cast();
                    }
                    write_success(out_result, NuxSurfaceDisposition::None, false)
                }
                Err(WorkerStartError::Import(diagnostic)) => {
                    write_failure(out_result, NuxStatus::ImportError, diagnostic)
                }
                Err(WorkerStartError::Runtime(diagnostic)) => {
                    write_failure(out_result, NuxStatus::RuntimeError, diagnostic)
                }
                Err(WorkerStartError::Panicked) => {
                    write_failure(out_result, NuxStatus::RuntimeError, PANIC_DIAGNOSTIC)
                }
            }
        },
    )
}

#[unsafe(no_mangle)]
/// Releases one runtime-context handle. Null is a no-op.
///
/// # Safety
///
/// A non-null pointer must be an owned handle returned by this library and
/// must not have been released before. Release must not race a call using the
/// same handle. Child handles may remain alive.
pub unsafe extern "C" fn nux_flow_runtime_context_free(context: *mut NuxFlowRuntimeContext) {
    ffi_guard((), || {
        if !context.is_null() {
            unsafe {
                #[cfg(feature = "apple-product")]
                drop(Box::from_raw(context.cast::<FlowRuntimeContextHandle>()));
            }
        }
    })
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Creates an independent logical screen session from a context.
///
/// # Safety
///
/// `context` must be live. The descriptor and output pointers must be valid,
/// aligned, and readable or writable as their direction requires. Calls may
/// originate on arbitrary threads; this library serializes runtime state.
pub unsafe extern "C" fn nux_flow_render_session_create(
    context: *const NuxFlowRuntimeContext,
    descriptor: *const NuxFlowSessionDescriptor,
    out_session: *mut *mut NuxFlowRenderSession,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || {},
        || {
            reset_out_handle(out_session);
            reset_out_handle(out_result);
            if context.is_null() || descriptor.is_null() || out_session.is_null() {
                return NuxStatus::NullArgument;
            }
            let context = unsafe { &*context.cast::<FlowRuntimeContextHandle>() };
            let struct_size = unsafe { read_struct_size(descriptor) };
            if struct_size < size_u32::<NuxFlowSessionDescriptor>() {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "flow session descriptor is smaller than this ABI requires",
                );
            }
            let descriptor = unsafe { *descriptor };
            let artboard_name = match optional_utf8_string(descriptor.artboard_name) {
                Ok(name) => name,
                Err(status) => {
                    return write_failure(out_result, status, "artboard name is not valid UTF-8");
                }
            };
            let state_machine_name = match optional_utf8_string(descriptor.state_machine_name) {
                Ok(name) => name,
                Err(status) => {
                    return write_failure(
                        out_result,
                        status,
                        "state-machine name is not valid UTF-8",
                    );
                }
            };
            let session_id = match context.worker.call(None, move |state| {
                state.create_session(artboard_name, state_machine_name)
            }) {
                Ok(Ok(session_id)) => session_id,
                Ok(Err(failure)) => return write_runtime_failure(out_result, failure),
                Err(error) => return write_worker_call_failure(out_result, error),
            };
            let session = Box::new(FlowRenderSessionHandle {
                token: Arc::new(SessionToken {
                    worker: Arc::clone(&context.worker),
                    id: session_id,
                }),
            });
            unsafe {
                *out_session = Box::into_raw(session).cast();
            }
            write_success(out_result, NuxSurfaceDisposition::None, false)
        },
    )
}

#[unsafe(no_mangle)]
/// Releases one render-session handle. Null is a no-op.
///
/// # Safety
///
/// A non-null pointer must be an owned handle returned by this library and not
/// previously released. Release must not race a call using the same handle.
/// Child surfaces may remain alive.
pub unsafe extern "C" fn nux_flow_render_session_free(session: *mut NuxFlowRenderSession) {
    ffi_guard((), || {
        if !session.is_null() {
            unsafe {
                #[cfg(feature = "apple-product")]
                drop(Box::from_raw(session.cast::<FlowRenderSessionHandle>()));
            }
        }
    })
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Creates logical Apple presentation state for a render session.
///
/// # Safety
///
/// Handles and output pointers must be valid. Calls may originate on arbitrary
/// threads; this library serializes runtime state. Swift remains responsible
/// for configuring its `CAMetalLayer` and acquiring each drawable.
pub unsafe extern "C" fn nux_flow_render_session_attach_apple_surface(
    session: *const NuxFlowRenderSession,
    descriptor: *const NuxAppleSurfaceDescriptor,
    out_surface: *mut *mut NuxAppleSurface,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || unsafe { poison_session_handle(session) },
        || {
            reset_out_handle(out_surface);
            reset_out_handle(out_result);
            if session.is_null() || descriptor.is_null() || out_surface.is_null() {
                return NuxStatus::NullArgument;
            }
            let session = unsafe { &*session.cast::<FlowRenderSessionHandle>() };
            let struct_size = unsafe { read_struct_size(descriptor) };
            if struct_size < size_u32::<NuxAppleSurfaceDescriptor>() {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "Apple surface descriptor is smaller than this ABI requires",
                );
            }
            let descriptor = unsafe { *descriptor };
            let session_id = session.token.id;
            let pixel_width = descriptor.pixel_width;
            let pixel_height = descriptor.pixel_height;
            let surface_id = match session.token.worker.call(Some(session_id), move |state| {
                state.attach_surface(session_id, pixel_width, pixel_height)
            }) {
                Ok(Ok(surface_id)) => surface_id,
                Ok(Err(failure)) => return write_runtime_failure(out_result, failure),
                Err(error) => return write_worker_call_failure(out_result, error),
            };
            let surface = Box::new(AppleSurfaceHandle {
                token: Arc::new(SurfaceToken {
                    session: Arc::clone(&session.token),
                    id: surface_id,
                }),
            });
            unsafe {
                *out_surface = Box::into_raw(surface).cast();
            }
            let disposition = if pixel_width == 0 || pixel_height == 0 {
                NUX_SURFACE_DISPOSITION_SKIPPED_ZERO_SIZE
            } else {
                NUX_SURFACE_DISPOSITION_RECREATED
            };
            write_success(out_result, disposition, false)
        },
    )
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Copies the renderer's Metal device for main-actor `CAMetalLayer` setup.
///
/// On success `out_metal_device` receives an Objective-C object pointer with
/// +1 ownership. The caller must transfer that ownership to ARC or release it.
///
/// # Safety
///
/// `surface` must be live and output pointers must be null or writable. Calls
/// may originate on arbitrary threads; this library serializes runtime state.
pub unsafe extern "C" fn nux_apple_surface_copy_metal_device(
    surface: *const NuxAppleSurface,
    out_metal_device: *mut *mut c_void,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || unsafe { poison_surface_handle(surface) },
        || {
            reset_out_handle(out_metal_device);
            reset_out_handle(out_result);
            if surface.is_null() || out_metal_device.is_null() {
                return NuxStatus::NullArgument;
            }
            let surface = unsafe { &*surface.cast::<AppleSurfaceHandle>() };
            let session_id = surface.token.session.id;
            let surface_id = surface.token.id;
            let device_identity =
                match surface
                    .token
                    .session
                    .worker
                    .call(Some(session_id), move |state| {
                        let attachment = state.surface_mut(session_id, surface_id)?;
                        attachment
                            .surface
                            .copy_metal_device(&attachment.factory)
                            .map(|device| device.expose_provenance())
                            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))
                    }) {
                    Ok(Ok(device_identity)) => device_identity,
                    Ok(Err(failure)) => return write_runtime_failure(out_result, failure),
                    Err(error) => return write_worker_call_failure(out_result, error),
                };
            let status = write_success(out_result, NuxSurfaceDisposition::None, false);
            unsafe {
                *out_metal_device = ptr::with_exposed_provenance_mut(device_identity);
            }
            status
        },
    )
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Resizes or zero-size-suspends one attached Apple surface.
///
/// # Safety
///
/// `surface` must be live and `out_result` must be null or writable. Calls may
/// originate on arbitrary threads; this library serializes runtime state.
pub unsafe extern "C" fn nux_apple_surface_resize(
    surface: *const NuxAppleSurface,
    pixel_width: u32,
    pixel_height: u32,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || unsafe { poison_surface_handle(surface) },
        || {
            reset_out_handle(out_result);
            if surface.is_null() {
                return NuxStatus::NullArgument;
            }
            let surface = unsafe { &*surface.cast::<AppleSurfaceHandle>() };
            let session_id = surface.token.session.id;
            let surface_id = surface.token.id;
            match surface
                .token
                .session
                .worker
                .call(Some(session_id), move |state| {
                    let attachment = state.surface_mut(session_id, surface_id)?;
                    attachment
                        .surface
                        .resize(&mut attachment.factory, pixel_width, pixel_height)
                        .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))
                }) {
                Ok(Ok(disposition)) => {
                    write_success(out_result, surface_disposition(disposition), false)
                }
                Ok(Err(failure)) => write_runtime_failure(out_result, failure),
                Err(error) => write_worker_call_failure(out_result, error),
            }
        },
    )
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Detaches presentation state while preserving logical session state.
///
/// # Safety
///
/// `surface` must be live and `out_result` must be null or writable. Calls may
/// originate on arbitrary threads; this library serializes runtime state.
pub unsafe extern "C" fn nux_apple_surface_detach(
    surface: *const NuxAppleSurface,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || unsafe { poison_surface_handle(surface) },
        || {
            reset_out_handle(out_result);
            if surface.is_null() {
                return NuxStatus::NullArgument;
            }
            let surface = unsafe { &*surface.cast::<AppleSurfaceHandle>() };
            let session_id = surface.token.session.id;
            let surface_id = surface.token.id;
            match surface
                .token
                .session
                .worker
                .call(Some(session_id), move |state| {
                    let attachment = state.surface_mut(session_id, surface_id)?;
                    attachment.surface.detach();
                    Ok::<(), RuntimeFailure>(())
                }) {
                Ok(Ok(())) => write_success(out_result, NuxSurfaceDisposition::None, false),
                Ok(Err(failure)) => write_runtime_failure(out_result, failure),
                Err(error) => write_worker_call_failure(out_result, error),
            }
        },
    )
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Reattaches logical presentation state after a detach.
///
/// # Safety
///
/// Handles, descriptor, and output storage must be valid. Calls may originate
/// on arbitrary threads; this library serializes runtime state.
pub unsafe extern "C" fn nux_apple_surface_reattach(
    surface: *const NuxAppleSurface,
    descriptor: *const NuxAppleSurfaceDescriptor,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || unsafe { poison_surface_handle(surface) },
        || {
            reset_out_handle(out_result);
            if surface.is_null() || descriptor.is_null() {
                return NuxStatus::NullArgument;
            }
            let surface = unsafe { &*surface.cast::<AppleSurfaceHandle>() };
            let struct_size = unsafe { read_struct_size(descriptor) };
            if struct_size < size_u32::<NuxAppleSurfaceDescriptor>() {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "Apple surface descriptor is smaller than this ABI requires",
                );
            }
            let descriptor = unsafe { *descriptor };
            let session_id = surface.token.session.id;
            let surface_id = surface.token.id;
            let pixel_width = descriptor.pixel_width;
            let pixel_height = descriptor.pixel_height;
            match surface
                .token
                .session
                .worker
                .call(Some(session_id), move |state| {
                    let attachment = state.surface_mut(session_id, surface_id)?;
                    attachment
                        .surface
                        .reattach(&mut attachment.factory, pixel_width, pixel_height)
                        .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))
                }) {
                Ok(Ok(disposition)) => {
                    write_success(out_result, surface_disposition(disposition), false)
                }
                Ok(Err(failure)) => write_runtime_failure(out_result, failure),
                Err(error) => write_worker_call_failure(out_result, error),
            }
        },
    )
}

#[unsafe(no_mangle)]
/// Releases one Apple-surface handle. Null is a no-op.
///
/// # Safety
///
/// A non-null pointer must be an owned handle returned by this library and not
/// previously released. Release must not race a call using the same handle.
pub unsafe extern "C" fn nux_apple_surface_free(surface: *mut NuxAppleSurface) {
    ffi_guard((), || {
        if !surface.is_null() {
            unsafe {
                #[cfg(feature = "apple-product")]
                drop(Box::from_raw(surface.cast::<AppleSurfaceHandle>()));
            }
        }
    })
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Advances a logical session and optionally renders its attached surface.
///
/// # Safety
///
/// `session` and `operation` must be live, valid pointers; `out_result` must
/// be null or writable. When rendering, `operation.apple_drawable` may be null
/// to report a bounded no-drawable outcome; otherwise it must be a live
/// `id<CAMetalDrawable>` retained until this synchronous call returns. A
/// drawable must not be supplied when rendering is disabled. Calls may
/// originate on arbitrary threads; this library serializes runtime state.
pub unsafe extern "C" fn nux_flow_render_session_advance(
    session: *const NuxFlowRenderSession,
    operation: *const NuxFrameOperation,
    out_result: *mut *mut NuxOperationResult,
) -> NuxStatus {
    ffi_guard_with_result(
        out_result,
        || unsafe { poison_session_handle(session) },
        || {
            reset_out_handle(out_result);
            if operation.is_null() {
                return NuxStatus::NullArgument;
            }
            let struct_size = unsafe { read_struct_size(operation) };
            if struct_size < size_u32::<NuxFrameOperation>() {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "frame operation is invalid",
                );
            }
            let operation = unsafe { *operation };
            let completion = match PendingFrameCompletion::from_operation(&operation) {
                Ok(completion) => completion,
                Err(diagnostic) => {
                    return write_failure(out_result, NuxStatus::InvalidArgument, diagnostic);
                }
            };
            if session.is_null() {
                return NuxStatus::NullArgument;
            }
            let session = unsafe { &*session.cast::<FlowRenderSessionHandle>() };
            if !operation.elapsed_seconds.is_finite() || operation.elapsed_seconds < 0.0 {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "frame operation is invalid",
                );
            }
            if !operation.render && !operation.apple_drawable.is_null() {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "a frame operation cannot supply an Apple drawable when rendering is disabled",
                );
            }
            if operation.completion_callback.is_some() && operation.apple_drawable.is_null() {
                return write_failure(
                    out_result,
                    NuxStatus::InvalidArgument,
                    "a frame completion callback requires an Apple drawable",
                );
            }
            let session_id = session.token.id;
            let elapsed_seconds = operation.elapsed_seconds;
            let render = operation.render;
            let drawable_identity = operation.apple_drawable.expose_provenance();
            match session.token.worker.call(Some(session_id), move |state| {
                state.require_live_session(session_id)?;
                let session = state.session_mut(session_id)?;
                let changed = if let Some(state_machine) = session.state_machine.as_mut() {
                    session
                        .instance
                        .advance_with_state_machine(state_machine, elapsed_seconds)
                } else {
                    session.instance.advance(elapsed_seconds)
                };
                if !render {
                    return Ok((NuxSurfaceDisposition::None, changed));
                }
                let attachment = session
                    .attachment
                    .as_mut()
                    .ok_or_else(|| RuntimeFailure::surface("surface is not attached"))?;
                let (viewport_width, viewport_height) = attachment.surface.dimensions();
                if viewport_width == 0 || viewport_height == 0 {
                    return Ok((NUX_SURFACE_DISPOSITION_SKIPPED_ZERO_SIZE, changed));
                }
                let (artboard_width, artboard_height) = session.instance.artboard_dimensions();
                let presentation_transform = centered_contain_transform(
                    artboard_width,
                    artboard_height,
                    viewport_width,
                    viewport_height,
                )?;
                let mut frame = attachment.factory.begin_frame(0x0000_0000);
                frame.transform(presentation_transform);
                session
                    .instance
                    .draw_with_render_cache(
                        &mut attachment.factory,
                        &mut frame,
                        &mut session.render_cache,
                    )
                    .map_err(|error| RuntimeFailure::runtime(format!("{error:#}")))?;
                let drawable = ptr::with_exposed_provenance_mut::<c_void>(drawable_identity);
                let completion = completion.into_renderer_completion();
                let (disposition, _metrics) = unsafe {
                    attachment
                        .surface
                        .present(&mut attachment.factory, frame, drawable, completion)
                }
                .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
                Ok((surface_disposition(disposition), changed))
            }) {
                Ok(Ok((disposition, changed))) => write_success(out_result, disposition, changed),
                Ok(Err(failure)) => write_runtime_failure(out_result, failure),
                Err(error) => write_worker_call_failure(out_result, error),
            }
        },
    )
}

#[unsafe(no_mangle)]
/// Returns an operation result's status, or `NULL_ARGUMENT` for null.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_operation_result_status(
    result: *const NuxOperationResult,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if result.is_null() {
            NuxStatus::NullArgument
        } else {
            unsafe { (*result).status }
        }
    })
}

#[unsafe(no_mangle)]
/// Returns an operation result's surface disposition.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_operation_result_surface_disposition(
    result: *const NuxOperationResult,
) -> NuxSurfaceDisposition {
    ffi_guard(NuxSurfaceDisposition::Fatal, || {
        if result.is_null() {
            NuxSurfaceDisposition::Fatal
        } else {
            unsafe { (*result).surface_disposition }
        }
    })
}

#[unsafe(no_mangle)]
/// Returns whether an operation changed logical runtime state.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_operation_result_changed(result: *const NuxOperationResult) -> bool {
    ffi_guard(false, || !result.is_null() && unsafe { (*result).changed })
}

#[unsafe(no_mangle)]
/// Borrows the diagnostic bytes stored by an operation result.
///
/// # Safety
///
/// `result` must be null or live, and `out_diagnostic` must point to writable
/// storage. A returned byte view expires when `result` is released.
pub unsafe extern "C" fn nux_operation_result_diagnostic(
    result: *const NuxOperationResult,
    out_diagnostic: *mut NuxByteView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_diagnostic.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_diagnostic = NuxByteView::default();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let diagnostic = unsafe { &(*result).diagnostic };
        unsafe {
            *out_diagnostic = NuxByteView {
                data: diagnostic.as_ptr(),
                len: u64::try_from(diagnostic.len()).unwrap_or(u64::MAX),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Releases one operation result. Null is a no-op.
///
/// # Safety
///
/// A non-null pointer must be an owned result returned by this library and
/// must not have been released before.
pub unsafe extern "C" fn nux_operation_result_free(result: *mut NuxOperationResult) {
    ffi_guard((), || {
        if !result.is_null() {
            unsafe {
                drop(Box::from_raw(result));
            }
        }
    })
}

fn size_u32<T>() -> u32 {
    u32::try_from(std::mem::size_of::<T>()).unwrap_or(u32::MAX)
}

unsafe fn read_struct_size<T>(value: *const T) -> u32 {
    // SAFETY: every versioned input begins with a u32 `struct_size`, and the
    // FFI contract requires the non-null pointer to be aligned and readable
    // for at least that prefix. The caller's declared size is validated before
    // any full-structure read occurs.
    unsafe { value.cast::<u32>().read() }
}

fn byte_vec(view: NuxByteView, maximum_length: usize) -> Result<Vec<u8>, NuxStatus> {
    let len = usize::try_from(view.len).map_err(|_| NuxStatus::InvalidArgument)?;
    if len > maximum_length || len > isize::MAX as usize {
        return Err(NuxStatus::InvalidArgument);
    }
    if view.data.is_null() && len != 0 {
        return Err(NuxStatus::NullArgument);
    }
    if len == 0 {
        return Ok(Vec::new());
    }
    // SAFETY: the caller promises the view remains valid for the duration of
    // the importing call. Copying here prevents a caller-owned lifetime from
    // leaking into the retained runtime context.
    Ok(unsafe { slice::from_raw_parts(view.data, len) }.to_vec())
}

fn optional_utf8_string(view: NuxByteView) -> Result<Option<String>, NuxStatus> {
    if view.data.is_null() && view.len == 0 {
        return Ok(None);
    }
    let bytes = byte_vec(view, MAX_SELECTOR_BYTE_LENGTH)?;
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| NuxStatus::InvalidArgument)
}

#[cfg(feature = "apple-product")]
unsafe fn poison_session_handle(session: *const NuxFlowRenderSession) {
    if session.is_null() {
        return;
    }
    let handle = unsafe { &*session.cast::<FlowRenderSessionHandle>() };
    handle.token.worker.poison_session(handle.token.id);
}

#[cfg(feature = "apple-product")]
unsafe fn poison_surface_handle(surface: *const NuxAppleSurface) {
    if surface.is_null() {
        return;
    }
    let handle = unsafe { &*surface.cast::<AppleSurfaceHandle>() };
    handle
        .token
        .session
        .worker
        .poison_session(handle.token.session.id);
}

fn reset_out_handle<T>(out: *mut *mut T) {
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
}

fn write_success(
    out_result: *mut *mut NuxOperationResult,
    surface_disposition: NuxSurfaceDisposition,
    changed: bool,
) -> NuxStatus {
    replace_result(
        out_result,
        NuxOperationResult::success(surface_disposition, changed),
    );
    NuxStatus::Ok
}

fn write_failure(
    out_result: *mut *mut NuxOperationResult,
    status: NuxStatus,
    diagnostic: impl Into<Vec<u8>>,
) -> NuxStatus {
    replace_result(out_result, NuxOperationResult::failure(status, diagnostic));
    status
}

#[cfg(feature = "apple-product")]
fn write_runtime_failure(
    out_result: *mut *mut NuxOperationResult,
    failure: RuntimeFailure,
) -> NuxStatus {
    write_failure(out_result, failure.status, failure.diagnostic)
}

#[cfg(feature = "apple-product")]
fn write_worker_call_failure(
    out_result: *mut *mut NuxOperationResult,
    error: WorkerCallError,
) -> NuxStatus {
    let diagnostic = match error {
        WorkerCallError::Panicked => PANIC_DIAGNOSTIC,
        WorkerCallError::Unavailable => "runtime worker is unavailable",
    };
    write_failure(out_result, NuxStatus::RuntimeError, diagnostic)
}

fn replace_result(out_result: *mut *mut NuxOperationResult, result: NuxOperationResult) {
    if out_result.is_null() {
        return;
    }
    let replacement = Box::into_raw(Box::new(result));
    let previous = unsafe { std::mem::replace(&mut *out_result, replacement) };
    if !previous.is_null() {
        unsafe {
            drop(Box::from_raw(previous));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    use objc2::rc::{Retained, autoreleasepool};
    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    use objc2::runtime::ProtocolObject;
    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    use objc2_core_foundation::CGSize;
    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    use objc2_metal::{MTLDevice, MTLPixelFormat};
    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    use objc2_quartz_core::CAMetalLayer;
    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    use std::sync::atomic::{AtomicBool, Ordering};

    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    unsafe extern "C" fn mark_frame_completed(context: *mut c_void) {
        if let Some(completed) = std::ptr::NonNull::new(context.cast::<AtomicBool>()) {
            unsafe {
                completed.as_ref().store(true, Ordering::Release);
            }
        }
    }

    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    fn wait_for_frame_completion(completed: &AtomicBool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !completed.load(Ordering::Acquire) {
            assert!(
                std::time::Instant::now() < deadline,
                "Metal frame completion callback timed out"
            );
            std::thread::yield_now();
        }
    }

    #[cfg(feature = "apple-product")]
    fn product_fixture_bytes() -> Vec<u8> {
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/animation/smi_test.riv");
        std::fs::read(fixture).expect("product fixture must be readable")
    }

    #[cfg(feature = "apple-product")]
    fn product_fixture_worker() -> Arc<RuntimeWorker> {
        match RuntimeWorker::spawn(product_fixture_bytes()) {
            Ok(worker) => worker,
            Err(_) => panic!("product fixture must create a runtime worker"),
        }
    }

    #[test]
    fn abi_compatibility_requires_exact_major_and_at_least_the_requested_minor() {
        assert_eq!(nux_runtime_require_abi(1, 0), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(2, 0), NuxStatus::AbiMismatch);
        assert_eq!(nux_runtime_require_abi(1, 1), NuxStatus::AbiMismatch);
    }

    #[test]
    fn provenance_is_process_static_json_with_required_identity_fields() {
        let mut view = NuxByteView::default();
        assert_eq!(
            unsafe { nux_runtime_build_provenance(&mut view) },
            NuxStatus::Ok
        );
        let bytes = unsafe { slice::from_raw_parts(view.data, view.len as usize) };
        let json = std::str::from_utf8(bytes).expect("build provenance must be UTF-8");
        for field in [
            "\"schemaVersion\":1",
            "\"runtimeVersion\"",
            "\"sourceRevision\"",
            "\"target\"",
            "\"profile\"",
            "\"rustc\"",
            "\"wgpuVersion\":\"30.0.0\"",
            "\"luaurVersion\":null",
        ] {
            assert!(json.contains(field), "missing {field} in {json}");
        }
        if let Some(profile) = option_env!("NUX_RUNTIME_BUILD_PROFILE") {
            let expected = format!("\"profile\":\"{profile}\"");
            assert!(json.contains(&expected), "missing {expected} in {json}");
        }
    }

    #[test]
    fn result_getters_are_null_safe_and_diagnostics_borrow_the_result() {
        let mut diagnostic = NuxByteView {
            data: std::ptr::NonNull::<u8>::dangling().as_ptr(),
            len: 99,
        };
        assert_eq!(
            unsafe { nux_operation_result_diagnostic(ptr::null(), &mut diagnostic) },
            NuxStatus::NullArgument
        );
        assert!(diagnostic.data.is_null());
        assert_eq!(diagnostic.len, 0);

        let result = Box::into_raw(Box::new(NuxOperationResult::failure(
            NuxStatus::InvalidArgument,
            b"bad request".to_vec(),
        )));
        assert_eq!(
            unsafe { nux_operation_result_diagnostic(result, &mut diagnostic) },
            NuxStatus::Ok
        );
        let bytes = unsafe { slice::from_raw_parts(diagnostic.data, diagnostic.len as usize) };
        assert_eq!(bytes, b"bad request");
        unsafe { nux_operation_result_free(result) };
    }

    #[test]
    fn byte_views_are_bounded_before_constructing_a_caller_slice() {
        let pointer = std::ptr::NonNull::<u8>::dangling().as_ptr();
        assert_eq!(
            byte_vec(
                NuxByteView {
                    data: pointer,
                    len: MAX_ARTIFACT_BYTE_LENGTH as u64 + 1,
                },
                MAX_ARTIFACT_BYTE_LENGTH,
            ),
            Err(NuxStatus::InvalidArgument)
        );
        assert_eq!(
            optional_utf8_string(NuxByteView {
                data: pointer,
                len: MAX_SELECTOR_BYTE_LENGTH as u64 + 1,
            }),
            Err(NuxStatus::InvalidArgument)
        );
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn versioned_inputs_reject_a_short_prefix_without_reading_the_full_struct() {
        let mut storage = std::mem::MaybeUninit::<NuxFlowImportRequest>::uninit();
        unsafe {
            storage.as_mut_ptr().cast::<u32>().write(size_u32::<u32>());
        }
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_runtime_context_create(storage.as_ptr(), &mut context, &mut result) },
            NuxStatus::InvalidArgument
        );
        assert!(context.is_null());
        assert!(!result.is_null());
        unsafe { nux_operation_result_free(result) };
    }

    #[test]
    fn surface_dispositions_are_stable_c_abi_values() {
        assert_eq!(NUX_SURFACE_DISPOSITION_NONE, 0);
        assert_eq!(NUX_SURFACE_DISPOSITION_PRESENTED, 1);
        assert_eq!(NUX_SURFACE_DISPOSITION_SKIPPED_TIMEOUT, 3);
        assert_eq!(NUX_SURFACE_DISPOSITION_RECREATED, 6);
        assert_eq!(NUX_SURFACE_DISPOSITION_FATAL, 9);
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn centered_contain_transform_scales_and_letterboxes_the_artboard() {
        assert_eq!(
            centered_contain_transform(100.0, 50.0, 300, 300).expect("valid contain transform"),
            Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 75.0])
        );
        assert_eq!(
            centered_contain_transform(100.0, 200.0, 300, 300).expect("valid contain transform"),
            Mat2D([1.5, 0.0, 0.0, 1.5, 75.0, 0.0])
        );
        assert!(centered_contain_transform(0.0, 50.0, 300, 300).is_err());
        assert!(centered_contain_transform(100.0, f32::NAN, 300, 300).is_err());
    }

    #[test]
    fn every_c_export_opens_with_the_panic_firewall() {
        let source = include_str!("lib.rs");
        let mut checked = 0usize;
        for prefix in ["pub unsafe extern \"C\" fn ", "pub extern \"C\" fn "] {
            for (index, _) in source.match_indices(prefix) {
                let rest = &source[index..];
                let body_start = rest.find('{').expect("extern function body");
                let body = rest[body_start + 1..].trim_start();
                let name_end = rest.find('(').expect("extern function parameters");
                let name = &rest[prefix.len()..name_end];
                assert!(
                    body.starts_with("ffi_guard(") || body.starts_with("ffi_guard_with_result("),
                    "extern C function `{name}` bypasses the panic firewall"
                );
                checked = checked.saturating_add(1);
            }
        }
        assert_eq!(
            checked, 20,
            "update the firewall audit for every new export"
        );
    }

    #[test]
    fn panic_firewall_converts_panics_to_the_declared_fallback() {
        assert_eq!(
            ffi_guard(NuxStatus::RuntimeError, || -> NuxStatus {
                panic!("deliberate ABI panic probe")
            }),
            NuxStatus::RuntimeError
        );
        ffi_guard((), || panic!("deliberate void ABI panic probe"));

        let poisoned = std::cell::Cell::new(false);
        let mut result = ptr::null_mut();
        assert_eq!(
            ffi_guard_with_result(
                &mut result,
                || poisoned.set(true),
                || -> NuxStatus { panic!("deliberate operation ABI panic probe") },
            ),
            NuxStatus::RuntimeError
        );
        assert!(poisoned.get());
        assert!(!result.is_null());
        assert_eq!(
            unsafe { nux_operation_result_status(result) },
            NuxStatus::RuntimeError
        );
        let mut diagnostic = NuxByteView::default();
        assert_eq!(
            unsafe { nux_operation_result_diagnostic(result, &mut diagnostic) },
            NuxStatus::Ok
        );
        let diagnostic = unsafe { slice::from_raw_parts(diagnostic.data, diagnostic.len as usize) };
        assert_eq!(diagnostic, PANIC_DIAGNOSTIC.as_bytes());
        unsafe { nux_operation_result_free(result) };
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn runtime_state_is_confined_to_one_worker_thread_across_callers() {
        let worker = product_fixture_worker();
        let caller_thread_id = thread::current().id();
        let owner_thread_id = worker
            .probe_thread_id()
            .expect("worker thread must answer a confinement probe");
        assert_ne!(owner_thread_id, caller_thread_id);

        let mut callers = Vec::new();
        for _ in 0..4 {
            let worker = Arc::clone(&worker);
            callers.push(thread::spawn(move || worker.probe_thread_id()));
        }
        for caller in callers {
            let observed = caller
                .join()
                .expect("probe caller must not panic")
                .expect("worker must answer every probe");
            assert_eq!(observed, owner_thread_id);
        }
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn worker_job_panics_poison_the_session_and_return_an_owned_diagnostic() {
        let worker = product_fixture_worker();
        let session_id = match worker.call(None, |state| state.create_session(None, None)) {
            Ok(Ok(session_id)) => session_id,
            _ => panic!("fixture must create a default render session"),
        };
        let panic_result = worker.call(Some(session_id), |_| -> () {
            panic!("deliberate worker panic probe");
        });
        assert_eq!(panic_result, Err(WorkerCallError::Panicked));

        let session_health = worker
            .call(Some(session_id), move |state| {
                state.require_live_session(session_id)
            })
            .expect("worker must remain available after a caught job panic");
        let failure = match session_health {
            Ok(()) => panic!("panicking session must be poisoned"),
            Err(failure) => failure,
        };
        assert_eq!(failure.status, NuxStatus::RuntimeError);
        assert_eq!(failure.diagnostic, PANIC_DIAGNOSTIC);

        let mut result = ptr::null_mut();
        assert_eq!(
            write_worker_call_failure(&mut result, WorkerCallError::Panicked),
            NuxStatus::RuntimeError
        );
        let mut diagnostic = NuxByteView::default();
        assert_eq!(
            unsafe { nux_operation_result_diagnostic(result, &mut diagnostic) },
            NuxStatus::Ok
        );
        let diagnostic = unsafe { slice::from_raw_parts(diagnostic.data, diagnostic.len as usize) };
        assert_eq!(diagnostic, PANIC_DIAGNOSTIC.as_bytes());
        unsafe { nux_operation_result_free(result) };
    }

    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    #[test]
    fn token_destruction_finishes_surface_and_session_cleanup_before_returning() {
        let worker = product_fixture_worker();
        let session_id = match worker.call(None, |state| state.create_session(None, None)) {
            Ok(Ok(session_id)) => session_id,
            _ => panic!("fixture must create a default render session"),
        };
        let surface_id = match worker.call(Some(session_id), move |state| {
            state.attach_surface(session_id, 8, 8)
        }) {
            Ok(Ok(surface_id)) => surface_id,
            _ => panic!("fixture must attach logical Apple presentation state"),
        };
        let session = Arc::new(SessionToken {
            worker: Arc::clone(&worker),
            id: session_id,
        });
        let surface = Arc::new(SurfaceToken {
            session: Arc::clone(&session),
            id: surface_id,
        });

        drop(surface);

        let surface_is_gone = worker
            .call(None, move |state| {
                state
                    .session(session_id)
                    .is_ok_and(|session| session.attachment.is_none())
            })
            .expect("worker must confirm synchronous surface cleanup");
        assert!(surface_is_gone);

        drop(session);

        let session_is_gone = worker
            .call(None, move |state| !state.sessions.contains_key(&session_id))
            .expect("worker must confirm synchronous session cleanup");
        assert!(session_is_gone);
    }

    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    #[test]
    fn public_c_abi_renders_to_cametal_layer_and_preserves_parent_first_ownership() {
        autoreleasepool(|_| {
            let bytes = product_fixture_bytes();
            let request = NuxFlowImportRequest {
                struct_size: size_u32::<NuxFlowImportRequest>(),
                artifact_bytes: NuxByteView {
                    data: bytes.as_ptr(),
                    len: bytes.len() as u64,
                },
            };
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
                NuxStatus::Ok
            );
            unsafe { nux_operation_result_free(result) };

            let session_descriptor = NuxFlowSessionDescriptor {
                struct_size: size_u32::<NuxFlowSessionDescriptor>(),
                artboard_name: NuxByteView::default(),
                state_machine_name: NuxByteView::default(),
            };
            let mut session = ptr::null_mut();
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_create(
                        context,
                        &session_descriptor,
                        &mut session,
                        &mut result,
                    )
                },
                NuxStatus::Ok
            );
            unsafe { nux_operation_result_free(result) };

            let layer = CAMetalLayer::new();
            let mut surface_descriptor = NuxAppleSurfaceDescriptor {
                struct_size: size_u32::<NuxAppleSurfaceDescriptor>(),
                pixel_width: 8,
                pixel_height: 8,
            };
            let mut surface = ptr::null_mut();
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_attach_apple_surface(
                        session,
                        &surface_descriptor,
                        &mut surface,
                        &mut result,
                    )
                },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_RECREATED
            );
            unsafe { nux_operation_result_free(result) };

            let mut metal_device = ptr::null_mut();
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_apple_surface_copy_metal_device(surface, &mut metal_device, &mut result)
                },
                NuxStatus::Ok
            );
            assert!(!metal_device.is_null());
            unsafe { nux_operation_result_free(result) };
            let metal_device: Retained<ProtocolObject<dyn MTLDevice>> = unsafe {
                Retained::from_raw(metal_device.cast())
                    .expect("copy_metal_device must return a +1 MTLDevice")
            };
            layer.setDevice(Some(&metal_device));
            layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            layer.setFramebufferOnly(true);
            layer.setAllowsNextDrawableTimeout(true);
            layer.setDrawableSize(CGSize::new(8.0, 8.0));

            let no_drawable_operation = NuxFrameOperation {
                struct_size: size_u32::<NuxFrameOperation>(),
                elapsed_seconds: 0.0,
                render: true,
                apple_drawable: ptr::null_mut(),
                completion_context: ptr::null_mut(),
                completion_callback: None,
            };
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_advance(session, &no_drawable_operation, &mut result)
                },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_SKIPPED_TIMEOUT
            );
            unsafe { nux_operation_result_free(result) };

            let drawable = layer
                .nextDrawable()
                .expect("configured CAMetalLayer must provide a drawable");
            let drawable_pointer = Retained::as_ptr(&drawable).cast_mut().cast::<c_void>();
            let invalid_operation = NuxFrameOperation {
                struct_size: size_u32::<NuxFrameOperation>(),
                elapsed_seconds: 0.0,
                render: false,
                apple_drawable: drawable_pointer,
                completion_context: ptr::null_mut(),
                completion_callback: None,
            };
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_advance(session, &invalid_operation, &mut result)
                },
                NuxStatus::InvalidArgument
            );
            unsafe { nux_operation_result_free(result) };

            let completed = AtomicBool::new(false);
            let mut operation = NuxFrameOperation {
                struct_size: size_u32::<NuxFrameOperation>(),
                elapsed_seconds: 0.0,
                render: true,
                apple_drawable: drawable_pointer,
                completion_context: (&completed as *const AtomicBool).cast_mut().cast(),
                completion_callback: Some(mark_frame_completed),
            };
            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_flow_render_session_advance(session, &operation, &mut result) },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_PRESENTED
            );
            unsafe { nux_operation_result_free(result) };
            wait_for_frame_completion(&completed);
            operation.completion_context = ptr::null_mut();
            operation.completion_callback = None;

            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_apple_surface_resize(surface, 0, 0, &mut result) },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_SKIPPED_ZERO_SIZE
            );
            unsafe { nux_operation_result_free(result) };

            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_apple_surface_detach(surface, &mut result) },
                NuxStatus::Ok
            );
            unsafe { nux_operation_result_free(result) };

            surface_descriptor.pixel_width = 16;
            surface_descriptor.pixel_height = 12;
            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_apple_surface_reattach(surface, &surface_descriptor, &mut result) },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_RECREATED
            );
            unsafe { nux_operation_result_free(result) };

            layer.setDrawableSize(CGSize::new(16.0, 12.0));
            let reattached_drawable = layer
                .nextDrawable()
                .expect("reattached CAMetalLayer must provide a drawable");
            operation.apple_drawable = Retained::as_ptr(&reattached_drawable)
                .cast_mut()
                .cast::<c_void>();
            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_flow_render_session_advance(session, &operation, &mut result) },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_PRESENTED
            );
            unsafe { nux_operation_result_free(result) };

            unsafe {
                // The public ownership contract allows children to outlive
                // their C parent handles. The surface retains both parents.
                nux_flow_runtime_context_free(context);
                nux_flow_render_session_free(session);
            }
            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_apple_surface_resize(surface, 20, 10, &mut result) },
                NuxStatus::Ok
            );
            unsafe {
                nux_operation_result_free(result);
                nux_apple_surface_free(surface);
            }
        });
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn context_import_session_advance_and_parent_first_teardown_use_the_product_handles() {
        let bytes = product_fixture_bytes();
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len() as u64,
            },
        };
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
            NuxStatus::Ok
        );
        assert!(!context.is_null());
        assert_eq!(
            unsafe { nux_operation_result_status(result) },
            NuxStatus::Ok
        );
        unsafe { nux_operation_result_free(result) };

        let artboard_name = b"artboard to nest";
        let state_machine_name = b"State Machine 1";
        let named_descriptor = NuxFlowSessionDescriptor {
            struct_size: size_u32::<NuxFlowSessionDescriptor>(),
            artboard_name: NuxByteView {
                data: artboard_name.as_ptr(),
                len: artboard_name.len() as u64,
            },
            state_machine_name: NuxByteView {
                data: state_machine_name.as_ptr(),
                len: state_machine_name.len() as u64,
            },
        };
        let mut named_session = ptr::null_mut();
        result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_flow_render_session_create(
                    context,
                    &named_descriptor,
                    &mut named_session,
                    &mut result,
                )
            },
            NuxStatus::Ok
        );
        unsafe {
            nux_operation_result_free(result);
            nux_flow_render_session_free(named_session);
        }

        let missing_name = b"missing artboard";
        let missing_descriptor = NuxFlowSessionDescriptor {
            struct_size: size_u32::<NuxFlowSessionDescriptor>(),
            artboard_name: NuxByteView {
                data: missing_name.as_ptr(),
                len: missing_name.len() as u64,
            },
            state_machine_name: NuxByteView::default(),
        };
        named_session = ptr::null_mut();
        result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_flow_render_session_create(
                    context,
                    &missing_descriptor,
                    &mut named_session,
                    &mut result,
                )
            },
            NuxStatus::NotFound
        );
        assert!(named_session.is_null());
        assert_eq!(
            unsafe { nux_operation_result_status(result) },
            NuxStatus::NotFound
        );
        unsafe { nux_operation_result_free(result) };

        let descriptor = NuxFlowSessionDescriptor {
            struct_size: size_u32::<NuxFlowSessionDescriptor>(),
            artboard_name: NuxByteView::default(),
            state_machine_name: NuxByteView::default(),
        };
        let mut session = ptr::null_mut();
        result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_flow_render_session_create(context, &descriptor, &mut session, &mut result)
            },
            NuxStatus::Ok
        );
        assert!(!session.is_null());
        unsafe {
            nux_operation_result_free(result);
            // Child handles retain their parents, so Swift teardown ordering
            // cannot turn a live session into a dangling reference.
            nux_flow_runtime_context_free(context);
        }

        let operation = NuxFrameOperation {
            struct_size: size_u32::<NuxFrameOperation>(),
            elapsed_seconds: 0.016,
            render: false,
            apple_drawable: ptr::null_mut(),
            completion_context: ptr::null_mut(),
            completion_callback: None,
        };
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_advance(session, &operation, &mut result) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_operation_result_status(result) },
            NuxStatus::Ok
        );
        unsafe {
            nux_operation_result_free(result);
            nux_flow_render_session_free(session);
        }
    }
}
