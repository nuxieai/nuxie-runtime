use nuxie_render_api::PersistentFactory;
use nuxie_renderer::NativeMetalFactory;
use nuxie_runtime::video::resources::{Allocation, DecoderBudget, DecoderRequest};
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    source::{artboard::RuntimeArtboardInstanceHandle, core::CoreHandle},
    video::{Video, playback::*},
};
use nuxie_video_host::apple::{AppleScenePlayer, AudioSessionOwnership, pump_run_loop};
use nuxie_video_host::pool::{ManagedPlayer, PlayerPool};
use std::time::{Duration, Instant};

pub struct MetalProof {
    factory: PersistentFactory<NativeMetalFactory>,
    artboard: RuntimeArtboardInstanceHandle,
    video: CoreHandle,
    player: AppleScenePlayer,
    deadline: Instant,
    suspended_at: Option<Instant>,
    os_pause_position: Option<f64>,
    os_pause_checked: bool,
    red: bool,
    blue: bool,
    loops: usize,
    rates_observed: u8,
    frames: usize,
    seeked: bool,
    pause_probe: Option<(Instant, f64)>,
    pause_checked: bool,
    pause_drift: f64,
    replaced: bool,
    replacement_source: String,
    audible: bool,
    preview: Vec<u8>,
}
impl MetalProof {
    pub fn new(source: &str) -> Self {
        Self::with_audio(source, false)
    }
    pub fn with_audio(source: &str, audible: bool) -> Self {
        Self::with_media(source, audible, false)
    }
    pub fn with_media(source: &str, audible: bool, embedded: bool) -> Self {
        let bytes = embedded.then(|| std::fs::read(source).unwrap());
        let mut factory = PersistentFactory::new(NativeMetalFactory::new(64, 32).unwrap());
        let file = File::import(
            &super::video_scene_with_media(bytes.as_deref()),
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
                    muted: !audible,
                    volume: 0.2,
                    audio_policy: AudioPolicy::Mix as u32,
                    ..Default::default()
                })
            })
            .unwrap();
        let mut player = if embedded {
            AppleScenePlayer::open_embedded(
                video.clone(),
                &std::env::temp_dir(),
                1024 * 1024,
                1024 * 1024,
                AudioSessionOwnership::RuntimeManaged,
            )
        } else {
            AppleScenePlayer::open_with_audio(
                video.clone(),
                source,
                1024 * 1024,
                AudioSessionOwnership::RuntimeManaged,
            )
        }
        .unwrap();
        for mode in [Allocation::Hardware, Allocation::Software] {
            assert!(matches!(
                player.tick_allocated(mode, |_| panic!("unsupported mode uploaded")),
                Err(nuxie_video_host::apple::AppleError::UnsupportedDecodeMode)
            ));
        }
        assert!(!player.owns_decoder());
        Self {
            factory,
            artboard,
            video,
            player,
            deadline: Instant::now() + Duration::from_secs(15),
            suspended_at: None,
            os_pause_position: None,
            os_pause_checked: false,
            red: false,
            blue: false,
            loops: 0,
            rates_observed: 0,
            frames: 0,
            seeked: false,
            pause_probe: None,
            pause_checked: false,
            pause_drift: 0.0,
            replaced: false,
            replacement_source: source.into(),
            audible,
            preview: Vec::new(),
        }
    }
    pub fn set_backgrounded(&mut self, suspended: bool) {
        if !suspended {
            if let (Some(start), Some(position)) =
                (self.suspended_at, self.os_pause_position.take())
            {
                let clock = self.player.clock().expect("paused native clock");
                let drift = (clock.seconds - position).abs();
                assert!(
                    !clock.playing && clock.rate == 0.0 && drift < 0.03,
                    "iOS background clock advanced: {drift}"
                );
                if start.elapsed() >= Duration::from_millis(500) {
                    self.os_pause_checked = true;
                    println!(
                        "NUX_VIDEO_OS_LIFECYCLE Apple paused {:.3}s, native clock drift={:.6}s",
                        start.elapsed().as_secs_f64(),
                        drift
                    );
                }
            }
        }
        self.player
            .set_suspended(SuspensionReason::Background, suspended)
            .unwrap();
        if suspended {
            self.suspended_at.get_or_insert_with(Instant::now);
            self.os_pause_position = self.player.clock().map(|clock| clock.seconds);
        } else if let Some(start) = self.suspended_at.take() {
            self.deadline += start.elapsed();
        }
    }
    pub fn caption(&self) -> String {
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
    fn render_tick(&mut self, allocation: Allocation) {
        let mut decoded = None;
        if self
            .player
            .tick_allocated(allocation, |frame| {
                assert_eq!((frame.width, frame.height), (64, 32));
                decoded = Some((frame.pts, frame.rgba[..4].to_vec()));
                let image = self
                    .factory
                    .borrow()
                    .upload_rgba8_premul_srgb(
                        frame.width,
                        frame.height,
                        frame.width * 4,
                        &frame.rgba,
                    )
                    .unwrap();
                Ok(std::rc::Rc::from(image))
            })
            .unwrap()
            .presented
        {
            let (pts, pixel) = decoded.unwrap();
            let mut render = self.factory.borrow().begin_frame(0xff000000).unwrap();
            self.artboard.draw(&mut render);
            let rendered = render.finish().unwrap();
            super::verify_composition(&rendered).unwrap();
            let center = &rendered[(16 * 64 + 32) * 4..(16 * 64 + 32) * 4 + 4];
            assert!(
                center[0].abs_diff(pixel[0]) < 8 && center[2].abs_diff(pixel[2]) < 8,
                "rendered={center:?}, decoded={pixel:?}"
            );
            self.preview = rendered;
            self.frames += 1;
            if pts < 0.9 {
                assert!(pixel[0] > 200 && pixel[2] < 30);
                assert_eq!(self.caption(), "Red scene");
                self.red = true;
            }
            if pts > 1.1 {
                assert!(pixel[2] > 200 && pixel[0] < 30);
                assert_eq!(self.caption(), if pts < 2.0 { "Blue scene" } else { "" });
                self.blue = true;
            }
        }
    }
    pub fn tick(&mut self) -> bool {
        self.tick_allocated(Allocation::PlatformManaged)
    }
    fn tick_allocated(&mut self, allocation: Allocation) -> bool {
        if let Some((start, position)) = self.pause_probe {
            let clock = self
                .player
                .clock()
                .expect("paused native clock must remain available");
            assert!(!clock.playing, "native player continued during suspension");
            self.pause_drift = self.pause_drift.max((clock.seconds - position).abs());
            assert!(
                self.pause_drift < 0.03,
                "suspended media clock advanced {} seconds",
                self.pause_drift
            );
            if start.elapsed() < Duration::from_millis(250) {
                return false;
            }
            self.player
                .set_suspended(SuspensionReason::Background, false)
                .unwrap();
            self.pause_probe = None;
            self.pause_checked = true;
        }
        if self.red && !self.pause_checked && self.player.clock().is_some_and(|clock| clock.playing)
        {
            self.player
                .set_suspended(SuspensionReason::Background, true)
                .unwrap();
            let position = self.player.clock().unwrap().seconds;
            self.pause_probe = Some((Instant::now(), position));
            return false;
        }
        if self.red && !self.replaced {
            let (old_generation, settings) = self
                .video
                .with_downcast::<Video, _>(|v| (v.playback.generation(), v.playback.settings()))
                .unwrap();
            let generation = self
                .player
                .replace_source(&self.replacement_source, settings)
                .unwrap();
            assert!(generation > old_generation);
            self.video
                .with_downcast_mut::<Video, _>(|v| {
                    assert!(!v.has_video_frame());
                    assert!(!v.playback.accept_frame(old_generation, 1.5));
                })
                .unwrap();
            self.red = false;
            self.replaced = true;
        }
        if self.red && !self.seeked {
            let before = self
                .video
                .with_downcast::<Video, _>(|v| v.playback.generation())
                .unwrap();
            self.player
                .tick_allocated(nuxie_runtime::video::resources::Allocation::Poster, |_| {
                    panic!("denied player must not upload")
                })
                .unwrap();
            assert!(
                self.video
                    .with_downcast::<Video, _>(
                        |v| v.playback.generation() > before && !v.has_video_frame()
                    )
                    .unwrap()
            );
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
        self.render_tick(allocation);
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
                    && self.replaced
                    && self.pause_checked
                    && self.frames >= 3
                    && self.loops == 2
                    && self.rates_observed == 3
            );
            self.video
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Dispose))
                .unwrap()
                .unwrap();
            assert!(
                !self
                    .player
                    .tick(|_| panic!("disposed player must not upload"))
                    .unwrap()
            );
            println!(
                "NUX_VIDEO_CLOCK pause drift={} seconds over >=250ms without decoder/frame ticks",
                self.pause_drift
            );
            println!(
                "NUX_VIDEO_PROOF passed: {} frames, red/blue Metal pixel oracle, source replacement, seek, two range loops, decoder reclamation/reopen, completion, disposal; audible requested={}; acceleration unknown",
                self.frames, self.audible
            );
            return true;
        }
        assert!(
            Instant::now() < self.deadline,
            "playback timeout: state={state:?}, frames={}",
            self.frames
        );
        false
    }
}
impl ManagedPlayer for MetalProof {
    type Error = nuxie_video_host::apple::AppleError;
    fn allocation(&self) -> Allocation {
        self.player.allocation()
    }
    fn owns_decoder(&self) -> bool {
        self.player.owns_decoder()
    }
    fn can_decode(&self) -> bool {
        self.player.can_decode()
    }
    fn reclaim(&mut self) -> Result<(), Self::Error> {
        self.player.reclaim()
    }
}

