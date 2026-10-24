//! Device-shaped collapsed-row regression and host timing probe for UNIV-5289.
use nux_capi::*;
use nuxie::hosted_scene::{WalkCounts, measure_walks};
use nuxie::live_move::device_fixture as fixture;
use std::{ffi::c_void, mem::size_of, ptr, time::Instant};
fn sv(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
unsafe extern "C" fn collect(ctx: *mut c_void, info: *const NuxVideoInfo) {
    unsafe { &mut *ctx.cast::<Vec<usize>>() }.push(unsafe { (*info).component_id });
}
unsafe fn videos(p: *mut NuxPlayer) -> Vec<usize> {
    let mut ids = Vec::new();
    assert_eq!(
        unsafe { nux_player_visit_videos(p, Some(collect), ptr::from_mut(&mut ids).cast()) },
        NuxStatus::Ok
    );
    ids
}
unsafe fn advance(p: *mut NuxPlayer) {
    let pointer = NuxPlayerPointerEvent {
        kind: NUX_PLAYER_POINTER_KIND_MOVE,
        x: 799.0,
        y: 599.0,
        pointer_id: 0,
        timestamp_seconds: 0.0,
    };
    let input = NuxPlayerStep {
        pointers: &pointer,
        pointer_count: 1,
        struct_size: size_of::<NuxPlayerStep>() as u32,
        elapsed_seconds: 1.0 / 60.0,
        ..Default::default()
    };
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_step(p, &input, &mut result) },
        NuxStatus::Ok
    );
    let mut scheduling = NuxPlayerSchedulingInfo {
        struct_size: size_of::<NuxPlayerSchedulingInfo>() as u32,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_player_step_result_scheduling(result, &mut scheduling) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(p, scheduling.render_revision) },
        NuxStatus::Ok
    );
    unsafe {
        nux_player_step_result_free(result);
    }
}
unsafe fn semantic_ids(player: *mut NuxPlayer) -> Vec<u32> {
    let mut snapshot = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_semantic_snapshot(player, &mut snapshot) },
        NuxStatus::Ok
    );
    let mut info = NuxSemanticSnapshotInfo {
        struct_size: size_of::<NuxSemanticSnapshotInfo>() as u32,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_semantic_snapshot_info(snapshot, &mut info) },
        NuxStatus::Ok
    );
    let mut ids: Vec<_> = (0..info.node_count)
        .map(|index| {
            let mut node = NuxSemanticNodeView {
                struct_size: size_of::<NuxSemanticNodeView>() as u32,
                ..Default::default()
            };
            assert_eq!(
                unsafe { nux_semantic_snapshot_node(snapshot, index, &mut node) },
                NuxStatus::Ok
            );
            node.id
        })
        .collect();
    unsafe {
        nux_semantic_snapshot_free(snapshot);
    }
    ids.sort_unstable();
    assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    ids
}
unsafe extern "C" fn ignore_action(_: *mut c_void, _: *const NuxVideoAction) {}
unsafe fn measure(rows: usize, flight: bool, late_semantics: bool) -> (usize, usize, WalkCounts) {
    // Round 9 opts a Page into live moves through its authored LiveSteps.
    // Keep those declarations for the one-mounted-row baseline as well.
    let bytes = fixture::device_fixture(rows.max(2));
    let (mut file, mut artboard, mut vm, mut player, mut result) = (
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
    );
    assert_eq!(
        unsafe {
            nux_file_import_with_video_capabilities(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &NuxVideoPlaybackCapabilities {
                    playback_available: 1,
                    ..Default::default()
                },
                &mut file,
                &mut result,
            )
        },
        NuxStatus::Ok
    );
    unsafe {
        nux_capi_result_free(result);
    }
    assert_eq!(
        unsafe { nux_artboard_instance_new_named(file, sv("Page"), &mut artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(artboard, &mut vm) },
        NuxStatus::Ok
    );
    if rows == 1 {
        let remove = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
            instance: vm,
            path: sv("screens"),
            index: 1,
            ..Default::default()
        };
        let batch = NuxViewModelMutationBatch {
            mutations: &remove,
            mutation_count: 1,
            ..Default::default()
        };
        let mut mutation_result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_mutate(&batch, &mut mutation_result) },
            NuxStatus::Ok
        );
        unsafe { nux_view_model_mutation_result_free(mutation_result) };
    }
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, vm) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_new_default(artboard, &mut player) },
        NuxStatus::Ok
    );
    if !late_semantics {
        assert_eq!(
            unsafe { nux_player_enable_semantics(player) },
            NuxStatus::Ok
        );
    }
    unsafe {
        advance(player);
    }
    for row in 1..rows {
        assert_eq!(
            unsafe { nux_player_set_row_collapsed(player, sv("screens"), row as u32, 1) },
            NuxStatus::Ok
        );
    }
    unsafe {
        advance(player);
    }
    if late_semantics {
        assert_eq!(
            unsafe { nux_player_enable_semantics(player) },
            NuxStatus::Ok
        );
        unsafe {
            advance(player);
        }
    }
    if flight {
        assert!(rows > 1);
        assert_eq!(
            unsafe { nux_player_set_row_collapsed(player, sv("screens"), 1, 0) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_step_begin(player, sv("B"), &sv("t1")) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_set_row_collapsed(player, sv("screens"), 0, 1) },
            NuxStatus::Ok
        );
        unsafe {
            advance(player);
        }
    }
    let iterations = 20;
    let (mut step, mut enumeration, mut visibility, mut semantics) = (0.0, 0.0, 0.0, 0.0);
    let (mut count, mut visible, mut nodes) = (0, 0, 0);
    let mut visits = WalkCounts::default();
    for frame in 0..iterations {
        measure_walks(true);
        let t = Instant::now();
        unsafe {
            advance(player);
        }
        step += t.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            unsafe { nux_artboard_instance_draw(artboard) },
            NuxStatus::Ok
        );
        let t = Instant::now();
        let ids = unsafe { videos(player) };
        enumeration += t.elapsed().as_secs_f64() * 1000.0;
        count = ids.len();
        let t = Instant::now();
        visible = 0;
        for id in ids {
            let mut value = 0;
            assert_eq!(
                unsafe {
                    nux_player_video_is_visible(player, id, 0.0, 0.0, 800.0, 600.0, &mut value)
                },
                NuxStatus::Ok
            );
            visible += value as usize;
        }
        visibility += t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_semantic_snapshot(player, &mut snapshot) },
            NuxStatus::Ok
        );
        let mut info = NuxSemanticSnapshotInfo {
            struct_size: size_of::<NuxSemanticSnapshotInfo>() as u32,
            ..Default::default()
        };
        assert_eq!(
            unsafe { nux_semantic_snapshot_info(snapshot, &mut info) },
            NuxStatus::Ok
        );
        nodes = info.node_count;
        unsafe {
            nux_semantic_snapshot_free(snapshot);
        }
        semantics += t.elapsed().as_secs_f64() * 1000.0;
        if frame == iterations - 1 {
            println!(
                "STEP_PHASES rows={rows} flight={flight} phases={:?}",
                nuxie::hosted_scene::measured_step_phases()
            );
        }
        let current = measure_walks(false);
        if frame != 0 {
            assert_eq!(current, visits, "same work on each frame");
        }
        visits = current;
    }
    println!(
        "COLLAPSED_VISITS rows={rows} flight={flight} late_semantics={late_semantics} visits={visits:?}"
    );
    println!(
        "COLLAPSED_DEVICE rows={rows} flight={flight} late_semantics={late_semantics} collapsed={} iterations={iterations} videos={count} visible={visible} semantic_nodes={nodes} step_ms={:.6} video_enumeration_ms={:.6} video_visibility_batch_ms={:.6} semantic_snapshot_ms={:.6}",
        rows - 1,
        step / iterations as f64,
        enumeration / iterations as f64,
        visibility / iterations as f64,
        semantics / iterations as f64
    );
    let semantics_before = unsafe { semantic_ids(player) };
    let videos_before = unsafe { videos(player) };
    let active_row = u32::from(flight);
    assert_eq!(
        unsafe { nux_player_set_row_collapsed(player, sv("screens"), active_row, 1) },
        NuxStatus::Ok
    );
    unsafe {
        advance(player);
    }
    let dormant_ids = unsafe { semantic_ids(player) };
    assert!(
        dormant_ids.is_empty(),
        "rows={rows} flight={flight}: dormant semantic ids {dormant_ids:?}"
    );
    assert!(unsafe { videos(player) }.is_empty());
    assert_eq!(
        unsafe { nux_player_set_row_collapsed(player, sv("screens"), active_row, 0) },
        NuxStatus::Ok
    );
    unsafe {
        advance(player);
    }
    assert_eq!(
        unsafe { semantic_ids(player) },
        semantics_before,
        "wake restores every nested semantic id"
    );
    assert_eq!(
        unsafe { videos(player) },
        videos_before,
        "wake restores every video id"
    );
    // SDKs also mutate media state between enumeration and later lookups.
    // Render revision changes must not cause another hosted-scene walk.
    measure_walks(true);
    for id in unsafe { videos(player) } {
        assert_eq!(
            unsafe { nux_player_video_command(player, id, 1, 0.0, 0) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe {
                nux_player_video_step(player, id, 0, 0, 0.0, Some(ignore_action), ptr::null_mut())
            },
            NuxStatus::Ok
        );
        let mut visible = 0;
        assert_eq!(
            unsafe {
                nux_player_video_is_visible(player, id, 0.0, 0.0, 800.0, 600.0, &mut visible)
            },
            NuxStatus::Ok
        );
    }
    unsafe {
        videos(player);
    }
    assert_eq!(
        measure_walks(false),
        WalkCounts::default(),
        "media updates reuse the frame's map"
    );
    unsafe {
        nux_player_free(player);
        nux_view_model_instance_free(vm);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
    (visible, nodes, visits)
}
#[test]
fn live_moves_collapsed_rows_do_not_request_visible_video_decoders() {
    unsafe {
        let baseline = measure(1, false, false);
        let collapsed = measure(40, false, false);
        let flight = measure(40, true, false);
        let late = measure(40, false, true);
        assert_eq!(
            late, baseline,
            "enabling semantics after collapse preserves the same active surface"
        );
        assert_eq!(
            collapsed.0, baseline.0,
            "Collapsed rows must not demand visible video decoders. Retained occurrence IDs may remain enumerable."
        );
        assert_eq!(
            collapsed.2, baseline.2,
            "collapsed queries visit only active hosted artboards"
        );
        assert_eq!(
            baseline.2.video_lookup, 0,
            "video batch lookups reuse the occurrence map"
        );
        assert!(
            baseline.2.video_enumeration > 0
                && baseline.2.semantic_validation > 0
                && baseline.2.owner_capture > 0
                && baseline.2.advance > 0
                && baseline.2.layout > 0
                && baseline.2.draw > 0
                && baseline.2.hit > 0
        );
        assert_eq!(
            flight.1,
            baseline.1 - 1,
            "B has the moving unit and no inline comparison field"
        );
        assert_eq!(flight.2.video_enumeration, baseline.2.video_enumeration);
        assert_eq!(flight.2.video_lookup, 0);
        assert_eq!(flight.2.semantic_validation, baseline.2.semantic_validation);
        assert_eq!(flight.2.owner_capture, baseline.2.owner_capture);
        assert_eq!(
            collapsed.1, baseline.1,
            "Collapsed rows must not publish semantic nodes"
        );
    }
}
