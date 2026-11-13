//! Isolated Luau audio lanes. Only the processor's own VM crosses threads:
//! no facade handle, host callback capture, registry TLS or scene object
//! does. The processor runs on the safe VM (`safe.rs`).
#[cfg(target_arch = "wasm32")]
use super::luau_audio_web::Instant;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
mod browser_kernel;
#[cfg(target_arch = "wasm32")]
pub(super) use browser_kernel::BrowserKernel;
mod safe;
use safe::SafeProcessor as Processor;

pub(super) use super::luau_audio_shared::{
    SharedBlob, SharedRegion, SharedRing, SharedSnapshot, SharedValue,
};
use nuxie_runtime::source::r#async::work_pool::{vm_completions_queued, vm_completions_taken};
use nuxie_runtime::{AudioEngine, AudioRenderer, AudioRendererNode};

/// Interleaved processor output format. Offline hosts choose all three fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessorFormat {
    pub sample_rate: u32,
    pub channels: u32,
    pub max_frames: u32,
}
impl Default for ProcessorFormat {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            channels: 2,
            max_frames: 1024,
        }
    }
}
#[derive(Clone, Copy)]
struct OfflineRender {
    on: bool,
    format: ProcessorFormat,
    frame: u64,
}
static OFFLINE_RENDER: Mutex<OfflineRender> = Mutex::new(OfflineRender {
    on: false,
    format: ProcessorFormat {
        sample_rate: 48_000,
        channels: 2,
        max_frames: 1024,
    },
    frame: 0,
});
fn offline_state() -> OfflineRender {
    *lock(&OFFLINE_RENDER)
}
/// Like upstream, configuring either enabled or disabled mode resets its clock.
pub(super) fn set_render_offline(on: bool, format: ProcessorFormat) {
    *lock(&OFFLINE_RENDER) = OfflineRender {
        on,
        format: ProcessorFormat {
            sample_rate: format.sample_rate.max(1),
            channels: format.channels.max(1),
            max_frames: format.max_frames.max(1),
        },
        frame: 0,
    };
}
pub(super) fn render_offline() -> bool {
    offline_state().on
}
pub(super) fn set_offline_frame(frame: u64) {
    lock(&OFFLINE_RENDER).frame = frame;
}
pub(super) fn offline_clock() -> Option<(u64, u32)> {
    let state = offline_state();
    state.on.then_some((state.frame, state.format.sample_rate))
}

