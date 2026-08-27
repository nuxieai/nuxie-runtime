#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::lua::rive_lua_libs::{
    LuaState, ScriptedPromise, ScriptingContext,
};
#[cfg(all(not(target_family = "wasm"), feature = "rive_decoders"))]
use crate::mechanical_port::source::{
    async_work::{WorkPool, WorkTask},
    decoders::bitmap_decoder::{Bitmap, PixelFormat},
};
#[cfg(all(not(target_family = "wasm"), feature = "rive_decoders"))]
pub struct ImageDecodeTask {
    encoded_data: Vec<u8>,
    width: u32,
    height: u32,
    state: Option<*mut LuaState>,
    promise_ref: Option<i32>,
    bitmap: Option<Bitmap>,
    error_message: String,
    owner_id: u64,
}
#[cfg(all(not(target_family = "wasm"), feature = "rive_decoders"))]
impl WorkTask for ImageDecodeTask {
    fn execute(&mut self) -> bool {
        let Some(mut bitmap) = Bitmap::decode(&self.encoded_data) else {
            self.error_message = "failed to decode image data".into();
            return false;
        };
        if bitmap.pixel_format() != PixelFormat::RgbaPremul {
            bitmap.set_pixel_format(PixelFormat::RgbaPremul);
        }
        self.width = bitmap.width();
        self.height = bitmap.height();
        self.bitmap = Some(bitmap);
        true
    }
    fn on_complete(&mut self) {
        let (Some(state), Some(reference)) = (self.state, self.promise_ref) else {
            return;
        };
        let state = unsafe { &mut *state };
        let mut promise = state.registry_rive::<ScriptedPromise>(reference);
        if promise.as_ref().is_some_and(ScriptedPromise::is_pending) {
            let bitmap = self.bitmap.as_ref().unwrap();
            if bitmap.pixel_format() != PixelFormat::RgbaPremul {
                state.push_string("internal error: decoded image is not RGBAPremul");
                promise.as_mut().unwrap().reject(state, state.top());
                state.pop(1);
            } else {
                state.new_table();
                state.new_buffer(bitmap.bytes());
                state.set_field(-2, "data");
                state.push_number(self.width as f64);
                state.set_field(-2, "width");
                state.push_number(self.height as f64);
                state.set_field(-2, "height");
                promise.as_mut().unwrap().resolve(state, state.top());
                state.pop(1);
            }
        }
        self.bitmap = None;
        self.encoded_data.clear();
        self.encoded_data.shrink_to_fit();
        state.unref(reference);
        self.promise_ref = None;
    }
    fn on_error(&mut self, error: &str) {
        let (Some(state), Some(reference)) = (self.state, self.promise_ref) else {
            return;
        };
        let state = unsafe { &mut *state };
        if let Some(mut promise) = state
            .registry_rive::<ScriptedPromise>(reference)
            .filter(ScriptedPromise::is_pending)
        {
            state.push_string(error);
            promise.reject(state, state.top());
            state.pop(1);
        }
        state.unref(reference);
        self.promise_ref = None;
    }
    fn on_cancel(&mut self) {
        self.promise_ref = None;
        self.state = None;
    }
}
#[cfg(not(target_family = "wasm"))]
pub fn context_decode_image_impl(state: &mut LuaState) -> i32 {
    let Some(data) = state.to_buffer(2).map(<[u8]>::to_vec) else {
        return state.type_error(2, "buffer");
    };
    if data.is_empty() {
        return state.error("decodeImage: empty buffer");
    }
    #[cfg(not(feature = "rive_decoders"))]
    {
        return state.error("decodeImage: not supported on this platform");
    }
    #[cfg(feature = "rive_decoders")]
    {
        let context = state.thread_data::<dyn ScriptingContext>();
        let main = state.main_thread();
        let promise = ScriptedPromise::new(main);
        let promise_index = state.new_rive(promise);
        state.push_value(promise_index);
        let promise_ref = state.reference(-1);
        state.pop(1);
        let task = ImageDecodeTask {
            encoded_data: data,
            width: 0,
            height: 0,
            state: Some(main),
            promise_ref: Some(promise_ref),
            bitmap: None,
            error_message: String::new(),
            owner_id: context.owner_id(),
        };
        let task_ref = context.work_pool().submit_retained(task);
        state
            .to_rive_mut::<ScriptedPromise>(promise_index)
            .set_on_cancel(move |state| {
                task_ref.cancel();
                state.unref(promise_ref);
            });
        state.push_value(promise_index);
        1
    }
}
#[cfg(target_family = "wasm")]
use std::{cell::RefCell, collections::HashMap};
#[cfg(target_family = "wasm")]
struct PendingDecode {
    state: *mut LuaState,
    promise_ref: i32,
}
#[cfg(target_family = "wasm")]
thread_local! {static PENDING:RefCell<HashMap<u32,PendingDecode>>=RefCell::new(HashMap::new());static NEXT:RefCell<u32>=const{RefCell::new(1)};}
#[cfg(target_family = "wasm")]
fn next_decode_id() -> u32 {
    NEXT.with(|next| {
        PENDING.with(|pending| {
            loop {
                let id = *next.borrow();
                *next.borrow_mut() = id.wrapping_add(1);
                if id != 0 && !pending.borrow().contains_key(&id) {
                    return id;
                }
            }
        })
    })
}
#[cfg(target_family = "wasm")]
pub fn wasm_image_decode_complete(id: u32, width: i32, height: i32, mut pixels: Vec<u8>) {
    let Some(pending) = PENDING.with(|p| p.borrow_mut().remove(&id)) else {
        return;
    };
    let state = unsafe { &mut *pending.state };
    if let Some(mut promise) = state
        .registry_rive::<ScriptedPromise>(pending.promise_ref)
        .filter(ScriptedPromise::is_pending)
    {
        for rgba in pixels.chunks_exact_mut(4) {
            let a = rgba[3];
            if a < 255 {
                for channel in &mut rgba[..3] {
                    *channel = ((*channel as u16 * a as u16 + 127) / 255) as u8;
                }
            }
        }
        state.new_table();
        state.new_buffer(&pixels);
        state.set_field(-2, "data");
        state.push_number(width as f64);
        state.set_field(-2, "width");
        state.push_number(height as f64);
        state.set_field(-2, "height");
        promise.resolve(state, state.top());
        state.pop(1);
    }
    state.unref(pending.promise_ref);
}
#[cfg(target_family = "wasm")]
pub fn wasm_image_decode_error(id: u32, message: &str) {
    let Some(pending) = PENDING.with(|p| p.borrow_mut().remove(&id)) else {
        return;
    };
    let state = unsafe { &mut *pending.state };
    if let Some(mut promise) = state
        .registry_rive::<ScriptedPromise>(pending.promise_ref)
        .filter(ScriptedPromise::is_pending)
    {
        state.push_string(message);
        promise.reject(state, state.top());
        state.pop(1);
    }
    state.unref(pending.promise_ref);
}
#[cfg(target_family = "wasm")]
pub fn wasm_cancel_pending_decodes(main: *mut LuaState) {
    PENDING.with(|p| {
        p.borrow_mut().retain(|_, pending| {
            if pending.state == main {
                unsafe { &mut *main }.unref(pending.promise_ref);
                false
            } else {
                true
            }
        })
    });
}
#[cfg(target_family = "wasm")]
pub fn context_decode_image_impl(state: &mut LuaState) -> i32 {
    let Some(data) = state.to_buffer(2).map(<[u8]>::to_vec) else {
        return state.type_error(2, "buffer");
    };
    if data.is_empty() {
        return state.error("decodeImage: empty buffer");
    }
    let main = state.main_thread();
    let promise_index = state.new_rive(ScriptedPromise::new(main));
    state.push_value(promise_index);
    let promise_ref = state.reference(-1);
    state.pop(1);
    let id = next_decode_id();
    PENDING.with(|p| {
        p.borrow_mut().insert(
            id,
            PendingDecode {
                state: main,
                promise_ref,
            },
        )
    });
    wasm_start_image_decode(id, &data);
    state
        .to_rive_mut::<ScriptedPromise>(promise_index)
        .set_on_cancel(move |state| {
            PENDING.with(|p| {
                p.borrow_mut().remove(&id);
            });
            state.unref(promise_ref);
        });
    state.push_value(promise_index);
    1
}
