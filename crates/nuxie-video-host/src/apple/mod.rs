//! AVFoundation playback without a native view overlay. AVPlayer owns A/V
//! timing; frames are copied into an owned RGBA buffer for the renderer upload.
//! This initial path does a CPU copy and does not claim zero-copy or a known
//! hardware decode path. All methods, including Drop, run on the main thread.
use nuxie_runtime::video::playback::{
    AudioPolicy, Command, DecoderAction, Playback, SuspensionReason,
};
use std::{
    ffi::{CString, c_char, c_void},
    marker::PhantomData,
    ptr::NonNull,
    rc::Rc,
};

unsafe extern "C" {
    fn nux_video_apple_is_main_thread() -> bool;
    fn nux_video_apple_open(source: *const c_char, generation: u64) -> *mut c_void;
    fn nux_video_apple_close(handle: *mut c_void);
    fn nux_video_apple_clock(
        handle: *mut c_void,
        generation: *mut u64,
        seconds: *mut f64,
        rate: *mut f64,
        playing: *mut bool,
    ) -> bool;
    fn nux_video_apple_audio_policy(handle: *mut c_void, policy: u32, managed: bool);
    fn nux_video_apple_audio_status(handle: *mut c_void) -> u32;
    fn nux_video_apple_action(handle: *mut c_void, action: i32, value: f64, generation: u64);
    fn nux_video_apple_poll(
        handle: *mut c_void,
        generation: *mut u64,
        time: *mut f64,
        width: *mut u32,
        height: *mut u32,
    ) -> i32;
    fn nux_video_apple_copy_rgba(handle: *mut c_void, out: *mut u8, capacity: usize) -> bool;
    fn nux_video_apple_pump(seconds: f64);
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppleError {
    MainThreadRequired,
    InvalidSource,
    UnsupportedDecodeMode,
    OpenFailed,
    DecodeFailed,
    FrameTooLarge,
    SourceMaterializationFailed,
    Disposed,
}

/// Storage remains alive while the renderer uploads it; native decode surfaces
/// are released immediately after the bounded copy. Pixels are opaque SDR RGBA.
pub use crate::scene::Frame;
pub enum Observation {
    Ready {
        generation: u64,
        duration: f64,
    },
    Playing(u64),
    Ended(u64),
    Frame(Frame),
    /// Actual decoded image selected by a successful seek in this generation.
    SelectedSeekFrame(Frame),
}
/// Session ownership is explicit because AVAudioSession is process-wide.
/// Apps with an existing audio engine keep ownership and implement authored
/// mix/duck/exclusive policy themselves. RuntimeManaged delegates ownership
/// while any audible video is playing; muted videos never activate a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSessionOwnership {
    HostManaged,
    RuntimeManaged,
}
pub struct ApplePlayer {
    handle: Option<NonNull<c_void>>,
    max_frame_bytes: usize,
    _main_thread: PhantomData<Rc<()>>,
}
impl ApplePlayer {
    /// `source` is a retained local filename or host-resolved URL. Embedded
    /// content is materialized by the caller's cache, not per decoded frame.
    pub fn open(source: &str, generation: u64, max_frame_bytes: usize) -> Result<Self, AppleError> {
        Self::open_with_audio(
            source,
            generation,
            max_frame_bytes,
            AudioPolicy::Automatic,
            AudioSessionOwnership::HostManaged,
        )
    }
    pub fn open_with_audio(
        source: &str,
        generation: u64,
        max_frame_bytes: usize,
        policy: AudioPolicy,
        ownership: AudioSessionOwnership,
    ) -> Result<Self, AppleError> {
        if !unsafe { nux_video_apple_is_main_thread() } {
            return Err(AppleError::MainThreadRequired);
        }
        let source = CString::new(source).map_err(|_| AppleError::InvalidSource)?;
        let handle = NonNull::new(unsafe { nux_video_apple_open(source.as_ptr(), generation) })
            .ok_or(AppleError::OpenFailed)?;
        unsafe {
            nux_video_apple_audio_policy(
                handle.as_ptr(),
                policy as u32,
                ownership == AudioSessionOwnership::RuntimeManaged,
            );
        }
        Ok(Self {
            handle: Some(handle),
            max_frame_bytes,
            _main_thread: PhantomData,
        })
    }
    pub fn apply(&mut self, action: DecoderAction) -> Result<(), AppleError> {
        let handle = self.handle.ok_or(AppleError::Disposed)?;
        let (code, value, generation) = match action {
            DecoderAction::Play => (0, 0.0, 0),
            DecoderAction::Pause => (1, 0.0, 0),
            DecoderAction::Seek {
                seconds,
                generation,
            } if seconds.is_finite() && seconds >= 0.0 => (2, seconds, generation),
            DecoderAction::Rate(rate) if rate.is_finite() && rate > 0.0 => (3, f64::from(rate), 0),
            DecoderAction::Volume(volume)
                if volume.is_finite() && (0.0..=1.0).contains(&volume) =>
            {
                (4, f64::from(volume), 0)
            }
            DecoderAction::Dispose => {
                self.close();
                return Ok(());
            }
            _ => return Err(AppleError::InvalidSource),
        };
        unsafe {
            nux_video_apple_action(handle.as_ptr(), code, value, generation);
        }
        Ok(())
    }
    pub fn clock(&self) -> Option<crate::scene::MediaClock> {
        let handle = self.handle?;
        let (mut generation, mut seconds, mut rate, mut playing) = (0, 0.0, 0.0, false);
        let valid = unsafe {
            nux_video_apple_clock(
                handle.as_ptr(),
                &mut generation,
                &mut seconds,
                &mut rate,
                &mut playing,
            )
        };
        valid.then_some(crate::scene::MediaClock {
            generation,
            seconds,
            rate,
            playing,
        })
    }
    pub fn poll(&mut self) -> Result<Option<Observation>, AppleError> {
        let handle = self.handle.ok_or(AppleError::Disposed)?;
        let (mut generation, mut time, mut width, mut height) = (0, 0.0, 0, 0);
        let kind = unsafe {
            nux_video_apple_poll(
                handle.as_ptr(),
                &mut generation,
                &mut time,
                &mut width,
                &mut height,
            )
        };
        Ok(match kind {
            0 => None,
            1 => Some(Observation::Ready {
                generation,
                duration: time,
            }),
            2 => Some(Observation::Playing(generation)),
            3 => Some(Observation::Ended(generation)),
            4 | 5 => {
                let length = (width as usize)
                    .checked_mul(height as usize)
                    .and_then(|n| n.checked_mul(4))
                    .filter(|&n| n > 0 && n <= self.max_frame_bytes)
                    .ok_or(AppleError::FrameTooLarge)?;
                let mut rgba = vec![0; length];
                if !unsafe { nux_video_apple_copy_rgba(handle.as_ptr(), rgba.as_mut_ptr(), length) }
                {
                    return Err(AppleError::DecodeFailed);
                }
                let frame = Frame {
                    generation,
                    pts: time,
                    width,
                    height,
                    rgba,
                };
                Some(if kind == 5 {
                    Observation::SelectedSeekFrame(frame)
                } else {
                    Observation::Frame(frame)
                })
            }
            _ => return Err(AppleError::DecodeFailed),
        })
    }
    /// Apply queued runtime intent, then process at most one observation. The
    /// caller uploads any returned frame into the scene's persistent factory.
    pub fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, AppleError> {
        if let Some(handle) = self.handle {
            let flags = unsafe { nux_video_apple_audio_status(handle.as_ptr()) };
            // An entire interruption may occur while the host's render loop is
            // stopped. Preserve its end edge so a still-requested play retries.
            if flags & 10 != 0 {
                for action in playback.update_suspension(SuspensionReason::Interruption, true) {
                    self.apply(action)?;
                }
            }
            if flags & 2 != 0 {
                playback
                    .enqueue(Command::Pause)
                    .map_err(|_| AppleError::DecodeFailed)?;
                for action in playback.drain_actions() {
                    self.apply(action)?;
                }
            }
            for action in playback.update_suspension(SuspensionReason::Interruption, flags & 1 != 0)
            {
                self.apply(action)?;
            }
            if flags & 4 != 0 {
                playback.observed_play_blocked(playback.generation());
            }
        }
        for action in playback.drain_actions() {
            self.apply(action)?;
        }
        if self.handle.is_none() {
            return Ok(None);
        }
        let observation = match self.poll() {
            Ok(value) => value,
            Err(error) => {
                playback.failed(playback.generation());
                self.close();
                return Err(error);
            }
        };
        let actions = match observation {
            Some(Observation::Ready {
                generation,
                duration,
            }) => playback.opened(generation, duration),
            Some(Observation::Playing(generation)) => {
                playback.observed_playing(generation);
                Vec::new()
            }
            Some(Observation::Ended(generation)) => playback.ended(generation),
            Some(Observation::SelectedSeekFrame(frame)) => {
                playback.observe_selected_seek_frame(frame.generation, frame.pts);
                return Ok(Some(frame));
            }
            Some(Observation::Frame(frame)) => return Ok(Some(frame)),
            None => Vec::new(),
        };
        for action in actions {
            self.apply(action)?;
        }
        Ok(None)
    }
    pub fn close(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe {
                nux_video_apple_close(handle.as_ptr());
            }
        }
    }
}
impl Drop for ApplePlayer {
    fn drop(&mut self) {
        self.close();
    }
}

