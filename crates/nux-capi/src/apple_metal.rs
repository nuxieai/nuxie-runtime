//! Narrow Apple/Metal extension for the portable C ABI.
//!
//! This module owns renderer and presentation mechanics only. The caller owns
//! `CAMetalLayer` configuration, drawable acquisition, frame scheduling, and
//! every product concept layered above a runtime-native player.

use super::{
    HandleKind, NuxCapiResult, NuxPlayer, NuxStatus, RendererDomain, RendererDomainBinding,
    RendererDomainCacheKey, enter_handle, enter_occurrence, ffi_guard,
    ffi_guard_with_handle_result, ffi_guard_with_result, publish_result, register_handle,
    remove_handle, struct_size_supports, write_caller_struct,
};
use dispatch2::{DispatchQueue, DispatchQueueGlobalPriority, GlobalQueueIdentifier};
use nuxie::{Mat2D, PersistentFactory, Renderer};
use nuxie_renderer::{
    NativeMetalExecutionInventory, NativeMetalFactory, RenderMode, RendererError,
};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLDevice;
use objc2_quartz_core::CAMetalDrawable;
use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

pub type NuxRendererDisposition = u32;
pub const NUX_RENDERER_DISPOSITION_NONE: NuxRendererDisposition = 0;
pub const NUX_RENDERER_DISPOSITION_PRESENTED: NuxRendererDisposition = 1;
pub const NUX_RENDERER_DISPOSITION_SKIPPED_ZERO_SIZE: NuxRendererDisposition = 2;
pub const NUX_RENDERER_DISPOSITION_SKIPPED_TIMEOUT: NuxRendererDisposition = 3;
pub const NUX_RENDERER_DISPOSITION_SKIPPED_OCCLUDED: NuxRendererDisposition = 4;
pub const NUX_RENDERER_DISPOSITION_RECONFIGURED: NuxRendererDisposition = 5;
pub const NUX_RENDERER_DISPOSITION_RECREATED: NuxRendererDisposition = 6;
pub const NUX_RENDERER_DISPOSITION_DEVICE_LOST: NuxRendererDisposition = 7;
pub const NUX_RENDERER_DISPOSITION_OUT_OF_MEMORY: NuxRendererDisposition = 8;

pub type NuxRendererHealth = u32;
pub const NUX_RENDERER_HEALTH_HEALTHY: NuxRendererHealth = 0;
pub const NUX_RENDERER_HEALTH_DEVICE_LOST: NuxRendererHealth = 1;
pub const NUX_RENDERER_HEALTH_OUT_OF_MEMORY: NuxRendererHealth = 2;
pub const NUX_RENDERER_HEALTH_FAILED: NuxRendererHealth = 3;

pub type NuxRendererFit = u32;
/// Preserve authored artboard coordinates without applying a viewport fit.
pub const NUX_RENDERER_FIT_NONE: NuxRendererFit = 0;
/// Uniformly scale and center the authored artboard inside the renderer surface.
pub const NUX_RENDERER_FIT_CONTAIN_CENTER: NuxRendererFit = 1;

pub type NuxMetalDrawableState = u32;
pub const NUX_METAL_DRAWABLE_STATE_AVAILABLE: NuxMetalDrawableState = 0;
pub const NUX_METAL_DRAWABLE_STATE_TIMEOUT: NuxMetalDrawableState = 1;
pub const NUX_METAL_DRAWABLE_STATE_OCCLUDED: NuxMetalDrawableState = 2;

/// Called exactly once after Metal finishes using a submitted drawable, or
/// after the operation is skipped or rejected. Invocation is always deferred
/// to a system dispatch queue and never occurs inline on the calling stack.
type RendererCompletionCallback = unsafe extern "C" fn(context: *mut c_void);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxMetalRenderOperation {
    /// Must be initialized to `sizeof(NuxMetalRenderOperation)`.
    pub struct_size: u32,
    /// Swift-reported drawable availability; Rust never queries UI state.
    pub drawable_state: NuxMetalDrawableState,
    /// Synchronously borrowed live `id<CAMetalDrawable>` when AVAILABLE.
    pub drawable: *mut c_void,
    /// Premultiplied ARGB clear color.
    pub clear_color: u32,
    /// Caller-owned context consumed by `completion_callback`.
    pub completion_context: *mut c_void,
    /// Both completion fields must be null or non-null together.
    pub completion_callback: Option<unsafe extern "C" fn(context: *mut c_void)>,
    /// Optional viewport-fit policy. Older struct prefixes default to NONE.
    pub fit: NuxRendererFit,
}

pub const NUX_METAL_RENDER_OPERATION_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxMetalRenderOperation, completion_callback)
        + std::mem::size_of::<Option<unsafe extern "C" fn(context: *mut c_void)>>();

impl Default for NuxMetalRenderOperation {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            drawable_state: NUX_METAL_DRAWABLE_STATE_TIMEOUT,
            drawable: ptr::null_mut(),
            clear_color: 0,
            completion_context: ptr::null_mut(),
            completion_callback: None,
            fit: NUX_RENDERER_FIT_NONE,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxRendererOutcome {
    /// Must be initialized to `sizeof(NuxRendererOutcome)`.
    pub struct_size: u32,
    pub disposition: NuxRendererDisposition,
    pub health: NuxRendererHealth,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub draw_calls: u64,
    pub logical_flushes: u64,
    pub atomic_strategy_partitions: u64,
}

pub const NUX_RENDERER_OUTCOME_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxRendererOutcome, atomic_strategy_partitions)
        + std::mem::size_of::<u64>();

