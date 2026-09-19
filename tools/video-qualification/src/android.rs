use jni::{
    JNIEnv,
    objects::{JObject, JString},
};
use nuxie_render_api::PersistentFactory;
use nuxie_renderer::{NativeVulkanFactory, RenderMode};
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    source::{artboard::RuntimeArtboardInstanceHandle, core::CoreHandle},
    video::{Video, playback::*},
};
use nuxie_video_host::{android::AndroidScenePlayer, pool::ManagedPlayer};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};
struct Proof {
    factory: PersistentFactory<NativeVulkanFactory>,
    artboard: RuntimeArtboardInstanceHandle,
    video: CoreHandle,
    player: AndroidScenePlayer,
    clock: Instant,
    deadline: Instant,
    red: bool,
    blue: bool,
    loops: usize,
    rates_observed: u8,
    frames: usize,
    seeked: bool,
    pause_started: Option<Instant>,
    pause_position: Option<f64>,
    pause_drift: f64,
    pause_checked: bool,
    decoder_info: Option<String>,
}
impl Proof {
    fn new(env: &mut JNIEnv<'_>, activity: &JObject<'_>, source: &str, embedded: bool) -> Self {
        Self::with_sync(env, activity, source, embedded, false)
    }
    fn with_sync(
        env: &mut JNIEnv<'_>,
        activity: &JObject<'_>,
        source: &str,
        embedded: bool,
        sync: bool,
    ) -> Self {
        let mut factory = PersistentFactory::new(NativeVulkanFactory::new(64, 32).unwrap());
        let file = File::import(
            &super::video_scene_with_media(embedded.then_some(if sync {
                &include_bytes!("../../../fixtures/video/red-blue-sync.mp4")[..]
            } else {
                &include_bytes!("../../../fixtures/video/red-blue-audio.mp4")[..]
            })),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        video
            .with_downcast_mut::<Video, _>(|v| {
                v.playback = Playback::new(PlaybackSettings {
                    autoplay: true,
                    muted: true,
                    ..Default::default()
                })
            })
            .unwrap();
        let mut player = if embedded {
            AndroidScenePlayer::open_embedded(
                env,
                activity,
                video.clone(),
                std::path::Path::new(source).parent().unwrap(),
                1024 * 1024,
                1024 * 1024,
                5.0,
                true,
            )
        } else {
            AndroidScenePlayer::open(env, activity, video.clone(), source, 1024 * 1024, 5.0, true)
        }
        .unwrap();
        for mode in [
            nuxie_runtime::video::resources::Allocation::Hardware,
            nuxie_runtime::video::resources::Allocation::Software,
        ] {
            assert!(matches!(
                player.tick(env, mode, 0.0, |_| panic!("unsupported mode uploaded")),
                Err(nuxie_video_host::android::AndroidError::UnsupportedDecodeMode)
            ));
        }
        assert!(!player.owns_decoder());
        Self {
            factory,
            artboard,
            video,
            player,
            clock: Instant::now(),
            deadline: Instant::now() + Duration::from_secs(20),
            red: false,
            blue: false,
            loops: 0,
            rates_observed: 0,
            frames: 0,
            seeked: false,
            pause_started: None,
            pause_position: None,
            pause_drift: 0.0,
            pause_checked: false,
            decoder_info: None,
        }
    }
    fn caption(&self) -> String {
        self.video
            .with_downcast::<Video, _>(|v| {
                if v.has_video_frame() {
                    v.caption_text()
                } else {
                    String::new()
                }
            })
            .unwrap_or_default()
    }
    fn render_tick(&mut self, env: &mut JNIEnv<'_>) {
        use nuxie_runtime::video::resources::Allocation;
        let now = self.clock.elapsed().as_secs_f64();
        let mut pts = 0.0;
        let status = self
            .player
            .tick(env, Allocation::PlatformManaged, now, |frame| {
                pts = frame.pts;
                let image = self
                    .factory
                    .borrow()
                    .upload_canonical_rgba8_premul_srgb(
                        frame.width,
                        frame.height,
                        frame.width * 4,
                        &frame.rgba,
                    )
                    .unwrap();
                let image: std::rc::Rc<dyn nuxie_render_api::RenderImage> =
                    std::rc::Rc::from(image);
                let view = nuxie_render_api::Factory::make_gpu_canvas_image_view(
                    &mut *self.factory.borrow_mut(),
                    image.clone(),
                )
                .expect("decoded frame must be available to GPU image sampling");
                assert!(std::rc::Rc::ptr_eq(&image, &view));
                Ok(view)
            })
            .unwrap();
        if status.presented {
            self.artboard.update_pass(true);
            let mut render = self
                .factory
                .borrow()
                .begin_frame(0xff000000, RenderMode::Msaa)
                .unwrap();
            self.artboard.draw(&mut render);
            let pixels = render.finish().unwrap();
            super::verify_composition(&pixels).unwrap();
            let pixel = &pixels[(16 * 64 + 32) * 4..(16 * 64 + 32) * 4 + 4];
            if pts < 0.9 {
                assert!(pixel[0] > 200 && pixel[2] < 30, "red oracle: {pixel:?}");
                assert_eq!(self.caption(), "Red scene");
                self.red = true;
            }
            if pts > 1.1 {
                assert!(pixel[2] > 200 && pixel[0] < 30, "blue oracle: {pixel:?}");
                assert_eq!(self.caption(), if pts < 2.0 { "Blue scene" } else { "" });
                self.blue = true;
            }
            self.frames += 1;
            if let Some(info) = self.player.decoder_info().unwrap() {
                self.decoder_info = Some(format!("{info:?}"));
            }
        }
    }
    fn tick(&mut self, env: &mut JNIEnv<'_>) -> bool {
        use nuxie_runtime::video::resources::Allocation;
        let now = self.clock.elapsed().as_secs_f64();
        if let Some(start) = self.pause_started {
            if let Some(clock) = self.player.clock() {
                assert!(
                    !clock.playing && clock.rate == 0.0,
                    "suspended Android media is still playing"
                );
                let position = *self.pause_position.get_or_insert(clock.seconds);
                self.pause_drift = self.pause_drift.max((clock.seconds - position).abs());
                assert!(
                    self.pause_drift < 0.03,
                    "paused media clock advanced {} seconds",
                    self.pause_drift
                );
                if start.elapsed() >= Duration::from_millis(300) {
                    self.player
                        .set_suspended(SuspensionReason::Background, false)
                        .unwrap();
                    self.pause_checked = true;
                    self.pause_started = None;
                    println!(
                        "NUX_VIDEO_CLOCK Android pause drift={} seconds over >=300ms without decoder/frame ticks",
                        self.pause_drift
                    );
                }
            }
            assert!(
                start.elapsed() < Duration::from_secs(2),
                "Android native clock unavailable during pause"
            );
            return false;
        }
        if self.red && !self.pause_checked {
            self.player
                .set_suspended(SuspensionReason::Background, true)
                .unwrap();
            self.pause_started = Some(Instant::now());
            return false;
        }
        if self.red && !self.seeked {
            self.player
                .tick(env, Allocation::Poster, now, |_| {
                    panic!("denied decoder uploaded")
                })
                .unwrap();
            self.video
                .with_downcast_mut::<Video, _>(|v| {
                    v.playback.enqueue(Command::LoopRange {
                        start: 1.3,
                        end: 1.7,
                    })?;
                    v.playback.enqueue(Command::Rate(0.5))?;
                    v.playback.enqueue(Command::Loop(true))?;
                    v.playback.enqueue(Command::Seek(1.3))
                })
                .unwrap()
                .unwrap();
            self.seeked = true;
        }
        self.render_tick(env);
        // Sample the platform clock, independently of the requested settings.
        if let Some(clock) = self.player.clock().filter(|clock| clock.playing) {
            if (clock.rate - 0.5).abs() < 0.01 {
                self.rates_observed |= 1;
            }
            if (clock.rate - 1.5).abs() < 0.01 {
                self.rates_observed |= 2;
            }
        }
        self.video
            .with_downcast_mut::<Video, _>(|v| {
                while let Some(event) = v.playback.pop_event() {
                    if event == PlaybackEvent::Looped {
                        self.loops += 1;
                        if self.loops == 1 {
                            v.playback.enqueue(Command::Rate(1.5)).unwrap();
                        }
                        if self.loops == 2 {
                            v.playback.enqueue(Command::Rate(1.0)).unwrap();
                            v.playback.enqueue(Command::Loop(false)).unwrap();
                        }
                    }
                }
            })
            .unwrap();
        let state = self
            .video
            .with_downcast::<Video, _>(|v| v.playback.state())
            .unwrap();
        if state == PlaybackState::Ended {
            assert!(
                self.red
                    && self.blue
                    && self.seeked
                    && self.pause_checked
                    && self.frames >= 2
                    && self.loops == 2
                    && self.rates_observed == 3,
                "incomplete pixel/rate proof: red={}, blue={}, frames={}, rates={}",
                self.red,
                self.blue,
                self.frames,
                self.rates_observed
            );
            self.video
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Dispose))
                .unwrap()
                .unwrap();
            self.player
                .tick(env, Allocation::PlatformManaged, now, |_| {
                    panic!("disposed decoder uploaded")
                })
                .unwrap();
            return true;
        }
        assert!(
            Instant::now() < self.deadline,
            "video timeout: {state:?}, frames={}",
            self.frames
        );
        false
    }
}
struct SyncProof {
    leader: Proof,
    follower: Proof,
    group: std::rc::Rc<nuxie_video_host::sync::RegisteredSynchronizationGroup>,
    follower_generation: u64,
    start: Instant,
    started_follower: bool,
    corrections: usize,
    maximum_drift: f64,
    settled_peak: f64,
    settled_since: Option<Instant>,
}
impl SyncProof {
    fn new(env: &mut JNIEnv<'_>, activity: &JObject<'_>, source: &str, embedded: bool) -> Self {
        let leader = Proof::with_sync(env, activity, source, embedded, true);
        let follower = Proof::with_sync(env, activity, source, embedded, true);
        follower
            .video
            .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Pause))
            .unwrap()
            .unwrap();
        let group = nuxie_video_host::sync::RegisteredSynchronizationGroup::new(
            leader.video.clone(),
            vec![follower.video.clone()],
            0.06,
            0.25,
            std::rc::Rc::new(std::cell::Cell::new(true)),
        )
        .unwrap();
        Self {
            leader,
            follower,
            group,
            follower_generation: 0,
            start: Instant::now(),
            started_follower: false,
            corrections: 0,
            maximum_drift: 0.0,
            settled_peak: 0.0,
            settled_since: None,
        }
    }
    fn tick(&mut self, env: &mut JNIEnv<'_>) -> bool {
        let drift = self
            .leader
            .player
            .clock()
            .zip(self.follower.player.clock())
            .filter(|(leader, follower)| leader.playing && follower.playing)
            .map(|(leader, follower)| leader.seconds - follower.seconds);
        self.leader.render_tick(env);
        self.follower.render_tick(env);
        assert_eq!(self.group.take_error(), None);
        let generation = self
            .follower
            .video
            .with_downcast::<Video, _>(|v| v.playback.generation())
            .unwrap();
        if generation != self.follower_generation {
            self.corrections += 1;
            self.follower_generation = generation;
        }
        if !self.started_follower
            && self
                .leader
                .player
                .clock()
                .is_some_and(|c| c.playing && c.seconds >= 0.4)
        {
            self.follower
                .video
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Play))
                .unwrap()
                .unwrap();
            self.started_follower = true;
        }
        if let Some(drift) = drift {
            println!(
                "NUX_VIDEO_SYNC_AUTO Android t={:.6} drift={:.6} generation={}",
                self.start.elapsed().as_secs_f64(),
                drift,
                generation
            );
            self.maximum_drift = self.maximum_drift.max(drift.abs());
            if self.corrections > 0 && drift.abs() <= 0.06 {
                self.settled_since.get_or_insert_with(Instant::now);
                self.settled_peak = self.settled_peak.max(drift.abs());
            } else {
                self.settled_since = None;
                self.settled_peak = 0.0;
            }
        } else {
            self.settled_since = None;
            self.settled_peak = 0.0;
        }
        if self.leader.blue
            && self.follower.blue
            && self
                .settled_since
                .is_some_and(|s| s.elapsed() >= Duration::from_millis(300))
        {
            assert!(self.maximum_drift > 0.25 && self.corrections > 0);
            assert!(self.leader.frames >= 3 && self.follower.frames >= 3);
            self.group.command(Command::Dispose).unwrap();
            for proof in [&mut self.leader, &mut self.follower] {
                proof
                    .player
                    .tick(
                        env,
                        nuxie_runtime::video::resources::Allocation::PlatformManaged,
                        self.start.elapsed().as_secs_f64(),
                        |_| panic!("disposed sync member uploaded"),
                    )
                    .unwrap();
                assert!(!proof.player.owns_decoder());
            }
            return true;
        }
        assert!(
            Instant::now() < self.leader.deadline,
            "sync timeout: {}",
            self.metrics()
        );
        false
    }
    fn metrics(&self) -> String {
        format!(
            "injected drift={:.6}s; corrections={}; settled peak={:.6}s for >=300ms; frames={}/{}; owned decoders={}",
            self.maximum_drift,
            self.corrections,
            self.settled_peak,
            self.leader.frames,
            self.follower.frames,
            usize::from(self.leader.player.owns_decoder())
                + usize::from(self.follower.player.owns_decoder())
        )
    }
}
pub struct VulkanBenchmark {
    factory: PersistentFactory<NativeVulkanFactory>,
    artboard: RuntimeArtboardInstanceHandle,
    player: AndroidScenePlayer,
    start: Instant,
    first: Option<f64>,
    previous: Option<f64>,
    max_gap: f64,
    last_pts: f64,
    max_gap_pts: (f64, f64),
    costs: Vec<f64>,
    upload_costs: Vec<f64>,
    render_costs: Vec<f64>,
    first_decoded_ms: Option<f64>,
    max_poll_ms: f64,
    red: usize,
    blue: usize,
    preview: Vec<u8>,
    result: String,
    completed: bool,
    decoder_info: String,
}
impl VulkanBenchmark {
    pub fn new(env: &mut JNIEnv<'_>, activity: &JObject<'_>, source: &str) -> Self {
        let mut factory = PersistentFactory::new(NativeVulkanFactory::new(1280, 720).unwrap());
        let file = File::import(
            &super::video_scene_with_dimensions(None, 1280, 720),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        video
            .with_downcast_mut::<Video, _>(|v| {
                v.playback = Playback::new(PlaybackSettings {
                    autoplay: true,
                    looping: true,
                    muted: true,
                    ..Default::default()
                });
            })
            .unwrap();
        let player = AndroidScenePlayer::open(
            env,
            activity,
            video.clone(),
            source,
            8 * 1024 * 1024,
            5.0,
            true,
        )
        .unwrap();
        Self {
            factory,
            artboard,
            player,
            start: Instant::now(),
            first: None,
            previous: None,
            max_gap: 0.0,
            last_pts: 0.0,
            max_gap_pts: (0.0, 0.0),
            costs: Vec::new(),
            upload_costs: Vec::new(),
            render_costs: Vec::new(),
            first_decoded_ms: None,
            max_poll_ms: 0.0,
            red: 0,
            blue: 0,
            preview: Vec::new(),
            result: String::new(),
            completed: false,
            decoder_info: "unknown".into(),
        }
    }
    pub fn tick(&mut self, env: &mut JNIEnv<'_>) -> bool {
        if self.completed {
            return true;
        }
        if self.start.elapsed() >= Duration::from_secs(32) {
            self.player.reclaim().unwrap();
            assert!(!self.player.owns_decoder());
            assert!(self.costs.len() > 1, "no benchmark frames");
            let elapsed = self.previous.unwrap() - self.first.unwrap();
            let fps = (self.costs.len() - 1) as f64 / elapsed;
            self.costs.sort_by(f64::total_cmp);
            let p50 = self.costs[self.costs.len() / 2];
            let p95 = self.costs[self.costs.len() * 95 / 100];
            self.result = format!(
                "frames={} red={} blue={} fps={fps:.3} first_frame_ms={:.3} frame_work_p50_ms={p50:.3} frame_work_p95_ms={p95:.3} max_gap_ms={:.3} owned_decoders=0 platform_selected=true",
                self.costs.len(),
                self.red,
                self.blue,
                self.first.unwrap() * 1000.0,
                self.max_gap * 1000.0
            );
            self.result.push_str(&format!(
                " decoder={} max_gap_from_pts={:.6} max_gap_to_pts={:.6}",
                self.decoder_info, self.max_gap_pts.0, self.max_gap_pts.1
            ));
            self.upload_costs.sort_by(f64::total_cmp);
            self.render_costs.sort_by(f64::total_cmp);
            self.result.push_str(&format!(" first_decoded_ms={:.3} poll_max_ms={:.3} upload_p95_ms={:.3} upload_max_ms={:.3} render_p95_ms={:.3} render_max_ms={:.3} frame_work_max_ms={:.3}",
                self.first_decoded_ms.unwrap(), self.max_poll_ms,
                self.upload_costs[self.upload_costs.len() * 95 / 100], self.upload_costs.last().unwrap(),
                self.render_costs[self.render_costs.len() * 95 / 100], self.render_costs.last().unwrap(), self.costs.last().unwrap()));
            println!("NUX_VIDEO_BENCHMARK {}", self.result);
            assert!(
                self.red > 300 && self.blue > 300 && fps >= 24.0,
                "720p/30 sustained playback below qualification floor"
            );
            self.completed = true;
            return true;
        }
        let tick_start = Instant::now();
        let mut pixel = None;
        let mut pts = 0.0;
        let mut upload_ms = 0.0;
        let status = self
            .player
            .tick(
                env,
                nuxie_runtime::video::resources::Allocation::PlatformManaged,
                self.start.elapsed().as_secs_f64(),
                |frame| {
                    assert_eq!((frame.width, frame.height), (1280, 720));
                    pixel = Some(frame.rgba[..4].to_vec());
                    pts = frame.pts;
                    self.first_decoded_ms
                        .get_or_insert(self.start.elapsed().as_secs_f64() * 1000.0);
                    let upload_start = Instant::now();
                    let image = self
                        .factory
                        .borrow()
                        .upload_canonical_rgba8_premul_srgb(
                            frame.width,
                            frame.height,
                            frame.width * 4,
                            &frame.rgba,
                        )
                        .unwrap();
                    upload_ms = upload_start.elapsed().as_secs_f64() * 1000.0;
                    Ok(std::rc::Rc::from(image))
                },
            )
            .unwrap();
        self.max_poll_ms = self
            .max_poll_ms
            .max(tick_start.elapsed().as_secs_f64() * 1000.0 - upload_ms);
        if status.presented {
            self.upload_costs.push(upload_ms);
            let render_start = Instant::now();
            self.artboard.update_pass(true);
            if let Some(info) = self.player.decoder_info().unwrap() {
                self.decoder_info = format!("{info:?}");
            }
            let mut render = self
                .factory
                .borrow()
                .begin_frame(0xff000000, RenderMode::Msaa)
                .unwrap();
            nuxie_render_api::Renderer::scale(&mut render, 20.0, 22.5);
            self.artboard.draw(&mut render);
            let full_pixels = render.finish().unwrap();
            self.render_costs
                .push(render_start.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(full_pixels.len(), 1280 * 720 * 4);
            // Sample centers of the original oracle's cells after output scaling.
            let mut pixels = Vec::with_capacity(64 * 32 * 4);
            for y in 0..32 {
                for x in 0..64 {
                    let offset = (((y as f64 + 0.5) * 22.5) as usize * 1280 + x * 20 + 10) * 4;
                    pixels.extend_from_slice(&full_pixels[offset..offset + 4]);
                }
            }
            super::verify_composition(&pixels).unwrap();
            let pixel = pixel.unwrap();
            let center = &pixels[(16 * 64 + 32) * 4..(16 * 64 + 32) * 4 + 4];
            assert!(center[0].abs_diff(pixel[0]) < 8 && center[2].abs_diff(pixel[2]) < 8);
            if pixel[0] > 200 && pixel[2] < 30 {
                self.red += 1;
            } else if pixel[2] > 200 && pixel[0] < 30 {
                self.blue += 1;
            } else {
                panic!("unexpected fixture color {pixel:?}");
            }
            let now = self.start.elapsed().as_secs_f64();
            self.first.get_or_insert(now);
            if let Some(last) = self.previous {
                if now - last > self.max_gap {
                    self.max_gap = now - last;
                    self.max_gap_pts = (self.last_pts, pts);
                }
            }
            self.previous = Some(now);
            self.last_pts = pts;
            self.preview = pixels;
            self.costs.push(tick_start.elapsed().as_secs_f64() * 1000.0);
        }
        false
    }
}
enum AndroidProof {
    Single(Proof),
    Sync(SyncProof),
    Benchmark(VulkanBenchmark),
}
impl AndroidProof {
    fn tick(&mut self, env: &mut JNIEnv<'_>) -> bool {
        match self {
            Self::Single(p) => p.tick(env),
            Self::Sync(p) => p.tick(env),
            Self::Benchmark(p) => p.tick(env),
        }
    }
    fn caption(&self) -> String {
        match self {
            Self::Single(p) => p.caption(),
            Self::Sync(p) => p.follower.caption(),
            Self::Benchmark(_) => String::new(),
        }
    }
    fn metrics(&self) -> String {
        match self {
            Self::Single(p) => format!(
                "native clock pause checked={}; drift={:.6}s over >=300ms without decoder/frame ticks; frames={}; decoder={}",
                p.pause_checked,
                p.pause_drift,
                p.frames,
                p.decoder_info.as_deref().unwrap_or("unknown")
            ),
            Self::Sync(p) => p.metrics(),
            Self::Benchmark(p) => p.result.clone(),
        }
    }
}
struct OsPauseProbe {
    start: Instant,
    baseline: Option<f64>,
    samples: usize,
    drift: f64,
}
thread_local! {static OS_PAUSE: RefCell<Option<OsPauseProbe>> = const { RefCell::new(None) };}
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_lifecycleReady(
    _env: JNIEnv,
    _: JObject,
) -> jni::sys::jboolean {
    PROOF.with(|slot| match slot.borrow().as_ref() {
        Some(AndroidProof::Single(p)) => u8::from(p.frames >= 3 && p.pause_checked),
        _ => 0,
    })
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_lifecyclePause(
    mut env: JNIEnv,
    _: JObject,
    suspended: jni::sys::jboolean,
) {
    guard(&mut env, |_| {
        PROOF.with(|slot| {
            if let Some(AndroidProof::Single(p)) = slot.borrow_mut().as_mut() {
                if suspended == 0 {
                    OS_PAUSE.with(|probe| {
                        let probe = probe.borrow_mut().take().expect("OS pause started");
                        assert!(probe.start.elapsed() >= Duration::from_millis(500));
                        assert!(probe.samples >= 2, "no paused native clock samples");
                        assert!(probe.drift < 0.03, "OS background clock advanced");
                        println!(
                            "NUX_VIDEO_OS_LIFECYCLE paused samples={} drift={:.6}s elapsed={:.3}s",
                            probe.samples,
                            probe.drift,
                            probe.start.elapsed().as_secs_f64()
                        );
                    });
                } else {
                    OS_PAUSE.with(|probe| {
                        *probe.borrow_mut() = Some(OsPauseProbe {
                            start: Instant::now(),
                            baseline: None,
                            samples: 0,
                            drift: 0.0,
                        })
                    });
                }
                p.player
                    .set_suspended(SuspensionReason::Background, suspended != 0)
                    .unwrap();
            }
        });
        0
    });
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_lifecycleSample(
    mut env: JNIEnv,
    _: JObject,
) {
    guard(&mut env, |_| {
        PROOF.with(|slot| {
            if let Some(AndroidProof::Single(p)) = slot.borrow().as_ref() {
                if let Some(clock) = p.player.clock() {
                    OS_PAUSE.with(|probe| {
                        if let Some(probe) = probe.borrow_mut().as_mut() {
                            assert!(
                                !clock.playing && clock.rate == 0.0,
                                "OS background media still playing"
                            );
                            let baseline = *probe.baseline.get_or_insert(clock.seconds);
                            probe.drift = probe.drift.max((clock.seconds - baseline).abs());
                            probe.samples += 1;
                        }
                    });
                }
            }
        });
        0
    });
}
thread_local! {static PROOF:RefCell<Option<AndroidProof>>=const{RefCell::new(None)};}
fn guard(env: &mut JNIEnv<'_>, action: impl FnOnce(&mut JNIEnv<'_>) -> i32) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(env))) {
        Ok(value) => value,
        Err(error) => {
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap_or("Rust video proof failed");
            let _ = env.throw_new("java/lang/IllegalStateException", message);
            -1
        }
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_open(
    mut env: JNIEnv,
    activity: JObject,
    source: JString,
    embedded: jni::sys::jboolean,
    sync: jni::sys::jboolean,
    benchmark: jni::sys::jboolean,
) -> i32 {
    guard(&mut env, |env| {
        let path: String = env.get_string(&source).unwrap().into();
        let proof = if benchmark != 0 {
            AndroidProof::Benchmark(VulkanBenchmark::new(env, &activity, &path))
        } else if sync != 0 {
            AndroidProof::Sync(SyncProof::new(env, &activity, &path, embedded != 0))
        } else {
            AndroidProof::Single(Proof::new(env, &activity, &path, embedded != 0))
        };
        PROOF.with(|slot| *slot.borrow_mut() = Some(proof));
        0
    })
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_tick(
    mut env: JNIEnv,
    _: JObject,
) -> i32 {
    guard(&mut env, |env| {
        PROOF.with(|slot| {
            if slot.borrow_mut().as_mut().expect("open proof").tick(env) {
                1
            } else {
                0
            }
        })
    })
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_close(
    mut env: JNIEnv,
    _: JObject,
) {
    guard(&mut env, |_| {
        PROOF.with(|slot| *slot.borrow_mut() = None);
        0
    });
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_caption(
    env: JNIEnv,
    _: JObject,
) -> jni::sys::jstring {
    let text = PROOF.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(AndroidProof::caption)
            .unwrap_or_default()
    });
    env.new_string(text)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_metrics(
    env: JNIEnv,
    _: JObject,
) -> jni::sys::jstring {
    let text = PROOF.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(AndroidProof::metrics)
            .unwrap_or_default()
    });
    env.new_string(text)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_nuxie_videoqualification_MainActivity_preview(
    env: JNIEnv,
    _: JObject,
) -> jni::sys::jbyteArray {
    PROOF.with(|slot| {
        let proof = slot.borrow();
        let Some(AndroidProof::Benchmark(p)) = proof.as_ref() else {
            return std::ptr::null_mut();
        };
        if p.preview.is_empty() {
            return std::ptr::null_mut();
        }
        env.byte_array_from_slice(&p.preview)
            .map(|a| a.into_raw())
            .unwrap_or(std::ptr::null_mut())
    })
}
