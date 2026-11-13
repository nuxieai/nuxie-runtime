//! Browser host boundary. Only copied bytes and shared-object descriptors cross
//! Wasm instances; Lua values, Arc addresses and allocator state never do.
use serde_json::Value;

mod host;
pub use host::{alloc, free, kernel_request};

/// Processor execution deadlines need the host clock. Hostless builds refuse
/// every processor, offline lanes included, instead of running unbounded.
pub(super) fn require_host() -> Result<(), String> {
    if cfg!(feature = "browser-audio-host") {
        Ok(())
    } else {
        Err("audio needs the browser audio host".into())
    }
}

pub(super) fn request(value: Value) -> Result<Value, String> {
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    let mut output = vec![0; 1024];
    // A short output buffer must not execute the operation twice: operation 2
    // drains the response cached by the host on the same synchronous call stack.
    let mut length = host::call(1, &bytes, &mut output);
    if length < 0 {
        let required = length
            .checked_neg()
            .ok_or("invalid browser audio response")? as usize;
        output
            .try_reserve_exact(required.saturating_sub(output.len()))
            .map_err(|_| "cannot allocate browser audio response")?;
        output.resize(required, 0);
        length = host::call(2, &[], &mut output);
    }
    let response = usize::try_from(length)
        .ok()
        .and_then(|length| output.get(..length))
        .ok_or("invalid browser audio response length")?;
    let result: Value = serde_json::from_slice(response).map_err(|e| e.to_string())?;
    if result.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err(result
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("browser audio host failed")
            .into());
    }
    Ok(result.get("value").cloned().unwrap_or_default())
}

/// Binary shared operations avoid JSON serialization on the audio callback.
pub(super) fn raw_shared(
    op: u32,
    owner: u32,
    id: u32,
    bytes: &mut [u8],
    offset: u32,
    value: u32,
) -> Result<usize, String> {
    let result = host::shared(op, owner, id, bytes, offset, value);
    if result < 0.0 {
        Err("invalid or closed shared audio transport".into())
    } else {
        Ok(result as usize)
    }
}

/// Browser worklets may lack `performance`; the host supplies the same
/// performance.now / Date.now selection as upstream's lane implementation.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(super) struct Instant(f64);
impl Instant {
    pub fn now() -> Self {
        Self(host::now())
    }
    /// Never `None`: the clock is a float. Matches `std::time::Instant`.
    pub fn checked_add(self, value: std::time::Duration) -> Option<Self> {
        Some(Self(self.0 + value.as_secs_f64() * 1000.0))
    }
}

use super::luau_audio_processor_engine::BrowserKernel;
use std::cell::RefCell;
thread_local! {
    static KERNEL: RefCell<Option<BrowserKernel>> = const { RefCell::new(None) };
    static RESPONSE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// One kernel request (`host::kernel_request`): runs the JSON request in
/// `bytes` and returns the address of its JSON response, `response_len()`
/// bytes kept until the next request.
fn run_kernel_request(bytes: &[u8]) -> *const u8 {
    let result = (|| -> Result<Value, String> {
        let request: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        KERNEL.with(|slot| {
            // Starting a processor runs its script; a request arriving from
            // inside that (a re-entrant host call) is refused, not a panic.
            let mut slot = slot
                .try_borrow_mut()
                .map_err(|_| "processor kernel is busy")?;
            match request.get("op").and_then(Value::as_str) {
                Some("start") => {
                    if slot.is_some() {
                        return Err("processor already initialized".into());
                    }
                    *slot = Some(BrowserKernel::start(&request)?);
                    Ok(Value::Null)
                }
                Some("poll") => Ok(slot.as_ref().map(BrowserKernel::poll).unwrap_or_else(
                    || serde_json::json!({"state":"stopped","error":null,"logs":[]}),
                )),
                Some("stop") => {
                    *slot = None;
                    Ok(Value::Null)
                }
                _ => Err("unknown processor kernel operation".into()),
            }
        })
    })();
    let value = match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value}),
        Err(error) => serde_json::json!({"ok":false,"error":error}),
    };
    // A `Value` has string keys only, so it always serializes.
    let bytes = serde_json::to_vec(&value)
        .unwrap_or_else(|_| br#"{"ok":false,"error":"cannot encode response"}"#.to_vec());
    RESPONSE.with(|slot| match slot.try_borrow_mut() {
        Ok(mut slot) => {
            *slot = bytes;
            slot.as_ptr()
        }
        Err(_) => std::ptr::null(),
    })
}
pub fn response_len() -> usize {
    RESPONSE.with(|slot| slot.try_borrow().map_or(0, |slot| slot.len()))
}
/// Renders `frames` frames; null when nothing was rendered (no processor, a
/// failed one, or a call from inside a running processor).
pub fn render(frames: u32, low: u32, high: u32) -> *const f32 {
    KERNEL.with(|slot| {
        let Ok(mut slot) = slot.try_borrow_mut() else {
            return std::ptr::null();
        };
        slot.as_mut().map_or(std::ptr::null(), |lane| {
            lane.render(frames, u64::from(low) | (u64::from(high) << 32))
        })
    })
}

/// Wake the main runtime after a worklet state/log event. Embedders attach this
/// export to createAudioHost so idle artboards also observe asynchronous errors.
pub fn notify(id: u32) {
    super::luau_audio_processor_engine::browser_notify(id);
}
