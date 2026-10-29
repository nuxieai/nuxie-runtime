//! Luau host adaptation of upstream audio sample jobs and streams through aec8aea4.
//! Decoder jobs never retain Lua values. Stream pumping runs on the owner lane;
//! processors consume framed PCM through the shared reader without decoding.
use super::luau_audio_samples::PcmStreamDescriptor;
use super::luau_audio_shared::{DecodeReservation, MAX_SHARED_BYTES, SharedBlob, SharedBudget};
use super::{lua_audio_processors, lua_image_decode, lua_promise};
use luaur_rt::{
    AnyUserData, Error, Lua, RegistryKey, Result, UserData, UserDataFields, UserDataMethods, Value,
};
use nuxie_runtime::{
    AudioReader, AudioSource, WorkCallbacks, WorkCancellationHandle, WorkPool, WorkStatusHandle,
    WorkTask, get_global_work_pool,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    rc::{Rc, Weak},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

const CHUNKS_PER_PUMP: u32 = 16;
const HEADER_BYTES: usize = 16;
const LAST: u32 = 1;
const DISCONTINUITY: u32 = 2;
const COOPERATIVE_SLICE_MICROS: i64 = 2_000;
const DECODE_ERROR: &str = "failed to decode audio samples";
fn lock<T>(value: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|e| e.into_inner())
}

struct Completion {
    sequence: u64,
    request: u64,
    result: std::result::Result<Vec<f32>, String>,
}
#[derive(Default)]
struct CompletionQueue(VecDeque<Completion>);
impl CompletionQueue {
    fn push(&mut self, value: Completion) {
        self.0.push_back(value);
        nuxie_runtime::vm_completions_queued(1);
    }
    fn pop(&mut self) -> Option<Completion> {
        let value = self.0.pop_front();
        if value.is_some() {
            nuxie_runtime::vm_completions_taken(1);
        }
        value
    }
}
impl Drop for CompletionQueue {
    fn drop(&mut self) {
        nuxie_runtime::vm_completions_taken(self.0.len());
    }
}
struct DecodeTask {
    request: u64,
    reader: AudioReader,
    max_frames: u64,
    stop: Arc<AtomicBool>,
    samples: Vec<f32>,
    result: Option<Vec<f32>>,
    completions: Arc<Mutex<CompletionQueue>>,
}
impl DecodeTask {
    // None is upstream Clock::time_point::max(): worker and offline decoding.
    // Each call performs at least one block, even when another pending decode
    // already used the owner's shared cooperative slice.
    fn decode_until(&mut self, until: Option<i64>) -> bool {
        let mut done = false;
        loop {
            if self.stop.load(Ordering::Relaxed)
                || !self
                    .reader
                    .read_block(self.max_frames, &mut self.samples, &mut done)
            {
                self.samples = Vec::new();
                return true;
            }
            if done {
                break;
            }
            if until.is_some_and(|end| {
                nuxie_runtime::mechanical_port::source::engine_now_micros() >= end
            }) {
                return false;
            }
        }
        if !self.samples.is_empty() {
            self.result = Some(std::mem::take(&mut self.samples));
        }
        true
    }
}
impl WorkCallbacks for DecodeTask {
    fn execute(&mut self, error: &mut String) -> bool {
        self.decode_until(None);
        if self.result.is_none() {
            *error = DECODE_ERROR.into();
            return false;
        }
        true
    }
    fn on_complete(&mut self) {
        if let Some(samples) = self.result.take() {
            lock(&self.completions).push(Completion {
                sequence: lua_image_decode::next_completion_sequence(),
                request: self.request,
                result: Ok(samples),
            });
        }
    }
    fn on_error(&mut self, error: &str) {
        lock(&self.completions).push(Completion {
            sequence: lua_image_decode::next_completion_sequence(),
            request: self.request,
            result: Err(error.into()),
        });
    }
    fn on_cancel(&mut self) {
        self.samples = Vec::new();
        self.result = None;
    }
}
struct PendingDecode {
    promise: RegistryKey,
    cancellation: Option<WorkCancellationHandle>,
    status: Option<WorkStatusHandle>,
    sliced: Option<DecodeTask>,
    stop: Arc<AtomicBool>,
    reservation: DecodeReservation,
}
impl Drop for PendingDecode {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(cancellation) = &self.cancellation {
            cancellation.cancel();
        }
    }
}
struct StreamWait {
    sequence: u64,
    promise: RegistryKey,
    stream: Rc<RefCell<HostStream>>,
}
impl StreamWait {
    fn ready(&self) -> bool {
        let stream = self.stream.borrow();
        !stream.live || !stream.controller || stream.ended || stream.descriptor.ring.used() != 0
    }
}
impl Drop for StreamWait {
    fn drop(&mut self) {
        nuxie_runtime::vm_completions_taken(1);
    }
}
struct Registry {
    next: Cell<u64>,
    owner: u64,
    budget: Arc<SharedBudget>,
    pending: RefCell<HashMap<u64, PendingDecode>>,
    completions: Arc<Mutex<CompletionQueue>>,
    waits: RefCell<VecDeque<StreamWait>>,
    streams: RefCell<Vec<Rc<RefCell<HostStream>>>>,
}
impl Registry {
    fn id(&self) -> u64 {
        let id = self.next.get();
        self.next
            .set(id.checked_add(1).expect("audio request identity exhausted"));
        id
    }
    fn cancel(&self, id: u64) {
        self.pending.borrow_mut().remove(&id);
    }
    fn shutdown(&self) {
        self.pending.borrow_mut().clear();
        self.waits.borrow_mut().clear();
        for stream in self.streams.borrow_mut().drain(..) {
            stream.borrow_mut().close();
        }
    }
}
struct RegistryOwner(Rc<Registry>);
impl Drop for RegistryOwner {
    fn drop(&mut self) {
        self.0.shutdown();
    }
}
pub(super) fn install(lua: &Lua) -> Result<()> {
    if lua.app_data_ref::<RegistryOwner>().is_none() {
        let budget = lua_audio_processors::shared_budget(lua)?;
        lua.set_app_data(RegistryOwner(Rc::new(Registry {
            next: Cell::new(1),
            owner: WorkPool::next_owner_id(),
            budget,
            pending: RefCell::new(HashMap::new()),
            completions: Arc::default(),
            waits: RefCell::default(),
            streams: RefCell::default(),
        })));
    }
    Ok(())
}
fn registry(lua: &Lua) -> Result<Rc<Registry>> {
    install(lua)?;
    Ok(lua.app_data_ref::<RegistryOwner>().unwrap().0.clone())
}
pub(super) fn shutdown(lua: &Lua) {
    if let Some(owner) = lua.app_data_ref::<RegistryOwner>() {
        owner.0.shutdown();
    }
}
fn installed(lua: &Lua) -> Option<Rc<Registry>> {
    lua.app_data_ref::<RegistryOwner>()
        .map(|owner| owner.0.clone())
}

