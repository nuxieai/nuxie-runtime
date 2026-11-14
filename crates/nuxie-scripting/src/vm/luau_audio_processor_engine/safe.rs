//! The audio processor on the safe VM (`nuxie-luau-vm`)
//! (docs/luau-safe-vm-embedding-core.md, sections 0 and 5.7): a
//! `HostedVm<AudioHost>`, plain data that moves to the audio thread as one
//! value. The host functions are table entries; a channel's methods are
//! natives whose data is the channel's index (the deleted vendored
//! processor bound it as an upvalue). Everything the host keeps is a `Root`; the memory
//! budget and its sticky exhaustion flag are the core's.
//!
//! Behavior follows `raw.rs` line for line: the same globals, deadlines,
//! sticky failures and error texts.
use super::*;
use std::collections::BTreeSet;
use std::num::NonZeroU64;

use super::super::luau_audio_samples::{PcmStreamDescriptor, PcmStreamReader};
use super::super::luau_audio_shared::{
    MAX_SHARED_DEPTH, SHARED_BOOLEAN_BYTES, SHARED_CONTAINER_BYTES, SHARED_HANDLE_BYTES,
    SHARED_NIL_BYTES, SHARED_NUMBER_BYTES, SharedValueBudget, shared_bytes_size,
};
use nuxie_luau_vm::{
    CallStatus, Error, HostEntry, HostError, HostFnId, HostPlainPtr, HostedCtx, HostedVm, Limits,
    PlainCtx, Root, Stack, Type,
};

const STOPPED_TEXT: &str = "audio processor stopped";
const BUDGET_TEXT: &str = "audio processor execution budget exceeded";

/// The host functions, by their entries in the VM's table.
#[derive(Clone, Copy)]
struct Entries {
    print: HostFnId,
    require: HostFnId,
    stream_reader: HostFnId,
    sample_rate: HostFnId,
    channels: HostFnId,
    pcm_read_into: HostFnId,
    pcm_field: HostFnId,
    shared_field: HostFnId,
    blob_read: HostFnId,
    blob_to_buffer: HostFnId,
    region_load: HostFnId,
    region_store: HostFnId,
    region_add: HostFnId,
    region_cas: HostFnId,
    region_exclusive: HostFnId,
    ring_send: HostFnId,
    ring_receive: HostFnId,
    ring_receive_into: HostFnId,
    snapshot_publish: HostFnId,
    snapshot_latest: HostFnId,
}

/// The processor's host state: `raw.rs`'s `Context` without the
/// allocator's counters (the core's budget) and with roots for refs.
struct AudioHost {
    shared: Arc<Shared>,
    deadline: Option<Instant>,
    expired: bool,
    initializing: bool,
    failure: Option<String>,
    /// The two interrupt errors, rooted strings: raising one allocates
    /// nothing when the budget is spent.
    stop_error: Option<Root>,
    budget_error: Option<Root>,
    modules: BTreeMap<String, Arc<[u8]>>,
    loaded: BTreeMap<String, Root>,
    loading: BTreeSet<String>,
    channels: Vec<SharedValue>,
    /// Channel objects by their table's identity; the tables are rooted
    /// (`channel_roots`), so an identity never names another table.
    channel_objects: BTreeMap<u64, usize>,
    channel_roots: Vec<Root>,
    /// `Audio.streamReader` results, by the data of their two natives; a
    /// slot empties when the collector has freed both (`release_native`).
    pcm_readers: Vec<Option<PcmSlot>>,
    /// Host bytes the readers hold, against `host_memory_limit`.
    host_memory_used: usize,
    /// The processor VM's own memory ceiling: a processor's host memory
    /// budget is the same number, as a script VM's is (`HostMemoryBudget`).
    host_memory_limit: usize,
    format: ProcessorFormat,
    entries: Option<Entries>,
}

impl nuxie_luau_vm::Host for AudioHost {
    type Loans = nuxie_luau_vm::NoLoan;

    fn interrupt(&mut self) -> Result<(), HostError> {
        if self.initializing && self.shared.cancelled.load(Ordering::Acquire) {
            return Err(rooted_error(self.stop_error, STOPPED_TEXT));
        }
        if self.deadline.is_some_and(|end| Instant::now() >= end) {
            self.expired = true;
        }
        if self.expired {
            return Err(rooted_error(self.budget_error, BUDGET_TEXT));
        }
        Ok(())
    }
}

fn rooted_error(root: Option<Root>, text: &str) -> HostError {
    root.map_or_else(|| HostError::raw(text), HostError::from_root)
}

/// C `lua_error` with a string: the value is exactly `message`.
fn error(message: impl Into<String>) -> HostError {
    HostError::raw(message)
}

/// An error the script must not survive: recorded, then raised.
fn host_error(host: &mut AudioHost, message: &str) -> HostError {
    host.failure = Some(message.into());
    error(message)
}

/// One stream reader, its charge, and how many of its natives (`readInto`,
/// the `__index` field function) are alive.
struct PcmSlot {
    reader: PcmStreamReader,
    natives: u8,
    bytes: usize,
}

impl AudioHost {
    fn pcm_reader_fits(&self, bytes: usize) -> bool {
        bytes <= self.host_memory_limit.saturating_sub(self.host_memory_used)
    }

    fn reader(&mut self, data: u32) -> Option<&mut PcmStreamReader> {
        self.pcm_readers
            .get_mut(data as usize)
            .and_then(Option::as_mut)
            .map(|slot| &mut slot.reader)
    }