pub struct MetalPoolProof {
    pool: PlayerPool<MetalProof>,
    phase: u8,
    completed: usize,
    peak: usize,
    preview: Vec<u8>,
    caption: String,
}
fn pool_request(id: u64, priority: u32) -> DecoderRequest {
    DecoderRequest {
        id,
        priority,
        visible: true,
        hardware_supported: false,
        managed_supported: true,
        software_supported: false,
        pixels_per_second: 64 * 32 * 30,
    }
}
impl MetalPoolProof {
    fn new(source: &str, audible: bool, embedded: bool) -> Self {
        let mut pool = PlayerPool::default();
        for id in [1, 2] {
            assert!(
                pool.insert(
                    pool_request(id, 0),
                    MetalProof::with_media(source, audible, embedded)
                )
                .is_ok()
            );
        }
        Self {
            pool,
            phase: 0,
            completed: 0,
            peak: 0,
            preview: Vec::new(),
            caption: String::new(),
        }
    }
    fn tick(&mut self) -> bool {
        let slots = if self.phase < 2 { 1 } else { 2 };
        let results = self
            .pool
            .tick(
                DecoderBudget {
                    max_players: slots,
                    hardware_players: 0,
                    managed_players: slots,
                    managed_pixels_per_second: slots as u64 * 64 * 32 * 30,
                    software_pixels_per_second: 0,
                },
                |_, proof, allocation| Ok(proof.tick_allocated(allocation)),
            )
            .unwrap();
        let active = self.pool.active_decoders();
        assert!(active <= slots);
        self.peak = self.peak.max(active);
        for id in [1, 2] {
            if let Some(proof) = self.pool.get_mut(id) {
                if !proof.preview.is_empty() && !proof.caption().is_empty() {
                    self.preview.clone_from(&proof.preview);
                    self.caption = proof.caption();
                }
            }
        }
        for (id, result) in results {
            if result.unwrap() {
                assert!(self.pool.remove(id).unwrap());
                self.completed += 1;
            }
        }
        if self.phase == 0 && self.pool.get_mut(1).is_some_and(|p| p.red) {
            assert!(
                !self.pool.get_mut(2).unwrap().red,
                "denied occurrence must not decode"
            );
            self.pool.update_request(pool_request(2, 10)).unwrap();
            self.phase = 1;
        } else if self.phase == 1 && self.pool.get_mut(2).is_some_and(|p| p.red) {
            assert!(
                !self.pool.get_mut(1).unwrap().owns_decoder(),
                "lower priority decoder must close"
            );
            self.phase = 2;
        }
        if self.completed < 2 {
            return false;
        }
        assert_eq!(self.phase, 2);
        assert_eq!(self.peak, 2, "must exercise concurrent native decoders");
        assert_eq!(self.pool.active_decoders(), 0);
        println!(
            "NUX_VIDEO_POOL passed: priority handoff with one slot, concurrent independent playback with two slots, final owned decoder handles zero; acceleration unknown"
        );
        true
    }
    fn set_backgrounded(&mut self, suspended: bool) {
        for id in [1, 2] {
            if let Some(proof) = self.pool.get_mut(id) {
                proof.set_backgrounded(suspended);
            }
        }
    }
}
fn run_pool_proof(source: &str) {
    let mut proof = MetalPoolProof::new(source, false, false);
    loop {
        pump_run_loop(0.01).unwrap();
        if proof.tick() {
            break;
        }
    }
}

