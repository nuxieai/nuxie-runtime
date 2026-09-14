//! Android/Vulkan extension: CPU export and GPU window presentation share
//! the same retained factory, handle admission and deferred scene recording.

use super::{
    HandleKind, NuxCapiResult, NuxFile, NuxFileImportConfig, NuxPlayer, NuxStatus,
    PendingHandlePublication, RendererDomain, RendererDomainBinding, enter_handle,
    enter_occurrence, ffi_guard, ffi_guard_with_handle_result, ffi_guard_with_result,
    publish_result, register_handle, remove_handle,
};
use nuxie::PersistentFactory;
use nuxie::render_api::Mat2D;
#[cfg(test)]
use nuxie_renderer::RenderMode;
use nuxie_renderer::deferred::cmd::deferred_replayer::take_frame;
use nuxie_renderer::{NativeVulkanFactory, RendererError};
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

mod deferred;
use deferred::{AndroidVulkanFactory, AndroidVulkanFrameSink};

pub type NuxAndroidVulkanRendererFit = u32;
/// Preserve authored artboard coordinates without applying a viewport fit.
pub const NUX_ANDROID_VULKAN_RENDERER_FIT_NONE: NuxAndroidVulkanRendererFit = 0;
/// Uniformly scale and center the authored artboard inside the output frame.
pub const NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER: NuxAndroidVulkanRendererFit = 1;

pub type NuxAndroidVulkanPixelFormat = u32;
/// Tightly packed, top-row-first RGBA8 UNORM with premultiplied alpha.
pub const NUX_ANDROID_VULKAN_PIXEL_FORMAT_RGBA8_PREMULTIPLIED: NuxAndroidVulkanPixelFormat = 1;

struct PendingPresentation {
    occurrence: Rc<super::ArtboardOccurrence>,
    revision: u64,
}

enum RenderDelivery { Undelivered, Completed, Submitted }

struct AndroidVulkanRendererState {
    pending: RefCell<Option<PendingPresentation>>,
    factory: PersistentFactory<crate::asset_hooks::AssetFactory<AndroidVulkanFactory>>,
    pixel_width: u32,
    pixel_height: u32,
}

impl AndroidVulkanRendererState {
    fn validate_pending(&self, player: &NuxPlayer) -> Result<(), ApiFailure> {
        if let Some(pending) = self.pending.borrow().as_ref() {
            if !Rc::ptr_eq(&pending.occurrence, &player.artboard) {
                return Err(ApiFailure::new(NuxStatus::HandleMismatch,
                    "pending surface frame belongs to another occurrence"));
            }
        }
        Ok(())
    }

    fn complete_pending(&self, player: &NuxPlayer) -> Result<(), ApiFailure> {
        self.validate_pending(player)?;
        let pending = self.pending.borrow_mut().take().ok_or_else(||
            ApiFailure::new(NuxStatus::RuntimeError, "surface completion has no submitted occurrence"))?;
        pending.occurrence.refresh_bound_view_model_invalidation()
            .map_err(|status| ApiFailure::new(status, "pending occurrence revision overflowed"))?;
        // Completion names the submitted revision, never a later phase/input
        // write. Leave a newer revision dirty so it receives its own frame.
        if pending.occurrence.render_revision.get() == pending.revision {
            pending.occurrence.acknowledge_presented(pending.revision)
                .map_err(|status| ApiFailure::new(status, "pending occurrence acknowledgement failed"))?;
        }
        Ok(())
    }

    fn discard_pending(&self) -> Result<(), ApiFailure> {
        #[cfg(target_os = "android")]
        self.factory.borrow().native.borrow_mut().drain_surface_frame()
            .map_err(renderer_failure)?;
        self.pending.borrow_mut().take();
        Ok(())
    }
}

impl Drop for AndroidVulkanRendererState {
    fn drop(&mut self) {
        // Keep the submitted occurrence alive until native work is drained.
        // Imported players may retain the factory after this renderer is freed.
        #[cfg(target_os = "android")]
        {
            let _ = self.factory.borrow().native.borrow_mut().detach_android_surface();
        }
        self.pending.get_mut().take();
    }
}

/// Vulkan renderer with CPU export and optional Android surface presentation.
/// The handle and every CPU frame are affine to the thread that created them.
pub struct NuxAndroidVulkanRenderer {
    state: RefCell<AndroidVulkanRendererState>,
    domain: Arc<RendererDomain>,
}