    /// A native naming reader slot `data` was made.
    fn named(&mut self, data: u32) {
        if let Some(Some(slot)) = self.pcm_readers.get_mut(data as usize) {
            slot.natives = slot.natives.saturating_add(1);
        }
    }

    /// Drops reader slot `data` if no native names it.
    fn release_unnamed(&mut self, data: u32) {
        let Some(entry) = self.pcm_readers.get_mut(data as usize) else {
            return;
        };
        if let Some(slot) = entry.as_ref()
            && slot.natives == 0
        {
            let bytes = slot.bytes;
            *entry = None;
            self.host_memory_used = self.host_memory_used.saturating_sub(bytes);
        }
    }

    /// The collector freed native `data` of entry `id`; a reader both of whose
    /// natives are freed is dropped and its bytes refunded.
    fn release_native(&mut self, id: HostFnId, data: u32) {
        let Some(entries) = self.entries else {
            return;
        };
        if id != entries.pcm_read_into && id != entries.pcm_field {
            return;
        }
        if let Some(Some(slot)) = self.pcm_readers.get_mut(data as usize) {
            slot.natives = slot.natives.saturating_sub(1);
        }
        self.release_unnamed(data);
    }

    fn entries(&self) -> Result<Entries, HostError> {
        self.entries
            .ok_or_else(|| error("audio processor host functions are not installed"))
    }

    fn channel(&self, data: u32) -> Result<SharedValue, HostError> {
        self.channels
            .get(data as usize)
            .cloned()
            .ok_or_else(|| error("audio processor channel is missing"))
    }
}

/// What both kinds of host function context offer the shared helpers.
trait Lane {
    fn split(&mut self) -> (Stack<'_>, &mut AudioHost);
    fn push_host_function(&mut self, name: &str, id: HostFnId, data: u32)
    -> Result<(), HostError>;
}

impl Lane for PlainCtx<'_, AudioHost> {
    fn split(&mut self) -> (Stack<'_>, &mut AudioHost) {
        PlainCtx::split(self)
    }
    fn push_host_function(
        &mut self,
        name: &str,
        id: HostFnId,
        data: u32,
    ) -> Result<(), HostError> {
        PlainCtx::push_host_function(self, name, id, data)
    }
}

impl Lane for HostedCtx<'_, AudioHost> {
    fn split(&mut self) -> (Stack<'_>, &mut AudioHost) {
        HostedCtx::split(self)
    }
    fn push_host_function(
        &mut self,
        name: &str,
        id: HostFnId,
        data: u32,
    ) -> Result<(), HostError> {
        HostedCtx::push_host_function(self, name, id, data)
    }
}

/// `raw.rs`'s `diagnostic`: the error string on top, else `fallback`.
fn diagnostic(stack: &Stack<'_>, fallback: &str) -> String {
    match stack.type_of(-1) {
        Some(Type::String) => stack
            .string(-1)
            .map_or_else(|| fallback.into(), |s| String::from_utf8_lossy(s).into_owned()),
        _ => fallback.into(),
    }
}

/// `raw.rs`'s `call`: a protected call whose error comes back as its text.
fn call(ctx: &mut HostedCtx<'_, AudioHost>, args: usize, results: i32) -> Result<(), HostError> {
    match ctx.pcall_top(args, results)? {
        CallStatus::Ok => Ok(()),
        CallStatus::Error(_) => Err(error(diagnostic(
            &ctx.stack(),
            "audio processor Lua call failed",
        ))),
    }
}

