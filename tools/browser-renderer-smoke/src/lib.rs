#[cfg(target_arch = "wasm32")]
mod wasm {
    use nuxie::{
        BlendMode, BrowserBackendPreference, BrowserFactory, Factory, FillRule, ImageFilter,
        ImageSampler, ImageWrap, Mat2D, RecordingFactory, Renderer,
    };
    use nuxie_render_stream::RenderStream;
    use pixel_compare::{RgbaImage, Tolerance, compare};
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlCanvasElement;

    #[wasm_bindgen]
    pub async fn run_backend(
        canvas: HtmlCanvasElement,
        backend: String,
    ) -> Result<String, JsValue> {
        let preference = match backend.as_str() {
            "webgpu" => BrowserBackendPreference::WebGpu,
            "webgl2" => BrowserBackendPreference::WebGl2,
            "auto" => BrowserBackendPreference::Auto,
            value => return Err(JsValue::from_str(&format!("unknown backend {value}"))),
        };
        let mut factory = BrowserFactory::new(canvas, 64, 64, preference)
            .await
            .map_err(js_error)?;
        let selected = format!("{:?}", factory.backend()).to_ascii_lowercase();
        let fallback = factory.fallback_reason().is_some();

        let mut clip = factory.make_empty_render_path();
        clip.move_to(16.0, 8.0);
        clip.line_to(48.0, 8.0);
        clip.line_to(48.0, 56.0);
        clip.line_to(16.0, 56.0);
        clip.close();
        clip.fill_rule(FillRule::NonZero);

        let mut path = factory.make_empty_render_path();
        path.move_to(8.0, 8.0);
        path.line_to(56.0, 8.0);
        path.line_to(56.0, 56.0);
        path.line_to(8.0, 56.0);
        path.close();
        let shader = factory.make_linear_gradient(
            8.0,
            32.0,
            56.0,
            32.0,
            &[0xffff_0000, 0xff00_ff00],
            &[0.0, 1.0],
        );
        let mut paint = factory.make_render_paint();
        paint.shader(Some(shader.as_ref()));

        let mut transform_probe = factory.make_empty_render_path();
        transform_probe.move_to(0.0, 0.0);
        transform_probe.line_to(4.0, 0.0);
        transform_probe.line_to(4.0, 4.0);
        transform_probe.line_to(0.0, 4.0);
        transform_probe.close();
        let mut transform_paint = factory.make_render_paint();
        transform_paint.color(0xff33_66cc);

        let mut frame = factory.begin_frame(0xff10_2030).map_err(js_error)?;
        frame.save();
        frame.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 2.0]));
        frame.transform(Mat2D([2.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
        frame.draw_path(transform_probe.as_ref(), transform_paint.as_ref());
        frame.restore();
        frame.clip_path(clip.as_ref());
        frame.draw_path(path.as_ref(), paint.as_ref());
        let pixels = frame.finish().await.map_err(js_error)?;

        assert_pixels(&pixels)?;
        Ok(format!(
            "backend={selected} fallback={fallback} checksum={:016x}",
            fnv1a64(&pixels)
        ))
    }

    #[wasm_bindgen]
    pub fn recording_float_probe() -> String {
        let mut factory = RecordingFactory::new();
        factory.add_sample(0.1);
        factory.stream()
    }

    #[wasm_bindgen]
    pub async fn assert_webgl2_fail_closed(
        canvas: HtmlCanvasElement,
        abandoned_canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas, 16, 16, BrowserBackendPreference::WebGl2)
            .await
            .map_err(js_error)?;
        let mut triangle = factory.make_empty_render_path();
        triangle.move_to(0.0, 0.0);
        triangle.line_to(16.0, 0.0);
        triangle.line_to(8.0, 16.0);
        triangle.close();

        let mut frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        frame.clip_path(triangle.as_ref());
        match frame.finish().await {
            Err(error)
                if error
                    .to_string()
                    .contains("non-rectangular WebGL2 clip paths") => {}
            Err(error) => Err(JsValue::from_str(&format!(
                "unexpected unsupported-capability error: {error}"
            )))?,
            Ok(_) => Err(JsValue::from_str(
                "unsupported WebGL2 capability rendered without an error",
            ))?,
        }

        let unsupported_sampler = ImageSampler {
            wrap_x: ImageWrap::Repeat,
            wrap_y: ImageWrap::Mirror,
            filter: ImageFilter::Nearest,
        };
        let mut recovery_frame = factory.begin_frame(0xff12_3456).map_err(js_error)?;
        recovery_frame.draw_image(None, unsupported_sampler, BlendMode::Multiply, 1.0);
        recovery_frame.draw_image_mesh(
            None,
            unsupported_sampler,
            None,
            None,
            None,
            0,
            0,
            BlendMode::Multiply,
            1.0,
        );
        let recovery = recovery_frame.finish().await.map_err(js_error)?;
        if !recovery
            .chunks_exact(4)
            .all(|pixel| pixel == [0x12, 0x34, 0x56, 0xff])
        {
            return Err(JsValue::from_str(
                "an unsupported WebGL2 frame contaminated the next frame",
            ));
        }

        let mut rect = factory.make_empty_render_path();
        rect.move_to(0.0, 0.0);
        rect.line_to(12.0, 0.0);
        rect.line_to(12.0, 12.0);
        rect.line_to(0.0, 12.0);
        rect.close();
        let mut frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        frame.clip_path(rect.as_ref());
        frame.transform(Mat2D([
            0.70710677,
            0.70710677,
            -0.70710677,
            0.70710677,
            8.0,
            0.0,
        ]));
        frame.clip_path(rect.as_ref());
        match frame.finish().await {
            Err(error)
                if error
                    .to_string()
                    .contains("incompatible transformed WebGL2 clip rectangles") => {}
            Err(error) => Err(JsValue::from_str(&format!(
                "unexpected transformed-clip error: {error}"
            )))?,
            Ok(_) => Err(JsValue::from_str(
                "incompatible transformed WebGL2 clips rendered approximately",
            ))?,
        }

        let abandoned_factory =
            BrowserFactory::new(abandoned_canvas, 4, 4, BrowserBackendPreference::WebGl2)
                .await
                .map_err(js_error)?;
        drop(
            abandoned_factory
                .begin_frame(0xff00_0000)
                .map_err(js_error)?,
        );
        match abandoned_factory.begin_frame(0xff00_0000) {
            Err(error) if error.to_string().contains("abandoned frame") => {}
            Err(error) => {
                return Err(JsValue::from_str(&format!(
                    "unexpected abandoned-frame error: {error}"
                )));
            }
            Ok(_) => {
                return Err(JsValue::from_str(
                    "WebGL2 renderer accepted work after an abandoned frame",
                ));
            }
        }

        Ok("unsupported=fail-closed recovery=clean abandoned=poisoned".into())
    }

    #[wasm_bindgen]
    pub async fn run_stream_case(
        canvas: HtmlCanvasElement,
        stream_name: String,
        backend: String,
        stream_text: String,
        reference_png: Vec<u8>,
        max_channel_delta: u8,
        max_different_pixels: u32,
        expected_edge_radius: u32,
        max_off_edge_different_pixels: u32,
    ) -> Result<String, JsValue> {
        let preference = match backend.as_str() {
            "webgpu" => BrowserBackendPreference::WebGpu,
            "webgl2" => BrowserBackendPreference::WebGl2,
            value => return Err(JsValue::from_str(&format!("unknown backend {value}"))),
        };
        let stream = RenderStream::parse(&stream_text).map_err(js_error)?;
        let (width, height) = stream
            .frame_size
            .ok_or_else(|| JsValue::from_str("stream does not declare frameSize"))?;
        let clear = stream.clear_color.unwrap_or(0);
        let mut factory = BrowserFactory::new(canvas, width, height, preference)
            .await
            .map_err(js_error)?;
        let mut frame = factory.begin_frame(clear).map_err(js_error)?;
        stream
            .replay_frame(0, &mut factory, &mut frame)
            .map_err(js_error)?;
        let pixels = frame.finish().await.map_err(js_error)?;
        let actual = RgbaImage::new(width, height, pixels).map_err(js_error)?;
        let expected = RgbaImage::decode_png(&reference_png).map_err(js_error)?;
        let report = compare(
            &expected,
            &actual,
            Tolerance {
                max_channel_delta,
                max_different_pixels: u64::from(max_different_pixels),
            },
        )
        .map_err(js_error)?;
        let edge_mask = expected_edge_mask(&expected, max_channel_delta, expected_edge_radius);
        let mut off_edge_different_pixels = 0u64;
        let mut channel_max = [0u8; 4];
        let mut channel_different = [0u64; 4];
        for (pixel_index, (expected, actual)) in expected
            .pixels
            .chunks_exact(4)
            .zip(actual.pixels.chunks_exact(4))
            .enumerate()
        {
            let mut pixel_differs = false;
            for channel in 0..4 {
                let delta = expected[channel].abs_diff(actual[channel]);
                channel_max[channel] = channel_max[channel].max(delta);
                if delta > max_channel_delta {
                    channel_different[channel] += 1;
                    pixel_differs = true;
                }
            }
            if pixel_differs && !edge_mask[pixel_index] {
                off_edge_different_pixels += 1;
            }
        }
        let diagnostic_name = stream_name.strip_prefix("gm-").unwrap_or(&stream_name);
        for (x, y) in diagnostic_points(diagnostic_name) {
            let expected_pixel = pixel_at(&expected, x, y)
                .ok_or_else(|| JsValue::from_str("semantic probe is outside the reference"))?;
            let actual_pixel = pixel_at(&actual, x, y)
                .ok_or_else(|| JsValue::from_str("semantic probe is outside the output"))?;
            if expected_pixel
                .iter()
                .zip(actual_pixel)
                .any(|(expected, actual)| expected.abs_diff(actual) > max_channel_delta)
            {
                return Err(JsValue::from_str(&format!(
                    "stream={stream_name} backend={backend} semantic probe ({x},{y}) differs: expected={expected_pixel:?} actual={actual_pixel:?}"
                )));
            }
        }
        let result = format!(
            "stream={stream_name} backend={backend} different={} off-edge={} max-delta={} channel-max={channel_max:?} channel-different={channel_different:?} checksum={:016x}",
            report.different_pixels,
            off_edge_different_pixels,
            report.max_channel_delta,
            fnv1a64(&actual.pixels)
        );
        if !report.within_tolerance
            || off_edge_different_pixels > u64::from(max_off_edge_different_pixels)
        {
            return Err(JsValue::from_str(&format!(
                "{result} exceeds tolerance delta={max_channel_delta} pixels={max_different_pixels} off-edge={max_off_edge_different_pixels}"
            )));
        }
        Ok(result)
    }

    fn expected_edge_mask(image: &RgbaImage, delta: u8, radius: u32) -> Vec<bool> {
        let width = image.width as usize;
        let height = image.height as usize;
        let mut edges = vec![false; width * height];
        let pixel = |x: usize, y: usize| &image.pixels[(y * width + x) * 4..][..4];

        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                    if nx >= width || ny >= height {
                        continue;
                    }
                    if pixel(x, y)
                        .iter()
                        .zip(pixel(nx, ny))
                        .any(|(left, right)| left.abs_diff(*right) > delta)
                    {
                        edges[index] = true;
                        edges[ny * width + nx] = true;
                    }
                }
            }
        }

        if radius == 0 {
            return edges;
        }
        let mut dilated = vec![false; edges.len()];
        let radius = radius as usize;
        for y in 0..height {
            for x in 0..width {
                if !edges[y * width + x] {
                    continue;
                }
                let min_y = y.saturating_sub(radius);
                let max_y = (y + radius).min(height - 1);
                let min_x = x.saturating_sub(radius);
                let max_x = (x + radius).min(width - 1);
                for edge_y in min_y..=max_y {
                    for edge_x in min_x..=max_x {
                        dilated[edge_y * width + edge_x] = true;
                    }
                }
            }
        }
        dilated
    }

    fn pixel_at(image: &RgbaImage, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= image.width || y >= image.height {
            return None;
        }
        let offset = ((y * image.width + x) * 4) as usize;
        image.pixels[offset..offset + 4].try_into().ok()
    }

    fn diagnostic_points(stream_name: &str) -> Vec<(u32, u32)> {
        match stream_name {
            "degengrad" => {
                let mut points = Vec::new();
                for y in [125, 325, 525] {
                    for x in [125, 325, 525, 725] {
                        points.push((x, y));
                    }
                }
                points
            }
            "poly_clockwise" | "poly_evenOdd" => {
                vec![(120, 120), (360, 120), (120, 360), (360, 360)]
            }
            "image" => vec![(10, 10), (100, 100), (400, 100)],
            _ => vec![(10, 10), (20, 20), (58, 10), (100, 100)],
        }
    }

    fn assert_pixels(pixels: &[u8]) -> Result<(), JsValue> {
        if pixels.len() != 64 * 64 * 4 {
            return Err(JsValue::from_str(&format!(
                "unexpected pixel length {}",
                pixels.len()
            )));
        }
        let pixel = |x: usize, y: usize| {
            let offset = (y * 64 + x) * 4;
            &pixels[offset..offset + 4]
        };
        if pixel(20, 4) != [16, 32, 48, 255] {
            return Err(JsValue::from_str(&format!(
                "clear pixel mismatch: {:?}",
                pixel(20, 4)
            )));
        }
        if pixel(5, 3) != [51, 102, 204, 255] || pixel(14, 3) != [16, 32, 48, 255] {
            return Err(JsValue::from_str(&format!(
                "nested transform composition mismatch: inside={:?} outside={:?}",
                pixel(5, 3),
                pixel(14, 3)
            )));
        }
        if pixel(12, 32) != [16, 32, 48, 255] {
            return Err(JsValue::from_str(
                "rectangular clip did not reject the left sample",
            ));
        }
        let left = pixel(24, 32);
        let right = pixel(40, 32);
        if left == [16, 32, 48, 255] || right == [16, 32, 48, 255] || left == right {
            return Err(JsValue::from_str(&format!(
                "gradient samples are not distinct rendered colors: left={left:?} right={right:?}"
            )));
        }
        Ok(())
    }

    fn fnv1a64(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
    }

    fn js_error(error: impl ToString) -> JsValue {
        JsValue::from_str(&error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{assert_webgl2_fail_closed, recording_float_probe, run_backend, run_stream_case};
