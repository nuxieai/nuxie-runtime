//! Async `Context:decodeImage` owner for pinned `src/lua/lua_image_decode.cpp`.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::luau::{AnyUserData, Buffer, Lua, RegistryKey, Result, Value};
use nuxie_image_codec::{DecodedImageRgba, decode_image_rgba, decoded_rgba_len, preflight_encoded_image};
use nuxie_runtime::{
    WorkCallbacks, WorkCancellationHandle, WorkPool, WorkTask, get_global_work_pool,
};

use super::host_memory_budget::HostMemoryCharge;
use super::lua_promise;

const DECODE_ERROR: &str = "failed to decode image data";
/// Decodes one VM may have submitted and not yet settled or dropped: a fixed
/// count beside the host memory budget, so cancelled requests that still
/// hold their copies on the pool cannot pile up.
const MAX_IMAGE_DECODES_IN_FLIGHT: usize = 16;
// Assigned by work-pool delivery, not background execution: separate VM
// queues must retain the global callback order when the root drains them.
static NEXT_COMPLETION_SEQUENCE: AtomicU64 = AtomicU64::new(0);
pub(super) fn next_completion_sequence() -> u64 {
    NEXT_COMPLETION_SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

trait ScriptImageDecoder: Send + Sync {
    fn decode_rgba(&self, encoded: &[u8]) -> Option<DecodedImageRgba>;
}

struct RustScriptImageDecoder;

impl ScriptImageDecoder for RustScriptImageDecoder {
    /// The bounded decoder: at most 8,192 pixels a side and 64 MiB decoded,
    /// the image import policy. A larger image fails as any undecodable one.
    fn decode_rgba(&self, encoded: &[u8]) -> Option<DecodedImageRgba> {
        decode_image_rgba(encoded)
    }
}

/// One request counted in flight, from submit until its task or its
/// completion is dropped, on whichever thread that is. Dropping it also
/// marks the request's buffers released.
struct InFlight {
    count: Arc<AtomicUsize>,
    released: Arc<AtomicBool>,
}

impl InFlight {
    fn admit(count: &Arc<AtomicUsize>) -> Option<Self> {
        count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < MAX_IMAGE_DECODES_IN_FLIGHT).then(|| current.saturating_add(1))
            })
            .ok()
            .map(|_| Self {
                count: Arc::clone(count),
                released: Arc::new(AtomicBool::new(false)),
            })
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        // Every token was counted when admitted, so this never goes below 0.
        let _ = self
            .count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                Some(current.saturating_sub(1))
            });
        self.released.store(true, Ordering::Release);
    }
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

enum DecodeCompletion {
    Success {
        sequence: u64,
        request_id: u64,
        image: DecodedImageRgba,
        _in_flight: Option<InFlight>,
    },
    Failure {
        sequence: u64,
        request_id: u64,
        message: String,
        _in_flight: Option<InFlight>,
    },
}

/// Completions the work pool delivered for one VM's Lua thread. Each counts
/// toward the runtime's queued tally until it is taken or dropped, which lets
/// a root poll skip finding VMs while none has anything queued.
#[derive(Default)]
struct CompletionQueue(VecDeque<DecodeCompletion>);

impl CompletionQueue {
    fn push_back(&mut self, completion: DecodeCompletion) {
        self.0.push_back(completion);
        nuxie_runtime::vm_completions_queued(1);
    }

    fn take_front(&mut self, limit: usize) -> Vec<DecodeCompletion> {
        let count = self.0.len().min(limit);
        let taken = self.0.drain(..count).collect();
        nuxie_runtime::vm_completions_taken(count);
        taken
    }

