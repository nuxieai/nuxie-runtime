//! Session-local probe: does the android_vulkan capi arm render CONTENT?
#![cfg(feature = "android-vulkan")]

use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use nux_capi::*;

fn live_vulkan_test_required() -> bool {
    std::env::var_os("NUXIE_REQUIRE_LIVE_VULKAN_TESTS").as_deref()
        == Some(std::ffi::OsStr::new("1"))
}

fn probe_fixture_path() -> Option<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("NUX_PROBE_RIV") {
        return Some(path.into());
    }
    let repository_fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/flow/data_binding_test.riv");
    repository_fixture.is_file().then_some(repository_fixture)
}

// Minimal artboard with two missing external images. Keeping the fixture inline
// makes the hook-path regression independent of the optional live-test corpus.
const EXTERNAL_IMAGE_ARTBOARD: &[u8] = &[
    0x52, 0x49, 0x56, 0x45, 0x07, 0x00, 0xc5, 0x1b, 0xcb, 0x01, 0xcc, 0x01, 0xce, 0x01, 0xcf, 0x01,
    0xd0, 0x01, 0x00, 0x81, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x17, 0x00, 0x69, 0xcb, 0x01,
    0x09, 0x77, 0x61, 0x6c, 0x6c, 0x65, 0x2e, 0x6a, 0x70, 0x67, 0xcc, 0x01, 0xf2, 0x02, 0xcf, 0x01,
    0x00, 0x00, 0xf6, 0x43, 0xd0, 0x01, 0x00, 0x00, 0xfa, 0x43, 0x00, 0x69, 0xcb, 0x01, 0x07, 0x65,
    0x76, 0x65, 0x2e, 0x70, 0x6e, 0x67, 0xcc, 0x01, 0xbd, 0x02, 0xcf, 0x01, 0x00, 0x40, 0x7d, 0x44,
    0xd0, 0x01, 0x00, 0x00, 0x61, 0x44, 0x00, 0x01, 0x07, 0xbc, 0xa4, 0xa1, 0x44, 0x08, 0x00, 0x00,
    0xfa, 0x43, 0x04, 0x0c, 0x4e, 0x65, 0x77, 0x20, 0x41, 0x72, 0x74, 0x62, 0x6f, 0x61, 0x72, 0x64,
    0x00, 0x64, 0x04, 0x05, 0x77, 0x61, 0x6c, 0x6c, 0x65, 0x05, 0x00, 0x0d, 0x9d, 0x68, 0x0d, 0x44,
    0x0e, 0x01, 0x00, 0x7e, 0x43, 0xce, 0x01, 0x00, 0x00, 0x64, 0x04, 0x09, 0x65, 0x76, 0x65, 0x5f,
    0x72, 0x69, 0x67, 0x68, 0x74, 0x05, 0x00, 0x0f, 0xda, 0x0f, 0x49, 0xc0, 0x10, 0xdf, 0x0d, 0x86,
    0x3f, 0x11, 0x00, 0x00, 0x80, 0xbf, 0x0d, 0x4b, 0x0a, 0x5d, 0x44, 0x0e, 0xfe, 0xff, 0x75, 0x43,
    0xce, 0x01, 0x01, 0x00, 0x64, 0x04, 0x08, 0x65, 0x76, 0x65, 0x5f, 0x6c, 0x65, 0x66, 0x74, 0x05,
    0x00, 0x0d, 0xdc, 0x89, 0x86, 0x43, 0x0e, 0x28, 0x91, 0x6a, 0x43, 0xce, 0x01, 0x01, 0x00, 0x12,
    0x05, 0x05, 0x25, 0x31, 0x31, 0x31, 0xff, 0x00, 0x14, 0x05, 0x00, 0x00, 0x1c, 0x00, 0x1f, 0x37,
    0x0b, 0x41, 0x6e, 0x69, 0x6d, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x31, 0x3b, 0x02, 0x00, 0x19,
    0x33, 0x03, 0x00, 0x1a, 0x35, 0x0f, 0x00, 0x1e, 0x44, 0x02, 0x45, 0x06, 0x00, 0x1e, 0x43, 0x3c,
    0x44, 0x01, 0x46, 0xdb, 0x0f, 0x49, 0x40, 0x00,
];

const ENCODED_IMAGE: &[u8] = include_bytes!("fixtures/external-image.png");
const DECODED_PIXELS: &[u8] = &[0xff; 4 * 2 * 4];