impl Default for NuxRendererOutcome {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            disposition: NUX_RENDERER_DISPOSITION_NONE,
            health: NUX_RENDERER_HEALTH_HEALTHY,
            pixel_width: 0,
            pixel_height: 0,
            draw_calls: 0,
            logical_flushes: 0,
            atomic_strategy_partitions: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxRendererInfo {
    /// Must be initialized to `sizeof(NuxRendererInfo)`.
    pub struct_size: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub attached: bool,
    pub health: NuxRendererHealth,
    /// Changes whenever reattach replaces native renderer resources.
    pub generation: u64,
}

pub const NUX_RENDERER_INFO_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxRendererInfo, generation) + std::mem::size_of::<u64>();

impl Default for NuxRendererInfo {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            pixel_width: 0,
            pixel_height: 0,
            attached: false,
            health: NUX_RENDERER_HEALTH_HEALTHY,
            generation: 0,
        }
    }
}

pub(crate) struct RendererState {
    factory: PersistentFactory<NativeMetalFactory>,
    pixel_width: u32,
    pixel_height: u32,
    attached: bool,
}

impl RendererState {
    /// Internal upload seam for UNIV-1824: canonical RGBA8 premultiplied-sRGB
    /// pixels enter through the renderer factory, never as a Swift MTLTexture.
    #[allow(dead_code, reason = "consumed by the follow-up image-provider C API")]
    pub(crate) fn upload_rgba8_premul_srgb(
        &mut self,
        width: u32,
        height: u32,
        row_bytes: u32,
        pixels: &[u8],
    ) -> Result<Box<dyn nuxie::RenderImage>, RendererError> {
        self.factory
            .borrow_mut()
            .upload_rgba8_premul_srgb(width, height, row_bytes, pixels)
    }
}

/// Product-neutral native renderer. The handle owns one exact Metal domain and
/// is affine to its creator thread like the other portable C handles.
pub struct NuxRenderer {
    pub(crate) state: RefCell<RendererState>,
    domain: Arc<RendererDomain>,
}

impl NuxRenderer {
    /// Internal invalidation hook for renderer-owned provider resources.
    #[allow(dead_code, reason = "consumed by the follow-up image-provider C API")]
    pub(crate) fn domain_cache_key(&self) -> RendererDomainCacheKey {
        self.domain.cache_key()
    }
}

static NEXT_RENDERER_DOMAIN_ID: AtomicU64 = AtomicU64::new(1);

fn allocate_renderer_domain() -> Result<Arc<RendererDomain>, ApiFailure> {
    let id = NEXT_RENDERER_DOMAIN_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| {
            ApiFailure::new(
                NuxStatus::RuntimeError,
                "renderer domain identity space is exhausted",
            )
        })?;
    Ok(Arc::new(RendererDomain {
        id,
        generation: AtomicU64::new(1),
    }))
}

#[derive(Debug)]
struct ApiFailure {
    status: NuxStatus,
    message: String,
}