// Resolve the same target for format queries and processor startup. Queries
// return zero when startup would refuse a real-time lane.
struct ProcessorTarget {
    offline: bool,
    format: ProcessorFormat,
    plays: bool,
}
fn processor_target(engine: Option<&AudioEngine>) -> ProcessorTarget {
    let offline = offline_state();
    if offline.on {
        return ProcessorTarget {
            offline: true,
            format: offline.format,
            plays: true,
        };
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = engine;
        let format = super::luau_audio_web::request(serde_json::json!({"op":"processor.format"}))
            .ok()
            .and_then(|value| browser_processor_format(&value).ok());
        ProcessorTarget {
            offline: false,
            plays: format.is_some(),
            format: format.unwrap_or_default(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    ProcessorTarget {
        offline: false,
        plays: engine.is_some(),
        format: engine
            .map(|engine| ProcessorFormat {
                sample_rate: engine.sample_rate(),
                channels: engine.channels(),
                ..ProcessorFormat::default()
            })
            .unwrap_or_default(),
    }
}
#[cfg(target_arch = "wasm32")]
fn browser_processor_format(value: &serde_json::Value) -> Result<ProcessorFormat, String> {
    let field = |name: &str| {
        value[name]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| *n > 0)
            .ok_or_else(|| format!("invalid browser processor {name}"))
    };
    Ok(ProcessorFormat {
        sample_rate: field("sampleRate")?,
        channels: field("channels")?,
        max_frames: field("maxFrames")?,
    })
}
pub(super) fn processor_format(engine: Option<&AudioEngine>) -> ProcessorFormat {
    let target = processor_target(engine);
    if target.plays {
        target.format
    } else {
        ProcessorFormat {
            sample_rate: 0,
            channels: 0,
            max_frames: 0,
        }
    }
}

const STARTING: u8 = 0;
const RUNNING: u8 = 1;
const STOPPED: u8 = 2;
const FAILED: u8 = 3;
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// The largest char boundary of `value` at or below `maximum`.
fn char_floor(value: &str, maximum: usize) -> usize {
    (0..=maximum.min(value.len()))
        .rev()
        .find(|end| value.is_char_boundary(*end))
        .unwrap_or(0)
}
fn truncate(value: &mut String, maximum: usize) {
    let end = char_floor(value, maximum);
    value.truncate(end);
}
// Native upstream lane console: 8 KiB, 2-byte record length, 1024-byte lines.
// One serialized VM writer never waits for the control-thread reader.
const LOG_BYTES: usize = 8192;
const LOG_LINE_BYTES: usize = 1024;
struct LogRing {
    bytes: Box<[AtomicU8]>,
    write: AtomicUsize,
    read: AtomicUsize,
    dropped: AtomicUsize,
    reader: Mutex<()>,
}
impl LogRing {
    fn new() -> Self {
        Self {
            bytes: (0..LOG_BYTES).map(|_| AtomicU8::new(0)).collect(),
            write: AtomicUsize::new(0),
            read: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
            reader: Mutex::new(()),
        }
    }
    /// The ring's bytes from the free-running position `at` on, wrapping.
    fn from(&self, at: usize) -> impl Iterator<Item = &AtomicU8> {
        let at = at % LOG_BYTES;
        let (front, back) = self.bytes.split_at_checked(at).unwrap_or_default();
        back.iter().chain(front)
    }
    fn push(&self, line: &str) {
        let write = self.write.load(Ordering::Relaxed);
        let read = self.read.load(Ordering::Acquire);
        let line = line
            .as_bytes()
            .get(..char_floor(line, LOG_LINE_BYTES))
            .unwrap_or_default();
        // At most LOG_LINE_BYTES, so both fit in u16 and usize.
        let length = line.len() as u16;
        let record = line.len().saturating_add(2);
        if record > LOG_BYTES.saturating_sub(write.wrapping_sub(read)) {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        for (slot, byte) in self
            .from(write)
            .zip(length.to_le_bytes().iter().chain(line))
        {
            slot.store(*byte, Ordering::Relaxed);
        }
        self.write.store(write.wrapping_add(record), Ordering::Release);
    }
    fn has_lines(&self) -> bool {
        self.write.load(Ordering::Acquire) != self.read.load(Ordering::Acquire)
            || self.dropped.load(Ordering::Acquire) != 0
    }
    fn take(&self) -> Vec<String> {
        let _reader = lock(&self.reader);
        let mut read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);
        let mut result = Vec::new();
        while read != write {
            let mut bytes = self.from(read).map(|byte| byte.load(Ordering::Relaxed));
            let length = u16::from_le_bytes([
                bytes.next().unwrap_or_default(),
                bytes.next().unwrap_or_default(),
            ]);
            let line: Vec<u8> = bytes.take(usize::from(length)).collect();
            result.push(String::from_utf8_lossy(&line).into_owned());
            read = read.wrapping_add(usize::from(length).saturating_add(2));
        }
        self.read.store(read, Ordering::Release);
        let dropped = self.dropped.swap(0, Ordering::AcqRel);
        if dropped != 0 {
            result.push(format!("processor dropped {dropped} console lines: it wrote more than 8 KB between two handovers"));
        }
        result
    }
}

struct Shared {
    state: AtomicU8,
    cancelled: AtomicBool,
    error: OnceLock<String>,
    logs: LogRing,
    pending: AtomicBool,
    failure_reported: AtomicBool,
    retired: AtomicBool,
}
impl Shared {
    fn new(state: u8) -> Self {
        let result = Self {
            state: AtomicU8::new(state),
            cancelled: AtomicBool::new(false),
            error: OnceLock::new(),
            logs: LogRing::new(),
            pending: AtomicBool::new(false),
            failure_reported: AtomicBool::new(false),
            retired: AtomicBool::new(false),
        };
        if state == STARTING {
            result.notify();
        }
        result
    }
    fn notify(&self) {
        if self.retired.load(Ordering::Acquire) {
            return;
        }
        // Count first: a simultaneous poll must never decrement before the
        // corresponding increment. Undo the speculative count if coalesced.
        vm_completions_queued(1);
        if self.pending.swap(true, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
        // Retirement may have raced the early check and publication. Whichever
        // side clears the bit owns the matching decrement.
        if self.retired.load(Ordering::Acquire) && self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
    }
    fn retire(&self) {
        self.retired.store(true, Ordering::Release);
        if self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
    }
    fn poll_notification(&self) -> bool {
        if self.state.load(Ordering::Acquire) == STARTING {
            // Keep the root awake without reporting an undelivered transition.
            return false;
        }
        if self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
            true
        } else {
            false
        }
    }

    fn fail(&self, mut error: String) {
        truncate(&mut error, 1024);
        let _ = self.error.set(error);
        let _ = self
            .state
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |state| {
                (state < STOPPED).then_some(FAILED)
            });
        self.notify();
    }
}
impl Drop for Shared {
    fn drop(&mut self) {
        if self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
    }
}
struct Owner {
    shared: Arc<Shared>,
    engine: Option<AudioEngine>,
    node: Mutex<Option<AudioRendererNode>>,
    offline_vm: Mutex<Option<Processor>>,
    offline: bool,
    #[cfg(target_arch = "wasm32")]
    web_id: Option<u32>,
    format: ProcessorFormat,
}
impl Owner {
    fn stop(&self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(id) = self.web_id {
            if let Err(error) =
                super::luau_audio_web::request(serde_json::json!({"op":"processor.stop","id":id}))
            {
                self.shared.fail(error);
            }
        }
        self.shared.cancelled.store(true, Ordering::Release);
        let transitioned =
            self.shared
                .state
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |s| {
                    (s < STOPPED).then_some(STOPPED)
                });
        if transitioned.is_ok() {
            self.shared.notify();
        }
        if let Some(node) = lock(&self.node).take() {
            if let Some(engine) = self.engine.as_ref() {
                engine.detach_renderer(node);
            }
        }
        // Wait for an in-flight host pull before dropping its VM, as native
        // detachRenderer does for a device callback.
        lock(&self.offline_vm).take();
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.stop();
        #[cfg(target_arch = "wasm32")]
        if let Some(id) = self.web_id {
            WEB_OWNERS.with(|owners| {
                owners.borrow_mut().remove(&id);
            });
            let _ = super::luau_audio_web::request(
                serde_json::json!({"op":"processor.release","id":id}),
            );
        }
    }
}
#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_OWNERS: std::cell::RefCell<BTreeMap<u32, std::sync::Weak<Owner>>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}
#[cfg(target_arch = "wasm32")]
pub(super) fn browser_notify(id: u32) {
    let owner =
        WEB_OWNERS.with(|owners| owners.borrow().get(&id).and_then(std::sync::Weak::upgrade));
    if let Some(owner) = owner {
        owner.shared.notify();
    }
}
#[derive(Clone)]
pub struct ProcessorHandle(Arc<Owner>);
impl ProcessorHandle {
    pub(super) fn start(
        engine: Option<AudioEngine>,
        name: String,
        modules: BTreeMap<String, Arc<[u8]>>,
        input: SharedValue,
        hard_blocks: u32,
        memory_limit: usize,
    ) -> Result<Self, String> {
        if !modules.contains_key(&name) {
            return Err(format!("unknown audio processor module {name}"));
        }
        let target = processor_target(engine.as_ref());
        // Offline lanes have no engine attachment or block deadline.
        let engine = if target.offline { None } else { engine };
        let format = target.format;
        let hard_blocks = if target.offline { 0 } else { hard_blocks };
        #[cfg(target_arch = "wasm32")]
        if !target.offline {
            let encoded = super::luau_audio_shared::encode_wire(&input)?;
            let module_bytes: BTreeMap<_, _> = modules
                .iter()
                .map(|(name, bytes)| (name, bytes.as_ref()))
                .collect();
            let response = super::luau_audio_web::request(serde_json::json!({
                "op":"processor.start", "name":name, "modules":module_bytes, "input":encoded,
                "format":{"sampleRate":format.sample_rate,"channels":format.channels,"maxFrames":format.max_frames},
                "hardBlocks":hard_blocks,"memoryLimit":memory_limit
            }))?;
            let id = response
                .get("id")
                .and_then(serde_json::Value::as_u64)
                .and_then(|id| u32::try_from(id).ok())
                .ok_or("invalid browser processor id")?;
            let format = browser_processor_format(
                response.get("format").unwrap_or(&serde_json::Value::Null),
            )?;
            let owner = Arc::new(Owner {
                shared: Arc::new(Shared::new(STARTING)),
                engine: None,
                node: Mutex::new(None),
                offline_vm: Mutex::new(None),
                offline: false,
                format,
                web_id: Some(id),
            });
            WEB_OWNERS.with(|owners| {
                owners.borrow_mut().insert(id, Arc::downgrade(&owner));
            });
            return Ok(Self(owner));
        }
        let shared = Arc::new(Shared::new(STARTING));
        let owner = Arc::new(Owner {
            shared: shared.clone(),
            engine: engine.clone(),
            node: Mutex::new(None),
            offline_vm: Mutex::new(None),
            offline: target.offline,
            #[cfg(target_arch = "wasm32")]
            web_id: None,
            format,
        });
        if engine.is_none() && !target.offline {
            shared.fail("real time audio has no audio engine".into());
            return Ok(Self(owner));
        }
        let weak = Arc::downgrade(&owner);
        submit_initialization(Box::new(move || {
            if shared.cancelled.load(Ordering::Acquire) {
                return;
            }
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Processor::new(
                    shared.clone(),
                    name,
                    modules,
                    input,
                    format.sample_rate,
                    format.channels,
                    format.max_frames,
                    hard_blocks,
                    memory_limit,
                )
            }));
            match result {
                Ok(Ok(vm)) => {
                    if let Some(owner) = weak.upgrade() {
                        let mut node = lock(&owner.node);
                        if shared.state.load(Ordering::Acquire) == STARTING {
                            if let Some(engine) = engine.as_ref() {
                                *node = Some(engine.attach_renderer(vm));
                            } else {
                                *lock(&owner.offline_vm) = Some(vm);
                            }
                            let _ = shared.state.compare_exchange(
                                STARTING,
                                RUNNING,
                                Ordering::AcqRel,
                                Ordering::Acquire,
                            );
                            shared.notify();
                        }
                    }
                }
                Ok(Err(error)) => shared.fail(error),
                Err(_) => shared.fail("audio processor initialization panicked".into()),
            }
        }))?;
        Ok(Self(owner))
    }
    /// Format captured when this lane started; later host configuration changes
    /// do not change an already initialized processor.
    pub fn format(&self) -> ProcessorFormat {
        self.0.format
    }
    pub fn is_offline(&self) -> bool {
        self.0.offline
    }
    /// Pull this offline lane directly, without a device or mixer attachment.
    /// A starting/stopped/failed lane produces silence. Hosts wait for running
    /// when they need initialization to finish, and advance the shared offline
    /// clock explicitly after pulling all lanes for a block.
    pub fn render_offline(
        &self,
        output: &mut [f32],
        frames: u32,
        start_frame: u64,
    ) -> Result<(), String> {
        if !self.0.offline {
            return Err("processor belongs to the realtime audio engine".into());
        }
        let expected = (frames as usize)
            .checked_mul(self.0.format.channels as usize)
            .ok_or("offline audio output size overflow")?;
        if output.len() != expected {
            return Err("offline audio output does not match processor format".into());
        }
        output.fill(0.0);
        if let Some(vm) = lock(&self.0.offline_vm).as_mut() {
            vm.render(output, frames, start_frame);
        }
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    fn sync_web(&self) {
        let Some(id) = self.0.web_id else {
            return;
        };
        let result =
            super::luau_audio_web::request(serde_json::json!({"op":"processor.poll","id":id}));
        match result {
            Ok(value) => {
                if let Some(lines) = value.get("logs").and_then(serde_json::Value::as_array) {
                    for line in lines {
                        if let Some(line) = line.as_str() {
                            self.0.shared.logs.push(line);
                        }
                    }
                }
                match value.get("state").and_then(serde_json::Value::as_str) {
                    Some("running") => {
                        let _ = self.0.shared.state.compare_exchange(
                            STARTING,
                            RUNNING,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        );
                    }
                    Some("failed") if self.0.shared.state.load(Ordering::Acquire) < STOPPED => {
                        self.0.shared.fail(
                            value
                                .get("error")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("browser audio processor failed")
                                .into(),
                        )
                    }
                    Some("stopped") => {
                        let _ = self.0.shared.state.fetch_update(
                            Ordering::AcqRel,
                            Ordering::Acquire,
                            |s| (s < STOPPED).then_some(STOPPED),
                        );
                    }
                    _ => {}
                }
            }
            Err(error) if self.0.shared.state.load(Ordering::Acquire) < STOPPED => {
                self.0.shared.fail(error)
            }
            Err(_) => {}
        }
    }
    pub fn state(&self) -> &'static str {
        #[cfg(target_arch = "wasm32")]
        self.sync_web();
        match self.0.shared.state.load(Ordering::Acquire) {
            STARTING => "starting",
            RUNNING => "running",
            STOPPED => "stopped",
            _ => "failed",
        }
    }
    pub fn error(&self) -> Option<String> {
        self.0.shared.error.get().cloned()
    }
    pub(super) fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
    pub(super) fn has_logs(&self) -> bool {
        self.0.shared.logs.has_lines()
    }
    pub(super) fn has_pending_work(&self) -> bool {
        !self.0.shared.retired.load(Ordering::Acquire)
            && (self.state() == "starting" || self.0.shared.pending.load(Ordering::Acquire))
    }
    pub(super) fn abandon_notifications(&self) {
        self.0.shared.retire();
    }
    pub(super) fn poll_notification(&self) -> bool {
        self.0.shared.poll_notification()
    }
    pub(super) fn take_failure(&self) -> Option<String> {
        let error = self.0.shared.error.get()?;
        if self.0.shared.failure_reported.swap(true, Ordering::AcqRel) {
            None
        } else {
            Some(error.clone())
        }
    }
    pub(super) fn is_unreferenced_terminal(&self) -> bool {
        self.strong_count() == 1
            && matches!(self.state(), "stopped" | "failed")
            && !self.has_pending_work()
            && !self.has_logs()
            && (self.0.shared.error.get().is_none()
                || self.0.shared.failure_reported.load(Ordering::Acquire))
    }
    pub(super) fn take_logs(&self) -> Vec<String> {
        self.0.shared.logs.take()
    }
    pub fn stop(&self) {
        self.0.stop();
    }
}

