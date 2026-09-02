//! Exercise the real Apple CAPI import/player/drawable route, without replacing
//! its selected ORE context or replay host with a test implementation.
use super::{
    assert_result, operation, read_drawable_bgra, readable_layer, render, renderer, scheduling,
    string_view,
};
use nux_capi::*;
use objc2::rc::{Retained, autoreleasepool};
use std::ffi::c_void;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use runtime_test_support::authored_msl_gpu_canvas::{EXPECTED_PIXEL, HEIGHT, WIDTH, imported_file};

fn imported_player(renderer: *mut NuxRenderer, native_authority: bool) -> *mut NuxPlayer {
    let bytes = imported_file();
    let host = NuxHostCommandImportConfig {
        module_name: string_view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let config = NuxFileImportConfig {
        host_commands: &host,
        ..NuxFileImportConfig::default()
    };
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    // SAFETY: the shared repository fixture is the trusted exporter boundary
    // for these exact reviewed MSL/RSTB/Luau bytes. The comparison import uses
    // the same script authorization and bytes but grants no native-code proof.
    let status = unsafe {
        if native_authority {
            nux_file_import_metal_with_trusted_native_shaders(
                renderer,
                bytes.as_ptr(),
                bytes.len(),
                &config,
                &raw mut file,
                &raw mut result,
            )
        } else {
            nux_file_import_metal(
                renderer,
                bytes.as_ptr(),
                bytes.len(),
                &config,
                &raw mut file,
                &raw mut result,
            )
        }
    };
    unsafe { assert_result(result, NuxStatus::Ok) };
    assert_eq!(status, NuxStatus::Ok);
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
    assert!(scheduling(player, 0.0).render_required);
    // The player must retain the native File, scripts, recorder and resources
    // after the embedding releases both independent import handles.
    assert_eq!(
        unsafe { nux_artboard_instance_free(artboard) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    player
}

unsafe extern "C" fn completed(context: *mut c_void) {
    // SAFETY: one Arc strong reference is transferred into each operation;
    // the CAPI completion contract consumes it exactly once on its queue.
    let count = unsafe { Arc::from_raw(context.cast::<AtomicUsize>()) };
    count.fetch_add(1, Ordering::Release);
}

fn assert_pixels(pixels: &[u8], expected_bgra: [u8; 4], frame: usize) {
    assert_eq!(pixels.len(), (WIDTH * HEIGHT * 4) as usize);
    let mismatch = pixels
        .chunks_exact(4)
        .enumerate()
        .find(|(_, pixel)| *pixel != expected_bgra);
    assert!(
        mismatch.is_none(),
        "CAPI frame {frame} pixel mismatch: {mismatch:?}; expected {expected_bgra:?}"
    );
}

fn render_two_frames(native_authority: bool) {
    autoreleasepool(|_| {
        let renderer = renderer(WIDTH, HEIGHT);
        let player = imported_player(renderer, native_authority);
        let surface = readable_layer(renderer, WIDTH, HEIGHT);
        for frame in 0..2 {
            if frame != 0 {
                scheduling(player, 1.0 / 60.0);
            }
            let drawable = surface
                .nextDrawable()
                .expect("required live Metal drawable");
            let completion = Arc::new(AtomicUsize::new(0));
            let mut operation = operation(
                NUX_METAL_DRAWABLE_STATE_AVAILABLE,
                Retained::as_ptr(&drawable).cast_mut().cast(),
            );
            operation.clear_color = 0xff11_2233;
            operation.completion_context = Arc::into_raw(completion.clone()).cast_mut().cast();
            operation.completion_callback = Some(completed);
            let outcome = unsafe { render(renderer, player, operation, NuxStatus::Ok) };
            assert_eq!(outcome.disposition, NUX_RENDERER_DISPOSITION_PRESENTED);
            assert_eq!(outcome.health, NUX_RENDERER_HEALTH_HEALTHY);
            assert_eq!((outcome.pixel_width, outcome.pixel_height), (WIDTH, HEIGHT));
            let deadline = Instant::now() + Duration::from_secs(30);
            while completion.load(Ordering::Acquire) == 0 && Instant::now() < deadline {
                std::thread::yield_now();
            }
            assert_eq!(completion.load(Ordering::Acquire), 1, "frame completion");
            let pixels = read_drawable_bgra(&drawable, WIDTH, HEIGHT);
            if native_authority {
                assert!(
                    outcome.draw_calls > 0,
                    "the GPUCanvas image must reach the drawable"
                );
                let [red, green, blue, alpha] = EXPECTED_PIXEL;
                assert_pixels(&pixels, [blue, green, red, alpha], frame);
            } else {
                // e949 lookup returns nil when no permitted native shader can
                // be built. The generator's assertion is logged and leaves the
                // scripted drawable inert; the CAPI may still present its clear.
                assert_eq!(
                    outcome.draw_calls, 0,
                    "generic trust must not execute native GPU work"
                );
                assert_pixels(&pixels, [0x33, 0x22, 0x11, 0xff], frame);
            }
        }
        assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
        assert_eq!(unsafe { nux_renderer_free(renderer) }, NuxStatus::Ok);
    });
}

#[test]
fn trusted_gpu_canvas_replays_to_real_drawables_after_import_handles_are_released() {
    render_two_frames(true);
}

#[test]
fn generic_script_trust_cannot_render_native_gpu_canvas() {
    render_two_frames(false);
}