impl ApiFailure {
    fn new(status: NuxStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

fn renderer_failure(error: RendererError) -> ApiFailure {
    let status = if matches!(
        error,
        RendererError::InvalidDrawable(_) | RendererError::InvalidTextureExtent { .. }
    ) {
        NuxStatus::InvalidArgument
    } else {
        NuxStatus::RuntimeError
    };
    ApiFailure::new(status, error.to_string())
}

fn with_result(
    out_result: *mut *mut NuxCapiResult,
    body: impl FnOnce() -> Result<(), ApiFailure>,
) -> NuxStatus {
    ffi_guard_with_result(out_result, || {
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        match body() {
            Ok(()) => {
                publish_result(out_result, NuxStatus::Ok, "");
                NuxStatus::Ok
            }
            Err(failure) => {
                publish_result(out_result, failure.status, failure.message);
                failure.status
            }
        }
    })
}

/// Hot-path variant: diagnostics are optional and allocated only on failure.
/// A supplied result slot is cleared before any operation work begins.
fn with_optional_failure_result(
    out_result: *mut *mut NuxCapiResult,
    body: impl FnOnce() -> Result<(), ApiFailure>,
) -> NuxStatus {
    ffi_guard_with_result(out_result, || match body() {
        Ok(()) => NuxStatus::Ok,
        Err(failure) => {
            if !out_result.is_null() {
                publish_result(out_result, failure.status, &failure.message);
            }
            failure.status
        }
    })
}

fn with_owned_pointer_result<T>(
    out_owned: *mut *mut c_void,
    out_result: *mut *mut NuxCapiResult,
    body: impl FnOnce() -> Result<T, ApiFailure>,
    into_raw: impl FnOnce(T) -> *mut c_void,
) -> NuxStatus {
    if unsafe { reject_aliased_outputs(out_owned, out_result) } {
        return NuxStatus::InvalidArgument;
    }
    if !out_owned.is_null() {
        unsafe { *out_owned = ptr::null_mut() };
    }
    ffi_guard_with_result(out_result, || {
        if out_owned.is_null() || out_result.is_null() {
            if !out_result.is_null() {
                publish_result(
                    out_result,
                    NuxStatus::NullArgument,
                    "an output pointer is null",
                );
            }
            return NuxStatus::NullArgument;
        }
        match body() {
            Ok(owned) => {
                // Publish the fallible diagnostic first. If it panics, `owned`
                // unwinds as RAII and no +1 pointer has crossed the boundary.
                publish_result(out_result, NuxStatus::Ok, "");
                unsafe { *out_owned = into_raw(owned) };
                NuxStatus::Ok
            }
            Err(failure) => {
                publish_result(out_result, failure.status, &failure.message);
                failure.status
            }
        }
    })
}

fn outputs_alias<T, U>(left: *const T, right: *const U) -> bool {
    !left.is_null() && !right.is_null() && left.cast::<c_void>() == right.cast::<c_void>()
}

unsafe fn reject_aliased_outputs<T>(output: *mut T, out_result: *mut *mut NuxCapiResult) -> bool {
    if !outputs_alias(output, out_result) {
        return false;
    }
    // The shared storage cannot simultaneously contain either typed output.
    // Both representations begin with at least a pointer-sized writable slot,
    // so clear it once and publish neither.
    unsafe { *out_result = ptr::null_mut() };
    true
}

fn validate_output<T>(out: *mut T, minimum_size: usize) -> Result<(), ApiFailure> {
    if out.is_null() {
        return Err(ApiFailure::new(
            NuxStatus::NullArgument,
            "renderer output pointer is null",
        ));
    }
    let caller_size = unsafe { out.cast::<u32>().read() };
    if !struct_size_supports(caller_size, minimum_size) {
        return Err(ApiFailure::new(
            NuxStatus::InvalidStructSize,
            "renderer output struct_size is too small",
        ));
    }
    Ok(())
}

unsafe fn read_operation(
    operation: *const NuxMetalRenderOperation,
) -> Result<NuxMetalRenderOperation, ApiFailure> {
    if operation.is_null() {
        return Err(ApiFailure::new(
            NuxStatus::NullArgument,
            "render operation is null",
        ));
    }
    let caller_size = unsafe { operation.cast::<u32>().read() };
    if !struct_size_supports(caller_size, NUX_METAL_RENDER_OPERATION_V3_MIN_SIZE) {
        return Err(ApiFailure::new(
            NuxStatus::InvalidStructSize,
            "render operation struct_size is too small",
        ));
    }
    let mut value = NuxMetalRenderOperation::default();
    let read_len = usize::try_from(caller_size)
        .unwrap_or(usize::MAX)
        .min(std::mem::size_of::<NuxMetalRenderOperation>());
    unsafe {
        ptr::copy_nonoverlapping(
            operation.cast::<u8>(),
            (&mut value as *mut NuxMetalRenderOperation).cast::<u8>(),
            read_len,
        );
    }
    match value.fit {
        NUX_RENDERER_FIT_NONE | NUX_RENDERER_FIT_CONTAIN_CENTER => {}
        _ => {
            return Err(ApiFailure::new(
                NuxStatus::InvalidArgument,
                "render operation fit is unsupported",
            ));
        }
    }
    Ok(value)
}

fn centered_contain_transform(
    bounds: (f32, f32, f32, f32),
    viewport: (u32, u32),
) -> Result<Mat2D, ApiFailure> {
    let (x, y, width, height) = bounds;
    let (viewport_width, viewport_height) = viewport;
    if !x.is_finite()
        || !y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
        || viewport_width == 0
        || viewport_height == 0
    {
        return Err(ApiFailure::new(
            NuxStatus::InvalidArgument,
            "artboard bounds and renderer dimensions must be finite and positive",
        ));
    }
    let scale = (viewport_width as f32 / width).min(viewport_height as f32 / height);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(ApiFailure::new(
            NuxStatus::InvalidArgument,
            "artboard fit scale must be finite and positive",
        ));
    }
    let offset_x = (viewport_width as f32 - width * scale) * 0.5 - x * scale;
    let offset_y = (viewport_height as f32 - height * scale) * 0.5 - y * scale;
    if !offset_x.is_finite() || !offset_y.is_finite() {
        return Err(ApiFailure::new(
            NuxStatus::InvalidArgument,
            "artboard fit translation must be finite",
        ));
    }
    Ok(Mat2D([scale, 0.0, 0.0, scale, offset_x, offset_y]))
}

fn validate_completion_fields(
    context: *mut c_void,
    callback: Option<RendererCompletionCallback>,
) -> Result<(), ApiFailure> {
    if callback.is_some() == context.is_null() {
        return Err(ApiFailure::new(
            NuxStatus::InvalidArgument,
            "completion callback and context must be supplied together",
        ));
    }
    Ok(())
}

fn defer_completion(callback: RendererCompletionCallback, context_identity: usize) {
    DispatchQueue::global_queue(GlobalQueueIdentifier::Priority(
        DispatchQueueGlobalPriority::Default,
    ))
    .exec_async(move || unsafe {
        callback(ptr::with_exposed_provenance_mut(context_identity));
    });
}

struct PendingCompletion {
    callback: Option<RendererCompletionCallback>,
    context_identity: usize,
}

impl PendingCompletion {
    fn new(operation: &NuxMetalRenderOperation) -> Result<Self, ApiFailure> {
        validate_completion_fields(operation.completion_context, operation.completion_callback)?;
        Ok(Self {
            callback: operation.completion_callback,
            context_identity: operation.completion_context.expose_provenance(),
        })
    }
}

impl Drop for PendingCompletion {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            defer_completion(callback, self.context_identity);
        }
    }
}

fn validate_drawable(
    state: NuxMetalDrawableState,
    drawable: *mut c_void,
) -> Result<bool, ApiFailure> {
    match state {
        NUX_METAL_DRAWABLE_STATE_AVAILABLE if !drawable.is_null() => Ok(true),
        NUX_METAL_DRAWABLE_STATE_TIMEOUT | NUX_METAL_DRAWABLE_STATE_OCCLUDED
            if drawable.is_null() =>
        {
            Ok(false)
        }
        NUX_METAL_DRAWABLE_STATE_AVAILABLE => Err(ApiFailure::new(
            NuxStatus::NullArgument,
            "AVAILABLE requires a non-null CAMetalDrawable",
        )),
        NUX_METAL_DRAWABLE_STATE_TIMEOUT | NUX_METAL_DRAWABLE_STATE_OCCLUDED => {
            Err(ApiFailure::new(
                NuxStatus::InvalidArgument,
                "TIMEOUT and OCCLUDED require a null drawable",
            ))
        }
        _ => Err(ApiFailure::new(
            NuxStatus::InvalidArgument,
            "drawable_state is invalid",
        )),
    }
}

