//! Main-script controls for isolated, Luau-authored audio processors.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use super::luau_audio_processor_engine::{
    ProcessorHandle, SharedRing, SharedSnapshot, SharedValue,
};
use super::luau_audio_shared::{RegionGuard, SharedBlob, SharedBudget, SharedRegion};
use luaur_rt::{
    Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods, Value,
};
#[cfg(not(target_arch = "wasm32"))]
use nuxie_runtime::AudioEngine;

#[derive(Default)]
struct ProcessorOwnerInner {
    disposed: Cell<bool>,
    processors: RefCell<Vec<Rc<OwnedProcessor>>>,
}
impl ProcessorOwnerInner {
    fn dispose(&self) {
        self.disposed.set(true);
        for processor in self.processors.borrow_mut().drain(..) {
            processor.handle.stop();
            processor.handle.abandon_notifications();
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
        self.0
            .processors
            .borrow_mut()
            .retain(|processor| !processor.can_release());
        Ok(())
    }
}

struct OwnedProcessor {
    handle: ProcessorHandle,
    observed_state: Cell<&'static str>,
    failure_reported: Cell<bool>,
}
impl OwnedProcessor {
    fn pending(&self) -> bool {
        let state = self.handle.state();
        state == "starting"
            || state != self.observed_state.get()
            || self.handle.has_pending_work()
            || self.handle.has_logs()
            || (state == "failed" && !self.failure_reported.get())
    }
    fn can_release(&self) -> bool {
        self.handle.is_unreferenced_terminal() && !self.pending()
    }
}
#[derive(Default)]
struct VmOwners(RefCell<Vec<WeakProcessorOwner>>);
fn live_owners(lua: &Lua) -> Vec<ProcessorOwner> {
    lua.app_data_ref::<VmOwners>()
        .map(|owners| {
            owners
                .0
                .borrow()
                .iter()
                .filter_map(WeakProcessorOwner::upgrade)
                .collect()
        })
        .unwrap_or_default()
}
/// Pending processor initialization and undelivered events participate in the
/// runtime's ordinary async polling, including when no script callback runs.
pub(super) fn processors(lua: &Lua) -> Vec<ProcessorHandle> {
    live_owners(lua)
        .iter()
        .flat_map(|owner| {
            owner
                .0
                .processors
                .borrow()
                .iter()
                .map(|p| p.handle.clone())
                .collect::<Vec<_>>()
        })
        .collect()
}
pub(super) fn has_pending_work(lua: &Lua) -> bool {
    live_owners(lua).iter().any(|owner| {
        owner
            .0
            .processors
            .borrow()
            .iter()
            .any(|processor| processor.pending())
    })
}
/// Deliver processor events on the ordinary script thread. Release registry
/// borrows before invoking an application-provided logging sink.
pub(super) fn poll(lua: &Lua) -> bool {
    let owners = live_owners(lua);
    let processors: Vec<Rc<OwnedProcessor>> = owners
        .iter()
        .flat_map(|owner| owner.0.processors.borrow().clone())
        .collect();
    let logging = lua
        .app_data_ref::<super::logging_scripting_context::LoggingScriptingContext>()
        .map(|logging| logging.clone());
    let mut changed = false;
    for processor in processors {
        changed |= processor.handle.poll_notification();
        let state = processor.handle.state();
        changed |= processor.observed_state.replace(state) != state;
        if state == "failed" {
            processor.handle.stop();
        }
        let failure = if state == "failed" && !processor.failure_reported.replace(true) {
            changed = true;
            processor.handle.take_failure()
        } else {
            None
        };
        let lines = processor.handle.take_logs();
        changed |= !lines.is_empty();
        // Mark/drain before delivering so a reentrant sink cannot duplicate an
        // event. An external handle keeps terminal processors tracked.
        if let Some(logging) = &logging {
            for line in lines {
                logging.begin_line();
                logging.append(line.as_bytes());
                logging.end_line();
            }
            if let Some(error) = failure {
                logging.print_error(Some(error.as_bytes()));
            }
        }
    }
    for owner in owners {
        owner
            .0
            .processors
            .borrow_mut()
            .retain(|processor| !processor.can_release());
    }
    changed
}

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
    if let Some(budget) = lua.app_data_ref::<ProcessorSharedBudget>() {
        budget.0.close();
    }
}

struct ProcessorSharedBudget(Arc<SharedBudget>);
struct ProcessorMemoryLimit(usize);
struct ProcessorHardBlocks(u32);

pub(super) fn set_hard_blocks(lua: &Lua, blocks: u32) {
    lua.set_app_data(ProcessorHardBlocks(blocks));
}

pub(super) fn set_memory_limit(lua: &Lua, bytes: usize) {
    lua.set_app_data(ProcessorMemoryLimit(bytes));
}

#[derive(Default)]
struct ProcessorModules(RefCell<BTreeMap<String, Arc<[u8]>>>);

