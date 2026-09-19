//! Real AVFoundation -> public C ABI -> Metal readback qualification.
//! Run on the main thread: cargo run -p nux-capi --features apple-metal --example video_apple
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod proof {
    use nux_capi::*;
    use nuxie_binary::{FixtureProperty as P, FixtureRecord as R, FixtureValue as V};
    use nuxie_runtime::video::playback::DecoderAction;
    use nuxie_video_host::apple::{ApplePlayer, Observation, pump_run_loop};
    use objc2::{
        rc::{Retained, autoreleasepool},
        runtime::ProtocolObject,
    };
    use objc2_core_foundation::CGSize;
    use objc2_metal::{MTLBuffer, MTLDevice, MTLPixelFormat, MTLResourceOptions};
    use objc2_quartz_core::CAMetalLayer;
    use std::{
        ffi::c_void,
        ptr,
        time::{Duration, Instant},
    };

    #[derive(Default)]
    struct Info {
        id: usize,
        generation: u64,
        state: u32,
        source: String,
    }
    unsafe extern "C" fn info(data: *mut c_void, view: *const NuxVideoInfo) {
        let view = unsafe { &*view };
        let source =
            unsafe { std::slice::from_raw_parts(view.source_key.data.cast(), view.source_key.len) };
        unsafe {
            *data.cast::<Info>() = Info {
                id: view.component_id,
                generation: view.generation,
                state: view.state,
                source: String::from_utf8_lossy(source).into_owned(),
            };
        }
    }
    unsafe extern "C" fn action(data: *mut c_void, value: *const NuxVideoAction) {
        let value = unsafe { &*value };
        let command = match value.kind {
            0 => DecoderAction::Play,
            1 => DecoderAction::Pause,
            2 => DecoderAction::Seek {
                seconds: value.value,
                generation: value.generation,
            },
            3 => DecoderAction::Rate(value.value as f32),
            4 => DecoderAction::Volume(value.value as f32),
            5 => DecoderAction::Dispose,
            _ => return,
        };
        unsafe { &mut *data.cast::<Vec<DecoderAction>>() }.push(command);
    }
    unsafe extern "C" fn caption(data: *mut c_void, _: NuxStringView, text: NuxStringView) {
        let bytes = unsafe { std::slice::from_raw_parts(text.data.cast(), text.len) };
        unsafe {
            *data.cast::<String>() = String::from_utf8_lossy(bytes).into_owned();
        }
    }
    fn ok(status: NuxStatus) {
        assert_eq!(status, NuxStatus::Ok);
    }
    fn scene(source: &str) -> Vec<u8> {
        let record = |type_key, props: Vec<(u16, V)>| R {
            type_key,
            properties: props
                .into_iter()
                .map(|(key, value)| P { key, value })
                .collect(),
        };
        nuxie_binary::encode_runtime_file(&nuxie_binary::RuntimeFile::from_fixture_records(vec![
        record(23, vec![]),
        record(60000, vec![(60000,V::String(source.into())),(208,V::Double(64.0)),(207,V::Double(32.0))]),
        record(1, vec![(7,V::Double(64.0)),(8,V::Double(32.0))]),
        record(60001, vec![(5,V::Uint(0)),(206,V::Uint(0)),(13,V::Double(32.0)),(14,V::Double(16.0)),
            (60013,V::String(r#"{"version":1,"language":"en","cues":[{"start":0,"end":1,"text":"Red scene"},{"start":1,"end":2,"text":"Blue scene"}]}"#.into()))]),
    ]).unwrap()).unwrap()
    }
    unsafe fn step(
        player: *mut NuxPlayer,
        id: usize,
        observation: u32,
        generation: u64,
        value: f64,
        decoder: &mut ApplePlayer,
    ) {
        let mut actions = Vec::new();
        ok(unsafe {
            nux_player_video_step(
                player,
                id,
                observation,
                generation,
                value,
                Some(action),
                ptr::from_mut(&mut actions).cast(),
            )
        });
        for action in actions {
            decoder.apply(action).unwrap();
        }
    }
    pub fn run_sync() {
        autoreleasepool(|_| unsafe {
            let capabilities = NuxVideoPlaybackCapabilities {
                playback_available: 1,
                ..Default::default()
            };
            let source = format!(
                "{}/../../fixtures/video/red-blue-sync.mp4",
                env!("CARGO_MANIFEST_DIR")
            );
            let bytes = scene(&source);
            let (mut renderer, mut result) = (ptr::null_mut(), ptr::null_mut());
            ok(nux_renderer_new_metal(64, 32, &mut renderer, &mut result));
            nux_capi_result_free(result);
            let mut players = [ptr::null_mut(); 2];
            let mut decoders = Vec::new();
            for player in &mut players {
                let (mut file, mut artboard) = (ptr::null_mut(), ptr::null_mut());
                ok(nux_file_import_metal(
                    renderer,
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxFileImportConfig {
                        video_playback: &capabilities,
                        ..Default::default()
                    },
                    &mut file,
                    &mut result,
                ));
                nux_capi_result_free(result);
                ok(nux_artboard_instance_new(file, 0, &mut artboard));
                ok(nux_player_new_static(artboard, player));
                ok(nux_artboard_instance_free(artboard));
                ok(nux_file_free(file));
                decoders.push(ApplePlayer::open(&source, 0, 1024 * 1024).unwrap());
                ok(nux_player_video_command(*player, 1, 5, 1.0, 0));
            }
            let members = players.map(|player| NuxVideoSyncMember {
                player,
                component_id: 1,
            });
            let mut group = ptr::null_mut();
            ok(nux_video_sync_group_new(
                members.as_ptr(),
                2,
                0.06,
                0.25,
                &mut group,
            ));
            ok(nux_player_video_command(players[0], 1, 0, 0.0, 0));
            let start = Instant::now();
            let mut follower_started = false;
            let mut frames = [0usize; 2];
            let mut blue = [false; 2];
            let mut corrections = 0;
            let mut maximum_drift = 0.0f64;
            let mut settled_peak = 0.0f64;
            let mut settled_since = None;
            unsafe extern "C" fn measured(data: *mut c_void, view: *const NuxVideoSyncMeasurement) {
                let view = unsafe { &*view };
                unsafe { &mut *data.cast::<Vec<(f64, bool)>>() }
                    .push((view.drift_seconds, view.corrected != 0));
            }
            loop {
                assert!(
                    start.elapsed() < Duration::from_secs(20),
                    "C ABI synchronization timed out: max={maximum_drift}, corrections={corrections}, frames={frames:?}"
                );
                pump_run_loop(0.01).unwrap();
                for index in 0..2 {
                    let player = players[index];
                    let decoder = &mut decoders[index];
                    step(player, 1, 0, 0, 0.0, decoder);
                    match decoder.poll().unwrap() {
                        Some(Observation::Ready {
                            generation,
                            duration,
                        }) => step(player, 1, 1, generation, duration, decoder),
                        Some(Observation::Playing(generation)) => {
                            step(player, 1, 2, generation, 0.0, decoder)
                        }
                        Some(Observation::Ended(generation)) => {
                            step(player, 1, 3, generation, 0.0, decoder)
                        }
                        Some(Observation::Frame(frame)) => {
                            let view = NuxVideoFrame {
                                struct_size: size_of::<NuxVideoFrame>() as u32,
                                generation: frame.generation,
                                presentation_seconds: frame.pts,
                                width: frame.width,
                                height: frame.height,
                                row_bytes: frame.width * 4,
                                pixels: NuxByteView {
                                    data: frame.rgba.as_ptr(),
                                    len: frame.rgba.len(),
                                },
                            };
                            ok(nux_player_video_present_metal(renderer, player, 1, &view));
                            if frame.pts > 1.1 {
                                assert!(frame.rgba[2] > 200 && frame.rgba[0] < 30);
                                blue[index] = true;
                            }
                            frames[index] += 1;
                        }
                        None => (),
                    }
                }
                if !follower_started
                    && decoders[0]
                        .clock()
                        .is_some_and(|clock| clock.playing && clock.seconds >= 0.4)
                {
                    ok(nux_player_video_command(players[1], 1, 0, 0.0, 0));
                    follower_started = true;
                }
                let samples: Vec<_> = decoders
                    .iter()
                    .map(|decoder| match decoder.clock() {
                        Some(clock) => NuxVideoClockSample {
                            generation: clock.generation,
                            seconds: clock.seconds,
                            rate: clock.rate,
                            playing: u32::from(clock.playing),
                            available: 1,
                        },
                        None => NuxVideoClockSample {
                            generation: 0,
                            seconds: 0.0,
                            rate: 0.0,
                            playing: 0,
                            available: 0,
                        },
                    })
                    .collect();
                let mut measurements = Vec::<(f64, bool)>::new();
                ok(nux_video_sync_group_update(
                    group,
                    start.elapsed().as_secs_f64(),
                    samples.as_ptr(),
                    samples.len(),
                    Some(measured),
                    ptr::from_mut(&mut measurements).cast(),
                ));
                if measurements.is_empty() {
                    settled_since = None;
                    settled_peak = 0.0;
                }
                for (drift, corrected) in measurements {
                    maximum_drift = maximum_drift.max(drift.abs());
                    if corrected {
                        corrections += 1;
                    }
                    if corrections > 0 && drift.abs() <= 0.06 {
                        settled_since.get_or_insert_with(Instant::now);
                        settled_peak = settled_peak.max(drift.abs());
                    } else {
                        settled_since = None;
                        settled_peak = 0.0;
                    }
                }
                if blue.iter().all(|value| *value)
                    && settled_since
                        .is_some_and(|time| time.elapsed() >= Duration::from_millis(300))
                {
                    break;
                }
            }
            assert!(
                maximum_drift > 0.25 && corrections > 0 && frames.iter().all(|count| *count >= 3)
            );
            ok(nux_video_sync_group_command(group, 8, 0.0, 0));
            for (player, decoder) in players.into_iter().zip(&mut decoders) {
                step(player, 1, 0, 0, 0.0, decoder);
                assert!(decoder.clock().is_none());
                ok(nux_player_free(player));
            }
            ok(nux_video_sync_group_free(group));
            ok(nux_renderer_free(renderer));
            println!(
                "PASS: public C ABI group -> two AVFoundation clocks -> Metal frame submission; injected drift={maximum_drift:.6}s, corrections={corrections}, settled peak={settled_peak:.6}s for >=300ms, frames={frames:?}, both decoders disposed"
            );
        });
    }
    pub fn run() {
        autoreleasepool(|_| unsafe {
            let capabilities = NuxVideoPlaybackCapabilities {
                playback_available: 1,
                ..Default::default()
            };
            let source = format!(
                "{}/../../fixtures/video/red-blue-audio.mp4",
                env!("CARGO_MANIFEST_DIR")
            );
            let bytes = scene(&source);
            let (mut renderer, mut result, mut file, mut artboard, mut player) = (
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            ok(nux_renderer_new_metal(64, 32, &mut renderer, &mut result));
            nux_capi_result_free(result);
            ok(nux_file_import_metal(
                renderer,
                bytes.as_ptr(),
                bytes.len(),
                &NuxFileImportConfig {
                    video_playback: &capabilities,
                    ..Default::default()
                },
                &mut file,
                &mut result,
            ));
            nux_capi_result_free(result);
            ok(nux_artboard_instance_new(file, 0, &mut artboard));
            ok(nux_player_new_static(artboard, &mut player));
            let mut catalog = Info::default();
            ok(nux_player_visit_videos(
                player,
                Some(info),
                ptr::from_mut(&mut catalog).cast(),
            ));
            assert_eq!(catalog.source, source);
            let mut decoder =
                ApplePlayer::open(&catalog.source, catalog.generation, 1024 * 1024).unwrap();
            ok(nux_player_video_command(player, catalog.id, 5, 1.0, 0));
            ok(nux_player_video_command(player, catalog.id, 0, 0.0, 0));
            let mut device_pointer = ptr::null_mut();
            ok(nux_renderer_copy_metal_device(
                renderer,
                &mut device_pointer,
                &mut result,
            ));
            nux_capi_result_free(result);
            let device: Retained<ProtocolObject<dyn MTLDevice>> =
                Retained::from_raw(device_pointer.cast()).unwrap();
            let layer = CAMetalLayer::new();
            layer.setDevice(Some(&device));
            layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            layer.setFramebufferOnly(false);
            layer.setDrawableSize(CGSize::new(64.0, 32.0));
            layer.setAllowsNextDrawableTimeout(true);
            let buffer = device
                .newBufferWithLength_options(256 * 32, MTLResourceOptions::StorageModeShared)
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            let (mut red, mut blue, mut frames) = (false, false, 0);
            let mut loops = 0;
            loop {
                assert!(Instant::now() < deadline, "C ABI playback timed out");
                pump_run_loop(0.01).unwrap();
                step(player, catalog.id, 0, catalog.generation, 0.0, &mut decoder);
                match decoder.poll().unwrap() {
                    Some(Observation::Ready {
                        generation,
                        duration,
                    }) => step(player, catalog.id, 1, generation, duration, &mut decoder),
                    Some(Observation::Playing(generation)) => {
                        step(player, catalog.id, 2, generation, 0.0, &mut decoder)
                    }
                    Some(Observation::Ended(generation)) => {
                        step(player, catalog.id, 3, generation, 0.0, &mut decoder)
                    }
                    Some(Observation::Frame(frame)) => {
                        let pixels = NuxVideoFrame {
                            struct_size: size_of::<NuxVideoFrame>() as u32,
                            generation: frame.generation,
                            presentation_seconds: frame.pts,
                            width: frame.width,
                            height: frame.height,
                            row_bytes: frame.width * 4,
                            pixels: NuxByteView {
                                data: frame.rgba.as_ptr(),
                                len: frame.rgba.len(),
                            },
                        };
                        ok(nux_player_video_present_metal(
                            renderer, player, catalog.id, &pixels,
                        ));
                        ok(nux_player_visit_videos(
                            player,
                            Some(info),
                            ptr::from_mut(&mut catalog).cast(),
                        ));
                        if catalog.generation != frame.generation {
                            continue;
                        }
                        let mut stepped = ptr::null_mut();
                        ok(nux_player_step(
                            player,
                            &NuxPlayerStep::default(),
                            &mut stepped,
                        ));
                        nux_player_step_result_free(stepped);
                        let drawable = layer.nextDrawable().expect("Metal drawable");
                        let op = NuxMetalRenderOperation {
                            drawable_state: NUX_METAL_DRAWABLE_STATE_AVAILABLE,
                            drawable: Retained::as_ptr(&drawable).cast_mut().cast(),
                            readback_buffer: Retained::as_ptr(&buffer).cast_mut().cast(),
                            readback_bytes_per_row: 256,
                            ..Default::default()
                        };
                        let mut outcome = NuxRendererOutcome::default();
                        ok(nux_renderer_render_player(
                            renderer,
                            player,
                            &op,
                            &mut outcome,
                            &mut result,
                        ));
                        assert_eq!(outcome.disposition, NUX_RENDERER_DISPOSITION_PRESENTED);
                        let captured = std::slice::from_raw_parts(
                            buffer.contents().as_ptr().cast::<u8>(),
                            256 * 32,
                        );
                        let pixel = &captured[16 * 256 + 32 * 4..16 * 256 + 32 * 4 + 4];
                        let mut text = String::new();
                        ok(nux_player_video_caption(
                            player,
                            catalog.id,
                            Some(caption),
                            ptr::from_mut(&mut text).cast(),
                        ));
                        if frame.pts < 0.9 {
                            assert!(pixel[2] > 200 && pixel[0] < 30, "BGRA red {pixel:?}");
                            assert_eq!(text, "Red scene");
                            if !red {
                                ok(nux_player_video_set_loop_range(
                                    player, catalog.id, 1.3, 1.7,
                                ));
                                ok(nux_player_video_command(player, catalog.id, 9, 1.0, 0));
                                ok(nux_player_video_command(player, catalog.id, 2, 1.3, 0));
                            }
                            red = true;
                        }
                        if frame.pts > 1.1 {
                            assert!(pixel[0] > 200 && pixel[2] < 30, "BGRA blue {pixel:?}");
                            assert_eq!(text, if frame.pts < 2.0 { "Blue scene" } else { "" });
                            blue = true;
                        }
                        frames += 1;
                    }
                    None => (),
                }
                ok(nux_player_visit_videos(
                    player,
                    Some(info),
                    ptr::from_mut(&mut catalog).cast(),
                ));
                let mut event = NuxVideoEvent { kind: 0, state: 0 };
                loop {
                    let status = nux_player_video_next_event(player, catalog.id, &mut event);
                    if status == NuxStatus::NotFound {
                        break;
                    }
                    ok(status);
                    if event.kind == 2 {
                        loops += 1;
                        if loops == 2 {
                            ok(nux_player_video_command(player, catalog.id, 9, 0.0, 0));
                        }
                    }
                }
                if catalog.state == 6 {
                    break;
                }
            }
            assert!(red && blue && frames >= 2);
            assert_eq!(loops, 2);
            ok(nux_player_video_command(player, catalog.id, 8, 0.0, 0));
            step(player, catalog.id, 0, catalog.generation, 0.0, &mut decoder);
            ok(nux_player_free(player));
            ok(nux_artboard_instance_free(artboard));
            ok(nux_file_free(file));
            ok(nux_renderer_free(renderer));
            println!(
                "PASS: public C ABI -> AVFoundation -> public Metal presentation/readback; {frames} frames, red/blue pixels, seek, two range loops, captions, completion, disposal"
            );
        });
    }
}
fn main() {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        proof::run();
        proof::run_sync();
    }
}