    fn front(&self) -> Option<&DecodeCompletion> {
        self.0.front()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Drop for CompletionQueue {
    fn drop(&mut self) {
        nuxie_runtime::vm_completions_taken(self.0.len());
    }
}

struct ImageDecodeTask {
    request_id: u64,
    decoder: Arc<dyn ScriptImageDecoder>,
    encoded: Mutex<Vec<u8>>,
    decoded: Mutex<Option<DecodedImageRgba>>,
    completions: Arc<Mutex<CompletionQueue>>,
    /// Moves into the completion, which then holds the decoded image.
    in_flight: Option<InFlight>,
}

impl ImageDecodeTask {
    fn new(
        request_id: u64,
        decoder: Arc<dyn ScriptImageDecoder>,
        encoded: Vec<u8>,
        completions: Arc<Mutex<CompletionQueue>>,
        in_flight: InFlight,
    ) -> Self {
        Self {
            request_id,
            decoder,
            encoded: Mutex::new(encoded),
            decoded: Mutex::new(None),
            completions,
            in_flight: Some(in_flight),
        }
    }

    fn release_buffers(&self) {
        let mut encoded = lock_unpoisoned(&self.encoded);
        encoded.clear();
        encoded.shrink_to_fit();
        lock_unpoisoned(&self.decoded).take();
    }
}

impl WorkCallbacks for ImageDecodeTask {
    fn execute(&mut self, error_message: &mut String) -> bool {
        let encoded = lock_unpoisoned(&self.encoded);
        let Some(decoded) = self.decoder.decode_rgba(&encoded) else {
            *error_message = DECODE_ERROR.to_owned();
            return false;
        };
        *lock_unpoisoned(&self.decoded) = Some(decoded);
        true
    }

    fn on_complete(&mut self) {
        if let Some(image) = lock_unpoisoned(&self.decoded).take() {
            lock_unpoisoned(&self.completions).push_back(DecodeCompletion::Success {
                sequence: next_completion_sequence(),
                request_id: self.request_id,
                image,
                _in_flight: self.in_flight.take(),
            });
        }
        self.release_buffers();
    }

    fn on_error(&mut self, error: &str) {
        lock_unpoisoned(&self.completions).push_back(DecodeCompletion::Failure {
            sequence: next_completion_sequence(),
            request_id: self.request_id,
            message: error.to_owned(),
            _in_flight: self.in_flight.take(),
        });
        self.release_buffers();
    }

    fn on_cancel(&mut self) {
        self.release_buffers();
    }
}

struct PendingDecode {
    promise: RegistryKey,
    cancellation: WorkCancellationHandle,
    /// The encoded copy and the decoded image, until the promise settles;
    /// the settled image is then the VM's buffer. A cancelled request keeps
    /// it until its task lets go of the buffers.
    charge: HostMemoryCharge,
    released: Arc<AtomicBool>,
}

/// A cancelled request whose task may still hold its buffers.
struct CancelledDecode {
    released: Arc<AtomicBool>,
    _charge: HostMemoryCharge,
}

pub(super) struct ImageDecodeRegistry {
    next_request_id: Cell<u64>,
    owner_id: u64,
    decoder: Arc<dyn ScriptImageDecoder>,
    pending: RefCell<HashMap<u64, PendingDecode>>,
    completions: Arc<Mutex<CompletionQueue>>,
    in_flight: Arc<AtomicUsize>,
    /// At most `MAX_IMAGE_DECODES_IN_FLIGHT`: each holds an in-flight slot.
    cancelled: RefCell<Vec<CancelledDecode>>,
}

impl ImageDecodeRegistry {
    fn new(decoder: Arc<dyn ScriptImageDecoder>) -> Self {
        Self {
            next_request_id: Cell::new(1),
            owner_id: WorkPool::next_owner_id(),
            decoder,
            pending: RefCell::new(HashMap::new()),
            completions: Arc::new(Mutex::new(CompletionQueue::default())),
            in_flight: Arc::new(AtomicUsize::new(0)),
            cancelled: RefCell::new(Vec::new()),
        }
    }

    fn next_request_id(&self) -> u64 {
        let request_id = self.next_request_id.get();
        let mut next = request_id.wrapping_add(1);
        while next == 0 || self.pending.borrow().contains_key(&next) {
            next = next.wrapping_add(1);
        }
        self.next_request_id.set(next);
        request_id
    }

    fn cancel(&self, request_id: u64) {
        let removed = self.pending.borrow_mut().remove(&request_id);
        if let Some(pending) = removed {
            pending.cancellation.cancel();
            self.release_cancelled();
            if !pending.released.load(Ordering::Acquire) {
                // Within the in-flight cap the slots were reserved at start.
                self.cancelled.borrow_mut().push(CancelledDecode {
                    released: pending.released,
                    _charge: pending.charge,
                });
            }
        }
    }

    /// Refunds cancelled requests whose tasks have let go of their buffers.
    fn release_cancelled(&self) {
        self.cancelled
            .borrow_mut()
            .retain(|cancelled| !cancelled.released.load(Ordering::Acquire));
    }

    fn cancel_all(&self) {
        for pending in self
            .pending
            .borrow_mut()
            .drain()
            .map(|(_, pending)| pending)
        {
            pending.cancellation.cancel();
        }
    }
}

struct ImageDecodeRegistryOwner(Rc<ImageDecodeRegistry>);

impl Drop for ImageDecodeRegistryOwner {
    fn drop(&mut self) {
        self.0.cancel_all();
    }
}

pub(super) fn install(lua: &Lua) {
    if lua.app_data_ref::<ImageDecodeRegistryOwner>().is_none() {
        lua.set_app_data(ImageDecodeRegistryOwner(Rc::new(ImageDecodeRegistry::new(
            Arc::new(RustScriptImageDecoder),
        ))));
    }
}

fn registry(lua: &Lua) -> Result<Rc<ImageDecodeRegistry>> {
    lua.app_data_ref::<ImageDecodeRegistryOwner>()
        .map(|owner| Rc::clone(&owner.0))
        .ok_or_else(|| crate::luau::Error::runtime("image decode registry is not installed"))
}

pub(super) fn start(lua: &Lua, encoded: Buffer) -> Result<AnyUserData> {
    if encoded.is_empty() {
        return Err(crate::luau::Error::runtime("decodeImage: empty buffer"));
    }

    let registry = registry(lua)?;
    registry.release_cancelled();
    let in_flight = InFlight::admit(&registry.in_flight).ok_or_else(|| {
        crate::luau::Error::runtime(format!(
            "decodeImage: at most {MAX_IMAGE_DECODES_IN_FLIGHT} images may be decoding at once"
        ))
    })?;
    // The copy is charged before it is made, the decoded image before it is
    // submitted, from the header. An image the header check refuses fails
    // to decode, so only its copy is charged.
    let mut charge = HostMemoryCharge::for_lua(lua, encoded.len())?;
    let encoded = encoded.to_vec();
    let decoded_bytes = preflight_encoded_image(&encoded)
        .and_then(|image| decoded_rgba_len(image.width, image.height))
        .unwrap_or(0);
    charge.grow(decoded_bytes)?;
    let released = Arc::clone(&in_flight.released);
    let promise = lua_promise::new_pending(lua)?;
    let promise_ref = lua.create_registry_value(promise.clone())?;
    let request_id = registry.next_request_id();
    let mut task = WorkTask::new(ImageDecodeTask::new(
        request_id,
        Arc::clone(&registry.decoder),
        encoded,
        Arc::clone(&registry.completions),
        in_flight,
    ));
    task.set_owner_id(registry.owner_id);
    registry.pending.borrow_mut().insert(
        request_id,
        PendingDecode {
            promise: promise_ref,
            cancellation: task.cancellation_handle(),
            charge,
            released,
        },
    );

    let cancel_registry = Rc::clone(&registry);
    let on_cancel = lua.create_function(move |_, ()| {
        cancel_registry.cancel(request_id);
        Ok(())
    })?;
    lua_promise::set_on_cancel(lua, promise.clone(), on_cancel)?;
    get_global_work_pool().submit(Some(Box::new(task)));
    Ok(promise)
}

/// The registry `install` placed in this VM's app data, if any. The VM keeps
/// it once found, so per-frame async polling skips the app-data lookup.
pub(super) fn installed_registry(lua: &Lua) -> Option<Rc<ImageDecodeRegistry>> {
    lua.app_data_ref::<ImageDecodeRegistryOwner>()
        .map(|owner| Rc::clone(&owner.0))
}

pub(super) fn has_pending_work(lua: &Lua) -> bool {
    let Some(owner) = lua.app_data_ref::<ImageDecodeRegistryOwner>() else {
        return false;
    };
    registry_has_pending_work(&owner.0)
}

pub(super) fn registry_has_pending_work(registry: &ImageDecodeRegistry) -> bool {
    // The wasm32-unknown-unknown lane uses the same Rust decoder and pool;
    // there are no browser-native requests outside this owner to count.
    // Pool callbacks hand off to the Lua-thread queue before settling promises.
    if !lock_unpoisoned(&registry.completions).is_empty() {
        return true;
    }
    nuxie_runtime::get_global_work_pool_if_exists()
        .is_some_and(|pool| pool.has_pending_work_for_owner(registry.owner_id))
}

#[cfg(feature = "upstream-test-seams")]
pub(super) fn upstream_test_owner_id(lua: &Lua) -> u64 {
    install(lua);
    installed_registry(lua).map_or(0, |registry| registry.owner_id)
}

pub(super) fn poll_completed(lua: &Lua) -> Result<bool> {
    poll_completed_limit(lua, usize::MAX)
}

pub(super) fn pending_completion_sequence(lua: &Lua) -> Option<u64> {
    let owner = lua.app_data_ref::<ImageDecodeRegistryOwner>()?;
    registry_pending_completion_sequence(&owner.0)
}

pub(super) fn registry_pending_completion_sequence(registry: &ImageDecodeRegistry) -> Option<u64> {
    let completions = lock_unpoisoned(&registry.completions);
    completions.front().map(|completion| match completion {
        DecodeCompletion::Success { sequence, .. } | DecodeCompletion::Failure { sequence, .. } => {
            *sequence
        }
    })
}

pub(super) fn poll_next_completed(lua: &Lua) -> Result<bool> {
    poll_completed_limit(lua, 1)
}

fn poll_completed_limit(lua: &Lua, limit: usize) -> Result<bool> {
    let Some(registry) = lua
        .app_data_ref::<ImageDecodeRegistryOwner>()
        .map(|owner| Rc::clone(&owner.0))
    else {
        return Ok(false);
    };
    poll_registry_completed(lua, registry, limit)
}

pub(super) fn poll_registry_completed(
    lua: &Lua,
    registry: Rc<ImageDecodeRegistry>,
    limit: usize,
) -> Result<bool> {
    let completions = lock_unpoisoned(&registry.completions).take_front(limit);
    let mut settled = false;
    for completion in completions {
        let request_id = match &completion {
            DecodeCompletion::Success { request_id, .. }
            | DecodeCompletion::Failure { request_id, .. } => *request_id,
        };
        let Some(pending) = registry.pending.borrow_mut().remove(&request_id) else {
            continue;
        };
        settled = true;
        let promise = lua.registry_value::<AnyUserData>(&pending.promise)?;
        match completion {
            DecodeCompletion::Success { image, .. } => {
                let result = lua.create_table();
                let result = (|| {
                    result.set("data", lua.create_buffer(image.pixels)?)?;
                    result.set("width", image.width)?;
                    result.set("height", image.height)?;
                    Result::Ok(Value::Table(result))
                })();
                match result {
                    Ok(result) => lua_promise::resolve(lua, promise, result)?,
                    Err(_) => lua_promise::reject(lua, promise, DECODE_ERROR.to_owned())?,
                }
            }
            DecodeCompletion::Failure { message, .. } => {
                lua_promise::reject(lua, promise, message)?;
            }
        }
    }
    registry.release_cancelled();
    Ok(settled)
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::time::{Duration, Instant};

    use crate::luau::Table;
    use nuxie_runtime::{ScriptInstance, ScriptingVm as RuntimeScriptingVm};

    use crate::vm::view_model::ScriptedContext;
    use crate::vm::{LuaScriptInstance, ScriptVm};

    fn lua_with_context() -> Lua {
        let lua = Lua::new();
        lua_promise::install_promise_globals(&lua).unwrap();
        install(&lua);
        let context = lua
            .create_userdata(ScriptedContext::new(
                &lua,
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .unwrap();
        lua.globals().set("context", context).unwrap();
        lua
    }

    fn poll_work_until(mut complete: impl FnMut() -> bool) {
        let start = Instant::now();
        loop {
            get_global_work_pool().poll_completed_work(16);
            if complete() {
                return;
            }
            // With threading, polling only delivers completed callbacks; it
            // does not execute the decode. A poll count is not a worker deadline.
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "image decode did not settle"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn drain_work(lua: &Lua) {
        poll_work_until(|| {
            poll_completed(lua).unwrap();
            registry(lua).unwrap().pending.borrow().is_empty()
        });
    }

    // Inventory row 4: requests in flight had no bound.
    #[test]
    fn decodes_in_flight_are_capped() {
        let lua = lua_with_context();
        let (failed, error): (bool, String) = lua
            .load(
                "pending = {} \
                 for i = 1, 16 do pending[i] = context:decodeImage(buffer.create(4)) end \
                 local ok, err = pcall(function() return context:decodeImage(buffer.create(4)) end) \
                 return not ok, tostring(err)",
            )
            .eval()
            .unwrap();
        assert!(failed);
        assert!(error.contains("at most 16 images may be decoding at once"), "{error}");
        drain_work(&lua);
        lua.load("context:decodeImage(buffer.create(4))").exec().unwrap();
        drain_work(&lua);
    }

    // Inventory rows 3 and 4: the encoded copy and the decoded image are
    // charged to the VM's host memory budget until the promise settles.
    #[test]
    fn a_decode_is_charged_until_it_settles() {
        let lua = lua_with_context();
        super::super::host_memory_budget::HostMemoryBudget::install(&lua, 64).unwrap();
        let budget = super::super::host_memory_budget::HostMemoryBudget::for_lua(&lua).unwrap();
        let (failed, error): (bool, String) = lua
            .load(
                "local ok, err = pcall(function() return context:decodeImage(buffer.create(65)) end) \
                 return not ok, tostring(err)",
            )
            .eval()
            .unwrap();
        assert!(failed);
        assert!(error.contains("64-byte host memory budget"), "{error}");
        lua.load("context:decodeImage(buffer.create(40))").exec().unwrap();
        assert_eq!(budget.used(), 40);
        drain_work(&lua);
        assert_eq!(budget.used(), 0);
    }

    // Codex review: a cancelled request was refunded while its task still
    // held the encoded copy.
    #[test]
    fn a_cancelled_decode_is_charged_until_its_task_lets_go() {
        let lua = lua_with_context();
        let budget = super::super::host_memory_budget::HostMemoryBudget::for_lua(&lua).unwrap();
        lua.load("context:decodeImage(buffer.create(40)):cancel()")
            .exec()
            .unwrap();
        // The pool has not delivered the task's callbacks: it holds the copy.
        assert_eq!(budget.used(), 40);
        let registry = registry(&lua).unwrap();
        poll_work_until(|| {
            poll_completed(&lua).unwrap();
            registry.in_flight.load(Ordering::Acquire) == 0
        });
        poll_completed(&lua).unwrap();
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn upstream_decode_image_cancel_sets_promise_to_cancelled() {
        let lua = lua_with_context();
        let status: String = lua
            .load(
                "local p = context:decodeImage(buffer.create(4)); \
                 p:cancel(); return p:getStatus()",
            )
            .eval()
            .unwrap();
        assert_eq!(status, "Cancelled");
    }

    #[test]
    fn upstream_decode_image_cancel_does_not_fire_and_then() {
        let lua = lua_with_context();
        let called: bool = lua
            .load(
                "local called = false; \
                 local p = context:decodeImage(buffer.create(4)); \
                 p:andThen(function() called = true end); \
                 p:cancel(); return called",
            )
            .eval()
            .unwrap();
        assert!(!called);
    }

    #[test]
    fn upstream_decode_image_cancel_fires_on_cancel_hook() {
        let lua = lua_with_context();
        let fired: bool = lua
            .load(
                "local fired = false; \
                 local p = context:decodeImage(buffer.create(4)); \
                 p:onCancel(function() fired = true end); \
                 p:cancel(); return fired",
            )
            .eval()
            .unwrap();
        assert!(fired);
    }

    #[test]
    fn decode_image_resolves_premultiplied_rgba_after_work_pool_poll() {
        let lua = lua_with_context();
        let mut encoded = Vec::new();
        image_webp::WebPEncoder::new(&mut encoded)
            .encode(
                &[240, 120, 60, 128, 10, 20, 30, 255],
                2,
                1,
                image_webp::ColorType::Rgba8,
            )
            .unwrap();
        lua.globals()
            .set("encoded", lua.create_buffer(encoded).unwrap())
            .unwrap();
        lua.load(
            "decoded = nil; decodeError = nil; \
             decodePromise = context:decodeImage(encoded); \
             decodePromise:andThen( \
                 function(value) decoded = value end, \
                 function(reason) decodeError = reason end)",
        )
        .exec()
        .unwrap();

        assert_eq!(
            lua.load("return decodePromise:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Pending"
        );
        drain_work(&lua);

        let status = lua
            .load("return decodePromise:getStatus()")
            .eval::<String>()
            .unwrap();
        let decoded_value = lua.globals().get::<Value>("decoded").unwrap();
        assert!(
            matches!(decoded_value, Value::Table(_)),
            "status={status} decoded={decoded_value:?}"
        );
        let decoded: Table = lua.globals().get("decoded").unwrap();
        let pixels: Buffer = decoded.get("data").unwrap();
        assert_eq!(decoded.get::<u32>("width").unwrap(), 2);
        assert_eq!(decoded.get::<u32>("height").unwrap(), 1);
        assert_eq!(pixels.to_vec(), [120, 60, 30, 128, 10, 20, 30, 255]);
        assert_eq!(status, "Fulfilled");
        assert!(matches!(
            lua.globals().get::<Value>("decodeError"),
            Ok(Value::Nil)
        ));
    }

    #[test]
    fn parked_script_settles_decode_from_root_async_poll_without_lua_reentry() {
        let lua = lua_with_context();
        let mut encoded = Vec::new();
        image_webp::WebPEncoder::new(&mut encoded)
            .encode(&[4, 8, 12, 255], 1, 1, image_webp::ColorType::Rgba8)
            .unwrap();
        lua.globals()
            .set("encoded", lua.create_buffer(encoded).unwrap())
            .unwrap();
        lua.load(
            "settled = false; decodePromise = context:decodeImage(encoded); \
             decodePromise:andThen(function() settled = true end)",
        )
        .exec()
        .unwrap();
        let table = lua.create_table();
        let mut instance = LuaScriptInstance::new(table);

        poll_work_until(|| instance.poll_async_work().unwrap());

        assert!(lua.globals().get::<bool>("settled").unwrap());
    }

    #[test]
    fn script_callbacks_leave_async_completion_for_the_root_poll_boundary() {
        let lua = lua_with_context();
        let mut encoded = Vec::new();
        image_webp::WebPEncoder::new(&mut encoded)
            .encode(&[4, 8, 12, 255], 1, 1, image_webp::ColorType::Rgba8)
            .unwrap();
        lua.globals()
            .set("encoded", lua.create_buffer(encoded).unwrap())
            .unwrap();
        let table: Table = lua
            .load(
                "decodePromise = context:decodeImage(encoded); \
                 return { advance = function() \
                     observedStatus = decodePromise:getStatus(); return false \
                 end }",
            )
            .eval()
            .unwrap();
        let mut instance = LuaScriptInstance::new(table);

        poll_work_until(|| !lock_unpoisoned(&registry(&lua).unwrap().completions).is_empty());
        assert!(
            !lock_unpoisoned(&registry(&lua).unwrap().completions).is_empty(),
            "decode completion never reached the VM-owned queue"
        );

        assert!(
            !instance
                .call_advance_truthy(1.0 / 60.0, &mut nuxie_runtime::NoopScriptHost)
                .unwrap()
        );
        assert_eq!(
            lua.globals().get::<String>("observedStatus").unwrap(),
            "Pending",
            "ordinary script callbacks must not replace Artboard::advance as the async poll authority"
        );

        assert!(instance.poll_async_work().unwrap());
        assert_eq!(
            lua.load("return decodePromise:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Fulfilled"
        );
    }

    #[test]
    fn vm_callbacks_leave_async_completion_for_the_artboard_poll_boundary() {
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let lua = vm.lua();
        let context = lua
            .create_userdata(ScriptedContext::new(
                &lua,
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .unwrap();
        lua.globals().set("context", context).unwrap();
        let mut encoded = Vec::new();
        image_webp::WebPEncoder::new(&mut encoded)
            .encode(&[4, 8, 12, 255], 1, 1, image_webp::ColorType::Rgba8)
            .unwrap();
        lua.globals()
            .set("encoded", lua.create_buffer(encoded).unwrap())
            .unwrap();
        lua.load("decodePromise = context:decodeImage(encoded)")
            .exec()
            .unwrap();

        poll_work_until(|| !lock_unpoisoned(&registry(lua).unwrap().completions).is_empty());
        vm.eval::<()>("local callbackRan = true").unwrap();
        assert_eq!(
            lua.load("return decodePromise:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Pending",
            "ordinary VM callbacks must not replace Artboard::advance as the async poll authority"
        );

        assert!(RuntimeScriptingVm::poll_async_work(&vm).unwrap());
        assert_eq!(
            lua.load("return decodePromise:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Fulfilled"
        );
    }

    #[test]
    fn decode_image_rejects_invalid_encoded_bytes_after_work_pool_poll() {
        let lua = lua_with_context();
        lua.load(
            "decodeError = nil; decodePromise = context:decodeImage(buffer.create(4)); \
             decodePromise:catch(function(reason) decodeError = reason end)",
        )
        .exec()
        .unwrap();
        drain_work(&lua);

        assert_eq!(
            lua.load("return decodePromise:getStatus()")
                .eval::<String>()
                .unwrap(),
            "Rejected"
        );
        assert_eq!(
            lua.globals().get::<String>("decodeError").unwrap(),
            DECODE_ERROR
        );
    }
}