/// Register trusted, validated bytecode; validation is performed by ScriptVm.
/// The bytes, never main-VM Lua handles, are copied into each isolated processor.
pub(super) fn register_module(lua: &Lua, name: &str, bytecode: Arc<[u8]>) -> Result<()> {
    if lua.app_data_ref::<ProcessorModules>().is_none() {
        lua.set_app_data(ProcessorModules::default());
    }
    lua.app_data_ref::<ProcessorModules>()
        .expect("installed processor module registry")
        .0
        .borrow_mut()
        .insert(name.to_owned(), bytecode);
    Ok(())
}

struct ScriptedProcessor(ProcessorHandle);
impl UserData for ScriptedProcessor {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("state", |_, this| Ok(this.0.state()));
        fields.add_field_method_get("failed", |_, this| Ok(this.0.error()));
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("stop", |_, this, args: MultiValue| {
            arity(&args, 0, 0)?;
            this.0.stop();
            Ok(())
        });
    }
}

struct ScriptedRing(Arc<SharedRing>);
impl UserData for ScriptedRing {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("send", |_, this, args: MultiValue| {
            arity(&args, 1, 1)?;
            this.0
                .send(to_shared(
                    args.front().expect("one argument"),
                    &mut Vec::new(),
                )?)
                .map_err(Error::runtime)
        });
        methods.add_method("receive", |lua, this, args: MultiValue| {
            arity(&args, 0, 0)?;
            this.0
                .try_receive()
                .map_err(Error::runtime)?
                .map_or(Ok(Value::Nil), |value| from_shared(lua, value))
        });
        methods.add_method("receiveInto", |lua, this, target: Table| {
            let Some(value) = this.0.try_receive().map_err(Error::runtime)? else {
                return Ok(false);
            };
            fill_table(lua, &target, value)?;
            Ok(true)
        });
    }
}
struct ScriptedSnapshot(Arc<SharedSnapshot>);
impl UserData for ScriptedSnapshot {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("publish", |_, this, args: MultiValue| {
            arity(&args, 1, 1)?;
            this.0
                .publish(to_shared(
                    args.front().expect("one argument"),
                    &mut Vec::new(),
                )?)
                .map_err(Error::runtime)
        });
        methods.add_method("latest", |lua, this, args: MultiValue| {
            arity(&args, 0, 0)?;
            let Some((sequence, value)) = this.0.try_latest().map_err(Error::runtime)? else {
                return Ok(Value::Nil);
            };
            let table = lua.create_table();
            table.raw_set("sequence", sequence as f64)?;
            table.raw_set("value", from_shared(lua, value)?)?;
            Ok(Value::Table(table))
        });
    }
}
fn fill_table(lua: &Lua, target: &Table, value: SharedValue) -> Result<()> {
    let Value::Table(source) = from_shared(lua, value)? else {
        return Err(Error::runtime(
            "receiveInto requires a record or array message",
        ));
    };
    let keys: Vec<Value> = target
        .pairs::<Value, Value>()
        .map(|pair| pair.map(|(key, _)| key))
        .collect::<Result<_>>()?;
    for key in keys {
        target.raw_set(key, Value::Nil)?;
    }
    for pair in source.pairs::<Value, Value>() {
        let (key, value) = pair?;
        target.raw_set(key, value)?;
    }
    Ok(())
}
fn byte_capacity(args: &MultiValue, allow_empty: bool) -> Result<usize> {
    arity(args, 1, 1)?;
    let capacity = match args.front() {
        Some(Value::Integer(value)) => *value as f64,
        Some(Value::Number(value)) => *value,
        _ => return Err(Error::runtime("shared byte capacity must be an integer")),
    };
    if !capacity.is_finite()
        || capacity < 0.0
        || (!allow_empty && capacity == 0.0)
        || capacity.fract() != 0.0
        || capacity > u32::MAX as f64
    {
        return Err(Error::runtime("invalid shared byte capacity"));
    }
    Ok(capacity as usize)
}

