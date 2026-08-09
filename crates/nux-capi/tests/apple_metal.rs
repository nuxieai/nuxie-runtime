#![cfg(all(feature = "apple-metal", any(target_os = "ios", target_os = "macos")))]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "Apple ABI integration tests use bounded fixture dimensions and explicit FFI assertions"
)]

use nux_capi::*;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CGSize;
use objc2_metal::{MTLDevice, MTLPixelFormat};
use objc2_quartz_core::CAMetalLayer;
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr;

#[cfg(feature = "scripting")]
#[path = "support/composed_import.rs"]
mod composed_import;
#[cfg(feature = "scripting")]
use composed_import::scripted_view_model_asset_fixture;

struct AppleDecodeProbe {
    pixels: Vec<u8>,
    calls: usize,
    retains: usize,
    releases: usize,
    nested_abi: u32,
}

unsafe extern "C" fn retain_decode_pixels(owner: *mut c_void) {
    let probe = unsafe { &mut *owner.cast::<AppleDecodeProbe>() };
    probe.retains += 1;
    probe.nested_abi = unsafe { nux_capi_abi_version() };
}

unsafe extern "C" fn release_decode_pixels(owner: *mut c_void) {
    let probe = unsafe { &mut *owner.cast::<AppleDecodeProbe>() };
    probe.releases += 1;
}

unsafe extern "C" fn decode_apple_image(
    context: *mut c_void,
    request: *const NuxImageDecodeRequest,
    out_image: *mut NuxDecodedImage,
) -> NuxAssetCallbackStatus {
    let probe = unsafe { &mut *context.cast::<AppleDecodeProbe>() };
    let request = unsafe { &*request };
    let encoded = unsafe { std::slice::from_raw_parts(request.encoded.data, request.encoded.len) };
    let decoded = nuxie_image_codec::decode_image_rgba(encoded).expect("fixture image decodes");
    probe.calls += 1;
    probe.pixels = decoded.pixels;
    unsafe {
        *out_image = NuxDecodedImage {
            width: decoded.width,
            height: decoded.height,
            row_bytes: decoded.width * 4,
            pixel_format: NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
            pixels: NuxRetainedBytes {
                data: probe.pixels.as_ptr(),
                len: probe.pixels.len(),
                owner: context,
                retain: Some(retain_decode_pixels),
                release: Some(release_decode_pixels),
                ..NuxRetainedBytes::default()
            },
            ..NuxDecodedImage::default()
        };
    }
    NUX_ASSET_CALLBACK_STATUS_OK
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    let path = PathBuf::from(
        std::env::var_os("NUX_RUNTIME_DIR")
            .or_else(|| std::env::var_os("RIVE_RUNTIME_DIR"))
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    std::fs::read(path).expect("read fixture")
}

unsafe fn assert_result(result: *mut NuxCapiResult, expected: NuxStatus) {
    assert!(!result.is_null());
    let mut status = NuxStatus::Ok;
    assert_eq!(
        unsafe { nux_capi_result_status(result, &raw mut status) },
        NuxStatus::Ok
    );
    assert_eq!(status, expected);
    assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
}

fn player_retaining_released_import_owners(fixture: &str, artboard_index: usize) -> *mut NuxPlayer {
    let bytes = fixture_bytes(fixture);
    let mut file = ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &raw mut file) },
        NuxStatus::Ok
    );
    let mut artboard = ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, artboard_index, &raw mut artboard) },
        NuxStatus::Ok
    );
    let mut player = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(artboard, &raw mut player) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_free(artboard) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    player
}