pub struct MetalSyncProof {
    leader: MetalProof,
    follower: MetalProof,
    group: std::rc::Rc<nuxie_video_host::sync::RegisteredSynchronizationGroup>,
    follower_generation: u64,
    start: Instant,
    started_follower: bool,
    corrections: usize,
    maximum_drift: f64,
    settled_peak: f64,
    settled_since: Option<Instant>,
    preview: Vec<u8>,
    caption: String,
}
impl MetalSyncProof {
    fn new(source: &str, audible: bool, embedded: bool) -> Self {
        let leader = MetalProof::with_media(source, audible, embedded);
        let follower = MetalProof::with_media(source, false, embedded);
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
            preview: Vec::new(),
            caption: String::new(),
        }
    }
    fn tick(&mut self) -> bool {
        // Independent native-clock oracle sampled before the host tick can
        // enqueue corrections. This harness never calls correct/report itself.
        let drift = self
            .leader
            .player
            .clock()
            .zip(self.follower.player.clock())
            .filter(|(leader, follower)| leader.playing && follower.playing)
            .map(|(leader, follower)| leader.seconds - follower.seconds);
        self.leader.render_tick(Allocation::PlatformManaged);
        self.follower.render_tick(Allocation::PlatformManaged);
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
                "NUX_VIDEO_SYNC_AUTO sample t={:.6} drift={:.6} generation={}",
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
        self.preview.clone_from(&self.follower.preview);
        self.caption = self.follower.caption();
        if self.leader.blue
            && self.follower.blue
            && self
                .settled_since
                .is_some_and(|s| s.elapsed() >= Duration::from_millis(300))
        {
            assert!(
                self.maximum_drift > 0.25 && self.corrections > 0,
                "must correct a real injected offset"
            );
            assert!(self.leader.frames >= 3 && self.follower.frames >= 3);
            self.group.command(Command::Dispose).unwrap();
            for proof in [&mut self.leader, &mut self.follower] {
                proof
                    .player
                    .tick(|_| panic!("disposed sync member uploaded"))
                    .unwrap();
                assert!(!proof.player.owns_decoder());
            }
            println!(
                "NUX_VIDEO_SYNC_AUTO passed: injected drift={:.6}s, corrections={}, settled peak={:.6}s for >=300ms, frames={}/{}, owned decoders=0",
                self.maximum_drift,
                self.corrections,
                self.settled_peak,
                self.leader.frames,
                self.follower.frames
            );
            return true;
        }
        assert!(
            Instant::now() < self.leader.deadline,
            "sync timeout: max drift={}, corrections={}, frames={}/{}",
            self.maximum_drift,
            self.corrections,
            self.leader.frames,
            self.follower.frames
        );
        false
    }
    fn set_backgrounded(&mut self, suspended: bool) {
        self.leader.set_backgrounded(suspended);
        self.follower.set_backgrounded(suspended);
        if suspended {
            self.settled_since = None;
            self.settled_peak = 0.0;
        }
    }
}
fn run_sync_proof(source: &str) {
    for embedded in [false, true] {
        let mut proof = MetalSyncProof::new(source, false, embedded);
        loop {
            pump_run_loop(0.01).unwrap();
            if proof.tick() {
                break;
            }
        }
        println!("NUX_VIDEO_SYNC embedded={embedded}");
    }
}