fn outcome(
    state: &RendererState,
    disposition: NuxRendererDisposition,
    inventory: Option<NativeMetalExecutionInventory>,
) -> NuxRendererOutcome {
    let (draw_calls, logical_flushes, atomic_strategy_partitions) =
        inventory.map_or((0, 0, 0), |inventory| {
            (
                u64::try_from(inventory.draw_calls).unwrap_or(u64::MAX),
                u64::try_from(inventory.logical_flushes).unwrap_or(u64::MAX),
                if inventory.mode == RenderMode::ClockwiseAtomic {
                    u64::try_from(inventory.atomic_draw_groups).unwrap_or(u64::MAX)
                } else {
                    0
                },
            )
        });
    NuxRendererOutcome {
        struct_size: u32::try_from(std::mem::size_of::<NuxRendererOutcome>()).unwrap_or(u32::MAX),
        disposition,
        health: NUX_RENDERER_HEALTH_HEALTHY,
        pixel_width: state.pixel_width,
        pixel_height: state.pixel_height,
        draw_calls,
        logical_flushes,
        atomic_strategy_partitions,
    }
}

fn write_outcome(
    out: *mut NuxRendererOutcome,
    value: &NuxRendererOutcome,
) -> Result<(), ApiFailure> {
    unsafe { write_caller_struct(out, value, NUX_RENDERER_OUTCOME_V3_MIN_SIZE) }
        .map_err(|status| ApiFailure::new(status, "renderer outcome could not be written"))
}

/// Creates a Metal renderer without touching UIKit, AppKit, or CAMetalLayer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_new_metal(
    pixel_width: u32,
    pixel_height: u32,
    out_renderer: *mut *mut NuxRenderer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(out_renderer, out_result, HandleKind::Renderer, || {
        if !out_renderer.is_null() {
            unsafe { *out_renderer = ptr::null_mut() };
        }
        if out_result.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_result = ptr::null_mut() };
        if out_renderer.is_null() {
            publish_result(out_result, NuxStatus::NullArgument, "out_renderer is null");
            return NuxStatus::NullArgument;
        }
        let domain = match allocate_renderer_domain() {
            Ok(domain) => domain,
            Err(failure) => {
                publish_result(out_result, failure.status, failure.message);
                return failure.status;
            }
        };
        let factory = match NativeMetalFactory::new(pixel_width.max(1), pixel_height.max(1)) {
            Ok(factory) => factory,
            Err(error) => {
                let failure = renderer_failure(error);
                publish_result(out_result, failure.status, failure.message);
                return failure.status;
            }
        };
        let renderer = Box::into_raw(Box::new(NuxRenderer {
            state: RefCell::new(RendererState {
                factory: PersistentFactory::new(factory),
                pixel_width,
                pixel_height,
                attached: true,
            }),
            domain,
        }));
        unsafe { *out_renderer = renderer };
        register_handle(renderer, HandleKind::Renderer, thread::current().id());
        publish_result(out_result, NuxStatus::Ok, "");
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_free(renderer: *mut NuxRenderer) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if renderer.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(renderer, HandleKind::Renderer) {
            return status;
        }
        unsafe { drop(Box::from_raw(renderer)) };
        NuxStatus::Ok
    })
}