fn apple_asset_player(fixture: &str) -> (*mut NuxPlayer, Box<AppleDecodeProbe>) {
    let bytes = fixture_bytes(fixture);
    let mut probe = Box::new(AppleDecodeProbe {
        pixels: Vec::new(),
        calls: 0,
        retains: 0,
        releases: 0,
        nested_abi: NUX_CAPI_ABI_VERSION,
    });
    let hooks = NuxAppleAssetHooks {
        context: std::ptr::from_mut(probe.as_mut()).cast(),
        decode_image: Some(decode_apple_image),
        ..NuxAppleAssetHooks::default()
    };
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_with_apple_assets(
                bytes.as_ptr(),
                bytes.len(),
                &hooks,
                &raw mut file,
                &raw mut result,
            )
        },
        NuxStatus::Ok
    );
    unsafe { assert_result(result, NuxStatus::Ok) };
    let mut artboard = ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &raw mut artboard) },
        NuxStatus::Ok
    );
    let mut player = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(artboard, &raw mut player) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_free(artboard) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    (player, probe)
}

fn renderer(width: u32, height: u32) -> *mut NuxRenderer {
    let mut renderer = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_renderer_new_metal(width, height, &raw mut renderer, &raw mut result) },
        NuxStatus::Ok
    );
    unsafe { assert_result(result, NuxStatus::Ok) };
    renderer
}

fn layer(renderer: *mut NuxRenderer, width: u32, height: u32) -> Retained<CAMetalLayer> {
    let mut device_pointer = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_renderer_copy_metal_device(renderer, &raw mut device_pointer, &raw mut result)
        },
        NuxStatus::Ok
    );
    unsafe { assert_result(result, NuxStatus::Ok) };
    let device: Retained<ProtocolObject<dyn MTLDevice>> =
        unsafe { Retained::from_raw(device_pointer.cast()).expect("copied +1 MTLDevice") };
    let layer = CAMetalLayer::new();
    layer.setDevice(Some(&device));
    layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
    layer.setFramebufferOnly(true);
    layer.setDrawableSize(CGSize::new(width.into(), height.into()));
    layer.setMaximumDrawableCount(2);
    layer.setAllowsNextDrawableTimeout(true);
    layer
}

fn operation(state: NuxMetalDrawableState, drawable: *mut c_void) -> NuxMetalRenderOperation {
    NuxMetalRenderOperation {
        drawable_state: state,
        drawable,
        clear_color: 0xff11_2233,
        ..NuxMetalRenderOperation::default()
    }
}

unsafe fn render(
    renderer: *mut NuxRenderer,
    player: *mut NuxPlayer,
    mut operation: NuxMetalRenderOperation,
    expected: NuxStatus,
) -> NuxRendererOutcome {
    let mut outcome = NuxRendererOutcome::default();
    let mut result = ptr::dangling_mut();
    let status = unsafe {
        nux_renderer_render_player(
            renderer,
            player,
            &raw mut operation,
            &raw mut outcome,
            &raw mut result,
        )
    };
    assert_eq!(status, expected);
    if expected == NuxStatus::Ok {
        // The per-frame result slot is failure-only and allocation-free.
        assert!(result.is_null());
    } else {
        unsafe { assert_result(result, expected) };
    }
    outcome
}

fn scheduling(player: *mut NuxPlayer, elapsed_seconds: f32) -> NuxPlayerSchedulingInfo {
    let operation = NuxPlayerStep {
        elapsed_seconds,
        ..NuxPlayerStep::default()
    };
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_step(player, &operation, &raw mut result) },
        NuxStatus::Ok
    );
    let mut scheduling = NuxPlayerSchedulingInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_scheduling(result, &raw mut scheduling) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_step_result_free(result) },
        NuxStatus::Ok
    );
    scheduling
}