pub(crate) unsafe fn import_android_vulkan_file_with_authority(
    renderer: *mut NuxAndroidVulkanRenderer,
    bytes: *const u8,
    len: usize,
    config: *const NuxFileImportConfig,
    out_file: *mut *mut NuxFile,
    out_result: *mut *mut NuxCapiResult,
    authority: super::NativeShaderImportAuthority,
    program_adapter: Option<Arc<dyn nuxie::ScriptProgramAdapter>>,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _renderer_call = match enter_handle(renderer, HandleKind::AndroidVulkanRenderer) {
            Ok(guard) => guard,
            Err(status) => {
                if !out_result.is_null() {
                    publish_result(out_result, status, "renderer handle is unavailable");
                }
                return status;
            }
        };
        let Some(renderer) = (unsafe { renderer.as_ref() }) else {
            if !out_result.is_null() {
                publish_result(out_result, NuxStatus::NullArgument, "renderer is null");
            }
            return NuxStatus::NullArgument;
        };
        let state = match renderer.state.try_borrow() {
            Ok(state) => state,
            Err(_) => {
                if !out_result.is_null() {
                    publish_result(out_result, NuxStatus::ReentrantCall, "renderer is active");
                }
                return NuxStatus::ReentrantCall;
            }
        };
        let factory = state.factory.clone();
        let generation = renderer.domain.generation.load(Ordering::Relaxed);
        drop(state);
        unsafe {
            super::asset_hooks::nux_file_import_configured_with_factory(
                bytes,
                len,
                config,
                out_file,
                out_result,
                factory,
                RendererDomainBinding::AndroidVulkan {
                    domain: Arc::clone(&renderer.domain),
                    generation,
                },
                authority,
                program_adapter,
            )
        }
    })
}

/// Imports a file into this renderer's retained factory domain. The returned
/// file and every occurrence created from it remain bound to this exact Vulkan
/// renderer generation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_import_android_vulkan(
    renderer: *mut NuxAndroidVulkanRenderer,
    bytes: *const u8,
    len: usize,
    config: *const NuxFileImportConfig,
    out_file: *mut *mut NuxFile,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    unsafe {
        import_android_vulkan_file_with_authority(
            renderer,
            bytes,
            len,
            config,
            out_file,
            out_result,
            super::NativeShaderImportAuthority::Denied,
            None,
        )
    }
}

/// Owned frame pixels returned by the Android Vulkan renderer.
///
/// `data` exposes tightly packed, top-row-first RGBA8 UNORM bytes with
/// premultiplied alpha. Its borrowed pointer remains valid until `_free`.
pub struct NuxAndroidVulkanFrame {
    pixels: Box<[u8]>,
    width: u32,
    height: u32,
    row_stride_bytes: u32,
}

static NEXT_ANDROID_VULKAN_RENDERER_DOMAIN_ID: AtomicU64 = AtomicU64::new(1);

