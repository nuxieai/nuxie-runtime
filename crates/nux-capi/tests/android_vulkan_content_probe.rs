//! Session-local probe: does the android_vulkan capi arm render CONTENT?
#![cfg(feature = "android-vulkan")]

use std::ptr;

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