#[test]
fn native_c_renderer_lifecycle_preserves_player_domain_and_player_lifetimes() {
    autoreleasepool(|_| {
        let player = player_retaining_released_import_owners("smi_test.riv", 1);
        let first = renderer(4, 3);
        let second = renderer(4, 3);
        let initial_scheduling = scheduling(player, 0.0);
        assert!(initial_scheduling.render_required);

        // Caller-reported availability skips never bind the occurrence.
        let outcome = unsafe {
            render(
                first,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_TIMEOUT, ptr::null_mut()),
                NuxStatus::Ok,
            )
        };
        assert_eq!(
            outcome.disposition,
            NUX_RENDERER_DISPOSITION_SKIPPED_TIMEOUT
        );
        let after_timeout = scheduling(player, 1.0);
        assert!(after_timeout.render_required);
        assert_eq!(
            after_timeout.render_revision,
            initial_scheduling.render_revision
        );
        let outcome = unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_OCCLUDED, ptr::null_mut()),
                NuxStatus::Ok,
            )
        };
        assert_eq!(
            outcome.disposition,
            NUX_RENDERER_DISPOSITION_SKIPPED_OCCLUDED
        );

        let first_layer = layer(first, 4, 3);
        let first_drawable = first_layer.nextDrawable().expect("first drawable");
        let first_pointer = Retained::as_ptr(&first_drawable).cast_mut().cast();
        let outcome = unsafe {
            render(
                first,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, first_pointer),
                NuxStatus::Ok,
            )
        };
        assert_eq!(outcome.disposition, NUX_RENDERER_DISPOSITION_PRESENTED);
        let after_present = scheduling(player, 1.0);
        assert!(!after_present.render_required);
        assert_eq!(
            after_present.render_revision,
            initial_scheduling.render_revision
        );
        assert_eq!(unsafe { nux_renderer_free(first) }, NuxStatus::Ok);

        // The player's retained domain outlives its public renderer handle; a
        // different domain stays rejected until the explicit player reset.
        let second_layer = layer(second, 4, 3);
        let second_drawable = second_layer.nextDrawable().expect("second drawable");
        let second_pointer = Retained::as_ptr(&second_drawable).cast_mut().cast();
        unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, second_pointer),
                NuxStatus::HandleMismatch,
            );
        }
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reset_player_domain(second, player, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        let after_domain_reset = scheduling(player, 1.0);
        assert!(after_domain_reset.render_required);
        assert!(after_domain_reset.render_revision > after_present.render_revision);

        let second_drawable = second_layer.nextDrawable().expect("reset drawable");
        let second_pointer = Retained::as_ptr(&second_drawable).cast_mut().cast();
        let outcome = unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, second_pointer),
                NuxStatus::Ok,
            )
        };
        assert_eq!(outcome.disposition, NUX_RENDERER_DISPOSITION_PRESENTED);
        let before_reattach = scheduling(player, 1.0);
        assert!(!before_reattach.render_required);

        // A live drawable with incompatible texture dimensions is rejected
        // deterministically before presentation.
        let wrong_layer = layer(second, 5, 3);
        let wrong_drawable = wrong_layer.nextDrawable().expect("wrong-sized drawable");
        let wrong_pointer = Retained::as_ptr(&wrong_drawable).cast_mut().cast();
        unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, wrong_pointer),
                NuxStatus::InvalidArgument,
            );
        }

        let mut control_outcome = NuxRendererOutcome::default();
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_detach(second, &raw mut control_outcome, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reset_player_domain(second, player, &raw mut result) },
            NuxStatus::InvalidArgument
        );
        unsafe { assert_result(result, NuxStatus::InvalidArgument) };

        result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_renderer_reattach(second, 4, 3, &raw mut control_outcome, &raw mut result)
            },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        assert_eq!(
            control_outcome.disposition,
            NUX_RENDERER_DISPOSITION_RECREATED
        );
        assert_eq!(
            unsafe { nux_player_acknowledge_presented(player, before_reattach.render_revision) },
            NuxStatus::HandleMismatch,
            "reattach invalidates an in-flight occurrence revision"
        );

        let replacement_layer = layer(second, 4, 3);
        let drawable = replacement_layer
            .nextDrawable()
            .expect("replacement drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::HandleMismatch,
            );
        }
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reset_player_domain(second, player, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };

        // Zero-sized surfaces are bounded without drawable acquisition.
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_resize(second, 0, 0, &raw mut control_outcome, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        let outcome = unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_TIMEOUT, ptr::null_mut()),
                NuxStatus::Ok,
            )
        };
        assert_eq!(
            outcome.disposition,
            NUX_RENDERER_DISPOSITION_SKIPPED_ZERO_SIZE
        );
        assert!(scheduling(player, 1.0).render_required);

        assert_eq!(unsafe { nux_renderer_free(second) }, NuxStatus::Ok);
        assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
    });
}

