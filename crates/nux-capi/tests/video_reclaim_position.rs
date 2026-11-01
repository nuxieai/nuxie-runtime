//! A reclaimed decoder resumes at the position the video kept (UNIV-5927).
//!
//! A host that can no longer show a video closes its decoder and calls
//! nux_player_video_reclaim_decoder; when it can show the video again it opens
//! a new decoder with the returned generation. The video keeps its position
//! through the reclaim, and the new decoder's ready observation seeks to it
//! before playing. A new platform player starts at time 0, and its first
//! picture can reach the runtime before the host has seen it ready (the iOS
//! SDK's tick reads the player's status before its frame path). That picture
//! must not replace the kept position. A first open has no kept position and
//! seeks nowhere.
use nuxie::video::playback::{DecoderAction, Playback, PlaybackSettings, SuspensionReason};

const FPS: f64 = 30.0;

/// VideoPlayer's <video autoplay loop muted>.
fn clip_settings() -> PlaybackSettings {
    PlaybackSettings {
        autoplay: true,
        looping: true,
        muted: true,
        ..Default::default()
    }
}

/// A decoder that opened, played and presented clip frames 0..=last.
fn playing_through(last: u32) -> Playback {
    let mut playback = Playback::new(clip_settings());
    let generation = playback.generation();
    let opened = playback.opened(generation, 4.0);
    assert!(opened.contains(&DecoderAction::Play), "{opened:?}");
    playback.observed_playing(generation);
    for frame in 0..=last {
        assert!(playback.accept_frame(generation, f64::from(frame) / FPS));
    }
    playback
}

/// What nux_player_video_reclaim_decoder does to playback for one call: a new
/// generation, then the allocation's resource suspension (a poster suspends,
/// a platform-managed decoder resumes).
fn reclaim(playback: &mut Playback, blocked: bool) -> u64 {
    let generation = playback.reclaim_decoder().unwrap();
    let _ = playback.update_suspension(SuspensionReason::Resources, blocked);
    generation
}

/// The host closes the decoder (the pool answers poster), then admits the
/// video again and opens a new decoder.
fn closed_and_reopened(playback: &mut Playback) -> u64 {
    reclaim(playback, true);
    reclaim(playback, false)
}

fn seeks(actions: &[DecoderAction]) -> Vec<f64> {
    actions
        .iter()
        .filter_map(|action| match action {
            DecoderAction::Seek { seconds, .. } => Some(*seconds),
            _ => None,
        })
        .collect()
}

fn assert_seeks_then_plays(actions: &[DecoderAction]) {
    let seek = actions
        .iter()
        .position(|action| matches!(action, DecoderAction::Seek { .. }));
    let play = actions
        .iter()
        .position(|action| *action == DecoderAction::Play);
    assert!(
        matches!((seek, play), (Some(seek), Some(play)) if seek < play),
        "seek, then play: {actions:?}"
    );
}

#[test]
fn a_reopened_decoder_ready_first_seeks_to_the_kept_position() {
    let mut playback = playing_through(54);
    let kept = playback.position();
    assert_eq!(kept, 54.0 / FPS);
    let before = playback.generation();
    let reopened = closed_and_reopened(&mut playback);
    assert_eq!(reopened, before + 2);
    assert_eq!(playback.position(), kept, "the reclaim keeps the position");
    let actions = playback.opened(reopened, 4.0);
    assert_eq!(seeks(&actions), vec![kept], "{actions:?}");
    assert_seeks_then_plays(&actions);
}

#[test]
fn an_early_first_picture_does_not_replace_the_kept_position() {
    let mut playback = playing_through(57);
    let kept = playback.position();
    assert_eq!(kept, 57.0 / FPS);
    let reopened = closed_and_reopened(&mut playback);
    // The new decoder starts at time 0, and its first picture arrives before
    // the host has seen it ready.
    let accepted = playback.accept_frame(reopened, 0.0);
    println!(
        "early picture: accepted {accepted}, position {}, state {:?}",
        playback.position(),
        playback.state()
    );
    let actions = playback.opened(reopened, 4.0);
    assert_eq!(
        seeks(&actions),
        vec![kept],
        "ready still seeks to the kept position: {actions:?}"
    );
    assert_seeks_then_plays(&actions);
}