pub(super) fn install(lua: &Lua, audio: &Table) -> Result<()> {
    if lua.app_data_ref::<ProcessorSharedBudget>().is_none() {
        lua.set_app_data(ProcessorSharedBudget(Arc::new(SharedBudget::default())));
    }
    if lua.app_data_ref::<VmOwners>().is_none() {
        lua.set_app_data(VmOwners::default());
    }
    if lua.app_data_ref::<ProcessorModules>().is_none() {
        lua.set_app_data(ProcessorModules::default());
    }
    let blob = lua.create_table();
    blob.raw_set(
        "from",
        lua.create_function(|lua, value: Value| {
            let bytes = match value {
                Value::Buffer(value) => value.to_vec(),
                Value::String(value) => value.as_bytes().to_vec(),
                _ => return Err(Error::runtime("Blob.from expects a buffer or string")),
            };
            let value = SharedBlob::with_budget(
                &bytes,
                lua.app_data_ref::<ProcessorSharedBudget>()
                    .expect("installed shared budget")
                    .0
                    .clone(),
            )
            .map_err(Error::runtime)?;
            lua.create_userdata(ScriptedBlob(Arc::new(value)))
        })?,
    )?;
    audio.raw_set("Blob", blob)?;
    let region = lua.create_table();
    region.raw_set(
        "create",
        lua.create_function(|lua, args: MultiValue| {
            let value = SharedRegion::with_budget(
                byte_capacity(&args, false)?,
                lua.app_data_ref::<ProcessorSharedBudget>()
                    .expect("installed shared budget")
                    .0
                    .clone(),
            )
            .map_err(Error::runtime)?;
            lua.create_userdata(ScriptedRegion(Arc::new(value)))
        })?,
    )?;
    audio.raw_set("Region", region)?;
    let ring = lua.create_table();
    ring.raw_set(
        "create",
        lua.create_function(|lua, args: MultiValue| {
            let ring = SharedRing::with_budget(
                byte_capacity(&args, false)?,
                lua.app_data_ref::<ProcessorSharedBudget>()
                    .expect("installed shared budget")
                    .0
                    .clone(),
            )
            .map_err(Error::runtime)?;
            lua.create_userdata(ScriptedRing(Arc::new(ring)))
        })?,
    )?;
    audio.raw_set("Ring", ring)?;
    let snapshot = lua.create_table();
    snapshot.raw_set(
        "create",
        lua.create_function(|lua, args: MultiValue| {
            let snapshot = SharedSnapshot::with_budget(
                byte_capacity(&args, true)?,
                lua.app_data_ref::<ProcessorSharedBudget>()
                    .expect("installed shared budget")
                    .0
                    .clone(),
            )
            .map_err(Error::runtime)?;
            lua.create_userdata(ScriptedSnapshot(Arc::new(snapshot)))
        })?,
    )?;
    audio.raw_set("Snapshot", snapshot)?;
    audio.set(
        "start",
        lua.create_function(|lua, args: MultiValue| {
            arity(&args, 1, 2)?;
            #[cfg(feature = "tools")]
            if !super::lua_audio::ScriptedAudioToolsState::allows_playback(lua) {
                return Err(Error::runtime(
                    "cannot start an audio processor while editor playback is disabled",
                ));
            }
            let name = match args.front() {
                Some(Value::String(value)) => String::from_utf8(value.as_bytes())
                    .map_err(|_| Error::runtime("audio processor module name must be UTF-8"))?
                    .to_owned(),
                _ => {
                    return Err(Error::runtime(
                        "Audio.start expects a registered processor module name",
                    ));
                }
            };
            let input = to_shared(args.get(1).unwrap_or(&Value::Nil), &mut Vec::new())?;
            let hard_blocks = lua
                .app_data_ref::<ProcessorHardBlocks>()
                .map_or(2, |policy| policy.0);
            let owner = super::view_model::ScriptViewModelFrameContext::for_lua(lua)
                .audio_processor_owner()
                .ok_or_else(|| {
                    Error::runtime(
                        "start audio processors in a script instance init or owned callback",
                    )
                })?;
            owner.admit()?;
            let modules = lua
                .app_data_ref::<ProcessorModules>()
                .expect("installed processor module registry")
                .0
                .borrow()
                .clone();
            if !modules.contains_key(&name) {
                return Err(Error::runtime(format!(
                    "audio processor module '{name}' is not registered"
                )));
            }
            #[cfg(target_arch = "wasm32")]
            let engine = None;
            #[cfg(not(target_arch = "wasm32"))]
            let engine = if super::luau_audio_processor_engine::render_offline() {
                None
            } else {
                Some(AudioEngine::runtime_engine())
            };
            let processor = ProcessorHandle::start(
                engine,
                name,
                modules,
                input,
                hard_blocks,
                lua.app_data_ref::<ProcessorMemoryLimit>()
                    .map_or(16 * 1024 * 1024, |limit| limit.0),
            )
            .map_err(Error::runtime)?;
            // Retain ownership before allocating the public handle: any subsequent
            // Lua allocation failure is covered by script-instance cleanup.
            owner
                .0
                .processors
                .borrow_mut()
                .push(Rc::new(OwnedProcessor {
                    handle: processor.clone(),
                    observed_state: Cell::new("starting"),
                    failure_reported: Cell::new(false),
                }));
            let owners = lua
                .app_data_ref::<VmOwners>()
                .expect("installed processor owner registry");
            let mut tracked = owners.0.borrow_mut();
            tracked.retain(|owner| owner.0.strong_count() != 0);
            if !tracked
                .iter()
                .any(|tracked| tracked.0.ptr_eq(&owner.downgrade().0))
            {
                tracked.push(owner.downgrade());
            }
            lua.create_userdata(ScriptedProcessor(processor))
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
fn to_shared(value: &Value, ancestors: &mut Vec<*const std::ffi::c_void>) -> Result<SharedValue> {
    Ok(match value {
        Value::Nil => SharedValue::Nil,
        Value::Boolean(value) => SharedValue::Boolean(*value),
        Value::Integer(value) => SharedValue::Number(*value as f64),
        Value::Number(value) => SharedValue::Number(*value),
        Value::String(value) => SharedValue::Bytes(value.as_bytes().to_vec()),
        Value::UserData(value) if value.is::<ScriptedBlob>() => {
            SharedValue::Blob(value.borrow::<ScriptedBlob>()?.0.clone())
        }
        Value::UserData(value) if value.is::<ScriptedRegion>() => {
            SharedValue::Region(value.borrow::<ScriptedRegion>()?.0.clone())
        }
        Value::UserData(userdata) if userdata.is::<ScriptedRing>() => {
            SharedValue::Ring(userdata.borrow::<ScriptedRing>()?.0.clone())
        }
        Value::UserData(userdata) if userdata.is::<ScriptedSnapshot>() => {
            SharedValue::Snapshot(userdata.borrow::<ScriptedSnapshot>()?.0.clone())
        }
        Value::Table(table) => {
            let pointer = table.to_pointer();
            if ancestors.contains(&pointer) {
                return Err(Error::runtime(
                    "audio messages cannot contain cyclic tables",
                ));
            }
            // Protect the host stack while traversing dynamic Luau data. Unlike
            // the upstream generated schema, a Lua table has unbounded nesting.
            if ancestors.len() >= 64 {
                return Err(Error::runtime("audio message nesting exceeds 64 tables"));
            }
            ancestors.push(pointer);
            let mut object = BTreeMap::new();
            let mut array = BTreeMap::new();
            for pair in table.pairs::<Value, Value>() {
                let (key, value) = pair?;
                let value = to_shared(&value, ancestors)?;
                match key {
                    Value::String(key) => {
                        let key = String::from_utf8(key.as_bytes())
                            .map_err(|_| Error::runtime("audio record keys must be UTF-8"))?;
                        object.insert(key.to_owned(), value);
                    }
                    Value::Integer(key) if key > 0 => {
                        array.insert(key as usize, value);
                    }
                    Value::Number(key)
                        if key > 0.0 && key.fract() == 0.0 && key <= usize::MAX as f64 =>
                    {
                        array.insert(key as usize, value);
                    }
                    _ => {
                        return Err(Error::runtime(
                            "audio tables must be string-keyed records or dense arrays",
                        ));
                    }
                }
            }
            ancestors.pop();
            if array.is_empty() {
                SharedValue::Object(object)
            } else {
                if !object.is_empty() || array.keys().copied().ne(1..=array.len()) {
                    return Err(Error::runtime(
                        "audio tables must be string-keyed records or dense arrays",
                    ));
                }
                SharedValue::Array(array.into_values().collect())
            }
        }
        _ => {
            return Err(Error::runtime(
                "audio data supports nil, booleans, numbers, strings, records and arrays",
            ));
        }
    })
}
fn from_shared(lua: &Lua, value: SharedValue) -> Result<Value> {
    Ok(match value {
        SharedValue::Blob(value) => Value::UserData(lua.create_userdata(ScriptedBlob(value))?),
        SharedValue::Region(value) => Value::UserData(lua.create_userdata(ScriptedRegion(value))?),
        SharedValue::Ring(value) => Value::UserData(lua.create_userdata(ScriptedRing(value))?),
        SharedValue::Snapshot(value) => {
            Value::UserData(lua.create_userdata(ScriptedSnapshot(value))?)
        }
        SharedValue::Nil => Value::Nil,
        SharedValue::Boolean(value) => Value::Boolean(value),
        SharedValue::Number(value) => Value::Number(value),
        SharedValue::Bytes(value) => Value::String(lua.create_string(value)),
        SharedValue::Array(values) => {
            let table = lua.create_table();
            for (index, value) in values.into_iter().enumerate() {
                table.raw_set(index + 1, from_shared(lua, value)?)?;
            }
            Value::Table(table)
        }
        SharedValue::Object(values) => {
            let table = lua.create_table();
            for (key, value) in values {
                table.raw_set(key, from_shared(lua, value)?)?;
            }
            Value::Table(table)
        }
    })
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::super::lua_audio::AUDIO_TEST_LOCK;
    use super::*;
    use crate::vm::{LuaScriptInstance, ScriptVm};
    use std::time::{Duration, Instant};

    #[test]
    fn shared_blob_region_main_api_and_scoped_view() {
        let lua = Lua::new();
        let audio = lua.create_table();
        install(&lua, &audio).unwrap();
        lua.globals().set("Audio", audio).unwrap();
        lua.load(
            r#"
            local source = buffer.fromstring("abc")
            local blob = Audio.Blob.from(source)
            buffer.writeu8(source,0,90)
            assert(buffer.tostring(blob:toBuffer()) == "abc")
            local out=buffer.fromstring("zzzz")
            assert(blob:read(1,out)==2 and buffer.tostring(out)=="bczz")
            local region=Audio.Region.create(9)
            assert(region:atomicStore(0,4294967295)==0)
            assert(region:atomicAdd(0,2)==4294967295)
            assert(region:atomicCas(0,1,9)==1 and region:atomicLoad(0)==9)
            local saved
            assert(region:tryExclusive(function(view)
                saved=view
                view:setU32(4,123)
                assert(view:getU32(4)==123)
                assert(not pcall(function() region:tryExclusive(function()end,0) end))
            end,0))
            assert(not pcall(function() saved:getU32(4) end))
            assert(not pcall(function() region:tryExclusive(function()error("trap")end,0) end))
            assert(region.poisoned and region:tryExclusive(function()end,0))
            local ring=Audio.Ring.create(256)
            assert(ring:send({blob=blob,region=region}))
            local value=ring:receive()
            assert(value.region:atomicLoad(0)==9)
            assert(buffer.tostring(value.blob:toBuffer())=="abc")
        "#,
        )
        .exec()
        .unwrap();
    }

    fn compile(source: &str) -> Vec<u8> {
        use luaur_compiler::functions::luau_compile::luau_compile;
        luaur_common::set_all_flags(true);
        let mut size = 0;
        let bytes = luau_compile(
            source.as_ptr().cast(),
            source.len(),
            std::ptr::null_mut(),
            &mut size,
        );
        assert!(!bytes.is_null());
        let result = unsafe { std::slice::from_raw_parts(bytes.cast(), size).to_vec() };
        unsafe extern "C" {
            fn free(pointer: *mut std::ffi::c_void);
        }
        unsafe {
            free(bytes.cast());
        }
        result
    }
    fn register(vm: &ScriptVm, name: &str, source: &str) {
        vm.register_audio_processor_module(name, &compile(source))
            .unwrap();
    }
    fn instance(vm: &ScriptVm, source: &str) -> LuaScriptInstance {
        vm.install_rive_globals().unwrap();
        #[cfg(feature = "tools")]
        vm.set_is_playing(true);
        let table: Table = vm.lua().load(source).eval().unwrap();
        let mut owned = vm.script_instance_from_table(table);
        assert!(owned.call_init_with_optional_factory(None).unwrap());
        owned
    }
    fn wait_running(vm: &ScriptVm) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let state: String = vm.lua().load("return retained.state").eval().unwrap();
            if state == "running" {
                return;
            }
            assert_eq!(
                state,
                "starting",
                "processor failed: {:?}",
                vm.lua()
                    .load("return retained.failed")
                    .eval::<Option<String>>()
                    .unwrap()
            );
            assert!(
                Instant::now() < deadline,
                "processor initialization did not finish"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    const PROCESSOR: &str = r#"
        return {
            init = function(self, rate, frames, channels, input)
                assert(mainOnly == nil, 'main VM leaked into audio VM')
                self.channels = channels
                self.gain = input.gain
                self.messages = input.messages
                self.meter = input.meter
                self.next = {}
            end,
            process = function(self, out, frames, startFrame)
                while self.messages:receiveInto(self.next) do
                    self.gain = self.next.gain
                end
                for i = 0, frames * self.channels - 1 do
                    buffer.writef32(out, i * 4, self.gain)
                end
                self.meter:publish({gain=self.gain, frame=startFrame})
            end,
        }
    "#;

    #[test]
    fn authored_audio_uses_isolated_module_and_shared_channels() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        register(&vm, "tone", PROCESSOR);
        let owned = instance(
            &vm,
            r#"
            return {init=function(self)
                mainOnly = 123
                messages = Audio.Ring.create(1024)
                meter = Audio.Snapshot.create(1024)
                assert(meter:latest() == nil)
                retained = Audio.start('tone', {gain=0.25, messages=messages, meter=meter})
                return true
            end}
        "#,
        );
        wait_running(&vm);
        let mut frames = vec![0.0; engine.buffer_frames() as usize * 2];
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|v| *v == 0.25));
        vm.lua()
            .load("assert(meter:latest().value.gain == 0.25); assert(messages:send({gain=0.5}))")
            .exec()
            .unwrap();
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|v| *v == 0.5));
        vm.lua()
            .load("assert(meter:latest().value.gain == 0.5)")
            .exec()
            .unwrap();
        drop(owned);
        vm.lua()
            .load("assert(retained.state == 'stopped'); retained:stop()")
            .exec()
            .unwrap();
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn authored_audio_uses_existing_script_asset_registration() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let mut payload = vec![0]; // Existing unsigned ScriptAsset envelope.
        payload.extend(compile(PROCESSOR));
        vm.register_module("asset-tone", &payload).unwrap();
        let _owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.start('asset-tone',{gain=0.125,messages=Audio.Ring.create(1024),meter=Audio.Snapshot.create(1024)}); return true end}",
        );
        wait_running(&vm);
        let mut frames = vec![0.0; engine.buffer_frames() as usize * 2];
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|value| *value == 0.125));
    }

    #[test]
    fn authored_audio_retention_does_not_restrict_ordinary_module_keys() {
        let vm = ScriptVm::new();
        let bytes = compile("return {}");
        let mut payload = vec![0];
        payload.extend_from_slice(&bytes);
        vm.register_module("", &payload).unwrap();
        // The retention helper accepts exact ordinary registry identities; a
        // processor load separately handles what its bytecode loader accepts.
        register_module(vm.lua(), "embedded\0key", Arc::from(bytes)).unwrap();
        let modules = vm.lua().app_data_ref::<ProcessorModules>().unwrap();
        assert!(modules.0.borrow().contains_key(""));
        assert!(modules.0.borrow().contains_key("embedded\0key"));
    }

    #[test]
    fn authored_audio_require_resolves_registered_bytes_in_processor_vm() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        register(&vm, "gain", "return {value=0.375}");
        register(
            &vm,
            "dependent",
            r#"
            local gain = require('gain')
            return {
                init=function(self,rate,frames,channels,input) self.channels=channels end,
                process=function(self,out,frames,startFrame)
                    for i=0,frames*self.channels-1 do buffer.writef32(out,i*4,gain.value) end
                end,
            }
        "#,
        );
        let _owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.start('dependent'); return true end}",
        );
        wait_running(&vm);
        let mut frames = vec![0.0; engine.buffer_frames() as usize * 2];
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|value| *value == 0.375));
    }

    #[test]
    fn authored_audio_logs_are_delivered_once_on_the_main_thread() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let main_thread = std::thread::current().id();
        let lines = Rc::new(RefCell::new(Vec::new()));
        let captured = lines.clone();
        vm.set_log_sink(move |_, bytes| {
            assert_eq!(std::thread::current().id(), main_thread);
            captured
                .borrow_mut()
                .push(String::from_utf8_lossy(bytes).into_owned());
        });
        register(
            &vm,
            "logger",
            r#"return {
            init=function(self,rate,frames,channels,input) print('processor init'); end,
            process=function(self,out,frames,startFrame)
                if not self.printed then print('processor block'); self.printed=true end
            end,
        }"#,
        );
        let _owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.start('logger'); return true end}",
        );
        wait_running(&vm);
        let mut frames = vec![0.0; engine.buffer_frames() as usize * 2];
        engine.read_audio_frames(&mut frames);
        assert!(
            lines.borrow().is_empty(),
            "audio thread must not invoke the application log sink"
        );
        poll(vm.lua());
        let observed = lines.borrow().clone();
        assert!(observed.iter().any(|line| line.contains("processor init")));
        assert!(observed.iter().any(|line| line.contains("processor block")));
        poll(vm.lua());
        assert_eq!(
            *lines.borrow(),
            observed,
            "processor log polling must drain each line once"
        );
    }

    #[test]
    fn authored_audio_idle_initialization_failure_is_polled_once() {
        use nuxie_runtime::ScriptingVm;
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let _engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let lines = Rc::new(RefCell::new(Vec::new()));
        let captured = lines.clone();
        vm.set_log_sink(move |_, bytes| {
            captured
                .borrow_mut()
                .push(String::from_utf8_lossy(bytes).into_owned())
        });
        register(
            &vm,
            "failed-init",
            "return {init=function() error('audio init rejected') end, process=function() end}",
        );
        let _owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.start('failed-init'); return true end}",
        );
        assert!(ScriptingVm::has_pending_async_work(&vm));
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut delivered = false;
        while !lines
            .borrow()
            .iter()
            .any(|line| line.contains("audio init rejected"))
        {
            delivered |= ScriptingVm::poll_async_work(&vm).unwrap();
            assert!(
                Instant::now() < deadline,
                "idle runtime did not report initialization failure"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(delivered);
        for _ in 0..3 {
            ScriptingVm::poll_async_work(&vm).unwrap();
        }
        assert_eq!(
            lines
                .borrow()
                .iter()
                .filter(|line| line.contains("audio init rejected"))
                .count(),
            1
        );
        // Seeing failure text need not mean the initialization worker has
        // published its final wake yet. Drain until this VM is actually idle.
        while ScriptingVm::has_pending_async_work(&vm) {
            ScriptingVm::poll_async_work(&vm).unwrap();
            assert!(
                Instant::now() < deadline,
                "failed processor kept idle VM pending"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(!ScriptingVm::has_pending_async_work(&vm));
        assert_eq!(
            lines
                .borrow()
                .iter()
                .filter(|line| line.contains("audio init rejected"))
                .count(),
            1
        );
    }

    #[test]
    fn authored_audio_new_start_preserves_retained_failed_processor_events() {
        use nuxie_runtime::ScriptingVm;
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        let lines = Rc::new(RefCell::new(Vec::new()));
        let captured = lines.clone();
        vm.set_log_sink(move |_, bytes| {
            captured
                .borrow_mut()
                .push(String::from_utf8_lossy(bytes).into_owned())
        });
        register(
            &vm,
            "failed-process",
            "return {init=function() end, process=function() print('last processor line'); error('audio block rejected') end}",
        );
        register(
            &vm,
            "silent",
            "return {init=function() end, process=function() end}",
        );
        let mut owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.start('failed-process'); return true end, advance=function(self) self.second=Audio.start('silent'); return true end}",
        );
        wait_running(&vm);
        let mut frames = vec![0.0; engine.buffer_frames() as usize * 2];
        engine.read_audio_frames(&mut frames);
        // Exercise admission before ordinary callback polling, while the old
        // failed handle is still externally retained and its events undelivered.
        owned
            .call_method_value_with_current_budget(super::super::ScriptMethod::Advance, &[])
            .unwrap();
        ScriptingVm::poll_async_work(&vm).unwrap();
        assert_eq!(
            lines
                .borrow()
                .iter()
                .filter(|line| line.contains("last processor line"))
                .count(),
            1
        );
        assert_eq!(
            lines
                .borrow()
                .iter()
                .filter(|line| line.contains("audio block rejected"))
                .count(),
            1
        );
        ScriptingVm::poll_async_work(&vm).unwrap();
        assert_eq!(
            lines
                .borrow()
                .iter()
                .filter(|line| line.contains("audio block rejected"))
                .count(),
            1
        );
    }

    #[test]
    fn authored_audio_invalid_input_and_shared_backpressure_are_visible() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let vm = ScriptVm::new();
        register(&vm, "tone", PROCESSOR);
        let _owned = instance(
            &vm,
            r#"
            return {init=function(self)
                local function rejects(f) assert(not pcall(f)) end
                rejects(function() Audio.start('missing') end)
                rejects(function() Audio.start('tone', {}, {hardBlocks=-1}) end)
                rejects(function() Audio.start('tone', {}, {hardBlocks=0.5}) end)
                local cyclic = {}; cyclic.self = cyclic
                rejects(function() Audio.start('tone', cyclic) end)
                rejects(function() Audio.start('tone', {callback=function() end}) end)
                local q = Audio.Ring.create(128)
                local full = false
                for i=1,1000 do
                    if not q:send({value=i}) then full=true; break end
                end
                assert(full)
                local value = {}; assert(q:receiveInto(value)); assert(value.value == 1)
                local s = Audio.Snapshot.create(1024)
                s:publish({value=7})
                assert(s:latest().value.value == 7)
                assert(s:latest().sequence == s:latest().sequence)
                return true
            end}
        "#,
        );
        assert!(vm.lua().load("Audio.start('tone')").exec().is_err());
    }

    #[test]
    fn authored_audio_vm_teardown_stops_retained_instance_and_handle() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let engine = AudioEngine::make_and_store(2, 48000).unwrap();
        let vm = ScriptVm::new();
        register(&vm, "tone", PROCESSOR);
        let owned = instance(
            &vm,
            "return {init=function(self) retained=Audio.start('tone',{gain=0.1,messages=Audio.Ring.create(1024),meter=Audio.Snapshot.create(1024)}); return true end}",
        );
        wait_running(&vm);
        let retained: luaur_rt::AnyUserData = vm.lua().globals().get("retained").unwrap();
        drop(vm);
        assert_eq!(
            retained.borrow::<ScriptedProcessor>().unwrap().0.state(),
            "stopped"
        );
        let handle = retained.borrow::<ScriptedProcessor>().unwrap().0.clone();
        assert!(
            !handle.has_pending_work(),
            "a retained userdata must not keep the dropped VM globally awake"
        );
        assert!(
            !handle.poll_notification(),
            "owner teardown must consume the queued wake notification"
        );
        let mut frames = vec![0.0; 256];
        engine.read_audio_frames(&mut frames);
        assert!(frames.iter().all(|v| *v == 0.0));
        drop(owned);
    }

    #[test]
    fn authored_audio_failed_main_init_stops_its_started_processor() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let vm = ScriptVm::new();
        register(&vm, "tone", PROCESSOR);
        vm.install_rive_globals().unwrap();
        #[cfg(feature = "tools")]
        vm.set_is_playing(true);
        let table: Table = vm.lua().load("return {init=function(self) retained=Audio.start('tone',{gain=0.1,messages=Audio.Ring.create(1024),meter=Audio.Snapshot.create(1024)}); return false end}").eval().unwrap();
        let mut owned = vm.script_instance_from_table(table);
        assert!(!owned.call_init_with_optional_factory(None).unwrap());
        vm.lua()
            .load("assert(retained.state == 'stopped')")
            .exec()
            .unwrap();
    }

    #[cfg(feature = "tools")]
    #[test]
    fn authored_audio_start_obeys_editor_playback_state() {
        let _guard = AUDIO_TEST_LOCK.lock().unwrap();
        let vm = ScriptVm::new();
        register(&vm, "tone", PROCESSOR);
        let mut owned = instance(
            &vm,
            "return {init=function(self) return true end, advance=function(self) retained=Audio.start('tone',{}); return true end}",
        );
        vm.set_is_playing(false);
        assert!(
            owned
                .call_method_value(super::super::ScriptMethod::Advance, &[])
                .is_err()
        );
    }
}