#[test]
fn embedded_images_redecode_after_renderer_migration_and_reattach() {
    autoreleasepool(|_| {
        let player = player_retaining_released_import_owners("walle.riv", 0);
        let first = renderer(32, 32);
        let first_layer = layer(first, 32, 32);
        let drawable = first_layer.nextDrawable().expect("first image drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        let first_outcome = unsafe {
            render(
                first,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::Ok,
            )
        };
        assert_eq!(
            first_outcome.disposition,
            NUX_RENDERER_DISPOSITION_PRESENTED
        );
        assert_eq!(unsafe { nux_renderer_free(first) }, NuxStatus::Ok);

        let second = renderer(32, 32);
        let second_layer = layer(second, 32, 32);
        let drawable = second_layer
            .nextDrawable()
            .expect("foreign-domain image drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::HandleMismatch,
            );
        }

        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reset_player_domain(second, player, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        let drawable = second_layer
            .nextDrawable()
            .expect("migrated image drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        let migrated = unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::Ok,
            )
        };
        assert_eq!(migrated.disposition, NUX_RENDERER_DISPOSITION_PRESENTED);
        assert!(
            migrated.draw_calls > 0,
            "embedded image scene must submit draw work"
        );

        let mut control = NuxRendererOutcome::default();
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_detach(second, &raw mut control, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reattach(second, 32, 32, &raw mut control, &raw mut result,) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        let replacement_layer = layer(second, 32, 32);
        let drawable = replacement_layer
            .nextDrawable()
            .expect("reattached image drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::HandleMismatch,
            );
        }
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reset_player_domain(second, player, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        let drawable = replacement_layer
            .nextDrawable()
            .expect("redecoded image drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        let redecoded = unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::Ok,
            )
        };
        assert_eq!(redecoded.disposition, NUX_RENDERER_DISPOSITION_PRESENTED);
        assert!(redecoded.draw_calls > 0);

        assert_eq!(unsafe { nux_renderer_free(second) }, NuxStatus::Ok);
        assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
    });
}

#[test]
fn apple_decoded_cpu_pixels_survive_file_release_domain_reset_and_reattach() {
    autoreleasepool(|_| {
        let (player, probe) = apple_asset_player("in_band_asset.riv");
        assert_eq!(probe.calls, 1);
        assert_eq!((probe.retains, probe.releases), (1, 1));
        assert_eq!(
            probe.nested_abi, 0,
            "callbacks cannot reenter scalar exports"
        );

        let first = renderer(64, 64);
        let first_layer = layer(first, 64, 64);
        let drawable = first_layer
            .nextDrawable()
            .expect("first Apple asset drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        let first_outcome = unsafe {
            render(
                first,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::Ok,
            )
        };
        assert!(first_outcome.draw_calls > 0);
        assert_eq!(unsafe { nux_renderer_free(first) }, NuxStatus::Ok);

        let second = renderer(64, 64);
        let second_layer = layer(second, 64, 64);
        let drawable = second_layer
            .nextDrawable()
            .expect("foreign domain drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::HandleMismatch,
            );
        }
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reset_player_domain(second, player, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        let drawable = second_layer.nextDrawable().expect("reset domain drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        let reset = unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::Ok,
            )
        };
        assert!(reset.draw_calls > 0);

        let mut control = NuxRendererOutcome::default();
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_detach(second, &raw mut control, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reattach(second, 64, 64, &raw mut control, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_renderer_reset_player_domain(second, player, &raw mut result) },
            NuxStatus::Ok
        );
        unsafe { assert_result(result, NuxStatus::Ok) };
        let replacement_layer = layer(second, 64, 64);
        let drawable = replacement_layer
            .nextDrawable()
            .expect("reattached drawable");
        let pointer = Retained::as_ptr(&drawable).cast_mut().cast();
        let reattached = unsafe {
            render(
                second,
                player,
                operation(NUX_METAL_DRAWABLE_STATE_AVAILABLE, pointer),
                NuxStatus::Ok,
            )
        };
        assert!(reattached.draw_calls > 0);
        assert_eq!(
            probe.calls, 1,
            "domain changes reupload canonical CPU pixels"
        );
        assert_eq!((probe.retains, probe.releases), (1, 1));
        assert_eq!(unsafe { nux_renderer_free(second) }, NuxStatus::Ok);
        assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
    });
}

