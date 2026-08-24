//! Narrow Android/WebGPU extension for the portable C ABI.
//!
//! Mirrors the Apple/Metal extension's ownership rules over the renderer's
//! platform-agnostic presentation seam: the platform (Kotlin) owns the
//! `ANativeWindow` and its lifecycle; this module owns surface
//! configuration, frame acquisition, GPU submission, and presentation.
//! The backend is whatever wgpu selects for the device (Vulkan on modern
//! Android, GLES otherwise); the native renderer can replace this
//! presentation path later without changing the C surface.
//!
//! External image/font asset factories are not wired yet (the Apple asset
//! adapter is platform-specific); artboards draw with the plain factory.

use super::{
    HandleKind, NuxCapiResult, NuxPlayer, NuxStatus, RendererDomain, RendererDomainBinding,
    enter_handle, enter_occurrence, ffi_guard, ffi_guard_with_handle_result,
    ffi_guard_with_result, publish_result, register_handle, remove_handle,
};
use nuxie::{Mat2D, PersistentFactory, Renderer};
use nuxie_renderer::{
    RenderMode, WgpuFactory, WgpuPresentationAcquireError, WgpuPresentationAlpha,
    WgpuPresentationSurface,
};
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};
use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

pub type NuxAndroidRenderDisposition = u32;
pub const NUX_ANDROID_RENDER_DISPOSITION_NONE: NuxAndroidRenderDisposition = 0;
pub const NUX_ANDROID_RENDER_DISPOSITION_PRESENTED: NuxAndroidRenderDisposition = 1;
pub const NUX_ANDROID_RENDER_DISPOSITION_SKIPPED_ACQUIRE: NuxAndroidRenderDisposition = 2;
pub const NUX_ANDROID_RENDER_DISPOSITION_SURFACE_OUTDATED: NuxAndroidRenderDisposition = 3;

pub type NuxAndroidRendererFit = u32;
/// Preserve authored artboard coordinates without applying a viewport fit.
pub const NUX_ANDROID_RENDERER_FIT_NONE: NuxAndroidRendererFit = 0;
/// Scale and center the artboard inside the surface, preserving aspect.
pub const NUX_ANDROID_RENDERER_FIT_CONTAIN_CENTER: NuxAndroidRendererFit = 1;

static NEXT_ANDROID_RENDERER_DOMAIN_ID: AtomicU64 = AtomicU64::new(1);

fn allocate_renderer_domain() -> Result<Arc<RendererDomain>, ApiFailure> {
    let id = NEXT_ANDROID_RENDERER_DOMAIN_ID
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

/// Borrowed `ANativeWindow*` supplied by the platform. The platform contract
/// keeps the window alive while a renderer created from it exists;
/// `recreate_surface` re-wraps the current window after destruction.
struct AndroidWindowTarget {
    window: NonNull<c_void>,
}

// SAFETY: the pointer is only consumed by wgpu surface creation, and the
// platform contract confines the renderer handle to one thread (the pinned
// runtime lane) while keeping the window alive for the surface's lifetime.
unsafe impl Send for AndroidWindowTarget {}
unsafe impl Sync for AndroidWindowTarget {}

impl HasWindowHandle for AndroidWindowTarget {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = AndroidNdkWindowHandle::new(self.window);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(handle)) })
    }
}

impl HasDisplayHandle for AndroidWindowTarget {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Android(AndroidDisplayHandle::new()))
        })
    }
}

struct AndroidRendererState {
    factory: PersistentFactory<WgpuFactory>,
    surface: WgpuPresentationSurface,
    pixel_width: u32,
    pixel_height: u32,
}

pub struct NuxAndroidRenderer {
    state: RefCell<AndroidRendererState>,
    domain: Arc<RendererDomain>,
}

fn acquire_disposition(error: WgpuPresentationAcquireError) -> NuxAndroidRenderDisposition {
    match error {
        WgpuPresentationAcquireError::Timeout | WgpuPresentationAcquireError::Occluded => {
            NUX_ANDROID_RENDER_DISPOSITION_SKIPPED_ACQUIRE
        }
        WgpuPresentationAcquireError::Outdated
        | WgpuPresentationAcquireError::Lost
        | WgpuPresentationAcquireError::Validation => {
            NUX_ANDROID_RENDER_DISPOSITION_SURFACE_OUTDATED
        }
    }
}

