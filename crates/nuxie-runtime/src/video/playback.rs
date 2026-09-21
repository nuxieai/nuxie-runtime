//! Host-independent playback intent and decoder-generation ownership.
//!
//! Hosts execute returned actions in order and report observations with the
//! generation they belong to. A suspended scene never asks a decoder to play.
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SuspensionReason {
    Hidden = 1,
    Background = 2,
    Interruption = 4,
    Resources = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AudioPolicy {
    Automatic = 0,
    Mix = 1,
    Duck = 2,
    Exclusive = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackSettings {
    pub autoplay: bool,
    pub looping: bool,
    pub loop_start: f64,
    /// Zero selects the source duration. Explicit ends are exclusive.
    pub loop_end: f64,
    pub muted: bool,
    pub volume: f32,
    pub rate: f32,
    pub priority: u32,
    pub audio_policy: u32,
    pub readiness: u32,
    pub reentry: u32,
}
impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            autoplay: false,
            looping: false,
            loop_start: 0.0,
            loop_end: 0.0,
            muted: false,
            volume: 1.0,
            rate: 1.0,
            priority: 0,
            audio_policy: 0,
            readiness: 0,
            reentry: 0,
        }
    }
}
impl PlaybackSettings {
    pub fn valid(self) -> bool {
        valid_loop_range(self.loop_start, self.loop_end)
            && self.volume.is_finite()
            && (0.0..=1.0).contains(&self.volume)
            && self.rate.is_finite()
            && self.rate > 0.0
            && self.audio_policy <= 3
            && self.readiness <= 1
            && self.reentry <= 2
    }
}
fn valid_loop_range(start: f64, end: f64) -> bool {
    start.is_finite() && start >= 0.0 && end.is_finite() && (end == 0.0 || end > start)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Opening,
    Ready,
    Playing,
    Paused,
    Seeking,
    Buffering,
    Ended,
    Failed,
    Disposed,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Play,
    Pause,
    Seek(f64),
    Interactive {
        id: u64,
        start: f64,
        end: f64,
        target: f64,
        play: bool,
    },
    Rate(f32),
    Volume(f32),
    Mute(bool),
    Loop(bool),
    LoopRange {
        start: f64,
        end: f64,
    },
    Suspend(bool),
    SuspendReason {
        reason: SuspensionReason,
        suspended: bool,
    },
    Reenter,
    Dispose,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecoderAction {
    Play,
    Pause,
    Seek { seconds: f64, generation: u64 },
    Rate(f32),
    Volume(f32),
    Dispose,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackEvent {
    State(PlaybackState),
    FirstFrame,
    Looped,
    PlayBlocked,
    ResourceLimited(bool),
    Error,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackError {
    InvalidValue,
    Failed,
    Disposed,
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestState {
    Pending,
    Playing,
    Settled,
    Completed,
    Cancelled,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractiveRequest {
    pub id: u64,
    pub state: RequestState,
    start: f64,
    end: f64,
    target: f64,
    play: bool,
    accepted_frame: bool,
}

#[derive(Debug)]
pub struct Playback {
    request: Option<InteractiveRequest>,
    next_request: u64,
    restore_held_frame: bool,
    selected_seek_frame: Option<(u64, u64, f64)>,
    settings: PlaybackSettings,
    state: PlaybackState,
    wants_play: bool,
    suspension_reasons: u8,
    ready: bool,
    generation: u64,
    position: f64,
    duration: Option<f64>,
    first_frame: bool,
    commands: VecDeque<Command>,
    pending_actions: Vec<DecoderAction>,
    events: VecDeque<PlaybackEvent>,
}
impl Default for Playback {
    fn default() -> Self {
        Self::new(PlaybackSettings::default())
    }
}
impl Playback {
    pub fn new(settings: PlaybackSettings) -> Self {
        Self {
            request: None,
            next_request: 0,
            restore_held_frame: false,
            selected_seek_frame: None,
            settings,
            state: if settings.valid() {
                PlaybackState::Opening
            } else {
                PlaybackState::Failed
            },
            wants_play: settings.autoplay,
            suspension_reasons: 0,
            ready: false,
            generation: 0,
            position: 0.0,
            duration: None,
            first_frame: false,
            commands: VecDeque::new(),
            pending_actions: Vec::new(),
            events: VecDeque::new(),
        }
    }
    /// Begin a new source occurrence. The host must close the previous decoder
    /// and open the replacement using the returned generation. Lifecycle vetoes
    /// survive replacement; old queued commands/events and frame clocks do not.
    pub fn replace_source(&mut self, settings: PlaybackSettings) -> Result<u64, PlaybackError> {
        let generation = self.validate_source_replacement(settings)?;
        let next_request = self.next_request;
        let suspension_reasons = self.suspension_reasons;
        *self = Self::new(settings);
        self.next_request = next_request;
        self.generation = generation;
        self.suspension_reasons = suspension_reasons;
        self.emit(PlaybackEvent::State(PlaybackState::Opening));
        Ok(generation)
    }
    /// Validate a replacement before a host closes its current decoder.
    /// The returned generation is only committed by replace_source.
    pub fn validate_source_replacement(
        &self,
        settings: PlaybackSettings,
    ) -> Result<u64, PlaybackError> {
        if self.state == PlaybackState::Disposed {
            return Err(PlaybackError::Disposed);
        }
        if !settings.valid() {
            return Err(PlaybackError::InvalidValue);
        }
        self.generation
            .checked_add(1)
            .ok_or(PlaybackError::InvalidValue)
    }
    /// Reclaim a decoder without replacing its source or losing authored intent.
    /// The next decoder must open with this generation. Its ready observation
    /// restores the retained position before issuing play.
    pub fn reclaim_decoder(&mut self) -> Result<u64, PlaybackError> {
        if matches!(self.state, PlaybackState::Disposed | PlaybackState::Failed) {
            return Err(if self.state == PlaybackState::Disposed {
                PlaybackError::Disposed
            } else {
                PlaybackError::Failed
            });
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(PlaybackError::InvalidValue)?;
        self.ready = false;
        self.restore_held_frame = true;
        self.pending_actions.clear();
        self.first_frame = false;
        self.transition(PlaybackState::Opening);
        Ok(self.generation)
    }
    pub fn settings(&self) -> PlaybackSettings {
        self.settings
    }
    pub fn state(&self) -> PlaybackState {
        self.state
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn position(&self) -> f64 {
        self.position
    }
    /// Duration reported by the current source's decoder, in seconds.
    /// Unknown until metadata arrives; source replacement clears this value.
    pub fn duration(&self) -> Option<f64> {
        self.duration
    }
    pub fn wants_play(&self) -> bool {
        self.wants_play
    }
    pub fn pop_event(&mut self) -> Option<PlaybackEvent> {
        self.events.pop_front()
    }
    pub fn request_status(&self) -> Option<InteractiveRequest> {
        self.request
    }
    /// Cancel only the current owner; a stale caller cannot pause its replacement.
    pub fn cancel_request(&mut self, id: u64) -> Result<bool, PlaybackError> {
        if self.request.is_none_or(|request| request.id != id) {
            return Ok(false);
        }
        self.can_enqueue(Command::Pause)?;
        self.commands
            .retain(|command| !matches!(command, Command::Interactive { .. }));
        self.commands.push_back(Command::Pause);
        if let Some(request) = &mut self.request {
            request.state = RequestState::Cancelled;
        }
        Ok(true)
    }
    /// Ranges are half-open; completion retains the last accepted in-range image.
    /// Audio stopping is observation-driven, not a sample-exact decoder boundary.
    pub fn play_range(&mut self, start: f64, end: f64) -> Result<u64, PlaybackError> {
        self.interactive(start, end, start, true)
    }
    /// New requests replace queued interactive work; ordinary commands retain order.
    pub fn scrub(&mut self, progress: f64, start: f64, end: f64) -> Result<u64, PlaybackError> {
        if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
            return Err(PlaybackError::InvalidValue);
        }
        // Scrubbing includes the endpoint, unlike play-once's exclusive end.
        self.interactive(start, end, start + progress * (end - start), false)
    }
    fn interactive(
        &mut self,
        start: f64,
        end: f64,
        target: f64,
        play: bool,
    ) -> Result<u64, PlaybackError> {
        if !play && self.commands.is_empty() {
            if let Some(request) = self.request {
                if !request.play
                    && request.start == start
                    && request.end == end
                    && request.target == target
                    && matches!(request.state, RequestState::Pending | RequestState::Settled)
                {
                    return Ok(request.id);
                }
            }
        }
        let id = self
            .next_request
            .checked_add(1)
            .ok_or(PlaybackError::InvalidValue)?;
        let command = Command::Interactive {
            id,
            start,
            end,
            target,
            play,
        };
        self.can_enqueue(command)?;
        self.commands
            .retain(|command| !matches!(command, Command::Interactive { .. }));
        self.selected_seek_frame = None;
        self.commands.push_back(command);
        self.next_request = id;
        self.request = Some(InteractiveRequest {
            id,
            state: RequestState::Pending,
            accepted_frame: false,
            start,
            end,
            target,
            play,
        });
        Ok(id)
    }
    pub fn enqueue(&mut self, command: Command) -> Result<(), PlaybackError> {
        self.can_enqueue(command)?;
        self.selected_seek_frame = None;
        self.commands.push_back(command);
        Ok(())
    }
    /// Preflight for an atomic authored group command on the owning thread.
    /// This does not reserve capacity; callers must not run callbacks between
    /// validation and enqueueing the same command to the validated occurrences.
    pub fn can_enqueue(&self, command: Command) -> Result<(), PlaybackError> {
        if self.state == PlaybackState::Disposed {
            return Err(PlaybackError::Disposed);
        }
        if self.state == PlaybackState::Failed && command != Command::Dispose {
            return Err(PlaybackError::Failed);
        }
        let valid = match command {
            Command::Interactive {
                start, end, target, ..
            } => {
                start.is_finite()
                    && end.is_finite()
                    && target.is_finite()
                    && start >= 0.0
                    && end > start
                    && target >= start
                    && target <= end
                    && self.duration.is_none_or(|duration| end <= duration)
            }
            Command::LoopRange { start, end } => {
                valid_loop_range(start, end)
                    && self.duration.is_none_or(|duration| start < duration)
            }
            Command::Loop(true) => self
                .duration
                .is_none_or(|duration| self.settings.loop_start < duration),
            Command::Seek(t) => t.is_finite() && t >= 0.0,
            Command::Rate(r) => r.is_finite() && r > 0.0,
            Command::Volume(v) => v.is_finite() && (0.0..=1.0).contains(&v),
            _ => true,
        };
        if !valid {
            return Err(PlaybackError::InvalidValue);
        }
        if self.commands.len() >= 256
            && !(matches!(command, Command::Interactive { .. })
                && self
                    .commands
                    .iter()
                    .any(|command| matches!(command, Command::Interactive { .. })))
        {
            return Err(PlaybackError::QueueFull);
        }
        Ok(())
    }
    fn emit(&mut self, event: PlaybackEvent) {
        if self.events.len() == 256 {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
    fn transition(&mut self, state: PlaybackState) {
        if self.state != state {
            self.state = state;
            self.emit(PlaybackEvent::State(state));
        }
    }
    fn seek(&mut self, seconds: f64, actions: &mut Vec<DecoderAction>) {
        self.selected_seek_frame = None;
        self.generation = self.generation.wrapping_add(1);
        self.position = self.duration.map_or(seconds, |d| seconds.min(d));
        if self.loop_enabled()
            && (self.position < self.settings.loop_start
                || self.loop_end().is_some_and(|end| self.position >= end))
        {
            self.position = self.settings.loop_start;
        }
        self.transition(PlaybackState::Seeking);
        actions.push(DecoderAction::Seek {
            seconds: self.position,
            generation: self.generation,
        });
    }
    pub fn is_suspended(&self, reason: SuspensionReason) -> bool {
        self.suspension_reasons & reason as u8 != 0
    }
    fn set_suspension(&mut self, reason: SuspensionReason, suspended: bool) {
        let changed = self.is_suspended(reason) != suspended;
        if suspended {
            self.suspension_reasons |= reason as u8;
        } else {
            self.suspension_reasons &= !(reason as u8);
        }
        if changed && reason == SuspensionReason::Resources {
            self.emit(PlaybackEvent::ResourceLimited(suspended));
        }
    }
    /// Lifecycle observations cannot be dropped because an authored command
    /// queue is full. Independent reasons prevent one owner resuming another.
    pub fn update_suspension(
        &mut self,
        reason: SuspensionReason,
        suspended: bool,
    ) -> Vec<DecoderAction> {
        let before = self.effective_play();
        self.set_suspension(reason, suspended);
        let after = self.effective_play();
        if before == after {
            return Vec::new();
        }
        if !after && !matches!(self.state, PlaybackState::Seeking | PlaybackState::Ended) {
            self.transition(PlaybackState::Paused);
        }
        vec![if after {
            DecoderAction::Play
        } else {
            DecoderAction::Pause
        }]
    }
    fn loop_enabled(&self) -> bool {
        self.settings.looping
            && self.request.is_none_or(|request| {
                matches!(
                    request.state,
                    RequestState::Cancelled | RequestState::Failed
                )
            })
    }
    fn loop_end(&self) -> Option<f64> {
        if self.settings.loop_end == 0.0 {
            self.duration
        } else {
            Some(self.duration.map_or(self.settings.loop_end, |duration| {
                duration.min(self.settings.loop_end)
            }))
        }
    }
    fn repair_loop_position(&mut self, actions: &mut Vec<DecoderAction>) {
        if self.loop_enabled()
            && (self.position < self.settings.loop_start
                || self.loop_end().is_some_and(|end| self.position >= end))
        {
            self.seek(self.settings.loop_start, actions);
        }
    }
    fn effective_play(&self) -> bool {
        self.wants_play && self.suspension_reasons == 0 && self.ready
    }
    /// Drain one ordered batch. Lifecycle state is applied before the final play
    /// action, so a play immediately followed by suspend cannot leak audio.
    pub fn drain_actions(&mut self) -> Vec<DecoderAction> {
        let mut actions = std::mem::take(&mut self.pending_actions);
        let before = self.effective_play();
        let mut retry_play = false;
        while let Some(command) = self.commands.pop_front() {
            if matches!(
                command,
                Command::Play
                    | Command::Seek(_)
                    | Command::Loop(_)
                    | Command::LoopRange { .. }
                    | Command::Reenter
                    | Command::Dispose
            ) {
                if let Some(request) = &mut self.request {
                    request.state = RequestState::Cancelled;
                }
            }
            match command {
                Command::Interactive {
                    id,
                    start,
                    end,
                    target,
                    play,
                } => {
                    self.request = Some(InteractiveRequest {
                        id,
                        state: RequestState::Pending,
                        accepted_frame: false,
                        start,
                        end,
                        target,
                        play,
                    });
                    self.wants_play = play;
                    self.seek(target, &mut actions);
                    retry_play = play;
                }
                Command::Loop(enabled) => {
                    self.settings.looping = enabled;
                    self.repair_loop_position(&mut actions);
                }
                Command::LoopRange { start, end } => {
                    self.settings.loop_start = start;
                    self.settings.loop_end = end;
                    self.repair_loop_position(&mut actions);
                }
                Command::Play => {
                    retry_play = self.state != PlaybackState::Playing;
                    if self.state == PlaybackState::Ended {
                        self.seek(0.0, &mut actions);
                    }
                    self.wants_play = true;
                }
                Command::Pause => self.wants_play = false,
                Command::Suspend(value) => self.set_suspension(SuspensionReason::Background, value),
                Command::SuspendReason { reason, suspended } => {
                    self.set_suspension(reason, suspended)
                }
                Command::Seek(seconds) => self.seek(seconds, &mut actions),
                Command::Rate(rate) => {
                    self.settings.rate = rate;
                    actions.push(DecoderAction::Rate(rate));
                }
                Command::Volume(volume) => {
                    self.settings.volume = volume;
                    actions.push(DecoderAction::Volume(if self.settings.muted {
                        0.0
                    } else {
                        volume
                    }));
                }
                Command::Mute(muted) => {
                    self.settings.muted = muted;
                    actions.push(DecoderAction::Volume(if muted {
                        0.0
                    } else {
                        self.settings.volume
                    }));
                }
                Command::Reenter => {
                    if self.settings.reentry == 0 {
                        self.seek(0.0, &mut actions);
                        self.wants_play = self.settings.autoplay;
                    }
                }
                Command::Dispose => {
                    self.selected_seek_frame = None;
                    self.generation = self.generation.wrapping_add(1);
                    self.ready = false;
                    self.wants_play = false;
                    self.commands.clear();
                    self.transition(PlaybackState::Disposed);
                    return vec![DecoderAction::Dispose];
                }
            }
        }
        let after = self.effective_play();
        if !after {
            actions.retain(|action| *action != DecoderAction::Play);
        }
        if before != after || (after && retry_play) {
            actions.push(if after {
                DecoderAction::Play
            } else {
                DecoderAction::Pause
            });
        }
        if self.ready && self.state != PlaybackState::Seeking && self.state != PlaybackState::Ended
        {
            // Playing is acknowledged by the decoder, not inferred from intent.
            if !after {
                self.transition(PlaybackState::Paused);
            }
        }
        actions
    }
    pub fn opened(&mut self, generation: u64, duration: f64) -> Vec<DecoderAction> {
        if generation != self.generation
            || self.ready
            || matches!(self.state, PlaybackState::Disposed | PlaybackState::Failed)
            || !duration.is_finite()
            || duration < 0.0
        {
            return Vec::new();
        }
        if self.loop_enabled() && self.settings.loop_start >= duration {
            self.failed(generation);
            return vec![DecoderAction::Dispose];
        }
        if self
            .request
            .is_some_and(|r| r.end > duration && r.state == RequestState::Pending)
        {
            self.failed(generation);
            return vec![DecoderAction::Dispose];
        }
        self.duration = Some(duration);
        if self.loop_enabled() {
            self.position = self.position.max(self.settings.loop_start);
        }
        self.ready = true;
        self.transition(PlaybackState::Ready);
        let mut actions = vec![
            DecoderAction::Rate(self.settings.rate),
            DecoderAction::Volume(if self.settings.muted {
                0.0
            } else {
                self.settings.volume
            }),
        ];
        if self.position > 0.0 {
            self.position = self.position.min(duration);
            actions.push(DecoderAction::Seek {
                seconds: self.position,
                generation: self.generation,
            });
        }
        if self.effective_play() {
            actions.push(DecoderAction::Play);
        }
        actions
    }
    pub fn observed_playing(&mut self, generation: u64) {
        if generation == self.generation && self.effective_play() {
            self.transition(PlaybackState::Playing);
        }
    }
    /// Certify the actual image selected by a successfully completed seek.
    /// Hosts must report decoded PTS, never the requested time or media clock,
    /// immediately before presenting those same pixels. Only a pending scrub
    /// at the source endpoint can use this one-shot receipt for an audio tail.
    pub fn observe_selected_seek_frame(&mut self, generation: u64, pts: f64) -> bool {
        let Some(request) = self.request else {
            return false;
        };
        if generation != self.generation
            || !self.ready
            || !self.commands.is_empty()
            || !pts.is_finite()
            || pts < request.start
            || pts > request.end
            || request.play
            || request.state != RequestState::Pending
            || request.target != request.end
            || self.duration != Some(request.end)
            || matches!(self.state, PlaybackState::Failed | PlaybackState::Disposed)
        {
            return false;
        }
        self.selected_seek_frame = Some((request.id, generation, pts));
        true
    }
    pub fn accept_frame(&mut self, generation: u64, pts: f64) -> bool {
        let selected_seek_frame = self.selected_seek_frame.take();
        if generation != self.generation
            || !pts.is_finite()
            || pts < 0.0
            || matches!(self.state, PlaybackState::Disposed | PlaybackState::Failed)
        {
            return false;
        }
        if let Some(request) = self.request {
            match request.state {
                RequestState::Pending | RequestState::Playing => {
                    if self
                        .commands
                        .iter()
                        .any(|command| matches!(command, Command::Interactive { .. }))
                    {
                        return false;
                    }
                    if pts < request.start {
                        return false;
                    }
                    if request.play && pts >= request.end {
                        self.wants_play = false;
                        if let Some(current) = &mut self.request {
                            current.state = if request.accepted_frame {
                                RequestState::Completed
                            } else {
                                RequestState::Failed
                            };
                        }
                        self.transition(PlaybackState::Paused);
                        self.pending_actions.push(DecoderAction::Pause);
                        return false;
                    }
                    if !request.play && pts > request.end {
                        return false;
                    }
                    // A seek can produce preroll within the same generation. Do not
                    // call that settled merely because it belongs to this range.
                    // 50 ms is an explicit presentation tolerance, not exact seek.
                    if !request.play
                        && !request.accepted_frame
                        && (pts - request.target).abs() > 0.05
                        && selected_seek_frame != Some((request.id, generation, pts))
                    {
                        return false;
                    }
                    if let Some(current) = &mut self.request {
                        current.accepted_frame = true;
                        current.state = if request.play {
                            RequestState::Playing
                        } else {
                            RequestState::Settled
                        };
                    }
                }
                RequestState::Completed | RequestState::Settled => {
                    if !self.restore_held_frame
                        || pts < request.start
                        || pts > request.end
                        || (request.play && pts == request.end)
                        || (pts - self.position).abs() > 0.05
                    {
                        return false;
                    }
                    self.restore_held_frame = false;
                }
                RequestState::Failed => return false,
                RequestState::Cancelled => {}
            }
        }
        if self.loop_enabled() {
            if pts < self.settings.loop_start {
                return false;
            }
            if self.loop_end().is_some_and(|end| pts >= end) {
                if self.effective_play() {
                    let actions = self.ended(generation);
                    self.pending_actions = actions;
                }
                return false;
            }
        }
        self.restore_held_frame = false;
        self.position = pts;
        if !self.first_frame {
            self.first_frame = true;
            self.emit(PlaybackEvent::FirstFrame);
        }
        if self.state == PlaybackState::Seeking {
            self.transition(if self.effective_play() {
                PlaybackState::Playing
            } else {
                PlaybackState::Paused
            });
        }
        true
    }
    pub fn ended(&mut self, generation: u64) -> Vec<DecoderAction> {
        if generation != self.generation
            || !self.ready
            || matches!(
                self.state,
                PlaybackState::Ended | PlaybackState::Disposed | PlaybackState::Failed
            )
        {
            return Vec::new();
        }
        if self
            .commands
            .iter()
            .any(|command| matches!(command, Command::Interactive { .. }))
        {
            return Vec::new();
        }
        if let Some(request) = &mut self.request {
            if request.play
                && matches!(request.state, RequestState::Pending | RequestState::Playing)
            {
                request.state = if request.accepted_frame {
                    RequestState::Completed
                } else {
                    RequestState::Failed
                };
                self.wants_play = false;
                self.transition(PlaybackState::Paused);
                return vec![DecoderAction::Pause];
            }
        }
        if self.loop_enabled() {
            let mut actions = Vec::new();
            self.seek(self.settings.loop_start, &mut actions);
            self.emit(PlaybackEvent::Looped);
            if self.effective_play() {
                actions.push(DecoderAction::Play);
            }
            actions
        } else {
            self.wants_play = false;
            self.transition(PlaybackState::Ended);
            vec![DecoderAction::Pause]
        }
    }
    pub fn observed_play_blocked(&mut self, generation: u64) {
        if generation == self.generation && self.effective_play() {
            self.transition(PlaybackState::Paused);
            self.emit(PlaybackEvent::PlayBlocked);
        }
    }
    pub fn observed_buffering(&mut self, generation: u64) {
        if generation == self.generation
            && self.effective_play()
            && !matches!(
                self.state,
                PlaybackState::Disposed
                    | PlaybackState::Failed
                    | PlaybackState::Seeking
                    | PlaybackState::Ended
            )
        {
            self.transition(PlaybackState::Buffering);
        }
    }
    pub fn failed(&mut self, generation: u64) {
        if generation == self.generation
            && !matches!(self.state, PlaybackState::Disposed | PlaybackState::Failed)
        {
            if let Some(request) = &mut self.request {
                request.state = RequestState::Failed;
            }
            self.ready = false;
            self.commands.clear();
            self.pending_actions.clear();
            self.transition(PlaybackState::Failed);
            self.emit(PlaybackEvent::Error);
        }
    }
}
