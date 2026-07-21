#[cfg(target_arch = "wasm32")]
mod wasm {
    use nuxie::{
        BlendMode, BrowserBackend, BrowserBackendPreference, BrowserFactory, BrowserResizeError,
        Factory, File, FillRule, GpuCanvasPlan, GpuCanvasShader, GpuCanvasShaderStage,
        GpuCanvasUniformBuffer, GpuCanvasVertexAttribute, GpuCanvasVertexBuffer,
        GpuCanvasVertexLayout, ImageFilter, ImageSampler, ImageWrap, Mat2D, RecordingFactory,
        RenderBuffer, RenderBufferFlags, RenderBufferType, Renderer,
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

    const MISSING_IMPORTED_UNIFORM_FRAGMENT_GLSL: &str = r#"#version 300 es
precision highp float;
layout(std140, binding = 0) uniform Tint { vec4 value; } tint;
layout(location = 0) out vec4 color;
void main() { color = tint.value; }
"#;

    const MISMATCHED_IMPORTED_VARYING_VERTEX_GLSL: &str = r#"#version 300 es
precision highp float;
precision highp int;
layout(location = 0) out vec2 varying_value;
void main() {
    uint index = uint(gl_VertexID);
    float x = float(int(index) - 1);
    float y = float(int(index & 1u) * 2 - 1);
    varying_value = vec2(1.0);
    gl_Position = vec4(x, y, 0.0, 1.0);
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
}
"#;

    const MISMATCHED_IMPORTED_VARYING_FRAGMENT_GLSL: &str = r#"#version 300 es
precision highp float;
layout(location = 0) in vec3 varying_value;
layout(location = 0) out vec4 color;
void main() { color = vec4(varying_value, 1.0); }
"#;

    const ANIMATED_UNIFORM_VERTEX_GLSL: &str = r#"#version 300 es
precision highp float;
precision highp int;
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 offset;
layout(location = 0) flat out float instance_value;
void main() {
    instance_value = float(gl_InstanceID);
    gl_Position = vec4(position + offset * 0.25, 0.0, 1.0);
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
}
"#;

    const ANIMATED_UNIFORM_FRAGMENT_GLSL: &str = r#"#version 300 es
precision highp float;
layout(location = 0) flat in float instance_value;
layout(std140, binding = 0) uniform Tint { vec4 value; } tint;
layout(location = 0) out vec4 color;
void main() {
    color = abs(instance_value - 5.0) < 0.25 ? tint.value : vec4(0.0, 0.0, 1.0, 1.0);
}
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

    fn image_mesh_buffers(
        factory: &mut BrowserFactory,
        positions: &[[f32; 2]],
        uvs: &[[f32; 2]],
        triangle_indices: &[u16],
    ) -> (
        Box<dyn RenderBuffer>,
        Box<dyn RenderBuffer>,
        Box<dyn RenderBuffer>,
    ) {
        assert_eq!(positions.len(), uvs.len());
        let mut vertices = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            std::mem::size_of_val(positions),
        );
        vertices
            .map_mut()
            .copy_from_slice(bytemuck::cast_slice(positions));
        vertices.unmap();
        let mut uv_buffer = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            std::mem::size_of_val(uvs),
        );
        uv_buffer
            .map_mut()
            .copy_from_slice(bytemuck::cast_slice(uvs));
        uv_buffer.unmap();
        let mut indices = factory.make_render_buffer(
            RenderBufferType::Index,
            RenderBufferFlags::MappedOnceAtInitialization,
            std::mem::size_of_val(triangle_indices),
        );
        indices
            .map_mut()
            .copy_from_slice(bytemuck::cast_slice(triangle_indices));
        indices.unmap();
        (vertices, uv_buffer, indices)
    }

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
    pub async fn assert_resize(
        canvas: HtmlCanvasElement,
        backend: String,
    ) -> Result<String, JsValue> {
        let preference = match backend.as_str() {
            "webgpu" => BrowserBackendPreference::WebGpu,
            "webgl2" => BrowserBackendPreference::WebGl2,
            value => return Err(JsValue::from_str(&format!("unknown backend {value}"))),
        };
        let mut factory = BrowserFactory::new(canvas.clone(), 8, 6, preference)
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
        let selected_backend_matches = matches!(
            (backend.as_str(), factory.backend()),
            ("webgpu", BrowserBackend::WebGpu) | ("webgl2", BrowserBackend::WebGl2)
        );
        if factory.size() != (8, 6) || !selected_backend_matches {
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
        Ok(format!("resize={backend} in-flight=rejected extent=13x9"))
    }

    #[wasm_bindgen]
    pub async fn assert_imported_gpu_canvas(
        canvas: HtmlCanvasElement,
        backend: String,
    ) -> Result<String, JsValue> {
        let (preference, expected_backend) = match backend.as_str() {
            "webgpu" => (BrowserBackendPreference::WebGpu, BrowserBackend::WebGpu),
            "webgl2" => (BrowserBackendPreference::WebGl2, BrowserBackend::WebGl2),
            value => return Err(JsValue::from_str(&format!("unknown backend {value}"))),
        };
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
        let mut factory = BrowserFactory::new(canvas, 32, 24, preference)
            .await
            .map_err(js_error)?;
        if factory.backend() != expected_backend || factory.fallback_reason().is_some() {
            return Err(JsValue::from_str(&format!(
                "explicit {backend} request selected {:?} with fallback {:?}",
                factory.backend(),
                factory.fallback_reason(),
            )));
        }

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
                "imported GPU canvas on {backend} produced red={red_pixels} black={opaque_black}; expected both halves of the canonical triangle fixture"
            )));
        }
        let selected = format!("{:?}", factory.backend()).to_ascii_lowercase();
        Ok(format!(
            "imported-gpu-canvas={backend} selected={selected} red={red_pixels}",
        ))
    }

    #[wasm_bindgen]
    pub async fn assert_direct_gpu_canvas_image(
        canvas: HtmlCanvasElement,
        backend: String,
    ) -> Result<String, JsValue> {
        let (preference, expected_backend) = match backend.as_str() {
            "webgpu" => (BrowserBackendPreference::WebGpu, BrowserBackend::WebGpu),
            "webgl2" => (BrowserBackendPreference::WebGl2, BrowserBackend::WebGl2),
            value => return Err(JsValue::from_str(&format!("unknown backend {value}"))),
        };
        let mut factory = BrowserFactory::new(canvas, 32, 24, preference)
            .await
            .map_err(js_error)?;
        if factory.backend() != expected_backend || factory.fallback_reason().is_some() {
            return Err(JsValue::from_str(&format!(
                "explicit {backend} direct GPU-canvas request selected {:?} with fallback {:?}",
                factory.backend(),
                factory.fallback_reason(),
            )));
        }
        let shader = imported_gpu_canvas_shader(IMPORTED_VERTEX_GLSL);
        let plan = imported_gpu_canvas_plan(32, 24, [0.0, 0.0, 1.0, 1.0]);
        let mut frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        let image = factory
            .make_gpu_canvas_image(&shader, &plan)
            .map_err(js_error)?;
        frame.draw_image(
            Some(image.as_ref()),
            ImageSampler::default(),
            BlendMode::SrcOver,
            1.0,
        );
        let pixels = frame.finish().await.map_err(js_error)?;
        let red_pixels = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [255, 0, 0, 255])
            .count();
        let blue_pixels = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [0, 0, 255, 255])
            .count();
        if red_pixels < 300 || blue_pixels < 300 {
            return Err(JsValue::from_str(&format!(
                "direct GPU-canvas image on {backend} produced red={red_pixels} blue={blue_pixels}"
            )));
        }
        Ok(format!(
            "direct-gpu-canvas={backend} red={red_pixels} blue={blue_pixels}"
        ))
    }

    #[wasm_bindgen]
    pub async fn assert_webgl2_image_mesh(canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 4, 2);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .map_err(js_error)?
                .write_image_data(&[
                    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255, 255, 0, 0,
                    255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
                ])
                .map_err(js_error)?;
        }

        let mut factory = BrowserFactory::new(canvas, 64, 64, BrowserBackendPreference::WebGl2)
            .await
            .map_err(js_error)?;
        let image = factory.decode_image(&encoded).map_err(js_error)?;
        let (vertices, uvs, indices) = image_mesh_buffers(
            &mut factory,
            &[[0.0, 0.0], [24.0, 0.0], [24.0, 24.0], [0.0, 24.0]],
            &[[0.5, 0.0], [1.0, 0.0], [1.0, 1.0], [0.5, 1.0]],
            &[0, 1, 2, 0, 2, 3],
        );

        let mut clip = factory.make_empty_render_path();
        clip.move_to(16.0, 12.0);
        clip.line_to(40.0, 12.0);
        clip.line_to(40.0, 32.0);
        clip.line_to(16.0, 32.0);
        clip.close();
        let mut mask = factory.make_empty_render_path();
        mask.move_to(16.0, 12.0);
        mask.line_to(40.0, 12.0);
        mask.line_to(28.0, 32.0);
        mask.close();
        let mut frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        frame.clip_path(clip.as_ref());
        frame.clip_path(mask.as_ref());
        frame.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 12.0, 8.0]));
        frame.draw_image_mesh(
            Some(image.as_ref()),
            ImageSampler {
                wrap_x: ImageWrap::Clamp,
                wrap_y: ImageWrap::Clamp,
                filter: ImageFilter::Nearest,
            },
            Some(vertices.as_ref()),
            Some(uvs.as_ref()),
            Some(indices.as_ref()),
            4,
            6,
            BlendMode::SrcOver,
            0.5,
        );
        let pixels = frame.finish().await.map_err(js_error)?;
        let pixel = |x: usize, y: usize| {
            let offset = (y * 64 + x) * 4;
            &pixels[offset..offset + 4]
        };
        let blue = pixel(22, 18);
        let yellow = pixel(34, 18);
        let clipped = pixel(14, 18);
        let outside = pixel(18, 28);
        if !(blue[2] >= 126 && blue[0] <= 1 && blue[1] <= 1 && blue[3] == 255)
            || !(yellow[0] >= 126 && yellow[1] >= 126 && yellow[2] <= 1 && yellow[3] == 255)
            || clipped != [0, 0, 0, 255]
            || outside != [0, 0, 0, 255]
        {
            return Err(JsValue::from_str(&format!(
                "WebGL2 image mesh crop mismatch: blue={blue:?} yellow={yellow:?} clipped={clipped:?} outside={outside:?}"
            )));
        }

        let (triangle_vertices, triangle_uvs, triangle_indices) = image_mesh_buffers(
            &mut factory,
            &[[0.0, 0.0], [20.0, 4.0], [6.0, 24.0]],
            &[[0.875, 0.25]; 3],
            &[0, 1, 2],
        );
        let mut triangle_frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        triangle_frame.transform(Mat2D([1.0, 0.2, 0.3, 1.0, 10.0, 8.0]));
        triangle_frame.draw_image_mesh(
            Some(image.as_ref()),
            ImageSampler {
                wrap_x: ImageWrap::Clamp,
                wrap_y: ImageWrap::Clamp,
                filter: ImageFilter::Nearest,
            },
            Some(triangle_vertices.as_ref()),
            Some(triangle_uvs.as_ref()),
            Some(triangle_indices.as_ref()),
            3,
            3,
            BlendMode::SrcOver,
            1.0,
        );
        let triangle_pixels = triangle_frame.finish().await.map_err(js_error)?;
        let triangle_pixel = &triangle_pixels[(19 * 64 + 21) * 4..][..4];
        if triangle_pixel != [255, 255, 0, 255] {
            return Err(JsValue::from_str(&format!(
                "general transformed WebGL2 image triangle mismatch: {triangle_pixel:?}"
            )));
        }

        let (sample_vertices, linear_uvs, sample_indices) = image_mesh_buffers(
            &mut factory,
            &[[8.0, 8.0], [32.0, 8.0], [32.0, 32.0], [8.0, 32.0]],
            &[[0.5, 0.25]; 4],
            &[0, 1, 2, 0, 2, 3],
        );
        let mut linear_frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        linear_frame.draw_image_mesh(
            Some(image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            Some(sample_vertices.as_ref()),
            Some(linear_uvs.as_ref()),
            Some(sample_indices.as_ref()),
            4,
            6,
            BlendMode::SrcOver,
            1.0,
        );
        let linear_pixels = linear_frame.finish().await.map_err(js_error)?;
        let linear_pixel = &linear_pixels[(16 * 64 + 16) * 4..][..4];
        if linear_pixel[0] > 1
            || !(120..=135).contains(&linear_pixel[1])
            || !(120..=135).contains(&linear_pixel[2])
            || linear_pixel[3] != 255
        {
            return Err(JsValue::from_str(&format!(
                "bilinear WebGL2 image mesh sample mismatch: {linear_pixel:?}"
            )));
        }

        let (repeat_vertices, repeat_uvs, repeat_indices) = image_mesh_buffers(
            &mut factory,
            &[[8.0, 8.0], [32.0, 8.0], [32.0, 32.0], [8.0, 32.0]],
            &[[1.125, 0.25]; 4],
            &[0, 1, 2, 0, 2, 3],
        );
        let mut repeat_frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        let repeat_sampler = ImageSampler {
            wrap_x: ImageWrap::Repeat,
            wrap_y: ImageWrap::Clamp,
            filter: ImageFilter::Nearest,
        };
        repeat_frame.draw_image_mesh(
            Some(image.as_ref()),
            repeat_sampler,
            Some(repeat_vertices.as_ref()),
            Some(repeat_uvs.as_ref()),
            Some(repeat_indices.as_ref()),
            4,
            6,
            BlendMode::SrcOver,
            1.0,
        );
        let repeat_pixels = repeat_frame.finish().await.map_err(js_error)?;
        let repeat_pixel = &repeat_pixels[(16 * 64 + 16) * 4..][..4];
        if repeat_pixel != [255, 0, 0, 255] {
            return Err(JsValue::from_str(&format!(
                "repeated WebGL2 image mesh sample mismatch: {repeat_pixel:?}"
            )));
        }

        for (label, sampler, blend_mode, expected_error) in [
            (
                "mirror",
                ImageSampler {
                    wrap_x: ImageWrap::Mirror,
                    wrap_y: ImageWrap::Clamp,
                    filter: ImageFilter::Nearest,
                },
                BlendMode::SrcOver,
                "mirrored image wrapping",
            ),
            (
                "advanced blend",
                ImageSampler::LINEAR_CLAMP,
                BlendMode::Screen,
                "advanced image blend modes",
            ),
        ] {
            let mut unsupported = factory.begin_frame(0xff00_0000).map_err(js_error)?;
            unsupported.draw_image_mesh(
                Some(image.as_ref()),
                sampler,
                Some(sample_vertices.as_ref()),
                Some(linear_uvs.as_ref()),
                Some(sample_indices.as_ref()),
                4,
                6,
                blend_mode,
                1.0,
            );
            match unsupported.finish().await {
                Err(error) if error.to_string().contains(expected_error) => {}
                Err(error) => {
                    return Err(JsValue::from_str(&format!(
                        "WebGL2 image mesh returned the wrong {label} error: {error}"
                    )));
                }
                Ok(_) => {
                    return Err(JsValue::from_str(&format!(
                        "WebGL2 image mesh silently accepted unsupported {label}"
                    )));
                }
            }
        }
        let recovery = factory
            .begin_frame(0xff12_3456)
            .map_err(js_error)?
            .finish()
            .await
            .map_err(js_error)?;
        if !recovery
            .chunks_exact(4)
            .all(|pixel| pixel == [0x12, 0x34, 0x56, 0xff])
        {
            return Err(JsValue::from_str(
                "unsupported WebGL2 image mesh state contaminated a later frame",
            ));
        }
        Ok("image-mesh=webgl2 indexed=cropped general-triangles=applied transform=applied clip-layer=applied opacity=applied samplers=nearest+bilinear+repeat unsupported=mirror+advanced-blend-fail-closed".into())
    }

    #[wasm_bindgen]
    pub async fn assert_webgpu_gpu_canvas_rejects_invalid_interface(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas, 8, 8, BrowserBackendPreference::WebGpu)
            .await
            .map_err(js_error)?;
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
                "an imported GPU-canvas validation error contaminated an unrelated frame",
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
    pub async fn assert_webgl2_gpu_canvas_rejects_invalid_interface(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas, 8, 8, BrowserBackendPreference::WebGl2)
            .await
            .map_err(js_error)?;
        let plan = imported_gpu_canvas_plan(8, 8, [0.0, 0.0, 0.0, 1.0]);
        let invalid_cases = [
            (
                imported_gpu_canvas_shader(INVALID_IMPORTED_VERTEX_GLSL),
                "vertex inputs",
            ),
            (
                imported_gpu_canvas_shader_stages(
                    IMPORTED_VERTEX_GLSL,
                    MISSING_IMPORTED_UNIFORM_FRAGMENT_GLSL,
                ),
                "uniform bindings",
            ),
            (
                imported_gpu_canvas_shader_stages(
                    MISMATCHED_IMPORTED_VARYING_VERTEX_GLSL,
                    MISMATCHED_IMPORTED_VARYING_FRAGMENT_GLSL,
                ),
                "inter-stage",
            ),
        ];
        for (shader, expected) in invalid_cases {
            match factory.make_gpu_canvas_image(&shader, &plan) {
                Err(error) if error.to_string().contains(expected) => {}
                Err(error) => {
                    return Err(JsValue::from_str(&format!(
                        "WebGL2 returned the wrong interface error for {expected}: {error}"
                    )));
                }
                Ok(_) => {
                    return Err(JsValue::from_str(&format!(
                        "WebGL2 accepted an imported GPU-canvas {expected} mismatch"
                    )));
                }
            }
        }

        let image = factory
            .make_gpu_canvas_image(&imported_gpu_canvas_shader(IMPORTED_VERTEX_GLSL), &plan)
            .map_err(js_error)?;
        let mut frame = factory.begin_frame(0xff65_4321).map_err(js_error)?;
        frame.draw_image(
            Some(image.as_ref()),
            ImageSampler::default(),
            BlendMode::SrcOver,
            1.0,
        );
        let pixels = frame.finish().await.map_err(js_error)?;
        let red = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [0xff, 0x00, 0x00, 0xff])
            .count();
        let black = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel == [0x00, 0x00, 0x00, 0xff])
            .count();
        if red < 20 || black < 20 {
            return Err(JsValue::from_str(
                "valid WebGL2 imported GPU-canvas image did not render after interface rejection",
            ));
        }
        Ok(
            "webgl2-gpu-canvas-interface=attributes+uniforms+interstage-rejected valid=clean"
                .into(),
        )
    }

    #[wasm_bindgen]
    pub async fn assert_imported_gpu_canvas_stress(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        const FRAME_COUNT: usize = 32;
        let mut factory = BrowserFactory::new(canvas, 32, 24, BrowserBackendPreference::WebGl2)
            .await
            .map_err(js_error)?;
        let shaders = [
            imported_gpu_canvas_shader(IMPORTED_VERTEX_GLSL),
            imported_gpu_canvas_shader_stages(
                IMPORTED_VERTEX_GLSL,
                r#"#version 300 es
precision highp float;
layout(location = 0) out vec4 color;
void main() { color = vec4(0.0, 1.0, 0.0, 1.0); }
"#,
            ),
        ];
        let plan = imported_gpu_canvas_plan(32, 24, [0.0, 0.0, 1.0, 1.0]);
        for frame_index in 0..FRAME_COUNT {
            let shader_index = frame_index % shaders.len();
            let image = factory
                .make_gpu_canvas_image(&shaders[shader_index], &plan)
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
                .filter(|pixel| *pixel == [0xff, 0x00, 0x00, 0xff])
                .count();
            let green = pixels
                .chunks_exact(4)
                .filter(|pixel| *pixel == [0x00, 0xff, 0x00, 0xff])
                .count();
            let blue = pixels
                .chunks_exact(4)
                .filter(|pixel| *pixel == [0x00, 0x00, 0xff, 0xff])
                .count();
            let expected = if shader_index == 0 { red } else { green };
            if expected < 300 || blue < 300 {
                return Err(JsValue::from_str(&format!(
                    "WebGL2 imported GPU-canvas stress frame {frame_index} produced red={red} green={green} blue={blue}"
                )));
            }
        }
        Ok(format!(
            "imported-gpu-canvas-stress=webgl2 frames={FRAME_COUNT} keys={}",
            shaders.len()
        ))
    }

    #[wasm_bindgen]
    pub async fn assert_webgl2_imported_gpu_canvas_uniform_animation(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        let mut factory = BrowserFactory::new(canvas, 32, 24, BrowserBackendPreference::WebGl2)
            .await
            .map_err(js_error)?;
        let shader = imported_gpu_canvas_shader_stages(
            ANIMATED_UNIFORM_VERTEX_GLSL,
            ANIMATED_UNIFORM_FRAGMENT_GLSL,
        );
        let mut plan = imported_gpu_canvas_plan(32, 24, [0.0, 0.0, 1.0, 1.0]);
        plan.first_instance = 5;
        plan.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 16],
        });
        let encode_f32s = |values: &[f32]| {
            values
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<_>>()
        };
        plan.vertex_layouts = vec![
            GpuCanvasVertexLayout {
                stride: 8,
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: "float32x2".into(),
                }],
            },
            GpuCanvasVertexLayout {
                stride: 8,
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 1,
                    offset: 0,
                    format: "float32x2".into(),
                }],
            },
        ];
        plan.vertex_buffers = vec![
            GpuCanvasVertexBuffer {
                slot: 1,
                bytes: encode_f32s(&[0.0; 6]),
            },
            GpuCanvasVertexBuffer {
                slot: 0,
                bytes: encode_f32s(&[-1.0, -1.0, 3.0, -1.0, -1.0, 3.0]),
            },
        ];

        for (frame_index, expected) in [[0xff, 0x00, 0x00, 0xff], [0x00, 0xff, 0x00, 0xff]]
            .into_iter()
            .enumerate()
        {
            for (offset, component) in expected
                .into_iter()
                .map(|component| f32::from(component) / 255.0)
                .enumerate()
            {
                plan.uniform_buffers[0].bytes[offset * 4..offset * 4 + 4]
                    .copy_from_slice(&component.to_le_bytes());
            }
            let image = factory
                .make_gpu_canvas_image(&shader, &plan)
                .map_err(js_error)?;
            let mut frame = factory.begin_frame(0xff00_00ff).map_err(js_error)?;
            frame.draw_image(
                Some(image.as_ref()),
                ImageSampler::default(),
                BlendMode::SrcOver,
                1.0,
            );
            let pixels = frame.finish().await.map_err(js_error)?;
            let matching = pixels
                .chunks_exact(4)
                .filter(|pixel| *pixel == expected)
                .count();
            if matching < 300 {
                return Err(JsValue::from_str(&format!(
                    "WebGL2 animated uniform frame {frame_index} produced only {matching} expected pixels"
                )));
            }
        }

        Ok(
            "imported-gpu-canvas-uniform-animation=webgl2 frames=2 first-instance=5 reversed-slots=applied"
                .into(),
        )
    }

    #[wasm_bindgen]
    pub async fn assert_webgpu_uniform_limit_rejection(
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        const UNIFORM_COUNT: usize = 13;
        let mut factory = BrowserFactory::new(canvas, 8, 8, BrowserBackendPreference::WebGpu)
            .await
            .map_err(js_error)?;
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
            .map(|index| nuxie::GpuCanvasUniformBuffer {
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

        let mut full_frame = factory.make_empty_render_path();
        full_frame.move_to(0.0, 0.0);
        full_frame.line_to(16.0, 0.0);
        full_frame.line_to(16.0, 16.0);
        full_frame.line_to(0.0, 16.0);
        full_frame.close();
        let mut white = factory.make_render_paint();
        white.color(0xffff_ffff);

        let mut frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        frame.clip_path(triangle.as_ref());
        frame.draw_path(full_frame.as_ref(), white.as_ref());
        let clipped = frame.finish().await.map_err(js_error)?;
        let pixel = |x: usize, y: usize| {
            let offset = (y * 16 + x) * 4;
            &clipped[offset..offset + 4]
        };
        if pixel(8, 4) != [0xff, 0xff, 0xff, 0xff]
            || pixel(1, 12) != [0x00, 0x00, 0x00, 0xff]
            || pixel(14, 12) != [0x00, 0x00, 0x00, 0xff]
        {
            return Err(JsValue::from_str(&format!(
                "non-rectangular WebGL2 clip mismatch: inside={:?} outside-left={:?} outside-right={:?}",
                pixel(8, 4),
                pixel(1, 12),
                pixel(14, 12),
            )));
        }

        let unsupported_sampler = ImageSampler {
            wrap_x: ImageWrap::Repeat,
            wrap_y: ImageWrap::Mirror,
            filter: ImageFilter::Nearest,
        };
        let shader = imported_gpu_canvas_shader(IMPORTED_VERTEX_GLSL);
        let plan = imported_gpu_canvas_plan(4, 4, [0.0, 0.0, 0.0, 1.0]);
        let image = factory
            .make_gpu_canvas_image(&shader, &plan)
            .map_err(js_error)?;
        let mut unsupported_frame = factory.begin_frame(0xff00_0000).map_err(js_error)?;
        unsupported_frame.draw_image(
            Some(image.as_ref()),
            unsupported_sampler,
            BlendMode::SrcOver,
            1.0,
        );
        match unsupported_frame.finish().await {
            Err(error) if error.to_string().contains("mirrored image wrapping") => {}
            Err(error) => {
                return Err(JsValue::from_str(&format!(
                    "unexpected mirrored-sampler error: {error}"
                )));
            }
            Ok(_) => {
                return Err(JsValue::from_str(
                    "WebGL2 rendered an unsupported mirrored image sampler",
                ));
            }
        }

        let recovery_frame = factory.begin_frame(0xff12_3456).map_err(js_error)?;
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
        frame.transform(Mat2D([
            0.70710677,
            -0.70710677,
            0.70710677,
            0.70710677,
            -5.656854,
            5.656854,
        ]));
        frame.draw_path(full_frame.as_ref(), white.as_ref());
        let transformed_clip = frame.finish().await.map_err(js_error)?;
        let pixel = |x: usize, y: usize| {
            let offset = (y * 16 + x) * 4;
            &transformed_clip[offset..offset + 4]
        };
        if pixel(8, 4) != [0xff, 0xff, 0xff, 0xff]
            || pixel(1, 1) != [0x00, 0x00, 0x00, 0xff]
            || pixel(14, 8) != [0x00, 0x00, 0x00, 0xff]
        {
            return Err(JsValue::from_str(&format!(
                "transformed WebGL2 clip intersection mismatch: inside={:?} outside-path={:?} outside-rect={:?}",
                pixel(8, 4),
                pixel(1, 1),
                pixel(14, 8),
            )));
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

        Ok("path-clip=exact unsupported=fail-closed recovery=clean abandoned=poisoned".into())
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
pub use wasm::{
    assert_direct_gpu_canvas_image, assert_imported_gpu_canvas, assert_imported_gpu_canvas_stress,
    assert_resize, assert_webgl2_fail_closed, assert_webgl2_gpu_canvas_rejects_invalid_interface,
    assert_webgl2_image_mesh, assert_webgl2_imported_gpu_canvas_uniform_animation,
    assert_webgpu_gpu_canvas_rejects_invalid_interface, assert_webgpu_uniform_limit_rejection,
    recording_float_probe, run_backend, run_stream_case,
};