/// Creates a WebGPU renderer presenting into the given `ANativeWindow`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_new_android_wgpu(
    window: *mut c_void,
    pixel_width: u32,
    pixel_height: u32,
    out_renderer: *mut *mut NuxAndroidRenderer,
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
        let Some(window) = NonNull::new(window) else {
            publish_result(out_result, NuxStatus::NullArgument, "window is null");
            return NuxStatus::NullArgument;
        };
        if pixel_width == 0 || pixel_height == 0 {
            publish_result(
                out_result,
                NuxStatus::InvalidArgument,
                "surface extent must be non-zero",
            );
            return NuxStatus::InvalidArgument;
        }
        let domain = match allocate_renderer_domain() {
            Ok(domain) => domain,
            Err(failure) => {
                publish_result(out_result, failure.status, failure.message);
                return failure.status;
            }
        };
        let factory =
            match WgpuFactory::new_with_mode(pixel_width, pixel_height, RenderMode::Msaa) {
                Ok(factory) => factory,
                Err(error) => {
                    publish_result(out_result, NuxStatus::RuntimeError, error.to_string());
                    return NuxStatus::RuntimeError;
                }
            };
        let surface = match factory.create_presentation_surface(
            AndroidWindowTarget { window },
            pixel_width,
            pixel_height,
            WgpuPresentationAlpha::Premultiplied,
        ) {
            Ok(surface) => surface,
            Err(error) => {
                publish_result(out_result, NuxStatus::RuntimeError, error.to_string());
                return NuxStatus::RuntimeError;
            }
        };
        let renderer = Box::into_raw(Box::new(NuxAndroidRenderer {
            state: RefCell::new(AndroidRendererState {
                factory: PersistentFactory::new(factory),
                surface,
                pixel_width,
                pixel_height,
            }),
            domain,
        }));
        unsafe { *out_renderer = renderer };
        register_handle(renderer, HandleKind::Renderer, thread::current().id());
        publish_result(out_result, NuxStatus::Ok, "");
        NuxStatus::Ok
    })
}

/// Reconfigures the presentation extent after the platform window resizes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_wgpu_resize(
    renderer: *mut NuxAndroidRenderer,
    pixel_width: u32,
    pixel_height: u32,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_result(out_result, || {
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        if pixel_width == 0 || pixel_height == 0 {
            return Err(ApiFailure::new(
                NuxStatus::InvalidArgument,
                "surface extent must be non-zero",
            ));
        }
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        let AndroidRendererState {
            factory,
            surface,
            pixel_width: width_slot,
            pixel_height: height_slot,
        } = &mut *state;
        surface.configure(pixel_width, pixel_height);
        factory
            .borrow_mut()
            .resize(pixel_width, pixel_height)
            .map_err(|error| ApiFailure::new(NuxStatus::RuntimeError, error.to_string()))?;
        *width_slot = pixel_width;
        *height_slot = pixel_height;
        Ok(())
    })
}

/// Re-wraps a recreated `ANativeWindow` after the platform surface was
/// destroyed and recreated, preserving renderer and session state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_wgpu_recreate_surface(
    renderer: *mut NuxAndroidRenderer,
    window: *mut c_void,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_result(out_result, || {
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let window = NonNull::new(window)
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "window is null"))?;
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        state
            .surface
            .recreate(AndroidWindowTarget { window })
            .map_err(|error| ApiFailure::new(NuxStatus::RuntimeError, error.to_string()))
    })
}

