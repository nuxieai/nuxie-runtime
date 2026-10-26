//! Nuxie-specific Luau controls for Rust audio processors. No Lua runs on the mixer thread.
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use luaur_rt::{Error, Lua, MultiValue, Result, Table, UserData, UserDataMethods, Value};
use nuxie_runtime::{
    AudioEngine, AudioProcessor, OscillatorConfig, ProcessorState, ProcessorUpdate,
};

const MAX_PROCESSORS_PER_INSTANCE: usize = 64;
const MAX_EXACT_FRAME: f64 = 9_007_199_254_740_991.0;

#[derive(Default)]
struct ProcessorOwnerInner {
    disposed: Cell<bool>,
    processors: RefCell<Vec<AudioProcessor>>,
}
impl ProcessorOwnerInner {
    fn dispose(&self) {
        self.disposed.set(true);
        for processor in self.processors.borrow_mut().drain(..) {
            processor.cancel();
        }
    }
}
impl Drop for ProcessorOwnerInner {
    fn drop(&mut self) {
        self.dispose();
    }
}

#[derive(Clone, Default)]
pub(super) struct ProcessorOwner(Rc<ProcessorOwnerInner>);
#[derive(Clone, Default)]
pub(super) struct WeakProcessorOwner(Weak<ProcessorOwnerInner>);
impl WeakProcessorOwner {
    pub(super) fn upgrade(&self) -> Option<ProcessorOwner> {
        self.0.upgrade().map(ProcessorOwner)
    }
}
impl ProcessorOwner {
    pub(super) fn downgrade(&self) -> WeakProcessorOwner {
        WeakProcessorOwner(Rc::downgrade(&self.0))
    }
    pub(super) fn dispose(&self) {
        self.0.dispose();
    }
    fn admit(&self) -> Result<()> {
        if self.0.disposed.get() {
            return Err(Error::runtime("audio processor owner has been disposed"));
        }
        let mut processors = self.0.processors.borrow_mut();
        processors.retain(|p| {
            !matches!(
                p.status().state,
                ProcessorState::Stopped | ProcessorState::Failed
            )
        });
        if processors.len() >= MAX_PROCESSORS_PER_INSTANCE {
            return Err(Error::runtime(
                "script instance audio processor limit reached (64)",
            ));
        }
        Ok(())
    }
}

#[derive(Default)]
struct VmOwners(RefCell<Vec<WeakProcessorOwner>>);
pub(super) fn shutdown(lua: &Lua) {
    if let Some(owners) = lua.app_data_ref::<VmOwners>() {
        for owner in owners
            .0
            .borrow_mut()
            .drain(..)
            .filter_map(|owner| owner.upgrade())
        {
            owner.dispose();
        }
    }
}

struct ScriptedProcessor(AudioProcessor);
impl UserData for ScriptedProcessor {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("start", |lua, this, args: MultiValue| {
            #[cfg(feature = "tools")]
            if !super::lua_audio::ScriptedAudioToolsState::allows_playback(lua) {
                return Err(Error::runtime(
                    "cannot start an audio processor while editor playback is disabled",
                ));
            }
            #[cfg(not(feature = "tools"))]
            let _ = lua;
            arity(&args, 0, 1)?;
            let options = optional_table(args.front(), &["atFrame"])?;
            this.0
                .start(frame(&options)?)
                .map_err(|e| Error::runtime(e.to_string()))
        });
        methods.add_method("set", |_, this, args: MultiValue| {
            arity(&args, 1, 2)?;
            let parameters = required_table(args.front(), &["frequency", "gain"])?;
            let options = optional_table(args.get(1), &["atFrame", "ramp"])?;
            let update = ProcessorUpdate {
                frequency: number(&Some(parameters.clone()), "frequency")?,
                gain: number(&Some(parameters), "gain")?,
            };
            if update.frequency.is_none() && update.gain.is_none() {
                return Err(Error::runtime("set requires frequency or gain"));
            }
            this.0
                .set(update, frame(&options)?, duration(&options, "ramp")?)
                .map_err(|e| Error::runtime(e.to_string()))
        });
        methods.add_method("stop", |_, this, args: MultiValue| {
            arity(&args, 0, 1)?;
            let options = optional_table(args.front(), &["atFrame", "fade"])?;
            this.0
                .stop(frame(&options)?, duration(&options, "fade")?)
                .map_err(|e| Error::runtime(e.to_string()))
        });
        methods.add_method("status", |lua, this, args: MultiValue| {
            arity(&args, 0, 0)?;
            let status = this.0.status();
            let result = lua.create_table();
            result.set("state", status.state.as_str())?;
            if let Some(error) = status.error {
                result.set("error", error)?;
            }
            Ok(result)
        });
        methods.add_method("meter", |lua, this, args: MultiValue| {
            arity(&args, 0, 0)?;
            let meter = this.0.meter();
            let result = lua.create_table();
            result.set("peak", meter.peak)?;
            result.set("rms", meter.rms)?;
            Ok(result)
        });
    }
}