pub enum AppleProof {
    Single(MetalProof),
    Pool(MetalPoolProof),
    Sync(MetalSyncProof),
}
impl AppleProof {
    fn tick(&mut self) -> bool {
        match self {
            Self::Single(p) => p.tick(),
            Self::Pool(p) => p.tick(),
            Self::Sync(p) => p.tick(),
        }
    }
    fn set_backgrounded(&mut self, suspended: bool) {
        match self {
            Self::Single(p) => p.set_backgrounded(suspended),
            Self::Pool(p) => p.set_backgrounded(suspended),
            Self::Sync(p) => p.set_backgrounded(suspended),
        }
    }
    fn caption(&self) -> String {
        match self {
            Self::Single(p) => p.caption(),
            Self::Pool(p) => p.caption.clone(),
            Self::Sync(p) => p.caption.clone(),
        }
    }
    fn preview(&self) -> &[u8] {
        match self {
            Self::Single(p) => &p.preview,
            Self::Pool(p) => &p.preview,
            Self::Sync(p) => &p.preview,
        }
    }
}

pub fn run_metal_proof(source: &str) {
    run_sync_proof(source);
    run_pool_proof(source);
    for embedded in [false, true] {
        let mut proof = MetalProof::with_media(source, false, embedded);
        loop {
            pump_run_loop(0.01).unwrap();
            if proof.tick() {
                break;
            }
        }
        println!("NUX_VIDEO_PROOF embedded={embedded}");
    }
}