/// Renders the player's retained artboard into the next platform frame and
/// presents it. `out_disposition` reports presented/skipped/outdated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_wgpu_render_player(
    renderer: *mut NuxAndroidRenderer,
    player: *mut NuxPlayer,
    clear_color: u32,
    fit: NuxAndroidRendererFit,
    out_disposition: *mut NuxAndroidRenderDisposition,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    ffi_guard_with_result(out_result, || {
        if !out_disposition.is_null() {
            unsafe { *out_disposition = NUX_ANDROID_RENDER_DISPOSITION_NONE };
        }
        if fit != NUX_ANDROID_RENDERER_FIT_NONE && fit != NUX_ANDROID_RENDERER_FIT_CONTAIN_CENTER {
            return with_optional_failure_result(out_result, || {
                Err(ApiFailure::new(NuxStatus::InvalidArgument, "unknown fit"))
            });
        }
        with_optional_failure_result(out_result, || {
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

            // Domain ownership mirrors Metal: a player renders through
            // exactly one renderer domain until explicitly reset.
            let generation = renderer_ref.domain.generation.load(Ordering::Relaxed);
            let existing_domain = { player.artboard.renderer_domain.borrow().clone() };
            match existing_domain {
                Some(RendererDomainBinding::AndroidWgpu {
                    domain: bound_domain,
                    generation: bound_generation,
                }) if bound_domain.id == renderer_ref.domain.id
                    && Arc::ptr_eq(&bound_domain, &renderer_ref.domain)
                    && bound_generation == generation => {}
                None => {
                    *player.artboard.renderer_domain.borrow_mut() =
                        Some(RendererDomainBinding::AndroidWgpu {
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

            let presentation_frame = match state.surface.acquire() {
                Ok(frame) => frame,
                Err(error) => {
                    if !out_disposition.is_null() {
                        unsafe { *out_disposition = acquire_disposition(error) };
                    }
                    return Ok(());
                }
            };

            let AndroidRendererState {
                factory,
                pixel_width,
                pixel_height,
                ..
            } = &mut *state;
            let mut frame = factory.borrow().begin_frame(clear_color);
            {
                let mut artboard = player.artboard.instance.try_borrow_mut().map_err(|_| {
                    ApiFailure::new(NuxStatus::ReentrantCall, "player occurrence is active")
                })?;
                if fit == NUX_ANDROID_RENDERER_FIT_CONTAIN_CENTER {
                    frame.transform(centered_contain_transform(
                        artboard.artboard_bounds(),
                        (*pixel_width, *pixel_height),
                    )?);
                }
                artboard
                    .draw(factory, &mut frame)
                    .map_err(|error| ApiFailure::new(NuxStatus::RuntimeError, error.to_string()))?;
            }

            pollster::block_on(presentation_frame.present(frame))
                .map_err(|error| ApiFailure::new(NuxStatus::RuntimeError, error.to_string()))?;
            player
                .artboard
                .acknowledge_presented(rendered_revision)
                .map_err(|status| {
                    ApiFailure::new(
                        status,
                        "presented player revision no longer matches the rendered occurrence",
                    )
                })?;
            if !out_disposition.is_null() {
                unsafe { *out_disposition = NUX_ANDROID_RENDER_DISPOSITION_PRESENTED };
            }
            Ok(())
        })
    })
}

/// Resets a player's Android renderer-domain binding so it can bind to a new
/// renderer (mirror of the Metal `nux_renderer_reset_player_domain`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_wgpu_reset_player_domain(
    player: *mut NuxPlayer,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    with_result(out_result, || {
        let _player_call = enter_handle(player, HandleKind::Player)
            .map_err(|status| ApiFailure::new(status, "player handle is unavailable"))?;
        let player = unsafe { player.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "player is null"))?;
        let _occurrence_call = enter_occurrence(&player.artboard)
            .map_err(|status| ApiFailure::new(status, "player occurrence is unavailable"))?;
        let artboard = player.artboard.instance.try_borrow().map_err(|_| {
            ApiFailure::new(NuxStatus::ReentrantCall, "player occurrence is active")
        })?;
        artboard.reset_renderer();
        *player.artboard.renderer_domain.borrow_mut() = None;
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

/// Releases the renderer. Bound players must be reset before binding again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_wgpu_free(
    renderer: *mut NuxAndroidRenderer,
) -> NuxStatus {
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