pub(super) fn install(lua: &Lua, audio: &Table) -> Result<()> {
    if lua.app_data_ref::<VmOwners>().is_none() {
        lua.set_app_data(VmOwners::default());
    }
    audio.set(
        "createProcessor",
        lua.create_function(|lua, args: MultiValue| {
            arity(&args, 1, 2)?;
            match args.front() {
                Some(Value::String(name)) if name.as_bytes() == b"oscillator" => {}
                _ => {
                    return Err(Error::runtime(
                        "Audio.createProcessor supports only 'oscillator'",
                    ));
                }
            }
            let parameters = optional_table(args.get(1), &["waveform", "frequency", "gain"])?;
            if let Some(parameters) = &parameters {
                match parameters.raw_get::<Value>("waveform")? {
                    Value::Nil => {}
                    Value::String(name) if name.as_bytes() == b"sine" => {}
                    _ => return Err(Error::runtime("oscillator waveform must be 'sine'")),
                }
            }
            let owner = super::view_model::ScriptViewModelFrameContext::for_lua(lua)
                .audio_processor_owner()
                .ok_or_else(|| {
                    Error::runtime(
                        "create audio processors in a script instance init or owned callback",
                    )
                })?;
            owner.admit()?;
            let config = OscillatorConfig {
                frequency: number(&parameters, "frequency")?.unwrap_or(440.0),
                gain: number(&parameters, "gain")?.unwrap_or(0.0),
            };
            let processor = AudioEngine::runtime_engine()
                .create_oscillator(config)
                .map_err(|e| Error::runtime(e.to_string()))?;
            let userdata = lua.create_userdata(ScriptedProcessor(processor.clone()))?;
            owner.0.processors.borrow_mut().push(processor);
            // Weak VM tracking cancels on explicit ScriptVm teardown even if external
            // Lua tables or instance handles keep the underlying Lua state alive.
            let owners = lua
                .app_data_ref::<VmOwners>()
                .expect("installed processor registry");
            let mut tracked = owners.0.borrow_mut();
            tracked.retain(|owner| owner.0.strong_count() != 0);
            if !tracked
                .iter()
                .any(|tracked| tracked.0.ptr_eq(&owner.downgrade().0))
            {
                tracked.push(owner.downgrade());
            }
            Ok(userdata)
        })?,
    )
}

