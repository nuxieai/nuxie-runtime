//! Async `Context:decodeImage` owner for pinned `src/lua/lua_image_decode.cpp`.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

use luaur_rt::{AnyUserData, Buffer, Lua, RegistryKey, Result, Value};
use nuxie_image_codec::{DecodedImageRgba, decode_image_rgba_unbounded};
use nuxie_runtime::{WorkTask, WorkTaskRef, WorkTaskState, with_global_work_pool};

use super::lua_promise;

const DECODE_ERROR: &str = "failed to decode image data";

trait ScriptImageDecoder: Send + Sync {
    fn decode_rgba(&self, encoded: &[u8]) -> Option<DecodedImageRgba>;
}

struct RustScriptImageDecoder;

impl ScriptImageDecoder for RustScriptImageDecoder {
    fn decode_rgba(&self, encoded: &[u8]) -> Option<DecodedImageRgba> {
        decode_image_rgba_unbounded(encoded)
    }
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

enum DecodeCompletion {
    Success {
        request_id: u64,
        image: DecodedImageRgba,
    },
    Failure {
        request_id: u64,
        message: String,
    },
}

struct ImageDecodeTask {
    state: WorkTaskState,
    request_id: u64,
    decoder: Arc<dyn ScriptImageDecoder>,
    encoded: Mutex<Vec<u8>>,
    decoded: Mutex<Option<DecodedImageRgba>>,
    completions: Arc<Mutex<VecDeque<DecodeCompletion>>>,
}

impl ImageDecodeTask {
    fn new(
        request_id: u64,
        owner_id: u64,
        decoder: Arc<dyn ScriptImageDecoder>,
        encoded: Vec<u8>,
        completions: Arc<Mutex<VecDeque<DecodeCompletion>>>,
    ) -> Self {
        let state = WorkTaskState::default();
        state.set_owner_id(owner_id);
        Self {
            state,
            request_id,
            decoder,
            encoded: Mutex::new(encoded),
            decoded: Mutex::new(None),
            completions,
        }
    }

    fn release_buffers(&self) {
        let mut encoded = lock_unpoisoned(&self.encoded);
        encoded.clear();
        encoded.shrink_to_fit();
        lock_unpoisoned(&self.decoded).take();
    }
}

impl WorkTask for ImageDecodeTask {
    fn state(&self) -> &WorkTaskState {
        &self.state
    }

    fn execute(&self) -> bool {
        let encoded = lock_unpoisoned(&self.encoded);
        let Some(decoded) = self.decoder.decode_rgba(&encoded) else {
            self.state.set_error_message(DECODE_ERROR);
            return false;
        };
        *lock_unpoisoned(&self.decoded) = Some(decoded);
        true
    }

    fn on_complete(&self) {
        if let Some(image) = lock_unpoisoned(&self.decoded).take() {
            lock_unpoisoned(&self.completions).push_back(DecodeCompletion::Success {
                request_id: self.request_id,
                image,
            });
        }
        self.release_buffers();
    }

    fn on_error(&self, error: &str) {
        lock_unpoisoned(&self.completions).push_back(DecodeCompletion::Failure {
            request_id: self.request_id,
            message: error.to_owned(),
        });
        self.release_buffers();
    }

    fn on_cancel(&self) {
        self.release_buffers();
    }
}

struct PendingDecode {
    promise: RegistryKey,
    task: WorkTaskRef<ImageDecodeTask>,
}

struct ImageDecodeRegistry {
    next_request_id: Cell<u64>,
    owner_id: u64,
    decoder: Arc<dyn ScriptImageDecoder>,
    pending: RefCell<HashMap<u64, PendingDecode>>,
    completions: Arc<Mutex<VecDeque<DecodeCompletion>>>,
}

impl ImageDecodeRegistry {
    fn new(decoder: Arc<dyn ScriptImageDecoder>) -> Self {
        Self {
            next_request_id: Cell::new(1),
            owner_id: nuxie_runtime::next_work_owner_id(),
            decoder,
            pending: RefCell::new(HashMap::new()),
            completions: Arc::new(Mutex::new(VecDeque::new())),
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
        if let Some(pending) = self.pending.borrow_mut().remove(&request_id) {
            pending.task.state().cancel();
        }
    }

    fn cancel_all(&self) {
        for pending in self
            .pending
            .borrow_mut()
            .drain()
            .map(|(_, pending)| pending)
        {
            pending.task.state().cancel();
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
        .ok_or_else(|| luaur_rt::Error::runtime("image decode registry is not installed"))
}

pub(super) fn start(lua: &Lua, encoded: Buffer) -> Result<AnyUserData> {
    if encoded.is_empty() {
        return Err(luaur_rt::Error::runtime("decodeImage: empty buffer"));
    }

    let registry = registry(lua)?;
    let promise = lua_promise::new_pending(lua)?;
    let promise_ref = lua.create_registry_value(promise.clone())?;
    let request_id = registry.next_request_id();
    let task = Arc::new(ImageDecodeTask::new(
        request_id,
        registry.owner_id,
        Arc::clone(&registry.decoder),
        encoded.to_vec(),
        Arc::clone(&registry.completions),
    ));
    registry.pending.borrow_mut().insert(
        request_id,
        PendingDecode {
            promise: promise_ref,
            task: Arc::clone(&task),
        },
    );

    let cancel_registry = Rc::clone(&registry);
    let on_cancel = lua.create_function(move |_, ()| {
        cancel_registry.cancel(request_id);
        Ok(())
    })?;
    lua_promise::set_on_cancel(lua, promise.clone(), on_cancel)?;
    with_global_work_pool(|pool| {
        pool.submit(Some(task));
    });
    Ok(promise)
}

pub(super) fn poll_completed(lua: &Lua) -> Result<bool> {
    let Some(registry) = lua
        .app_data_ref::<ImageDecodeRegistryOwner>()
        .map(|owner| Rc::clone(&owner.0))
    else {
        return Ok(false);
    };
    let completions = lock_unpoisoned(&registry.completions)
        .drain(..)
        .collect::<Vec<_>>();
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
    Ok(settled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use luaur_rt::Table;
    use nuxie_runtime::ScriptInstance;

    use crate::vm::LuaScriptInstance;
    use crate::vm::view_model::ScriptedContext;

    fn lua_with_context() -> Lua {
        let lua = Lua::new();
        lua_promise::install_promise_globals(&lua).unwrap();
        install(&lua);
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::new(RefCell::new(None)),
                Vec::new(),
                Rc::new(Cell::new(false)),
                None,
            ))
            .unwrap();
        lua.globals().set("context", context).unwrap();
        lua
    }

    fn drain_work(lua: &Lua) {
        for _ in 0..10_000 {
            with_global_work_pool(|pool| {
                pool.poll_completed_work(16);
            });
            poll_completed(lua).unwrap();
            if registry(lua).unwrap().pending.borrow().is_empty() {
                return;
            }
            std::thread::yield_now();
        }
        panic!("image decode did not settle");
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

        for _ in 0..10_000 {
            with_global_work_pool(|pool| {
                pool.poll_completed_work(16);
            });
            if instance.poll_async_work().unwrap() {
                break;
            }
            std::thread::yield_now();
        }

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

        for _ in 0..10_000 {
            with_global_work_pool(|pool| {
                pool.poll_completed_work(16);
            });
            if !lock_unpoisoned(&registry(&lua).unwrap().completions).is_empty() {
                break;
            }
            std::thread::yield_now();
        }
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
