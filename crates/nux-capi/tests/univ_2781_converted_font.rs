#![cfg(all(
    feature = "android-vulkan",
    feature = "scripting",
    feature = "android-authored-wgsl"
))]

use nux_capi::*;
use std::ffi::{CString, c_void};
use std::ptr;

const SCENE_BASE64: &str = include_str!("../../../fixtures/univ-2781/font-converter.riv.b64");
const FONT: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-a.ttf");

fn decode_fixture() -> Vec<u8> {
    let mut output = Vec::new();
    let mut word = 0_u32;
    let mut sextets = 0_u8;
    for byte in SCENE_BASE64
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        if byte == b'=' {
            break;
        }
        let value = match byte {
            b'A'..=b'Z' => byte.checked_sub(b'A').expect("matched ASCII range"),
            b'a'..=b'z' => byte
                .checked_sub(b'a')
                .and_then(|value| value.checked_add(26))
                .expect("matched ASCII range"),
            b'0'..=b'9' => byte
                .checked_sub(b'0')
                .and_then(|value| value.checked_add(52))
                .expect("matched ASCII range"),
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64 fixture byte"),
        };
        word = (word << 6) | u32::from(value);
        sextets = sextets.checked_add(1).expect("base64 quantum overflow");
        if sextets == 4 {
            output.extend_from_slice(&word.to_be_bytes()[1..]);
            word = 0;
            sextets = 0;
        }
    }
    if sextets == 2 {
        output.push((word >> 4) as u8);
    } else if sextets == 3 {
        output.extend_from_slice(&(word >> 2).to_be_bytes()[2..]);
    }
    output
}

unsafe extern "C" fn retain(_owner: *mut c_void) {}
unsafe extern "C" fn release(_owner: *mut c_void) {}

unsafe extern "C" fn lookup_font(
    context: *mut c_void,
    request: *const NuxExternalAssetRequest,
    out_bytes: *mut NuxRetainedBytes,
) -> NuxAssetCallbackStatus {
    let request = unsafe { &*request };
    if request.kind != NUX_ASSET_KIND_FONT {
        return NUX_ASSET_CALLBACK_STATUS_NOT_FOUND;
    }
    let requests = unsafe { &mut *context.cast::<usize>() };
    *requests = requests.saturating_add(1);
    unsafe {
        *out_bytes = NuxRetainedBytes {
            data: FONT.as_ptr(),
            len: FONT.len(),
            owner: ptr::null_mut(),
            retain: Some(retain),
            release: Some(release),
            ..NuxRetainedBytes::default()
        };
    }
    NUX_ASSET_CALLBACK_STATUS_OK
}

fn string_view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

#[test]
fn android_product_import_applies_external_font_to_converter_bound_text() {
    let scene = decode_fixture();
    assert!(scene.windows(8).any(|window| window == b"NUXPCV1\0"));

    let mut font_requests = 0_usize;
    let hooks = NuxAssetHooks {
        context: (&mut font_requests as *mut usize).cast(),
        lookup_external_asset: Some(lookup_font),
        ..NuxAssetHooks::default()
    };
    let host = NuxHostCommandImportConfig {
        module_name: string_view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let config = NuxFileImportConfig {
        host_commands: &host,
        asset_hooks: &hooks,
        ..NuxFileImportConfig::default()
    };
    let mut renderer = ptr::null_mut();
    let mut renderer_result = ptr::null_mut();
    let renderer_status =
        unsafe { nux_renderer_new_android_vulkan(256, 256, &mut renderer, &mut renderer_result) };
    if renderer_status != NuxStatus::Ok {
        eprintln!("skipping Vulkan render fixture: renderer unavailable ({renderer_status:?})");
        unsafe { nux_capi_result_free(renderer_result) };
        return;
    }
    unsafe { nux_capi_result_free(renderer_result) };
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_android_vulkan_with_trusted_wgsl(
                renderer,
                scene.as_ptr(),
                scene.len(),
                &config,
                &mut file,
                &mut result,
            )
        },
        NuxStatus::Ok
    );
    assert_eq!(font_requests, 1, "the authored external font must resolve");
    unsafe { nux_capi_result_free(result) };

    let mut artboard = ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    let mut view_model = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(artboard, &mut view_model) },
        NuxStatus::Ok
    );
    let label = CString::new("label").expect("static string");
    let content = CString::new("CONVERTER-PRODUCED").expect("static string");
    assert_eq!(
        unsafe { nux_view_model_instance_set_string(view_model, label.as_ptr(), content.as_ptr()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_advance(artboard, 0.0, ptr::null_mut()) },
        NuxStatus::Ok
    );

    let mut player = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(artboard, &mut player) },
        NuxStatus::Ok
    );
    let mut frame = ptr::null_mut();
    let mut render_result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_renderer_android_vulkan_render_player(
                renderer,
                player,
                0x0000_0000,
                NUX_ANDROID_VULKAN_RENDERER_FIT_CONTAIN_CENTER,
                &mut frame,
                &mut render_result,
            )
        },
        NuxStatus::Ok
    );
    assert!(render_result.is_null());
    let frame_bytes = unsafe {
        std::slice::from_raw_parts(
            nux_android_vulkan_frame_data(frame),
            nux_android_vulkan_frame_len(frame),
        )
    };
    assert!(
        frame_bytes.chunks_exact(4).any(|pixel| pixel[3] != 0),
        "the converter-bound external font must render visible glyph pixels"
    );

    unsafe {
        nux_android_vulkan_frame_free(frame);
        nux_player_free(player);
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
        nux_renderer_android_vulkan_free(renderer);
    }
}