fn allocate_renderer_domain() -> Result<Arc<RendererDomain>, ApiFailure> {
    let id = NEXT_ANDROID_VULKAN_RENDERER_DOMAIN_ID
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
    let status = if matches!(error, RendererError::InvalidTextureExtent { .. }) {
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

fn publish_optional_failure(out_result: *mut *mut NuxCapiResult, failure: ApiFailure) -> NuxStatus {
    if !out_result.is_null() {
        publish_result(out_result, failure.status, &failure.message);
    }
    failure.status
}

fn validate_extent(pixel_width: u32, pixel_height: u32) -> Result<(), ApiFailure> {
    if pixel_width == 0 || pixel_height == 0 {
        return Err(ApiFailure::new(
            NuxStatus::InvalidArgument,
            "Vulkan frame extent must be non-zero",
        ));
    }
    Ok(())
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

/// Creates a headless Vulkan renderer at the requested pixel extent.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_new_android_vulkan(
    pixel_width: u32,
    pixel_height: u32,
    out_renderer: *mut *mut NuxAndroidVulkanRenderer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(
        out_renderer,
        out_result,
        HandleKind::AndroidVulkanRenderer,
        || {
            if out_result.is_null() {
                return NuxStatus::NullArgument;
            }
            if out_renderer.is_null() {
                publish_result(out_result, NuxStatus::NullArgument, "out_renderer is null");
                return NuxStatus::NullArgument;
            }
            if let Err(failure) = validate_extent(pixel_width, pixel_height) {
                publish_result(out_result, failure.status, failure.message);
                return failure.status;
            }
            let domain = match allocate_renderer_domain() {
                Ok(domain) => domain,
                Err(failure) => {
                    publish_result(out_result, failure.status, failure.message);
                    return failure.status;
                }
            };
            let factory = match NativeVulkanFactory::new(pixel_width, pixel_height) {
                Ok(factory) => factory,
                Err(error) => {
                    let failure = renderer_failure(error);
                    publish_result(out_result, failure.status, failure.message);
                    return failure.status;
                }
            };
            let pending = PendingHandlePublication::new(
                NuxAndroidVulkanRenderer {
                    state: RefCell::new(AndroidVulkanRendererState {
                        pending: RefCell::new(None),
                        factory: PersistentFactory::new(crate::asset_hooks::AssetFactory::new(
                            AndroidVulkanFactory::new(factory),
                        )),
                        pixel_width,
                        pixel_height,
                    }),
                    domain,
                },
                HandleKind::AndroidVulkanRenderer,
            );
            register_handle(
                pending.handle,
                HandleKind::AndroidVulkanRenderer,
                thread::current().id(),
            );
            unsafe { *out_renderer = pending.finish() };
            publish_result(out_result, NuxStatus::Ok, "");
            NuxStatus::Ok
        },
    )
}

/// Recreates the headless target at a new non-zero extent. The durable domain
/// and generation are retained, so players already bound to this renderer
/// continue to render without an explicit reset.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_vulkan_resize(
    renderer: *mut NuxAndroidVulkanRenderer,
    pixel_width: u32,
    pixel_height: u32,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_result(out_result, || {
        validate_extent(pixel_width, pixel_height)?;
        let _renderer_call = enter_handle(renderer, HandleKind::AndroidVulkanRenderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        state.discard_pending()?;
        state
            .factory
            .borrow_mut()
            .resize(pixel_width, pixel_height)
            .map_err(renderer_failure)?;
        state.pixel_width = pixel_width;
        state.pixel_height = pixel_height;
        Ok(())
    })
}

// Keep handle/domain admission, recording, replay and revision acknowledgement
// identical for CPU export and Android presentation. The completion callback
// decides whether an output was actually delivered before acknowledging it.
fn with_rendered_player<T>(
    renderer: *mut NuxAndroidVulkanRenderer,
    player: *mut NuxPlayer,
    clear_color: u32,
    fit: NuxAndroidVulkanRendererFit,
    admit: impl FnOnce(&AndroidVulkanRendererState, &NuxPlayer) -> Result<Option<T>, ApiFailure>,
    complete: impl FnOnce(
        AndroidVulkanFrameSink,
        &AndroidVulkanRendererState,
    ) -> Result<(T, RenderDelivery), ApiFailure>,
) -> Result<T, ApiFailure> {
    if fit != NUX_ANDROID_VULKAN_RENDERER_FIT_NONE
        && fit != NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER
    {
        return Err(ApiFailure::new(
            NuxStatus::InvalidArgument,
            "unknown Android Vulkan renderer fit",
        ));
    }
    let _renderer_call = enter_handle(renderer, HandleKind::AndroidVulkanRenderer)
        .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
    let _player_call = enter_handle(player, HandleKind::Player)
        .map_err(|status| ApiFailure::new(status, "player handle is unavailable"))?;
    let renderer_ref = unsafe { renderer.as_ref() }
        .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
    let player = unsafe { player.as_ref() }
        .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "player is null"))?;
    let _occurrence_call = enter_occurrence(&player.artboard)
        .map_err(|status| ApiFailure::new(status, "player occurrence is unavailable"))?;
    let state = renderer_ref
        .state
        .try_borrow_mut()
        .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;

    let generation = renderer_ref.domain.generation.load(Ordering::Relaxed);
    match &player.artboard.renderer_domain {
        RendererDomainBinding::AndroidVulkan {
            domain: bound_domain,
            generation: bound_generation,
        } if bound_domain.id == renderer_ref.domain.id
            && Arc::ptr_eq(&bound_domain, &renderer_ref.domain)
            && *bound_generation == generation => {}
        _ => {
            return Err(ApiFailure::new(
                NuxStatus::HandleMismatch,
                "player was not imported through this Vulkan renderer generation",
            ));
        }
    }

    if let Some(output) = admit(&state, player)? {
        return Ok(output);
    }

    if state.pending.borrow().is_some() {
        return Err(ApiFailure::new(NuxStatus::ReentrantCall, "surface frame is still pending"));
    }
    player
        .artboard
        .refresh_bound_view_model_invalidation()
        .map_err(|status| ApiFailure::new(status, "player render revision overflowed"))?;
    let rendered_revision = player.artboard.render_revision.get();
    // Script callbacks allocate into the producer factory, so do
    // not hold its borrow while recording the artboard.
    let (mut session, replayer, native) = {
        let factory = state.factory.borrow();
        (
            factory.session.clone(),
            factory.replayer.clone(),
            factory.native.clone(),
        )
    };
    session.record_ore_replay_marker();
    let mut recording = session.make_screen_renderer(0);
    recording.save();
    {
        let artboard = player.artboard.instance.try_borrow_mut().map_err(|_| {
            ApiFailure::new(NuxStatus::ReentrantCall, "player occurrence is active")
        })?;
        if fit == NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER {
            recording.transform(centered_contain_transform(
                artboard.artboard_bounds(),
                (state.pixel_width, state.pixel_height),
            )?);
        }
        artboard.draw(recording.as_mut());
    }
    recording.restore();
    drop(recording);
    let frame = take_frame(&mut session);
    let mut sink = AndroidVulkanFrameSink::new(native, clear_color);
    replayer.borrow_mut().replay_frame(&frame, &mut sink);
    let (output, delivered) = complete(sink, &state)?;
    if matches!(delivered, RenderDelivery::Submitted) {
        *state.pending.borrow_mut() = Some(PendingPresentation {
            occurrence: Rc::clone(&player.artboard), revision: rendered_revision,
        });
    }
    if matches!(delivered, RenderDelivery::Completed) {
        player
            .artboard
            .acknowledge_presented(rendered_revision)
            .map_err(|status| {
                ApiFailure::new(
                    status,
                    "presented player revision no longer matches the rendered occurrence",
                )
            })?;
    }
    Ok(output)
}

