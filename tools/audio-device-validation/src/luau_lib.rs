//! Actual Luau controls driving the shipping Rust oscillator on device outputs.
//! VM ownership stays on the creating thread; callbacks see only `Probe`.
use luaur_rt::Table;
use nuxie_runtime::{AudioEngine, NoopScriptHost, ScriptInstance};
use nuxie_scripting::vm::{LuaScriptInstance, ScriptVm};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{CString, c_char};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::thread::{self, ThreadId};

#[cfg(target_os = "android")]
mod android;

#[derive(Clone, Copy, Default)]
struct Measurement {
    frames: u64,
    settled: u64,
    squares: f64,
    peak: f32,
    invalid: u64,
}

pub struct Probe {
    pub engine: AudioEngine,
    phase: AtomicU32,
    measurements: Mutex<[Measurement; 4]>,
    control_error: Mutex<Option<String>>,
    stopped: AtomicBool,
    ownership_dropped: AtomicBool,
    owner: ThreadId,
}
struct Control {
    // Keep VM and table alive after the instance goes away: this tests actual
    // instance ownership, not merely final userdata GC.
    _vm: ScriptVm,
    table: Table,
    instance: Option<LuaScriptInstance>,
}
thread_local! { static CONTROLS: RefCell<BTreeMap<usize, Control>> = RefCell::new(BTreeMap::new()); }

impl Probe {
    pub fn new(rate: u32, channels: u32, block: u32) -> Result<Arc<Self>, String> {
        AudioEngine::set_buffer_frames(block);
        let engine = AudioEngine::make_and_store(channels, rate).map_err(|e| e.to_string());
        AudioEngine::set_buffer_frames(0);
        let engine = engine?;
        let vm = ScriptVm::new();
        vm.install_rive_globals().map_err(|e| e.to_string())?;
        let table: Table = vm
            .lua()
            .load(include_str!("luau_fixture.lua"))
            .eval()
            .map_err(|e| e.to_string())?;
        let mut instance = vm.script_instance_from_table(table.clone());
        if !instance
            .call_init(&mut NoopScriptHost)
            .map_err(|e| e.to_string())?
        {
            return Err("script initialization returned false".into());
        }
        let probe = Arc::new(Self {
            engine,
            phase: AtomicU32::new(0),
            measurements: Mutex::new([Measurement::default(); 4]),
            control_error: Mutex::new(None),
            stopped: AtomicBool::new(false),
            ownership_dropped: AtomicBool::new(false),
            owner: thread::current().id(),
        });
        CONTROLS.with(|all| {
            all.borrow_mut().insert(
                Arc::as_ptr(&probe) as usize,
                Control {
                    _vm: vm,
                    table,
                    instance: Some(instance),
                },
            )
        });
        Ok(probe)
    }
    pub fn set_phase(&self, phase: u32) {
        if phase == 0 {
            return;
        }
        if self.owner != thread::current().id() {
            *self.control_error.lock().unwrap() = Some("controls called from wrong thread".into());
            return;
        }
        let result = CONTROLS.with(|all| -> Result<(), String> {
            let mut all = all.borrow_mut();
            let control = all
                .get_mut(&(self as *const Self as usize))
                .ok_or("missing control owner")?;
            let instance = control
                .instance
                .as_mut()
                .ok_or("instance already dropped")?;
            instance
                .call_advance_truthy(phase as f32, &mut NoopScriptHost)
                .map_err(|e| e.to_string())?;
            if phase == 3 {
                self.stopped.store(
                    control
                        .table
                        .get::<bool>("stopped")
                        .map_err(|e| e.to_string())?,
                    Ordering::Release,
                );
                drop(control.instance.take());
                self.ownership_dropped.store(true, Ordering::Release);
            }
            Ok(())
        });
        if let Err(error) = result {
            eprintln!("Luau audio probe control: {error}");
            *self.control_error.lock().unwrap() = Some(error);
        }
        self.phase.store(phase, Ordering::Release);
    }
    pub fn pull(&self, output: &mut [f32]) -> u64 {
        let read = self.engine.read_audio_frames(output);
        let phase = self.phase.load(Ordering::Acquire).min(3) as usize;
        let mut measurements = self.measurements.lock().unwrap();
        let m = &mut measurements[phase];
        // Allow commands, their 20ms ramps, and bounded cached mixer output to settle.
        let settle = u64::from(self.engine.sample_rate() / 10)
            .max(u64::from(self.engine.buffer_frames()) * 2);
        for frame in output.chunks_exact(self.engine.channels() as usize) {
            m.frames += 1;
            if m.frames <= settle {
                continue;
            }
            m.settled += 1;
            for &sample in frame {
                m.invalid += u64::from(!sample.is_finite());
                m.squares += f64::from(sample).powi(2);
                m.peak = m.peak.max(sample.abs());
            }
        }
        read
    }
    pub fn report(&self) -> String {
        let all = *self.measurements.lock().unwrap();
        let rms = all.map(|m| {
            (m.squares / (m.settled.max(1) * u64::from(self.engine.channels())) as f64).sqrt()
        });
        let control_ok = self.control_error.lock().unwrap().is_none();
        let stopped = self.stopped.load(Ordering::Acquire);
        let dropped = self.ownership_dropped.load(Ordering::Acquire);
        let pass = control_ok
            && stopped
            && dropped
            && all.iter().all(|m| m.settled >= 1024 && m.invalid == 0)
            && all[0].peak > 0.015
            && all[0].peak <= 0.021
            && all[1].peak > 0.005
            && all[1].peak <= 0.011
            && rms[0] > rms[1] * 1.8
            && all[2].peak > 0.004
            && all[2].peak <= 0.0051
            && rms[1] > rms[2] * 1.8
            && all[3].peak == 0.0;
        let phases = all.iter().enumerate().map(|(i,m)| format!("{{\"phase\":{i},\"frames\":{},\"settled_frames\":{},\"rms\":{},\"peak\":{},\"invalid\":{}}}",m.frames,m.settled,rms[i],m.peak,m.invalid)).collect::<Vec<_>>().join(",");
        format!(
            "{{\"pass\":{pass},\"mode\":\"luau-oscillator\",\"rate\":{},\"channels\":{},\"block_frames\":{},\"clock\":{},\"control_ok\":{control_ok},\"scheduled_stop_completed\":{stopped},\"instance_dropped_with_live_handle\":{dropped},\"phases\":[{phases}]}}",
            self.engine.sample_rate(),
            self.engine.channels(),
            self.engine.buffer_frames(),
            self.engine.time_in_frames()
        )
    }
    pub fn finish_controls(&self) {
        assert_eq!(
            self.owner,
            thread::current().id(),
            "destroy controls on their owner thread"
        );
        CONTROLS.with(|all| {
            all.borrow_mut().remove(&(self as *const Self as usize));
        });
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
    match Probe::new(rate, channels, block) {
        Ok(probe) => Arc::into_raw(probe).cast_mut(),
        Err(error) => {
            eprintln!("Luau audio probe init: {error}");
            std::ptr::null_mut()
        }
    }
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
        let p = unsafe { Arc::from_raw(probe) };
        p.finish_controls();
        drop(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_luau_controls_drive_and_teardown_oscillator() {
        let p = Probe::new(48000, 2, 256).unwrap();
        for phase in 0..4 {
            p.set_phase(phase);
            for _ in 0..200 {
                assert_eq!(p.pull(&mut [0.0; 512]), if phase == 3 { 0 } else { 256 });
            }
        }
        let report = p.report();
        p.finish_controls();
        assert!(report.contains("\"pass\":true"), "{report}");
    }
}