pub(super) fn to_blob(
    lua: &Lua,
    source: &AudioSource,
    sample_rate: u32,
    channels: u32,
) -> Result<AnyUserData> {
    let registry = registry(lua)?;
    let reader = source
        .make_reader(channels, sample_rate)
        .ok_or_else(|| Error::runtime(DECODE_ERROR))?;
    if reader.channels() == 0 {
        return Err(Error::runtime(DECODE_ERROR));
    }
    let frame_bytes = u64::from(reader.channels()) * 4;
    let max_frames = MAX_SHARED_BYTES as u64 / frame_bytes;
    let estimate = reader.length_in_frames().min(max_frames) * frame_bytes;
    let promise = lua_promise::new_pending(lua)?;
    let reservation = match registry.budget.reserve_decode(estimate as usize) {
        Ok(reservation) => reservation,
        Err(error) => {
            lua_promise::reject(lua, promise.clone(), error)?;
            return Ok(promise);
        }
    };
    let id = registry.id();
    let stop = Arc::new(AtomicBool::new(false));
    let decode = DecodeTask {
        request: id,
        reader,
        max_frames,
        stop: stop.clone(),
        samples: Vec::new(),
        result: None,
        completions: registry.completions.clone(),
    };
    let (sliced, task) = if get_global_work_pool().thread_count() == 0 {
        (Some(decode), None)
    } else {
        let mut task = WorkTask::new(decode);
        task.set_owner_id(registry.owner);
        (None, Some(task))
    };
    let reference = lua.create_registry_value(promise.clone())?;
    let weak = Rc::downgrade(&registry);
    let cancel = lua.create_function(move |_, ()| {
        if let Some(registry) = weak.upgrade() {
            registry.cancel(id);
        }
        Ok(())
    })?;
    lua_promise::set_on_cancel(lua, promise.clone(), cancel)?;
    registry.pending.borrow_mut().insert(
        id,
        PendingDecode {
            promise: reference,
            cancellation: task.as_ref().map(WorkTask::cancellation_handle),
            status: task.as_ref().map(WorkTask::status_handle),
            sliced,
            stop,
            reservation,
        },
    );
    if let Some(task) = task {
        get_global_work_pool().submit(Some(Box::new(task)));
    }
    Ok(promise)
}