/// Headless qualification helper. App hosts already drive their own main loop.
pub fn pump_run_loop(seconds: f64) -> Result<(), AppleError> {
    if !unsafe { nux_video_apple_is_main_thread() } {
        return Err(AppleError::MainThreadRequired);
    }
    if !seconds.is_finite() || !(0.0..=0.1).contains(&seconds) {
        return Err(AppleError::InvalidSource);
    }
    unsafe {
        nux_video_apple_pump(seconds);
    }
    Ok(())
}

/// Connect a player to the live scene occurrence controlled by Luau/Journeys.
/// Upload must use the same persistent factory used to import the scene.
pub struct AppleScenePlayer {
    scene: crate::scene::ScenePlayer<ApplePlayer>,
    source: String,
    policy: AudioPolicy,
    ownership: AudioSessionOwnership,
    max_frame_bytes: usize,
    clock: std::time::Instant,
    // Declared after player so the decoder closes before the file is removed.
    source_lease: Option<crate::source::EmbeddedFile>,
}
impl AppleScenePlayer {
    pub fn clock(&self) -> Option<crate::scene::MediaClock> {
        self.scene.clock()
    }

    pub fn open(
        video: nuxie_runtime::source::core::CoreHandle,
        source: &str,
        max_frame_bytes: usize,
    ) -> Result<Self, AppleError> {
        Self::open_with_audio(
            video,
            source,
            max_frame_bytes,
            AudioSessionOwnership::HostManaged,
        )
    }
    pub fn open_with_audio(
        video: nuxie_runtime::source::core::CoreHandle,
        source: &str,
        max_frame_bytes: usize,
        ownership: AudioSessionOwnership,
    ) -> Result<Self, AppleError> {
        let policy = video
            .with_downcast::<nuxie_runtime::video::Video, _>(|v| v.playback.settings().audio_policy)
            .ok_or(AppleError::InvalidSource)?;
        let policy = match policy {
            0 => AudioPolicy::Automatic,
            1 => AudioPolicy::Mix,
            2 => AudioPolicy::Duck,
            3 => AudioPolicy::Exclusive,
            _ => return Err(AppleError::InvalidSource),
        };
        Ok(Self {
            scene: crate::scene::ScenePlayer::new(video, 5.0, true)
                .map_err(|_| AppleError::InvalidSource)?,
            source: source.to_owned(),
            policy,
            ownership,
            max_frame_bytes,
            clock: std::time::Instant::now(),
            source_lease: None,
        })
    }
    pub fn open_embedded(
        video: nuxie_runtime::source::core::CoreHandle,
        directory: &std::path::Path,
        max_source_bytes: usize,
        max_frame_bytes: usize,
        ownership: AudioSessionOwnership,
    ) -> Result<Self, AppleError> {
        let asset = video
            .with_downcast::<nuxie_runtime::video::Video, _>(|v| v.asset())
            .flatten()
            .ok_or(AppleError::InvalidSource)?;
        let bytes = asset
            .with_downcast::<nuxie_runtime::video::VideoAsset, _>(|a| a.encoded_bytes())
            .flatten()
            .ok_or(AppleError::InvalidSource)?;
        let lease = crate::source::EmbeddedFile::materialize(directory, &bytes, max_source_bytes)
            .map_err(|_| AppleError::SourceMaterializationFailed)?;
        let mut player = Self::open_with_audio(
            video,
            lease.path().to_str().ok_or(AppleError::InvalidSource)?,
            max_frame_bytes,
            ownership,
        )?;
        player.source_lease = Some(lease);
        Ok(player)
    }
    /// Call from lifecycle callbacks even when frame updates are stopped.
    pub fn set_suspended(
        &mut self,
        reason: nuxie_runtime::video::playback::SuspensionReason,
        suspended: bool,
    ) -> Result<(), AppleError> {
        self.scene
            .set_suspended(reason, suspended)
            .map_err(|error| match error {
                crate::scene::SceneError::Decoder(error) => error,
                _ => AppleError::InvalidSource,
            })
    }
    /// Replace a resolved external source; caller retains acquisition policy.
    pub fn replace_source(
        &mut self,
        source: &str,
        settings: nuxie_runtime::video::playback::PlaybackSettings,
    ) -> Result<u64, AppleError> {
        if source.is_empty() || source.as_bytes().contains(&0) {
            return Err(AppleError::InvalidSource);
        }
        let policy = match settings.audio_policy {
            0 => AudioPolicy::Automatic,
            1 => AudioPolicy::Mix,
            2 => AudioPolicy::Duck,
            3 => AudioPolicy::Exclusive,
            _ => return Err(AppleError::InvalidSource),
        };
        let generation = self
            .scene
            .replace_source(settings)
            .map_err(|_| AppleError::InvalidSource)?;
        self.source = source.into();
        self.policy = policy;
        // The old decoder has closed before its extracted source is removed.
        self.source_lease = None;
        Ok(generation)
    }
    pub fn tick(
        &mut self,
        upload: impl FnMut(&Frame) -> Result<Rc<dyn nuxie_render_api::RenderImage>, AppleError>,
    ) -> Result<bool, AppleError> {
        self.tick_allocated(
            nuxie_runtime::video::resources::Allocation::PlatformManaged,
            upload,
        )
        .map(|status| status.presented)
    }
    /// AVPlayer chooses its codec implementation. Hosts must budget that
    /// platform-managed slot; requests to force hardware or software are rejected.
    pub fn tick_allocated(
        &mut self,
        allocation: nuxie_runtime::video::resources::Allocation,
        upload: impl FnMut(&Frame) -> Result<Rc<dyn nuxie_render_api::RenderImage>, AppleError>,
    ) -> Result<crate::scene::SceneStatus, AppleError> {
        if !matches!(
            allocation,
            nuxie_runtime::video::resources::Allocation::PlatformManaged
                | nuxie_runtime::video::resources::Allocation::Poster
        ) {
            return Err(AppleError::UnsupportedDecodeMode);
        }
        self.scene
            .tick(
                allocation,
                self.clock.elapsed().as_secs_f64(),
                |generation, _| {
                    ApplePlayer::open_with_audio(
                        &self.source,
                        generation,
                        self.max_frame_bytes,
                        self.policy,
                        self.ownership,
                    )
                },
                upload,
            )
            .map_err(|error| match error {
                crate::scene::SceneError::Decoder(error) => error,
                _ => AppleError::InvalidSource,
            })
    }
}

impl crate::scene::Decoder for ApplePlayer {
    type Error = AppleError;
    fn close(&mut self) -> Result<(), Self::Error> {
        ApplePlayer::close(self);
        Ok(())
    }
    fn clock(&self) -> Option<crate::scene::MediaClock> {
        ApplePlayer::clock(self)
    }
    fn apply(&mut self, action: DecoderAction) -> Result<(), Self::Error> {
        ApplePlayer::apply(self, action)
    }
    fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, Self::Error> {
        ApplePlayer::tick(self, playback)
    }
}

impl crate::pool::ManagedPlayer for AppleScenePlayer {
    type Error = AppleError;
    fn allocation(&self) -> nuxie_runtime::video::resources::Allocation {
        self.scene.allocation()
    }
    fn owns_decoder(&self) -> bool {
        self.scene.owns_decoder()
    }
    fn can_decode(&self) -> bool {
        self.scene.can_decode()
    }
    fn reclaim(&mut self) -> Result<(), Self::Error> {
        self.scene.reclaim().map_err(|error| match error {
            crate::scene::SceneError::Decoder(error) => error,
            _ => AppleError::InvalidSource,
        })
    }
}