#[test]
fn every_dual_output_renderer_api_rejects_aliasing_before_mutation() {
    let mut shared = ptr::dangling_mut::<NuxCapiResult>();
    let slot = &raw mut shared;
    assert_eq!(
        unsafe { nux_renderer_new_metal(1, 1, slot.cast(), slot) },
        NuxStatus::InvalidArgument
    );
    assert!(shared.is_null());

    let renderer = renderer(2, 2);
    macro_rules! assert_alias_rejected {
        ($call:expr) => {{
            shared = ptr::dangling_mut();
            assert_eq!($call, NuxStatus::InvalidArgument);
            assert!(shared.is_null());
        }};
    }
    assert_alias_rejected!(unsafe { nux_renderer_copy_metal_device(renderer, slot.cast(), slot) });
    assert_alias_rejected!(unsafe { nux_renderer_info(renderer, slot.cast(), slot) });
    assert_alias_rejected!(unsafe { nux_renderer_resize(renderer, 2, 2, slot.cast(), slot) });
    assert_alias_rejected!(unsafe { nux_renderer_detach(renderer, slot.cast(), slot) });
    assert_alias_rejected!(unsafe { nux_renderer_reattach(renderer, 2, 2, slot.cast(), slot) });
    assert_eq!(unsafe { nux_renderer_free(renderer) }, NuxStatus::Ok);
}

#[cfg(feature = "scripting")]
fn string_view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

#[cfg(feature = "scripting")]
fn scripted_asset_import(
    bytes: &[u8],
    host: &NuxHostCommandImportConfig,
    probe: &mut AppleDecodeProbe,
) -> *mut NuxFile {
    let hooks = NuxAppleAssetHooks {
        context: std::ptr::from_mut(probe).cast(),
        decode_image: Some(decode_apple_image),
        ..NuxAppleAssetHooks::default()
    };
    let expected = [
        NuxExpectedFileAssetDescriptor {
            ordinal: 0,
            kind: NUX_FILE_ASSET_KIND_SCRIPT,
            has_authored_id: 1,
            authored_id: 0,
            name: string_view("GenericHostChanges"),
            file_extension: string_view("lua"),
            is_embedded: 1,
            has_contents_record: 1,
            ..NuxExpectedFileAssetDescriptor::default()
        },
        NuxExpectedFileAssetDescriptor {
            ordinal: 1,
            kind: NUX_FILE_ASSET_KIND_IMAGE,
            has_authored_id: 1,
            authored_id: 7,
            name: string_view("pixel.png"),
            file_extension: string_view("png"),
            is_embedded: 1,
            has_contents_record: 1,
            required_provider_flags: NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE,
            ..NuxExpectedFileAssetDescriptor::default()
        },
    ];
    let config = NuxFileImportConfig {
        host_commands: host,
        apple_assets: &hooks,
        expected_assets: expected.as_ptr(),
        expected_asset_count: expected.len(),
        ..NuxFileImportConfig::default()
    };
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_configured(bytes.as_ptr(), bytes.len(), &config, &mut file, &mut result)
        },
        NuxStatus::Ok
    );
    unsafe { assert_result(result, NuxStatus::Ok) };
    file
}

