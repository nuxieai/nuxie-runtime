//! One worklet lane. Initialization is invoked by the worklet's message handler;
//! only render() is invoked by its AudioWorkletProcessor.process callback.
use super::*;
use serde_json::Value;

pub(in super::super) struct BrowserKernel {
    processor: RawProcessor,
    output: Vec<f32>,
}
impl BrowserKernel {
    pub(in super::super) fn start(value: &Value) -> Result<Self, String> {
        // Each worklet has a private Wasm instance, so it cannot inherit the
        // main Lua::new profile required by the pinned compiler's bytecode.
        luaur_common::set_all_flags(true);
        let modules = value["modules"]
            .as_object()
            .ok_or("missing processor modules")?
            .iter()
            .map(|(name, bytes)| {
                serde_json::from_value::<Vec<u8>>(bytes.clone())
                    .map(|bytes| (name.clone(), Arc::from(bytes)))
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let name = value["name"]
            .as_str()
            .ok_or("missing processor name")?
            .to_owned();
        let number = |key: &str| -> Result<u32, String> {
            value["format"][key]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .filter(|v| *v > 0)
                .ok_or_else(|| format!("invalid processor {key}"))
        };
        let rate = number("sampleRate")?;
        let channels = number("channels")?;
        let max_frames = number("maxFrames")?;
        let count = (channels as usize)
            .checked_mul(max_frames as usize)
            .ok_or("processor output too large")?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(|_| "cannot allocate processor output")?;
        output.resize(count, 0.0);
        let input = super::super::luau_audio_shared::decode_wire(value["input"].clone())?;
        let hard_blocks = value["hardBlocks"]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .ok_or("invalid hardBlocks")?;
        let memory_limit = value["memoryLimit"]
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or("invalid memoryLimit")?;
        let processor = RawProcessor::new(
            Arc::new(Shared::new(RUNNING)),
            name,
            modules,
            input,
            rate,
            channels,
            max_frames,
            hard_blocks,
            memory_limit,
        )?;
        Ok(Self { processor, output })
    }
    pub(in super::super) fn render(&mut self, frames: u32, start_frame: u64) -> *const f32 {
        if frames > self.processor.max_frames {
            self.processor
                .context
                .shared
                .fail("audio format exceeds initialized processor format".into());
            return std::ptr::null();
        }
        if self.processor.context.shared.state.load(Ordering::Acquire) != RUNNING {
            return std::ptr::null();
        }
        let count = frames as usize * self.processor.channels as usize;
        self.processor
            .render(&mut self.output[..count], frames, start_frame);
        if self.processor.context.shared.state.load(Ordering::Acquire) != RUNNING {
            std::ptr::null()
        } else {
            self.output.as_ptr()
        }
    }
    pub(in super::super) fn poll(&self) -> Value {
        let shared = &self.processor.context.shared;
        shared.poll_notification();
        serde_json::json!({"state":if shared.state.load(Ordering::Acquire) == RUNNING {"running"} else {"failed"},
            "error":shared.error.get(), "logs":shared.logs.take()})
    }
}
impl Drop for BrowserKernel {
    fn drop(&mut self) {
        self.processor.context.shared.retire();
    }
}