#[test]
fn a_first_open_has_no_kept_position_and_does_not_seek() {
    // Ready before any picture.
    let mut playback = Playback::new(clip_settings());
    let generation = playback.generation();
    let actions = playback.opened(generation, 4.0);
    assert!(seeks(&actions).is_empty(), "{actions:?}");
    assert!(actions.contains(&DecoderAction::Play), "{actions:?}");

    // Its first picture before ready, as a reopened decoder's can be.
    let mut playback = Playback::new(clip_settings());
    let generation = playback.generation();
    assert!(
        playback.accept_frame(generation, 0.0),
        "a first open shows its first picture before ready"
    );
    let actions = playback.opened(generation, 4.0);
    assert!(seeks(&actions).is_empty(), "{actions:?}");
    assert!(actions.contains(&DecoderAction::Play), "{actions:?}");
}

/// The same paths through the C entry points, in the order the iOS SDK's video
/// host calls them: a player advance before every tick, admission's
/// observation-0 step before each pool answer, the clock report after every
/// step, the Hidden suspend and resume commands, and IOSurface-backed
/// CVPixelBuffers like the ones AVPlayerItemVideoOutput hands over.
#[cfg(all(feature = "apple-metal", any(target_os = "ios", target_os = "macos")))]
mod through_the_c_api {
    use super::FPS;
    use nux_capi::*;
    use nuxie_binary::{FixtureProperty as P, FixtureRecord as R, FixtureValue as V};
    use std::{ffi::c_void, ptr};

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        static kCFTypeDictionaryKeyCallBacks: [usize; 6];
        static kCFTypeDictionaryValueCallBacks: [usize; 5];
        static kCFBooleanTrue: *const c_void;
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            count: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        fn CFRelease(object: *const c_void);
    }
    #[link(name = "CoreVideo", kind = "framework")]
    unsafe extern "C" {
        static kCVPixelBufferIOSurfacePropertiesKey: *const c_void;
        static kCVPixelBufferMetalCompatibilityKey: *const c_void;
        fn CVPixelBufferCreate(
            allocator: *const c_void,
            width: usize,
            height: usize,
            format: u32,
            attributes: *const c_void,
            out: *mut *mut c_void,
        ) -> i32;
    }
    struct PixelBuffer(*mut c_void);
    impl PixelBuffer {
        fn new() -> Self {
            unsafe {
                let dictionary = |keys: &[*const c_void], values: &[*const c_void]| {
                    CFDictionaryCreate(
                        ptr::null(),
                        keys.as_ptr(),
                        values.as_ptr(),
                        keys.len() as isize,
                        ptr::addr_of!(kCFTypeDictionaryKeyCallBacks).cast(),
                        ptr::addr_of!(kCFTypeDictionaryValueCallBacks).cast(),
                    )
                };
                let empty = dictionary(&[], &[]);
                let attributes = dictionary(
                    &[
                        kCVPixelBufferIOSurfacePropertiesKey,
                        kCVPixelBufferMetalCompatibilityKey,
                    ],
                    &[empty, kCFBooleanTrue],
                );
                let mut buffer = ptr::null_mut();
                let status = CVPixelBufferCreate(
                    ptr::null(),
                    64,
                    32,
                    u32::from_be_bytes(*b"BGRA"),
                    attributes,
                    &mut buffer,
                );
                CFRelease(attributes);
                CFRelease(empty);
                assert_eq!(status, 0);
                Self(buffer)
            }
        }
    }
    impl Drop for PixelBuffer {
        fn drop(&mut self) {
            unsafe { CFRelease(self.0) };
        }
    }

    /// One autoplaying, looping, muted video (component 1) on a 64x32 artboard.
    fn scene() -> Vec<u8> {
        let uint = |key, value| P {
            key,
            value: V::Uint(value),
        };
        let double = |key, value| P {
            key,
            value: V::Double(value),
        };
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
                            value: V::String("assets/standin-clip.mp4".into()),
                        },
                        double(208, 64.0),
                        double(207, 32.0),
                    ],
                },
                R {
                    type_key: 1,
                    properties: vec![double(7, 64.0), double(8, 32.0)],
                },
                R {
                    type_key: 60001,
                    properties: vec![
                        uint(60003, 1),
                        P {
                            key: 60004,
                            value: V::Bool(true),
                        },
                        P {
                            key: 60005,
                            value: V::Bool(true),
                        },
                        uint(5, 0),
                        uint(206, 0),
                    ],
                },
            ])
            .unwrap(),
        )
        .unwrap()
    }

    /// generation, state, position, wants play
    type Info = (u64, u32, f64, u32);
    /// kind (play 0, pause 1, seek 2, rate 3, volume 4, dispose 5), value, generation
    type Action = (u32, f64, u64);
    const PLAY: u32 = 0;
    const PAUSE: u32 = 1;
    const SEEK: u32 = 2;
    /// nux_player_video_command's suspend-reason command, and its Hidden reason.
    const SUSPEND_REASON: u32 = 6;
    const HIDDEN: u32 = 1;

    #[derive(Default)]
    struct Seen {
        infos: Vec<Info>,
        actions: Vec<Action>,
    }
    unsafe extern "C" fn info(data: *mut c_void, info: *const NuxVideoInfo) {
        let seen = unsafe { &mut *data.cast::<Seen>() };
        let info = unsafe { &*info };
        seen.infos.push((
            info.generation,
            info.state,
            info.position_seconds,
            info.wants_play,
        ));
    }
    unsafe extern "C" fn action(data: *mut c_void, action: *const NuxVideoAction) {
        let seen = unsafe { &mut *data.cast::<Seen>() };
        let action = unsafe { &*action };
        seen.actions
            .push((action.kind, action.value, action.generation));
    }

    fn seeks(actions: &[Action]) -> Vec<f64> {
        actions
            .iter()
            .filter(|action| action.0 == SEEK)
            .map(|action| action.1)
            .collect()
    }

    fn assert_seeks_to_then_plays(actions: &[Action], kept: f64) {
        let seeks = seeks(actions);
        assert_eq!(seeks.len(), 1, "one seek: {actions:?}");
        assert!(
            (seeks[0] - kept).abs() < 1e-9,
            "seeks to the kept {kept}: {actions:?}"
        );
        let seek = actions.iter().position(|action| action.0 == SEEK);
        let play = actions.iter().position(|action| action.0 == PLAY);
        assert!(
            matches!((seek, play), (Some(seek), Some(play)) if seek < play),
            "seek, then play: {actions:?}"
        );
    }

    struct Host {
        renderer: *mut NuxRenderer,
        file: *mut NuxFile,
        artboard: *mut NuxArtboardInstance,
        player: *mut NuxPlayer,
        buffer: PixelBuffer,
        seen: Box<Seen>,
        now: f64,
        log: Vec<String>,
    }
    impl Host {
        unsafe fn open() -> Self {
            unsafe {
                let (mut renderer, mut result) = (ptr::null_mut(), ptr::null_mut());
                assert_eq!(
                    nux_renderer_new_metal(64, 32, &mut renderer, &mut result),
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
                Self {
                    renderer,
                    file,
                    artboard,
                    player,
                    buffer: PixelBuffer::new(),
                    seen: Box::default(),
                    now: 1.0,
                    log: Vec::new(),
                }
            }
        }
        /// nux_player_visit_videos, as the host reads the video's state.
        unsafe fn info(&mut self) -> Info {
            self.seen.infos.clear();
            let context = ptr::from_mut(&mut *self.seen).cast();
            assert_eq!(
                unsafe { nux_player_visit_videos(self.player, Some(info), context) },
                NuxStatus::Ok
            );
            assert_eq!(self.seen.infos.len(), 1);
            self.seen.infos[0]
        }
        /// The page's player step, before its video tick.
        unsafe fn advance(&mut self, elapsed: f32) {
            self.now += f64::from(elapsed);
            let step = NuxPlayerStep {
                struct_size: size_of::<NuxPlayerStep>() as u32,
                correlation_id: 0,
                inputs: ptr::null(),
                input_count: 0,
                pointers: ptr::null(),
                pointer_count: 0,
                elapsed_seconds: elapsed,
                focus_inputs: ptr::null(),
                focus_input_count: 0,
            };
            let mut result = ptr::null_mut();
            unsafe {
                assert_eq!(
                    nux_player_step(self.player, &step, &mut result),
                    NuxStatus::Ok
                );
                assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            }
        }
        /// nux_player_video_step with one decoder observation.
        unsafe fn step(&mut self, observation: u32, generation: u64, value: f64) -> Vec<Action> {
            self.seen.actions.clear();
            let context = ptr::from_mut(&mut *self.seen).cast();
            assert_eq!(
                unsafe {
                    nux_player_video_step(
                        self.player,
                        1,
                        observation,
                        generation,
                        value,
                        Some(action),
                        context,
                    )
                },
                NuxStatus::Ok
            );
            let actions = std::mem::take(&mut self.seen.actions);
            self.log.push(format!(
                "step(obs {observation}, gen {generation}) -> {actions:?}"
            ));
            actions
        }
        unsafe fn command(&mut self, kind: u32, value: f64, reason: u32) {
            assert_eq!(
                unsafe { nux_player_video_command(self.player, 1, kind, value, reason) },
                NuxStatus::Ok
            );
            self.log.push(format!(
                "command(kind {kind}, value {value}, reason {reason})"
            ));
        }
        /// The clock report after a tick; available is ready and not seeking.
        unsafe fn clock(&mut self, generation: u64, seconds: f64, playing: bool, available: bool) {
            let sample = NuxVideoClockSample {
                generation,
                seconds,
                rate: if playing { 1.0 } else { 0.0 },
                playing: u32::from(playing),
                available: u32::from(available),
            };
            assert_eq!(
                unsafe { nux_player_video_report_clock(self.player, 1, self.now, &sample) },
                NuxStatus::Ok
            );
        }
        unsafe fn present(&mut self, generation: u64, seconds: f64) {
            let frame = NuxVideoPixelBufferFrame {
                struct_size: size_of::<NuxVideoPixelBufferFrame>() as u32,
                generation,
                presentation_seconds: seconds,
                pixel_buffer: self.buffer.0,
            };
            assert_eq!(
                unsafe {
                    nux_player_video_present_metal_pixel_buffer(
                        self.renderer,
                        self.player,
                        1,
                        &frame,
                    )
                },
                NuxStatus::Ok
            );
            self.log
                .push(format!("present(gen {generation}, pts {seconds:.4})"));
        }
        unsafe fn reclaim(&mut self, blocked: bool) -> u64 {
            let mut generation = 0;
            assert_eq!(
                unsafe {
                    nux_player_video_reclaim_decoder(
                        self.player,
                        1,
                        u32::from(blocked),
                        &mut generation,
                    )
                },
                NuxStatus::Ok
            );
            self.log
                .push(format!("reclaim(blocked {blocked}) -> gen {generation}"));
            generation
        }
        unsafe fn frame_shown(&mut self) -> bool {
            let mut value = 99;
            assert_eq!(
                unsafe { nux_player_video_readiness(self.player, 1, 0.0, 2.0, 0, &mut value) },
                NuxStatus::Ok
            );
            value == 1
        }
        /// The first decoder: ready, playing, then clip frames 0..=last at
        /// 30 fps, one page step each (advance, admission's step, the tick's
        /// playing step, the clock report, the picture). Returns its
        /// generation and the position it kept.
        unsafe fn play_through(&mut self, last: u32) -> (u64, f64) {
            unsafe {
                let (generation, _, _, _) = self.info();
                self.advance(1.0 / 60.0);
                self.step(0, generation, 0.0);
                let ready = self.step(1, generation, 4.0);
                assert!(
                    ready.iter().any(|a| a.0 == PLAY),
                    "autoplay plays: {ready:?}"
                );
                self.clock(generation, 0.0, false, true);
                for frame in 0..=last {
                    let pts = f64::from(frame) / FPS;
                    self.advance(1.0 / 30.0);
                    self.step(0, generation, 0.0);
                    self.step(2, generation, 0.0);
                    self.clock(generation, pts, true, true);
                    self.present(generation, pts);
                }
                let (_, _, kept, _) = self.info();
                assert!((kept - f64::from(last) / FPS).abs() < 1e-9, "{kept}");
                self.log.clear();
                (generation, kept)
            }
        }
        /// The host stops showing the video and the pool answers poster:
        /// admission's step, reclaim(blocked 1), zero-delta page steps while
        /// it is closed. Then the pool admits the video again: reclaim(blocked
        /// 0), the Hidden resume command to the new decoder, and a tick that
        /// sees it not ready yet.
        unsafe fn close_and_reopen(&mut self, generation: u64) -> u64 {
            unsafe {
                self.step(0, generation, 0.0);
                let blocked = self.reclaim(true);
                self.step(0, blocked, 0.0);
                for _ in 0..5 {
                    self.advance(0.0);
                    self.step(0, blocked, 0.0);
                }
                let during = self.info();
                let shown = self.frame_shown();
                self.log
                    .push(format!("closed: info {during:?}, frame shown {shown}"));
                self.step(0, blocked, 0.0);
                let reopened = self.reclaim(false);
                self.command(SUSPEND_REASON, 0.0, HIDDEN);
                self.step(0, reopened, 0.0);
                let tick = self.step(0, reopened, 0.0);
                assert!(tick.is_empty(), "nothing to do before ready: {tick:?}");
                self.clock(reopened, 0.0, false, false);
                let after = self.info();
                let shown = self.frame_shown();
                self.log.push(format!(
                    "reopened, before any picture: info {after:?}, frame shown {shown}"
                ));
                assert!(
                    !shown,
                    "the reopened video has no picture until one is presented"
                );
                reopened
            }
        }
        fn dump(&self, name: &str) {
            for line in &self.log {
                println!("[{name}] {line}");
            }
        }
    }
    impl Drop for Host {
        fn drop(&mut self) {
            unsafe {
                nux_player_free(self.player);
                nux_artboard_instance_free(self.artboard);
                nux_file_free(self.file);
                assert_eq!(nux_renderer_free(self.renderer), NuxStatus::Ok);
            }
        }
    }

    #[test]
    fn a_reopened_decoder_ready_first_seeks_to_the_kept_position() {
        unsafe {
            let mut host = Host::open();
            let (generation, kept) = host.play_through(54);
            let reopened = host.close_and_reopen(generation);
            host.advance(1.0 / 60.0);
            host.step(0, reopened, 0.0);
            let ready = host.step(1, reopened, 4.0);
            host.dump("ready first");
            assert_seeks_to_then_plays(&ready, kept);
        }
    }

    #[test]
    fn an_early_first_picture_does_not_replace_the_kept_position() {
        unsafe {
            let mut host = Host::open();
            let (generation, kept) = host.play_through(57);
            let reopened = host.close_and_reopen(generation);
            // A tick that read the player as not ready finds the new output's
            // first picture (time 0) in its frame path.
            host.advance(1.0 / 60.0);
            host.step(0, reopened, 0.0);
            host.step(0, reopened, 0.0);
            host.clock(reopened, 0.0, false, false);
            host.present(reopened, 0.0);
            let early = host.info();
            let shown = host.frame_shown();
            host.log.push(format!(
                "after the early picture: info {early:?}, frame shown {shown}"
            ));
            host.advance(1.0 / 60.0);
            host.step(0, reopened, 0.0);
            let ready = host.step(1, reopened, 4.0);
            host.dump("early picture");
            assert_seeks_to_then_plays(&ready, kept);
        }
    }

    #[test]
    fn a_first_open_does_not_seek_after_an_early_picture() {
        unsafe {
            let mut host = Host::open();
            let (generation, _, _, _) = host.info();
            host.advance(1.0 / 60.0);
            host.step(0, generation, 0.0);
            host.clock(generation, 0.0, false, false);
            host.present(generation, 0.0);
            let early = host.info();
            let shown = host.frame_shown();
            host.log.push(format!(
                "after the early picture: info {early:?}, frame shown {shown}"
            ));
            host.advance(1.0 / 60.0);
            host.step(0, generation, 0.0);
            let ready = host.step(1, generation, 4.0);
            host.dump("first open");
            assert!(seeks(&ready).is_empty(), "no seek: {ready:?}");
            assert!(ready.iter().any(|a| a.0 == PLAY), "plays: {ready:?}");
        }
    }

    /// The control: a decoder the host keeps while the video is hidden pauses
    /// and resumes in place. No reclaim, so no seek and no new generation.
    #[test]
    fn a_decoder_kept_while_hidden_neither_seeks_nor_resets() {
        unsafe {
            let mut host = Host::open();
            let (generation, kept) = host.play_through(57);
            host.step(0, generation, 0.0);
            host.command(SUSPEND_REASON, 1.0, HIDDEN);
            let paused = host.step(0, generation, 0.0);
            host.clock(generation, kept, false, true);
            for _ in 0..5 {
                host.advance(0.0);
                host.step(0, generation, 0.0);
                host.step(0, generation, 0.0);
            }
            let during = host.info();
            let shown_during = host.frame_shown();
            host.log.push(format!(
                "kept, while hidden: info {during:?}, frame shown {shown_during}"
            ));
            host.step(0, generation, 0.0);
            host.command(SUSPEND_REASON, 0.0, HIDDEN);
            let resumed = host.step(0, generation, 0.0);
            host.clock(generation, kept, false, true);
            let after = host.info();
            host.log.push(format!("kept, shown again: info {after:?}"));
            host.dump("kept");
            assert!(
                paused.iter().any(|a| a.0 == PAUSE),
                "Hidden pauses: {paused:?}"
            );
            assert!(
                resumed.iter().any(|a| a.0 == PLAY),
                "showing again plays: {resumed:?}"
            );
            assert!(
                !paused.iter().chain(&resumed).any(|a| a.0 == SEEK),
                "no seek either way: {paused:?} {resumed:?}"
            );
            assert_eq!(after.0, generation, "same decoder generation");
            assert!((after.2 - kept).abs() < 1e-9, "position kept: {after:?}");
            assert!(shown_during, "the kept picture stays on the video");
        }
    }
}