pub type NuxAndroidVulkanPresentation = u32;
pub const NUX_ANDROID_VULKAN_PRESENTATION_UNAVAILABLE: NuxAndroidVulkanPresentation = 0;
pub const NUX_ANDROID_VULKAN_PRESENTATION_PRESENTED: NuxAndroidVulkanPresentation = 1;
pub const NUX_ANDROID_VULKAN_PRESENTATION_SUBOPTIMAL: NuxAndroidVulkanPresentation = 2;
pub const NUX_ANDROID_VULKAN_PRESENTATION_REATTACH: NuxAndroidVulkanPresentation = 3;
/// Queued but not completed. Poll with the same occurrence without stepping.
pub const NUX_ANDROID_VULKAN_PRESENTATION_SUBMITTED: NuxAndroidVulkanPresentation = 4;

/// Attaches a live ANativeWindow on the renderer's owning thread. The caller must
/// exclude other graphics producers for this window. Vulkan retains a window
/// reference until detach, replacement or renderer destruction. The native alpha
/// flag must only be true when window composition guarantees premultiplied alpha.
/// Attachment preserves the renderer domain and adopts the reported surface extent.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_vulkan_attach_surface(
    renderer: *mut NuxAndroidVulkanRenderer,
    native_window: *mut std::ffi::c_void,
    pixel_width: u32,
    pixel_height: u32,
    native_premultiplied_alpha: bool,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_result(out_result, || {
        let window = std::ptr::NonNull::new(native_window)
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "native window is null"))?;
        validate_extent(pixel_width, pixel_height)?;
        let _call = enter_handle(renderer, HandleKind::AndroidVulkanRenderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        state.discard_pending()?;
        let extent = {
            let factory = state.factory.borrow();
            let mut native = factory.native.borrow_mut();
            unsafe {
                native.attach_android_surface(
                    window,
                    pixel_width,
                    pixel_height,
                    native_premultiplied_alpha,
                )
            }
            .map_err(renderer_failure)?;
            native.pixel_extent()
        };
        (state.pixel_width, state.pixel_height) = extent;
        Ok(())
    })
}

/// Drains and releases the attached Vulkan surface before the caller releases
/// its own ANativeWindow reference. The renderer and its imported players survive.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_vulkan_detach_surface(
    renderer: *mut NuxAndroidVulkanRenderer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_result(out_result, || {
        let _call = enter_handle(renderer, HandleKind::AndroidVulkanRenderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        state.discard_pending()?;
        state
            .factory
            .borrow()
            .native
            .borrow_mut()
            .detach_android_surface()
            .map_err(renderer_failure)
    })
}

