//! Live scene occurrence ownership shared by platform adapters. Allocation is
//! supplied from host capabilities/budgets; denied occurrences own no decoder.
use nuxie_render_api::RenderImage;
use nuxie_runtime::{
    source::core::CoreHandle,
    video::{
        Video,
        playback::{DecoderAction, Playback, PlaybackSettings, PlaybackState, SuspensionReason},
        readiness::{FirstFrameGate, Readiness},
        resources::Allocation,
    },
};
use std::rc::Rc;

/// Shared monotonic domain for authored groups across independently opened players.
fn group_clock_now() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        ORIGIN
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.performance())
            .map_or(0.0, |clock| clock.now() / 1000.0)
    }
}

pub struct Frame {
    pub generation: u64,
    pub pts: f64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
pub use nuxie_runtime::video::sync::MediaClock;
/// Implementations release all decoder/audio resources when dropped.
pub trait Decoder {
    type Error;
    /// None means unavailable, including while a seek is pending.
    fn clock(&self) -> Option<MediaClock> {
        None
    }
    /// Return success only after native resources are released. Failed closes
    /// are retried while the owning scene retains its allocation.
    fn close(&mut self) -> Result<(), Self::Error> {
        self.apply(DecoderAction::Dispose)
    }
    fn apply(&mut self, action: DecoderAction) -> Result<(), Self::Error>;
    fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, Self::Error>;
}
fn close_decoder<D: Decoder>(decoder: &mut Option<D>) -> Result<bool, SceneError<D::Error>> {
    if let Some(value) = decoder.as_mut() {
        value.close().map_err(SceneError::Decoder)?;
    }
    Ok(decoder.take().is_some())
}
#[derive(Debug)]
pub enum SceneError<E> {
    InvalidOccurrence,
    InvalidClock,
    Decoder(E),
}
#[derive(Debug, Clone, Copy)]
pub struct SceneStatus {
    pub presented: bool,
    pub readiness: Readiness,
    pub allocation: Allocation,
}
pub struct ScenePlayer<D: Decoder> {
    decoder: Option<D>,
    video: CoreHandle,
    allocation: Allocation,
    gate: Option<FirstFrameGate>,
    gate_generation: Option<u64>,
    timeout: f64,
    optional: bool,
    fallback: bool,
}
impl<D: Decoder> ScenePlayer<D> {
    pub fn new(
        video: CoreHandle,
        timeout: f64,
        optional: bool,
    ) -> Result<Self, SceneError<D::Error>> {
        if video.with_downcast::<Video, _>(|_| ()).is_none() {
            return Err(SceneError::InvalidOccurrence);
        }
        if FirstFrameGate::new(0, 0.0, timeout, true, optional).is_none() {
            return Err(SceneError::InvalidClock);
        }
        Ok(Self {
            decoder: None,
            video,
            allocation: Allocation::Poster,
            gate: None,
            gate_generation: None,
            timeout,
            optional,
            fallback: false,
        })
    }
    /// Begin a replacement media source for this occurrence. Validate before
    /// mutation, invalidate old callbacks, close its decoder, and reset readiness.
    /// The platform owner swaps its retained URL/bytes only after this succeeds.
    pub fn replace_source(
        &mut self,
        settings: PlaybackSettings,
    ) -> Result<u64, SceneError<D::Error>> {
        let generation = self
            .video
            .with_downcast_mut::<Video, _>(|video| {
                video
                    .playback
                    .validate_source_replacement(settings)
                    .map_err(|_| SceneError::InvalidOccurrence)?;
                close_decoder(&mut self.decoder)?;
                let generation = video
                    .playback
                    .replace_source(settings)
                    .map_err(|_| SceneError::InvalidOccurrence)?;
                video.clear_frame();
                Ok(generation)
            })
            .ok_or(SceneError::InvalidOccurrence)??;
        self.gate = None;
        self.gate_generation = None;
        self.fallback = false;
        Ok(generation)
    }
    /// Deliver lifecycle changes before the host stops rendering. This never
    /// opens a decoder or consumes a frame. Suspension reasons are independent,
    /// so foregrounding cannot undo an outstanding interruption or hidden view.
    pub fn set_suspended(
        &mut self,
        reason: SuspensionReason,
        suspended: bool,
    ) -> Result<(), SceneError<D::Error>> {
        self.video
            .with_downcast_mut::<Video, _>(|video| {
                let actions = video.playback.update_suspension(reason, suspended);
                if let Some(decoder) = self.decoder.as_mut() {
                    for action in actions {
                        if let Err(error) = decoder.apply(action) {
                            video.clear_frame();
                            video.playback.failed(video.playback.generation());
                            let _ = close_decoder(&mut self.decoder);
                            return Err(SceneError::Decoder(error));
                        }
                    }
                }
                Ok(())
            })
            .ok_or_else(|| {
                let _ = close_decoder(&mut self.decoder);
                SceneError::InvalidOccurrence
            })?
    }
    /// Close before admitting another occurrence. Safe even when the scene
    /// occurrence has been removed. A failed close retains native ownership
    /// and its allocation until a later retry acknowledges disposal.
    pub fn reclaim(&mut self) -> Result<(), SceneError<D::Error>> {
        let had_decoder = close_decoder(&mut self.decoder)?;
        self.allocation = Allocation::Poster;
        self.gate = None;
        self.gate_generation = None;
        self.fallback = false;
        self.video
            .with_downcast_mut::<Video, _>(|video| {
                if had_decoder
                    && !matches!(
                        video.playback.state(),
                        PlaybackState::Disposed | PlaybackState::Failed
                    )
                {
                    video
                        .playback
                        .reclaim_decoder()
                        .map_err(|_| SceneError::InvalidOccurrence)?;
                }
                video.clear_frame();
                video.apply_allocation(Allocation::Poster);
                video.playback.drain_actions();
                Ok(())
            })
            .ok_or(SceneError::InvalidOccurrence)?
    }
    pub fn allocation(&self) -> Allocation {
        self.allocation
    }
    pub fn clock(&self) -> Option<MediaClock> {
        let clock = self.decoder.as_ref()?.clock()?;
        self.video
            .with_downcast::<Video, _>(|video| {
                (clock.generation == video.playback.generation()
                    && !matches!(
                        video.playback.state(),
                        PlaybackState::Failed | PlaybackState::Disposed
                    )
                    && clock.seconds.is_finite()
                    && clock.seconds >= 0.0
                    && clock.rate.is_finite()
                    && clock.rate >= 0.0)
                    .then_some(clock)
            })
            .flatten()
    }
    #[cfg(target_os = "android")]
    pub(crate) fn decoder(&self) -> Option<&D> {
        self.decoder.as_ref()
    }
    pub fn can_decode(&self) -> bool {
        self.video
            .with_downcast::<Video, _>(|v| {
                !matches!(
                    v.playback.state(),
                    PlaybackState::Failed | PlaybackState::Disposed
                )
            })
            .unwrap_or(false)
    }
    pub fn owns_decoder(&self) -> bool {
        self.decoder.is_some()
    }
    /// Call once per host update, after priority allocation. The opener must
    /// honor the requested decode mode or return an error; it must not silently
    /// substitute an unbudgeted software decoder. All players denied in a batch
    /// should be ticked before newly admitted players to free capacity first.
    pub fn tick(
        &mut self,
        allocation: Allocation,
        now: f64,
        mut open: impl FnMut(u64, Allocation) -> Result<D, D::Error>,
        mut upload: impl FnMut(&Frame) -> Result<Rc<dyn RenderImage>, D::Error>,
    ) -> Result<SceneStatus, SceneError<D::Error>> {
        if !now.is_finite() || now < 0.0 {
            return Err(SceneError::InvalidClock);
        }
        let result = self
            .video
            .with_downcast_mut::<Video, _>(|video| {
                let terminal = matches!(
                    video.playback.state(),
                    PlaybackState::Failed | PlaybackState::Disposed
                );
                if self
                    .gate_generation
                    .is_some_and(|g| g != video.playback.generation())
                {
                    self.fallback = false;
                }
                if allocation != self.allocation || terminal {
                    if close_decoder(&mut self.decoder)? && !terminal {
                        video
                            .playback
                            .reclaim_decoder()
                            .map_err(|_| SceneError::InvalidOccurrence)?;
                    }
                    self.fallback = false;
                    video.clear_frame();
                    self.allocation = allocation;
                    self.gate = None;
                    self.gate_generation = None;
                }
                // No decoder exists during a mode transition. Applying allocation
                // cannot send play before the replacement's ready observation.
                video.apply_allocation(allocation);
                let mut presented = false;
                if allocation == Allocation::Poster || terminal || self.fallback {
                    video.clear_frame();
                    // Retain commands' intent (including seeks) while no decoder
                    // exists; opened() later restores current position/settings.
                    video.playback.drain_actions();
                } else {
                    if self.decoder.is_none() {
                        self.decoder = Some(
                            open(video.playback.generation(), allocation).map_err(|error| {
                                video.playback.failed(video.playback.generation());
                                SceneError::Decoder(error)
                            })?,
                        );
                    }
                    let result = self
                        .decoder
                        .as_mut()
                        .expect("decoder opened above")
                        .tick(&mut video.playback);
                    match result {
                        Ok(Some(frame)) => {
                            if frame.generation == video.playback.generation() {
                                match upload(&frame) {
                                    Ok(image) => {
                                        presented =
                                            video.present(frame.generation, image, frame.pts)
                                    }
                                    Err(error) => {
                                        video.clear_frame();
                                        video.playback.failed(frame.generation);
                                        let _ = close_decoder(&mut self.decoder);
                                        return Err(SceneError::Decoder(error));
                                    }
                                }
                            }
                        }
                        Ok(None) => (),
                        Err(error) => {
                            video.clear_frame();
                            video.playback.failed(video.playback.generation());
                            let _ = close_decoder(&mut self.decoder);
                            return Err(SceneError::Decoder(error));
                        }
                    }
                }
                if matches!(
                    video.playback.state(),
                    PlaybackState::Disposed | PlaybackState::Failed
                ) {
                    close_decoder(&mut self.decoder)?;
                    video.clear_frame();
                }
                if self.gate_generation != Some(video.playback.generation()) {
                    self.gate = FirstFrameGate::new(
                        video.playback.generation(),
                        now,
                        self.timeout,
                        video.playback.settings().readiness == 1,
                        self.optional,
                    );
                    self.gate_generation = Some(video.playback.generation());
                }
                let readiness = self
                    .gate
                    .as_mut()
                    .ok_or(SceneError::InvalidClock)?
                    .evaluate(
                        video.playback.generation(),
                        now,
                        video.has_video_frame(),
                        matches!(
                            video.playback.state(),
                            PlaybackState::Disposed | PlaybackState::Failed
                        ),
                        video.has_poster(),
                    );
                if video.playback.settings().readiness == 1
                    && matches!(readiness, Readiness::Poster | Readiness::Unavailable)
                    && self.decoder.is_some()
                {
                    close_decoder(&mut self.decoder)?;
                    video.clear_frame();
                    video
                        .playback
                        .reclaim_decoder()
                        .map_err(|_| SceneError::InvalidOccurrence)?;
                    self.gate_generation = Some(video.playback.generation());
                    self.fallback = true;
                }
                Ok(SceneStatus {
                    presented,
                    readiness,
                    allocation,
                })
            })
            .ok_or_else(|| {
                let _ = close_decoder(&mut self.decoder);
                SceneError::InvalidOccurrence
            })??;
        // Group correction errors remain observable through the authored group's
        // take_error interface; they do not turn an unrelated decoder into Failed.
        let _ = nuxie_runtime::video::sync::report_media_clock(
            &self.video,
            group_clock_now(),
            self.clock(),
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{
        FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
    };
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::{File, RuntimeFactoryHandle, video::playback::Command};
    use std::cell::Cell;

    struct CountedDecoder(Rc<Cell<usize>>);
    impl Drop for CountedDecoder {
        fn drop(&mut self) {
            self.0.set(self.0.get() - 1);
        }
    }
    impl Decoder for CountedDecoder {
        type Error = ();
        fn apply(&mut self, _: DecoderAction) -> Result<(), ()> {
            Ok(())
        }
        fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, ()> {
            playback.drain_actions();
            playback.opened(playback.generation(), 10.0);
            Ok(None)
        }
    }
    fn record(name: &str, values: &[(u16, FixtureValue)]) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(name).unwrap().type_key.int,
            properties: values
                .iter()
                .map(|(key, value)| FixtureProperty {
                    key: *key,
                    value: value.clone(),
                })
                .collect(),
        }
    }
    fn video_occurrence() -> (
        nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle,
        CoreHandle,
    ) {
        let file = RuntimeFile::from_fixture_records(vec![
            record("Backboard", &[]),
            record("VideoAsset", &[]),
            record("Artboard", &[]),
            record(
                "Video",
                &[
                    (5, FixtureValue::Uint(0)),
                    (206, FixtureValue::Uint(0)),
                    (60003, FixtureValue::Uint(1)),
                    (60011, FixtureValue::Uint(1)),
                ],
            ),
        ])
        .unwrap();
        let bytes = encode_runtime_file(&file).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        (artboard, video)
    }
    #[test]
    fn terminal_source_is_ineligible_until_explicit_replacement() {
        let (_artboard, video) = video_occurrence();
        let mut player = ScenePlayer::<CountedDecoder>::new(video.clone(), 1.0, true).unwrap();
        assert!(player.can_decode());
        video
            .with_downcast_mut::<Video, _>(|v| v.playback.failed(v.playback.generation()))
            .unwrap();
        assert!(!player.can_decode());
        player.reclaim().unwrap();
        assert!(!player.can_decode());
        player.replace_source(PlaybackSettings::default()).unwrap();
        assert!(player.can_decode());
    }
    #[test]
    fn failed_disposal_retains_the_decoder_until_reclamation_is_acknowledged() {
        struct DelayedClose {
            allow: Rc<Cell<bool>>,
        }
        impl Decoder for DelayedClose {
            type Error = &'static str;
            fn apply(&mut self, action: DecoderAction) -> Result<(), Self::Error> {
                if action == DecoderAction::Dispose {
                    if !self.allow.get() {
                        return Err("worker still closing");
                    }
                }
                Ok(())
            }
            fn tick(&mut self, p: &mut Playback) -> Result<Option<Frame>, Self::Error> {
                p.drain_actions();
                p.opened(p.generation(), 10.0);
                Ok(None)
            }
        }
        let (_artboard, video) = video_occurrence();
        let allow = Rc::new(Cell::new(false));
        let mut player = ScenePlayer::new(video.clone(), 1.0, true).unwrap();
        player
            .tick(
                Allocation::Hardware,
                0.0,
                |_, _| {
                    Ok(DelayedClose {
                        allow: allow.clone(),
                    })
                },
                |_| panic!("no frame"),
            )
            .unwrap();
        let generation = video
            .with_downcast::<Video, _>(|v| v.playback.generation())
            .unwrap();
        assert!(matches!(
            player.reclaim(),
            Err(SceneError::Decoder("worker still closing"))
        ));
        assert!(player.owns_decoder());
        assert_eq!(player.allocation(), Allocation::Hardware);
        assert_eq!(
            video.with_downcast::<Video, _>(|v| v.playback.generation()),
            Some(generation)
        );
        assert!(matches!(
            player.replace_source(PlaybackSettings::default()),
            Err(SceneError::Decoder("worker still closing"))
        ));
        assert!(matches!(
            player.tick(
                Allocation::Poster,
                0.1,
                |_, _| panic!("must not open"),
                |_| panic!("no frame")
            ),
            Err(SceneError::Decoder("worker still closing"))
        ));
        assert!(player.owns_decoder());
        assert_eq!(player.allocation(), Allocation::Hardware);
        assert_eq!(
            video.with_downcast::<Video, _>(|v| v.playback.generation()),
            Some(generation)
        );
        allow.set(true);
        player.reclaim().unwrap();
        assert!(!player.owns_decoder());
        assert_eq!(player.allocation(), Allocation::Poster);
    }
    #[test]
    fn denied_occurrence_releases_decoder_and_reopens_at_retained_position() {
        let (_artboard, video) = video_occurrence();
        let live = Rc::new(Cell::new(0));
        let open = |_, _| {
            live.set(live.get() + 1);
            Ok(CountedDecoder(live.clone()))
        };
        let upload = |_: &Frame| -> Result<Rc<dyn RenderImage>, ()> { panic!("no frame expected") };
        let mut player = ScenePlayer::new(video.clone(), 1.0, true).unwrap();
        assert_eq!(
            player
                .tick(Allocation::Hardware, 0.0, open, upload)
                .unwrap()
                .readiness,
            Readiness::Waiting
        );
        assert_eq!(live.get(), 1);
        video.with_downcast_mut::<Video, _>(|v| {
            v.playback.accept_frame(v.playback.generation(), 2.0);
        });
        player.tick(Allocation::Poster, 0.2, open, upload).unwrap();
        assert_eq!(live.get(), 0);
        video.with_downcast_mut::<Video, _>(|v| {
            v.playback.enqueue(Command::Seek(4.0)).unwrap();
        });
        player.tick(Allocation::Poster, 0.3, open, upload).unwrap();
        player
            .tick(Allocation::Hardware, 0.4, open, upload)
            .unwrap();
        assert_eq!(live.get(), 1);
        assert_eq!(
            video.with_downcast::<Video, _>(|v| v.playback.position()),
            Some(4.0)
        );
        assert!(
            video
                .with_downcast::<Video, _>(|v| v.playback.wants_play())
                .unwrap()
        );
        assert_eq!(
            player
                .tick(Allocation::Hardware, 1.5, open, upload)
                .unwrap()
                .readiness,
            Readiness::Poster
        );
        assert_eq!(live.get(), 0, "readiness timeout must release decoder");
        player
            .tick(Allocation::Hardware, 2.0, open, upload)
            .unwrap();
        assert_eq!(live.get(), 0, "fallback must not reopen on each tick");
        player.tick(Allocation::Poster, 2.1, open, upload).unwrap();
        player
            .tick(Allocation::Hardware, 2.2, open, upload)
            .unwrap();
        assert_eq!(live.get(), 1);
        video.with_downcast_mut::<Video, _>(|v| {
            v.playback.enqueue(Command::Dispose).unwrap();
        });
        player
            .tick(Allocation::Hardware, 2.3, open, upload)
            .unwrap();
        assert_eq!(live.get(), 0, "dispose releases decoder in the same update");
        drop(player);
        assert_eq!(live.get(), 0);
    }
    #[test]
    fn replacement_is_atomic_releases_old_decoder_and_preserves_suspension() {
        use nuxie_runtime::video::playback::{DecoderAction, SuspensionReason};
        let (_artboard, video) = video_occurrence();
        let live = Rc::new(Cell::new(0));
        let open = |_, _| {
            live.set(live.get() + 1);
            Ok(CountedDecoder(live.clone()))
        };
        let upload = |_: &Frame| -> Result<Rc<dyn RenderImage>, ()> { panic!("no frame expected") };
        let mut player = ScenePlayer::new(video.clone(), 1.0, true).unwrap();
        player
            .tick(Allocation::Hardware, 0.0, open, upload)
            .unwrap();
        let (generation, settings) = video
            .with_downcast_mut::<Video, _>(|v| {
                v.playback
                    .update_suspension(SuspensionReason::Background, true);
                (v.playback.generation(), v.playback.settings())
            })
            .unwrap();
        assert!(
            player
                .replace_source(PlaybackSettings {
                    rate: f32::NAN,
                    ..settings
                })
                .is_err()
        );
        assert_eq!(live.get(), 1, "invalid replacement must retain old decoder");
        assert_eq!(
            video.with_downcast::<Video, _>(|v| v.playback.generation()),
            Some(generation)
        );
        let replacement = player.replace_source(settings).unwrap();
        assert!(replacement > generation);
        assert_eq!(live.get(), 0, "replacement closes old decoder immediately");
        video.with_downcast_mut::<Video, _>(|v| {
            assert!(!v.playback.accept_frame(generation, 0.5));
            assert!(v.playback.opened(generation, 10.0).is_empty());
            let actions = v.playback.opened(replacement, 10.0);
            assert!(
                !actions.contains(&DecoderAction::Play),
                "background veto survives replacement"
            );
            assert!(v.playback.wants_play());
            assert_eq!(v.playback.position(), 0.0);
        });
        player
            .tick(Allocation::Hardware, 0.1, open, upload)
            .unwrap();
        assert_eq!(live.get(), 1);
        video.with_downcast_mut::<Video, _>(|v| {
            assert!(
                v.playback
                    .update_suspension(SuspensionReason::Background, false)
                    .contains(&DecoderAction::Play)
            );
            v.playback.enqueue(Command::Dispose).unwrap();
        });
        player
            .tick(Allocation::Hardware, 0.2, open, upload)
            .unwrap();
        assert_eq!(live.get(), 0);
        assert!(
            player.replace_source(settings).is_err(),
            "disposed occurrence stays terminal"
        );
    }
    #[test]
    fn lifecycle_pauses_without_a_render_tick_and_keeps_independent_vetoes() {
        use std::cell::RefCell;
        struct ObservedDecoder(Rc<RefCell<Vec<DecoderAction>>>);
        impl Decoder for ObservedDecoder {
            type Error = ();
            fn apply(&mut self, action: DecoderAction) -> Result<(), ()> {
                self.0.borrow_mut().push(action);
                Ok(())
            }
            fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, ()> {
                let mut actions = playback.drain_actions();
                actions.extend(playback.opened(playback.generation(), 10.0));
                for action in actions {
                    self.apply(action)?;
                }
                Ok(None)
            }
        }
        let (_artboard, video) = video_occurrence();
        let actions = Rc::new(RefCell::new(Vec::new()));
        let mut player = ScenePlayer::new(video.clone(), 1.0, true).unwrap();
        player
            .tick(
                Allocation::Hardware,
                0.0,
                |_, _| Ok(ObservedDecoder(actions.clone())),
                |_| panic!("no decoded frame expected"),
            )
            .unwrap();
        assert!(actions.borrow().contains(&DecoderAction::Play));
        actions.borrow_mut().clear();
        video.with_downcast_mut::<Video, _>(|v| {
            assert!(v.playback.accept_frame(v.playback.generation(), 0.75));
            // Lifecycle delivery must work even when author commands fill the queue.
            for _ in 0..256 {
                v.playback.enqueue(Command::Play).unwrap();
            }
        });
        player
            .set_suspended(SuspensionReason::Background, true)
            .unwrap();
        assert_eq!(*actions.borrow(), vec![DecoderAction::Pause]);
        player
            .set_suspended(SuspensionReason::Hidden, true)
            .unwrap();
        player
            .set_suspended(SuspensionReason::Background, false)
            .unwrap();
        assert_eq!(*actions.borrow(), vec![DecoderAction::Pause]);
        player
            .set_suspended(SuspensionReason::Hidden, false)
            .unwrap();
        assert_eq!(
            *actions.borrow(),
            vec![DecoderAction::Pause, DecoderAction::Play]
        );
        video
            .with_downcast::<Video, _>(|v| {
                assert!(v.playback.wants_play());
                assert_eq!(v.playback.position(), 0.75);
            })
            .unwrap();
    }
}
