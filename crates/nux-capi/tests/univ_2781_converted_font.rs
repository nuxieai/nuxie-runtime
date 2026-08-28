#![cfg(all(
    feature = "android-vulkan",
    feature = "scripting",
    feature = "android-authored-wgsl"
))]

use nux_capi::*;
use std::ffi::{CString, c_void};
use std::ptr;

unsafe extern "C" {
    fn nux_file_import_configured_with_trusted_wgsl(
        bytes: *const u8,
        len: usize,
        config: *const NuxFileImportConfig,
        out_file: *mut *mut NuxFile,
        out_result: *mut *mut NuxCapiResult,
    ) -> NuxStatus;
}

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

#[derive(Default)]
struct DrawProbe {
    next: u64,
    draws: usize,
}

unsafe extern "C" fn make_path(
    user_data: *mut c_void,
    _path: *const NuxRawPathView,
    _fill_rule: u8,
) -> u64 {
    unsafe { make_handle(user_data) }
}

unsafe extern "C" fn make_handle(user_data: *mut c_void) -> u64 {
    let probe = unsafe { &mut *user_data.cast::<DrawProbe>() };
    probe.next = probe.next.checked_add(1).expect("fixture handle overflow");
    probe.next
}

unsafe extern "C" fn draw_path(user_data: *mut c_void, _path: u64, _paint: u64) {
    let probe = unsafe { &mut *user_data.cast::<DrawProbe>() };
    probe.draws = probe.draws.saturating_add(1);
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
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_configured_with_trusted_wgsl(
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

    let mut probe = DrawProbe::default();
    let callbacks = NuxRenderCallbacks {
        user_data: (&mut probe as *mut DrawProbe).cast(),
        make_render_path: Some(make_path),
        make_empty_render_path: Some(make_handle),
        make_render_paint: Some(make_handle),
        draw_path: Some(draw_path),
        ..NuxRenderCallbacks::default()
    };
    assert_eq!(
        unsafe { nux_artboard_instance_draw(artboard, &callbacks) },
        NuxStatus::Ok
    );
    assert!(
        probe.draws > 1,
        "the converter-bound label must produce font glyph paths; draws={}",
        probe.draws
    );

    unsafe {
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}
