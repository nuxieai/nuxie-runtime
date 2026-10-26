//! Browser host boundary. Only copied bytes and shared-object descriptors cross
//! Wasm instances; Lua values, Arc addresses and allocator state never do.
use serde_json::Value;

#[link(wasm_import_module = "nuxie_audio")]
unsafe extern "C" {
    fn host_call(op: u32, input: *const u8, length: usize, output: *mut u8, capacity: usize)
    -> i32;
    fn now_ms() -> f64;
}

pub(super) fn request(value: Value) -> Result<Value, String> {
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    let mut output = vec![0; 1024];
    // A short output buffer must not execute the operation twice: operation 2
    // drains the response cached by the host on the same synchronous call stack.
    let mut length = unsafe {
        host_call(
            1,
            bytes.as_ptr(),
            bytes.len(),
            output.as_mut_ptr(),
            output.len(),
        )
    };
    if length < 0 {
        let required = length
            .checked_neg()
            .ok_or("invalid browser audio response")? as usize;
        output.resize(required, 0);
        length = unsafe { host_call(2, std::ptr::null(), 0, output.as_mut_ptr(), output.len()) };
    }
    if length < 0 || length as usize > output.len() {
        return Err("invalid browser audio response length".into());
    }
    let result: Value =
        serde_json::from_slice(&output[..length as usize]).map_err(|e| e.to_string())?;
    if result["ok"].as_bool() != Some(true) {
        return Err(result["error"]
            .as_str()
            .unwrap_or("browser audio host failed")
            .into());
    }
    Ok(result["value"].clone())
}

/// Browser worklets may lack `performance`; the host supplies the same
/// performance.now / Date.now selection as upstream's lane implementation.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(super) struct Instant(f64);
impl Instant {
    pub fn now() -> Self {
        Self(unsafe { now_ms() })
    }
}
impl std::ops::Add<std::time::Duration> for Instant {
    type Output = Self;
    fn add(self, value: std::time::Duration) -> Self {
        Self(self.0 + value.as_secs_f64() * 1000.0)
    }
}

use super::luau_audio_processor_engine::BrowserKernel;
use std::cell::RefCell;
thread_local! {
    static KERNEL: RefCell<Option<BrowserKernel>> = const { RefCell::new(None) };
    static RESPONSE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// ABI buffers are owned by the caller until it invokes free with the same size.
pub fn alloc(length: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; length].into_boxed_slice()).cast::<u8>()
}
/// # Safety
/// `pointer` and `length` must be the exact live allocation returned by alloc.
pub unsafe fn free(pointer: *mut u8, length: usize) {
    unsafe {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            pointer, length,
        )));
    }
}
/// # Safety
/// The input must name readable memory in this instance for its full length.
pub unsafe fn kernel_request(pointer: *const u8, length: usize) -> *const u8 {
    let result = (|| -> Result<Value, String> {
        let bytes = unsafe { std::slice::from_raw_parts(pointer, length) };
        let request: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        KERNEL.with(|slot| {
            let mut slot = slot.borrow_mut();
            match request["op"].as_str() {
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
    RESPONSE.with(|slot| {
        let mut slot = slot.borrow_mut();
        *slot = serde_json::to_vec(&value).expect("JSON response");
        slot.as_ptr()
    })
}
pub fn response_len() -> usize {
    RESPONSE.with(|slot| slot.borrow().len())
}
pub fn render(frames: u32, low: u32, high: u32) -> *const f32 {
    KERNEL.with(|slot| {
        slot.borrow_mut().as_mut().map_or(std::ptr::null(), |lane| {
            lane.render(frames, u64::from(low) | (u64::from(high) << 32))
        })
    })
}

/// Wake the main runtime after a worklet state/log event. Embedders attach this
/// export to createAudioHost so idle artboards also observe asynchronous errors.
pub fn notify(id: u32) {
    super::luau_audio_processor_engine::browser_notify(id);
}
