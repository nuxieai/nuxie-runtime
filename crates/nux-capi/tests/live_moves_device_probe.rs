//! Device-shaped media identity regression.
use nux_capi::*;
use nuxie::live_move::device_fixture as fixture;
use std::{ffi::c_void, mem::size_of, ptr};
fn sv(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
#[derive(Clone, Debug)]
struct Occurrence {
    id: usize,
    generation: u64,
    source: (usize, usize),
    visible: bool,
    source_roi: bool,
}
unsafe extern "C" fn collect(ctx: *mut c_void, info: *const NuxVideoInfo) {
    let v = unsafe { &*info };
    unsafe { &mut *ctx.cast::<Vec<Occurrence>>() }.push(Occurrence {
        id: v.component_id,
        generation: v.generation,
        source: (v.source_artboard_index, v.source_component_id),
        visible: false,
        source_roi: false,
    });
}
unsafe fn step(p: *mut NuxPlayer) {
    let mut out = ptr::null_mut();
    let input = NuxPlayerStep {
        struct_size: size_of::<NuxPlayerStep>() as u32,
        elapsed_seconds: 0.0,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_player_step(p, &input, &mut out) },
        NuxStatus::Ok
    );
    unsafe { nux_player_step_result_free(out) };
}
unsafe fn snapshot(p: *mut NuxPlayer, label: &str) -> Vec<Occurrence> {
    let mut rows = Vec::<Occurrence>::new();
    assert_eq!(
        unsafe { nux_player_visit_videos(p, Some(collect), ptr::from_mut(&mut rows).cast()) },
        NuxStatus::Ok
    );
    for r in &mut rows {
        let mut visible = 0;
        assert_eq!(
            unsafe { nux_player_video_is_visible(p, r.id, 0., 0., 800., 600., &mut visible) },
            NuxStatus::Ok
        );
        r.visible = visible != 0;
        let mut roi = 0;
        assert_eq!(
            unsafe { nux_player_video_is_visible(p, r.id, 53., 70., 54., 100., &mut roi) },
            NuxStatus::Ok
        );
        r.source_roi = roi != 0;
        println!(
            "ROI boundary={label} id={} source_roi={}",
            r.id, r.source_roi
        );
        println!(
            "VIDEO boundary={label} id={} source={}:{} generation={} visible={}",
            r.id, r.source.0, r.source.1, r.generation, r.visible
        );
    }
    rows
}
#[test]
fn live_moves_video_source_identity_survives_landing_and_collapse() {
    unsafe {
        let bytes = fixture::control_fixture();
        let (mut file, mut artboard, mut vm, mut player, mut result) = (
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_eq!(
            nux_file_import_with_video_capabilities(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &NuxVideoPlaybackCapabilities {
                    playback_available: 1,
                    ..Default::default()
                },
                &mut file,
                &mut result
            ),
            NuxStatus::Ok
        );
        nux_capi_result_free(result);
        assert_eq!(
            nux_artboard_instance_new_named(file, sv("Page"), &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(artboard, &mut vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, vm),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        step(player);

        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 1),
            NuxStatus::Ok
        );
        step(player);
        let initial = snapshot(player, "source");
        assert_eq!(
            initial.iter().filter(|r| r.source_roi).count(),
            1,
            "source ROI is unambiguous"
        );
        let source = initial
            .iter()
            .find(|r| r.source_roi)
            .expect("source video intersects x=53..54 y=70..100")
            .clone();
        let harness_min = initial
            .iter()
            .filter(|r| r.source.0 == 3)
            .min_by_key(|r| r.id)
            .unwrap();
        println!(
            "TRACKING actual_source={} harness_min={} harness_is_source={}",
            source.id, harness_min.id, harness_min.source_roi
        );
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 0),
            NuxStatus::Ok
        );
        snapshot(player, "expand-B");
        let transition = sv("t1");
        assert_eq!(
            nux_player_step_begin(player, sv("B"), &transition),
            NuxStatus::Ok
        );
        snapshot(player, "takeoff");
        step(player);
        snapshot(player, "flight");
        assert_eq!(
            nux_player_step_end(player, sv("B"), &transition),
            NuxStatus::Ok
        );
        let landed = snapshot(player, "land-before-advance");
        step(player);
        let advanced = snapshot(player, "land-after-advance");
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 0, 1),
            NuxStatus::Ok
        );
        let collapsed = snapshot(player, "collapse-A-before-advance");
        step(player);
        let final_rows = snapshot(player, "collapse-A-after-advance");
        for rows in [&landed, &advanced, &collapsed, &final_rows] {
            let moved = rows
                .iter()
                .find(|r| r.id == source.id)
                .expect("moved source remains enumerable");
            assert_eq!(
                moved.id, source.id,
                "source Video ID changed at landing/collapse"
            );
            assert_eq!(
                moved.generation, source.generation,
                "source generation changed without SDK reclaim"
            );
        }
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 1),
            NuxStatus::Ok
        );
        let dormant = snapshot(player, "collapse-unit-before-advance");
        assert!(dormant.iter().all(|video| video.id != source.id));
        step(player);
        assert!(
            snapshot(player, "collapse-unit-after-advance")
                .iter()
                .all(|video| video.id != source.id)
        );
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 0),
            NuxStatus::Ok
        );
        let woken = snapshot(player, "wake-unit-before-advance");
        let restored = woken
            .iter()
            .find(|video| video.id == source.id)
            .expect("same video id returns immediately on wake");
        assert_eq!(restored.generation, source.generation);
        assert_eq!(restored.source, source.source);
        assert!(restored.visible);
        step(player);
        assert!(
            snapshot(player, "wake-unit-after-advance")
                .iter()
                .any(|video| video.id == source.id && video.visible)
        );
        println!(
            "PASS actual_source_survives id={} harness_min_was_source={}",
            source.id,
            harness_min.id == source.id
        );
        nux_player_free(player);
        nux_view_model_instance_free(vm);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}
