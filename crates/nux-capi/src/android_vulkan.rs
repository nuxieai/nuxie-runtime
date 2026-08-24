//! Headless Android/Vulkan extension for the portable C ABI.
//!
//! Android windowing remains entirely outside this crate. The SDK's JNI shim
//! owns `ANativeWindow` and blits the owned CPU frame returned here.

use super::{
    HandleKind, NuxCapiResult, NuxPlayer, NuxStatus, PendingHandlePublication, RendererDomain,
    RendererDomainBinding, enter_handle, enter_occurrence, ffi_guard, ffi_guard_with_handle_result,
    ffi_guard_with_result, publish_result, register_handle, remove_handle,
};
use nuxie::{Mat2D, PersistentFactory, Renderer};
use nuxie_renderer::{NativeVulkanFactory, RenderMode, RendererError};
use std::cell::RefCell;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

pub type NuxAndroidVulkanRendererFit = u32;
/// Preserve authored artboard coordinates without applying a viewport fit.
pub const NUX_ANDROID_VULKAN_RENDERER_FIT_NONE: NuxAndroidVulkanRendererFit = 0;
/// Uniformly scale and center the authored artboard inside the output frame.
pub const NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER: NuxAndroidVulkanRendererFit = 1;

pub type NuxAndroidVulkanPixelFormat = u32;
/// Tightly packed, top-row-first RGBA8 UNORM with premultiplied alpha.
pub const NUX_ANDROID_VULKAN_PIXEL_FORMAT_RGBA8_PREMULTIPLIED: NuxAndroidVulkanPixelFormat = 1;

struct AndroidVulkanRendererState {
    factory: PersistentFactory<NativeVulkanFactory>,
    pixel_width: u32,
    pixel_height: u32,
}

/// Product-neutral headless Vulkan renderer. The handle and every frame it
/// returns are affine to the thread that created them.
pub struct NuxAndroidVulkanRenderer {
    state: RefCell<AndroidVulkanRendererState>,
    domain: Arc<RendererDomain>,
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
    ffi_guard_with_handle_result(out_renderer, out_result, HandleKind::Renderer, || {
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
                    factory: PersistentFactory::new(factory),
                    pixel_width,
                    pixel_height,
                }),
                domain,
            },
            HandleKind::Renderer,
        );
        register_handle(pending.handle, HandleKind::Renderer, thread::current().id());
        unsafe { *out_renderer = pending.finish() };
        publish_result(out_result, NuxStatus::Ok, "");
        NuxStatus::Ok
    })
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
        let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
            .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
        let renderer = unsafe { renderer.as_ref() }
            .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
        let replacement =
            NativeVulkanFactory::new(pixel_width, pixel_height).map_err(renderer_failure)?;
        let mut state = renderer
            .state
            .try_borrow_mut()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        *state.factory.borrow_mut() = replacement;
        state.pixel_width = pixel_width;
        state.pixel_height = pixel_height;
        Ok(())
    })
}

/// Drops renderer-owned resources from the player's retained artboard and
/// binds it to this renderer's current durable domain.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_renderer_android_vulkan_reset_player_domain(
    renderer: *const NuxAndroidVulkanRenderer,
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
        let _state = renderer
            .state
            .try_borrow()
            .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;
        let _occurrence_call = enter_occurrence(&player.artboard)
            .map_err(|status| ApiFailure::new(status, "player occurrence is unavailable"))?;
        let artboard = player.artboard.instance.try_borrow().map_err(|_| {
            ApiFailure::new(NuxStatus::ReentrantCall, "player occurrence is active")
        })?;
        artboard.reset_renderer();
        let generation = renderer.domain.generation.load(Ordering::Relaxed);
        *player.artboard.renderer_domain.borrow_mut() =
            Some(RendererDomainBinding::AndroidVulkan {
                domain: Arc::clone(&renderer.domain),
                generation,
            });
        player.artboard.observed_renderer_generation.set(generation);
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
                if fit != NUX_ANDROID_VULKAN_RENDERER_FIT_NONE
                    && fit != NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER
                {
                    return Err(ApiFailure::new(
                        NuxStatus::InvalidArgument,
                        "unknown Android Vulkan renderer fit",
                    ));
                }
                let _renderer_call = enter_handle(renderer, HandleKind::Renderer)
                    .map_err(|status| ApiFailure::new(status, "renderer handle is unavailable"))?;
                let _player_call = enter_handle(player, HandleKind::Player)
                    .map_err(|status| ApiFailure::new(status, "player handle is unavailable"))?;
                let renderer_ref = unsafe { renderer.as_ref() }
                    .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "renderer is null"))?;
                let player = unsafe { player.as_ref() }
                    .ok_or_else(|| ApiFailure::new(NuxStatus::NullArgument, "player is null"))?;
                let _occurrence_call = enter_occurrence(&player.artboard).map_err(|status| {
                    ApiFailure::new(status, "player occurrence is unavailable")
                })?;
                let mut state = renderer_ref
                    .state
                    .try_borrow_mut()
                    .map_err(|_| ApiFailure::new(NuxStatus::ReentrantCall, "renderer is active"))?;

                let generation = renderer_ref.domain.generation.load(Ordering::Relaxed);
                let existing_domain = { player.artboard.renderer_domain.borrow().clone() };
                match existing_domain {
                    Some(RendererDomainBinding::AndroidVulkan {
                        domain: bound_domain,
                        generation: bound_generation,
                    }) if bound_domain.id == renderer_ref.domain.id
                        && Arc::ptr_eq(&bound_domain, &renderer_ref.domain)
                        && bound_generation == generation => {}
                    None => {
                        *player.artboard.renderer_domain.borrow_mut() =
                            Some(RendererDomainBinding::AndroidVulkan {
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
                    .map_err(|status| {
                        ApiFailure::new(status, "player render revision overflowed")
                    })?;
                let rendered_revision = player.artboard.render_revision.get();
                let mut frame = state
                    .factory
                    .borrow()
                    .begin_frame(clear_color, RenderMode::Msaa)
                    .map_err(renderer_failure)?;
                {
                    let mut artboard = player.artboard.instance.try_borrow_mut().map_err(|_| {
                        ApiFailure::new(NuxStatus::ReentrantCall, "player occurrence is active")
                    })?;
                    if fit == NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER {
                        frame.transform(centered_contain_transform(
                            artboard.artboard_bounds(),
                            (state.pixel_width, state.pixel_height),
                        )?);
                    }
                    artboard
                        .draw(&mut state.factory, &mut frame)
                        .map_err(|error| {
                            ApiFailure::new(NuxStatus::RuntimeError, format!("{error:#}"))
                        })?;
                }
                let pixels = frame.finish().map_err(renderer_failure)?;
                let row_stride_bytes = state.pixel_width.checked_mul(4).ok_or_else(|| {
                    ApiFailure::new(NuxStatus::RuntimeError, "frame row stride overflowed")
                })?;
                let expected_len =
                    usize::try_from(u64::from(row_stride_bytes) * u64::from(state.pixel_height))
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
                player
                    .artboard
                    .acknowledge_presented(rendered_revision)
                    .map_err(|status| {
                        ApiFailure::new(
                            status,
                            "presented player revision no longer matches the rendered occurrence",
                        )
                    })?;
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
                Ok(())
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
        if let Err(status) = remove_handle(renderer, HandleKind::Renderer) {
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
