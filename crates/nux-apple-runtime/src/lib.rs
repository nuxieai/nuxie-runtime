//! Product C ABI for the Nuxie Apple flow runtime.

#[cfg(feature = "apple-product")]
mod artifact;
mod session_v12;

pub use session_v12::*;

#[cfg(all(feature = "apple-product", panic = "abort"))]
compile_error!(
    "nux-apple-runtime's apple-product feature requires panic=unwind; use the release-apple profile"
);

use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use std::slice;

#[cfg(feature = "apple-product")]
use artifact::{
    ArtifactAuthorization, ArtifactDiagnosticSeverity, ExternalAssetInput, ExternalAssetKind,
    FlowArtifactImportInput, MAX_EXTERNAL_ASSET_COUNT, SelectedArtifactSigningKey,
    validate_flow_artifact_import,
};
#[cfg(feature = "apple-product")]
use nuxie::{
    ApplePresentationCompletion, AppleSurface, File, Mat2D, RenderMode, Renderer,
    SurfaceDisposition, WgpuFactory,
    flow_session::{FlowPlayerSelector, FlowSession, FlowSessionConfig, FlowSessionErrorKind},
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
pub const NUX_RUNTIME_ABI_MINOR: u16 = 6;
const MINIMUM_SUPPORTED_ABI_MINOR: u16 = 1;

const MAX_ARTIFACT_BYTE_LENGTH: usize = 67_108_864;
const MAX_MANIFEST_BYTE_LENGTH: usize = 4_194_304;
const MAX_SIGNATURE_BYTE_LENGTH: usize = 65_536;
const MAX_AUTHORIZATION_KEY_ID_BYTE_LENGTH: usize = 256;
const ED25519_PUBLIC_KEY_BYTE_LENGTH: usize = 32;
const MAX_EXTERNAL_ASSET_TOTAL_BYTE_LENGTH: usize = 134_217_728;
const MAX_SELECTOR_BYTE_LENGTH: usize = 4_096;
const MAX_ASSET_SOURCE_KEY_BYTE_LENGTH: usize = MAX_MANIFEST_BYTE_LENGTH;
const PANIC_DIAGNOSTIC: &str = "runtime panicked; the affected flow session is terminated";
const RESULT_LIMIT_DIAGNOSTIC_CODE: &[u8] = b"nux_runtime.result_limit_exceeded";
const SCRIPT_RESOURCE_DIAGNOSTIC_CODE: &[u8] = b"nux_runtime.script_resource_exceeded";
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

/// Stable-width script authorization result set during artifact import.
pub type NuxScriptAuthorization = u32;

pub const NUX_SCRIPT_AUTHORIZATION_NOT_APPLICABLE: NuxScriptAuthorization = 0;
pub const NUX_SCRIPT_AUTHORIZATION_VISUAL_ONLY: NuxScriptAuthorization = 1;
pub const NUX_SCRIPT_AUTHORIZATION_AUTHENTICATED: NuxScriptAuthorization = 2;

/// Stable-width structured diagnostic severity.
pub type NuxDiagnosticSeverity = u32;

pub const NUX_DIAGNOSTIC_SEVERITY_DEBUG: NuxDiagnosticSeverity = 0;
pub const NUX_DIAGNOSTIC_SEVERITY_WARNING: NuxDiagnosticSeverity = 1;
pub const NUX_DIAGNOSTIC_SEVERITY_FATAL: NuxDiagnosticSeverity = 2;

/// Stable-width external artifact asset kind.
pub type NuxFlowExternalAssetKind = u32;

pub const NUX_FLOW_EXTERNAL_ASSET_KIND_IMAGE: NuxFlowExternalAssetKind = 1;
pub const NUX_FLOW_EXTERNAL_ASSET_KIND_FONT: NuxFlowExternalAssetKind = 2;

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
pub struct NuxFlowAuthorizationKey {
    pub struct_size: u32,
    pub key_id: NuxByteView,
    /// Exactly 32 raw Ed25519 public-key bytes.
    pub ed25519_public_key: NuxByteView,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// One element of `NuxFlowImportRequest.external_assets`. Because the array has
/// no independent stride, every element must use this exact published size.
pub struct NuxFlowExternalAsset {
    pub struct_size: u32,
    pub kind: NuxFlowExternalAssetKind,
    /// Serialized `FileAsset.assetId`, not an asset-list ordinal.
    pub asset_id: u32,
    pub required: bool,
    /// Distinguishes explicitly omitted optional content from supplied empty bytes.
    pub provided: bool,
    pub unique_name: NuxByteView,
    pub source_key: NuxByteView,
    pub expected_sha256: NuxByteView,
    /// Supplied encoded bytes. Image content is decoded during trusted import
    /// and must fit the Apple-safe 8,192-pixel/64-MiB decoded-image limits.
    /// Invalid required images abort import; invalid optional images are
    /// omitted with a structured warning.
    pub bytes: NuxByteView,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Full ABI 1.1 artifact-import contract. `struct_size` must cover this entire
/// published layout; the artifact manifest and acquisition identities are
/// required for every import.
pub struct NuxFlowImportRequest {
    pub struct_size: u32,
    /// Exact verified visual-runtime bytes. The field is container-neutral so
    /// the current RIV adapter can later be replaced without changing sessions.
    pub artifact_bytes: NuxByteView,
    /// UTF-8 acquisition identity used to prevent cross-flow replay.
    pub expected_flow_id: NuxByteView,
    /// UTF-8 acquisition identity used to prevent cross-build replay.
    pub expected_build_id: NuxByteView,
    /// Exact signed artifact manifest bytes.
    pub manifest_bytes: NuxByteView,
    /// Optional exact detached signature-envelope bytes. Only `{NULL, 0}` is
    /// absent; a non-null empty view is present malformed evidence.
    pub signature_envelope_bytes: NuxByteView,
    /// Optional Nuxie-selected validation material. This is evidence, never a
    /// caller-supplied authorization decision.
    pub selected_key: *const NuxFlowAuthorizationKey,
    /// Ordered manifest asset inputs, already resolved to bytes or an explicit omission.
    pub external_assets: *const NuxFlowExternalAsset,
    pub external_asset_count: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Frozen ABI-major-1 diagnostic output layout. Callers initialize
/// `struct_size` to the exact published size before invoking an accessor.
pub struct NuxDiagnosticView {
    pub struct_size: u32,
    pub severity: NuxDiagnosticSeverity,
    pub code: NuxByteView,
    pub message: NuxByteView,
}

impl Default for NuxDiagnosticView {
    fn default() -> Self {
        Self {
            struct_size: size_u32::<Self>(),
            severity: NUX_DIAGNOSTIC_SEVERITY_DEBUG,
            code: NuxByteView::default(),
            message: NuxByteView::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowSessionDescriptor {
    pub struct_size: u32,
    /// UTF-8 authored artboard name. A null view selects the default artboard.
    pub artboard_name: NuxByteView,
    /// UTF-8 authored state-machine name. A null view uses the shared authored
    /// fallback policy: default state machine, state-machine zero, linear
    /// animation zero, then a static artboard.
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
    gpu_generation: u64,
    sessions: HashMap<SessionId, SessionState>,
    next_session_id: SessionId,
    next_surface_id: SurfaceId,
}

#[cfg(feature = "apple-product")]
struct SessionState {
    is_fatal: bool,
    fatal_diagnostic: Option<String>,
    flow_session: FlowSession,
    // A stable address is part of the script renderer-domain contract. The
    // factory belongs to the logical session, not to its optional surface.
    factory: Box<WgpuFactory>,
    renderer_generation: u64,
    legacy_timestamp_seconds: f64,
    #[cfg(test)]
    render_attempts: usize,
    #[cfg(test)]
    injected_device_loss: bool,
    #[cfg(test)]
    panic_on_next_configured_operation: bool,
    attachment: Option<SurfaceState>,
}

#[cfg(feature = "apple-product")]
struct SurfaceState {
    id: SurfaceId,
    surface: AppleSurface,
}

#[cfg(feature = "apple-product")]
impl SessionState {
    fn terminalize(&mut self, diagnostic: impl Into<String>) {
        self.is_fatal = true;
        self.fatal_diagnostic = Some(diagnostic.into());
    }

    fn preflight_present(
        &self,
        drawable_available: bool,
    ) -> Result<Option<SurfaceDisposition>, RuntimeFailure> {
        let attachment = self
            .attachment
            .as_ref()
            .ok_or_else(|| RuntimeFailure::surface("surface is not attached"))?;
        #[cfg(test)]
        if self.injected_device_loss {
            return Ok(Some(SurfaceDisposition::DeviceLost));
        }
        attachment
            .surface
            .preflight_present(&self.factory, drawable_available)
            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))
    }

    fn requires_device_recovery(&self) -> bool {
        #[cfg(test)]
        if self.injected_device_loss {
            return true;
        }
        self.factory.device_is_lost()
    }
}

#[cfg(feature = "apple-product")]
fn terminalize_after_committed_advance_failure(
    session: &mut SessionState,
    phase: &str,
    failure: RuntimeFailure,
) -> RuntimeFailure {
    session.terminalize(format!(
        "flow session is terminal after a committed advance failed during {phase}: {}",
        failure.diagnostic
    ));
    failure
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
    Import { code: String, message: String },
    Runtime(String),
    Panicked,
}

#[cfg(feature = "apple-product")]
struct RuntimeImportDiagnostic {
    severity: NuxDiagnosticSeverity,
    code: String,
    message: String,
}

#[cfg(feature = "apple-product")]
struct RuntimeImportMetadata {
    authorization: NuxScriptAuthorization,
    authenticated_key_id: Option<String>,
    diagnostics: Vec<RuntimeImportDiagnostic>,
}

#[cfg(feature = "apple-product")]
struct WorkerInitialization {
    thread_id: ThreadId,
    metadata: RuntimeImportMetadata,
}

#[cfg(feature = "apple-product")]
fn import_runtime_input(
    input: FlowArtifactImportInput,
) -> Result<(File, RuntimeImportMetadata), WorkerStartError> {
    let validated =
        validate_flow_artifact_import(input).map_err(|error| WorkerStartError::Import {
            code: error.code.to_owned(),
            message: error.message,
        })?;
    let mut file = validated.file;
    let mut diagnostics = validated
        .diagnostics
        .into_iter()
        .map(|diagnostic| RuntimeImportDiagnostic {
            severity: match diagnostic.severity {
                ArtifactDiagnosticSeverity::Warning => NUX_DIAGNOSTIC_SEVERITY_WARNING,
            },
            code: diagnostic.code.to_owned(),
            message: diagnostic.message,
        })
        .collect::<Vec<_>>();
    for asset in validated.external_assets {
        let Some(bytes) = asset.bytes else {
            continue;
        };
        let kind_label = match asset.kind {
            ExternalAssetKind::Image => "image",
            ExternalAssetKind::Font => "font",
        };
        let attachment: Result<(), String> = match asset.kind {
            ExternalAssetKind::Image => WgpuFactory::validate_image_bytes(&bytes)
                .map_err(|error| error.to_string())
                .and_then(|()| {
                    file.attach_external_image_asset_bytes(asset.asset_id, bytes)
                        .map_err(|error| error.to_string())
                }),
            ExternalAssetKind::Font => file
                .attach_external_font_asset_bytes(asset.asset_id, bytes)
                .map_err(|error| error.to_string()),
        };
        if let Err(error) = attachment {
            if !asset.required {
                diagnostics.push(RuntimeImportDiagnostic {
                    severity: NUX_DIAGNOSTIC_SEVERITY_WARNING,
                    code: "artifact.asset.optional_invalid".to_owned(),
                    message: format!(
                        "optional {kind_label} asset {} '{}' could not be decoded or attached: {error}",
                        asset.asset_id, asset.unique_name
                    ),
                });
                continue;
            }
            return Err(WorkerStartError::Import {
                code: "artifact.asset.attach_failed".to_owned(),
                message: format!(
                    "asset {} '{}' could not be attached: {error}",
                    asset.asset_id, asset.unique_name
                ),
            });
        }
    }
    let (authorization, authenticated_key_id) = match validated.authorization {
        ArtifactAuthorization::Authenticated { key_id } => {
            (NUX_SCRIPT_AUTHORIZATION_AUTHENTICATED, Some(key_id))
        }
        ArtifactAuthorization::VisualOnly { .. } => (NUX_SCRIPT_AUTHORIZATION_VISUAL_ONLY, None),
    };
    Ok((
        file,
        RuntimeImportMetadata {
            authorization,
            authenticated_key_id,
            diagnostics,
        },
    ))
}

#[cfg(feature = "apple-product")]
#[derive(Debug)]
struct RuntimeFailure {
    status: NuxStatus,
    diagnostic_code: &'static [u8],
    diagnostic: String,
}

#[cfg(feature = "apple-product")]
impl RuntimeFailure {
    fn new(status: NuxStatus, diagnostic: impl Into<String>) -> Self {
        Self::with_code(status, diagnostic_code_for_status(status), diagnostic)
    }

    fn with_code(
        status: NuxStatus,
        diagnostic_code: &'static [u8],
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            status,
            diagnostic_code,
            diagnostic: diagnostic.into(),
        }
    }

    fn flow_session(kind: FlowSessionErrorKind, diagnostic: impl Into<String>) -> Self {
        let (status, diagnostic_code) = match kind {
            FlowSessionErrorKind::NotFound => (
                NuxStatus::NotFound,
                diagnostic_code_for_status(NuxStatus::NotFound),
            ),
            FlowSessionErrorKind::InvalidArgument
            | FlowSessionErrorKind::LimitExceeded
            | FlowSessionErrorKind::Conflict => (
                NuxStatus::InvalidArgument,
                diagnostic_code_for_status(NuxStatus::InvalidArgument),
            ),
            FlowSessionErrorKind::ResultLimitExceeded => {
                (NuxStatus::RuntimeError, RESULT_LIMIT_DIAGNOSTIC_CODE)
            }
            FlowSessionErrorKind::ScriptResourceExceeded => {
                (NuxStatus::RuntimeError, SCRIPT_RESOURCE_DIAGNOSTIC_CODE)
            }
            FlowSessionErrorKind::Runtime => (
                NuxStatus::RuntimeError,
                diagnostic_code_for_status(NuxStatus::RuntimeError),
            ),
        };
        Self::with_code(status, diagnostic_code, diagnostic)
    }

    fn runtime(diagnostic: impl Into<String>) -> Self {
        Self::new(NuxStatus::RuntimeError, diagnostic)
    }

    fn surface(diagnostic: impl Into<String>) -> Self {
        Self::new(NuxStatus::SurfaceError, diagnostic)
    }
}

#[cfg(feature = "apple-product")]
fn runtime_failure_from_flow_session(
    error: nuxie::flow_session::FlowSessionError,
) -> RuntimeFailure {
    RuntimeFailure::flow_session(error.kind(), error.message())
}

#[cfg(feature = "apple-product")]
impl WorkerState {
    // Script-enabled Files are intentionally confined to this worker thread.
    // `Arc` provides same-thread shared ownership to its sessions; neither the
    // File nor its Luau VM crosses the worker boundary.
    #[allow(clippy::arc_with_non_send_sync)]
    fn new(file: File) -> Self {
        Self {
            owner_thread_id: thread::current().id(),
            file: Arc::new(file),
            shared_gpu_factory: None,
            gpu_generation: 0,
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
        let mut factory = self.make_session_factory()?;
        let renderer_generation = self.gpu_generation;
        let (flow_session, _) = FlowSession::create_with_factory(
            Arc::clone(&self.file),
            FlowSessionConfig {
                artboard_name,
                player: state_machine_name.map(FlowPlayerSelector::StateMachine),
            },
            factory.as_mut(),
        )
        .map_err(runtime_failure_from_flow_session)?;
        let id = self.allocate_session_id()?;
        self.sessions.insert(
            id,
            SessionState {
                is_fatal: false,
                fatal_diagnostic: None,
                flow_session,
                factory,
                renderer_generation,
                legacy_timestamp_seconds: 0.0,
                #[cfg(test)]
                render_attempts: 0,
                #[cfg(test)]
                injected_device_loss: false,
                #[cfg(test)]
                panic_on_next_configured_operation: false,
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
        let session = self.session(id)?;
        if session.is_fatal {
            Err(RuntimeFailure::runtime(
                session
                    .fatal_diagnostic
                    .as_deref()
                    .unwrap_or(PANIC_DIAGNOSTIC),
            ))
        } else {
            Ok(())
        }
    }

    fn make_session_factory(&mut self) -> Result<Box<WgpuFactory>, RuntimeFailure> {
        if self.shared_gpu_factory.is_none() {
            let factory = WgpuFactory::new_with_mode(1, 1, RenderMode::Msaa)
                .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
            let generation = self
                .gpu_generation
                .checked_add(1)
                .ok_or_else(|| RuntimeFailure::surface("GPU generation space is exhausted"))?;
            self.shared_gpu_factory = Some(factory);
            self.gpu_generation = generation;
        }
        let Some(factory) = self.shared_gpu_factory.as_ref() else {
            return Err(RuntimeFailure::surface(
                "shared GPU factory initialization produced no factory",
            ));
        };
        let factory = factory
            .new_session_factory(1, 1, RenderMode::Msaa)
            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
        Ok(Box::new(factory))
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
        let id = self.allocate_surface_id()?;
        let session = self.session_mut(session_id)?;
        let surface = AppleSurface::attach(&mut session.factory, width, height)
            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
        self.session_mut(session_id)?.attachment = Some(SurfaceState { id, surface });
        Ok(id)
    }

    fn session_surface_mut(
        &mut self,
        session_id: SessionId,
        surface_id: SurfaceId,
    ) -> Result<(&mut WgpuFactory, &mut SurfaceState), RuntimeFailure> {
        self.require_live_session(session_id)?;
        let session = self.session_mut(session_id)?;
        let attachment = session
            .attachment
            .as_mut()
            .filter(|attachment| attachment.id == surface_id)
            .ok_or_else(|| RuntimeFailure::surface("surface is detached"))?;
        Ok((session.factory.as_mut(), attachment))
    }

    fn reattach_surface(
        &mut self,
        session_id: SessionId,
        surface_id: SurfaceId,
        width: u32,
        height: u32,
    ) -> Result<SurfaceDisposition, RuntimeFailure> {
        self.require_live_session(session_id)?;
        let session = self.session(session_id)?;
        if session
            .attachment
            .as_ref()
            .is_none_or(|attachment| attachment.id != surface_id)
        {
            return Err(RuntimeFailure::surface("surface is detached"));
        }
        if !session.requires_device_recovery() {
            let (factory, attachment) = self.session_surface_mut(session_id, surface_id)?;
            return attachment
                .surface
                .reattach(factory, width, height)
                .map_err(|error| RuntimeFailure::surface(format!("{error:#}")));
        }

        // A real device-loss notification is shared by the base factory and
        // all derived session factories. The test-only loss seam is scoped to
        // one session, but still forces the same base-domain replacement so it
        // proves the production transaction without exposing a fault control.
        #[cfg(test)]
        let force_base_replacement = session.injected_device_loss;
        #[cfg(not(test))]
        let force_base_replacement = false;
        let replace_base = force_base_replacement
            || self
                .shared_gpu_factory
                .as_ref()
                .is_none_or(WgpuFactory::device_is_lost);
        let candidate_base = if replace_base {
            Some(
                WgpuFactory::new_with_mode(1, 1, RenderMode::Msaa)
                    .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?,
            )
        } else {
            None
        };
        let base = candidate_base
            .as_ref()
            .or(self.shared_gpu_factory.as_ref())
            .ok_or_else(|| {
                RuntimeFailure::surface("shared GPU factory recovery produced no factory")
            })?;
        let mut candidate_factory = base
            .new_session_factory(width.max(1), height.max(1), RenderMode::Msaa)
            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
        let candidate_surface = AppleSurface::attach(&mut candidate_factory, width, height)
            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
        let candidate_generation = if candidate_base.is_some() {
            self.gpu_generation
                .checked_add(1)
                .ok_or_else(|| RuntimeFailure::surface("GPU generation space is exhausted"))?
        } else {
            self.gpu_generation
        };

        // Commit only after the complete replacement graph exists. Assigning
        // through the Box keeps the exact live Factory address that scripts
        // bind as their renderer domain while dropping every old GPU handle.
        let WorkerState {
            shared_gpu_factory,
            gpu_generation,
            sessions,
            ..
        } = self;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| RuntimeFailure::runtime("render session is unavailable"))?;
        let attachment = session
            .attachment
            .as_mut()
            .filter(|attachment| attachment.id == surface_id)
            .ok_or_else(|| RuntimeFailure::surface("surface is detached"))?;
        if let Some(candidate_base) = candidate_base {
            *shared_gpu_factory = Some(candidate_base);
            *gpu_generation = candidate_generation;
        }
        *session.factory = candidate_factory;
        session.renderer_generation = candidate_generation;
        session.flow_session.reset_renderer();
        attachment.surface = candidate_surface;
        #[cfg(test)]
        {
            session.injected_device_loss = false;
        }
        Ok(if width == 0 || height == 0 {
            SurfaceDisposition::SkippedZeroSize
        } else {
            SurfaceDisposition::Recreated
        })
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
            session.terminalize(PANIC_DIAGNOSTIC);
        }
    }
}

#[cfg(feature = "apple-product")]
fn centered_contain_transform(
    artboard_x: f32,
    artboard_y: f32,
    artboard_width: f32,
    artboard_height: f32,
    viewport_width: u32,
    viewport_height: u32,
) -> Result<Mat2D, RuntimeFailure> {
    if !artboard_x.is_finite()
        || !artboard_y.is_finite()
        || !artboard_width.is_finite()
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
    let offset_x = (viewport_width - artboard_width * scale) * 0.5 - artboard_x * scale;
    let offset_y = (viewport_height - artboard_height * scale) * 0.5 - artboard_y * scale;
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
    #[cfg(test)]
    fn spawn(artifact_bytes: Vec<u8>) -> Result<Arc<Self>, WorkerStartError> {
        use sha2::{Digest as _, Sha256};

        let artifact_sha256 = Sha256::digest(&artifact_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "flowId": "test-flow",
            "buildId": "test-build",
            "renderer": "rive",
            "riv": {
                "path": "flow.riv",
                "sha256": artifact_sha256,
                "sizeBytes": artifact_bytes.len(),
            },
            "assets": {
                "images": [],
                "fonts": [],
            },
        }))
        .map_err(|error| WorkerStartError::Runtime(error.to_string()))?;
        Self::spawn_input(FlowArtifactImportInput {
            expected_flow_id: "test-flow".to_owned(),
            expected_build_id: "test-build".to_owned(),
            artifact_bytes,
            manifest_bytes,
            signature_envelope_bytes: None,
            selected_key: None,
            external_assets: Vec::new(),
        })
        .map(|(worker, _)| worker)
    }

    fn spawn_input(
        input: FlowArtifactImportInput,
    ) -> Result<(Arc<Self>, RuntimeImportMetadata), WorkerStartError> {
        let (sender, receiver) = mpsc::channel();
        let (initialization_sender, initialization_receiver) = mpsc::sync_channel(1);
        let join_handle = thread::Builder::new()
            .name("nuxie-flow-runtime".to_owned())
            .spawn(move || {
                let state = panic::catch_unwind(AssertUnwindSafe(|| {
                    import_runtime_input(input)
                        .map(|(file, metadata)| (WorkerState::new(file), metadata))
                }));
                let (state, metadata) = match state {
                    Ok(Ok(initialized)) => initialized,
                    Ok(Err(error)) => {
                        let _ = initialization_sender.send(Err(error));
                        return;
                    }
                    Err(_) => {
                        let _ = initialization_sender.send(Err(WorkerStartError::Panicked));
                        return;
                    }
                };
                let _ = initialization_sender.send(Ok(WorkerInitialization {
                    thread_id: thread::current().id(),
                    metadata,
                }));
                worker_loop(state, receiver);
            })
            .map_err(|error| WorkerStartError::Runtime(error.to_string()))?;

        let initialization = initialization_receiver.recv().map_err(|_| {
            WorkerStartError::Runtime("runtime worker stopped during initialization".to_owned())
        });
        let initialization = match initialization {
            Ok(Ok(initialization)) => initialization,
            Ok(Err(error)) => {
                let _ = join_handle.join();
                return Err(error);
            }
            Err(error) => {
                let _ = join_handle.join();
                return Err(error);
            }
        };
        Ok((
            Arc::new(Self {
                sender,
                join_handle: Mutex::new(Some(join_handle)),
                thread_id: initialization.thread_id,
            }),
            initialization.metadata,
        ))
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

#[derive(Clone)]
struct OwnedDiagnostic {
    severity: NuxDiagnosticSeverity,
    code: Vec<u8>,
    message: Vec<u8>,
}

pub struct NuxOperationResult {
    status: NuxStatus,
    surface_disposition: NuxSurfaceDisposition,
    changed: bool,
    script_authorization: NuxScriptAuthorization,
    authenticated_key_id: Vec<u8>,
    diagnostics: Vec<OwnedDiagnostic>,
    // Scalar compatibility view: the first structured diagnostic message.
    diagnostic: Vec<u8>,
}

impl NuxOperationResult {
    fn success(surface_disposition: NuxSurfaceDisposition, changed: bool) -> Self {
        Self {
            status: NuxStatus::Ok,
            surface_disposition,
            changed,
            script_authorization: NUX_SCRIPT_AUTHORIZATION_NOT_APPLICABLE,
            authenticated_key_id: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic: Vec::new(),
        }
    }

    #[cfg(feature = "apple-product")]
    fn import_success(metadata: RuntimeImportMetadata) -> Self {
        let diagnostics = metadata
            .diagnostics
            .into_iter()
            .map(|diagnostic| OwnedDiagnostic {
                severity: diagnostic.severity,
                code: diagnostic.code.into_bytes(),
                message: diagnostic.message.into_bytes(),
            })
            .collect::<Vec<_>>();
        let diagnostic = diagnostics
            .first()
            .map(|diagnostic| diagnostic.message.clone())
            .unwrap_or_default();
        Self {
            status: NuxStatus::Ok,
            surface_disposition: NuxSurfaceDisposition::None,
            changed: false,
            script_authorization: metadata.authorization,
            authenticated_key_id: metadata
                .authenticated_key_id
                .map(String::into_bytes)
                .unwrap_or_default(),
            diagnostics,
            diagnostic,
        }
    }

    fn failure(status: NuxStatus, diagnostic: impl Into<Vec<u8>>) -> Self {
        Self::failure_with_code(status, diagnostic_code_for_status(status), diagnostic)
    }

    fn failure_with_code(
        status: NuxStatus,
        code: impl Into<Vec<u8>>,
        diagnostic: impl Into<Vec<u8>>,
    ) -> Self {
        let diagnostic = diagnostic.into();
        Self {
            status,
            surface_disposition: NuxSurfaceDisposition::Fatal,
            changed: false,
            script_authorization: NUX_SCRIPT_AUTHORIZATION_NOT_APPLICABLE,
            authenticated_key_id: Vec::new(),
            diagnostics: vec![OwnedDiagnostic {
                severity: NUX_DIAGNOSTIC_SEVERITY_FATAL,
                code: code.into(),
                message: diagnostic.clone(),
            }],
            diagnostic,
        }
    }

    fn import_failure(
        status: NuxStatus,
        code: impl Into<Vec<u8>>,
        message: impl Into<Vec<u8>>,
    ) -> Self {
        let message = message.into();
        Self {
            status,
            surface_disposition: NuxSurfaceDisposition::Fatal,
            changed: false,
            script_authorization: NUX_SCRIPT_AUTHORIZATION_NOT_APPLICABLE,
            authenticated_key_id: Vec::new(),
            diagnostics: vec![OwnedDiagnostic {
                severity: NUX_DIAGNOSTIC_SEVERITY_FATAL,
                code: code.into(),
                message: message.clone(),
            }],
            diagnostic: message,
        }
    }
}

fn diagnostic_code_for_status(status: NuxStatus) -> &'static [u8] {
    match status {
        NUX_STATUS_OK => b"nux_runtime.ok",
        NUX_STATUS_NULL_ARGUMENT => b"nux_runtime.null_argument",
        NUX_STATUS_IMPORT_ERROR => b"nux_runtime.import_error",
        NUX_STATUS_NOT_FOUND => b"nux_runtime.not_found",
        NUX_STATUS_RUNTIME_ERROR => b"nux_runtime.runtime_error",
        NUX_STATUS_INVALID_ARGUMENT => b"nux_runtime.invalid_argument",
        NUX_STATUS_ABI_MISMATCH => b"nux_runtime.abi_mismatch",
        NUX_STATUS_SURFACE_ERROR => b"nux_runtime.surface_error",
        _ => b"nux_runtime.unknown",
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
/// Checks whether this runtime supports the requested full import contract.
/// ABI 1.0's manifest-free import prefix is intentionally unsupported.
pub extern "C" fn nux_runtime_require_abi(required_major: u16, minimum_minor: u16) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if required_major == NUX_RUNTIME_ABI_MAJOR
            && (MINIMUM_SUPPORTED_ABI_MINOR..=NUX_RUNTIME_ABI_MINOR).contains(&minimum_minor)
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
/// Imports one verified visual artifact into a retained runtime context.
/// The request must provide the full ABI 1.1 import layout; the former ABI 1.0
/// artifact-only prefix is rejected.
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
            let input = match unsafe { copy_runtime_import_input(request) } {
                Ok(input) => input,
                Err(status) => {
                    return write_import_failure(
                        out_result,
                        status,
                        "artifact.request.invalid",
                        "flow import request contains an invalid or oversized view",
                    );
                }
            };
            match RuntimeWorker::spawn_input(input) {
                Ok((worker, metadata)) => {
                    let context = Box::new(FlowRuntimeContextHandle { worker });
                    unsafe {
                        *out_context = Box::into_raw(context).cast();
                    }
                    replace_result(out_result, NuxOperationResult::import_success(metadata));
                    NuxStatus::Ok
                }
                Err(WorkerStartError::Import { code, message }) => {
                    write_import_failure(out_result, NuxStatus::ImportError, code, message)
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
/// Creates an independent logical screen session from a context through the
/// legacy ABI 1.1 surface. Cycle-zero host outputs produced while scripts are
/// initialized are intentionally not returned by this entry point; use
/// `nux_flow_render_session_create_configured` when those outputs are needed.
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
                        let (factory, attachment) =
                            state.session_surface_mut(session_id, surface_id)?;
                        attachment
                            .surface
                            .copy_metal_device(factory)
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
                    let (factory, attachment) =
                        state.session_surface_mut(session_id, surface_id)?;
                    attachment
                        .surface
                        .resize(factory, pixel_width, pixel_height)
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
                    let (_, attachment) = state.session_surface_mut(session_id, surface_id)?;
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
/// Reattaches logical presentation state after a detach. If the session's GPU
/// domain reported device loss, this call transactionally replaces the
/// session's renderer and presentation resources, refreshing the shared base
/// device when needed while preserving logical flow state and factory address.
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
                    state.reattach_surface(session_id, surface_id, pixel_width, pixel_height)
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
                let timestamp_seconds =
                    session.legacy_timestamp_seconds + f64::from(elapsed_seconds);
                if !timestamp_seconds.is_finite() {
                    return Err(RuntimeFailure::runtime("legacy timestamp overflowed"));
                }
                let preflight_disposition = if render {
                    session.preflight_present(drawable_identity != 0)?
                } else {
                    None
                };
                if matches!(preflight_disposition, Some(SurfaceDisposition::DeviceLost)) {
                    return Ok((NUX_SURFACE_DISPOSITION_DEVICE_LOST, false));
                }
                let mut result = session
                    .flow_session
                    .perform_with_factory(
                        nuxie::flow_session::FlowOperation::Advance(
                            nuxie::flow_session::FlowAdvance {
                                timestamp_seconds,
                                delta_seconds: elapsed_seconds,
                                render,
                            },
                        ),
                        session.factory.as_mut(),
                    )
                    .map_err(runtime_failure_from_flow_session)?;
                session.legacy_timestamp_seconds = timestamp_seconds;
                let changed = result.dirty;
                if !render {
                    return Ok((NuxSurfaceDisposition::None, changed));
                }
                if let Some(disposition) = preflight_disposition {
                    return Ok((surface_disposition(disposition), changed));
                }
                let Some((viewport_width, viewport_height)) = session
                    .attachment
                    .as_ref()
                    .map(|attachment| attachment.surface.dimensions())
                else {
                    let failure = RuntimeFailure::runtime("preflighted surface became unavailable");
                    return Err(terminalize_after_committed_advance_failure(
                        session,
                        "presentation setup",
                        failure,
                    ));
                };
                let bounds = session.flow_session.artboard_bounds();
                let presentation_transform = match centered_contain_transform(
                    bounds.x,
                    bounds.y,
                    bounds.width,
                    bounds.height,
                    viewport_width,
                    viewport_height,
                ) {
                    Ok(transform) => transform,
                    Err(failure) => {
                        return Err(terminalize_after_committed_advance_failure(
                            session,
                            "presentation transform",
                            failure,
                        ));
                    }
                };
                let mut frame = session.factory.begin_frame(0x0000_0000);
                frame.transform(presentation_transform);
                #[cfg(test)]
                {
                    session.render_attempts = session.render_attempts.saturating_add(1);
                }
                let draw_result = session.flow_session.draw_into_result(
                    session.factory.as_mut(),
                    &mut frame,
                    &mut result,
                );
                if let Err(error) = draw_result {
                    let failure = runtime_failure_from_flow_session(error);
                    return Err(terminalize_after_committed_advance_failure(
                        session, "drawing", failure,
                    ));
                }
                let drawable = ptr::with_exposed_provenance_mut::<c_void>(drawable_identity);
                let completion = completion.into_renderer_completion();
                let presentation = {
                    let Some(attachment) = session.attachment.as_mut() else {
                        let failure =
                            RuntimeFailure::runtime("preflighted surface became unavailable");
                        return Err(terminalize_after_committed_advance_failure(
                            session,
                            "presentation setup",
                            failure,
                        ));
                    };
                    unsafe {
                        attachment.surface.present(
                            &mut session.factory,
                            frame,
                            drawable,
                            completion,
                        )
                    }
                };
                let (disposition, _metrics) = match presentation {
                    Ok(presentation) => presentation,
                    Err(error) => {
                        let failure = RuntimeFailure::surface(format!("{error:#}"));
                        return Err(terminalize_after_committed_advance_failure(
                            session,
                            "presentation",
                            failure,
                        ));
                    }
                };
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
/// Returns the artifact import's script authorization, or `NOT_APPLICABLE`.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_operation_result_script_authorization(
    result: *const NuxOperationResult,
) -> NuxScriptAuthorization {
    ffi_guard(NUX_SCRIPT_AUTHORIZATION_NOT_APPLICABLE, || {
        if result.is_null() {
            NUX_SCRIPT_AUTHORIZATION_NOT_APPLICABLE
        } else {
            unsafe { (*result).script_authorization }
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows the authenticated key ID stored by an import result.
///
/// # Safety
///
/// `result` must be live and `out_key_id` writable. The returned view expires
/// when `result` is released.
pub unsafe extern "C" fn nux_operation_result_authenticated_key_id(
    result: *const NuxOperationResult,
    out_key_id: *mut NuxByteView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_key_id.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_key_id = NuxByteView::default();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let key_id = unsafe { &(*result).authenticated_key_id };
        if key_id.is_empty() {
            return NuxStatus::NotFound;
        }
        unsafe {
            *out_key_id = NuxByteView {
                data: key_id.as_ptr(),
                len: u64::try_from(key_id.len()).unwrap_or(u64::MAX),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of phase-ordered structured diagnostics in a result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_operation_result_diagnostic_count(
    result: *const NuxOperationResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(unsafe { (*result).diagnostics.len() }).unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one structured diagnostic by stable result order.
///
/// # Safety
///
/// `result` must be live and `out_diagnostic` writable with `struct_size`
/// initialized to the exact ABI-major-1 layout size. Returned views expire when
/// `result` is released.
pub unsafe extern "C" fn nux_operation_result_diagnostic_at(
    result: *const NuxOperationResult,
    index: u64,
    out_diagnostic: *mut NuxDiagnosticView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_diagnostic.is_null() {
            return NuxStatus::NullArgument;
        }
        let caller_struct_size = unsafe { read_struct_size(out_diagnostic) };
        if caller_struct_size != size_u32::<NuxDiagnosticView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            *out_diagnostic = NuxDiagnosticView::default();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let Some(diagnostic) = (unsafe { &(*result).diagnostics }).get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_diagnostic = NuxDiagnosticView {
                struct_size: size_u32::<NuxDiagnosticView>(),
                severity: diagnostic.severity,
                code: NuxByteView {
                    data: diagnostic.code.as_ptr(),
                    len: u64::try_from(diagnostic.code.len()).unwrap_or(u64::MAX),
                },
                message: NuxByteView {
                    data: diagnostic.message.as_ptr(),
                    len: u64::try_from(diagnostic.message.len()).unwrap_or(u64::MAX),
                },
            };
        }
        NuxStatus::Ok
    })
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

#[cfg(feature = "apple-product")]
fn required_utf8_string(view: NuxByteView, maximum_length: usize) -> Result<String, NuxStatus> {
    let bytes = byte_vec(view, maximum_length)?;
    if bytes.is_empty() {
        return Err(NuxStatus::InvalidArgument);
    }
    String::from_utf8(bytes).map_err(|_| NuxStatus::InvalidArgument)
}

#[cfg(feature = "apple-product")]
fn optional_byte_vec(
    view: NuxByteView,
    maximum_length: usize,
) -> Result<Option<Vec<u8>>, NuxStatus> {
    if view.data.is_null() && view.len == 0 {
        return Ok(None);
    }
    byte_vec(view, maximum_length).map(Some)
}

#[cfg(feature = "apple-product")]
unsafe fn copy_runtime_import_input(
    request: *const NuxFlowImportRequest,
) -> Result<FlowArtifactImportInput, NuxStatus> {
    let struct_size = unsafe { read_struct_size(request) };
    if struct_size < size_u32::<NuxFlowImportRequest>() {
        return Err(NuxStatus::InvalidArgument);
    }

    let request = unsafe { request.read() };
    let artifact_bytes = byte_vec(request.artifact_bytes, MAX_ARTIFACT_BYTE_LENGTH)?;
    if request.manifest_bytes.len == 0 {
        return Err(NuxStatus::InvalidArgument);
    }
    let expected_flow_id =
        required_utf8_string(request.expected_flow_id, MAX_SELECTOR_BYTE_LENGTH)?;
    let expected_build_id =
        required_utf8_string(request.expected_build_id, MAX_SELECTOR_BYTE_LENGTH)?;
    let manifest_bytes = byte_vec(request.manifest_bytes, MAX_MANIFEST_BYTE_LENGTH)?;
    let signature_envelope_bytes =
        optional_byte_vec(request.signature_envelope_bytes, MAX_SIGNATURE_BYTE_LENGTH)?;
    let selected_key = if request.selected_key.is_null() {
        None
    } else {
        let struct_size = unsafe { read_struct_size(request.selected_key) };
        if struct_size < size_u32::<NuxFlowAuthorizationKey>() {
            return Err(NuxStatus::InvalidArgument);
        }
        let selected_key = unsafe { request.selected_key.read() };
        let key_id =
            required_utf8_string(selected_key.key_id, MAX_AUTHORIZATION_KEY_ID_BYTE_LENGTH)?;
        let public_key = byte_vec(
            selected_key.ed25519_public_key,
            ED25519_PUBLIC_KEY_BYTE_LENGTH,
        )?;
        let public_key: [u8; ED25519_PUBLIC_KEY_BYTE_LENGTH] = public_key
            .try_into()
            .map_err(|_| NuxStatus::InvalidArgument)?;
        Some(SelectedArtifactSigningKey { key_id, public_key })
    };
    let external_asset_count =
        usize::try_from(request.external_asset_count).map_err(|_| NuxStatus::InvalidArgument)?;
    if external_asset_count > MAX_EXTERNAL_ASSET_COUNT
        || (external_asset_count != 0 && request.external_assets.is_null())
    {
        return Err(NuxStatus::InvalidArgument);
    }
    let external_asset_array_size = external_asset_count
        .checked_mul(std::mem::size_of::<NuxFlowExternalAsset>())
        .ok_or(NuxStatus::InvalidArgument)?;
    if external_asset_array_size > isize::MAX as usize {
        return Err(NuxStatus::InvalidArgument);
    }
    let external_asset_views = if external_asset_count == 0 {
        &[][..]
    } else {
        // SAFETY: the caller promises an array of `external_asset_count`
        // readable elements for this synchronous call. Every nested view is
        // copied below before the runtime worker can retain the import.
        unsafe { slice::from_raw_parts(request.external_assets, external_asset_count) }
    };
    let mut external_assets = Vec::with_capacity(external_asset_count);
    let mut cumulative_asset_bytes = 0usize;
    for asset in external_asset_views {
        // Array elements have no separate stride parameter. Accepting a larger
        // element declaration would make the second element start ambiguous,
        // so this ABI revision requires the exact published element size.
        if asset.struct_size != size_u32::<NuxFlowExternalAsset>() {
            return Err(NuxStatus::InvalidArgument);
        }
        let kind = match asset.kind {
            NUX_FLOW_EXTERNAL_ASSET_KIND_IMAGE => ExternalAssetKind::Image,
            NUX_FLOW_EXTERNAL_ASSET_KIND_FONT => ExternalAssetKind::Font,
            _ => return Err(NuxStatus::InvalidArgument),
        };
        let unique_name = required_utf8_string(asset.unique_name, MAX_SELECTOR_BYTE_LENGTH)?;
        let source_key = required_utf8_string(asset.source_key, MAX_ASSET_SOURCE_KEY_BYTE_LENGTH)?;
        let expected_sha256 =
            required_utf8_string(asset.expected_sha256, MAX_SELECTOR_BYTE_LENGTH)?;
        let input = if asset.provided {
            let bytes = byte_vec(asset.bytes, MAX_EXTERNAL_ASSET_TOTAL_BYTE_LENGTH)?;
            cumulative_asset_bytes = cumulative_asset_bytes
                .checked_add(bytes.len())
                .ok_or(NuxStatus::InvalidArgument)?;
            if cumulative_asset_bytes > MAX_EXTERNAL_ASSET_TOTAL_BYTE_LENGTH {
                return Err(NuxStatus::InvalidArgument);
            }
            ExternalAssetInput::Supplied {
                kind,
                asset_id: asset.asset_id,
                unique_name,
                source_key,
                expected_sha256,
                required: asset.required,
                bytes,
            }
        } else {
            if !asset.bytes.data.is_null() || asset.bytes.len != 0 {
                return Err(NuxStatus::InvalidArgument);
            }
            ExternalAssetInput::Omitted {
                kind,
                asset_id: asset.asset_id,
                unique_name,
                source_key,
                expected_sha256,
                required: asset.required,
            }
        };
        external_assets.push(input);
    }

    Ok(FlowArtifactImportInput {
        expected_flow_id,
        expected_build_id,
        artifact_bytes,
        manifest_bytes,
        signature_envelope_bytes,
        selected_key,
        external_assets,
    })
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

fn write_failure_with_code(
    out_result: *mut *mut NuxOperationResult,
    status: NuxStatus,
    code: impl Into<Vec<u8>>,
    diagnostic: impl Into<Vec<u8>>,
) -> NuxStatus {
    replace_result(
        out_result,
        NuxOperationResult::failure_with_code(status, code, diagnostic),
    );
    status
}

fn write_import_failure(
    out_result: *mut *mut NuxOperationResult,
    status: NuxStatus,
    code: impl Into<Vec<u8>>,
    message: impl Into<Vec<u8>>,
) -> NuxStatus {
    replace_result(
        out_result,
        NuxOperationResult::import_failure(status, code, message),
    );
    status
}

#[cfg(feature = "apple-product")]
fn write_runtime_failure(
    out_result: *mut *mut NuxOperationResult,
    failure: RuntimeFailure,
) -> NuxStatus {
    write_failure_with_code(
        out_result,
        failure.status,
        failure.diagnostic_code,
        failure.diagnostic,
    )
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
    fn current_import_request_without_manifest(bytes: &[u8]) -> NuxFlowImportRequest {
        NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len() as u64,
            },
            expected_flow_id: NuxByteView::default(),
            expected_build_id: NuxByteView::default(),
            manifest_bytes: NuxByteView::default(),
            signature_envelope_bytes: NuxByteView::default(),
            selected_key: ptr::null(),
            external_assets: ptr::null(),
            external_asset_count: 0,
        }
    }

    #[cfg(feature = "apple-product")]
    #[repr(C)]
    struct AbiOneZeroImportPrefix {
        struct_size: u32,
        artifact_bytes: NuxByteView,
    }

    #[cfg(feature = "apple-product")]
    struct UnsignedImportRequest {
        request: NuxFlowImportRequest,
        _manifest_bytes: Vec<u8>,
    }

    #[cfg(feature = "apple-product")]
    fn unsigned_import_request(bytes: &[u8]) -> UnsignedImportRequest {
        use sha2::{Digest as _, Sha256};

        let artifact_sha256 = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "flowId": "test-flow",
            "buildId": "test-build",
            "renderer": "rive",
            "riv": {
                "path": "flow.riv",
                "sha256": artifact_sha256,
                "sizeBytes": bytes.len(),
            },
            "assets": {
                "images": [],
                "fonts": [],
            },
        }))
        .expect("test manifest encodes");
        let flow_id = b"test-flow";
        let build_id = b"test-build";
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len() as u64,
            },
            expected_flow_id: NuxByteView {
                data: flow_id.as_ptr(),
                len: flow_id.len() as u64,
            },
            expected_build_id: NuxByteView {
                data: build_id.as_ptr(),
                len: build_id.len() as u64,
            },
            manifest_bytes: NuxByteView {
                data: manifest_bytes.as_ptr(),
                len: manifest_bytes.len() as u64,
            },
            signature_envelope_bytes: NuxByteView::default(),
            selected_key: ptr::null(),
            external_assets: ptr::null(),
            external_asset_count: 0,
        };
        UnsignedImportRequest {
            request,
            _manifest_bytes: manifest_bytes,
        }
    }

    #[cfg(feature = "apple-product")]
    fn push_test_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    #[cfg(feature = "apple-product")]
    fn test_property_key(type_name: &str, property_name: &str) -> u16 {
        let definition = nuxie_schema::definition_by_name(type_name).expect("fixture type exists");
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|ancestor| {
                nuxie_schema::definition_by_name(ancestor)
                    .expect("fixture ancestor exists")
                    .properties
                    .iter()
            }))
            .find(|property| property.name == property_name)
            .expect("fixture property exists")
            .key
            .int
    }

    #[cfg(feature = "apple-product")]
    fn push_test_object(
        bytes: &mut Vec<u8>,
        type_name: &str,
        properties: impl FnOnce(&mut Vec<u8>),
    ) {
        push_test_var_uint(
            bytes,
            u64::from(
                nuxie_schema::definition_by_name(type_name)
                    .expect("fixture type exists")
                    .type_key
                    .int,
            ),
        );
        properties(bytes);
        push_test_var_uint(bytes, 0);
    }

    #[cfg(feature = "apple-product")]
    fn push_test_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
        push_test_var_uint(bytes, u64::from(test_property_key(type_name, name)));
        push_test_var_uint(bytes, value);
    }

    #[cfg(feature = "apple-product")]
    fn push_test_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
        push_test_var_uint(bytes, u64::from(test_property_key(type_name, name)));
        push_test_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value.as_bytes());
    }

    #[cfg(feature = "apple-product")]
    fn external_image_artifact_bytes() -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_test_var_uint(&mut bytes, 7);
        push_test_var_uint(&mut bytes, 0);
        push_test_var_uint(&mut bytes, 992);
        push_test_var_uint(&mut bytes, 0);
        push_test_object(&mut bytes, "Backboard", |_| {});
        push_test_object(&mut bytes, "ImageAsset", |bytes| {
            push_test_uint(bytes, "ImageAsset", "assetId", 1);
            push_test_string(bytes, "ImageAsset", "name", "image.png");
        });
        push_test_object(&mut bytes, "Artboard", |_| {});
        bytes
    }

    #[cfg(feature = "apple-product")]
    fn with_external_image_import_request<R>(
        image_bytes: &[u8],
        required: bool,
        body: impl FnOnce(&NuxFlowImportRequest) -> R,
    ) -> R {
        use sha2::{Digest as _, Sha256};

        let artifact_bytes = external_image_artifact_bytes();
        let artifact_sha256 = Sha256::digest(&artifact_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let image_sha256 = Sha256::digest(image_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "flowId": "flow-image-validation",
            "buildId": "build-image-validation",
            "renderer": "rive",
            "riv": {
                "path": "flow.riv",
                "sha256": artifact_sha256,
                "sizeBytes": artifact_bytes.len(),
            },
            "assets": {
                "images": [{
                    "riveAssetId": 1,
                    "riveUniqueName": "image-1",
                    "sourceAssetKey": "hero",
                    "sha256": image_sha256,
                    "required": required,
                }],
                "fonts": [],
            },
        }))
        .expect("manifest encodes");
        let flow_id = b"flow-image-validation";
        let build_id = b"build-image-validation";
        let unique_name = b"image-1";
        let source_key = b"hero";
        let external_asset = NuxFlowExternalAsset {
            struct_size: size_u32::<NuxFlowExternalAsset>(),
            kind: NUX_FLOW_EXTERNAL_ASSET_KIND_IMAGE,
            asset_id: 1,
            required,
            provided: true,
            unique_name: NuxByteView {
                data: unique_name.as_ptr(),
                len: unique_name.len() as u64,
            },
            source_key: NuxByteView {
                data: source_key.as_ptr(),
                len: source_key.len() as u64,
            },
            expected_sha256: NuxByteView {
                data: image_sha256.as_ptr(),
                len: image_sha256.len() as u64,
            },
            bytes: NuxByteView {
                data: image_bytes.as_ptr(),
                len: image_bytes.len() as u64,
            },
        };
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: artifact_bytes.as_ptr(),
                len: artifact_bytes.len() as u64,
            },
            expected_flow_id: NuxByteView {
                data: flow_id.as_ptr(),
                len: flow_id.len() as u64,
            },
            expected_build_id: NuxByteView {
                data: build_id.as_ptr(),
                len: build_id.len() as u64,
            },
            manifest_bytes: NuxByteView {
                data: manifest_bytes.as_ptr(),
                len: manifest_bytes.len() as u64,
            },
            signature_envelope_bytes: NuxByteView::default(),
            selected_key: ptr::null(),
            external_assets: &external_asset,
            external_asset_count: 1,
        };
        body(&request)
    }

    #[cfg(feature = "apple-product")]
    fn oversized_external_image_bytes() -> Vec<u8> {
        const OVERSIZED_WIDTH: u32 = 8_193;

        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, OVERSIZED_WIDTH, 1);
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .expect("oversized test PNG header encodes")
                .write_image_data(&vec![0; OVERSIZED_WIDTH as usize])
                .expect("oversized test PNG pixels encode");
        }
        assert!(encoded.len() < 256, "test PNG must remain compact");
        encoded
    }

    #[cfg(feature = "apple-product")]
    fn oversized_pixel_budget_image_header_bytes() -> Vec<u8> {
        const OVERSIZED_SQUARE_DIMENSION: u32 = 4_097;

        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(
                &mut encoded,
                OVERSIZED_SQUARE_DIMENSION,
                OVERSIZED_SQUARE_DIMENSION,
            );
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .expect("pixel-budget test PNG header encodes");
            writer
                .write_chunk(png::chunk::IDAT, &[])
                .expect("pixel-budget test PNG writes an empty data chunk");
        }
        assert!(encoded.len() < 128, "test PNG header must remain compact");
        encoded
    }

    #[cfg(feature = "apple-product")]
    fn product_fixture_worker() -> Arc<RuntimeWorker> {
        match RuntimeWorker::spawn(product_fixture_bytes()) {
            Ok(worker) => worker,
            Err(_) => panic!("product fixture must create a runtime worker"),
        }
    }

    #[test]
    fn abi_compatibility_requires_the_full_import_contract() {
        assert_eq!(nux_runtime_require_abi(1, 0), NuxStatus::AbiMismatch);
        assert_eq!(nux_runtime_require_abi(1, 1), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(2, 0), NuxStatus::AbiMismatch);
        assert_eq!(nux_runtime_require_abi(1, 2), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(1, 3), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(1, 4), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(1, 5), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(1, 6), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(1, 7), NuxStatus::AbiMismatch);
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn signed_import_crosses_the_public_c_seam_as_authenticated() {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use ed25519_dalek::{Signer as _, SigningKey};
        use sha2::{Digest as _, Sha256};

        let artifact_bytes = product_fixture_bytes();
        let artifact_sha256 = Sha256::digest(&artifact_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "flowId": "flow-c-abi",
            "buildId": "build-c-abi",
            "renderer": "rive",
            "riv": {
                "path": "flow.riv",
                "sha256": artifact_sha256,
                "sizeBytes": artifact_bytes.len(),
            },
            "assets": { "images": [], "fonts": [] },
        }))
        .expect("manifest encodes");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let signature = signing_key.sign(&manifest_bytes);
        let signature_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "signs": "nuxie-manifest.json",
            "algorithm": "ed25519",
            "keyId": "test-key",
            "signatureBase64": BASE64.encode(signature.to_bytes()),
        }))
        .expect("signature envelope encodes");
        let flow_id = b"flow-c-abi";
        let build_id = b"build-c-abi";
        let key_id = b"test-key";
        let public_key = signing_key.verifying_key().to_bytes();
        let selected_key = NuxFlowAuthorizationKey {
            struct_size: size_u32::<NuxFlowAuthorizationKey>(),
            key_id: NuxByteView {
                data: key_id.as_ptr(),
                len: key_id.len() as u64,
            },
            ed25519_public_key: NuxByteView {
                data: public_key.as_ptr(),
                len: public_key.len() as u64,
            },
        };
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: artifact_bytes.as_ptr(),
                len: artifact_bytes.len() as u64,
            },
            expected_flow_id: NuxByteView {
                data: flow_id.as_ptr(),
                len: flow_id.len() as u64,
            },
            expected_build_id: NuxByteView {
                data: build_id.as_ptr(),
                len: build_id.len() as u64,
            },
            manifest_bytes: NuxByteView {
                data: manifest_bytes.as_ptr(),
                len: manifest_bytes.len() as u64,
            },
            signature_envelope_bytes: NuxByteView {
                data: signature_bytes.as_ptr(),
                len: signature_bytes.len() as u64,
            },
            selected_key: &selected_key,
            external_assets: ptr::null(),
            external_asset_count: 0,
        };
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();

        assert_eq!(
            unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
            NuxStatus::Ok
        );
        assert!(!context.is_null());
        assert_eq!(
            unsafe { nux_operation_result_script_authorization(result) },
            NUX_SCRIPT_AUTHORIZATION_AUTHENTICATED
        );
        assert_eq!(unsafe { nux_operation_result_diagnostic_count(result) }, 0);
        let mut authenticated_key_id = NuxByteView::default();
        assert_eq!(
            unsafe { nux_operation_result_authenticated_key_id(result, &mut authenticated_key_id) },
            NuxStatus::Ok
        );
        let authenticated_key_id = unsafe {
            slice::from_raw_parts(authenticated_key_id.data, authenticated_key_id.len as usize)
        };
        assert_eq!(authenticated_key_id, key_id);

        unsafe {
            nux_operation_result_free(result);
            nux_flow_runtime_context_free(context);
        }

        let signature_sentinel = 0u8;
        let malformed_signature_request = NuxFlowImportRequest {
            signature_envelope_bytes: NuxByteView {
                data: &signature_sentinel,
                len: 0,
            },
            ..request
        };
        context = ptr::null_mut();
        result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_flow_runtime_context_create(
                    &malformed_signature_request,
                    &mut context,
                    &mut result,
                )
            },
            NuxStatus::Ok
        );
        let mut diagnostic = NuxDiagnosticView::default();
        assert_eq!(
            unsafe { nux_operation_result_diagnostic_at(result, 0, &mut diagnostic) },
            NuxStatus::Ok
        );
        let code =
            unsafe { slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize) };
        assert_eq!(code, b"artifact.authentication.malformed");
        unsafe {
            nux_operation_result_free(result);
            nux_flow_runtime_context_free(context);
        }
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn signed_external_asset_views_are_copied_validated_and_attached() {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use ed25519_dalek::{Signer as _, SigningKey};
        use sha2::{Digest as _, Sha256};

        let artifact_bytes = external_image_artifact_bytes();
        let image_bytes = include_bytes!(
            "../../../fixtures/renderer/reference/metal/first-light-triangle-clockwise-atomic.png"
        )
        .as_slice();
        let artifact_sha256 = Sha256::digest(&artifact_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let image_sha256 = Sha256::digest(image_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "flowId": "flow-assets",
            "buildId": "build-assets",
            "renderer": "rive",
            "riv": {
                "path": "flow.riv",
                "sha256": artifact_sha256,
                "sizeBytes": artifact_bytes.len(),
            },
            "assets": {
                "images": [{
                    "riveAssetId": 1,
                    "riveUniqueName": "image-1",
                    "sourceAssetKey": "hero",
                    "sha256": image_sha256,
                    "required": true,
                }],
                "fonts": [],
            },
        }))
        .expect("manifest encodes");
        let signing_key = SigningKey::from_bytes(&[11; 32]);
        let signature = signing_key.sign(&manifest_bytes);
        let signature_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "signs": "nuxie-manifest.json",
            "algorithm": "ed25519",
            "keyId": "test-assets-key",
            "signatureBase64": BASE64.encode(signature.to_bytes()),
        }))
        .expect("signature envelope encodes");
        let flow_id = b"flow-assets";
        let build_id = b"build-assets";
        let key_id = b"test-assets-key";
        let unique_name = b"image-1";
        let source_key = b"hero";
        let public_key = signing_key.verifying_key().to_bytes();
        let selected_key = NuxFlowAuthorizationKey {
            struct_size: size_u32::<NuxFlowAuthorizationKey>(),
            key_id: NuxByteView {
                data: key_id.as_ptr(),
                len: key_id.len() as u64,
            },
            ed25519_public_key: NuxByteView {
                data: public_key.as_ptr(),
                len: public_key.len() as u64,
            },
        };
        let external_asset = NuxFlowExternalAsset {
            struct_size: size_u32::<NuxFlowExternalAsset>(),
            kind: NUX_FLOW_EXTERNAL_ASSET_KIND_IMAGE,
            asset_id: 1,
            required: true,
            provided: true,
            unique_name: NuxByteView {
                data: unique_name.as_ptr(),
                len: unique_name.len() as u64,
            },
            source_key: NuxByteView {
                data: source_key.as_ptr(),
                len: source_key.len() as u64,
            },
            expected_sha256: NuxByteView {
                data: image_sha256.as_ptr(),
                len: image_sha256.len() as u64,
            },
            bytes: NuxByteView {
                data: image_bytes.as_ptr(),
                len: image_bytes.len() as u64,
            },
        };
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: artifact_bytes.as_ptr(),
                len: artifact_bytes.len() as u64,
            },
            expected_flow_id: NuxByteView {
                data: flow_id.as_ptr(),
                len: flow_id.len() as u64,
            },
            expected_build_id: NuxByteView {
                data: build_id.as_ptr(),
                len: build_id.len() as u64,
            },
            manifest_bytes: NuxByteView {
                data: manifest_bytes.as_ptr(),
                len: manifest_bytes.len() as u64,
            },
            signature_envelope_bytes: NuxByteView {
                data: signature_bytes.as_ptr(),
                len: signature_bytes.len() as u64,
            },
            selected_key: &selected_key,
            external_assets: &external_asset,
            external_asset_count: 1,
        };
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();

        assert_eq!(
            unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
            NuxStatus::Ok
        );
        assert!(!context.is_null());
        assert_eq!(
            unsafe { nux_operation_result_script_authorization(result) },
            NUX_SCRIPT_AUTHORIZATION_AUTHENTICATED
        );

        unsafe {
            nux_operation_result_free(result);
            nux_flow_runtime_context_free(context);
        }
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn required_undecodable_external_image_fails_trusted_import() {
        with_external_image_import_request(b"not an encoded image", true, |request| {
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();

            assert_eq!(
                unsafe { nux_flow_runtime_context_create(request, &mut context, &mut result) },
                NuxStatus::ImportError
            );
            assert!(context.is_null());
            assert_eq!(unsafe { nux_operation_result_diagnostic_count(result) }, 1);
            let mut diagnostic = NuxDiagnosticView::default();
            assert_eq!(
                unsafe { nux_operation_result_diagnostic_at(result, 0, &mut diagnostic) },
                NuxStatus::Ok
            );
            let code = unsafe {
                slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize)
            };
            assert_eq!(code, b"artifact.asset.attach_failed");
            unsafe { nux_operation_result_free(result) };
        });
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn optional_undecodable_external_image_is_omitted_with_a_warning() {
        with_external_image_import_request(b"not an encoded image", false, |request| {
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();

            assert_eq!(
                unsafe { nux_flow_runtime_context_create(request, &mut context, &mut result) },
                NuxStatus::Ok
            );
            assert!(!context.is_null());
            let diagnostic_count = unsafe { nux_operation_result_diagnostic_count(result) };
            let mut diagnostic_codes = Vec::new();
            for index in 0..diagnostic_count {
                let mut diagnostic = NuxDiagnosticView::default();
                assert_eq!(
                    unsafe { nux_operation_result_diagnostic_at(result, index, &mut diagnostic) },
                    NuxStatus::Ok
                );
                diagnostic_codes.push(
                    unsafe {
                        slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize)
                    }
                    .to_vec(),
                );
            }
            assert!(
                diagnostic_codes
                    .iter()
                    .any(|code| code == b"artifact.asset.optional_invalid")
            );
            unsafe {
                nux_operation_result_free(result);
                nux_flow_runtime_context_free(context);
            }
        });
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn required_oversized_external_image_fails_trusted_import() {
        let image_bytes = oversized_external_image_bytes();
        with_external_image_import_request(&image_bytes, true, |request| {
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();

            assert_eq!(
                unsafe { nux_flow_runtime_context_create(request, &mut context, &mut result) },
                NuxStatus::ImportError
            );
            assert!(context.is_null());
            let mut diagnostic = NuxDiagnosticView::default();
            assert_eq!(
                unsafe { nux_operation_result_diagnostic_at(result, 0, &mut diagnostic) },
                NuxStatus::Ok
            );
            let code = unsafe {
                slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize)
            };
            assert_eq!(code, b"artifact.asset.attach_failed");
            unsafe { nux_operation_result_free(result) };
        });
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn optional_oversized_external_image_is_omitted_with_a_warning() {
        let image_bytes = oversized_external_image_bytes();
        with_external_image_import_request(&image_bytes, false, |request| {
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();

            assert_eq!(
                unsafe { nux_flow_runtime_context_create(request, &mut context, &mut result) },
                NuxStatus::Ok
            );
            assert!(!context.is_null());
            let diagnostic_count = unsafe { nux_operation_result_diagnostic_count(result) };
            let mut found_optional_invalid = false;
            for index in 0..diagnostic_count {
                let mut diagnostic = NuxDiagnosticView::default();
                assert_eq!(
                    unsafe { nux_operation_result_diagnostic_at(result, index, &mut diagnostic) },
                    NuxStatus::Ok
                );
                let code = unsafe {
                    slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize)
                };
                found_optional_invalid |= code == b"artifact.asset.optional_invalid";
            }
            assert!(found_optional_invalid);
            unsafe {
                nux_operation_result_free(result);
                nux_flow_runtime_context_free(context);
            }
        });
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn required_image_over_decoded_pixel_budget_fails_from_its_compact_header() {
        let image_bytes = oversized_pixel_budget_image_header_bytes();
        with_external_image_import_request(&image_bytes, true, |request| {
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();

            assert_eq!(
                unsafe { nux_flow_runtime_context_create(request, &mut context, &mut result) },
                NuxStatus::ImportError
            );
            assert!(context.is_null());
            let mut diagnostic = NuxDiagnosticView::default();
            assert_eq!(
                unsafe { nux_operation_result_diagnostic_at(result, 0, &mut diagnostic) },
                NuxStatus::Ok
            );
            let code = unsafe {
                slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize)
            };
            assert_eq!(code, b"artifact.asset.attach_failed");
            unsafe { nux_operation_result_free(result) };
        });
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn missing_signature_returns_visual_only_with_a_structured_warning() {
        use sha2::{Digest as _, Sha256};

        let artifact_bytes = product_fixture_bytes();
        let artifact_sha256 = Sha256::digest(&artifact_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "flowId": "flow-unsigned",
            "buildId": "build-unsigned",
            "renderer": "rive",
            "riv": {
                "path": "flow.riv",
                "sha256": artifact_sha256,
                "sizeBytes": artifact_bytes.len(),
            },
            "assets": { "images": [], "fonts": [] },
        }))
        .expect("manifest encodes");
        let flow_id = b"flow-unsigned";
        let build_id = b"build-unsigned";
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: artifact_bytes.as_ptr(),
                len: artifact_bytes.len() as u64,
            },
            expected_flow_id: NuxByteView {
                data: flow_id.as_ptr(),
                len: flow_id.len() as u64,
            },
            expected_build_id: NuxByteView {
                data: build_id.as_ptr(),
                len: build_id.len() as u64,
            },
            manifest_bytes: NuxByteView {
                data: manifest_bytes.as_ptr(),
                len: manifest_bytes.len() as u64,
            },
            signature_envelope_bytes: NuxByteView::default(),
            selected_key: ptr::null(),
            external_assets: ptr::null(),
            external_asset_count: 0,
        };
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();

        assert_eq!(
            unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_operation_result_script_authorization(result) },
            NUX_SCRIPT_AUTHORIZATION_VISUAL_ONLY
        );
        assert_eq!(unsafe { nux_operation_result_diagnostic_count(result) }, 1);
        let mut diagnostic = NuxDiagnosticView::default();
        assert_eq!(
            unsafe { nux_operation_result_diagnostic_at(result, 0, &mut diagnostic) },
            NuxStatus::Ok
        );
        assert_eq!(diagnostic.severity, NUX_DIAGNOSTIC_SEVERITY_WARNING);
        let code =
            unsafe { slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize) };
        assert_eq!(code, b"artifact.authentication.missing");

        unsafe {
            nux_operation_result_free(result);
            nux_flow_runtime_context_free(context);
        }
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn artifact_integrity_failure_returns_a_structured_fatal_diagnostic() {
        let artifact_bytes = product_fixture_bytes();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "flowId": "flow-tampered",
            "buildId": "build-tampered",
            "renderer": "rive",
            "riv": {
                "path": "flow.riv",
                "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                "sizeBytes": artifact_bytes.len(),
            },
            "assets": { "images": [], "fonts": [] },
        }))
        .expect("manifest encodes");
        let flow_id = b"flow-tampered";
        let build_id = b"build-tampered";
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: artifact_bytes.as_ptr(),
                len: artifact_bytes.len() as u64,
            },
            expected_flow_id: NuxByteView {
                data: flow_id.as_ptr(),
                len: flow_id.len() as u64,
            },
            expected_build_id: NuxByteView {
                data: build_id.as_ptr(),
                len: build_id.len() as u64,
            },
            manifest_bytes: NuxByteView {
                data: manifest_bytes.as_ptr(),
                len: manifest_bytes.len() as u64,
            },
            signature_envelope_bytes: NuxByteView::default(),
            selected_key: ptr::null(),
            external_assets: ptr::null(),
            external_asset_count: 0,
        };
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();

        assert_eq!(
            unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
            NuxStatus::ImportError
        );
        assert!(context.is_null());
        assert_eq!(unsafe { nux_operation_result_diagnostic_count(result) }, 1);
        let mut diagnostic = NuxDiagnosticView::default();
        assert_eq!(
            unsafe { nux_operation_result_diagnostic_at(result, 0, &mut diagnostic) },
            NuxStatus::Ok
        );
        assert_eq!(diagnostic.severity, NUX_DIAGNOSTIC_SEVERITY_FATAL);
        let code =
            unsafe { slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize) };
        assert_eq!(code, b"artifact.riv.hash_mismatch");
        unsafe { nux_operation_result_free(result) };
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
            "\"runtimeAbiMajor\":1",
            "\"runtimeAbiMinor\":6",
            "\"flowSessionAbiMinor\":6",
            "\"sourceRevision\"",
            "\"target\"",
            "\"profile\"",
            "\"rustc\"",
            "\"wgpuVersion\":\"30.0.0\"",
        ] {
            assert!(json.contains(field), "missing {field} in {json}");
        }
        let luaur_field = if cfg!(feature = "apple-product") {
            "\"luaurVersion\":\"0.1.8\""
        } else {
            "\"luaurVersion\":null"
        };
        assert!(
            json.contains(luaur_field),
            "missing {luaur_field} in {json}"
        );
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
        assert_eq!(unsafe { nux_operation_result_diagnostic_count(result) }, 1);
        let mut undersized = NuxDiagnosticView {
            struct_size: size_u32::<u32>(),
            ..NuxDiagnosticView::default()
        };
        assert_eq!(
            unsafe { nux_operation_result_diagnostic_at(result, 0, &mut undersized) },
            NuxStatus::InvalidArgument
        );
        let mut structured = NuxDiagnosticView::default();
        assert_eq!(
            unsafe { nux_operation_result_diagnostic_at(result, 0, &mut structured) },
            NuxStatus::Ok
        );
        assert_eq!(structured.severity, NUX_DIAGNOSTIC_SEVERITY_FATAL);
        let code =
            unsafe { slice::from_raw_parts(structured.code.data, structured.code.len as usize) };
        assert_eq!(code, b"nux_runtime.invalid_argument");
        assert_eq!(
            unsafe { nux_operation_result_diagnostic_at(result, 1, &mut structured) },
            NuxStatus::NotFound
        );
        assert!(structured.code.data.is_null());
        assert_eq!(
            unsafe { nux_operation_result_script_authorization(ptr::null()) },
            NUX_SCRIPT_AUTHORIZATION_NOT_APPLICABLE
        );
        unsafe { nux_operation_result_free(result) };
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn flow_session_failure_codes_cross_the_legacy_structured_diagnostic_seam() {
        let cases = [
            (
                FlowSessionErrorKind::ScriptResourceExceeded,
                SCRIPT_RESOURCE_DIAGNOSTIC_CODE,
            ),
            (
                FlowSessionErrorKind::ResultLimitExceeded,
                RESULT_LIMIT_DIAGNOSTIC_CODE,
            ),
            (
                FlowSessionErrorKind::Runtime,
                diagnostic_code_for_status(NuxStatus::RuntimeError),
            ),
        ];

        for (kind, expected_code) in cases {
            let mut result = ptr::null_mut();
            let failure = RuntimeFailure::flow_session(kind, "flow operation failed");
            assert_eq!(
                write_runtime_failure(&mut result, failure),
                NuxStatus::RuntimeError
            );
            assert_eq!(
                unsafe { nux_operation_result_status(result) },
                NuxStatus::RuntimeError
            );
            assert_eq!(unsafe { nux_operation_result_diagnostic_count(result) }, 1);

            let mut diagnostic = NuxDiagnosticView::default();
            assert_eq!(
                unsafe { nux_operation_result_diagnostic_at(result, 0, &mut diagnostic) },
                NuxStatus::Ok
            );
            let code = unsafe {
                slice::from_raw_parts(diagnostic.code.data, diagnostic.code.len as usize)
            };
            assert_eq!(code, expected_code);
            unsafe { nux_operation_result_free(result) };
        }
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

    #[cfg(feature = "apple-product")]
    #[test]
    fn abi_one_zero_import_prefix_is_rejected() {
        let artifact_bytes = product_fixture_bytes();
        let request = AbiOneZeroImportPrefix {
            struct_size: size_u32::<AbiOneZeroImportPrefix>(),
            artifact_bytes: NuxByteView {
                data: artifact_bytes.as_ptr(),
                len: artifact_bytes.len() as u64,
            },
        };
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();

        assert_eq!(
            unsafe {
                nux_flow_runtime_context_create(
                    (&request as *const AbiOneZeroImportPrefix).cast(),
                    &mut context,
                    &mut result,
                )
            },
            NuxStatus::InvalidArgument
        );
        assert!(context.is_null());
        assert!(!result.is_null());
        unsafe {
            nux_operation_result_free(result);
        }
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn current_and_truncated_import_requests_cannot_bypass_manifest_validation() {
        let artifact_bytes = product_fixture_bytes();
        for struct_size in [
            size_u32::<AbiOneZeroImportPrefix>() + 1,
            size_u32::<NuxFlowImportRequest>(),
        ] {
            let mut request = current_import_request_without_manifest(&artifact_bytes);
            request.struct_size = struct_size;
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();

            assert_eq!(
                unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
                NuxStatus::InvalidArgument,
                "request size {struct_size} must not bypass the ABI 1.1 manifest contract"
            );
            assert!(context.is_null());
            assert!(!result.is_null());
            unsafe { nux_operation_result_free(result) };
        }
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn external_asset_arrays_reject_unknown_element_strides() {
        let artifact_bytes = b"RIVE";
        let manifest_bytes = b"{}";
        let flow_id = b"flow";
        let build_id = b"build";
        let unique_name = b"image";
        let source_key = b"hero";
        let expected_sha256 = b"0000000000000000000000000000000000000000000000000000000000000000";
        let external_asset = NuxFlowExternalAsset {
            struct_size: size_u32::<NuxFlowExternalAsset>() + 1,
            kind: NUX_FLOW_EXTERNAL_ASSET_KIND_IMAGE,
            asset_id: 1,
            required: false,
            provided: false,
            unique_name: NuxByteView {
                data: unique_name.as_ptr(),
                len: unique_name.len() as u64,
            },
            source_key: NuxByteView {
                data: source_key.as_ptr(),
                len: source_key.len() as u64,
            },
            expected_sha256: NuxByteView {
                data: expected_sha256.as_ptr(),
                len: expected_sha256.len() as u64,
            },
            bytes: NuxByteView::default(),
        };
        let request = NuxFlowImportRequest {
            struct_size: size_u32::<NuxFlowImportRequest>(),
            artifact_bytes: NuxByteView {
                data: artifact_bytes.as_ptr(),
                len: artifact_bytes.len() as u64,
            },
            expected_flow_id: NuxByteView {
                data: flow_id.as_ptr(),
                len: flow_id.len() as u64,
            },
            expected_build_id: NuxByteView {
                data: build_id.as_ptr(),
                len: build_id.len() as u64,
            },
            manifest_bytes: NuxByteView {
                data: manifest_bytes.as_ptr(),
                len: manifest_bytes.len() as u64,
            },
            signature_envelope_bytes: NuxByteView::default(),
            selected_key: ptr::null(),
            external_assets: &external_asset,
            external_asset_count: 1,
        };
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();

        assert_eq!(
            unsafe { nux_flow_runtime_context_create(&request, &mut context, &mut result) },
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
            centered_contain_transform(0.0, 0.0, 100.0, 50.0, 300, 300)
                .expect("valid contain transform"),
            Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 75.0])
        );
        assert_eq!(
            centered_contain_transform(0.0, 0.0, 100.0, 200.0, 300, 300)
                .expect("valid contain transform"),
            Mat2D([1.5, 0.0, 0.0, 1.5, 75.0, 0.0])
        );
        assert_eq!(
            centered_contain_transform(10.0, -5.0, 100.0, 50.0, 300, 300)
                .expect("valid offset contain transform"),
            Mat2D([3.0, 0.0, 0.0, 3.0, -30.0, 90.0])
        );
        assert!(centered_contain_transform(0.0, 0.0, 0.0, 50.0, 300, 300).is_err());
        assert!(centered_contain_transform(0.0, 0.0, 100.0, f32::NAN, 300, 300).is_err());
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
            checked, 24,
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
    fn render_factory_is_session_owned_before_and_after_surface_attachment() {
        let worker = product_fixture_worker();
        let session_id = match worker.call(None, |state| state.create_session(None, None)) {
            Ok(Ok(session_id)) => session_id,
            _ => panic!("fixture must create a default render session"),
        };
        let factory_address = worker
            .call(Some(session_id), move |state| {
                state
                    .session_mut(session_id)
                    .map(|session| (&mut *session.factory as *mut WgpuFactory).addr())
            })
            .expect("worker must inspect the session factory")
            .expect("session factory must exist before surface attachment");

        let surface_id = match worker.call(Some(session_id), move |state| {
            state.attach_surface(session_id, 8, 8)
        }) {
            Ok(Ok(surface_id)) => surface_id,
            _ => panic!("fixture must attach logical Apple presentation state"),
        };
        let attached_factory_address = worker
            .call(Some(session_id), move |state| {
                state
                    .session_mut(session_id)
                    .map(|session| (&mut *session.factory as *mut WgpuFactory).addr())
            })
            .expect("worker must inspect the attached session factory")
            .expect("session factory must remain available after attachment");
        assert_eq!(attached_factory_address, factory_address);

        worker
            .call(Some(session_id), move |state| {
                state.remove_surface(session_id, surface_id)
            })
            .expect("worker must detach logical Apple presentation state");
        let detached_factory_address = worker
            .call(Some(session_id), move |state| {
                state
                    .session_mut(session_id)
                    .map(|session| (&mut *session.factory as *mut WgpuFactory).addr())
            })
            .expect("worker must inspect the detached session factory")
            .expect("session factory must survive surface detachment");
        assert_eq!(detached_factory_address, factory_address);
    }

    #[cfg(all(feature = "apple-product", any(target_os = "ios", target_os = "macos")))]
    #[test]
    fn device_loss_reattach_transactionally_recovers_one_session_and_refreshes_the_shared_base() {
        autoreleasepool(|_| {
            let worker = product_fixture_worker();
            let create_session =
                || match worker.call(None, |state| state.create_session(None, None)) {
                    Ok(Ok(session_id)) => session_id,
                    _ => panic!("fixture must create a render session"),
                };
            let affected_id = create_session();
            let sibling_id = create_session();
            let attach_surface = |session_id| match worker.call(Some(session_id), move |state| {
                state.attach_surface(session_id, 8, 8)
            }) {
                Ok(Ok(surface_id)) => surface_id,
                _ => panic!("fixture must attach logical Apple presentation state"),
            };
            let affected_surface_id = attach_surface(affected_id);
            let sibling_surface_id = attach_surface(sibling_id);

            let affected_token = Arc::new(SessionToken {
                worker: Arc::clone(&worker),
                id: affected_id,
            });
            let affected_session = Box::into_raw(Box::new(FlowRenderSessionHandle {
                token: Arc::clone(&affected_token),
            }))
            .cast::<NuxFlowRenderSession>();
            let affected_surface = Box::into_raw(Box::new(AppleSurfaceHandle {
                token: Arc::new(SurfaceToken {
                    session: Arc::clone(&affected_token),
                    id: affected_surface_id,
                }),
            }))
            .cast::<NuxAppleSurface>();

            let sibling_token = Arc::new(SessionToken {
                worker: Arc::clone(&worker),
                id: sibling_id,
            });
            let sibling_session = Box::into_raw(Box::new(FlowRenderSessionHandle {
                token: Arc::clone(&sibling_token),
            }))
            .cast::<NuxFlowRenderSession>();
            let sibling_surface = Box::into_raw(Box::new(AppleSurfaceHandle {
                token: Arc::new(SurfaceToken {
                    session: Arc::clone(&sibling_token),
                    id: sibling_surface_id,
                }),
            }))
            .cast::<NuxAppleSurface>();

            let configure_layer = |surface: *const NuxAppleSurface| {
                let mut metal_device = ptr::null_mut();
                let mut result = ptr::null_mut();
                assert_eq!(
                    unsafe {
                        nux_apple_surface_copy_metal_device(surface, &mut metal_device, &mut result)
                    },
                    NuxStatus::Ok
                );
                unsafe { nux_operation_result_free(result) };
                let metal_device: Retained<ProtocolObject<dyn MTLDevice>> = unsafe {
                    Retained::from_raw(metal_device.cast())
                        .expect("copy_metal_device returns a retained device")
                };
                let layer = CAMetalLayer::new();
                layer.setDevice(Some(&metal_device));
                layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
                layer.setFramebufferOnly(true);
                layer.setAllowsNextDrawableTimeout(true);
                layer.setDrawableSize(CGSize::new(8.0, 8.0));
                layer
            };
            let sibling_layer = configure_layer(sibling_surface);

            let mut operation = NuxFrameOperation {
                struct_size: size_u32::<NuxFrameOperation>(),
                elapsed_seconds: 0.25,
                render: false,
                apple_drawable: ptr::null_mut(),
                completion_context: ptr::null_mut(),
                completion_callback: None,
            };
            let mut result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_advance(affected_session, &operation, &mut result)
                },
                NuxStatus::Ok
            );
            unsafe { nux_operation_result_free(result) };

            let (
                factory_address,
                flow_session_address,
                original_generation,
                original_gpu_generation,
            ) = worker
                .call(Some(affected_id), move |state| {
                    let gpu_generation = state.gpu_generation;
                    let session = state.session_mut(affected_id)?;
                    session.injected_device_loss = true;
                    Ok::<_, RuntimeFailure>((
                        (&mut *session.factory as *mut WgpuFactory).addr(),
                        std::ptr::addr_of_mut!(session.flow_session).addr(),
                        session.renderer_generation,
                        gpu_generation,
                    ))
                })
                .expect("worker accepts the test-only device-loss seam")
                .expect("affected session remains live before loss");
            let sibling_generation = worker
                .call(Some(sibling_id), move |state| {
                    state
                        .session(sibling_id)
                        .map(|session| session.renderer_generation)
                })
                .expect("worker inspects sibling generation")
                .expect("sibling remains live");
            assert_eq!(sibling_generation, original_generation);

            operation.elapsed_seconds = 0.5;
            operation.render = true;
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_advance(affected_session, &operation, &mut result)
                },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_DEVICE_LOST
            );
            unsafe { nux_operation_result_free(result) };

            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_apple_surface_detach(affected_surface, &mut result) },
                NuxStatus::Ok
            );
            unsafe { nux_operation_result_free(result) };

            let mut descriptor = NuxAppleSurfaceDescriptor {
                struct_size: size_u32::<NuxAppleSurfaceDescriptor>(),
                pixel_width: u32::MAX,
                pixel_height: 8,
            };
            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_apple_surface_reattach(affected_surface, &descriptor, &mut result) },
                NuxStatus::SurfaceError
            );
            unsafe { nux_operation_result_free(result) };
            worker
                .call(Some(affected_id), move |state| {
                    let gpu_generation = state.gpu_generation;
                    let session = state.session_mut(affected_id)?;
                    assert_eq!(
                        (&mut *session.factory as *mut WgpuFactory).addr(),
                        factory_address
                    );
                    assert_eq!(
                        std::ptr::addr_of_mut!(session.flow_session).addr(),
                        flow_session_address
                    );
                    assert_eq!(session.renderer_generation, original_generation);
                    assert_eq!(gpu_generation, original_gpu_generation);
                    assert_eq!(session.legacy_timestamp_seconds, 0.25);
                    assert!(session.injected_device_loss);
                    assert!(!session.is_fatal);
                    assert!(
                        session
                            .attachment
                            .as_ref()
                            .is_some_and(|attachment| !attachment.surface.is_attached())
                    );
                    Ok::<(), RuntimeFailure>(())
                })
                .expect("worker inspects failed recovery")
                .expect("failed recovery leaves the session retryable");

            descriptor.pixel_width = 8;
            result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_apple_surface_reattach(affected_surface, &descriptor, &mut result) },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_RECREATED
            );
            unsafe { nux_operation_result_free(result) };
            let recovered_generation = original_gpu_generation
                .checked_add(1)
                .expect("the fixture has generation capacity");
            worker
                .call(Some(affected_id), move |state| {
                    let gpu_generation = state.gpu_generation;
                    let session = state.session_mut(affected_id)?;
                    assert_eq!(
                        (&mut *session.factory as *mut WgpuFactory).addr(),
                        factory_address
                    );
                    assert_eq!(
                        std::ptr::addr_of_mut!(session.flow_session).addr(),
                        flow_session_address
                    );
                    assert_eq!(session.legacy_timestamp_seconds, 0.25);
                    assert_eq!(session.renderer_generation, recovered_generation);
                    assert_eq!(gpu_generation, recovered_generation);
                    assert!(!session.injected_device_loss);
                    assert!(!session.is_fatal);
                    Ok::<(), RuntimeFailure>(())
                })
                .expect("worker inspects successful recovery")
                .expect("successful recovery keeps the logical session live");

            let affected_layer = configure_layer(affected_surface);
            let affected_drawable = affected_layer
                .nextDrawable()
                .expect("recovered layer provides a drawable");
            operation.apple_drawable = Retained::as_ptr(&affected_drawable)
                .cast_mut()
                .cast::<c_void>();
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_advance(affected_session, &operation, &mut result)
                },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_PRESENTED
            );
            unsafe { nux_operation_result_free(result) };

            let sibling_drawable = sibling_layer
                .nextDrawable()
                .expect("the existing sibling's old domain remains usable");
            operation.apple_drawable = Retained::as_ptr(&sibling_drawable)
                .cast_mut()
                .cast::<c_void>();
            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_advance(sibling_session, &operation, &mut result)
                },
                NuxStatus::Ok
            );
            assert_eq!(
                unsafe { nux_operation_result_surface_disposition(result) },
                NUX_SURFACE_DISPOSITION_PRESENTED
            );
            unsafe { nux_operation_result_free(result) };
            let sibling_stayed_on_old_generation = worker
                .call(Some(sibling_id), move |state| {
                    state
                        .session(sibling_id)
                        .map(|session| session.renderer_generation == original_generation)
                })
                .expect("worker inspects the sibling after recovery")
                .expect("sibling remains live");
            assert!(sibling_stayed_on_old_generation);

            let new_session_id = create_session();
            let new_session_uses_refreshed_base = worker
                .call(Some(new_session_id), move |state| {
                    state
                        .session(new_session_id)
                        .map(|session| session.renderer_generation == recovered_generation)
                })
                .expect("worker inspects the post-recovery session")
                .expect("post-recovery session remains live");
            assert!(new_session_uses_refreshed_base);
            worker
                .call(None, move |state| state.remove_session(new_session_id))
                .expect("worker removes the post-recovery session");

            unsafe {
                nux_apple_surface_free(sibling_surface);
                nux_flow_render_session_free(sibling_session);
                nux_apple_surface_free(affected_surface);
                nux_flow_render_session_free(affected_session);
            }
        });
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
            let request = unsigned_import_request(&bytes);
            let mut context = ptr::null_mut();
            let mut result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_runtime_context_create(&request.request, &mut context, &mut result)
                },
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
            let session_handle = unsafe { &*session.cast::<FlowRenderSessionHandle>() };
            let session_id = session_handle.token.id;
            let render_attempts = session_handle
                .token
                .worker
                .call(Some(session_id), move |state| {
                    state
                        .session(session_id)
                        .map(|session| session.render_attempts)
                })
                .expect("worker must report render attempts")
                .expect("render session must remain live");
            assert_eq!(
                render_attempts, 0,
                "a missing drawable must skip frame construction and drawing"
            );

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

            result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_flow_render_session_advance(session, &no_drawable_operation, &mut result)
                },
                NuxStatus::SurfaceError,
                "a detached surface must fail before considering zero size or drawable availability"
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
        let request = unsigned_import_request(&bytes);
        let mut context = ptr::null_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_runtime_context_create(&request.request, &mut context, &mut result) },
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
