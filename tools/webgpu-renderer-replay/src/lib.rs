//! Browser-only replay root for the exact WebGPU renderer campaign.

#[cfg(target_arch = "wasm32")]
mod wasm {
    use nuxie_render_stream::RenderStream;
    use nuxie_renderer::{NativeWebGpuFactory, RenderMode};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WebGpuReplayResult {
        width: u32,
        height: u32,
        adapter: String,
    }

    #[wasm_bindgen]
    impl WebGpuReplayResult {
        #[wasm_bindgen(getter)]
        pub fn width(&self) -> u32 {
            self.width
        }

        #[wasm_bindgen(getter)]
        pub fn height(&self) -> u32 {
            self.height
        }

        #[wasm_bindgen(getter)]
        pub fn adapter(&self) -> String {
            self.adapter.clone()
        }
    }

    #[wasm_bindgen]
    pub fn run_webgpu_replay(
        stream_text: &str,
        mode: &str,
        frame_index: usize,
    ) -> Result<WebGpuReplayResult, JsValue> {
        let stream = RenderStream::parse(stream_text).map_err(js_error)?;
        let (width, height) = stream
            .frame_size
            .ok_or_else(|| JsValue::from_str("recorded stream does not declare frameSize"))?;
        let clear = stream.clear_color.unwrap_or(0);
        let mode = match mode {
            "msaa" => RenderMode::Msaa,
            "clockwise-atomic" => RenderMode::ClockwiseAtomic,
            value => {
                return Err(JsValue::from_str(&format!(
                    "unsupported exact WebGPU mode `{value}`"
                )));
            }
        };
        let mut factory = NativeWebGpuFactory::new(width, height).map_err(js_error)?;
        let adapter = factory.adapter_name().to_owned();
        let mut frame = factory.begin_frame(clear, mode).map_err(js_error)?;
        stream
            .replay_frame(frame_index, &mut factory, &mut frame)
            .map_err(js_error)?;
        frame.finish_present().map_err(js_error)?;
        Ok(WebGpuReplayResult {
            width,
            height,
            adapter,
        })
    }

    fn js_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{WebGpuReplayResult, run_webgpu_replay};