#[derive(Default)]
struct AssetHookProbe {
    lookups: AtomicUsize,
    decodes: AtomicUsize,
    retains: AtomicUsize,
    releases: AtomicUsize,
}

unsafe extern "C" fn retain_asset_bytes(owner: *mut c_void) {
    unsafe { &*owner.cast::<AssetHookProbe>() }
        .retains
        .fetch_add(1, Ordering::Relaxed);
}

unsafe extern "C" fn release_asset_bytes(owner: *mut c_void) {
    unsafe { &*owner.cast::<AssetHookProbe>() }
        .releases
        .fetch_add(1, Ordering::Relaxed);
}

fn retained_bytes(data: &'static [u8], owner: *mut c_void) -> NuxRetainedBytes {
    NuxRetainedBytes {
        data: data.as_ptr(),
        len: data.len(),
        owner,
        retain: Some(retain_asset_bytes),
        release: Some(release_asset_bytes),
        ..NuxRetainedBytes::default()
    }
}

unsafe extern "C" fn provide_external_image(
    context: *mut c_void,
    request: *const NuxExternalAssetRequest,
    out_bytes: *mut NuxRetainedBytes,
) -> NuxAssetCallbackStatus {
    let request = unsafe { &*request };
    assert_eq!(request.kind, NUX_ASSET_KIND_IMAGE);
    unsafe { &*context.cast::<AssetHookProbe>() }
        .lookups
        .fetch_add(1, Ordering::Relaxed);
    unsafe { *out_bytes = retained_bytes(ENCODED_IMAGE, context) };
    NUX_ASSET_CALLBACK_STATUS_OK
}

unsafe extern "C" fn decode_external_image(
    context: *mut c_void,
    request: *const NuxImageDecodeRequest,
    out_image: *mut NuxDecodedImage,
) -> NuxAssetCallbackStatus {
    let request = unsafe { &*request };
    let encoded = unsafe { std::slice::from_raw_parts(request.encoded.data, request.encoded.len) };
    assert_eq!(encoded, ENCODED_IMAGE);
    unsafe { &*context.cast::<AssetHookProbe>() }
        .decodes
        .fetch_add(1, Ordering::Relaxed);
    unsafe {
        *out_image = NuxDecodedImage {
            width: 4,
            height: 2,
            row_bytes: 16,
            pixel_format: NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
            pixels: retained_bytes(DECODED_PIXELS, context),
            ..NuxDecodedImage::default()
        };
    }
    NUX_ASSET_CALLBACK_STATUS_OK
}

#[test]
fn portable_asset_hooks_reach_the_android_vulkan_render_path() {
    let probe = AssetHookProbe::default();
    let hooks = NuxAssetHooks {
        context: std::ptr::from_ref(&probe).cast_mut().cast(),
        lookup_external_asset: Some(provide_external_image),
        decode_image: Some(decode_external_image),
        ..NuxAssetHooks::default()
    };

    unsafe {
        let mut file = ptr::null_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            nux_file_import_with_assets(
                EXTERNAL_IMAGE_ARTBOARD.as_ptr(),
                EXTERNAL_IMAGE_ARTBOARD.len(),
                &hooks,
                &mut file,
                &mut result,
            ),
            NuxStatus::Ok
        );
        assert_eq!(probe.lookups.load(Ordering::Relaxed), 2);
        assert_eq!(probe.decodes.load(Ordering::Relaxed), 2);
        assert_eq!(probe.retains.load(Ordering::Relaxed), 4);
        assert_eq!(probe.releases.load(Ordering::Relaxed), 4);
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);

        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);

        let mut renderer = ptr::null_mut();
        result = ptr::null_mut();
        let create_status = nux_renderer_new_android_vulkan(64, 64, &mut renderer, &mut result);
        if create_status != NuxStatus::Ok && !live_vulkan_test_required() {
            assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);
            assert_eq!(nux_player_free(player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
            return;
        }
        assert_eq!(create_status, NuxStatus::Ok);
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);

        let mut frame = ptr::null_mut();
        result = ptr::null_mut();
        assert_eq!(
            nux_renderer_android_vulkan_render_player(
                renderer,
                player,
                0xff00_0000,
                NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER,
                &mut frame,
                &mut result,
            ),
            NuxStatus::Ok
        );
        assert_eq!(probe.lookups.load(Ordering::Relaxed), 2);
        assert_eq!(probe.decodes.load(Ordering::Relaxed), 2);
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);
        assert_eq!(nux_android_vulkan_frame_free(frame), NuxStatus::Ok);
        assert_eq!(nux_renderer_android_vulkan_free(renderer), NuxStatus::Ok);
        assert_eq!(nux_player_free(player), NuxStatus::Ok);
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
    }
}