type InitJob = Box<dyn FnOnce() + Send>;
#[cfg(not(target_arch = "wasm32"))]
fn submit_initialization(job: InitJob) -> Result<(), String> {
    static QUEUE: OnceLock<Result<std::sync::mpsc::Sender<InitJob>, String>> = OnceLock::new();
    let sender = QUEUE
        .get_or_init(|| {
            let (sender, receiver) = std::sync::mpsc::channel::<InitJob>();
            std::thread::Builder::new()
                .name("luau-audio-init".into())
                .spawn(move || {
                    while let Ok(job) = receiver.recv() {
                        job();
                    }
                })
                .map_err(|error| format!("cannot initialize audio processor worker: {error}"))?;
            Ok(sender)
        })
        .as_ref()
        .map_err(Clone::clone)?;
    sender
        .send(job)
        .map_err(|_| "audio processor initialization worker stopped".into())
}

#[cfg(target_arch = "wasm32")]
fn submit_initialization(job: InitJob) -> Result<(), String> {
    // Browser realtime init is a worklet message, never process(). Offline
    // hosts explicitly pull on their own thread and may initialize synchronously.
    job();
    Ok(())
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    fn compile(source: &str) -> Arc<[u8]> {
        luaur_common::set_all_flags(true);
        let mut size = 0;
        let bytes = luaur_compiler::functions::luau_compile::luau_compile(
            source.as_ptr().cast(),
            source.len(),
            std::ptr::null_mut(),
            &mut size,
        );
        assert!(!bytes.is_null());
        let result: Arc<[u8]> = unsafe { std::slice::from_raw_parts(bytes.cast(), size) }.into();
        unsafe extern "C" {
            fn free(pointer: *mut std::ffi::c_void);
        }
        unsafe {
            free(bytes.cast());
        }
        result
    }
    fn shared() -> Arc<Shared> {
        Arc::new(Shared::new(RUNNING))
    }
    fn processor(source: &str, input: SharedValue) -> Processor {
        let mut vm = Processor::new(
            shared(),
            "processor".into(),
            BTreeMap::from([("processor".into(), compile(source))]),
            input,
            48_000,
            2,
            1024,
            2,
            16 * 1024 * 1024,
        )
        .unwrap();
        // Native upstream gives the first block an extended warm-up budget.
        vm.warm = true;
        vm
    }
    #[test]
    fn pcm_stream_reader_runs_in_isolated_luau_with_actual_format() {
        let descriptor = super::super::luau_audio_samples::PcmStreamDescriptor::new(
            128,
            48_000,
            2,
            2,
            Arc::new(super::super::luau_audio_shared::SharedBudget::default()),
        )
        .unwrap();
        let mut record = Vec::new();
        record.extend(0u32.to_le_bytes());
        record.extend(0u32.to_le_bytes());
        record.extend(5f64.to_le_bytes());
        for value in [0.25f32, 0.5, 0.75, 1.0] {
            record.extend(value.to_le_bytes());
        }
        descriptor.ring.write_record(&record).unwrap();
        let mut vm = processor(
            r#"return {
            init=function(self,rate,maxFrames,channels,input)
                assert(Audio.processorSampleRate()==rate and Audio.processorChannels()==channels)
                self.reader=Audio.streamReader(input)
            end,
            process=function(self,out,frames,startFrame)
                assert(self.reader:readInto(out)==2)
                assert(self.reader.frame==7 and not self.reader.ended)
            end
        }"#,
            descriptor.shared_value(),
        );
        let mut output = [0.0f32; 4];
        vm.render(&mut output, 2, 0);
        assert_eq!(output, [0.25, 0.5, 0.75, 1.0]);
        assert_eq!(descriptor.reader_count().unwrap(), 1);
        drop(vm);
        assert_eq!(descriptor.reader_count().unwrap(), 0);
    }
    #[test]
    fn authored_pcm_and_shared_controls_survive_callback_thread_migration() {
        let messages = Arc::new(SharedRing::new(4096).unwrap());
        let meter = Arc::new(SharedSnapshot::new(4096).unwrap());
        let mut vm = processor(
            r#"
            return {
                init=function(self,rate,maxFrames,channels,input)
                    assert(rate==48000 and maxFrames==1024 and channels==2)
                    self.link=input; self.gain=0; self.next={gain=0}
                end,
                process=function(self,out,frames,start)
                    while self.link.messages:receiveInto(self.next) do self.gain=self.next.gain end
                    for i=0,frames-1 do
                        buffer.writef32(out,i*8,self.gain)
                        buffer.writef32(out,i*8+4,-self.gain)
                    end
                    self.link.meter:publish({frames=frames,start=start,gain=self.gain})
                end,
            }
        "#,
            SharedValue::Object(BTreeMap::from([
                ("messages".into(), SharedValue::Ring(messages.clone())),
                ("meter".into(), SharedValue::Snapshot(meter.clone())),
            ])),
        );
        assert!(
            messages
                .send(SharedValue::Object(BTreeMap::from([(
                    "gain".into(),
                    SharedValue::Number(0.25)
                )])))
                .unwrap()
        );
        vm = std::thread::spawn(move || {
            let mut out = [0.0; 128];
            vm.render(&mut out, 64, 123);
            assert!(out.chunks_exact(2).all(|p| p == [0.25, -0.25]));
            vm
        })
        .join()
        .unwrap();
        assert!(
            messages
                .send(SharedValue::Object(BTreeMap::from([(
                    "gain".into(),
                    SharedValue::Number(0.5)
                )])))
                .unwrap()
        );
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 187);
        assert!(out.chunks_exact(2).all(|p| p == [0.5, -0.5]));
        let (_, SharedValue::Object(values)) = meter.latest().unwrap() else {
            panic!("snapshot missing")
        };
        assert!(matches!(
            values.get("start"),
            Some(SharedValue::Number(187.0))
        ));
    }
    #[test]
    fn caught_execution_deadline_still_fails_only_its_processor() {
        let mut broken = processor(
            "return {process=function() pcall(function() while true do end end) end}",
            SharedValue::Nil,
        );
        let mut good = processor(
            "return {process=function(self,out,n) for i=0,n*2-1 do buffer.writef32(out,i*4,0.25) end end}",
            SharedValue::Nil,
        );
        let mut out = [1.0; 128];
        broken.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(broken.finished());
        assert!(
            broken
                .shared()
                .error
                .get()
                .unwrap()
                .contains("budget")
        );
        good.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.25));
        assert!(!good.finished());
    }
    #[test]
    fn failed_chunk_preserves_preceding_successful_chunk() {
        let mut vm = processor(
            "return {process=function(self,out,n,start) if start>=1024 then error('bad block') end for i=0,n*2-1 do buffer.writef32(out,i*4,0.125) end end}",
            SharedValue::Nil,
        );
        let mut out = vec![0.0; 4096];
        vm.render(&mut out, 2048, 0);
        assert!(out[..2048].iter().all(|v| *v == 0.125));
        assert!(out[2048..].iter().all(|v| *v == 0.0));
        assert!(vm.finished());
    }
    #[test]
    fn caught_memory_exhaustion_is_terminal() {
        let mut vm = processor(
            "return {process=function(self,out,n) pcall(function() buffer.create(32*1024*1024) end) end}",
            SharedValue::Nil,
        );
        let mut out = [1.0; 128];
        vm.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(vm.finished());
        assert!(vm.shared().error.get().unwrap().contains("memory"));
    }
    #[test]
    fn precompiled_dependency_is_loaded_and_cached_in_isolated_vm() {
        let vm=Processor::new(shared(),"processor".into(),BTreeMap::from([
            ("processor".into(),compile("local a=require('helper');local b=require('helper');assert(a==b);return {process=function(self,out,n)for i=0,n*2-1 do buffer.writef32(out,i*4,a.value)end end}")),
            ("helper".into(),compile("return {value=0.25}")),
        ]),SharedValue::Nil,48000,2,1024,2,16*1024*1024).unwrap();
        let mut vm = vm;
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.25));
    }
    #[test]
    fn receive_into_clears_previous_fields_and_array_tail() {
        let ring = Arc::new(SharedRing::new(4096).unwrap());
        let mut vm = processor(
            r#"
            return {
                init=function(self,rate,maxFrames,channels,input)
                    self.ring=input;self.next={obsolete=99,[1]=10,[2]=20};self.calls=0
                end,
                process=function(self,out,n,start)
                    assert(self.ring:receiveInto(self.next))
                    assert(self.next.obsolete==nil and self.next[1]==nil and self.next[2]==nil)
                    if self.calls==0 then assert(self.next.fresh==7) else assert(self.next.fresh==nil) end
                    self.calls+=1
                end,
            }
        "#,
            SharedValue::Ring(ring.clone()),
        );
        assert!(
            ring.send(SharedValue::Object(BTreeMap::from([(
                "fresh".into(),
                SharedValue::Number(7.0)
            )])))
            .unwrap()
        );
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 0);
        assert!(!vm.finished(), "{:?}", vm.shared().error.get());
        assert!(ring.send(SharedValue::Object(BTreeMap::new())).unwrap());
        vm.render(&mut out, 64, 64);
        assert!(!vm.finished(), "{:?}", vm.shared().error.get());
    }
    #[test]
    fn processor_message_past_the_value_limit_is_a_script_error() {
        // A 2048-slot table reached 2049 times is 4,198,402 values once
        // copied; the processor VM holds it in about 64 KiB.
        let ring = Arc::new(SharedRing::new(16).unwrap());
        let mut vm = Processor::new(
            shared(),
            "processor".into(),
            BTreeMap::from([(
                "processor".into(),
                compile(
                    r#"
            return {
                init=function(self,rate,maxFrames,channels,input) self.ring=input end,
                process=function(self,out,n,start)
                    local leaf = {}
                    for i = 1, 2048 do leaf[i] = true end
                    local root = {}
                    for i = 1, 2049 do root[i] = leaf end
                    local ok, err = pcall(self.ring.send, self.ring, root)
                    assert(not ok and string.find(tostring(err), "4194304 values"), tostring(err))
                    assert(self.ring:send({leaf}) == false)
                end,
            }
        "#,
                ),
            )]),
            SharedValue::Ring(ring.clone()),
            48_000,
            2,
            1024,
            // No deadline: the refused copy takes host time, not VM time.
            0,
            16 * 1024 * 1024,
        )
        .unwrap();
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 0);
        assert!(!vm.finished(), "{:?}", vm.shared().error.get());
    }
    #[test]
    fn bounded_logs_drop_whole_lines_and_failure_keeps_first() {
        let shared = Shared::new(RUNNING);
        let line = "💛".repeat(300);
        for _ in 0..20 {
            shared.logs.push(&line);
        }
        let logs = shared.logs.take();
        assert_eq!(logs.len(), 8);
        for text in &logs[..7] {
            assert_eq!(text.len(), 1024);
            assert_eq!(text, "💛".repeat(256).as_str());
        }
        assert!(logs[7].contains("dropped 13 console lines"));
        assert!(!shared.logs.has_lines());
        shared.fail(line.clone());
        shared.fail("second error".into());
        assert_eq!(shared.error.get().unwrap(), &"💛".repeat(256));
        assert!(shared.pending.load(Ordering::Acquire));
        assert!(shared.poll_notification());
        assert!(!shared.poll_notification());
    }
    #[test]
    fn pending_initialization_is_not_acknowledged_until_state_changes() {
        let shared = Shared::new(STARTING);
        assert!(!shared.poll_notification());
        assert!(shared.pending.load(Ordering::Acquire));
        shared.state.store(RUNNING, Ordering::Release);
        shared.notify();
        assert!(shared.poll_notification());
        assert!(!shared.pending.load(Ordering::Acquire));
    }
    #[test]
    fn retired_owner_cannot_leave_a_notification_for_a_dropped_vm() {
        let shared = Arc::new(Shared::new(STARTING));
        let worker = shared.clone();
        let background = std::thread::spawn(move || {
            for _ in 0..10000 {
                worker.notify();
            }
        });
        shared.retire();
        background.join().unwrap();
        assert!(!shared.pending.load(Ordering::Acquire));
        shared.state.store(RUNNING, Ordering::Release);
        shared.notify();
        shared.fail("late init failure".into());
        assert!(!shared.pending.load(Ordering::Acquire));
    }
    struct ResetOffline;
    impl Drop for ResetOffline {
        fn drop(&mut self) {
            set_render_offline(false, ProcessorFormat::default());
        }
    }
    #[test]
    fn queried_processor_format_uses_startup_target() {
        let _guard = super::super::lua_audio::AUDIO_TEST_LOCK.lock().unwrap();
        let _reset = ResetOffline;
        set_render_offline(false, ProcessorFormat::default());
        assert_eq!(
            processor_format(None),
            ProcessorFormat {
                sample_rate: 0,
                channels: 0,
                max_frames: 0
            }
        );
        let engine = AudioEngine::new(1, 32000).unwrap();
        assert_eq!(
            processor_format(Some(&engine)),
            ProcessorFormat {
                sample_rate: 32000,
                channels: 1,
                max_frames: 1024
            }
        );
        let format = ProcessorFormat {
            sample_rate: 22050,
            channels: 2,
            max_frames: 17,
        };
        set_render_offline(true, format);
        assert_eq!(processor_format(Some(&engine)), format);
        assert_eq!(processor_format(None), format);
    }
    #[test]
    fn offline_host_format_clock_and_rendering_follow_upstream() {
        let _guard = super::super::lua_audio::AUDIO_TEST_LOCK.lock().unwrap();
        let _reset = ResetOffline;
        set_render_offline(
            true,
            ProcessorFormat {
                sample_rate: 22050,
                channels: 1,
                max_frames: 17,
            },
        );
        assert_eq!(offline_clock(), Some((0, 22050)));
        let module = compile(
            r#"return {
            init=function(self,rate,maxFrames,channels,input)
                assert(rate==22050 and maxFrames==17 and channels==1)
            end,
            process=function(self,out,n,start)
                for i=0,n-1 do buffer.writef32(out,i*4,(start+i)/1000) end
            end,
        }"#,
        );
        let handle = ProcessorHandle::start(
            None,
            "offline".into(),
            BTreeMap::from([("offline".into(), module)]),
            SharedValue::Nil,
            999,
            16 * 1024 * 1024,
        )
        .unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while handle.state() == "starting" && Instant::now() < end {
            std::thread::yield_now();
        }
        assert_eq!(handle.state(), "running", "{:?}", handle.error());
        assert!(handle.is_offline());
        assert!(handle.0.engine.is_none());
        assert_eq!(lock(&handle.0.offline_vm).as_ref().unwrap().hard_blocks, 0);
        let mut out = vec![0.0; 40];
        handle.render_offline(&mut out, 40, 50).unwrap();
        for (i, value) in out.iter().enumerate() {
            assert!((*value - (50 + i) as f32 / 1000.0).abs() < 1e-7);
        }
        // Pulls do not advance the main lane until the host has rendered every lane.
        assert_eq!(offline_clock(), Some((0, 22050)));
        set_offline_frame(22050);
        let vm = super::super::ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let (time, frame, rate): (f64, f64, u32) = vm
            .lua()
            .load("return Audio.time(),Audio.timeFrame(),Audio.sampleRate()")
            .eval()
            .unwrap();
        assert_eq!((time, frame, rate), (1.0, 22050.0, 22050));
        set_render_offline(
            true,
            ProcessorFormat {
                sample_rate: 0,
                channels: 0,
                max_frames: 0,
            },
        );
        assert_eq!(offline_clock(), Some((0, 1)));
        assert_eq!(
            offline_state().format,
            ProcessorFormat {
                sample_rate: 1,
                channels: 1,
                max_frames: 1
            }
        );
        assert_eq!(
            handle.format(),
            ProcessorFormat {
                sample_rate: 22050,
                channels: 1,
                max_frames: 17
            }
        );
        handle.stop();
        out.fill(1.0);
        handle.render_offline(&mut out, 40, 90).unwrap();
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(handle.render_offline(&mut out, 39, 90).is_err());
        set_render_offline(false, ProcessorFormat::default());
        assert_eq!(offline_clock(), None);
        assert_eq!(offline_state().frame, 0);
    }
    #[test]
    fn offline_block_has_no_deadline_even_after_warmup() {
        let _guard = super::super::lua_audio::AUDIO_TEST_LOCK.lock().unwrap();
        let _reset = ResetOffline;
        set_render_offline(
            true,
            ProcessorFormat {
                sample_rate: 48000,
                channels: 1,
                max_frames: 1,
            },
        );
        let module = compile(
            "return {process=function(self,out,n,start) local finish=os.clock()+0.01;while os.clock()<finish do end;buffer.writef32(out,0,0.25) end}",
        );
        let handle = ProcessorHandle::start(
            None,
            "offline".into(),
            BTreeMap::from([("offline".into(), module)]),
            SharedValue::Nil,
            1,
            16 * 1024 * 1024,
        )
        .unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while handle.state() == "starting" && Instant::now() < end {
            std::thread::yield_now();
        }
        assert_eq!(handle.state(), "running", "{:?}", handle.error());
        let mut out = [0.0];
        handle.render_offline(&mut out, 1, 0).unwrap();
        handle.render_offline(&mut out, 1, 1).unwrap();
        assert_eq!(out, [0.25]);
        assert_eq!(handle.state(), "running", "{:?}", handle.error());
        handle.stop();
    }
    #[test]
    fn processor_reads_blob_and_region_atomic_words() {
        let budget = Arc::new(super::super::luau_audio_shared::SharedBudget::default());
        let blob = Arc::new(SharedBlob::with_budget(&[7, 8, 9, 10], budget.clone()).unwrap());
        let region = Arc::new(SharedRegion::with_budget(8, budget).unwrap());
        let mut vm = processor(
            r#"return {
            init=function(self,rate,maxFrames,channels,input)
                self.blob=input.blob;self.region=input.region
                assert(self.blob.size==4 and self.region.size==8 and not self.region.poisoned)
                self.copy=self.blob:toBuffer();buffer.writeu8(self.copy,0,99)
                assert(self.blob:read(1,self.copy)==3)
                assert(buffer.readu8(self.copy,0)==8)
                assert(buffer.readu8(self.copy,1)==9)
                assert(buffer.readu8(self.copy,2)==10)
                self.region:atomicStore(0,12)
            end,
            process=function(self,out,n,start)
                assert(self.region:atomicAdd(0,3)==12)
                assert(self.region:atomicCas(0,15,21)==15)
                assert(self.region:atomicLoad(0)==21)
                for i=0,n*2-1 do buffer.writef32(out,i*4,0.25) end
            end,
        }"#,
            SharedValue::Object(BTreeMap::from([
                ("blob".into(), SharedValue::Blob(blob.clone())),
                ("region".into(), SharedValue::Region(region.clone())),
            ])),
        );
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 0);
        assert_eq!(
            vm.shared().state.load(Ordering::Acquire),
            RUNNING,
            "{:?}",
            vm.shared().error.get()
        );
        assert_eq!(region.atomic_load(0).unwrap(), 21);
        assert!(out.iter().all(|v| *v == 0.25));
        let mut original = [0; 4];
        assert_eq!(blob.read(0, &mut original), 4);
        assert_eq!(original, [7, 8, 9, 10]);
    }
    #[test]
    fn processor_cannot_catch_and_continue_after_region_lock_or_handle_transfer() {
        let budget = Arc::new(super::super::luau_audio_shared::SharedBudget::default());
        let region = Arc::new(SharedRegion::with_budget(8, budget).unwrap());
        for source in [
            "return {init=function(self,r,m,c,input)self.input=input end,process=function(self,out,n,start)pcall(function()self.input.region:tryExclusive(function()end)end)end}",
            "return {init=function(self,r,m,c,input)self.input=input end,process=function(self,out,n,start)pcall(function()self.input.ring:send({region=self.input.region})end)end}",
        ] {
            let ring = Arc::new(SharedRing::new(4096).unwrap());
            let mut vm = processor(
                source,
                SharedValue::Object(BTreeMap::from([
                    ("region".into(), SharedValue::Region(region.clone())),
                    ("ring".into(), SharedValue::Ring(ring.clone())),
                ])),
            );
            let mut out = [1.0; 128];
            vm.render(&mut out, 64, 0);
            assert!(vm.finished());
            assert!(out.iter().all(|v| *v == 0.0));
            assert!(vm.shared().error.get().unwrap().contains("processor"));
            assert!(ring.receive().is_none());
        }
    }
}
