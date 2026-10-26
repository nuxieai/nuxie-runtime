//! Device qualification only: generated PCM plus a decoded WAV fixture through
//! the production Rust mixer. This is not a shipping C or scripting API.
use nuxie_audio::{AudioEngine, AudioRenderer, AudioRendererNode, AudioSound, AudioSource};
use std::ffi::{CString, c_char};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, AtomicU64, Ordering},
};

#[cfg(target_os = "android")]
mod android;

#[derive(Default)]
pub struct Signals {
    pub phase: AtomicU32,
    pub calls: [AtomicU64; 4],
    pub drops: AtomicU64,
    pub clock_errors: AtomicU64,
}
struct Tone {
    signals: Arc<Signals>,
    channels: usize,
    rate: f32,
    angle: f32,
    next: Option<u64>,
}
impl AudioRenderer for Tone {
    fn render(&mut self, out: &mut [f32], frames: u32, start: u64) {
        if self.next.is_some_and(|next| next != start) {
            self.signals.clock_errors.fetch_add(1, Ordering::Relaxed);
        }
        self.next = Some(start + u64::from(frames));
        let phase = self.signals.phase.load(Ordering::Acquire).min(3) as usize;
        self.signals.calls[phase].fetch_add(1, Ordering::Relaxed);
        let (hz, gain) = if phase == 0 {
            (440.0, 0.02)
        } else {
            (880.0, 0.01)
        };
        for frame in out.chunks_exact_mut(self.channels) {
            let value = self.angle.sin() * gain;
            frame.fill(value);
            self.angle =
                (self.angle + std::f32::consts::TAU * hz / self.rate) % std::f32::consts::TAU;
        }
    }
}
impl Drop for Tone {
    fn drop(&mut self) {
        self.signals.drops.fetch_add(1, Ordering::Release);
    }
}

#[derive(Default, Clone, Copy)]
struct Measurement {
    frames: u64,
    settled_frames: u64,
    squares: f64,
    peak: f32,
    invalid: u64,
}
/// Host serializes phase changes and destruction. Pulls may run on its audio
/// thread, but destruction requires stopping/joining that thread first.
pub struct Probe {
    pub engine: AudioEngine,
    pub signals: Arc<Signals>,
    sound: AudioSound,
    node: Mutex<Option<AudioRendererNode>>,
    measurements: Mutex<[Measurement; 4]>,
}