/// Renders into the attached surface without CPU pixel readback. A successful
/// call writes PRESENTED, UNAVAILABLE, SUBOPTIMAL, REATTACH or SUBMITTED.
/// SUBMITTED retains the exact occurrence/revision; call again with that occurrence
/// to poll without stepping. A completing poll does not record another frame.
/// Phase/input writes remain dirty if newer than the completed revision. Only a delivered frame
/// acknowledges the player's rendered revision. SUBOPTIMAL and REATTACH require
/// reattachment; REATTACH did not deliver this frame and preserves the player.
/// Errors require caller recovery. out_result is optional and failure-only;
/// out_presentation is required and reset on entry.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_vulkan_present_player(
    renderer: *mut NuxAndroidVulkanRenderer,
    player: *mut NuxPlayer,
    clear_color: u32,
    fit: NuxAndroidVulkanRendererFit,
    out_presentation: *mut NuxAndroidVulkanPresentation,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_result(out_result, || {
        let result = (|| -> Result<(), ApiFailure> {
            if out_presentation.is_null() {
                return Err(ApiFailure::new(
                    NuxStatus::NullArgument,
                    "out_presentation is null",
                ));
            }
            unsafe { *out_presentation = NUX_ANDROID_VULKAN_PRESENTATION_UNAVAILABLE };
            let outcome =
                with_rendered_player(renderer, player, clear_color, fit, |state, player| {
                    state.validate_pending(player)?;
                    let factory = state.factory.borrow();
                    let admission = factory.native.borrow_mut().prepare_surface_frame()
                        .map_err(renderer_failure)?;
                    Ok(match admission {
                        nuxie_renderer::NativeVulkanSurfaceAdmission::Ready => None,
                        nuxie_renderer::NativeVulkanSurfaceAdmission::Submitted =>
                            Some(NUX_ANDROID_VULKAN_PRESENTATION_SUBMITTED),
                        nuxie_renderer::NativeVulkanSurfaceAdmission::Presented => {
                            state.complete_pending(player)?;
                            Some(NUX_ANDROID_VULKAN_PRESENTATION_PRESENTED)
                        }
                        nuxie_renderer::NativeVulkanSurfaceAdmission::Suboptimal => {
                            state.complete_pending(player)?;
                            Some(NUX_ANDROID_VULKAN_PRESENTATION_SUBOPTIMAL)
                        }
                        nuxie_renderer::NativeVulkanSurfaceAdmission::Unavailable =>
                            Some(NUX_ANDROID_VULKAN_PRESENTATION_UNAVAILABLE),
                        nuxie_renderer::NativeVulkanSurfaceAdmission::Reattach => {
                            state.pending.borrow_mut().take();
                            Some(NUX_ANDROID_VULKAN_PRESENTATION_REATTACH)
                        }
                    })
                }, |sink, _state| {
                    let (outcome, delivered) = match sink.finish_and_present()? {
                        nuxie_renderer::NativeVulkanPresentation::Submitted => {
                            (NUX_ANDROID_VULKAN_PRESENTATION_SUBMITTED, RenderDelivery::Submitted)
                        }
                        nuxie_renderer::NativeVulkanPresentation::Presented => {
                            (NUX_ANDROID_VULKAN_PRESENTATION_PRESENTED, RenderDelivery::Completed)
                        }
                        nuxie_renderer::NativeVulkanPresentation::Suboptimal => {
                            (NUX_ANDROID_VULKAN_PRESENTATION_SUBOPTIMAL, RenderDelivery::Completed)
                        }
                        nuxie_renderer::NativeVulkanPresentation::Reattach => {
                            (NUX_ANDROID_VULKAN_PRESENTATION_REATTACH, RenderDelivery::Undelivered)
                        }
                        nuxie_renderer::NativeVulkanPresentation::Unavailable => {
                            (NUX_ANDROID_VULKAN_PRESENTATION_UNAVAILABLE, RenderDelivery::Undelivered)
                        }
                    };
                    Ok((outcome, delivered))
                })?;
            unsafe { *out_presentation = outcome };
            Ok(())
        })();
        match result {
            Ok(()) => NuxStatus::Ok,
            Err(failure) => publish_optional_failure(out_result, failure),
        }
    })
}

/// Renders a player into a newly owned CPU frame. `out_result` is optional and
/// failure-only: when supplied it stays NULL on success and owns a diagnostic
/// on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_vulkan_render_player(
    renderer: *mut NuxAndroidVulkanRenderer,
    player: *mut NuxPlayer,
    clear_color: u32,
    fit: NuxAndroidVulkanRendererFit,
    out_frame: *mut *mut NuxAndroidVulkanFrame,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_handle_result(
        out_frame,
        out_result,
        HandleKind::AndroidVulkanFrame,
        || {
            let result = (|| -> Result<(), ApiFailure> {
                if out_frame.is_null() {
                    return Err(ApiFailure::new(
                        NuxStatus::NullArgument,
                        "out_frame is null",
                    ));
                }
                with_rendered_player(renderer, player, clear_color, fit, |state, _| { state.discard_pending()?; Ok(None) }, |sink, state| {
                    let pixels = sink.finish()?;
                    let row_stride_bytes = state.pixel_width.checked_mul(4).ok_or_else(|| {
                        ApiFailure::new(NuxStatus::RuntimeError, "frame row stride overflowed")
                    })?;
                    let expected_len = usize::try_from(
                        u64::from(row_stride_bytes) * u64::from(state.pixel_height),
                    )
                    .map_err(|_| {
                        ApiFailure::new(NuxStatus::RuntimeError, "frame byte length overflowed")
                    })?;
                    if pixels.len() != expected_len {
                        return Err(ApiFailure::new(
                            NuxStatus::RuntimeError,
                            format!(
                                "Vulkan readback returned {} bytes, expected {expected_len}",
                                pixels.len()
                            ),
                        ));
                    }
                    let pending = PendingHandlePublication::new(
                        NuxAndroidVulkanFrame {
                            pixels: pixels.into_boxed_slice(),
                            width: state.pixel_width,
                            height: state.pixel_height,
                            row_stride_bytes,
                        },
                        HandleKind::AndroidVulkanFrame,
                    );
                    register_handle(
                        pending.handle,
                        HandleKind::AndroidVulkanFrame,
                        thread::current().id(),
                    );
                    unsafe { *out_frame = pending.finish() };
                    Ok(((), RenderDelivery::Completed))
                })
            })();
            match result {
                Ok(()) => NuxStatus::Ok,
                Err(failure) => publish_optional_failure(out_result, failure),
            }
        },
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_vulkan_free(
    renderer: *mut NuxAndroidVulkanRenderer,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if renderer.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(renderer, HandleKind::AndroidVulkanRenderer) {
            return status;
        }
        unsafe { drop(Box::from_raw(renderer)) };
        NuxStatus::Ok
    })
}