/// The channel a channel method's data names.
fn channel(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<SharedValue, HostError> {
    let data = ctx.data();
    ctx.host().channel(data)
}

fn data_of(index: usize) -> Result<u32, HostError> {
    u32::try_from(index).map_err(|_| error("too many audio processor channels"))
}

/// The bytes of an `f32` sample buffer, read as little-endian samples.
/// The clock reading `duration` from now; `None` past the clock's range.
fn deadline_in(duration: Duration) -> Option<Instant> {
    Instant::now().checked_add(duration)
}
fn copy_samples(out: &mut [f32], bytes: &[u8]) {
    for (sample, chunk) in out.iter_mut().zip(bytes.chunks_exact(4)) {
        if let [a, b, c, d] = *chunk {
            *sample = f32::from_le_bytes([a, b, c, d]);
        }
    }
}

pub(super) struct SafeProcessor {
    vm: HostedVm<AudioHost>,
    object: Root,
    process: Root,
    output: Root,
    sample_rate: u32,
    pub(super) channels: u32,
    pub(super) max_frames: u32,
    pub(super) hard_blocks: u32,
    pub(super) warm: bool,
}

// The whole processor moves to the audio thread (plan decision: no
// `unsafe impl Send`; the compiler checks it).
const _: fn() = || {
    fn send<T: Send>() {}
    send::<SafeProcessor>();
};

impl SafeProcessor {
    /// The lane's state, shared with its handle.
    pub(super) fn shared(&self) -> &Arc<Shared> {
        &self.vm.host().shared
    }

    // The lane names both processors through one alias, so this takes the
    // raw processor's parameters.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        shared: Arc<Shared>,
        name: String,
        modules: BTreeMap<String, Arc<[u8]>>,
        input: SharedValue,
        sample_rate: u32,
        channels: u32,
        max_frames: u32,
        hard_blocks: u32,
        memory_limit: usize,
    ) -> Result<Self, String> {
        // The execution deadlines below read the host clock; without the browser
        // audio host they could never expire, so processor code never runs.
        #[cfg(target_arch = "wasm32")]
        super::super::luau_audio_web::require_host()?;
        let host = AudioHost {
            shared,
            deadline: deadline_in(Duration::from_millis(1000)),
            expired: false,
            initializing: true,
            failure: None,
            stop_error: None,
            budget_error: None,
            modules,
            loaded: BTreeMap::new(),
            loading: BTreeSet::new(),
            channels: Vec::new(),
            channel_objects: BTreeMap::new(),
            channel_roots: Vec::new(),
            pcm_readers: Vec::new(),
            host_memory_used: 0,
            host_memory_limit: memory_limit,
            format: ProcessorFormat {
                sample_rate,
                channels,
                max_frames,
            },
            entries: None,
        };
        let limits = Limits {
            max_memory_bytes: Some(memory_limit),
            ..Limits::default()
        };
        let mut vm = HostedVm::with_host(limits, host)
            .map_err(|_| String::from("cannot allocate audio processor VM"))?;
        let entries = install(&mut vm).map_err(|e| e.message())?;
        vm.host_mut().entries = Some(entries);
        vm.set_interrupt_enabled(true);
        let result = vm
            .enter(|ctx| globals(ctx, entries))
            .and_then(|()| {
                vm.sandbox_globals();
                vm.enter(|ctx| start(ctx, &name, &input, sample_rate, channels, max_frames))
            });
        let (object, process, output) =
            result.map_err(|e| lua_message(&e, "audio processor initialization failed"))?;
        if let Some(error) = vm.host().failure.as_ref() {
            return Err(error.clone());
        }
        if vm.memory_exhausted() {
            return Err("audio processor memory limit exceeded".into());
        }
        vm.drain_freed_natives(AudioHost::release_native);
        let host = vm.host_mut();
        host.deadline = None;
        host.initializing = false;
        Ok(Self {
            vm,
            object,
            process,
            output,
            sample_rate,
            channels,
            max_frames,
            hard_blocks,
            warm: false,
        })
    }

    /// Runs the collector to the end of a cycle twice and reclaims what it
    /// freed, as the blocks' collection steps eventually do.
    #[cfg(test)]
    pub(super) fn collect_garbage(&mut self) {
        self.vm.collect_step(-1);
        self.vm.collect_step(-1);
        self.vm.drain_freed_natives(AudioHost::release_native);
    }

    /// Host bytes the processor's readers hold.
    #[cfg(test)]
    pub(super) fn host_memory_used(&self) -> usize {
        self.vm.host().host_memory_used
    }

    fn process_block(&mut self, out: &mut [f32], frames: u32, start: u64) -> Result<(), String> {
        if frames > self.max_frames
            || Some(out.len()) != (frames as usize).checked_mul(self.channels as usize)
        {
            return Err("audio format exceeds initialized processor format".into());
        }
        let ms = if self.hard_blocks == 0 {
            0
        } else {
            let rate = NonZeroU64::new(u64::from(self.sample_rate))
                .ok_or("invalid processor sample rate")?;
            let span = u64::from(frames)
                .saturating_mul(1000)
                .saturating_mul(u64::from(self.hard_blocks))
                .div_ceil(rate.get());
            span.max(if self.warm { 1 } else { 100 })
        };
        let host = self.vm.host_mut();
        host.expired = false;
        // A span past the clock's range has no deadline, as if unbounded.
        host.deadline = (ms != 0)
            .then(|| deadline_in(Duration::from_millis(ms)))
            .flatten();
        self.warm = true;
        let (process, object, output) = (self.process, self.object, self.output);
        let result = self.vm.enter(|ctx| {
            let mut stack = ctx.stack();
            stack.push_root(output)?;
            if let Some(bytes) = stack.buffer_mut(-1) {
                bytes.fill(0);
            }
            stack.set_top(0)?;
            stack.push_root(process)?;
            stack.push_root(object)?;
            stack.push_root(output)?;
            stack.push_number(frames as f64)?;
            stack.push_number(start as f64)?;
            call(ctx, 4, 0)?;
            ctx.host().interrupt_now()?;
            // The buffer is rooted and the collector does not move it. Copy
            // only the active frames; an author may retain the buffer safely.
            let mut stack = ctx.stack();
            stack.push_root(output)?;
            if let Some(bytes) = stack.buffer(-1) {
                copy_samples(out, bytes);
            }
            ctx.collect_step(0);
            ctx.host().interrupt_now()?;
            Ok(())
        });
        // Readers the block's collection step freed; this allocates nothing.
        self.vm.drain_freed_natives(AudioHost::release_native);
        let host = self.vm.host_mut();
        host.deadline = None;
        if let Some(error) = host.failure.as_ref() {
            return Err(error.clone());
        }
        if self.vm.memory_exhausted() {
            return Err("audio processor memory limit exceeded".into());
        }
        if self.vm.host().expired {
            return Err(BUDGET_TEXT.into());
        }
        result.map_err(|e| lua_message(&e, "audio processor processing failed"))
    }
}

impl AudioHost {
    /// `raw.rs`'s manual `interrupt(l, 0)` between host steps.
    fn interrupt_now(&mut self) -> Result<(), HostError> {
        nuxie_luau_vm::Host::interrupt(self)
    }
}

/// A Lua error's text, as `raw.rs` reads it from the stack: the string
/// raised (every error the processor raises is one), else `fallback`.
fn lua_message(e: &Error, fallback: &str) -> String {
    match e {
        Error::Lua { message, .. } => String::from_utf8_lossy(message).into_owned(),
        Error::Load(_) | Error::Host(_) => e.message(),
        Error::Conversion { .. } => fallback.into(),
    }
}