fn fixture() -> Vec<u8> {
    let rate = 24000u32;
    let count = rate * 12;
    let data_len = count * 2;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..count {
        let sample = ((std::f64::consts::TAU * 220.0 * f64::from(i) / f64::from(rate)).sin()
            * 0.025
            * f64::from(i16::MAX)) as i16;
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}
impl Probe {
    pub fn from_engine(engine: AudioEngine) -> Self {
        let signals = Arc::new(Signals::default());
        let source = Arc::new(AudioSource::from_encoded(fixture()).expect("generated PCM WAV"));
        let sound = engine
            .play(source, 0, 0, 0, None)
            .expect("fixture playback");
        let node = engine.attach_renderer(Tone {
            signals: signals.clone(),
            channels: engine.channels() as usize,
            rate: engine.sample_rate() as f32,
            angle: 0.0,
            next: None,
        });
        Self {
            engine,
            signals,
            sound,
            node: Mutex::new(Some(node)),
            measurements: Mutex::new([Measurement::default(); 4]),
        }
    }
    pub fn set_phase(&self, phase: u32) {
        assert!(phase < 4);
        // Detach completes before publishing phase2: any phase2 renderer call
        // is therefore a real lifetime/ordering failure.
        if phase >= 2 {
            if let Some(node) = self.node.lock().unwrap().take() {
                self.engine.detach_renderer(node);
            }
        }
        if phase == 3 {
            self.sound.stop(0);
        }
        self.signals.phase.store(phase, Ordering::Release);
    }
    pub fn pull(&self, output: &mut [f32]) -> u64 {
        let phase = self.signals.phase.load(Ordering::Acquire).min(3) as usize;
        let read = self.engine.read_audio_frames(output);
        let channels = self.engine.channels() as usize;
        let mut all = self.measurements.lock().unwrap();
        let m = &mut all[phase];
        let settle = u64::from(self.engine.buffer_frames()) * 2;
        for frame in output.chunks_exact(channels) {
            m.frames += 1;
            if m.frames <= settle {
                continue;
            }
            m.settled_frames += 1;
            for &sample in frame {
                if !sample.is_finite() {
                    m.invalid += 1;
                }
                m.squares += f64::from(sample) * f64::from(sample);
                m.peak = m.peak.max(sample.abs());
            }
        }
        read
    }
    pub fn report(&self) -> String {
        let m = *self.measurements.lock().unwrap();
        let calls = self
            .signals
            .calls
            .each_ref()
            .map(|v| v.load(Ordering::Acquire));
        let drops = self.signals.drops.load(Ordering::Acquire);
        let clock_errors = self.signals.clock_errors.load(Ordering::Acquire);
        let rms = m.map(|m| {
            (m.squares / (m.settled_frames.max(1) * u64::from(self.engine.channels())) as f64)
                .sqrt()
        });
        let pass = rms[0] > rms[1] * 1.10
            && rms[1] > rms[2] * 1.03
            && m.iter().all(|m| m.settled_frames >= 1024 && m.invalid == 0)
            && m[..3].iter().all(|m| m.peak > 0.005 && m.peak < 0.1)
            && m[3].peak == 0.0
            && calls[0] > 0
            && calls[1] > 0
            && calls[2] == 0
            && calls[3] == 0
            && drops == 1
            && clock_errors == 0;
        let phases = m.iter().enumerate().map(|(i,m)| format!("{{\"phase\":{i},\"frames\":{},\"settled_frames\":{},\"rms\":{},\"peak\":{},\"invalid\":{},\"renderer_calls\":{}}}", m.frames,m.settled_frames,(m.squares / (m.settled_frames.max(1) * u64::from(self.engine.channels())) as f64).sqrt(),m.peak,m.invalid,calls[i])).collect::<Vec<_>>().join(",");
        format!(
            "{{\"pass\":{pass},\"rate\":{},\"channels\":{},\"block_frames\":{},\"clock\":{},\"renderer_drops\":{drops},\"clock_errors\":{clock_errors},\"phases\":[{phases}]}}",
            self.engine.sample_rate(),
            self.engine.channels(),
            self.engine.buffer_frames(),
            self.engine.time_in_frames()
        )
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        if let Some(node) = self.node.get_mut().unwrap().take() {
            self.engine.detach_renderer(node);
        }
        self.sound.stop(0);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nux_audio_probe_new(rate: u32, channels: u32, block: u32) -> *mut Probe {
    if !(8000..=192000).contains(&rate)
        || !(1..=8).contains(&channels)
        || !(1..=8192).contains(&block)
    {
        return std::ptr::null_mut();
    }
    AudioEngine::set_buffer_frames(block);
    let engine = AudioEngine::new(channels, rate).unwrap();
    AudioEngine::set_buffer_frames(0);
    Box::into_raw(Box::new(Probe::from_engine(engine)))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_audio_probe_pull(
    probe: *mut Probe,
    out: *mut f32,
    frames: u32,
) -> u64 {
    if probe.is_null() || out.is_null() || frames > 65536 {
        return 0;
    }
    let p = unsafe { &*probe };
    p.pull(unsafe {
        std::slice::from_raw_parts_mut(out, frames as usize * p.engine.channels() as usize)
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_audio_probe_set_phase(probe: *mut Probe, phase: u32) {
    if !probe.is_null() && phase < 4 {
        unsafe { &*probe }.set_phase(phase);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_audio_probe_report_json(probe: *mut Probe) -> *mut c_char {
    if probe.is_null() {
        return std::ptr::null_mut();
    }
    CString::new(unsafe { &*probe }.report())
        .unwrap()
        .into_raw()
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_audio_probe_free_json(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_audio_probe_destroy(probe: *mut Probe) {
    if !probe.is_null() {
        drop(unsafe { Box::from_raw(probe) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_mixes_changes_detaches_and_silences() {
        let p = unsafe { Box::from_raw(nux_audio_probe_new(48000, 2, 256)) };
        for phase in 0..4 {
            p.set_phase(phase);
            for _ in 0..100 {
                assert_eq!(p.pull(&mut [0.0; 512]), 256);
            }
        }
        assert!(p.report().contains("\"pass\":true"), "{}", p.report());
    }
}
