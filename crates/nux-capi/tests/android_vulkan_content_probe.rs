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

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
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

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("fixture type");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("fixture ancestor")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("fixture property")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, body: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .expect("fixture type")
                .type_key
                .int,
        ),
    );
    body(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, property_name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, property_name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// Repository-owned schema fixture with one external image used by one drawable.
fn external_image_artboard() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 2, 26_520, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ImageAsset", |bytes| {
        push_uint(bytes, "ImageAsset", "assetId", 7);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 64.0);
        push_f32(bytes, "Artboard", "height", 64.0);
    });
    push_object(&mut bytes, "Image", |bytes| {
        push_uint(bytes, "Image", "parentId", 0);
        push_uint(bytes, "Image", "assetId", 0);
    });
    bytes
}

const ENCODED_IMAGE: &[u8] = include_bytes!("fixtures/external-image.png");
const STUB_PIXEL: [u8; 4] = [0x12, 0x34, 0x56, 0xff];
const DECODED_PIXELS: &[u8] = &[
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x12, 0x34, 0x56, 0xff, 0xff, 0xff, 0xff, 0xff,
];

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

unsafe fn frame_contains_pixel(frame: *const NuxAndroidVulkanFrame, expected: [u8; 4]) -> bool {
    let len = unsafe { nux_android_vulkan_frame_len(frame) };
    let pixels = unsafe { std::slice::from_raw_parts(nux_android_vulkan_frame_data(frame), len) };
    pixels.chunks_exact(4).any(|pixel| pixel == expected)
}

#[test]
fn portable_asset_hooks_reach_the_android_vulkan_render_path() {
    let rive = external_image_artboard();
    let probe = AssetHookProbe::default();
    let hooks = NuxAssetHooks {
        context: std::ptr::from_ref(&probe).cast_mut().cast(),
        lookup_external_asset: Some(provide_external_image),
        decode_image: Some(decode_external_image),
        ..NuxAssetHooks::default()
    };

    unsafe {
        let mut renderer = ptr::null_mut();
        let mut result = ptr::null_mut();
        let create_status = nux_renderer_new_android_vulkan(64, 64, &mut renderer, &mut result);
        if create_status != NuxStatus::Ok && !live_vulkan_test_required() {
            assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);
            return;
        }
        assert_eq!(create_status, NuxStatus::Ok);
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);

        let config = NuxFileImportConfig {
            asset_hooks: &hooks,
            ..NuxFileImportConfig::default()
        };
        let mut file = ptr::null_mut();
        result = ptr::null_mut();
        assert_eq!(
            nux_file_import_android_vulkan(
                renderer,
                rive.as_ptr(),
                rive.len(),
                &config,
                &mut file,
                &mut result,
            ),
            NuxStatus::Ok
        );
        assert_eq!(probe.lookups.load(Ordering::Relaxed), 1);
        assert_eq!(probe.decodes.load(Ordering::Relaxed), 1);
        assert_eq!(probe.retains.load(Ordering::Relaxed), 2);
        assert_eq!(probe.releases.load(Ordering::Relaxed), 2);
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);

        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);

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
        assert_eq!(probe.lookups.load(Ordering::Relaxed), 1);
        assert_eq!(probe.decodes.load(Ordering::Relaxed), 1);
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);

        assert!(
            frame_contains_pixel(frame, STUB_PIXEL),
            "factory-at-import did not upload host-decoded pixels"
        );
        assert!(
            !frame_contains_pixel(frame, [0xff, 0x80, 0x00, 0xff]),
            "factory-at-import used the fixture PNG's native orange texel"
        );
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
        let mut renderer: *mut NuxAndroidVulkanRenderer = ptr::null_mut();
        let mut result: *mut NuxCapiResult = ptr::null_mut();
        let create_status = nux_renderer_new_android_vulkan(400, 800, &mut renderer, &mut result);
        if create_status != NuxStatus::Ok {
            if !required {
                nux_capi_result_free(result);
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

        let mut file: *mut NuxFile = ptr::null_mut();
        assert_eq!(
            nux_file_import_android_vulkan(
                renderer,
                bytes.as_ptr(),
                bytes.len(),
                &NuxFileImportConfig::default(),
                &mut file,
                &mut result,
            ),
            NuxStatus::Ok
        );
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);
        let mut artboard: *mut NuxArtboardInstance = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        let mut player: *mut NuxPlayer = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);

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
