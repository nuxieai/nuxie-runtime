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

#[test]
fn native_c_renderer_lifecycle_preserves_player_domain_and_player_lifetimes() {
    autoreleasepool(|_| {
        let player = player_retaining_released_import_owners("smi_test.riv", 1);
        let first = renderer(4, 3);
        let second = renderer(4, 3);

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
