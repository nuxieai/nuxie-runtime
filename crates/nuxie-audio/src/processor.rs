//! Luau-facing control primitives. The callback sees fixed storage and atomics,
//! never script objects or the producer mutex.
use crate::{AudioEngine, AudioRenderer, AudioRendererNode};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub const PROCESSOR_COMMAND_CAPACITY: usize = 128;
const MAX_FRAME: u64 = (1_u64 << 53) - 1;
const IMMEDIATE: u64 = u64::MAX;

#[derive(Debug, Clone, Copy)]
pub struct OscillatorConfig {
    pub frequency: f64,
    pub gain: f64,
}
impl Default for OscillatorConfig {
    fn default() -> Self {
        Self {
            frequency: 440.0,
            gain: 0.0,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessorUpdate {
    pub frequency: Option<f64>,
    pub gain: Option<f64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum ProcessorState {
    Prepared,
    Scheduled,
    Running,
    Stopping,
    Stopped,
    Failed,
}
impl ProcessorState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Scheduled => "scheduled",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }
    fn from_raw(value: usize) -> Self {
        match value {
            0 => Self::Prepared,
            1 => Self::Scheduled,
            2 => Self::Running,
            3 => Self::Stopping,
            4 => Self::Stopped,
            _ => Self::Failed,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ProcessorStatus {
    pub state: ProcessorState,
    pub error: Option<&'static str>,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessorMeter {
    pub peak: f32,
    pub rms: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessorError(pub &'static str);
impl std::fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ProcessorError {}
type Result<T> = std::result::Result<T, ProcessorError>;

#[derive(Clone, Copy, Default)]
struct Command {
    kind: u64,
    at: u64,
    duration: u64,
    frequency: f64,
    gain: f64,
    flags: u64,
}
struct Slot {
    kind: AtomicU64,
    at: AtomicU64,
    duration: AtomicU64,
    frequency: AtomicU64,
    gain: AtomicU64,
    flags: AtomicU64,
}
impl Slot {
    fn new() -> Self {
        Self {
            kind: AtomicU64::new(0),
            at: AtomicU64::new(0),
            duration: AtomicU64::new(0),
            frequency: AtomicU64::new(0),
            gain: AtomicU64::new(0),
            flags: AtomicU64::new(0),
        }
    }
    fn write(&self, c: Command) {
        self.kind.store(c.kind, Ordering::Relaxed);
        self.at.store(c.at, Ordering::Relaxed);
        self.duration.store(c.duration, Ordering::Relaxed);
        self.frequency
            .store(c.frequency.to_bits(), Ordering::Relaxed);
        self.gain.store(c.gain.to_bits(), Ordering::Relaxed);
        self.flags.store(c.flags, Ordering::Relaxed);
    }
    fn read(&self) -> Command {
        Command {
            kind: self.kind.load(Ordering::Relaxed),
            at: self.at.load(Ordering::Relaxed),
            duration: self.duration.load(Ordering::Relaxed),
            frequency: f64::from_bits(self.frequency.load(Ordering::Relaxed)),
            gain: f64::from_bits(self.gain.load(Ordering::Relaxed)),
            flags: self.flags.load(Ordering::Relaxed),
        }
    }
}
struct Shared {
    slots: [Slot; PROCESSOR_COMMAND_CAPACITY],
    head: AtomicU64,
    tail: AtomicU64,
    outstanding: AtomicUsize,
    state: AtomicUsize,
    meter: AtomicU64,
}
impl Shared {
    fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| Slot::new()),
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            outstanding: AtomicUsize::new(0),
            state: AtomicUsize::new(ProcessorState::Prepared as usize),
            meter: AtomicU64::new(0),
        }
    }
    // Serialized by Producer. Capacity includes both the ring and future events
    // already consumed into the renderer's fixed pending array.
    fn push(&self, c: Command) -> Result<()> {
        if self.outstanding.load(Ordering::Acquire) >= PROCESSOR_COMMAND_CAPACITY {
            return Err(ProcessorError("audio processor command queue is full"));
        }
        self.outstanding.fetch_add(1, Ordering::AcqRel);
        let tail = self.tail.load(Ordering::Relaxed);
        self.slots[tail as usize % PROCESSOR_COMMAND_CAPACITY].write(c);
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }
    fn state(&self) -> ProcessorState {
        ProcessorState::from_raw(self.state.load(Ordering::Acquire))
    }
}
struct Producer {
    started: bool,
    terminal: bool,
    node: Option<AudioRendererNode>,
}
struct Control {
    engine: AudioEngine,
    shared: Arc<Shared>,
    producer: Mutex<Producer>,
    config: OscillatorConfig,
}
impl Drop for Control {
    fn drop(&mut self) {
        let p = self.producer.get_mut().unwrap_or_else(|e| e.into_inner());
        if let Some(node) = p.node.take() {
            self.engine.detach_renderer(node);
        }
        self.shared
            .state
            .store(ProcessorState::Stopped as usize, Ordering::Release);
    }
}
/// Clones share one lifecycle. Last-handle drop synchronously detaches on the
/// control thread; script owners must retain a handle until instance teardown.
#[derive(Clone)]
pub struct AudioProcessor {
    inner: Arc<Control>,
}
impl std::fmt::Debug for AudioProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioProcessor")
            .field("status", &self.status())
            .finish()
    }
}
impl AudioEngine {
    pub fn create_oscillator(&self, config: OscillatorConfig) -> Result<AudioProcessor> {
        validate_frequency(config.frequency, self.sample_rate())?;
        validate_gain(config.gain)?;
        Ok(AudioProcessor {
            inner: Arc::new(Control {
                engine: self.clone(),
                shared: Arc::new(Shared::new()),
                producer: Mutex::new(Producer {
                    started: false,
                    terminal: false,
                    node: None,
                }),
                config,
            }),
        })
    }
}
fn validate_frequency(v: f64, rate: u32) -> Result<()> {
    if !v.is_finite() || v <= 0.0 || v >= f64::from(rate) * 0.5 {
        Err(ProcessorError(
            "frequency must be finite, positive and below Nyquist",
        ))
    } else {
        Ok(())
    }
}
fn validate_gain(v: f64) -> Result<()> {
    if !v.is_finite() || !(0.0..=1.0).contains(&v) {
        Err(ProcessorError("gain must be finite and in [0, 1]"))
    } else {
        Ok(())
    }
}
fn at_frame(at: Option<u64>) -> Result<u64> {
    match at {
        Some(v) if v > MAX_FRAME => Err(ProcessorError("atFrame exceeds exact frame range")),
        Some(v) => Ok(v),
        None => Ok(IMMEDIATE),
    }
}
fn duration(seconds: f64, rate: u32) -> Result<u64> {
    let frames = seconds * f64::from(rate);
    if !seconds.is_finite() || seconds < 0.0 || frames > MAX_FRAME as f64 {
        Err(ProcessorError(
            "duration must be finite, nonnegative and within exact frame range",
        ))
    } else {
        Ok(frames.round() as u64)
    }
}
impl AudioProcessor {
    pub fn start(&self, at: Option<u64>) -> Result<()> {
        let at = at_frame(at)?;
        let mut p = self
            .inner
            .producer
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if p.terminal {
            return Err(ProcessorError("audio processor is stopped"));
        }
        if p.started {
            return Err(ProcessorError("audio processor has already started"));
        }
        self.inner.shared.push(Command {
            kind: 1,
            at,
            ..Command::default()
        })?;
        self.inner
            .shared
            .state
            .store(ProcessorState::Scheduled as usize, Ordering::Release);
        p.started = true;
        p.node = Some(self.inner.engine.attach_renderer(Oscillator::new(
            self.inner.shared.clone(),
            self.inner.config,
            self.inner.engine.sample_rate(),
            self.inner.engine.channels(),
        )));
        Ok(())
    }
    pub fn set(&self, update: ProcessorUpdate, at: Option<u64>, ramp_seconds: f64) -> Result<()> {
        if let Some(v) = update.frequency {
            validate_frequency(v, self.inner.engine.sample_rate())?;
        }
        if let Some(v) = update.gain {
            validate_gain(v)?;
        }
        if update.frequency.is_none() && update.gain.is_none() {
            return Err(ProcessorError("parameter update is empty"));
        }
        let at = at_frame(at)?;
        let duration = duration(ramp_seconds, self.inner.engine.sample_rate())?;
        let p = self
            .inner
            .producer
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if p.terminal {
            return Err(ProcessorError("audio processor is stopped"));
        }
        self.inner.shared.push(Command {
            kind: 2,
            at,
            duration,
            frequency: update.frequency.unwrap_or(0.0),
            gain: update.gain.unwrap_or(0.0),
            flags: u64::from(update.frequency.is_some()) | u64::from(update.gain.is_some()) << 1,
        })
    }
    pub fn stop(&self, at: Option<u64>, fade_seconds: f64) -> Result<()> {
        let at = at_frame(at)?;
        let duration = duration(fade_seconds, self.inner.engine.sample_rate())?;
        let mut p = self
            .inner
            .producer
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if p.terminal {
            return Ok(());
        }
        if !p.started {
            p.terminal = true;
            self.inner
                .shared
                .state
                .store(ProcessorState::Stopped as usize, Ordering::Release);
            return Ok(());
        }
        self.inner.shared.push(Command {
            kind: 3,
            at,
            duration,
            ..Command::default()
        })?;
        p.terminal = true;
        Ok(())
    }
    pub fn cancel(&self) {
        // Serialize the complete teardown with other control calls. The audio
        // callback never takes this mutex, so waiting for detach is safe here.
        // A concurrent second cancel must not return while the first is waiting.
        let mut producer = self
            .inner
            .producer
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        producer.terminal = true;
        if let Some(node) = producer.node.take() {
            self.inner.engine.detach_renderer(node);
        }
        self.inner
            .shared
            .state
            .store(ProcessorState::Stopped as usize, Ordering::Release);
        self.inner.shared.meter.store(0, Ordering::Release);
    }
    pub fn status(&self) -> ProcessorStatus {
        ProcessorStatus {
            state: self.inner.shared.state(),
            error: None,
        }
    }
    pub fn meter(&self) -> ProcessorMeter {
        let v = self.inner.shared.meter.load(Ordering::Acquire);
        ProcessorMeter {
            peak: f32::from_bits(v as u32),
            rms: f32::from_bits((v >> 32) as u32),
        }
    }
}
#[derive(Clone, Copy)]
struct Ramp {
    from: f64,
    to: f64,
    start: u64,
    end: u64,
}
impl Ramp {
    fn fixed(v: f64) -> Self {
        Self {
            from: v,
            to: v,
            start: 0,
            end: 0,
        }
    }
    fn value(self, frame: u64) -> f64 {
        if frame >= self.end || self.end == self.start {
            self.to
        } else if frame <= self.start {
            self.from
        } else {
            self.from
                + (self.to - self.from) * (frame - self.start) as f64
                    / (self.end - self.start) as f64
        }
    }
    fn set(&mut self, to: f64, frame: u64, duration: u64) {
        *self = Self {
            from: self.value(frame),
            to,
            start: frame,
            end: frame.saturating_add(duration),
        };
    }
}
struct Oscillator {
    shared: Arc<Shared>,
    pending: [Command; PROCESSOR_COMMAND_CAPACITY],
    len: usize,
    phase: f64,
    frequency: Ramp,
    gain: Ramp,
    rate: f64,
    channels: usize,
    running: bool,
    stop_at: Option<u64>,
}
impl Oscillator {
    fn new(shared: Arc<Shared>, c: OscillatorConfig, rate: u32, channels: u32) -> Self {
        Self {
            shared,
            pending: [Command::default(); PROCESSOR_COMMAND_CAPACITY],
            len: 0,
            phase: 0.0,
            frequency: Ramp::fixed(c.frequency),
            gain: Ramp::fixed(c.gain),
            rate: f64::from(rate),
            channels: channels as usize,
            running: false,
            stop_at: None,
        }
    }
    fn commands(&mut self, start: u64) {
        let mut head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);
        while head != tail {
            let mut c = self.shared.slots[head as usize % PROCESSOR_COMMAND_CAPACITY].read();
            if c.at == IMMEDIATE || c.at < start {
                c.at = start;
            }
            let mut at = self.len;
            while at > 0 && self.pending[at - 1].at > c.at {
                self.pending[at] = self.pending[at - 1];
                at -= 1;
            }
            self.pending[at] = c;
            self.len += 1;
            head = head.wrapping_add(1);
        }
        self.shared.head.store(head, Ordering::Release);
    }
    fn apply(&mut self, c: Command, frame: u64) {
        // Stop is terminal at its scheduled frame, including commands accepted
        // earlier whose timestamps fall within or after the terminal fade.
        if self.stop_at.is_some() {
            return;
        }
        match c.kind {
            1 => {
                self.running = true;
                self.shared
                    .state
                    .store(ProcessorState::Running as usize, Ordering::Release);
            }
            2 => {
                if c.flags & 1 != 0 {
                    self.frequency.set(c.frequency, frame, c.duration);
                }
                if c.flags & 2 != 0 {
                    self.gain.set(c.gain, frame, c.duration);
                }
            }
            3 => {
                let end = if self.running {
                    frame.saturating_add(c.duration)
                } else {
                    frame
                };
                self.gain.set(0.0, frame, c.duration);
                self.stop_at = Some(end);
                self.shared
                    .state
                    .store(ProcessorState::Stopping as usize, Ordering::Release);
            }
            _ => {}
        }
    }
}
impl AudioRenderer for Oscillator {
    fn render(&mut self, out: &mut [f32], frames: u32, start: u64) {
        out.fill(0.0);
        self.commands(start);
        let mut peak = 0.0_f32;
        let mut sum = 0.0_f64;
        for i in 0..frames as usize {
            let frame = start.saturating_add(i as u64);
            while self.len > 0 && self.pending[0].at <= frame {
                let c = self.pending[0];
                self.pending.copy_within(1..self.len, 0);
                self.len -= 1;
                self.apply(c, frame);
                self.shared.outstanding.fetch_sub(1, Ordering::Release);
            }
            if self.stop_at.is_some_and(|end| frame >= end) {
                self.running = false;
                self.shared
                    .state
                    .store(ProcessorState::Stopped as usize, Ordering::Release);
                break;
            }
            if self.running {
                let sample = (self.phase.sin() * self.gain.value(frame)) as f32;
                for c in 0..self.channels {
                    out[i * self.channels + c] = sample;
                }
                peak = peak.max(sample.abs());
                sum += f64::from(sample) * f64::from(sample);
                self.phase = (self.phase
                    + std::f64::consts::TAU * self.frequency.value(frame) / self.rate)
                    % std::f64::consts::TAU;
            }
        }
        // The next sample may be the fade endpoint exactly at this block boundary.
        if self
            .stop_at
            .is_some_and(|end| start.saturating_add(u64::from(frames)) >= end)
        {
            self.running = false;
            self.shared
                .state
                .store(ProcessorState::Stopped as usize, Ordering::Release);
        }
        let rms = if frames == 0 {
            0.0
        } else {
            (sum / f64::from(frames)).sqrt() as f32
        };
        self.shared.meter.store(
            u64::from(peak.to_bits()) | (u64::from(rms.to_bits()) << 32),
            Ordering::Release,
        );
    }
    fn finished(&self) -> bool {
        self.shared.state() == ProcessorState::Stopped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn renderer() -> (Arc<Shared>, Oscillator) {
        let shared = Arc::new(Shared::new());
        let r = Oscillator::new(
            shared.clone(),
            OscillatorConfig {
                frequency: 1.0,
                gain: 1.0,
            },
            8,
            1,
        );
        (shared, r)
    }
    #[test]
    fn schedule_inside_block_and_stop_endpoint() {
        let (s, mut r) = renderer();
        s.push(Command {
            kind: 1,
            at: 2,
            ..Command::default()
        })
        .unwrap();
        s.push(Command {
            kind: 3,
            at: 6,
            ..Command::default()
        })
        .unwrap();
        let mut out = [9.0; 8];
        r.render(&mut out, 8, 0);
        assert_eq!(&out[..3], &[0.0, 0.0, 0.0]);
        assert!((out[3] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((out[4] - 1.0).abs() < 1e-6);
        assert_eq!(&out[6..], &[0.0, 0.0]);
        assert!(r.finished());
    }
    #[test]
    fn equal_time_updates_are_ordered_and_phase_persists() {
        let (s, mut r) = renderer();
        s.push(Command {
            kind: 1,
            at: 0,
            ..Command::default()
        })
        .unwrap();
        s.push(Command {
            kind: 2,
            at: 2,
            gain: 0.25,
            flags: 2,
            ..Command::default()
        })
        .unwrap();
        s.push(Command {
            kind: 2,
            at: 2,
            gain: 0.5,
            flags: 2,
            ..Command::default()
        })
        .unwrap();
        let mut out = [0.0; 4];
        r.render(&mut out, 4, 0);
        assert!((out[2] - 0.5).abs() < 1e-6);
        assert!((out[3] - 0.5 * std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
    }
    #[test]
    fn terminal_fade_overrides_previously_accepted_future_updates() {
        let (s, mut renderer) = renderer();
        s.push(Command {
            kind: 1,
            at: 0,
            ..Command::default()
        })
        .unwrap();
        // Submitted before stop but scheduled inside its fade.
        s.push(Command {
            kind: 2,
            at: 4,
            gain: 1.0,
            flags: 2,
            ..Command::default()
        })
        .unwrap();
        s.push(Command {
            kind: 3,
            at: 2,
            duration: 4,
            ..Command::default()
        })
        .unwrap();
        let mut output = [0.0; 8];
        renderer.render(&mut output, 8, 0);
        assert!((output[3] - 0.75 * std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((output[5] + 0.25 * std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!(output[6..].iter().all(|sample| *sample == 0.0));
        assert!(renderer.finished());
    }
    #[test]
    fn interrupted_ramp_starts_at_current_value() {
        let mut r = Ramp::fixed(0.0);
        r.set(1.0, 0, 8);
        assert_eq!(r.value(4), 0.5);
        r.set(0.0, 4, 4);
        assert_eq!(r.value(4), 0.5);
        assert_eq!(r.value(6), 0.25);
        assert_eq!(r.value(8), 0.0);
    }
    #[test]
    fn pending_events_keep_capacity_until_execution() {
        let (s, mut r) = renderer();
        for i in 0..PROCESSOR_COMMAND_CAPACITY {
            s.push(Command {
                kind: 2,
                at: 100 + i as u64,
                gain: 0.1,
                flags: 2,
                ..Command::default()
            })
            .unwrap();
        }
        let mut out = [0.0; 1];
        r.render(&mut out, 1, 0);
        assert!(s.push(Command::default()).is_err());
        assert_eq!(r.len, PROCESSOR_COMMAND_CAPACITY);
        r.render(&mut out, 1, 100);
        assert!(
            s.push(Command {
                kind: 2,
                at: 300,
                ..Command::default()
            })
            .is_ok()
        );
    }
    #[test]
    fn validations_stop_before_start_and_cancel() {
        let e = AudioEngine::new(2, 48000).unwrap();
        assert!(
            e.create_oscillator(OscillatorConfig {
                frequency: 24000.0,
                gain: 0.1
            })
            .is_err()
        );
        let p = e.create_oscillator(OscillatorConfig::default()).unwrap();
        assert!(
            p.set(
                ProcessorUpdate {
                    gain: Some(f64::NAN),
                    ..Default::default()
                },
                None,
                0.0
            )
            .is_err()
        );
        assert!(p.start(Some(MAX_FRAME + 1)).is_err());
        p.stop(None, 0.1).unwrap();
        p.stop(None, 0.0).unwrap();
        assert_eq!(p.status().state, ProcessorState::Stopped);
        assert!(p.start(None).is_err());
        p.cancel();
    }
    #[test]
    fn fade_finishes_and_engine_unlinks_without_destroying_control() {
        let e = AudioEngine::new(1, 48000).unwrap();
        let p = e
            .create_oscillator(OscillatorConfig {
                gain: 0.1,
                ..Default::default()
            })
            .unwrap();
        p.start(None).unwrap();
        p.stop(Some(10), 10.0 / 48000.0).unwrap();
        let mut out = [0.0; 128];
        e.read_audio_frames(&mut out);
        assert!(out[1..20].iter().any(|v| *v != 0.0));
        assert!(out[20..].iter().all(|v| *v == 0.0));
        assert_eq!(p.status().state, ProcessorState::Stopped);
        assert_eq!(e.read_audio_frames(&mut out), 0);
        p.cancel();
    }
}