#[cfg(feature = "scripting")]
fn step_scripted_player(
    player: *mut NuxPlayer,
    pointers: &[NuxPlayerPointerEvent],
    correlation_id: u64,
    elapsed_seconds: f32,
) -> *mut NuxPlayerStepResult {
    let step = NuxPlayerStep {
        pointers: pointers.as_ptr(),
        pointer_count: pointers.len(),
        elapsed_seconds,
        correlation_id,
        ..NuxPlayerStep::default()
    };
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_step(player, &step, &mut result) },
        NuxStatus::Ok
    );
    result
}

#[cfg(feature = "scripting")]
#[test]
fn configured_script_and_asset_import_hydrates_steps_and_renders_on_metal() {
    autoreleasepool(|_| {
        let bytes = scripted_view_model_asset_fixture(
            br#"
                local bridge = require("bridge")
                return function(context)
                    return {
                        init = function(_self) return true end,
                        performAction = function(_self, _invocation)
                            local root = context:viewModel()
                            if root ~= nil then
                                root.amount.value = 10
                                root.amount.value = 20
                            end
                            bridge.command("performed", nil)
                        end,
                    }
                end
            "#,
        );
        let host = NuxHostCommandImportConfig {
            module_name: string_view("bridge"),
            ..NuxHostCommandImportConfig::default()
        };
        let mut probe = AppleDecodeProbe {
            pixels: Vec::new(),
            calls: 0,
            retains: 0,
            releases: 0,
            nested_abi: NUX_CAPI_ABI_VERSION,
        };
        let file = scripted_asset_import(&bytes, &host, &mut probe);
        assert_eq!(probe.calls, 1);
        assert_eq!((probe.retains, probe.releases), (1, 1));
        assert_eq!(probe.nested_abi, 0, "callback reentry is rejected");

        let mut artboard = ptr::null_mut();
        assert_eq!(
            unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
            NuxStatus::Ok
        );
        let mut view_model = ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut view_model) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_player_new_state_machine_named(
                    artboard,
                    string_view("HostCommands"),
                    &mut player,
                )
            },
            NuxStatus::Ok
        );
        let renderer = renderer(100, 100);
        let surface = layer(renderer, 100, 100);
        let drawable = surface.nextDrawable().expect("Metal drawable");
        let outcome = unsafe {
            render(
                renderer,
                player,
                operation(
                    NUX_METAL_DRAWABLE_STATE_AVAILABLE,
                    Retained::as_ptr(&drawable).cast_mut().cast(),
                ),
                NuxStatus::Ok,
            )
        };
        assert!(outcome.draw_calls > 0);

        let initialized = step_scripted_player(player, &[], 0, 0.0);
        assert_eq!(
            unsafe { nux_player_step_result_free(initialized) },
            NuxStatus::Ok
        );
        let click = [
            NuxPlayerPointerEvent {
                kind: NUX_PLAYER_POINTER_KIND_DOWN,
                x: 50.0,
                y: 50.0,
                pointer_id: 0,
                timestamp_seconds: 0.0,
            },
            NuxPlayerPointerEvent {
                kind: NUX_PLAYER_POINTER_KIND_UP,
                x: 50.0,
                y: 50.0,
                pointer_id: 0,
                timestamp_seconds: 0.0,
            },
        ];
        let stepped = step_scripted_player(player, &click, 44, 0.016);
        let mut info = NuxPlayerStepInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_info(stepped, &mut info) },
            NuxStatus::Ok
        );
        assert_eq!(info.host_command_count, 1);
        assert_eq!(info.view_model_change_count, 2);

        assert_eq!(
            unsafe { nux_player_step_result_free(stepped) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { nux_renderer_free(renderer) }, NuxStatus::Ok);
        assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
        assert_eq!(
            unsafe { nux_artboard_instance_free(artboard) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_view_model_instance_free(view_model) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    });
}