#[test]
fn fixture_renders_content_through_the_android_vulkan_arm() {
    let required = live_vulkan_test_required();
    let Some(riv_path) = probe_fixture_path() else {
        assert!(
            !required,
            "required live Vulkan fixture is unavailable; set NUX_PROBE_RIV or provide fixtures/flow/data_binding_test.riv"
        );
        return;
    };
    let bytes = std::fs::read(&riv_path).expect("read riv");

    unsafe {
        let mut file: *mut NuxFile = ptr::null_mut();
        assert_eq!(
            nux_file_import(bytes.as_ptr(), bytes.len(), &mut file),
            NuxStatus::Ok
        );
        let mut artboard: *mut NuxArtboardInstance = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        let mut player: *mut NuxPlayer = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);

        let mut renderer: *mut NuxAndroidVulkanRenderer = ptr::null_mut();
        let mut result: *mut NuxCapiResult = ptr::null_mut();
        let create_status = nux_renderer_new_android_vulkan(400, 800, &mut renderer, &mut result);
        if create_status != NuxStatus::Ok {
            if !required {
                nux_capi_result_free(result);
                nux_player_free(player);
                nux_artboard_instance_free(artboard);
                nux_file_free(file);
                return;
            }
            let mut view: NuxCapiDiagnosticView = std::mem::zeroed();
            view.struct_size = std::mem::size_of::<NuxCapiDiagnosticView>() as u32;
            if !result.is_null() && nux_capi_result_diagnostic(result, &mut view) == NuxStatus::Ok {
                let msg =
                    std::slice::from_raw_parts(view.message.data as *const u8, view.message.len);
                panic!("renderer creation failed: {}", String::from_utf8_lossy(msg));
            }
            panic!("renderer creation failed with status {create_status:?}");
        }
        nux_capi_result_free(result);
        result = ptr::null_mut();

        // Step once like the host frame loop does.
        let step = NuxPlayerStep {
            struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
            elapsed_seconds: 0.1,
            ..std::mem::zeroed()
        };
        let mut step_result: *mut NuxPlayerStepResult = ptr::null_mut();
        assert_eq!(
            nux_player_step(player, &step, &mut step_result),
            NuxStatus::Ok
        );
        if !step_result.is_null() {
            nux_player_step_result_free(step_result);
        }

        let mut frame: *mut NuxAndroidVulkanFrame = ptr::null_mut();
        let status = nux_renderer_android_vulkan_render_player(
            renderer,
            player,
            0xFFFF00FF,
            NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER,
            &mut frame,
            &mut result,
        );
        if status != NuxStatus::Ok {
            let mut view: NuxCapiDiagnosticView = std::mem::zeroed();
            view.struct_size = std::mem::size_of::<NuxCapiDiagnosticView>() as u32;
            if !result.is_null() && nux_capi_result_diagnostic(result, &mut view) == NuxStatus::Ok {
                let msg =
                    std::slice::from_raw_parts(view.message.data as *const u8, view.message.len);
                panic!("render failed: {}", String::from_utf8_lossy(msg));
            }
            panic!("render failed with status {status:?}");
        }
        nux_capi_result_free(result);

        let len = nux_android_vulkan_frame_len(frame);
        let data = std::slice::from_raw_parts(nux_android_vulkan_frame_data(frame), len);
        let mut colors = std::collections::HashSet::new();
        for px in data.chunks_exact(4) {
            colors.insert([px[0], px[1], px[2], px[3]]);
        }
        std::fs::write("/tmp/nux-probe-frame.raw", data).unwrap();
        eprintln!(
            "PROBE: {} distinct colors across {} pixels (frame dumped to /tmp/nux-probe-frame.raw, 400x800 RGBA)",
            colors.len(),
            len / 4
        );
        assert!(
            colors.len() >= 4,
            "content did not render: only {} distinct colors",
            colors.len()
        );
        let pixel = |x: usize, y: usize| &data[(y * 400 + x) * 4..][..4];
        assert_eq!(
            pixel(335, 360),
            [0x00, 0x00, 0x00, 0xff],
            "top-row-first frame is vertically inverted or missing the Color toggle"
        );
        nux_android_vulkan_frame_free(frame);
        nux_renderer_android_vulkan_free(renderer);
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}
