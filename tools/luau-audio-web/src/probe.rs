//! Browser qualification host. Compilation belongs to this test host only;
//! production processor modules remain precompiled ScriptAssets.
use luaur_rt::{Function, Table};
use nuxie_runtime::{NoopScriptHost, ScriptInstance, ScriptingVm};
use nuxie_scripting::vm::{AudioProcessorFormat, LuaScriptInstance, ScriptVm};
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct Probe {
    vm: Option<ScriptVm>,
    instance: Option<LuaScriptInstance>,
    // Intentionally retained after instance and VM teardown.
    table: Table,
}
thread_local! { static PROBE: RefCell<Option<Probe>> = const { RefCell::new(None) }; }
fn compile(source: &str) -> Result<Vec<u8>, String> {
    luaur_common::set_all_flags(true);
    let mut length = 0;
    let pointer = luaur_compiler::functions::luau_compile::luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut length,
    );
    if pointer.is_null() {
        return Err("qualification compilation failed".into());
    }
    let bytes = unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), length) }.to_vec();
    unsafe extern "C" {
        fn free(pointer: *mut std::ffi::c_void);
    }
    unsafe {
        free(pointer.cast());
    }
    if bytes.first() == Some(&0) {
        return Err(String::from_utf8_lossy(&bytes[1..]).into_owned());
    }
    Ok(bytes)
}
impl Probe {
    fn new() -> Result<Self, String> {
        let vm = ScriptVm::new();
        vm.install_rive_globals().map_err(|e| e.to_string())?;
        for (name, source) in [
            ("qualification", include_str!("qualification_processor.lua")),
            (
                "thrower",
                "return {process=function() error('expected processor failure') end}",
            ),
            (
                "runaway",
                "return {process=function() pcall(function() while true do end end) end}",
            ),
        ] {
            vm.register_audio_processor_module(name, &compile(source)?)
                .map_err(|e| e.to_string())?;
        }
        let main = compile(include_str!("qualification_main.lua"))?;
        // SAFETY: these bytes were just produced by the matching pinned Luau compiler.
        let function = unsafe { vm.lua().load_bytecode("qualification_main", &main) }
            .map_err(|e| e.to_string())?;
        let table: Table = function.call(()).map_err(|e| e.to_string())?;
        let mut instance = vm.script_instance_from_table(table.clone());
        if !instance
            .call_init(&mut NoopScriptHost)
            .map_err(|e| e.to_string())?
        {
            return Err("qualification instance refused init".into());
        }
        Ok(Self {
            vm: Some(vm),
            instance: Some(instance),
            table,
        })
    }
    fn report(&mut self) -> Result<String, String> {
        if let Some(vm) = self.vm.as_ref() {
            vm.poll_async_work().map_err(|e| e.to_string())?;
        }
        let f: Function = self.table.get("report").map_err(|e| e.to_string())?;
        let report: Table = f.call(self.table.clone()).map_err(|e| e.to_string())?;
        let get =
            |name: &str| -> Result<String, String> { report.get(name).map_err(|e| e.to_string()) };
        let failure = |name: &str| -> Result<Option<String>, String> {
            report.get(name).map_err(|e| e.to_string())
        };
        let meter: Option<Table> = report.get("meter").map_err(|e| e.to_string())?;
        let meter = match meter {
            Some(meter) => {
                let n = |name: &str| -> Result<f64, String> {
                    meter.get(name).map_err(|e| e.to_string())
                };
                serde_json::json!({"start":n("start")?,"frames":n("frames")?,"gain":n("gain")?,"blocks":n("blocks")?,"blobByte":n("blobByte")?})
            }
            None => serde_json::Value::Null,
        };
        let blocks: f64 = report.get("blocks").map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"state":get("state")?,"failure":failure("failure")?,"thrower":get("thrower")?,"throwerFailure":failure("throwerFailure")?,"runaway":get("runaway")?,"runawayFailure":failure("runawayFailure")?,"meter":meter,"blocks":blocks,"instanceAlive":self.instance.is_some(),"vmAlive":self.vm.is_some()}).to_string())
    }
    fn invoke(&self, name: &str, value: Option<f64>) -> Result<(), String> {
        let f: Function = self.table.get(name).map_err(|e| e.to_string())?;
        match value {
            Some(v) => f.call::<()>((self.table.clone(), v)),
            None => f.call::<()>(self.table.clone()),
        }
        .map_err(|e| e.to_string())
    }
}
fn with_probe<T>(f: impl FnOnce(&mut Probe) -> Result<T, String>) -> Result<T, String> {
    PROBE.with(|slot| {
        let mut probe = slot.borrow_mut();
        f(probe
            .as_mut()
            .ok_or("qualification probe has not started")?)
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_init() -> Result<(), String> {
    probe_dispose();
    ScriptVm::set_audio_render_offline(false, AudioProcessorFormat::default());
    let probe = Probe::new()?;
    PROBE.with(|slot| *slot.borrow_mut() = Some(probe));
    Ok(())
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_tick() -> Result<String, String> {
    with_probe(Probe::report)
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_gain(value: f64) -> Result<(), String> {
    with_probe(|p| p.invoke("gain", Some(value)))
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_region(value: f64) -> Result<(), String> {
    with_probe(|p| p.invoke("regionGain", Some(value)))
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_failures() -> Result<(), String> {
    with_probe(|p| {
        p.table
            .set("spawnFailures", true)
            .map_err(|e| e.to_string())?;
        p.instance
            .as_mut()
            .ok_or("instance already dropped")?
            .call_advance_truthy(0.0, &mut NoopScriptHost)
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_stop() -> Result<(), String> {
    with_probe(|p| p.invoke("stopTone", None))
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_drop_instance() -> Result<(), String> {
    with_probe(|p| {
        drop(p.instance.take());
        Ok(())
    })
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_drop_vm() -> Result<(), String> {
    with_probe(|p| {
        drop(p.vm.take());
        Ok(())
    })
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_retained_state() -> Result<String, String> {
    with_probe(|p| {
        let getter: Function = p.table.get("retainedState").map_err(|e| e.to_string())?;
        getter.call(p.table.clone()).map_err(|e| e.to_string())
    })
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_dispose() {
    PROBE.with(|slot| {
        slot.borrow_mut().take();
    });
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn nuxie_audio_notify(id: u32) {
    nuxie_scripting::vm::luau_audio_web::notify(id);
}

fn offline_reference() -> Result<Vec<f32>, String> {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ScriptVm::set_audio_render_offline(false, AudioProcessorFormat::default());
        }
    }
    ScriptVm::set_audio_render_offline(
        true,
        AudioProcessorFormat {
            sample_rate: 48000,
            channels: 2,
            max_frames: 128,
        },
    );
    let _reset = Reset;
    let probe = Probe::new()?;
    let handles = probe.vm.as_ref().unwrap().audio_processors();
    let handle = handles.first().ok_or("offline processor missing")?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while handle.state() == "starting" && std::time::Instant::now() < end {
            std::thread::yield_now();
        }
    }
    if handle.state() != "running" {
        return Err(format!(
            "offline processor {}: {:?}",
            handle.state(),
            handle.error()
        ));
    }
    let mut samples = vec![0.0; 256];
    handle.render_offline(&mut samples, 128, 0)?;
    if handle.state() != "running" {
        return Err(format!("offline render failed: {:?}", handle.error()));
    }
    ScriptVm::set_audio_offline_frame(128);
    let (frame, rate) = ScriptVm::audio_offline_clock().ok_or("offline clock unavailable")?;
    if frame != 128 || rate != 48000 {
        return Err("offline clock mismatch".into());
    }
    Ok(samples)
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn probe_offline_reference() -> Result<String, String> {
    Ok(
        serde_json::json!({"rate":48000,"channels":2,"frames":128,"samples":offline_reference()?})
            .to_string(),
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn native_offline_fixture_matches_exact_ramp() {
        let samples = super::offline_reference().unwrap();
        for i in 0..128 {
            let expected = 0.125 * ((i % 64) as f32) / 64.0;
            assert_eq!(samples[i * 2], expected);
            assert_eq!(samples[i * 2 + 1], -expected);
        }
    }
}
