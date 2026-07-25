#[cfg(target_arch = "wasm32")]
mod wasm {
    use nuxie::{
        BlendMode, BrowserFactory, BrowserResizeError, Factory, File, FillRule, GpuCanvasPlan,
        GpuCanvasShader, GpuCanvasShaderStage, GpuCanvasUniformBuffer, ImageSampler, Mat2D,
        RecordingFactory, Renderer,
    };
    use nuxie_render_stream::RenderStream;
    use pixel_compare::{RgbaImage, Tolerance, compare};
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlCanvasElement;

    const IMPORTED_GPU_CANVAS_RIV: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/imported-gpu-canvas.riv"));

    const IMPORTED_VERTEX_GLSL: &str = r#"#version 300 es
precision highp float;
precision highp int;
void main() {
    uint index = uint(gl_VertexID);
    float x = float(int(index) - 1);
    float y = float(int(index & 1u) * 2 - 1);
    gl_Position = vec4(x, y, 0.0, 1.0);
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
}
"#;

    const IMPORTED_FRAGMENT_GLSL: &str = r#"#version 300 es
precision highp float;
layout(location = 0) out vec4 color;
void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }
"#;

    const INVALID_IMPORTED_VERTEX_GLSL: &str = r#"#version 300 es
precision highp float;
layout(location = 0) in vec2 position;
void main() { gl_Position = vec4(position, 0.0, 1.0); }
"#;

    fn imported_gpu_canvas_shader(vertex_source: &str) -> GpuCanvasShader {
        imported_gpu_canvas_shader_stages(vertex_source, IMPORTED_FRAGMENT_GLSL)
    }

    fn imported_gpu_canvas_shader_stages(
        vertex_source: &str,
        fragment_source: &str,
    ) -> GpuCanvasShader {
        GpuCanvasShader {
            vertex: GpuCanvasShaderStage {
                source: vertex_source.into(),
                logical_entry_point: "vs_main".into(),
                physical_entry_point: "main".into(),
            },
            fragment: GpuCanvasShaderStage {
                source: fragment_source.into(),
                logical_entry_point: "fs_main".into(),
                physical_entry_point: "main".into(),
            },
        }
    }

    fn imported_gpu_canvas_plan(width: u32, height: u32, clear_color: [f64; 4]) -> GpuCanvasPlan {
        GpuCanvasPlan {
            width,
            height,
            clear_color,
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            uniform_buffers: Vec::new(),
            vertex_layouts: Vec::new(),
            vertex_buffers: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn run_backend(canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas, 64, 64)
            .await
            .map_err(js_error)?;

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
        Ok(format!("backend=webgpu checksum={:016x}", fnv1a64(&pixels)))
    }

    #[wasm_bindgen]
    pub async fn assert_resize(canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas.clone(), 8, 6)
            .await
            .map_err(js_error)?;
        let frame = factory.begin_frame(0xff12_3456).map_err(js_error)?;
        match factory.resize(13, 9) {
            Err(BrowserResizeError::FrameInFlight) => {}
            Err(error) => {
                return Err(JsValue::from_str(&format!(
                    "unexpected in-flight resize error: {error}"
                )));
            }
            Ok(()) => {
                return Err(JsValue::from_str(
                    "browser factory resized while a frame was in flight",
                ));
            }
        }
        if factory.size() != (8, 6) {
            return Err(JsValue::from_str(
                "in-flight resize changed readable factory state",
            ));
        }
        if frame.finish().await.map_err(js_error)?.len() != 8 * 6 * 4 {
            return Err(JsValue::from_str(
                "in-flight frame changed extent after rejected resize",
            ));
        }
        factory.resize(13, 9).map_err(js_error)?;
        if factory.size() != (13, 9) || canvas.width() != 13 || canvas.height() != 9 {
            return Err(JsValue::from_str(
                "resize did not update the factory and canvas extent",
            ));
        }
        let pixels = factory
            .begin_frame(0xff65_4321)
            .map_err(js_error)?
            .finish()
            .await
            .map_err(js_error)?;
        if pixels.len() != 13 * 9 * 4 {
            return Err(JsValue::from_str(
                "resized frame returned the old pixel extent",
            ));
        }
        Ok("resize=webgpu in-flight=rejected extent=13x9".into())
    }

    #[wasm_bindgen]
    pub async fn assert_imported_gpu_canvas(canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        let file = File::import_with_unsigned_scripts(IMPORTED_GPU_CANVAS_RIV).map_err(js_error)?;
        let artboard = file
            .default_artboard()
            .ok_or_else(|| JsValue::from_str("imported GPU-canvas fixture has no artboard"))?;
        if artboard.dimensions() != Some((32.0, 24.0)) {
            return Err(JsValue::from_str(&format!(
                "imported GPU-canvas fixture has unexpected artboard dimensions {:?}",
                artboard.dimensions(),
            )));
        }
        let mut instance = artboard.instantiate().map_err(js_error)?;
        let mut factory = BrowserFactory::new(canvas, 32, 24)
            .await
            .map_err(js_error)?;
        let mut frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        instance
            .draw(&mut factory, &mut frame)
            .map_err(|error| JsValue::from_str(&format!("{error:#}")))?;
        let pixels = frame.finish().await.map_err(js_error)?;
        let red_pixels = pixels
            .chunks_exact(4)
            .filter(|pixel| pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10 && pixel[3] > 240)
            .count();
        let opaque_black = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [0, 0, 0, 255])
            .count();
        if red_pixels < 300 || opaque_black < 300 {
            return Err(JsValue::from_str(&format!(
                "imported GPU canvas on webgpu produced red={red_pixels} black={opaque_black}; expected both halves of the canonical triangle fixture"
            )));
        }
        Ok(format!(
            "imported-gpu-canvas=webgpu selected=webgpu red={red_pixels}",
        ))
    }

    #[wasm_bindgen]
    pub async fn assert_direct_gpu_canvas_image(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas, 32, 24)
            .await
            .map_err(js_error)?;
        let shader = imported_gpu_canvas_shader(IMPORTED_VERTEX_GLSL);
        let plan = imported_gpu_canvas_plan(32, 24, [0.0, 0.0, 1.0, 1.0]);
        let image = factory
            .make_gpu_canvas_image(&shader, &plan)
            .map_err(js_error)?;
        let mut frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        frame.draw_image(
            Some(image.as_ref()),
            ImageSampler::default(),
            BlendMode::SrcOver,
            1.0,
        );
        let pixels = frame.finish().await.map_err(js_error)?;
        let red = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [255, 0, 0, 255])
            .count();
        let blue = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [0, 0, 255, 255])
            .count();
        if red < 300 || blue < 300 {
            return Err(JsValue::from_str(&format!(
                "direct GPU-canvas image produced red={red} blue={blue}"
            )));
        }
        Ok(format!("direct-gpu-canvas=webgpu red={red} blue={blue}"))
    }

    #[wasm_bindgen]
    pub async fn assert_webgpu_gpu_canvas_rejects_invalid_interface(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas, 8, 8).await.map_err(js_error)?;
        let unrelated = factory.begin_frame(0xff12_3456).map_err(js_error)?;
        let invalid_shader = imported_gpu_canvas_shader(INVALID_IMPORTED_VERTEX_GLSL);
        let plan = imported_gpu_canvas_plan(8, 8, [0.0, 0.0, 0.0, 1.0]);
        match factory.make_gpu_canvas_image(&invalid_shader, &plan) {
            Err(error) if error.to_string().contains("vertex inputs") => {}
            Err(error) => {
                return Err(JsValue::from_str(&format!(
                    "unexpected synchronous GPU-canvas interface error: {error}"
                )));
            }
            Ok(_) => {
                return Err(JsValue::from_str(
                    "invalid imported GPU-canvas interface reached backend allocation",
                ));
            }
        }
        let unrelated_pixels = unrelated.finish().await.map_err(js_error)?;
        if !unrelated_pixels
            .chunks_exact(4)
            .all(|pixel| pixel == [0x12, 0x34, 0x56, 0xff])
        {
            return Err(JsValue::from_str(
                "GPU-canvas validation contaminated an unrelated frame",
            ));
        }
        let valid_shader = imported_gpu_canvas_shader(IMPORTED_VERTEX_GLSL);
        let valid_image = factory
            .make_gpu_canvas_image(&valid_shader, &plan)
            .map_err(js_error)?;
        let mut valid_frame = factory.begin_frame(0xff65_4321).map_err(js_error)?;
        valid_frame.draw_image(
            Some(valid_image.as_ref()),
            ImageSampler::default(),
            BlendMode::SrcOver,
            1.0,
        );
        let valid_pixels = valid_frame.finish().await.map_err(js_error)?;
        let red = valid_pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [0xff, 0x00, 0x00, 0xff])
            .count();
        let black = valid_pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [0x00, 0x00, 0x00, 0xff])
            .count();
        if red < 20 || black < 20 {
            return Err(JsValue::from_str(
                "valid imported GPU-canvas image did not render cleanly after synchronous rejection",
            ));
        }
        Ok("gpu-canvas-interface=sync-rejected unrelated=clean valid=clean".into())
    }

    #[wasm_bindgen]
    pub async fn assert_webgpu_uniform_limit_rejection(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        const UNIFORM_COUNT: usize = 13;
        let mut factory = BrowserFactory::new(canvas, 8, 8).await.map_err(js_error)?;
        let unrelated = factory.begin_frame(0xff12_3456).map_err(js_error)?;
        let mut fragment = String::from(
            "#version 300 es\nprecision highp float;\nlayout(location = 0) out vec4 color;\n",
        );
        for index in 0..UNIFORM_COUNT {
            let group = index / 7;
            let binding = index % 7;
            fragment.push_str(&format!(
                "layout(std140, set = {group}, binding = {binding}) uniform U{index} {{ vec4 value; }} u{index};\n"
            ));
        }
        fragment.push_str("void main() { color = ");
        for index in 0..UNIFORM_COUNT {
            if index != 0 {
                fragment.push_str(" + ");
            }
            fragment.push_str(&format!("u{index}.value"));
        }
        fragment.push_str("; }\n");
        let shader = imported_gpu_canvas_shader_stages(IMPORTED_VERTEX_GLSL, &fragment);
        let mut plan = imported_gpu_canvas_plan(8, 8, [0.0, 0.0, 0.0, 1.0]);
        plan.uniform_buffers = (0..UNIFORM_COUNT)
            .map(|index| GpuCanvasUniformBuffer {
                group: (index / 7) as u32,
                binding: (index % 7) as u32,
                bytes: vec![0; 16],
            })
            .collect();
        match factory.make_gpu_canvas_image(&shader, &plan) {
            Err(error) if error.to_string().contains("uniform buffers") => {}
            Err(error) => {
                return Err(JsValue::from_str(&format!(
                    "WebGPU returned the wrong per-stage uniform-limit error: {error}"
                )));
            }
            Ok(_) => {
                return Err(JsValue::from_str(
                    "WebGPU accepted 13 fragment-stage uniform buffers in one call",
                ));
            }
        }
        let unrelated_pixels = unrelated.finish().await.map_err(js_error)?;
        if !unrelated_pixels
            .chunks_exact(4)
            .all(|pixel| pixel == [0x12, 0x34, 0x56, 0xff])
        {
            return Err(JsValue::from_str(
                "WebGPU uniform-limit rejection contaminated an unrelated frame",
            ));
        }
        let valid_plan = imported_gpu_canvas_plan(8, 8, [0.0, 0.0, 0.0, 1.0]);
        let valid_image = factory
            .make_gpu_canvas_image(
                &imported_gpu_canvas_shader(IMPORTED_VERTEX_GLSL),
                &valid_plan,
            )
            .map_err(js_error)?;
        let mut valid_frame = factory.begin_frame(0xff65_4321).map_err(js_error)?;
        valid_frame.draw_image(
            Some(valid_image.as_ref()),
            ImageSampler::default(),
            BlendMode::SrcOver,
            1.0,
        );
        valid_frame.finish().await.map_err(js_error)?;
        Ok("webgpu-uniform-limit=same-call-rejected unrelated=clean valid=clean".into())
    }

    #[wasm_bindgen]
    pub async fn run_stream_case(
        canvas: HtmlCanvasElement,
        stream_name: String,
        stream_text: String,
        reference_png: Vec<u8>,
        max_channel_delta: u8,
        max_different_pixels: u32,
        expected_edge_radius: u32,
        max_off_edge_different_pixels: u32,
    ) -> Result<String, JsValue> {
        let stream = RenderStream::parse(&stream_text).map_err(js_error)?;
        let (width, height) = stream
            .frame_size
            .ok_or_else(|| JsValue::from_str("stream does not declare frameSize"))?;
        let clear = stream.clear_color.unwrap_or(0);
        let mut factory = BrowserFactory::new(canvas, width, height)
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
                    "stream={stream_name} backend=webgpu semantic probe ({x},{y}) differs: expected={expected_pixel:?} actual={actual_pixel:?}"
                )));
            }
        }
        let result = format!(
            "stream={stream_name} backend=webgpu different={} off-edge={} max-delta={} channel-max={channel_max:?} channel-different={channel_different:?} checksum={:016x}",
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

    #[wasm_bindgen]
    pub fn recording_float_probe() -> String {
        let mut factory = RecordingFactory::new();
        factory.add_sample(0.1);
        factory.stream()
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
pub use wasm::{
    assert_direct_gpu_canvas_image, assert_imported_gpu_canvas, assert_resize,
    assert_webgpu_gpu_canvas_rejects_invalid_interface, assert_webgpu_uniform_limit_rejection,
    recording_float_probe, run_backend, run_stream_case,
};