impl AudioRenderer for SafeProcessor {
    fn render(&mut self, out: &mut [f32], frames: u32, start: u64) {
        out.fill(0.0);
        if self.shared().state.load(Ordering::Acquire) != RUNNING {
            return;
        }
        const FORMAT_TEXT: &str = "audio format exceeds initialized processor format";
        let channels = self.channels as usize;
        // `out` was zeroed above; a failed block zeroes itself and the blocks
        // after it are never written.
        let Some(mut rest) = (frames as usize)
            .checked_mul(channels)
            .and_then(|samples| out.get_mut(..samples))
        else {
            self.shared().fail(FORMAT_TEXT.into());
            return;
        };
        let mut remaining = frames;
        let mut frame = start;
        while remaining != 0 && self.shared().state.load(Ordering::Acquire) == RUNNING {
            let count = remaining.min(self.max_frames);
            let Some((block, tail)) = (count != 0)
                .then(|| rest.split_at_mut_checked((count as usize).saturating_mul(channels)))
                .flatten()
            else {
                self.shared().fail(FORMAT_TEXT.into());
                break;
            };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.process_block(block, count, frame)
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    block.fill(0.0);
                    self.shared().fail(error);
                    break;
                }
                Err(_) => {
                    block.fill(0.0);
                    self.shared().fail("audio processor panicked".into());
                    break;
                }
            }
            rest = tail;
            remaining = remaining.saturating_sub(count);
            frame = frame.wrapping_add(u64::from(count));
        }
    }
    fn finished(&self) -> bool {
        self.shared().state.load(Ordering::Acquire) >= STOPPED
    }
}

// ---- setup -------------------------------------------------------------------

fn install(vm: &mut HostedVm<AudioHost>) -> Result<Entries, Error> {
    let mut plain =
        |f: HostPlainPtr<AudioHost>| vm.add_host_fn(HostEntry::Plain(f), false);
    let sample_rate = plain(processor_sample_rate)?;
    let channels = plain(processor_channels)?;
    // A reader's natives own its slot: the collector lists them when it frees
    // them, and `release_native` drops the reader.
    let pcm_read_into = vm.add_host_fn(HostEntry::Plain(pcm_read_into), true)?;
    let pcm_field = vm.add_host_fn(HostEntry::Plain(pcm_field), true)?;
    let mut plain =
        |f: HostPlainPtr<AudioHost>| vm.add_host_fn(HostEntry::Plain(f), false);
    let shared_field = plain(shared_field)?;
    let blob_read = plain(blob_read)?;
    let blob_to_buffer = plain(blob_to_buffer)?;
    let region_load = plain(region_atomic_load)?;
    let region_store = plain(region_atomic_store)?;
    let region_add = plain(region_atomic_add)?;
    let region_cas = plain(region_atomic_cas)?;
    let region_exclusive = plain(region_exclusive)?;
    let ring_send = plain(ring_send)?;
    let ring_receive = plain(ring_receive)?;
    let ring_receive_into = plain(ring_receive_into)?;
    let snapshot_publish = plain(snapshot_publish)?;
    let snapshot_latest = plain(snapshot_latest)?;
    // Full, not plain: it reclaims freed readers through the VM first.
    let stream_reader = vm.add_host_fn(HostEntry::Full { f: pcm_reader, k: None }, false)?;
    let print = vm.add_host_fn(HostEntry::Full { f: print, k: None }, false)?;
    let require = vm.add_host_fn(HostEntry::Full { f: require, k: None }, false)?;
    Ok(Entries {
        print,
        require,
        stream_reader,
        sample_rate,
        channels,
        pcm_read_into,
        pcm_field,
        shared_field,
        blob_read,
        blob_to_buffer,
        region_load,
        region_store,
        region_add,
        region_cas,
        region_exclusive,
        ring_send,
        ring_receive,
        ring_receive_into,
        snapshot_publish,
        snapshot_latest,
    })
}

/// The interrupt errors, `print`, `require` and `Audio`, before the
/// sandbox freezes the globals.
fn globals(ctx: &mut HostedCtx<'_, AudioHost>, entries: Entries) -> Result<(), HostError> {
    let mut stack = ctx.stack();
    stack.push_string(STOPPED_TEXT.as_bytes())?;
    let stop = stack.root(-1)?;
    stack.push_string(BUDGET_TEXT.as_bytes())?;
    let budget = stack.root(-1)?;
    stack.set_top(0)?;
    let host = ctx.host();
    host.stop_error = Some(stop);
    host.budget_error = Some(budget);
    ctx.stack().push_globals()?;
    ctx.push_host_function("print", entries.print, 0)?;
    ctx.stack().raw_set_field(1, b"print")?;
    ctx.push_host_function("require", entries.require, 0)?;
    ctx.stack().raw_set_field(1, b"require")?;
    ctx.stack().push_table(0, 3)?;
    for (name, id) in [
        ("streamReader", entries.stream_reader),
        ("processorSampleRate", entries.sample_rate),
        ("processorChannels", entries.channels),
    ] {
        ctx.push_host_function(name, id, 0)?;
        ctx.stack().raw_set_field(-2, name.as_bytes())?;
    }
    ctx.stack().raw_set_field(1, b"Audio")?;
    ctx.stack().set_top(0)
}