struct ScriptedBlob(Arc<SharedBlob>);
impl UserData for ScriptedBlob {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.0.size()));
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "read",
            |_, this, (offset, buffer): (usize, luaur_rt::Buffer)| {
                let mut bytes = vec![0; buffer.len()];
                let count = this.0.read(offset, &mut bytes);
                buffer.write_bytes(0, &bytes[..count]);
                Ok(count)
            },
        );
        methods.add_method("toBuffer", |lua, this, (): ()| {
            let mut bytes = vec![0; this.0.size()];
            this.0.read(0, &mut bytes);
            lua.create_buffer(bytes)
        });
    }
}
struct ScriptedRegion(Arc<SharedRegion>);
#[derive(Default)]
struct RegionHeld(Cell<bool>);
struct ScriptedRegionView(Rc<RefCell<Option<RegionGuard>>>);
impl UserData for ScriptedRegionView {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "read",
            |_, this, (offset, buffer): (usize, luaur_rt::Buffer)| {
                let guard = this.0.borrow();
                let guard = guard
                    .as_ref()
                    .ok_or_else(|| Error::runtime("region view has expired"))?;
                let mut bytes = vec![0; buffer.len()];
                guard.read(offset, &mut bytes).map_err(Error::runtime)?;
                buffer.write_bytes(0, &bytes);
                Ok(())
            },
        );
        methods.add_method(
            "write",
            |_, this, (offset, buffer): (usize, luaur_rt::Buffer)| {
                let guard = this.0.borrow();
                guard
                    .as_ref()
                    .ok_or_else(|| Error::runtime("region view has expired"))?
                    .write(offset, &buffer.to_vec())
                    .map_err(Error::runtime)
            },
        );
        methods.add_method("getU32", |_, this, offset: usize| {
            let guard = this.0.borrow();
            let mut bytes = [0; 4];
            guard
                .as_ref()
                .ok_or_else(|| Error::runtime("region view has expired"))?
                .read(offset, &mut bytes)
                .map_err(Error::runtime)?;
            Ok(u32::from_le_bytes(bytes))
        });
        methods.add_method("setU32", |_, this, (offset, value): (usize, u32)| {
            let guard = this.0.borrow();
            guard
                .as_ref()
                .ok_or_else(|| Error::runtime("region view has expired"))?
                .write(offset, &value.to_le_bytes())
                .map_err(Error::runtime)
        });
    }
}
fn region_exclusive(
    lua: &Lua,
    region: &Arc<SharedRegion>,
    callback: luaur_rt::Function,
    timeout: f64,
) -> Result<bool> {
    if !timeout.is_finite() || timeout < 0.0 {
        return Err(Error::runtime("invalid region timeout"));
    }
    if lua.app_data_ref::<RegionHeld>().is_none() {
        lua.set_app_data(RegionHeld::default());
    }
    if lua
        .app_data_ref::<RegionHeld>()
        .expect("installed region state")
        .0
        .replace(true)
    {
        return Err(Error::runtime("nested region locking is forbidden"));
    }
    let result = (|| {
        let Some(guard) = region
            .try_lock(std::time::Duration::try_from_secs_f64(timeout).map_err(Error::runtime)?)
            .map_err(Error::runtime)?
        else {
            return Ok(false);
        };
        let scoped = Rc::new(RefCell::new(Some(guard)));
        let view = lua.create_userdata(ScriptedRegionView(scoped.clone()))?;
        let called = callback.call::<()>(view);
        if called.is_err() {
            if let Some(guard) = scoped.borrow().as_ref() {
                guard.poison();
            }
        }
        scoped.borrow_mut().take();
        called?;
        Ok(true)
    })();
    lua.app_data_ref::<RegionHeld>()
        .expect("installed region state")
        .0
        .set(false);
    result
}
impl UserData for ScriptedRegion {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.0.size()));
        fields.add_field_method_get("poisoned", |_, this| Ok(this.0.poisoned()));
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("atomicLoad", |_, this, offset: usize| {
            this.0.atomic_load(offset).map_err(Error::runtime)
        });
        methods.add_method("atomicStore", |_, this, (offset, value): (usize, u32)| {
            this.0.atomic_store(offset, value).map_err(Error::runtime)
        });
        methods.add_method("atomicAdd", |_, this, (offset, value): (usize, u32)| {
            this.0.atomic_add(offset, value).map_err(Error::runtime)
        });
        methods.add_method(
            "atomicCas",
            |_, this, (offset, expected, desired): (usize, u32, u32)| {
                this.0
                    .atomic_cas(offset, expected, desired)
                    .map_err(Error::runtime)
            },
        );
        // Public upstream host primitive takes seconds. Do not guess private
        // AssemblyScript helper defaults: this Luau adapter requires a timeout.
        methods.add_method(
            "tryExclusive",
            |lua, this, (callback, timeout_seconds): (luaur_rt::Function, f64)| {
                region_exclusive(lua, &this.0, callback, timeout_seconds)
            },
        );
    }
}
