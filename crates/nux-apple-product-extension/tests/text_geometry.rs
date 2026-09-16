//! Published native-input geometry with the canonical external font.

#![cfg(all(feature = "apple-runtime", any(target_os = "ios", target_os = "macos")))]

use nux_apple_product_extension::nux_product_file_import_configured;
use nux_capi::*;
use std::ffi::{CString, c_void};
use std::ptr;

const SCENE: &[u8] = include_bytes!("../../../fixtures/univ-3199/screen.riv");
const FONT: &[u8] = include_bytes!("../../../fixtures/univ-3199/fixture-sans.otf");

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

fn metal_renderer() -> *mut NuxRenderer {
    let mut renderer = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_renderer_new_metal(1, 1, &mut renderer, &mut result) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
    assert!(!renderer.is_null());
    renderer
}

fn string_view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

#[test]
fn published_text_input_geometry_matches_authored_boxes_after_metric_changes() {
    let scene = SCENE;
    let renderer = metal_renderer();

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
            nux_product_file_import_configured(
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
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
        NuxStatus::Ok
    );
    let mut player = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(artboard, &mut player) },
        NuxStatus::Ok
    );
    let mut mismatches = Vec::new();
    let mut previous_result: *mut NuxPlayerStepResult = ptr::null_mut();
    for (font_size, line_height) in [(18.0, 24.0), (36.0, 48.0), (24.0, -1.0), (18.0, 24.0)] {
        for (path, value) in [
            ("requestedFontSize", font_size),
            ("requestedLineHeight", line_height),
        ] {
            let path = CString::new(path).unwrap();
            assert_eq!(
                unsafe { nux_view_model_instance_set_number(view_model, path.as_ptr(), value) },
                NuxStatus::Ok
            );
        }
        if !previous_result.is_null() {
            let mut stale = NuxTextRunGeometry {
                struct_size: std::mem::size_of::<NuxTextRunGeometry>() as u32,
                render_revision: u64::MAX,
                ..Default::default()
            };
            assert_eq!(
                unsafe {
                    nux_player_text_run_geometry(
                        player,
                        previous_result,
                        string_view("bound Run"),
                        &mut stale,
                    )
                },
                NuxStatus::HandleMismatch,
                "shared ViewModel mutation must invalidate geometry before another step"
            );
            assert_eq!(
                stale.render_revision,
                u64::MAX,
                "failed reads must preserve output"
            );
            assert_eq!(
                unsafe { nux_player_step_result_free(previous_result) },
                NuxStatus::Ok
            );
        }
        let step = NuxPlayerStep {
            struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(player, &step, &mut result) },
            NuxStatus::Ok
        );
        for (run, expected_y) in [("bound Run", 24.0), ("fixed Run", 264.0)] {
            let mut geometry = NuxTextRunGeometry {
                struct_size: std::mem::size_of::<NuxTextRunGeometry>() as u32,
                ..Default::default()
            };
            assert_eq!(
                unsafe {
                    nux_player_text_run_geometry(player, result, string_view(run), &mut geometry)
                },
                NuxStatus::Ok
            );
            // The committed font selects OS/2 typo metrics (ascent 2728,
            // units/em 2896). At the runtime's 2048-unit shaping scale this
            // is an independently calculated 1929-unit ascent.
            assert_eq!(geometry.has_first_baseline, 1);
            let effective_font_size = if run == "bound Run" { font_size } else { 18.0 };
            let expected_baseline = 1929.0 / 2048.0 * effective_font_size;
            assert!(
                (geometry.first_baseline - expected_baseline).abs() < 0.001,
                "{run}: baseline {}, expected {expected_baseline}",
                geometry.first_baseline
            );
            assert_eq!(
                geometry.has_layout_ancestor, 1,
                "published field must retain its layout owner"
            );
            let actual = [
                geometry.layout_ancestor_transform[4] + geometry.layout_ancestor_min_x,
                geometry.layout_ancestor_transform[5] + geometry.layout_ancestor_min_y,
                geometry.layout_ancestor_max_x - geometry.layout_ancestor_min_x,
                geometry.layout_ancestor_max_y - geometry.layout_ancestor_min_y,
            ];
            eprintln!(
                "{run} metrics={font_size}/{line_height} box={actual:?} world={:?} content={:?}",
                geometry.world_transform, geometry.content_transform
            );
            for (actual, expected) in actual.into_iter().zip([24.0, expected_y, 342.0, 220.0]) {
                if (actual - expected).abs() >= 0.001 {
                    mismatches.push(format!("{run} metrics={font_size}/{line_height}: actual {actual}, expected {expected}"));
                }
            }
        }
        previous_result = result;
    }
    assert_eq!(
        unsafe { nux_player_step_result_free(previous_result) },
        NuxStatus::Ok
    );
    unsafe {
        nux_player_free(player);
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
        nux_renderer_free(renderer);
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