struct HostStream {
    reader: AudioReader,
    descriptor: PcmStreamDescriptor,
    samples: Vec<f32>,
    record: Vec<u8>,
    queued: bool,
    queued_last: bool,
    full_record_bytes: usize,
    used_after_fill: usize,
    position: f64,
    discontinuity: bool,
    looping: bool,
    ended: bool,
    controller: bool,
    live: bool,
    wake: bool,
}
impl HostStream {
    fn wake(&mut self, value: bool) {
        if self.wake != value {
            self.wake = value;
            if value {
                nuxie_runtime::vm_completions_queued(1);
            } else {
                nuxie_runtime::vm_completions_taken(1);
            }
        }
    }
    fn close(&mut self) {
        self.live = false;
        self.wake(false);
    }
    fn decode_record(&mut self) -> std::result::Result<(), String> {
        let mut decoded = self.reader.read_into(&mut self.samples);
        if matches!(decoded, Ok(0)) && self.looping && self.reader.rewind() {
            self.position = 0.0;
            decoded = self.reader.read_into(&mut self.samples);
        }
        let (frames, failed) = match decoded {
            Ok(frames) => (frames, false),
            Err(error) => (error.frames_read, true),
        };
        self.queued_last = failed || frames == 0;
        let flags = if self.queued_last { LAST } else { 0 }
            | if self.discontinuity { DISCONTINUITY } else { 0 };
        self.record.clear();
        self.record
            .extend_from_slice(&self.descriptor.generation()?.to_le_bytes());
        self.record.extend_from_slice(&flags.to_le_bytes());
        self.record.extend_from_slice(&self.position.to_le_bytes());
        for sample in &self.samples[..frames as usize * self.reader.channels() as usize] {
            self.record.extend_from_slice(&sample.to_le_bytes());
        }
        self.discontinuity = false;
        self.position += frames as f64;
        self.queued = true;
        Ok(())
    }
    fn fill(&mut self, max_chunks: u32) -> std::result::Result<(), String> {
        for _ in 0..max_chunks {
            if self.ended || !self.live {
                break;
            }
            if !self.queued {
                self.decode_record()?;
            }
            if !self.descriptor.ring.write_record(&self.record)? {
                break;
            }
            self.queued = false;
            self.ended = self.queued_last;
        }
        self.used_after_fill = self.descriptor.ring.used();
        self.wake(!self.ended && self.live);
        Ok(())
    }
    fn pump(&mut self) -> std::result::Result<(), String> {
        let played = self
            .used_after_fill
            .saturating_sub(self.descriptor.ring.used());
        let chunks = (2 * (played / self.full_record_bytes) + 1).min(u32::MAX as usize) as u32;
        self.fill(CHUNKS_PER_PUMP.max(chunks))
    }
    fn restart(&mut self, frame: u64) -> std::result::Result<bool, String> {
        if !self.live {
            return Ok(false);
        }
        let Some(landed) = self.reader.seek(frame) else {
            return Ok(false);
        };
        self.descriptor.bump_generation()?;
        self.record.resize(HEADER_BYTES + self.samples.len() * 4, 0);
        while self.descriptor.ring.read_record(&mut self.record)? != 0 {}
        self.queued = false;
        self.ended = false;
        self.position = landed as f64;
        self.discontinuity = true;
        self.fill(CHUNKS_PER_PUMP)?;
        Ok(true)
    }
    fn set_loop(&mut self, value: bool) -> std::result::Result<(), String> {
        self.looping = value;
        if !value || !self.live {
            return Ok(());
        }
        if self.ended && self.reader.rewind() {
            self.ended = false;
            self.position = 0.0;
            self.fill(CHUNKS_PER_PUMP)?;
        } else if self.queued_last && self.queued && self.record.len() == HEADER_BYTES {
            self.queued = false;
        }
        Ok(())
    }
}
impl Drop for HostStream {
    fn drop(&mut self) {
        self.wake(false);
    }
}
struct StreamController {
    stream: Weak<RefCell<HostStream>>,
    released: Cell<bool>,
}
impl StreamController {
    fn stream(&self) -> Result<Rc<RefCell<HostStream>>> {
        self.stream
            .upgrade()
            .filter(|s| !self.released.get() && s.borrow().live)
            .ok_or_else(|| Error::runtime("audio stream released"))
    }
    fn release(&self) {
        if self.released.replace(true) {
            return;
        }
        if let Some(stream) = self.stream.upgrade() {
            let mut stream = stream.borrow_mut();
            stream.controller = false;
            stream.wake(true);
        }
    }
}
impl Drop for StreamController {
    fn drop(&mut self) {
        self.release();
    }
}
impl UserData for StreamController {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("sampleRate", |_, this| {
            Ok(this.stream()?.borrow().reader.sample_rate())
        });
        fields.add_field_method_get("channels", |_, this| {
            Ok(this.stream()?.borrow().reader.channels())
        });
        fields.add_field_method_get("ended", |_, this| Ok(this.stream()?.borrow().ended));
        fields.add_field_method_get("loop", |_, this| Ok(this.stream()?.borrow().looping));
        fields.add_field_method_set("loop", |_, this, value: bool| {
            this.stream()?
                .borrow_mut()
                .set_loop(value)
                .map_err(Error::runtime)
        });
        fields.add_field_method_get("descriptor", |lua, this| {
            lua_audio_processors::descriptor_value(lua, &this.stream()?.borrow().descriptor)
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("reader", |lua, this, ()| {
            lua_audio_processors::stream_reader_value(
                lua,
                this.stream()?.borrow().descriptor.clone(),
            )
        });
        methods.add_method("seekFrame", |_, this, frame: f64| {
            this.stream()?
                .borrow_mut()
                .restart(if frame > 0.0 {
                    frame.min(1.8e19) as u64
                } else {
                    0
                })
                .map_err(Error::runtime)
        });
        methods.add_method("release", |_, this, ()| {
            this.release();
            Ok(())
        });
    }
}
pub(super) fn stream(
    lua: &Lua,
    source: &AudioSource,
    sample_rate: u32,
    channels: u32,
    chunk_frames: u32,
    buffer_frames: u32,
) -> Result<AnyUserData> {
    if chunk_frames == 0 {
        return Err(Error::runtime("audio stream chunkFrames must be nonzero"));
    }
    let registry = registry(lua)?;
    let reader = source
        .make_reader(channels, sample_rate)
        .ok_or_else(|| Error::runtime(DECODE_ERROR))?;
    if reader.channels() == 0 {
        return Err(Error::runtime(DECODE_ERROR));
    }
    let chunk_bytes =
        HEADER_BYTES as u64 + u64::from(chunk_frames) * u64::from(reader.channels()) * 4;
    if chunk_bytes > MAX_SHARED_BYTES as u64 {
        return Err(Error::runtime("audio stream chunk exceeds shared limit"));
    }
    let records = (u64::from(buffer_frames).div_ceil(u64::from(chunk_frames))).max(1);
    // SharedRing prefixes each hold-free record with its four-byte length.
    let full_record_bytes = chunk_bytes + 4;
    let capacity = records
        .checked_mul(full_record_bytes)
        .filter(|size| *size <= MAX_SHARED_BYTES as u64)
        .ok_or_else(|| Error::runtime("audio stream buffer exceeds shared limit"))?;
    let descriptor = PcmStreamDescriptor::new(
        capacity as usize,
        reader.sample_rate(),
        reader.channels(),
        chunk_frames,
        registry.budget.clone(),
    )
    .map_err(Error::runtime)?;
    let samples = vec![0.0; chunk_frames as usize * reader.channels() as usize];
    let mut stream = HostStream {
        reader,
        descriptor,
        samples,
        record: Vec::with_capacity(chunk_bytes as usize),
        queued: false,
        queued_last: false,
        full_record_bytes: full_record_bytes as usize,
        used_after_fill: 0,
        position: 0.0,
        discontinuity: false,
        looping: false,
        ended: false,
        controller: true,
        live: true,
        wake: false,
    };
    stream.fill(CHUNKS_PER_PUMP).map_err(Error::runtime)?;
    let stream = Rc::new(RefCell::new(stream));
    let promise = lua_promise::new_pending(lua)?;
    let key = lua.create_registry_value(promise.clone())?;
    let weak = Rc::downgrade(&stream);
    lua_promise::set_on_cancel(
        lua,
        promise.clone(),
        lua.create_function(move |_, ()| {
            if let Some(stream) = weak.upgrade() {
                let mut stream = stream.borrow_mut();
                stream.controller = false;
                stream.wake(true);
            }
            Ok(())
        })?,
    )?;
    nuxie_runtime::vm_completions_queued(1);
    registry.waits.borrow_mut().push_back(StreamWait {
        sequence: lua_image_decode::next_completion_sequence(),
        promise: key,
        stream: stream.clone(),
    });
    registry.streams.borrow_mut().push(stream);
    Ok(promise)
}

pub(super) fn pending_sequence(lua: &Lua) -> Option<u64> {
    let registry = installed(lua)?;
    let decode = lock(&registry.completions).0.front().map(|c| c.sequence);
    let stream = registry
        .waits
        .borrow()
        .iter()
        .find(|wait| wait.ready())
        .map(|w| w.sequence);
    match (decode, stream) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}
pub(super) fn has_pending_work(lua: &Lua) -> bool {
    installed(lua).is_some_and(|r| {
        !r.pending.borrow().is_empty()
            || !r.waits.borrow().is_empty()
            || !lock(&r.completions).0.is_empty()
            || r.streams.borrow().iter().any(|s| s.borrow().wake)
    })
}
fn advance_decodes(registry: &Registry, offline: bool, slice_end: i64) {
    // Upstream main waits retain creation order. HashMap is only lookup storage.
    // No registry borrow crosses decoding or a threaded join.
    let mut jobs: Vec<_> = registry
        .pending
        .borrow()
        .iter()
        .map(|(id, job)| (*id, job.status.clone()))
        .collect();
    jobs.sort_by_key(|(id, _)| *id);
    for (id, status) in jobs {
        if let Some(status) = status {
            if offline {
                get_global_work_pool().join_task(&status, std::time::Duration::from_secs(10));
            }
            continue;
        }
        let task = registry
            .pending
            .borrow_mut()
            .get_mut(&id)
            .and_then(|job| job.sliced.take());
        let Some(mut task) = task else { continue };
        if task.decode_until((!offline).then_some(slice_end)) {
            if task.result.is_some() {
                task.on_complete();
            } else {
                task.on_error(DECODE_ERROR);
            }
        } else if let Some(pending) = registry.pending.borrow_mut().get_mut(&id) {
            pending.sliced = Some(task);
        }
    }
}

pub(super) fn poll(lua: &Lua, limit: usize) -> Result<bool> {
    let Some(registry) = installed(lua) else {
        return Ok(false);
    };
    let slice_end = nuxie_runtime::mechanical_port::source::engine_now_micros()
        .saturating_add(COOPERATIVE_SLICE_MICROS);
    advance_decodes(
        &registry,
        super::luau_audio_processor_engine::render_offline(),
        slice_end,
    );
    let mut settled = false;
    let mut outcomes: Vec<(AnyUserData, std::result::Result<Value, String>)> = Vec::new();
    for _ in 0..limit {
        let decode = lock(&registry.completions).0.front().map(|c| c.sequence);
        let wait = registry
            .waits
            .borrow()
            .iter()
            .enumerate()
            .find(|(_, wait)| wait.ready())
            .map(|(index, wait)| (index, wait.sequence));
        if decode.is_none() && wait.is_none() {
            break;
        }
        if wait.is_some_and(|(_, sequence)| decode.is_none_or(|d| sequence < d)) {
            let index = wait.unwrap().0;
            let wait = registry.waits.borrow_mut().remove(index).unwrap();
            let promise = lua.registry_value::<AnyUserData>(&wait.promise)?;
            if wait.stream.borrow().live && wait.stream.borrow().controller {
                let controller = lua.create_userdata(StreamController {
                    stream: Rc::downgrade(&wait.stream),
                    released: Cell::new(false),
                })?;
                outcomes.push((promise, Ok(Value::UserData(controller))));
            } else if wait.stream.borrow().controller {
                outcomes.push((
                    promise,
                    Err("audio stream closed before initial fill".into()),
                ));
            }
            settled = true;
            continue;
        }
        let completion = lock(&registry.completions).pop().unwrap();
        let Some(pending) = registry.pending.borrow_mut().remove(&completion.request) else {
            continue;
        };
        let promise = lua.registry_value::<AnyUserData>(&pending.promise)?;
        // Refund the estimate before charging the actual decoded size.
        drop(pending);
        let value = completion.result.and_then(|samples| {
            let bytes: Vec<u8> = samples
                .iter()
                .flat_map(|sample| sample.to_le_bytes())
                .collect();
            SharedBlob::with_budget(&bytes, registry.budget.clone()).map(Arc::new)
        });
        match value {
            Ok(blob) => {
                let value = lua_audio_processors::blob_value(lua, blob)?;
                outcomes.push((promise, Ok(value)));
            }
            Err(error) => outcomes.push((promise, Err(error))),
        }
        settled = true;
    }
    let streams = registry.streams.borrow().clone();
    for stream in streams {
        let mut stream = stream.borrow_mut();
        if !stream.controller && (stream.ended || stream.descriptor.external_holds() == 0) {
            stream.close();
            continue;
        }
        if let Err(_) = stream.pump() {
            stream.close();
        }
    }
    registry
        .streams
        .borrow_mut()
        .retain(|stream| stream.borrow().live);
    // Upstream holds script outcomes until all stream pumps have run.
    for (promise, outcome) in outcomes {
        match outcome {
            Ok(value) => lua_promise::resolve(lua, promise, value)?,
            Err(error) => lua_promise::reject(lua, promise, error)?,
        }
    }
    Ok(settled)
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    // Exercise the production owner boundary, including on assertion unwind.
    // Raw Lua plus app-data RegistryKeys can otherwise outlive TLS teardown.
    struct OwnedLua(super::super::ScriptVm);
    impl std::ops::Deref for OwnedLua {
        type Target = Lua;
        fn deref(&self) -> &Lua {
            self.0.lua()
        }
    }
    fn lua() -> OwnedLua {
        let owner = OwnedLua(super::super::ScriptVm::new());
        let lua = owner.0.lua();
        lua_promise::install_promise_globals(&lua).unwrap();
        let audio = lua.create_table();
        lua_audio_processors::install(&lua, &audio).unwrap();
        lua.globals().set("Audio", audio).unwrap();
        owner
    }
    fn source() -> AudioSource {
        AudioSource::from_buffered(vec![0.25, 0.5, 0.75, 1.0], 1, 48_000).unwrap()
    }
    fn decode_task(
        request: u64,
        samples: Vec<f32>,
        completions: Arc<Mutex<CompletionQueue>>,
    ) -> DecodeTask {
        DecodeTask {
            request,
            reader: AudioSource::from_buffered(samples, 1, 48_000)
                .unwrap()
                .make_reader(1, 48_000)
                .unwrap(),
            max_frames: u64::MAX,
            stop: Arc::new(AtomicBool::new(false)),
            samples: Vec::new(),
            result: None,
            completions,
        }
    }
    #[test]
    fn upstream_audio_samples_decode_slice_retains_pcm_and_resumes() {
        let samples: Vec<f32> = (0..12_289).map(|i| i as f32 / 16_384.0).collect();
        let mut task = decode_task(1, samples.clone(), Arc::default());
        // An expired shared deadline still permits exactly one source block.
        assert!(!task.decode_until(Some(i64::MIN)));
        assert_eq!(task.samples, samples[..4096]);
        assert!(task.result.is_none());
        assert!(!task.decode_until(Some(i64::MIN)));
        assert_eq!(task.samples, samples[..8192]);
        assert!(task.decode_until(None));
        assert_eq!(task.result.as_deref(), Some(samples.as_slice()));
        assert!(task.samples.is_empty());
    }
    #[test]
    fn upstream_audio_samples_cancel_between_slices_discards_partial_pcm() {
        let mut task = decode_task(1, vec![0.5; 12_289], Arc::default());
        assert!(!task.decode_until(Some(i64::MIN)));
        assert_eq!(task.samples.len(), 4096);
        task.stop.store(true, Ordering::Relaxed);
        assert!(task.decode_until(None));
        assert!(task.samples.is_empty());
        assert_eq!(task.samples.capacity(), 0);
        assert!(task.result.is_none());
    }
    #[test]
    fn upstream_audio_samples_each_pending_decode_gets_a_block_in_creation_order() {
        let lua = lua();
        let registry = registry(&lua).unwrap();
        // Insert backwards deliberately: lookup storage must not set wait order.
        for id in [2, 1] {
            let promise = lua_promise::new_pending(&lua).unwrap();
            let task = decode_task(id, vec![id as f32; 5000], registry.completions.clone());
            registry.pending.borrow_mut().insert(
                id,
                PendingDecode {
                    promise: lua.create_registry_value(promise).unwrap(),
                    cancellation: None,
                    status: None,
                    stop: task.stop.clone(),
                    sliced: Some(task),
                    reservation: registry.budget.reserve_decode(5000 * 4).unwrap(),
                },
            );
        }
        advance_decodes(&registry, false, i64::MIN);
        for pending in registry.pending.borrow().values() {
            assert_eq!(pending.sliced.as_ref().unwrap().samples.len(), 4096);
        }
        assert!(lock(&registry.completions).0.is_empty());
        advance_decodes(&registry, false, i64::MIN);
        advance_decodes(&registry, false, i64::MIN);
        assert_eq!(
            lock(&registry.completions)
                .0
                .iter()
                .map(|item| item.request)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert!(
            registry
                .pending
                .borrow()
                .values()
                .all(|job| job.sliced.is_none())
        );
        shutdown(&lua);
    }
    #[test]
    fn upstream_audio_samples_selects_owner_slices_only_without_pool_threads() {
        let lua = lua();
        let registry = registry(&lua).unwrap();
        let _promise = to_blob(&lua, &source(), 48_000, 1).unwrap();
        {
            let pending = registry.pending.borrow();
            let job = pending.values().next().unwrap();
            let sliced = get_global_work_pool().thread_count() == 0;
            assert_eq!(job.sliced.is_some(), sliced);
            assert_eq!(job.status.is_some(), !sliced);
            assert_eq!(job.cancellation.is_some(), !sliced);
            if sliced {
                assert!(!get_global_work_pool().has_pending_work_for_owner(registry.owner));
            }
        }
        settle(&lua);
        shutdown(&lua);
    }
    fn settle(lua: &Lua) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            get_global_work_pool().poll_completed_work(16);
            poll(lua, usize::MAX).unwrap();
            if registry(lua).unwrap().pending.borrow().is_empty() {
                break;
            }
            assert!(Instant::now() < deadline, "audio decode did not settle");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn controller(lua: &Lua, chunk: u32, capacity: u32) -> AnyUserData {
        let promise = stream(lua, &source(), 48_000, 1, chunk, capacity).unwrap();
        lua.globals().set("pending", promise).unwrap();
        lua.load("pending:andThen(function(value) controller = value end)")
            .exec()
            .unwrap();
        poll(lua, usize::MAX).unwrap();
        lua.globals().get("controller").unwrap()
    }
    #[test]
    fn upstream_audio_samples_vm_drop_cancels_decode_and_releases_pending_stream() {
        let owner = lua();
        let registry = registry(&owner).unwrap();
        drop(to_blob(&owner, &source(), 48_000, 1).unwrap());
        drop(stream(&owner, &source(), 48_000, 1, 1, 1).unwrap());
        let stop = registry
            .pending
            .borrow()
            .values()
            .next()
            .unwrap()
            .stop
            .clone();
        let host = registry.streams.borrow()[0].clone();
        assert_eq!(registry.waits.borrow().len(), 1);
        drop(owner);
        assert!(registry.pending.borrow().is_empty());
        assert!(registry.waits.borrow().is_empty());
        assert!(registry.streams.borrow().is_empty());
        assert!(stop.load(Ordering::Relaxed));
        assert!(!host.borrow().live);
        assert!(!host.borrow().wake);
    }
    #[test]
    fn upstream_audio_samples_blob_decodes_async_and_preserves_pcm() {
        let lua = lua();
        let promise = to_blob(&lua, &source(), 48_000, 1).unwrap();
        lua.globals().set("pending", promise).unwrap();
        lua.load("pending:andThen(function(value) blob = value end)")
            .exec()
            .unwrap();
        settle(&lua);
        let actual: Vec<f32> = lua.load("local b = blob:toBuffer(); return {buffer.readf32(b,0),buffer.readf32(b,4),buffer.readf32(b,8),buffer.readf32(b,12)}").eval().unwrap();
        assert_eq!(actual, [0.25, 0.5, 0.75, 1.0]);
        shutdown(&lua);
    }
    #[test]
    fn upstream_audio_samples_cancel_removes_decode_and_never_resolves() {
        let lua = lua();
        lua.globals()
            .set("pending", to_blob(&lua, &source(), 48_000, 1).unwrap())
            .unwrap();
        lua.load("called = false; pending:andThen(function() called=true end); pending:cancel()")
            .exec()
            .unwrap();
        assert!(registry(&lua).unwrap().pending.borrow().is_empty());
        settle(&lua);
        assert!(!lua.globals().get::<bool>("called").unwrap());
        assert_eq!(
            lua.load("return pending:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Cancelled"
        );
    }
    #[test]
    fn upstream_audio_samples_offline_poll_joins_decode() {
        let _guard = super::super::lua_audio::AUDIO_TEST_LOCK.lock().unwrap();
        use super::super::luau_audio_processor_engine::{ProcessorFormat, set_render_offline};
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                set_render_offline(false, ProcessorFormat::default());
            }
        }
        let _reset = Reset;
        set_render_offline(true, ProcessorFormat::default());
        let lua = lua();
        lua.globals()
            .set("pending", to_blob(&lua, &source(), 48_000, 1).unwrap())
            .unwrap();
        lua.load("pending:andThen(function(value) size=value.size end)")
            .exec()
            .unwrap();
        // No global WorkPool polling: offline owner settlement joins its job.
        assert!(poll(&lua, usize::MAX).unwrap());
        assert_eq!(lua.globals().get::<usize>("size").unwrap(), 16);
        shutdown(&lua);
    }
    #[test]
    fn upstream_audio_samples_select_wait_then_pump_then_deliver() {
        let lua = lua();
        let promise = stream(&lua, &source(), 48_000, 1, 1, 1).unwrap();
        lua.globals().set("pending", promise).unwrap();
        // Consume the initial record before the promise is delivered. The callback
        // must remain pending until the next poll selects that refill.
        let host = registry(&lua).unwrap().streams.borrow()[0].clone();
        let mut reader = super::super::luau_audio_samples::PcmStreamReader::new(
            host.borrow().descriptor.clone(),
        )
        .unwrap();
        let mut bytes = [0; 4];
        assert_eq!(reader.read_into_bytes(&mut bytes).unwrap(), 1);
        lua.load("pending:andThen(function(stream) local reader = stream:reader(); observed = reader:readInto(buffer.create(4)); controller=stream end)").exec().unwrap();
        poll(&lua, usize::MAX).unwrap();
        assert_eq!(
            lua.load("return pending:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Pending"
        );
        assert!(pending_sequence(&lua).is_some());
        poll(&lua, usize::MAX).unwrap();
        assert_eq!(lua.globals().get::<usize>("observed").unwrap(), 1);
        poll(&lua, usize::MAX).unwrap();
        assert_eq!(reader.read_into_bytes(&mut bytes).unwrap(), 1);
        assert_eq!(f32::from_le_bytes(bytes), 0.75);
        shutdown(&lua);
    }
    #[test]
    fn upstream_audio_samples_seek_invalidates_unread_prefetch() {
        let lua = lua();
        let value = controller(&lua, 2, 4);
        let stream = value
            .borrow::<StreamController>()
            .unwrap()
            .stream()
            .unwrap();
        let mut reader = super::super::luau_audio_samples::PcmStreamReader::new(
            stream.borrow().descriptor.clone(),
        )
        .unwrap();
        let mut bytes = [0; 4];
        assert_eq!(reader.read_into_bytes(&mut bytes).unwrap(), 1);
        assert_eq!(f32::from_le_bytes(bytes), 0.25);
        assert!(stream.borrow_mut().restart(2).unwrap());
        assert_eq!(reader.read_into_bytes(&mut bytes).unwrap(), 1);
        assert_eq!(f32::from_le_bytes(bytes), 0.75);
        assert_eq!(reader.frame(), 3.0);
        shutdown(&lua);
    }
    #[test]
    fn upstream_audio_samples_release_pumps_retained_reader_then_reclaims() {
        let lua = lua();
        let value = controller(&lua, 1, 1);
        let stream = value
            .borrow::<StreamController>()
            .unwrap()
            .stream()
            .unwrap();
        let descriptor = stream.borrow().descriptor.clone();
        value.borrow::<StreamController>().unwrap().release();
        assert!(
            value
                .borrow::<StreamController>()
                .unwrap()
                .stream()
                .is_err()
        );
        poll(&lua, usize::MAX).unwrap();
        assert!(stream.borrow().live);
        drop(descriptor);
        poll(&lua, usize::MAX).unwrap();
        assert!(!stream.borrow().live);
    }
    #[test]
    fn upstream_audio_samples_loop_rewinds_without_empty_source_spin() {
        let lua = lua();
        let value = controller(&lua, 2, 2);
        let stream = value
            .borrow::<StreamController>()
            .unwrap()
            .stream()
            .unwrap();
        stream.borrow_mut().set_loop(true).unwrap();
        let mut reader = super::super::luau_audio_samples::PcmStreamReader::new(
            stream.borrow().descriptor.clone(),
        )
        .unwrap();
        let mut bytes = [0; 8];
        for expected in [[0.25, 0.5], [0.75, 1.0], [0.25, 0.5]] {
            assert_eq!(reader.read_into_bytes(&mut bytes).unwrap(), 2);
            assert_eq!(
                [
                    f32::from_le_bytes(bytes[..4].try_into().unwrap()),
                    f32::from_le_bytes(bytes[4..].try_into().unwrap())
                ],
                expected
            );
            poll(&lua, usize::MAX).unwrap();
        }
        shutdown(&lua);
        let empty = AudioSource::from_buffered(Vec::new(), 1, 48_000).unwrap();
        super::stream(&lua, &empty, 48_000, 1, 1, 1).unwrap();
        let host = registry(&lua).unwrap().streams.borrow()[0].clone();
        host.borrow_mut().set_loop(true).unwrap();
        // The first terminal record already fills this one-record ring. Loop
        // retries the empty source once, queues another LAST, then yields.
        assert!(!host.borrow().ended);
        assert!(host.borrow().queued && host.borrow().queued_last);
        let mut reader = super::super::luau_audio_samples::PcmStreamReader::new(
            host.borrow().descriptor.clone(),
        )
        .unwrap();
        assert_eq!(reader.read_into_bytes(&mut [0; 4]).unwrap(), 0);
        assert!(reader.ended());
        host.borrow_mut().pump().unwrap();
        assert!(host.borrow().ended);
        shutdown(&lua);
    }
}
