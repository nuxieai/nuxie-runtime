use nux_capi::*;
use std::{ffi::c_void, ptr};
#[path = "support/video_properties.rs"]
mod fixture;

type Actions = Vec<(u32, f64, u64)>;
fn ok(status: NuxStatus) {
    assert_eq!(status, NuxStatus::Ok);
}
fn import(bytes: &[u8]) -> *mut NuxFile {
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    let capabilities = NuxVideoPlaybackCapabilities {
        playback_available: 1,
        ..Default::default()
    };
    unsafe {
        ok(nux_file_import_with_video_capabilities(
            bytes.as_ptr(),
            bytes.len(),
            &NuxRenderCallbacks::default(),
            &capabilities,
            &mut file,
            &mut result,
        ));
        nux_capi_result_free(result);
    }
    file
}
fn step(player: *mut NuxPlayer, seconds: f32) {
    unsafe {
        let mut result = ptr::null_mut();
        let step = NuxPlayerStep {
            elapsed_seconds: seconds,
            ..Default::default()
        };
        ok(nux_player_step(player, &step, &mut result));
        nux_player_step_result_free(result);
    }
}
fn observe(player: *mut NuxPlayer, id: usize, kind: u32, generation: u64, value: f64) -> Actions {
    unsafe extern "C" fn action(data: *mut c_void, action: *const NuxVideoAction) {
        let a = unsafe { &*action };
        unsafe { &mut *data.cast::<Actions>() }.push((a.kind, a.value, a.generation));
    }
    let mut actions = Vec::new();
    unsafe {
        ok(nux_player_video_step(
            player,
            id,
            kind,
            generation,
            value,
            Some(action),
            ptr::from_mut(&mut actions).cast(),
        ));
    }
    actions
}
struct VideoFixture {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    values: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
}
impl VideoFixture {
    fn new(timeline: bool) -> Self {
        let mut scene = Self {
            file: import(&fixture::fixture(0, timeline)),
            artboard: ptr::null_mut(),
            values: ptr::null_mut(),
            player: ptr::null_mut(),
        };
        unsafe {
            ok(nux_artboard_instance_new(
                scene.file,
                0,
                &mut scene.artboard,
            ));
            ok(nux_view_model_instance_new_default(
                scene.artboard,
                &mut scene.values,
            ));
            ok(nux_artboard_instance_bind_view_model(
                scene.artboard,
                scene.values,
            ));
            if timeline {
                ok(nux_player_new_linear_animation_named(
                    scene.artboard,
                    NuxStringView {
                        data: c"Volume".as_ptr(),
                        len: 6,
                    },
                    &mut scene.player,
                ));
            } else {
                ok(nux_player_new_default(scene.artboard, &mut scene.player));
            }
        }
        scene.step(0.0);
        scene.observe(0, 0, 0.0);
        scene
    }
    fn step(&self, seconds: f32) {
        step(self.player, seconds);
    }
    fn boolean(&self, name: &std::ffi::CStr, value: bool) {
        ok(unsafe { nux_view_model_instance_set_bool(self.values, name.as_ptr(), value) });
        self.step(0.0);
    }
    fn number(&self, name: &std::ffi::CStr, value: f32) {
        ok(unsafe { nux_view_model_instance_set_number(self.values, name.as_ptr(), value) });
        self.step(0.0);
    }
    fn value(&self, index: usize) -> (f32, u32) {
        unsafe {
            let mut snapshot = ptr::null_mut();
            ok(nux_view_model_instance_snapshot(self.values, &mut snapshot));
            let mut row = NuxViewModelSnapshotValueView::default();
            ok(nux_view_model_snapshot_value(snapshot, index, &mut row));
            let result = (row.number_value, row.bool_value);
            nux_view_model_snapshot_free(snapshot);
            result
        }
    }
    fn observe(&self, kind: u32, generation: u64, value: f64) -> Actions {
        observe(self.player, 1, kind, generation, value)
    }
}
impl Drop for VideoFixture {
    fn drop(&mut self) {
        unsafe {
            nux_player_free(self.player);
            nux_view_model_instance_free(self.values);
            nux_artboard_instance_free(self.artboard);
            nux_file_free(self.file);
        }
    }
}