/// Copies the renderer's MTLDevice with Objective-C +1 ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_copy_metal_device(
    renderer: *const NuxRenderer,
    out_device: *mut *mut c_void,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_owned_pointer_result(
        out_device,
        out_result,
        || {
            let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
                .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
            let renderer = unsafe { renderer.as_ref() }
                .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
            let state = renderer
                .state
                .try_borrow()
                .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
            Ok(state.factory.borrow().retained_metal_device())
        },
        |device: Retained<ProtocolObject<dyn MTLDevice>>| Retained::into_raw(device).cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_info(
    renderer: *const NuxRenderer,
    out_info: *mut NuxRendererInfo,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    if unsafe { reject_aliased_outputs(out_info, out_result) } {
        return NuxStatus::InvalidArgument;
    }
    with_result(out_result, || {
        validate_output(out_info, NUX_RENDERER_INFO_V3_MIN_SIZE)?;
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let state = renderer
            .state
            .try_borrow()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        let value = NuxRendererInfo {
            struct_size: u32::try_from(std::mem::size_of::<NuxRendererInfo>()).unwrap_or(u32::MAX),
            pixel_width: state.pixel_width,
            pixel_height: state.pixel_height,
            attached: state.attached,
            health: NUX_RENDERER_HEALTH_HEALTHY,
            generation: renderer.domain.generation.load(Ordering::Relaxed),
        };
        unsafe { write_caller_struct(out_info, &value, NUX_RENDERER_INFO_V3_MIN_SIZE) }
            .map_err(|status| ApiFailure::new(status, "renderer info could not be written"))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_resize(
    renderer: *mut NuxRenderer,
    pixel_width: u32,
    pixel_height: u32,
    out_outcome: *mut NuxRendererOutcome,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    if unsafe { reject_aliased_outputs(out_outcome, out_result) } {
        return NuxStatus::InvalidArgument;
    }
    with_result(out_result, || {
        validate_output(out_outcome, NUX_RENDERER_OUTCOME_V3_MIN_SIZE)?;
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        if pixel_width != 0 && pixel_height != 0 {
            state
                .factory
                .borrow_mut()
                .resize(pixel_width, pixel_height)
                .map_err(renderer_failure)?;
        }
        state.pixel_width = pixel_width;
        state.pixel_height = pixel_height;
        let disposition = if pixel_width == 0 || pixel_height == 0 {
            NUX_RENDERER_DISPOSITION_SKIPPED_ZERO_SIZE
        } else {
            NUX_RENDERER_DISPOSITION_RECONFIGURED
        };
        write_outcome(out_outcome, &outcome(&state, disposition, None))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_detach(
    renderer: *mut NuxRenderer,
    out_outcome: *mut NuxRendererOutcome,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    if unsafe { reject_aliased_outputs(out_outcome, out_result) } {
        return NuxStatus::InvalidArgument;
    }
    with_result(out_result, || {
        validate_output(out_outcome, NUX_RENDERER_OUTCOME_V3_MIN_SIZE)?;
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        state.attached = false;
        write_outcome(
            out_outcome,
            &outcome(&state, NUX_RENDERER_DISPOSITION_NONE, None),
        )
    })
}

/// Reattaches with a fresh native device domain. Players bound to an older
/// generation return HANDLE_MISMATCH until reset explicitly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_reattach(
    renderer: *mut NuxRenderer,
    pixel_width: u32,
    pixel_height: u32,
    out_outcome: *mut NuxRendererOutcome,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    if unsafe { reject_aliased_outputs(out_outcome, out_result) } {
        return NuxStatus::InvalidArgument;
    }
    with_result(out_result, || {
        validate_output(out_outcome, NUX_RENDERER_OUTCOME_V3_MIN_SIZE)?;
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        if state.attached {
            return Err(ApiFailure::new(
                NuxStatus::InvalidArgument,
                "renderer must be detached before reattach",
            ));
        }
        let factory = NativeMetalFactory::new(pixel_width.max(1), pixel_height.max(1))
            .map_err(renderer_failure)?;
        let next_generation = renderer
            .domain
            .generation
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |generation| {
                generation.checked_add(1)
            })
            .map_err(|_| {
                ApiFailure::new(
                    NuxStatus::RuntimeError,
                    "renderer generation space is exhausted",
                )
            })?
            .saturating_add(1);
        debug_assert_ne!(next_generation, 0);
        *state.factory.borrow_mut() = factory;
        state.pixel_width = pixel_width;
        state.pixel_height = pixel_height;
        state.attached = true;
        let disposition = if pixel_width == 0 || pixel_height == 0 {
            NUX_RENDERER_DISPOSITION_SKIPPED_ZERO_SIZE
        } else {
            NUX_RENDERER_DISPOSITION_RECREATED
        };
        write_outcome(out_outcome, &outcome(&state, disposition, None))
    })
}

/// Drops renderer-owned resources from the player's retained artboard and
/// binds it to this renderer's current generation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_reset_player_domain(
    renderer: *const NuxRenderer,
    player: *mut NuxPlayer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_result(out_result, || {
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let _player_call = enter_handle(player, HandleKind::Player)
            .map_err(|status| ApiFailure::new(status, "player handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let player = unsafe { player.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "player is null"))?;
        {
            let state = renderer
                .state
                .try_borrow()
                .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
            if !state.attached {
                return Err(ApiFailure::new(
                    NuxStatus::InvalidArgument,
                    "cannot bind a player to a detached renderer",
                ));
            }
        }
        let _occurrence_call = enter_occurrence(&player.artboard)
            .map_err(|status| ApiFailure::new(status, "player occurrence is unavailable"))?;
        let artboard = player.artboard.instance.try_borrow().map_err(|_| {
            ApiFailure::new(NuxStatus::ReentrantCall, "player occurrence is active")
        })?;
        artboard.reset_renderer();
        *player.artboard.renderer_domain.borrow_mut() = Some(RendererDomainBinding::Metal {
            domain: Arc::clone(&renderer.domain),
            generation: renderer.domain.generation.load(Ordering::Relaxed),
        });
        player
            .artboard
            .observed_renderer_generation
            .set(renderer.domain.generation.load(Ordering::Relaxed));
        player.artboard.invalidate_render().map_err(|status| {
            player.artboard.poisoned.set(true);
            ApiFailure::new(
                status,
                "player render revision overflowed during domain reset",
            )
        })?;
        Ok(())
    })
}

/// Renders the player's retained artboard into a caller-acquired
/// CAMetalDrawable and schedules presentation. The drawable is borrowed only
/// until this synchronous function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_render_player(
    renderer: *mut NuxRenderer,
    player: *mut NuxPlayer,
    operation: *const NuxMetalRenderOperation,
    out_outcome: *mut NuxRendererOutcome,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_result(out_result, || {
        let operation = match unsafe { read_operation(operation) } {
            Ok(operation) => operation,
            Err(failure) => return with_optional_failure_result(out_result, || Err(failure)),
        };
        let _completion = match PendingCompletion::new(&operation) {
            Ok(completion) => completion,
            Err(failure) => return with_optional_failure_result(out_result, || Err(failure)),
        };
        if unsafe { reject_aliased_outputs(out_outcome, out_result) } {
            return NuxStatus::InvalidArgument;
        }
        let drawable_available =
            match validate_drawable(operation.drawable_state, operation.drawable) {
                Ok(available) => available,
                Err(failure) => {
                    return with_optional_failure_result(out_result, || Err(failure));
                }
            };
        with_optional_failure_result(out_result, || {
            validate_output(out_outcome, NUX_RENDERER_OUTCOME_V3_MIN_SIZE)?;
            let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
                .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
            let _player_call = enter_handle(player, HandleKind::Player)
                .map_err(|status| ApiFailure::new(status, "player handle is unavailable"))?;
            let renderer_ref = unsafe { renderer.as_ref() }
                .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
            let player = unsafe { player.as_ref() }
                .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "player is null"))?;
            let _occurrence_call = enter_occurrence(&player.artboard)
                .map_err(|status| ApiFailure::new(status, "player occurrence is unavailable"))?;
            let mut state = renderer_ref
                .state
                .try_borrow_mut()
                .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
            if !state.attached {
                return Err(ApiFailure::new(
                    NuxStatus::InvalidArgument,
                    "renderer is not attached",
                ));
            }
            if state.pixel_width == 0 || state.pixel_height == 0 {
                return write_outcome(
                    out_outcome,
                    &outcome(&state, NUX_RENDERER_DISPOSITION_SKIPPED_ZERO_SIZE, None),
                );
            }
            if !drawable_available {
                let disposition = if operation.drawable_state == NUX_METAL_DRAWABLE_STATE_OCCLUDED {
                    NUX_RENDERER_DISPOSITION_SKIPPED_OCCLUDED
                } else {
                    NUX_RENDERER_DISPOSITION_SKIPPED_TIMEOUT
                };
                return write_outcome(out_outcome, &outcome(&state, disposition, None));
            }

            // Domain ownership starts only when an available drawable causes
            // this call to touch renderer-owned resources. Timeout, occlusion,
            // and zero-size preflight never lock a player to a backend.
            let generation = renderer_ref.domain.generation.load(Ordering::Relaxed);
            let existing_domain = { player.artboard.renderer_domain.borrow().clone() };
            match existing_domain {
                Some(RendererDomainBinding::Metal {
                    domain: bound_domain,
                    generation: bound_generation,
                }) if bound_domain.id == renderer_ref.domain.id
                    && Arc::ptr_eq(&bound_domain, &renderer_ref.domain)
                    && bound_generation == generation => {}
                None => {
                    *player.artboard.renderer_domain.borrow_mut() =
                        Some(RendererDomainBinding::Metal {
                            domain: Arc::clone(&renderer_ref.domain),
                            generation,
                        });
                    player.artboard.observed_renderer_generation.set(generation);
                }
                Some(_) => {
                    return Err(ApiFailure::new(
                        NuxStatus::HandleMismatch,
                        "player is bound to another renderer domain; reset it explicitly",
                    ));
                }
            }
            player
                .artboard
                .refresh_bound_view_model_invalidation()
                .map_err(|status| ApiFailure::new(status, "player render revision overflowed"))?;
            let rendered_revision = player.artboard.render_revision.get();
            let drawable = unsafe {
                NonNull::new(operation.drawable)
                    .ok_or_else(|| {
                        ApiFailure::new(
                            NuxStatus::NullArgument,
                            "AVAILABLE requires a non-null CAMetalDrawable",
                        )
                    })?
                    .cast::<ProtocolObject<dyn CAMetalDrawable>>()
                    .as_ref()
            };
            let mut frame = state
                .factory
                .borrow()
                .begin_drawable_frame(drawable, operation.clear_color)
                .map_err(renderer_failure)?;
            let mut artboard = player.artboard.instance.try_borrow_mut().map_err(|_| {
                ApiFailure::new(NuxStatus::ReentrantCall, "player occurrence is active")
            })?;
            if operation.fit == NUX_RENDERER_FIT_CONTAIN_CENTER {
                frame.transform(centered_contain_transform(
                    artboard.artboard_bounds(),
                    (state.pixel_width, state.pixel_height),
                )?);
            }
            if let Some(assets) = player.artboard.apple_assets.as_ref() {
                let mut factory = assets.wrap_factory(&mut state.factory);
                artboard.draw(&mut factory, &mut *frame).map_err(|error| {
                    ApiFailure::new(NuxStatus::RuntimeError, format!("{error:#}"))
                })?;
            } else {
                artboard
                    .draw(&mut state.factory, &mut *frame)
                    .map_err(|error| {
                        ApiFailure::new(NuxStatus::RuntimeError, format!("{error:#}"))
                    })?;
            }
            drop(artboard);
            let inventory = frame.finish().map_err(renderer_failure)?;
            player
                .artboard
                .acknowledge_presented(rendered_revision)
                .map_err(|status| {
                    ApiFailure::new(
                        status,
                        "presented player revision no longer matches the rendered occurrence",
                    )
                })?;
            write_outcome(
                out_outcome,
                &outcome(&state, NUX_RENDERER_DISPOSITION_PRESENTED, Some(inventory)),
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nux_capi_result_free;
    use std::cell::Cell;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use std::time::{Duration, Instant};

    thread_local! {
        static IN_RENDER_CALL: Cell<bool> = const { Cell::new(false) };
    }

    struct CompletionProbe {
        calls: AtomicUsize,
        inline: AtomicBool,
    }

    unsafe extern "C" fn probing_completion(context: *mut c_void) {
        let probe = unsafe { &*context.cast::<CompletionProbe>() };
        IN_RENDER_CALL.with(|active| {
            if active.get() {
                probe.inline.store(true, Ordering::Release);
            }
        });
        probe.calls.fetch_add(1, Ordering::Release);
    }

    fn operation_with_probe(probe: *mut CompletionProbe) -> NuxMetalRenderOperation {
        NuxMetalRenderOperation {
            completion_context: probe.cast(),
            completion_callback: Some(probing_completion),
            ..NuxMetalRenderOperation::default()
        }
    }

    fn wait_for_completion(probe: &CompletionProbe) {
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(2))
            .expect("two-second completion deadline fits in Instant");
        while probe.calls.load(Ordering::Acquire) == 0 && Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert_eq!(probe.calls.load(Ordering::Acquire), 1);
        assert!(!probe.inline.load(Ordering::Acquire));
    }

    fn call_with_probe(
        mut operation: NuxMetalRenderOperation,
        outcome: *mut NuxRendererOutcome,
        result: *mut *mut NuxCapiResult,
    ) -> NuxStatus {
        IN_RENDER_CALL.with(|active| active.set(true));
        let status = unsafe {
            nux_renderer_render_player(
                ptr::null_mut(),
                ptr::null_mut(),
                &raw mut operation,
                outcome,
                result,
            )
        };
        IN_RENDER_CALL.with(|active| active.set(false));
        status
    }

    unsafe extern "C" fn completion(_: *mut c_void) {}

    #[test]
    fn completion_context_and_callback_are_an_owned_pair() {
        assert!(validate_completion_fields(ptr::null_mut(), None).is_ok());
        assert!(validate_completion_fields(ptr::dangling_mut(), Some(completion)).is_ok());
        assert!(validate_completion_fields(ptr::null_mut(), Some(completion)).is_err());
        assert!(validate_completion_fields(ptr::dangling_mut(), None).is_err());
    }

    #[test]
    fn drawable_availability_is_explicit_and_pointer_consistent() {
        assert!(matches!(
            validate_drawable(NUX_METAL_DRAWABLE_STATE_AVAILABLE, ptr::dangling_mut()),
            Ok(true)
        ));
        assert!(matches!(
            validate_drawable(NUX_METAL_DRAWABLE_STATE_TIMEOUT, ptr::null_mut()),
            Ok(false)
        ));
        assert!(matches!(
            validate_drawable(NUX_METAL_DRAWABLE_STATE_OCCLUDED, ptr::null_mut()),
            Ok(false)
        ));
        assert!(validate_drawable(NUX_METAL_DRAWABLE_STATE_AVAILABLE, ptr::null_mut()).is_err());
        assert!(validate_drawable(NUX_METAL_DRAWABLE_STATE_TIMEOUT, ptr::dangling_mut()).is_err());
        assert!(validate_drawable(u32::MAX, ptr::null_mut()).is_err());
    }

    #[test]
    fn renderer_outcomes_are_stable_fixed_width_values() {
        assert_eq!(NUX_RENDERER_DISPOSITION_PRESENTED, 1);
        assert_eq!(NUX_RENDERER_DISPOSITION_SKIPPED_TIMEOUT, 3);
        assert_eq!(NUX_RENDERER_DISPOSITION_SKIPPED_OCCLUDED, 4);
        assert_eq!(NUX_RENDERER_DISPOSITION_RECREATED, 6);
        assert_eq!(NUX_RENDERER_DISPOSITION_OUT_OF_MEMORY, 8);
        assert_eq!(std::mem::size_of::<NuxRendererDisposition>(), 4);
        assert_eq!(std::mem::size_of::<NuxRendererHealth>(), 4);
    }

    #[test]
    fn centered_contain_fit_scales_and_letterboxes_authored_bounds() {
        assert_eq!(NUX_RENDERER_FIT_NONE, 0);
        assert_eq!(NUX_RENDERER_FIT_CONTAIN_CENTER, 1);
        assert_eq!(std::mem::size_of::<NuxRendererFit>(), 4);

        assert_eq!(
            centered_contain_transform((0.0, 0.0, 100.0, 50.0), (300, 300))
                .expect("landscape bounds fit"),
            Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 75.0])
        );
        assert_eq!(
            centered_contain_transform((0.0, 0.0, 100.0, 200.0), (300, 300))
                .expect("portrait bounds fit"),
            Mat2D([1.5, 0.0, 0.0, 1.5, 75.0, 0.0])
        );
        assert_eq!(
            centered_contain_transform((10.0, -5.0, 100.0, 50.0), (300, 300))
                .expect("offset bounds fit"),
            Mat2D([3.0, 0.0, 0.0, 3.0, -30.0, 90.0])
        );
        for bounds in [(0.0, 0.0, 0.0, 50.0), (0.0, 0.0, 100.0, f32::NAN)] {
            assert!(centered_contain_transform(bounds, (300, 300)).is_err());
        }
        assert!(centered_contain_transform((0.0, 0.0, 100.0, 50.0), (0, 300)).is_err());
    }

    #[test]
    fn legacy_render_operation_prefix_defaults_to_raw_coordinates() {
        #[repr(C)]
        struct LegacyRenderOperation {
            struct_size: u32,
            drawable_state: NuxMetalDrawableState,
            drawable: *mut c_void,
            clear_color: u32,
            completion_context: *mut c_void,
            completion_callback: Option<RendererCompletionCallback>,
        }

        let legacy = LegacyRenderOperation {
            struct_size: u32::try_from(std::mem::size_of::<LegacyRenderOperation>())
                .expect("legacy operation size fits u32"),
            drawable_state: NUX_METAL_DRAWABLE_STATE_TIMEOUT,
            drawable: ptr::null_mut(),
            clear_color: 0,
            completion_context: ptr::null_mut(),
            completion_callback: None,
        };
        let operation =
            unsafe { read_operation((&raw const legacy).cast::<NuxMetalRenderOperation>()) }
                .expect("legacy operation prefix remains accepted");
        assert_eq!(operation.fit, NUX_RENDERER_FIT_NONE);

        let invalid = NuxMetalRenderOperation {
            fit: u32::MAX,
            ..NuxMetalRenderOperation::default()
        };
        assert!(unsafe { read_operation(&raw const invalid) }.is_err());
    }

    #[test]
    fn valid_completion_is_deferred_once_on_validation_failures() {
        // A null optional result slot must not prevent completion ownership.
        let probe = Box::into_raw(Box::new(CompletionProbe {
            calls: AtomicUsize::new(0),
            inline: AtomicBool::new(false),
        }));
        let mut outcome = NuxRendererOutcome::default();
        let status = call_with_probe(
            operation_with_probe(probe),
            &raw mut outcome,
            ptr::null_mut(),
        );
        assert_eq!(status, NuxStatus::NullArgument);
        wait_for_completion(unsafe { &*probe });
        unsafe { drop(Box::from_raw(probe)) };

        // An invalid drawable pair is rejected after acquiring completion.
        let probe = Box::into_raw(Box::new(CompletionProbe {
            calls: AtomicUsize::new(0),
            inline: AtomicBool::new(false),
        }));
        let mut operation = operation_with_probe(probe);
        operation.drawable_state = NUX_METAL_DRAWABLE_STATE_AVAILABLE;
        let status = call_with_probe(operation, &raw mut outcome, ptr::null_mut());
        assert_eq!(status, NuxStatus::NullArgument);
        wait_for_completion(unsafe { &*probe });
        unsafe { drop(Box::from_raw(probe)) };

        // Aliased slots are cleared and cannot receive either output type.
        let probe = Box::into_raw(Box::new(CompletionProbe {
            calls: AtomicUsize::new(0),
            inline: AtomicBool::new(false),
        }));
        let mut shared = ptr::dangling_mut::<NuxCapiResult>();
        let shared_slot = &raw mut shared;
        let status = call_with_probe(
            operation_with_probe(probe),
            shared_slot.cast::<NuxRendererOutcome>(),
            shared_slot,
        );
        assert_eq!(status, NuxStatus::InvalidArgument);
        assert!(shared.is_null());
        wait_for_completion(unsafe { &*probe });
        unsafe { drop(Box::from_raw(probe)) };

        // Invalid handles publish a bounded failure only when requested.
        let probe = Box::into_raw(Box::new(CompletionProbe {
            calls: AtomicUsize::new(0),
            inline: AtomicBool::new(false),
        }));
        let mut result = ptr::dangling_mut();
        let status = call_with_probe(
            operation_with_probe(probe),
            &raw mut outcome,
            &raw mut result,
        );
        assert_eq!(status, NuxStatus::NullArgument);
        assert!(!result.is_null());
        wait_for_completion(unsafe { &*probe });
        assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
        unsafe { drop(Box::from_raw(probe)) };
    }

    #[test]
    fn short_operation_prefix_cannot_transfer_an_unread_completion() {
        let probe = Box::into_raw(Box::new(CompletionProbe {
            calls: AtomicUsize::new(0),
            inline: AtomicBool::new(false),
        }));
        let mut operation = operation_with_probe(probe);
        operation.struct_size = u32::try_from(std::mem::size_of::<u32>())
            .expect("u32 size fits in the ABI struct-size field");
        let mut outcome = NuxRendererOutcome::default();
        let status = call_with_probe(operation, &raw mut outcome, ptr::null_mut());
        assert_eq!(status, NuxStatus::InvalidStructSize);
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(unsafe { &*probe }.calls.load(Ordering::Acquire), 0);
        unsafe { drop(Box::from_raw(probe)) };
    }

    #[test]
    fn control_and_hot_path_result_firewalls_replace_partial_publications() {
        fn assert_generic_runtime_failure(result: *mut NuxCapiResult) {
            assert!(!result.is_null());
            let mut result_status = NuxStatus::Ok;
            assert_eq!(
                unsafe { crate::nux_capi_result_status(result, &raw mut result_status) },
                NuxStatus::Ok
            );
            assert_eq!(result_status, NuxStatus::RuntimeError);
            assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
        }

        let mut control_result = ptr::null_mut();
        let control_result_slot = &raw mut control_result;
        let status = with_result(control_result_slot, || -> Result<(), ApiFailure> {
            publish_result(control_result_slot, NuxStatus::Ok, "partial control result");
            panic!("injected panic after control result publication");
        });
        assert_eq!(status, NuxStatus::RuntimeError);
        assert_generic_runtime_failure(control_result);

        let mut hot_result = ptr::null_mut();
        let hot_result_slot = &raw mut hot_result;
        let status = with_optional_failure_result(hot_result_slot, || -> Result<(), ApiFailure> {
            publish_result(
                hot_result_slot,
                NuxStatus::InvalidArgument,
                "partial hot result",
            );
            panic!("injected panic after hot result publication");
        });
        assert_eq!(status, NuxStatus::RuntimeError);
        assert_generic_runtime_failure(hot_result);
    }

    #[test]
    fn owned_pointer_and_renderer_create_roll_back_after_result_publication_panics() {
        struct DropProbe(Arc<AtomicUsize>);
        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Release);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        let mut owned = ptr::dangling_mut::<c_void>();
        let mut result = ptr::null_mut();
        crate::panic_after_next_result_publication();
        let status = with_owned_pointer_result(
            &raw mut owned,
            &raw mut result,
            || Ok::<_, ApiFailure>(DropProbe(Arc::clone(&drops))),
            |_owned| ptr::dangling_mut(),
        );
        assert_eq!(status, NuxStatus::RuntimeError);
        assert!(owned.is_null());
        assert_eq!(drops.load(Ordering::Acquire), 1);
        let mut result_status = NuxStatus::Ok;
        assert_eq!(
            unsafe { crate::nux_capi_result_status(result, &raw mut result_status) },
            NuxStatus::Ok
        );
        assert_eq!(result_status, NuxStatus::RuntimeError);
        assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);

        let mut renderer = ptr::null_mut();
        result = ptr::null_mut();
        crate::panic_after_next_result_publication();
        let status = unsafe { nux_renderer_new_metal(1, 1, &raw mut renderer, &raw mut result) };
        assert_eq!(status, NuxStatus::RuntimeError);
        assert!(renderer.is_null());
        assert_eq!(
            unsafe { crate::nux_capi_result_status(result, &raw mut result_status) },
            NuxStatus::Ok
        );
        assert_eq!(result_status, NuxStatus::RuntimeError);
        assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
    }
}
