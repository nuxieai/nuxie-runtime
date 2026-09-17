use nux_capi::*;
use nuxie_binary::{FixtureProperty as P, FixtureRecord as R, FixtureValue as V};
use std::{ffi::c_void, ptr};

unsafe fn import_video(
    bytes: *const u8,
    len: usize,
    callbacks: *const NuxRenderCallbacks,
    out_file: *mut *mut NuxFile,
) -> NuxStatus {
    let capabilities = NuxVideoPlaybackCapabilities {
        playback_available: 1,
        ..Default::default()
    };
    let mut result = ptr::null_mut();
    let status = unsafe {
        nux_file_import_with_video_capabilities(
            bytes,
            len,
            callbacks,
            &capabilities,
            out_file,
            &mut result,
        )
    };
    unsafe {
        nux_capi_result_free(result);
    }
    status
}

fn scene() -> Vec<u8> {
    nuxie_binary::encode_runtime_file(
        &nuxie_binary::RuntimeFile::from_fixture_records(vec![
            R {
                type_key: 23,
                properties: vec![],
            },
            R {
                type_key: 60000,
                properties: vec![
                    P {
                        key: 60000,
                        value: V::String("assets/greeting.mp4".into()),
                    },
                    P {
                        key: 208,
                        value: V::Double(64.0),
                    },
                    P {
                        key: 207,
                        value: V::Double(32.0),
                    },
                ],
            },
            R {
                type_key: 1,
                properties: vec![
                    P {
                        key: 7,
                        value: V::Double(64.0),
                    },
                    P {
                        key: 8,
                        value: V::Double(32.0),
                    },
                ],
            },
            R {
                type_key: 60001,
                properties: vec![
                    P {
                        key: 5,
                        value: V::Uint(0),
                    },
                    P {
                        key: 206,
                        value: V::Uint(0),
                    },
                ],
            },
        ])
        .unwrap(),
    )
    .unwrap()
}
#[derive(Default)]
struct Probe {
    player: *const NuxPlayer,
    infos: Vec<(usize, u64, u32, String)>,
    actions: Vec<(u32, f64, u64)>,
    reentry: Option<NuxStatus>,
}
unsafe extern "C" fn info(data: *mut c_void, info: *const NuxVideoInfo) {
    let probe = unsafe { &mut *data.cast::<Probe>() };
    let info = unsafe { &*info };
    let source =
        unsafe { std::slice::from_raw_parts(info.source_key.data.cast(), info.source_key.len) };
    probe.infos.push((
        info.component_id,
        info.generation,
        info.state,
        String::from_utf8_lossy(source).into_owned(),
    ));
    probe.reentry =
        Some(unsafe { nux_player_video_command(probe.player, info.component_id, 0, 0.0, 0) });
}
unsafe extern "C" fn action(data: *mut c_void, action: *const NuxVideoAction) {
    let probe = unsafe { &mut *data.cast::<Probe>() };
    let action = unsafe { &*action };
    probe
        .actions
        .push((action.kind, action.value, action.generation));
}
#[test]
fn c_bridge_uses_live_occurrence_and_rejects_reentry_wrong_thread_and_stale_decoder() {
    let bytes = scene();
    let (mut file, mut artboard, mut player) = (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
    unsafe {
        assert_eq!(
            import_video(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file
            ),
            NuxStatus::Ok
        );
        let mut descriptor = NuxFileAssetDescriptorView::default();
        assert_eq!(
            nux_file_asset_descriptor(file, 0, &mut descriptor),
            NuxStatus::Ok
        );
        assert_eq!(descriptor.kind, NUX_FILE_ASSET_KIND_VIDEO);
        assert_eq!(
            descriptor.required_provider_flags,
            NUX_FILE_ASSET_PROVIDER_VIDEO_PLAYBACK
        );
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
        // The player retains the actual scene after its initial handles close.
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
        let mut probe = Probe {
            player,
            ..Default::default()
        };
        let context = ptr::from_mut(&mut probe).cast();
        assert_eq!(
            nux_player_visit_videos(player, Some(info), context),
            NuxStatus::Ok
        );
        assert_eq!(probe.infos, vec![(1, 0, 0, "assets/greeting.mp4".into())]);
        assert_eq!(probe.reentry, Some(NuxStatus::ReentrantCall));
        let address = player as usize;
        assert_eq!(
            std::thread::spawn(move || nux_player_video_command(
                address as *const NuxPlayer,
                1,
                0,
                0.0,
                0
            ))
            .join()
            .unwrap(),
            NuxStatus::WrongThread
        );
        assert_eq!(
            nux_player_video_command(player, 99, 0, 0.0, 0),
            NuxStatus::NotFound
        );
        assert_eq!(
            nux_player_video_command(player, 1, 5, 2.0, 0),
            NuxStatus::InvalidArgument
        );
        assert_eq!(
            nux_player_video_command(player, 1, 2, f64::NAN, 0),
            NuxStatus::InvalidArgument
        );
        assert_eq!(
            nux_player_video_command(player, 1, 0, 0.0, 0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_video_step(player, 1, 1, 0, 2.0, Some(action), context),
            NuxStatus::Ok
        );
        assert_eq!(probe.actions, vec![(3, 1.0, 0), (4, 1.0, 0), (0, 0.0, 0)]);
        let mut event = NuxVideoEvent {
            kind: 99,
            state: 99,
        };
        assert_eq!(
            nux_player_video_next_event(player, 1, &mut event),
            NuxStatus::Ok
        );
        assert_eq!((event.kind, event.state), (0, 1));
        assert_eq!(
            nux_player_video_next_event(player, 1, &mut event),
            NuxStatus::NotFound
        );
        for (limited, edge) in [(1.0, true), (1.0, false), (0.0, true)] {
            assert_eq!(
                nux_player_video_command(player, 1, 6, limited, 8),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_video_step(player, 1, 0, 0, 0.0, Some(action), context),
                NuxStatus::Ok
            );
            let mut resource_events = Vec::new();
            while nux_player_video_next_event(player, 1, &mut event) == NuxStatus::Ok {
                if event.kind == 5 {
                    resource_events.push(event.state);
                }
            }
            assert_eq!(
                resource_events,
                if edge { vec![limited as u32] } else { vec![] }
            );
        }
        probe.actions.clear();
        assert_eq!(
            nux_player_video_command(player, 1, 2, 1.5, 0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_video_step(player, 1, 0, 0, 0.0, Some(action), context),
            NuxStatus::Ok
        );
        assert_eq!(probe.actions, vec![(2, 1.5, 1)]);
        assert_eq!(
            nux_player_video_step(player, 1, 2, 0, 0.0, Some(action), context),
            NuxStatus::Ok
        );
        probe.infos.clear();
        assert_eq!(
            nux_player_visit_videos(player, Some(info), context),
            NuxStatus::Ok
        );
        assert_eq!(probe.infos[0].1, 1);
        assert_eq!(probe.infos[0].2, 4); // stale playing did not complete the seek
        assert_eq!(nux_player_free(player), NuxStatus::Ok);
        assert_eq!(
            nux_player_video_command(player, 1, 0, 0.0, 0),
            NuxStatus::HandleMismatch
        );
    }
}

#[cfg(all(feature = "apple-metal", any(target_os = "ios", target_os = "macos")))]
#[test]
fn metal_video_frames_validate_renderer_domain_dimensions_and_generation() {
    unsafe {
        let (mut renderer, mut other, mut result) =
            (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        assert_eq!(
            nux_renderer_new_metal(64, 32, &mut renderer, &mut result),
            NuxStatus::Ok
        );
        nux_capi_result_free(result);
        assert_eq!(
            nux_renderer_new_metal(64, 32, &mut other, &mut result),
            NuxStatus::Ok
        );
        nux_capi_result_free(result);
        let bytes = scene();
        let capabilities = NuxVideoPlaybackCapabilities {
            playback_available: 1,
            ..Default::default()
        };
        let (mut file, mut artboard, mut player) =
            (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        assert_eq!(
            nux_file_import_metal(
                renderer,
                bytes.as_ptr(),
                bytes.len(),
                &NuxFileImportConfig {
                    video_playback: &capabilities,
                    ..Default::default()
                },
                &mut file,
                &mut result
            ),
            NuxStatus::Ok
        );
        nux_capi_result_free(result);
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
        let pixels = [255u8, 0, 0, 255].repeat(64 * 32);
        let mut frame = NuxVideoFrame {
            struct_size: std::mem::size_of::<NuxVideoFrame>() as u32,
            generation: 0,
            presentation_seconds: 0.5,
            width: 64,
            height: 32,
            row_bytes: 64 * 4,
            pixels: NuxByteView {
                data: pixels.as_ptr(),
                len: pixels.len(),
            },
        };
        assert_eq!(
            nux_player_video_present_metal(other, player, 1, &frame),
            NuxStatus::HandleMismatch
        );
        assert_eq!(
            nux_player_video_present_metal(renderer, player, 1, &frame),
            NuxStatus::Ok
        );
        frame.row_bytes = 1;
        assert_eq!(
            nux_player_video_present_metal(renderer, player, 1, &frame),
            NuxStatus::InvalidArgument
        );
        frame.row_bytes = 64 * 4;
        frame.generation = 999;
        frame.presentation_seconds = 1.5;
        assert_eq!(
            nux_player_video_present_metal(renderer, player, 1, &frame),
            NuxStatus::Ok
        );
        let mut probe = Probe {
            player,
            ..Default::default()
        };
        assert_eq!(
            nux_player_visit_videos(player, Some(info), ptr::from_mut(&mut probe).cast()),
            NuxStatus::Ok
        );
        assert_eq!(probe.infos[0].1, 0);
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
        assert_eq!(nux_renderer_free(renderer), NuxStatus::Ok);
        assert_eq!(nux_renderer_free(other), NuxStatus::Ok);
    }
}

unsafe extern "C" fn caption(data: *mut c_void, language: NuxStringView, text: NuxStringView) {
    let output = unsafe { &mut *data.cast::<(String, String)>() };
    let copy = |view: NuxStringView| {
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(view.data.cast(), view.len) })
            .into_owned()
    };
    *output = (copy(language), copy(text));
}
fn text_view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}
#[test]
fn captions_are_copied_atomically_and_follow_media_seek() {
    unsafe {
        let bytes = scene();
        let (mut file, mut artboard, mut player) =
            (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        assert_eq!(
            import_video(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file
            ),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
        let first = String::from("Hello");
        let cues = [
            NuxVideoCaptionCue {
                start_seconds: 0.0,
                end_seconds: 1.0,
                text: text_view(&first),
            },
            NuxVideoCaptionCue {
                start_seconds: 1.0,
                end_seconds: 2.0,
                text: text_view("Again"),
            },
        ];
        assert_eq!(
            nux_player_video_set_captions(player, 1, text_view("en"), cues.as_ptr(), cues.len()),
            NuxStatus::Ok
        );
        drop(first);
        let mut output = (String::new(), String::new());
        let target = ptr::from_mut(&mut output).cast();
        assert_eq!(
            nux_player_video_caption(player, 1, Some(caption), target),
            NuxStatus::Ok
        );
        assert_eq!(output, ("en".into(), "Hello".into()));
        let bad = NuxVideoCaptionCue {
            start_seconds: f64::NAN,
            end_seconds: 1.0,
            text: text_view("invalid"),
        };
        assert_eq!(
            nux_player_video_set_captions(player, 1, text_view("en"), &bad, 1),
            NuxStatus::InvalidArgument
        );
        assert_eq!(
            nux_player_video_set_captions(player, 1, text_view("en"), ptr::dangling(), 100_001),
            NuxStatus::LimitExceeded
        );
        assert_eq!(
            nux_player_video_caption(player, 1, Some(caption), target),
            NuxStatus::Ok
        );
        assert_eq!(output.1, "Hello");
        let mut probe = Probe {
            player,
            ..Default::default()
        };
        assert_eq!(
            nux_player_video_command(player, 1, 2, 1.0, 0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_video_step(
                player,
                1,
                0,
                0,
                0.0,
                Some(action),
                ptr::from_mut(&mut probe).cast()
            ),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_video_caption(player, 1, Some(caption), target),
            NuxStatus::Ok
        );
        assert_eq!(output.1, "Again");
        assert_eq!(
            nux_player_video_set_captions(player, 1, NuxStringView::default(), ptr::null(), 0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_video_caption(player, 1, Some(caption), target),
            NuxStatus::Ok
        );
        assert_eq!(output, (String::new(), String::new()));
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn synchronization_c_api_uses_native_clocks_and_retains_thread_affine_occurrences() {
    unsafe fn revision(player: *mut NuxPlayer) -> u64 {
        let step = NuxPlayerStep {
            struct_size: size_of::<NuxPlayerStep>() as u32,
            correlation_id: 0,
            inputs: ptr::null(),
            input_count: 0,
            pointers: ptr::null(),
            pointer_count: 0,
            elapsed_seconds: 0.0,
        };
        let mut result = ptr::null_mut();
        let mut scheduling = NuxPlayerSchedulingInfo::default();
        unsafe {
            assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
            assert_eq!(
                nux_player_step_result_scheduling(result, &mut scheduling),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
        }
        scheduling.render_revision
    }
    unsafe fn make_player() -> *mut NuxPlayer {
        let bytes = scene();
        let (mut file, mut artboard, mut player) =
            (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        unsafe {
            assert_eq!(
                import_video(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
        }
        player
    }
    struct SyncProbe {
        group: *mut NuxVideoSyncGroup,
        measurements: Vec<(usize, f64, u32)>,
        reentry: Option<NuxStatus>,
    }
    unsafe extern "C" fn measured(data: *mut c_void, view: *const NuxVideoSyncMeasurement) {
        let probe = unsafe { &mut *data.cast::<SyncProbe>() };
        let view = unsafe { &*view };
        probe
            .measurements
            .push((view.member_index, view.drift_seconds, view.corrected));
        probe.reentry = Some(unsafe { nux_video_sync_group_free(probe.group) });
    }
    unsafe {
        let leader = make_player();
        let follower = make_player();
        assert_eq!(
            nux_player_video_report_clock(leader, 1, 0.0, ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_video_report_clock(leader, 1, f64::NAN, ptr::null()),
            NuxStatus::InvalidArgument
        );
        let invalid_clock = NuxVideoClockSample {
            generation: 0,
            seconds: 0.0,
            rate: 1.0,
            playing: 2,
            available: 1,
        };
        assert_eq!(
            nux_player_video_report_clock(leader, 1, 0.0, &invalid_clock),
            NuxStatus::InvalidArgument
        );
        let members = [
            NuxVideoSyncMember {
                player: leader,
                component_id: 1,
            },
            NuxVideoSyncMember {
                player: follower,
                component_id: 1,
            },
        ];
        let mut group = ptr::null_mut();
        assert_eq!(
            nux_video_sync_group_new(members.as_ptr(), 1, 0.06, 0.25, &mut group),
            NuxStatus::InvalidArgument
        );
        assert!(group.is_null());
        let duplicate = [members[0], members[0]];
        assert_eq!(
            nux_video_sync_group_new(duplicate.as_ptr(), 2, 0.06, 0.25, &mut group),
            NuxStatus::InvalidArgument
        );
        assert!(group.is_null());
        assert_eq!(
            nux_video_sync_group_new(members.as_ptr(), 2, f64::NAN, 0.25, &mut group),
            NuxStatus::InvalidArgument
        );
        assert_eq!(
            nux_video_sync_group_new(members.as_ptr(), 2, 0.06, 0.25, &mut group),
            NuxStatus::Ok
        );
        let address = group as usize;
        assert_eq!(
            std::thread::spawn(move || nux_video_sync_group_command(
                address as *const NuxVideoSyncGroup,
                0,
                0.0,
                0
            ))
            .join()
            .unwrap(),
            NuxStatus::WrongThread
        );
        let before = [revision(leader), revision(follower)];
        assert_eq!([revision(leader), revision(follower)], before);
        assert_eq!(
            nux_video_sync_group_command(group, 0, 0.0, 0),
            NuxStatus::Ok
        );
        assert!(revision(leader) > before[0]);
        assert!(revision(follower) > before[1]);
        let mut actions = Probe::default();
        let action_context = ptr::from_mut(&mut actions).cast();
        for player in [leader, follower] {
            assert_eq!(
                nux_player_video_step(player, 1, 1, 0, 10.0, Some(action), action_context),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_video_step(player, 1, 2, 0, 0.0, Some(action), action_context),
                NuxStatus::Ok
            );
        }
        let mut samples = [
            NuxVideoClockSample {
                generation: 0,
                seconds: 1.0,
                rate: 1.0,
                playing: 1,
                available: 1,
            },
            NuxVideoClockSample {
                generation: 0,
                seconds: 0.5,
                rate: 1.0,
                playing: 1,
                available: 1,
            },
        ];
        let mut probe = SyncProbe {
            group,
            measurements: Vec::new(),
            reentry: None,
        };
        let context = ptr::from_mut(&mut probe).cast();
        samples[1].seconds = f64::NAN;
        assert_eq!(
            nux_video_sync_group_update(group, 1.0, samples.as_ptr(), 2, Some(measured), context),
            NuxStatus::InvalidArgument
        );
        assert_eq!(
            nux_video_sync_group_update(group, 1.0, samples.as_ptr(), 65, None, ptr::null_mut()),
            NuxStatus::InvalidArgument
        );
        samples[1].seconds = 0.5;
        // A stale decoder cannot trigger a correction.
        samples[1].generation = 99;
        assert_eq!(
            nux_video_sync_group_update(group, 1.0, samples.as_ptr(), 2, Some(measured), context),
            NuxStatus::Ok
        );
        assert!(probe.measurements.is_empty());
        samples[1].generation = 0;
        assert_eq!(
            nux_video_sync_group_update(group, 1.0, samples.as_ptr(), 2, Some(measured), context),
            NuxStatus::Ok
        );
        assert_eq!(probe.measurements, vec![(1, -0.5, 1)]);
        assert_eq!(probe.reentry, Some(NuxStatus::ReentrantCall));
        actions.actions.clear();
        assert_eq!(
            nux_player_video_step(follower, 1, 0, 0, 0.0, Some(action), action_context),
            NuxStatus::Ok
        );
        assert_eq!(actions.actions, vec![(2, 1.0, 1)]);
        assert_eq!(
            nux_video_sync_group_update(group, 0.9, samples.as_ptr(), 2, None, ptr::null_mut()),
            NuxStatus::InvalidArgument
        );
        assert_eq!(
            nux_video_sync_group_set_loop_range(group, 4.0, 2.0),
            NuxStatus::InvalidArgument
        );
        assert_eq!(nux_player_free(leader), NuxStatus::Ok);
        assert_eq!(nux_player_free(follower), NuxStatus::Ok);
        // The group owns the scene lifetime after both public player handles close.
        assert_eq!(
            nux_video_sync_group_command(group, 1, 0.0, 0),
            NuxStatus::Ok
        );
        assert_eq!(nux_video_sync_group_free(group), NuxStatus::Ok);
        assert_eq!(
            nux_video_sync_group_command(group, 0, 0.0, 0),
            NuxStatus::HandleMismatch
        );
        assert_eq!(nux_video_sync_group_free(ptr::null_mut()), NuxStatus::Ok);
    }
}

#[test]
fn legacy_imports_reject_video_and_capability_records_are_validated() {
    let bytes = scene();
    let callbacks = NuxRenderCallbacks::default();
    unsafe {
        let (mut file, mut result) = (ptr::null_mut(), ptr::null_mut());
        assert_eq!(
            nux_file_import(bytes.as_ptr(), bytes.len(), &callbacks, &mut file),
            NuxStatus::ImportError
        );
        assert!(file.is_null());
        assert_eq!(
            nux_file_import_with_result(
                bytes.as_ptr(),
                bytes.len(),
                &callbacks,
                &mut file,
                &mut result
            ),
            NuxStatus::ImportError
        );
        assert!(file.is_null());
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);
        let mut capabilities = NuxVideoPlaybackCapabilities::default();
        for (size, available, expected) in [
            (4, 1, NuxStatus::InvalidStructSize),
            (
                size_of::<NuxVideoPlaybackCapabilities>() as u32,
                2,
                NuxStatus::InvalidArgument,
            ),
            (
                size_of::<NuxVideoPlaybackCapabilities>() as u32,
                0,
                NuxStatus::ImportError,
            ),
            (
                size_of::<NuxVideoPlaybackCapabilities>() as u32,
                1,
                NuxStatus::Ok,
            ),
        ] {
            capabilities.struct_size = size;
            capabilities.playback_available = available;
            assert_eq!(
                nux_file_import_with_video_capabilities(
                    bytes.as_ptr(),
                    bytes.len(),
                    &callbacks,
                    &capabilities,
                    &mut file,
                    &mut result
                ),
                expected
            );
            assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);
            if expected == NuxStatus::Ok {
                assert_eq!(nux_file_free(file), NuxStatus::Ok);
            } else {
                assert!(file.is_null());
            }
        }
    }
}