/// Qualification-only ABI. Handles and source strings must be valid; all calls
/// occur on the main thread. No Rust unwind may cross the Objective-C boundary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_proof_open(
    source: *const std::ffi::c_char,
    audible: bool,
    embedded: bool,
    pool: bool,
    sync: bool,
) -> *mut AppleProof {
    if source.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(source) = (unsafe { std::ffi::CStr::from_ptr(source) }).to_str() else {
        return std::ptr::null_mut();
    };
    std::panic::catch_unwind(|| {
        Box::into_raw(Box::new(if sync {
            AppleProof::Sync(MetalSyncProof::new(source, audible, embedded))
        } else if pool {
            AppleProof::Pool(MetalPoolProof::new(source, audible, embedded))
        } else {
            AppleProof::Single(MetalProof::with_media(source, audible, embedded))
        }))
    })
    .unwrap_or(std::ptr::null_mut())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_proof_tick(proof: *mut AppleProof) -> i32 {
    if proof.is_null() {
        return -1;
    }
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        (&mut *proof).tick()
    })) {
        Ok(false) => 0,
        Ok(true) => 1,
        Err(_) => -1,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_proof_close(proof: *mut AppleProof) {
    if !proof.is_null() {
        unsafe {
            drop(Box::from_raw(proof));
        }
    }
}

/// The host calls this before stopping its render timer and before resuming it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_proof_backgrounded(
    proof: *mut AppleProof,
    suspended: bool,
) -> i32 {
    if proof.is_null() {
        return 0;
    }
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        (&mut *proof).set_backgrounded(suspended);
    })) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Qualification-only lifecycle readiness/result, for the single-player proof.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_proof_lifecycle_status(proof: *const AppleProof) -> i32 {
    if proof.is_null() {
        return 0;
    }
    match unsafe { &*proof } {
        AppleProof::Single(p) if p.os_pause_checked => 2,
        AppleProof::Single(p)
            if p.pause_checked && p.frames >= 3 && p.player.clock().is_some_and(|c| c.playing) =>
        {
            1
        }
        _ => 0,
    }
}

/// Qualification-only bounded caption copy. Caller provides writable capacity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_proof_caption(
    proof: *const AppleProof,
    output: *mut u8,
    capacity: usize,
) -> usize {
    if proof.is_null() || output.is_null() {
        return 0;
    }
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let text = unsafe { &*proof }.caption();
        if text.len() > capacity {
            return 0;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(text.as_ptr(), output, text.len());
        }
        text.len()
    }))
    .unwrap_or(0)
}

/// Copies the latest verified 64x32 RGBA frame for the qualification app preview.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_proof_preview(
    proof: *const AppleProof,
    output: *mut u8,
    capacity: usize,
) -> usize {
    if proof.is_null() || output.is_null() {
        return 0;
    }
    let pixels = unsafe { &*proof }.preview();
    if pixels.len() > capacity {
        return 0;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), output, pixels.len());
    }
    pixels.len()
}