/// Loads the processor module and runs its `init`: the module table, its
/// `process` and the output buffer, rooted.
fn start(
    ctx: &mut HostedCtx<'_, AudioHost>,
    name: &str,
    input: &SharedValue,
    sample_rate: u32,
    channels: u32,
    max_frames: u32,
) -> Result<(Root, Root, Root), HostError> {
    load_module(ctx, name)?;
    if ctx.stack().type_of(-1) != Some(Type::Table) {
        return Err(error("audio processor module must return a table"));
    }
    let object = ctx.stack().root(-1)?;
    ctx.push_field(-1, b"process")?;
    if ctx.stack().type_of(-1) != Some(Type::Function) {
        return Err(error(
            "audio processor needs process(self,out,frames,startFrame)",
        ));
    }
    let process = ctx.stack().root(-1)?;
    ctx.stack().set_top(0)?;
    let bytes = (max_frames as usize)
        .checked_mul(channels as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(HostError::memory)?;
    ctx.stack().push_buffer(bytes)?;
    let output = ctx.stack().root(-1)?;
    ctx.stack().set_top(0)?;
    ctx.stack().push_root(object)?;
    ctx.push_field(-1, b"init")?;
    match ctx.stack().type_of(-1) {
        Some(Type::Function) => {
            let mut stack = ctx.stack();
            stack.push_root(object)?;
            stack.push_number(sample_rate as f64)?;
            stack.push_number(max_frames as f64)?;
            stack.push_number(channels as f64)?;
            push(ctx, input)?;
            ctx.host().interrupt_now()?;
            // Upstream gives each native initialization step its own
            // 1000 ms budget; the module entry and init are distinct.
            ctx.host().deadline = deadline_in(Duration::from_millis(1000));
            call(ctx, 5, 0)?;
        }
        Some(Type::Nil) => {}
        _ => return Err(error("audio processor init must be a function")),
    }
    ctx.stack().set_top(0)?;
    ctx.host().interrupt_now()?;
    Ok((object, process, output))
}

// ---- full host functions -------------------------------------------------------

fn print(ctx: &mut HostedCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let count = ctx.stack().top();
    let mut line = String::with_capacity(1024);
    for index in 1..=count {
        if index != 1 && line.len() < 1024 {
            line.push('\t');
        }
        let index = isize::try_from(index).map_err(|_| error("too many values to print"))?;
        ctx.push_tostring(index)?;
        let stack = ctx.stack();
        let Some(text) = stack.string(-1) else {
            return Err(error("tostring must return text for print"));
        };
        let remaining = 1024usize.saturating_sub(line.len());
        let text = text.get(..text.len().min(remaining)).unwrap_or_default();
        line.push_str(&String::from_utf8_lossy(text));
        truncate(&mut line, 1024);
        ctx.stack().pop(1)?;
    }
    let shared = &ctx.host().shared;
    shared.logs.push(&line);
    shared.notify();
    Ok(0)
}

fn require(ctx: &mut HostedCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let stack = ctx.stack();
    let Some(name) = stack.string(1) else {
        return Err(error("require expects module name"));
    };
    let name = String::from_utf8_lossy(name).into_owned();
    load_module(ctx, &name)?;
    Ok(1)
}

/// Pushes module `name`'s value, loading and running it the first time.
fn load_module(ctx: &mut HostedCtx<'_, AudioHost>, name: &str) -> Result<(), HostError> {
    if let Some(root) = ctx.host().loaded.get(name).copied() {
        return ctx.stack().push_root(root);
    }
    let Some(bytes) = ctx.host().modules.get(name).cloned() else {
        return Err(error(format!("unknown processor dependency {name}")));
    };
    if !ctx.host().loading.insert(name.into()) {
        return Err(error("cyclic audio processor module dependency"));
    }
    ctx.push_loaded(name.as_bytes(), &bytes)?;
    call(ctx, 0, 1)?;
    let root = ctx.stack().root(-1)?;
    let host = ctx.host();
    host.loaded.insert(name.into(), root);
    host.loading.remove(name);
    Ok(())
}

// ---- shared values in and out ------------------------------------------------

/// Pushes `value`; a channel becomes a table of methods over its index.
fn push(lane: &mut impl Lane, value: &SharedValue) -> Result<(), HostError> {
    match value {
        SharedValue::Nil => lane.split().0.push_nil(),
        SharedValue::Boolean(v) => lane.split().0.push_boolean(*v),
        SharedValue::Number(v) => lane.split().0.push_number(*v),
        SharedValue::Bytes(v) => lane.split().0.push_string(v),
        SharedValue::Array(values) => {
            lane.split().0.push_table(values.len(), 0)?;
            for (i, v) in (1usize..).zip(values.iter()) {
                push(lane, v)?;
                lane.split().0.raw_set_index(-2, i)?;
            }
            Ok(())
        }
        SharedValue::Object(values) => {
            lane.split().0.push_table(0, values.len())?;
            for (k, v) in values {
                lane.split().0.push_string(k.as_bytes())?;
                push(lane, v)?;
                lane.split().0.raw_set(-3)?;
            }
            Ok(())
        }
        SharedValue::Ring(_)
        | SharedValue::Snapshot(_)
        | SharedValue::Blob(_)
        | SharedValue::Region(_) => {
            let (mut stack, host) = lane.split();
            let entries = host.entries()?;
            let index = host.channels.len();
            let data = data_of(index)?;
            host.channels.push(value.clone());
            stack.push_table(0, 6)?;
            let root = stack.root(-1)?;
            host.channel_roots.push(root);
            host.channel_objects.insert(stack.to_pointer(-1), index);
            let methods: &[(&str, HostFnId)] = match value {
                SharedValue::Ring(_) => &[
                    ("send", entries.ring_send),
                    ("receive", entries.ring_receive),
                    ("receiveInto", entries.ring_receive_into),
                ],
                SharedValue::Snapshot(_) => &[
                    ("publish", entries.snapshot_publish),
                    ("latest", entries.snapshot_latest),
                ],
                SharedValue::Blob(_) => &[
                    ("read", entries.blob_read),
                    ("toBuffer", entries.blob_to_buffer),
                ],
                _ => &[
                    ("atomicLoad", entries.region_load),
                    ("atomicStore", entries.region_store),
                    ("atomicAdd", entries.region_add),
                    ("atomicCas", entries.region_cas),
                    ("exclusive", entries.region_exclusive),
                    ("tryExclusive", entries.region_exclusive),
                ],
            };
            for &(name, id) in methods {
                lane.push_host_function(name, id, data)?;
                lane.split().0.raw_set_field(-2, name.as_bytes())?;
            }
            if matches!(value, SharedValue::Blob(_) | SharedValue::Region(_)) {
                lane.split().0.push_table(0, 1)?;
                lane.push_host_function("shared field", entries.shared_field, data)?;
                let (mut stack, _) = lane.split();
                stack.raw_set_field(-2, b"__index")?;
                stack.set_metatable(-2)?;
            }
            Ok(())
        }
    }
}

/// Reads the value at `index` as shared data; `seen` holds the tables on
/// the path (cycles are refused).
fn read(
    stack: &mut Stack<'_>,
    host: &AudioHost,
    index: isize,
    seen: &mut BTreeSet<u64>,
    budget: &mut SharedValueBudget,
) -> Result<SharedValue, HostError> {
    if seen.len() >= MAX_SHARED_DEPTH {
        return Err(error(format!(
            "audio message nesting exceeds {MAX_SHARED_DEPTH} tables"
        )));
    }
    let index = if index < 0 {
        isize::try_from(stack.top())
            .unwrap_or(isize::MAX)
            .saturating_add(index)
            .saturating_add(1)
    } else {
        index
    };
    // Each value is charged before it is copied.
    let charge = |budget: &mut SharedValueBudget, encoded| budget.value(encoded).map_err(error);
    match stack.type_of(index) {
        None | Some(Type::Nil) => {
            charge(budget, SHARED_NIL_BYTES)?;
            Ok(SharedValue::Nil)
        }
        Some(Type::Boolean) => {
            charge(budget, SHARED_BOOLEAN_BYTES)?;
            Ok(SharedValue::Boolean(stack.to_boolean(index)))
        }
        Some(Type::Number) => {
            charge(budget, SHARED_NUMBER_BYTES)?;
            Ok(SharedValue::Number(stack.to_number(index).unwrap_or(0.0)))
        }
        Some(Type::String) => {
            let bytes = stack.string(index).unwrap_or_default();
            charge(budget, shared_bytes_size(bytes.len()))?;
            Ok(SharedValue::Bytes(bytes.to_vec()))
        }
        Some(Type::Table) => {
            charge(
                budget,
                if host.channel_objects.contains_key(&stack.to_pointer(index)) {
                    SHARED_HANDLE_BYTES
                } else {
                    SHARED_CONTAINER_BYTES
                },
            )?;
            let identity = stack.to_pointer(index);
            if let Some(channel) = host.channel_objects.get(&identity).copied() {
                return host.channel(data_of(channel)?);
            }
            if !seen.insert(identity) {
                return Err(error("cyclic shared data is not supported"));
            }
            let mut numbers = BTreeMap::new();
            let mut strings = BTreeMap::new();
            stack.push_nil()?;
            while stack.next(index)? {
                let v = read(stack, host, -1, seen, budget)?;
                match stack.type_of(-2) {
                    Some(Type::String) => {
                        let key = stack.string(-2).unwrap_or_default();
                        budget.key(key.len()).map_err(error)?;
                        let key = std::str::from_utf8(key)
                            .map_err(|_| error("shared data keys must be UTF-8 strings"))?;
                        strings.insert(key.to_owned(), v);
                    }
                    Some(Type::Number) => {
                        let key = stack.to_number(-2).unwrap_or(0.0);
                        if key < 1.0 || key.fract() != 0.0 || key > i32::MAX as f64 {
                            return Err(error(
                                "shared arrays need consecutive positive integer keys",
                            ));
                        }
                        numbers.insert(key as usize, v);
                    }
                    _ => return Err(error("shared data keys must be strings or array indices")),
                }
                stack.pop(1)?;
            }
            seen.remove(&identity);
            if !numbers.is_empty() {
                if !strings.is_empty() || numbers.keys().copied().ne(1..=numbers.len()) {
                    return Err(error("shared table must be an object or dense array"));
                }
                Ok(SharedValue::Array(numbers.into_values().collect()))
            } else {
                Ok(SharedValue::Object(strings))
            }
        }
        _ => Err(error(
            "shared payload must be plain data; handles are allowed only in processor input",
        )),
    }
}

/// Argument `index` as an index: a nonnegative integer up to `maximum`.
fn index_argument(stack: &Stack<'_>, index: isize, maximum: f64) -> Result<usize, HostError> {
    let value = stack.to_number(index).unwrap_or(f64::NAN);
    if stack.type_of(index) != Some(Type::Number)
        || !value.is_finite()
        || value < 0.0
        || value.fract() != 0.0
        || value > maximum
    {
        return Err(error(
            "shared offset/value must be a nonnegative integer in range",
        ));
    }
    Ok(value as usize)
}

// ---- plain host functions ------------------------------------------------------

fn processor_sample_rate(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let rate = ctx.host().format.sample_rate;
    ctx.stack().push_number(rate as f64)?;
    Ok(1)
}

fn processor_channels(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let channels = ctx.host().format.channels;
    ctx.stack().push_number(channels as f64)?;
    Ok(1)
}

fn pcm_reader(ctx: &mut HostedCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let (mut stack, host) = ctx.split();
    let value = read(&mut stack, host, 1, &mut BTreeSet::new(), &mut SharedValueBudget::default())?;
    let descriptor = PcmStreamDescriptor::from_shared(&value).map_err(|e| host_error(host, &e))?;
    // The reader's record buffer is charged before it is allocated, and the
    // slot with it; a full budget is an ordinary error the script can catch.
    let record = descriptor
        .record_capacity()
        .map_err(|e| host_error(host, &e))?;
    let bytes = record.saturating_add(std::mem::size_of::<Option<PcmSlot>>());
    // Readers the collector freed give their bytes back first. The collector
    // cannot see host bytes, so a budget that looks full may hold garbage:
    // then it finishes the cycle under way and runs one more, which frees
    // every reader no live value reaches, and reclaims again.
    let vm = ctx.vm_and_loan().0;
    vm.drain_freed_natives(AudioHost::release_native);
    if !vm.host().pcm_reader_fits(bytes) {
        vm.collect_step(-1);
        vm.collect_step(-1);
        vm.drain_freed_natives(AudioHost::release_native);
    }
    let (mut stack, host) = ctx.split();
    if !host.pcm_reader_fits(bytes) {
        return Err(error(format!(
            "script VM exceeded its {}-byte host memory budget",
            host.host_memory_limit
        )));
    }
    let index = host
        .pcm_readers
        .iter()
        .position(Option::is_none)
        .unwrap_or(host.pcm_readers.len());
    let data = data_of(index)?;
    let entries = host.entries()?;
    if index == host.pcm_readers.len() {
        host.pcm_readers
            .try_reserve(1)
            .map_err(|_| error("cannot allocate PCM reader"))?;
        host.pcm_readers.push(None);
    }
    let reader = PcmStreamReader::new(descriptor).map_err(|e| host_error(host, &e))?;
    let Some(entry) = host.pcm_readers.get_mut(index) else {
        return Err(error("audio processor stream reader is missing"));
    };
    *entry = Some(PcmSlot {
        reader,
        natives: 0,
        bytes,
    });
    host.host_memory_used = host.host_memory_used.saturating_add(bytes);
    let pushed = push_pcm_reader(ctx, entries, data);
    if pushed.is_err() {
        // A slot no native names would never be released.
        ctx.host().release_unnamed(data);
    }
    pushed
}

/// The reader table for slot `data`; counts each native as it is made.
fn push_pcm_reader(
    ctx: &mut HostedCtx<'_, AudioHost>,
    entries: Entries,
    data: u32,
) -> Result<usize, HostError> {
    ctx.stack().push_table(0, 1)?;
    ctx.push_host_function("readInto", entries.pcm_read_into, data)?;
    ctx.host().named(data);
    ctx.stack().raw_set_field(-2, b"readInto")?;
    ctx.stack().push_table(0, 1)?;
    ctx.push_host_function("PCM field", entries.pcm_field, data)?;
    ctx.host().named(data);
    let mut stack = ctx.stack();
    stack.raw_set_field(-2, b"__index")?;
    stack.set_metatable(-2)?;
    Ok(1)
}

fn pcm_read_into(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let index = ctx.data() as usize;
    let (mut stack, host) = ctx.split();
    let Some(output) = stack.buffer_mut(2) else {
        return Err(error("readInto expects an output buffer"));
    };
    let Some(reader) = host.reader(index as u32) else {
        return Err(error("audio processor stream reader is missing"));
    };
    let read = reader.read_into_bytes(output);
    let count = read.map_err(|e| host_error(host, &e))?;
    stack.push_number(count as f64)?;
    Ok(1)
}

fn pcm_field(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let index = ctx.data() as usize;
    let (mut stack, host) = ctx.split();
    let Some(reader) = host.reader(index as u32) else {
        return Err(error("audio processor stream reader is missing"));
    };
    enum Field {
        Number(f64),
        Boolean(bool),
        Nil,
    }
    let field = match stack.string(2) {
        Some(b"frame") => Field::Number(reader.frame()),
        Some(b"ended") => Field::Boolean(reader.ended()),
        Some(b"sampleRate") => Field::Number(reader.descriptor().sample_rate as f64),
        Some(b"channels") => Field::Number(reader.descriptor().channels as f64),
        _ => Field::Nil,
    };
    match field {
        Field::Number(n) => stack.push_number(n)?,
        Field::Boolean(b) => stack.push_boolean(b)?,
        Field::Nil => stack.push_nil()?,
    }
    Ok(1)
}

fn shared_field(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let channel = channel(ctx)?;
    let mut stack = ctx.stack();
    let (size, poisoned) = match (&channel, stack.string(2)) {
        (SharedValue::Blob(blob), Some(b"size")) => (Some(blob.size()), None),
        (SharedValue::Region(region), Some(b"size")) => (Some(region.size()), None),
        (SharedValue::Region(region), Some(b"poisoned")) => (None, Some(region.poisoned())),
        _ => (None, None),
    };
    match (size, poisoned) {
        (Some(size), _) => stack.push_number(size as f64)?,
        (_, Some(poisoned)) => stack.push_boolean(poisoned)?,
        _ => stack.push_nil()?,
    }
    Ok(1)
}

fn blob_read(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let SharedValue::Blob(blob) = channel(ctx)? else {
        return Err(error("audio processor channel is not a blob"));
    };
    let mut stack = ctx.stack();
    let offset = index_argument(&stack, 2, usize::MAX as f64)?;
    let Some(output) = stack.buffer_mut(3) else {
        return Err(error("Blob.read expects an output buffer"));
    };
    let count = blob.read(offset, output);
    stack.push_number(count as f64)?;
    Ok(1)
}

fn blob_to_buffer(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let SharedValue::Blob(blob) = channel(ctx)? else {
        return Err(error("audio processor channel is not a blob"));
    };
    let mut stack = ctx.stack();
    let output = stack.push_buffer(blob.size())?;
    blob.read(0, output);
    Ok(1)
}

fn region_atomic(ctx: &mut PlainCtx<'_, AudioHost>, operation: u8) -> Result<usize, HostError> {
    let SharedValue::Region(region) = channel(ctx)? else {
        return Err(error("audio processor channel is not a region"));
    };
    let (mut stack, host) = ctx.split();
    let offset = index_argument(&stack, 2, usize::MAX as f64)?;
    let value = if operation != 0 {
        index_argument(&stack, 3, u32::MAX as f64)? as u32
    } else {
        0
    };
    let result = match operation {
        0 => region.atomic_load(offset),
        1 => region.atomic_store(offset, value),
        2 => region.atomic_add(offset, value),
        _ => region.atomic_cas(offset, value, index_argument(&stack, 4, u32::MAX as f64)? as u32),
    };
    match result {
        Ok(value) => stack.push_number(value as f64)?,
        Err(message) => return Err(host_error(host, &message)),
    }
    Ok(1)
}

fn region_atomic_load(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    region_atomic(ctx, 0)
}
fn region_atomic_store(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    region_atomic(ctx, 1)
}
fn region_atomic_add(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    region_atomic(ctx, 2)
}
fn region_atomic_cas(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    region_atomic(ctx, 3)
}

fn region_exclusive(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    Err(host_error(
        ctx.host(),
        "a processor cannot lock a shared region; use atomic word operations",
    ))
}

fn ring_send(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let SharedValue::Ring(ring) = channel(ctx)? else {
        return Err(error("audio processor channel is not a ring"));
    };
    let (mut stack, host) = ctx.split();
    let value = read(&mut stack, host, 2, &mut BTreeSet::new(), &mut SharedValueBudget::default())?;
    match ring.send_from_processor(value) {
        Ok(ok) => stack.push_boolean(ok)?,
        Err(e) => return Err(host_error(host, &e)),
    }
    Ok(1)
}

fn ring_receive(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let SharedValue::Ring(ring) = channel(ctx)? else {
        return Err(error("audio processor channel is not a ring"));
    };
    match ring
        .receive_for_processor()
        .map_err(|e| host_error(ctx.host(), &e))?
    {
        Some(value) => push(ctx, &value)?,
        None => ctx.stack().push_nil()?,
    }
    Ok(1)
}

fn ring_receive_into(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    if ctx.stack().type_of(2) != Some(Type::Table) {
        return Err(error("receiveInto expects a destination table"));
    }
    let SharedValue::Ring(ring) = channel(ctx)? else {
        return Err(error("audio processor channel is not a ring"));
    };
    let Some(value) = ring
        .receive_for_processor()
        .map_err(|e| host_error(ctx.host(), &e))?
    else {
        ctx.stack().push_boolean(false)?;
        return Ok(1);
    };
    // Every key first, then each cleared, as the vendored processor did.
    let mut keys = Vec::new();
    let mut stack = ctx.stack();
    stack.push_nil()?;
    while stack.next(2)? {
        stack.pop(1)?;
        keys.push(stack.root(-1)?);
    }
    for key in keys {
        let mut stack = ctx.stack();
        let cleared = stack
            .push_root(key)
            .and_then(|()| stack.push_nil())
            .and_then(|()| stack.raw_set(2));
        ctx.release_root(key);
        cleared?;
    }
    match value {
        SharedValue::Object(fields) => {
            for (k, v) in fields {
                ctx.stack().push_string(k.as_bytes())?;
                push(ctx, &v)?;
                ctx.stack().raw_set(2)?;
            }
        }
        SharedValue::Array(values) => {
            for (i, v) in (1usize..).zip(values.iter()) {
                push(ctx, v)?;
                ctx.stack().raw_set_index(2, i)?;
            }
        }
        _ => return Err(error("receiveInto requires an object or array message")),
    }
    ctx.stack().push_boolean(true)?;
    Ok(1)
}

fn snapshot_publish(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let SharedValue::Snapshot(snapshot) = channel(ctx)? else {
        return Err(error("audio processor channel is not a snapshot"));
    };
    let (mut stack, host) = ctx.split();
    let value = read(&mut stack, host, 2, &mut BTreeSet::new(), &mut SharedValueBudget::default())?;
    match snapshot.publish_from_processor(value) {
        Ok(ok) => stack.push_boolean(ok)?,
        Err(e) => return Err(host_error(host, &e)),
    }
    Ok(1)
}

fn snapshot_latest(ctx: &mut PlainCtx<'_, AudioHost>) -> Result<usize, HostError> {
    let SharedValue::Snapshot(snapshot) = channel(ctx)? else {
        return Err(error("audio processor channel is not a snapshot"));
    };
    let latest = snapshot
        .latest_for_processor()
        .map_err(|e| host_error(ctx.host(), &e))?;
    let Some((sequence, value)) = latest else {
        ctx.stack().push_nil()?;
        return Ok(1);
    };
    let mut stack = ctx.stack();
    stack.push_table(0, 2)?;
    stack.push_number(sequence as f64)?;
    stack.raw_set_field(-2, b"sequence")?;
    push(ctx, &value)?;
    ctx.stack().raw_set_field(-2, b"value")?;
    Ok(1)
}
