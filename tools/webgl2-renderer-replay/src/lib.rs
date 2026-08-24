//! Browser-only replay root for the exact WebGL2 renderer campaign.

#[cfg(target_arch = "wasm32")]
mod wasm {
    use nuxie_render_stream::RenderStream;
    use nuxie_renderer::{RenderMode, WebGl2Factory};
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlCanvasElement;

    #[wasm_bindgen]
    pub struct WebGl2ReplayResult {
        width: u32,
        height: u32,
        png: Vec<u8>,
        adapter: String,
    }

    #[wasm_bindgen]
    impl WebGl2ReplayResult {
        #[wasm_bindgen(getter)]
        pub fn width(&self) -> u32 {
            self.width
        }

        #[wasm_bindgen(getter)]
        pub fn height(&self) -> u32 {
            self.height
        }

        #[wasm_bindgen(getter)]
        pub fn png(&self) -> Vec<u8> {
            self.png.clone()
        }

        #[wasm_bindgen(getter)]
        pub fn adapter(&self) -> String {
            self.adapter.clone()
        }
    }

    #[wasm_bindgen]
    pub fn run_webgl2_replay(
        canvas: HtmlCanvasElement,
        stream_text: &str,
        mode: &str,
        frame_index: usize,
    ) -> Result<WebGl2ReplayResult, JsValue> {
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
                    "unsupported exact WebGL2 mode `{value}`"
                )));
            }
        };
        let mut factory = WebGl2Factory::new(canvas, width, height).map_err(js_error)?;
        let adapter = factory.adapter_name().to_owned();
        let mut frame = factory.begin_frame(clear, mode).map_err(js_error)?;
        stream
            .replay_frame(frame_index, &mut factory, &mut frame)
            .map_err(js_error)?;
        let mut pixels = frame.finish().map_err(js_error)?;
        flip_rows(&mut pixels, width, height);
        let png = encode_png(width, height, &pixels).map_err(js_error)?;
        Ok(WebGl2ReplayResult {
            width,
            height,
            png,
            adapter,
        })
    }

    fn encode_png(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>, png::EncodingError> {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(pixels)?;
        }
        Ok(encoded)
    }

    fn flip_rows(pixels: &mut [u8], width: u32, height: u32) {
        let row_bytes = width as usize * 4;
        for y in 0..height as usize / 2 {
            let top = y * row_bytes;
            let bottom = (height as usize - 1 - y) * row_bytes;
            let (prefix, suffix) = pixels.split_at_mut(bottom);
            prefix[top..top + row_bytes].swap_with_slice(&mut suffix[..row_bytes]);
        }
    }

    fn js_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{WebGl2ReplayResult, run_webgl2_replay};