#[test]
fn bindings_control_video_and_decoder_observations_flow_back_in_one_step() {
    let s = VideoFixture::new(false);
    s.observe(1, 0, 12.5);
    s.step(0.0);
    assert_eq!(s.value(5).0, 12.5);
    assert_eq!(s.value(8).0, 1.0);
    for (muted, volume) in [(true, 0.0), (false, 1.0)] {
        s.boolean(c"muted", muted);
        assert_eq!(s.observe(0, 0, 0.0), [(4, volume, 0)]);
    }
    s.number(c"volume", 0.4);
    assert_eq!(s.observe(0, 0, 0.0), [(4, f64::from(0.4_f32), 0)]);
    s.number(c"volume", 2.0);
    assert!(s.observe(0, 0, 0.0).is_empty());
    s.boolean(c"muted", true);
    s.observe(0, 0, 0.0);
    s.boolean(c"muted", false);
    assert_eq!(s.observe(0, 0, 0.0), [(4, f64::from(0.4_f32), 0)]);
    s.boolean(c"paused", false);
    assert!(s.observe(0, 0, 0.0).contains(&(0, 0.0, 0)));
    s.boolean(c"paused", true);
    assert!(s.observe(0, 0, 0.0).contains(&(1, 0.0, 0)));
    s.boolean(c"paused", false);
    s.observe(0, 0, 0.0);
    s.number(c"seek", 3.0);
    assert!(s.observe(0, 0, 0.0).contains(&(2, 3.0, 1)));
    for (now, seconds) in [(1.0, 3.2), (1.016, 3.216), (1.032, 3.232)] {
        let clock = NuxVideoClockSample {
            generation: 1,
            seconds,
            rate: 1.0,
            playing: 1,
            available: 1,
        };
        ok(unsafe { nux_player_video_report_clock(s.player, 1, now, &clock) });
        s.step(0.0);
        assert_eq!(s.value(4).0, seconds as f32);
    }
    ok(unsafe { nux_player_video_command(s.player, 1, 1, 0.0, 0) });
    s.step(0.0);
    assert_eq!(s.value(6).1, 1);
    s.observe(3, 1, 0.0);
    s.step(0.0);
    assert_eq!(s.value(7).1, 1);
    assert_eq!(s.value(8).0, 6.0);
}

#[test]
fn linear_volume_key_reaches_decoder_at_half_a_second() {
    let s = VideoFixture::new(true);
    s.observe(1, 0, 12.5);
    s.step(0.5);
    assert!(s.observe(0, 0, 0.0).contains(&(4, 0.5, 0)));
    assert_eq!(s.value(9).0, 0.5);
}

#[test]
fn only_host_command_kinds_are_accepted_for_players_and_groups() {
    let a = VideoFixture::new(false);
    let b = VideoFixture::new(false);
    unsafe {
        let members = [
            NuxVideoSyncMember {
                player: a.player,
                component_id: 1,
            },
            NuxVideoSyncMember {
                player: b.player,
                component_id: 1,
            },
        ];
        let mut group = ptr::null_mut();
        ok(nux_video_sync_group_new(
            members.as_ptr(),
            2,
            0.06,
            0.25,
            &mut group,
        ));
        for kind in [0, 2, 3, 4, 5, 9] {
            assert_eq!(
                nux_player_video_command(a.player, 1, kind, 1.0, 0),
                NuxStatus::InvalidArgument
            );
            assert_eq!(
                nux_video_sync_group_command(group, kind, 1.0, 0),
                NuxStatus::InvalidArgument
            );
        }
        for kind in [1, 6, 7, 8] {
            ok(nux_player_video_command(a.player, 1, kind, 1.0, 2));
            ok(nux_video_sync_group_command(group, kind, 1.0, 2));
        }
        ok(nux_video_sync_group_free(group));
    }
}

#[test]
fn muted_binding_reaches_root_copy_and_list_row_decoder_queues() {
    unsafe extern "C" fn collect(data: *mut c_void, info: *const NuxVideoInfo) {
        unsafe { &mut *data.cast::<Vec<usize>>() }.push(unsafe { (*info).component_id });
    }
    unsafe {
        let file = import(&fixture::copies_and_rows());
        let (mut artboard, mut player, mut values) =
            (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        ok(nux_artboard_instance_new(file, 1, &mut artboard));
        ok(nux_player_new_default(artboard, &mut player));
        ok(nux_view_model_instance_new(file, 0, &mut values));
        let name = NuxStringView {
            data: c"Flags".as_ptr(),
            len: 5,
        };
        ok(nux_player_set_global_view_model(player, name, values));
        step(player, 0.0);
        let mut ids: Vec<usize> = Vec::new();
        ok(nux_player_visit_videos(
            player,
            Some(collect),
            ptr::from_mut(&mut ids).cast(),
        ));
        assert_eq!(ids.len(), 3);
        assert_eq!(
            ids.iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3
        );
        for id in &ids {
            observe(player, *id, 1, 0, 12.5);
        }
        for (muted, volume) in [(true, 0.0), (false, 1.0)] {
            ok(nux_view_model_instance_set_bool(
                values,
                c"muted".as_ptr(),
                muted,
            ));
            step(player, 0.0);
            for id in &ids {
                assert_eq!(
                    observe(player, *id, 0, 0, 0.0),
                    [(4, volume, 0)],
                    "occurrence {id}"
                );
            }
        }
        nux_player_free(player);
        nux_view_model_instance_free(values);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn reclaiming_a_decoder_publishes_opening_before_another_decoder_step() {
    let s = VideoFixture::new(false);
    s.observe(1, 0, 12.5);
    s.step(0.0);
    assert_eq!(s.value(8).0, 1.0);
    let clock = NuxVideoClockSample {
        generation: 0,
        seconds: 1.2,
        rate: 1.0,
        playing: 1,
        available: 1,
    };
    ok(unsafe { nux_player_video_report_clock(s.player, 1, 1.0, &clock) });
    s.step(0.0);
    assert_eq!(s.value(4).0, 1.2);
    let mut generation = 0;
    ok(unsafe { nux_player_video_reclaim_decoder(s.player, 1, 1, &mut generation) });
    assert_eq!(generation, 1);
    s.step(0.0);
    assert_eq!(s.value(8).0, 0.0);
    assert_eq!(s.value(4).0, 0.0);
}