fn arity(args: &MultiValue, min: usize, max: usize) -> Result<()> {
    if args.len() < min || args.len() > max {
        return Err(Error::runtime(format!(
            "expected {min}..={max} arguments, got {}",
            args.len()
        )));
    }
    Ok(())
}
fn optional_table(value: Option<&Value>, fields: &[&str]) -> Result<Option<Table>> {
    match value {
        None | Some(Value::Nil) => Ok(None),
        _ => required_table(value, fields).map(Some),
    }
}
fn required_table(value: Option<&Value>, fields: &[&str]) -> Result<Table> {
    let Some(Value::Table(table)) = value else {
        return Err(Error::runtime("expected parameter/options table"));
    };
    for pair in table.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let Value::String(key) = key else {
            return Err(Error::runtime("audio parameter keys must be strings"));
        };
        if !fields
            .iter()
            .any(|field| key.as_bytes() == field.as_bytes())
        {
            return Err(Error::runtime("unknown audio parameter or option"));
        }
    }
    Ok(table.clone())
}
fn number(table: &Option<Table>, key: &str) -> Result<Option<f64>> {
    let Some(table) = table else {
        return Ok(None);
    };
    let value = match table.raw_get::<Value>(key)? {
        Value::Nil => return Ok(None),
        Value::Integer(value) => value as f64,
        Value::Number(value) => value,
        _ => return Err(Error::runtime(format!("{key} must be a number"))),
    };
    if !value.is_finite() {
        return Err(Error::runtime(format!("{key} must be finite")));
    }
    Ok(Some(value))
}
fn duration(table: &Option<Table>, key: &str) -> Result<f64> {
    let value = number(table, key)?.unwrap_or(0.0);
    if value < 0.0 {
        return Err(Error::runtime(format!("{key} must be nonnegative seconds")));
    }
    Ok(value)
}
fn frame(table: &Option<Table>) -> Result<Option<u64>> {
    let Some(value) = number(table, "atFrame")? else {
        return Ok(None);
    };
    if !(0.0..=MAX_EXACT_FRAME).contains(&value) || value.fract() != 0.0 {
        return Err(Error::runtime(
            "atFrame must be a nonnegative exact integer below 2^53",
        ));
    }
    Ok(Some(value as u64))
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::super::lua_audio::AUDIO_TEST_LOCK;
    use super::*;
    use crate::vm::{LuaScriptInstance, ScriptVm};

    fn instance(vm: &ScriptVm, source: &str) -> LuaScriptInstance {
        vm.install_rive_globals().unwrap();
        #[cfg(feature = "tools")]
        vm.set_is_playing(true);
        let table: Table = vm.lua().load(source).eval().unwrap();
        let mut instance = vm.script_instance_from_table(table);
        assert!(instance.call_init_with_optional_factory(None).unwrap());
        instance
    }

    #[test]
    fn luau_audio_processor_controls_and_instance_teardown() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let owned = instance(
            &vm,
            r#"
            return { init = function(self)
                retained = Audio.createProcessor("oscillator", {frequency=12000, gain=0.2})
                assert(retained:status().state == "prepared")
                retained:start({atFrame=0})
                retained:set({frequency=6000, gain=0.1}, {atFrame=128, ramp=0.002})
                return true
            end }
        "#,
        );
        let mut frames = vec![0.0; 512];
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.05));
        vm.lua()
            .load("assert(retained:status().state == 'running'); assert(retained:meter().peak > 0)")
            .exec()
            .unwrap();
        drop(owned);
        vm.lua().load("assert(retained:status().state == 'stopped'); retained:stop(); assert(not pcall(function() retained:start() end))").exec().unwrap();
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn luau_audio_processor_rejects_invalid_inputs_and_queue_overflow() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let _engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let _owned = instance(
            &vm,
            r#"
            return { init = function(self)
                local function rejects(f) assert(not pcall(f)) end
                rejects(function() Audio.createProcessor("filter", {}) end)
                rejects(function() Audio.createProcessor("oscillator", {waveform="square"}) end)
                rejects(function() Audio.createProcessor("oscillator", {unknown=1}) end)
                rejects(function() Audio.createProcessor("oscillator", {gain=0/0}) end)
                rejects(function() Audio.createProcessor("oscillator", {frequency=24000}) end)
                rejects(function() Audio.createProcessor("oscillator", {gain="0.1"}) end)
                local p = Audio.createProcessor("oscillator")
                rejects(function() p:start({atFrame=0.5}) end)
                rejects(function() p:start({atFrame=-1}) end)
                rejects(function() p:start({atFrame=9007199254740992}) end)
                rejects(function() p:start({delay=1}) end)
                rejects(function() p:set({gain=0.1}, {ramp=-0.01}) end)
                rejects(function() p:set({gain=0.1, typo=1}) end)
                rejects(function() p:set({}) end)
                rejects(function() p:stop({fade=math.huge}) end)
                p:start()
                local full = false
                for i=1,4096 do
                    if not pcall(function() p:set({gain=0.1}, {atFrame=i*100}) end) then full=true; break end
                end
                assert(full, "bounded queue must reject overflow")
                return true
            end }
        "#,
        );
        assert!(
            vm.lua()
                .load("Audio.createProcessor('oscillator')")
                .exec()
                .is_err()
        );
    }

    #[test]
    fn luau_audio_processor_failed_init_cancels_retained_handle() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        #[cfg(feature = "tools")]
        vm.set_is_playing(true);
        let table: Table = vm
            .lua()
            .load(
                r#"return {init=function(self)
            retained=Audio.createProcessor('oscillator',{gain=0.1}); retained:start(); return false
        end}"#,
            )
            .eval()
            .unwrap();
        let mut owned = vm.script_instance_from_table(table);
        assert!(!owned.call_init_with_optional_factory(None).unwrap());
        vm.lua()
            .load("assert(retained:status().state == 'stopped')")
            .exec()
            .unwrap();
        let mut frames = vec![0.0; 256];
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn luau_audio_processor_vm_teardown_cancels_surviving_instance() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.createProcessor('oscillator',{gain=0.1}); retained:start(); return true end}",
        );
        let retained: luaur_rt::AnyUserData = vm.lua().globals().get("retained").unwrap();
        drop(vm);
        assert_eq!(
            retained
                .borrow::<ScriptedProcessor>()
                .unwrap()
                .0
                .status()
                .state,
            ProcessorState::Stopped
        );
        let mut frames = vec![0.0; 256];
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
        drop(owned);
    }

    #[cfg(feature = "tools")]
    #[test]
    fn luau_audio_processor_start_obeys_editor_playback_state() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let _engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let _owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.createProcessor('oscillator'); return true end}",
        );
        vm.set_is_playing(false);
        assert!(vm.lua().load("retained:start()").exec().is_err());
        vm.set_is_playing(true);
        vm.lua().load("retained:start()").exec().unwrap();
    }
}