/// Returns a borrowed pointer to the frame's tightly packed RGBA8 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_android_vulkan_frame_data(
    frame: *const NuxAndroidVulkanFrame,
) -> *const u8 {
    ffi_guard(ptr::null(), || {
        let Ok(_call) = enter_handle(frame, HandleKind::AndroidVulkanFrame) else {
            return ptr::null();
        };
        unsafe { frame.as_ref() }.map_or(ptr::null(), |frame| frame.pixels.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_android_vulkan_frame_len(
    frame: *const NuxAndroidVulkanFrame,
) -> usize {
    ffi_guard(0, || {
        let Ok(_call) = enter_handle(frame, HandleKind::AndroidVulkanFrame) else {
            return 0;
        };
        unsafe { frame.as_ref() }.map_or(0, |frame| frame.pixels.len())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_android_vulkan_frame_width(
    frame: *const NuxAndroidVulkanFrame,
) -> u32 {
    ffi_guard(0, || {
        let Ok(_call) = enter_handle(frame, HandleKind::AndroidVulkanFrame) else {
            return 0;
        };
        unsafe { frame.as_ref() }.map_or(0, |frame| frame.width)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_android_vulkan_frame_height(
    frame: *const NuxAndroidVulkanFrame,
) -> u32 {
    ffi_guard(0, || {
        let Ok(_call) = enter_handle(frame, HandleKind::AndroidVulkanFrame) else {
            return 0;
        };
        unsafe { frame.as_ref() }.map_or(0, |frame| frame.height)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_android_vulkan_frame_row_stride_bytes(
    frame: *const NuxAndroidVulkanFrame,
) -> u32 {
    ffi_guard(0, || {
        let Ok(_call) = enter_handle(frame, HandleKind::AndroidVulkanFrame) else {
            return 0;
        };
        unsafe { frame.as_ref() }.map_or(0, |frame| frame.row_stride_bytes)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_android_vulkan_frame_pixel_format(
    frame: *const NuxAndroidVulkanFrame,
) -> NuxAndroidVulkanPixelFormat {
    ffi_guard(0, || {
        let Ok(_call) = enter_handle(frame, HandleKind::AndroidVulkanFrame) else {
            return 0;
        };
        if unsafe { frame.as_ref() }.is_some() {
            NUX_ANDROID_VULKAN_PIXEL_FORMAT_RGBA8_PREMULTIPLIED
        } else {
            0
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_android_vulkan_frame_free(
    frame: *mut NuxAndroidVulkanFrame,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if frame.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(frame, HandleKind::AndroidVulkanFrame) {
            return status;
        }
        unsafe { drop(Box::from_raw(frame)) };
        NuxStatus::Ok
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn surface_admission_and_pending_completion_preserve_exact_revision() {
        use crate::*;
        fn varint(bytes: &mut Vec<u8>, mut value: u64) {
            while value >= 128 {
                bytes.push((value as u8 & 127) | 128);
                value >>= 7;
            }
            bytes.push(value as u8);
        }
        let mut bytes = b"RIVE".to_vec();
        for value in [7, 2, 0, 0] { varint(&mut bytes, value); }
        for name in ["Backboard", "Artboard"] {
            varint(&mut bytes, nuxie_schema::definition_by_name(name).unwrap().type_key.int.into());
            bytes.push(0);
        }
        unsafe {
            let mut renderer = ptr::null_mut();
            let mut result = ptr::null_mut();
            let status = nux_renderer_new_android_vulkan(2, 2, &mut renderer, &mut result);
            if status != NuxStatus::Ok {
                let mut diagnostic: NuxCapiDiagnosticView = std::mem::zeroed();
                diagnostic.struct_size = std::mem::size_of::<NuxCapiDiagnosticView>() as u32;
                let message = if !result.is_null() &&
                    nux_capi_result_diagnostic(result, &mut diagnostic) == NuxStatus::Ok {
                    String::from_utf8_lossy(std::slice::from_raw_parts(
                        diagnostic.message.data.cast::<u8>(), diagnostic.message.len)).into_owned()
                } else { format!("{status:?}") };
                nux_capi_result_free(result);
                assert_ne!(std::env::var("NUXIE_REQUIRE_LIVE_VULKAN_TESTS").as_deref(), Ok("1"),
                    "Vulkan unavailable: {message}");
                return;
            }
            nux_capi_result_free(result);
            let mut file = ptr::null_mut();
            result = ptr::null_mut();
            assert_eq!(nux_file_import_android_vulkan(renderer, bytes.as_ptr(), bytes.len(),
                &NuxFileImportConfig::default(), &mut file, &mut result), NuxStatus::Ok);
            nux_capi_result_free(result);
            let mut artboard = ptr::null_mut();
            assert_eq!(nux_artboard_instance_new(file, 0, &mut artboard), NuxStatus::Ok);
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
            let revision = (&*player).artboard.render_revision.get();
            assert_eq!((&*player).artboard.presented_render_revision.get(), 0);
            let skipped = with_rendered_player(renderer, player, 0xff112233,
                NUX_ANDROID_VULKAN_RENDERER_FIT_NONE, |_, _| Ok(Some(42)),
                |_, _| panic!("unavailable admission must not render")).unwrap();
            assert_eq!(skipped, 42);
            assert_eq!((&*player).artboard.presented_render_revision.get(), 0);
            let invalid = with_rendered_player::<u32>(renderer, player, 0, 999,
                |_, _| panic!("fit validation must precede admission"),
                |_, _| unreachable!());
            assert!(invalid.is_err());
            let pixels = with_rendered_player(renderer, player, 0xff112233,
                NUX_ANDROID_VULKAN_RENDERER_FIT_NONE, |_, _| Ok(None),
                |sink, _| Ok((sink.finish()?, RenderDelivery::Completed))).unwrap();
            assert_eq!(pixels, [0x11, 0x22, 0x33, 0xff].repeat(4));
            assert_eq!((&*player).artboard.presented_render_revision.get(), revision);
            let mut other_artboard = ptr::null_mut();
            let mut other_player = ptr::null_mut();
            assert_eq!(nux_artboard_instance_new(file, 0, &mut other_artboard), NuxStatus::Ok);
            assert_eq!(nux_player_new_default(other_artboard, &mut other_player), NuxStatus::Ok);
            for mutate_while_pending in [false, true] {
                let occurrence = &(&*player).artboard;
                occurrence.invalidate_render().unwrap();
                let submitted_revision = occurrence.render_revision.get();
                let old_presented = occurrence.presented_render_revision.get();
                let submitted = with_rendered_player(renderer, player, 0xff112233,
                    NUX_ANDROID_VULKAN_RENDERER_FIT_NONE, |_, _| Ok(None),
                    |sink, _| { sink.finish()?; Ok((4, RenderDelivery::Submitted)) }).unwrap();
                assert_eq!(submitted, 4);
                assert_eq!(occurrence.presented_render_revision.get(), old_presented);
                for _ in 0..3 {
                    let polled = with_rendered_player(renderer, player, 0,
                        NUX_ANDROID_VULKAN_RENDERER_FIT_NONE,
                        |state, player| { state.validate_pending(player)?; Ok(Some(4)) },
                        |_, _| panic!("pending polls must not record another frame")).unwrap();
                    assert_eq!(polled, 4);
                }
                let wrong = with_rendered_player::<u32>(renderer, other_player, 0,
                    NUX_ANDROID_VULKAN_RENDERER_FIT_NONE,
                    |state, player| { state.complete_pending(player)?; Ok(Some(1)) },
                    |_, _| unreachable!()).unwrap_err();
                assert_eq!(wrong.status, NuxStatus::HandleMismatch);
                if mutate_while_pending { occurrence.invalidate_render().unwrap(); }
                let completed = with_rendered_player(renderer, player, 0,
                    NUX_ANDROID_VULKAN_RENDERER_FIT_NONE,
                    |state, player| { state.complete_pending(player)?; Ok(Some(1)) },
                    |_, _| panic!("completing a poll must not record another frame")).unwrap();
                assert_eq!(completed, 1);
                assert_eq!(occurrence.presented_render_revision.get(),
                    if mutate_while_pending { old_presented } else { submitted_revision });
                let duplicate = (&*renderer).state.borrow().complete_pending(&*player).unwrap_err();
                assert_eq!(duplicate.status, NuxStatus::RuntimeError);
            }
            let occurrence = &(&*player).artboard;
            occurrence.invalidate_render().unwrap();
            with_rendered_player(renderer, player, 0,
                NUX_ANDROID_VULKAN_RENDERER_FIT_NONE, |_, _| Ok(None),
                |sink, _| { sink.finish()?; Ok(((), RenderDelivery::Submitted)) }).unwrap();
            let old_presented = occurrence.presented_render_revision.get();
            assert!((&*renderer).state.borrow().pending.borrow().is_some());
            result = ptr::null_mut();
            assert_eq!(nux_renderer_android_vulkan_resize(renderer, 2, 2, &mut result), NuxStatus::Ok);
            nux_capi_result_free(result);
            assert!((&*renderer).state.borrow().pending.borrow().is_none());
            assert_eq!(occurrence.presented_render_revision.get(), old_presented);
            let mut cpu_frame = ptr::null_mut();
            assert_eq!(nux_renderer_android_vulkan_render_player(renderer, player, 0,
                NUX_ANDROID_VULKAN_RENDERER_FIT_NONE, &mut cpu_frame, ptr::null_mut()), NuxStatus::Ok);
            assert_eq!(occurrence.presented_render_revision.get(), occurrence.render_revision.get());
            nux_android_vulkan_frame_free(cpu_frame);
            nux_player_free(other_player);
            nux_artboard_instance_free(other_artboard);
            nux_player_free(player);
            nux_artboard_instance_free(artboard);
            nux_file_free(file);
            nux_renderer_android_vulkan_free(renderer);
        }
    }

    #[test]
    fn centered_contain_fit_scales_and_letterboxes_authored_bounds() {
        assert_eq!(NUX_ANDROID_VULKAN_RENDERER_FIT_NONE, 0);
        assert_eq!(NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER, 1);
        assert_eq!(
            centered_contain_transform((0.0, 0.0, 100.0, 50.0), (300, 300))
                .expect("landscape bounds fit"),
            Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 75.0])
        );
        assert_eq!(
            centered_contain_transform((10.0, -5.0, 100.0, 50.0), (300, 300))
                .expect("offset bounds fit"),
            Mat2D([3.0, 0.0, 0.0, 3.0, -30.0, 90.0])
        );
        assert!(centered_contain_transform((0.0, 0.0, 0.0, 50.0), (300, 300)).is_err());
    }

    #[test]
    fn frame_accessors_expose_owned_tightly_packed_pixels() {
        let frame = Box::into_raw(Box::new(NuxAndroidVulkanFrame {
            pixels: vec![1, 2, 3, 4, 5, 6, 7, 8].into_boxed_slice(),
            width: 2,
            height: 1,
            row_stride_bytes: 8,
        }));
        register_handle(
            frame,
            HandleKind::AndroidVulkanFrame,
            thread::current().id(),
        );
        assert_eq!(unsafe { nux_android_vulkan_frame_len(frame) }, 8);
        assert_eq!(unsafe { nux_android_vulkan_frame_width(frame) }, 2);
        assert_eq!(unsafe { nux_android_vulkan_frame_height(frame) }, 1);
        assert_eq!(
            unsafe { nux_android_vulkan_frame_row_stride_bytes(frame) },
            8
        );
        assert_eq!(
            unsafe { nux_android_vulkan_frame_pixel_format(frame) },
            NUX_ANDROID_VULKAN_PIXEL_FORMAT_RGBA8_PREMULTIPLIED
        );
        let data = unsafe { nux_android_vulkan_frame_data(frame) };
        assert_eq!(
            unsafe { std::slice::from_raw_parts(data, 8) },
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(
            unsafe { nux_android_vulkan_frame_free(frame) },
            NuxStatus::Ok
        );
    }

    #[test]
    fn solid_clear_frame_is_rgba8_with_premultiplied_alpha() {
        let Ok(factory) = NativeVulkanFactory::new(2, 2) else {
            assert_ne!(
                std::env::var_os("NUXIE_REQUIRE_LIVE_VULKAN_TESTS").as_deref(),
                Some(std::ffi::OsStr::new("1")),
                "required live Vulkan test resource is unavailable"
            );
            return;
        };
        let frame = factory
            .begin_frame(0x80402010, RenderMode::Msaa)
            .expect("begin solid clear frame");
        let pixels = frame.finish().expect("read solid clear frame");
        assert_eq!(pixels.len(), 2 * 2 * 4);
        assert_eq!(&pixels[..4], &[32, 16, 8, 128]);
    }
}