/// Incrementally driven so iOS keeps normal application run-loop ownership.
pub struct MetalBenchmark {
    factory: PersistentFactory<NativeMetalFactory>,
    artboard: RuntimeArtboardInstanceHandle,
    player: AppleScenePlayer,
    start: Instant,
    first: Option<f64>,
    previous: Option<f64>,
    max_gap: f64,
    costs: Vec<f64>,
    red: usize,
    blue: usize,
    preview: Vec<u8>,
    result: String,
    completed: bool,
}
impl MetalBenchmark {
    pub fn new(source: &str) -> Self {
        let mut factory = PersistentFactory::new(NativeMetalFactory::new(1280, 720).unwrap());
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
        let player = AppleScenePlayer::open(video.clone(), source, 8 * 1024 * 1024).unwrap();
        Self {
            factory,
            artboard,
            player,
            start: Instant::now(),
            first: None,
            previous: None,
            max_gap: 0.0,
            costs: Vec::new(),
            red: 0,
            blue: 0,
            preview: Vec::new(),
            result: String::new(),
            completed: false,
        }
    }
    pub fn tick(&mut self) -> bool {
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
                "frames={} red={} blue={} fps={fps:.3} first_frame_ms={:.3} frame_work_p50_ms={p50:.3} frame_work_p95_ms={p95:.3} max_gap_ms={:.3} owned_decoders=0 acceleration=unknown",
                self.costs.len(),
                self.red,
                self.blue,
                self.first.unwrap() * 1000.0,
                self.max_gap * 1000.0
            );
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
        let status = self
            .player
            .tick_allocated(Allocation::PlatformManaged, |frame| {
                assert_eq!((frame.width, frame.height), (1280, 720));
                pixel = Some(frame.rgba[..4].to_vec());
                let image = self
                    .factory
                    .borrow()
                    .upload_rgba8_premul_srgb(
                        frame.width,
                        frame.height,
                        frame.width * 4,
                        &frame.rgba,
                    )
                    .unwrap();
                Ok(std::rc::Rc::from(image))
            })
            .unwrap();
        if status.presented {
            let mut render = self.factory.borrow().begin_frame(0xff000000).unwrap();
            nuxie_render_api::Renderer::scale(&mut render, 20.0, 22.5);
            self.artboard.draw(&mut render);
            let full_pixels = render.finish().unwrap();
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
                self.max_gap = self.max_gap.max(now - last);
            }
            self.previous = Some(now);
            self.preview = pixels;
            self.costs.push(tick_start.elapsed().as_secs_f64() * 1000.0);
        }
        false
    }
}
pub fn run_metal_benchmark(source: &str) {
    let mut benchmark = MetalBenchmark::new(source);
    loop {
        pump_run_loop(0.002).unwrap();
        if benchmark.tick() {
            break;
        }
    }
}
/// Qualification-only ABI. Main-thread ownership and valid pointers required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_benchmark_open(
    source: *const std::ffi::c_char,
) -> *mut MetalBenchmark {
    if source.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(source) = (unsafe { std::ffi::CStr::from_ptr(source) }).to_str() else {
        return std::ptr::null_mut();
    };
    std::panic::catch_unwind(|| Box::into_raw(Box::new(MetalBenchmark::new(source))))
        .unwrap_or(std::ptr::null_mut())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_benchmark_tick(proof: *mut MetalBenchmark) -> i32 {
    if proof.is_null() {
        return -1;
    }
    let proof = unsafe { &mut *proof };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| proof.tick())) {
        Ok(false) => 0,
        Ok(true) => 1,
        Err(_) => {
            proof.result = format!("FAIL: {}", proof.result);
            proof.completed = true;
            -1
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_benchmark_copy(
    proof: *const MetalBenchmark,
    output: *mut u8,
    capacity: usize,
    preview: bool,
) -> usize {
    if proof.is_null() || output.is_null() {
        return 0;
    }
    let proof = unsafe { &*proof };
    let bytes = if preview {
        proof.preview.as_slice()
    } else {
        proof.result.as_bytes()
    };
    if bytes.len() > capacity {
        return 0;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len());
    }
    bytes.len()
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_benchmark_close(proof: *mut MetalBenchmark) {
    if !proof.is_null() {
        drop(unsafe { Box::from_raw(proof) });
    }
}
