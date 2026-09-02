#![cfg(feature = "scripting")]

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie::{
    ColorInt, Factory, File, FileImportLimits, FillRule, GpuCanvasError, GpuCanvasPlan,
    GpuCanvasShader, GpuCanvasShaderStage, ImageDecodeError, PersistentFactory, RecordingFactory,
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderGpuCanvasShader, RenderImage,
    RenderPaint, RenderPath, RenderShader, RuntimeArtboardInstanceHandle, ScriptExecutionLimits,
    ScriptedFile, import_unsigned_scripted,
};
use nuxie_render_api::RawPath;
use nuxie_schema::definition_by_name;

use runtime_test_support::recording_gpu;

const SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local shader = context:shader("scene")
    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "chosen_vertex" },
        fragment = { module = shader, entryPoint = "chosen_fragment" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    local sampler = ImageSampler("clamp", "clamp", "nearest")
    canvas:resize(32, 24)
    return {
        draw = function(self, renderer)
            local pass = canvas:beginRenderPass {
                color = { { loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 } } },
            }
            pass:setPipeline(pipeline)
            pass:draw(3)
            pass:finish()
            renderer:drawImage(canvas.image, sampler, "srcOver", 1.0)
        end,
    }
end
"#;

const FOLDERED_SHADER_ALIAS_SCRIPT: &[u8] = br#"
return function(context)
    local bare = context:shader("scene")
    local qualified = context:shader("effects/scene")
    return {
        draw = function(self, renderer)
        end,
    }
end
"#;

const UNUSED_CONTENTLESS_SHADER_SCRIPT: &[u8] = br#"
return function(_context)
    return {
        draw = function(self, renderer)
        end,
    }
end
"#;

const REQUESTED_CONTENTLESS_SHADER_SCRIPT: &[u8] = br##"
return function(context)
    local shader = context:shader("scene")
    local returnCount = select("#", context:shader("scene"))
    if shader ~= nil or returnCount ~= 0 then
        error("contentless shader lookup must return zero values")
    end
    return {
        draw = function(self, renderer)
        end,
    }
end
"##;

const COLLIDING_SHADER_ALIAS_SCRIPT: &[u8] = br#"
return function(context)
    local bare = context:shader("scene")
    local first = context:shader("first/scene")
    local second = context:shader("second/scene")
    if bare == nil or first == nil or second == nil then
        error("every first-wins alias must remain reachable")
    end
    return {
        draw = function(self, renderer)
        end,
    }
end
"#;

const AUTHORED_BOOLEAN_INPUT_SCRIPT: &[u8] = br#"
return function(_context)
    return {
        animate = true,
        init = function(self)
            if self.animate ~= false then
                error("authored boolean input was not hydrated before init")
            end
            return true
        end,
        draw = function(self, _renderer)
        end,
    }
end
"#;

fn compile_luau(source: &[u8]) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null());
    // SAFETY: luaur returns a valid allocation containing output_size bytes.
    unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name).expect("fixture type exists");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            definition_by_name(ancestor)
                .expect("fixture ancestor exists")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("fixture property exists")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            definition_by_name(type_name)
                .expect("fixture type exists")
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_bool(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: bool) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.push(u8::from(value));
}

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
}

fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    put_u16(bytes, value.len() as u16);
    bytes.extend_from_slice(value.as_bytes());
}

fn shader_payload() -> Vec<u8> {
    shader_payload_with_marker("default")
}

fn shader_payload_with_marker(marker: &str) -> Vec<u8> {
    const WGSL: &str = r#"
@vertex
fn physical_vertex_0(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@vertex
fn physical_vertex_1(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn physical_fragment_0() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}

@fragment
fn physical_fragment_1() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;
    let wgsl = format!("// {marker}\n{WGSL}");
    const EMPTY_BINDING_MAP: &[u8] = &[2, 1, 14, 0, 0, 0, 0, 0];
    let entries = [
        (0, "default_vertex", "physical_vertex_0"),
        (0, "chosen_vertex", "physical_vertex_1"),
        (1, "default_fragment", "physical_fragment_0"),
        (1, "chosen_fragment", "physical_fragment_1"),
    ];
    let mut source = vec![entries.len() as u8];
    for (stage, logical, physical) in entries {
        source.push(stage);
        put_string(&mut source, logical);
        put_string(&mut source, physical);
    }
    put_u32(&mut source, wgsl.len() as u32);
    source.extend_from_slice(wgsl.as_bytes());

    let mut payload = vec![0];
    put_u32(&mut payload, 0x5253_5442);
    put_u16(&mut payload, 4);
    payload.extend_from_slice(&[2, 0]);
    payload.push(0);
    put_u32(&mut payload, 0);
    put_u32(&mut payload, source.len() as u32);
    payload.push(16);
    put_u32(&mut payload, source.len() as u32);
    put_u32(&mut payload, EMPTY_BINDING_MAP.len() as u32);
    payload.extend(source);
    payload.extend_from_slice(EMPTY_BINDING_MAP);
    payload
}

fn imported_file() -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(SCRIPT));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ShaderAsset", |bytes| {
        push_uint(bytes, "ShaderAsset", "assetId", 0);
        push_string(bytes, "ShaderAsset", "name", "scene");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &shader_payload());
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 1);
        push_string(bytes, "ScriptAsset", "name", "GpuNode");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 32.0);
        push_f32(bytes, "Artboard", "height", 24.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
    });
    bytes
}

fn authored_boolean_input_file() -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(AUTHORED_BOOLEAN_INPUT_SCRIPT));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "AuthoredInput");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 32.0);
        push_f32(bytes, "Artboard", "height", 24.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 0);
    });
    push_object(&mut bytes, "ScriptInputBoolean", |bytes| {
        push_uint(bytes, "ScriptInputBoolean", "parentId", 1);
        push_string(bytes, "ScriptInputBoolean", "name", "animate");
        push_bool(bytes, "ScriptInputBoolean", "propertyValue", false);
    });
    bytes
}

fn foldered_shader_alias_file() -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(FOLDERED_SHADER_ALIAS_SCRIPT));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ShaderAsset", |bytes| {
        push_uint(bytes, "ShaderAsset", "assetId", 0);
        push_string(bytes, "ShaderAsset", "name", "scene");
        push_string(bytes, "ShaderAsset", "folderPath", "effects");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &shader_payload());
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 1);
        push_string(bytes, "ScriptAsset", "name", "AliasLookup");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 32.0);
        push_f32(bytes, "Artboard", "height", 24.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
    });
    bytes
}

fn contentless_shader_file(script: &[u8]) -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(script));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ShaderAsset", |bytes| {
        push_uint(bytes, "ShaderAsset", "assetId", 0);
        push_string(bytes, "ShaderAsset", "name", "scene");
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 1);
        push_string(bytes, "ScriptAsset", "name", "ContentlessShader");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 32.0);
        push_f32(bytes, "Artboard", "height", 24.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
    });
    bytes
}

fn colliding_shader_alias_file() -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(COLLIDING_SHADER_ALIAS_SCRIPT));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    for (asset_id, folder, marker) in [(0, "first", "first-owner"), (1, "second", "second-owner")] {
        push_object(&mut bytes, "ShaderAsset", |bytes| {
            push_uint(bytes, "ShaderAsset", "assetId", asset_id);
            push_string(bytes, "ShaderAsset", "name", "scene");
            push_string(bytes, "ShaderAsset", "folderPath", folder);
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(
                bytes,
                "FileAssetContents",
                "bytes",
                &shader_payload_with_marker(marker),
            );
        });
    }
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 2);
        push_string(bytes, "ScriptAsset", "name", "AliasCollisions");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 32.0);
        push_f32(bytes, "Artboard", "height", 24.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 2);
    });
    bytes
}

#[derive(Clone)]
struct TestImage {
    identity: Rc<()>,
    width: u32,
    height: u32,
}

impl RenderImage for TestImage {
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.identity) as usize
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

struct TestGpuCanvasShader(
    GpuCanvasShader,
    nuxie_ore_metal::gpu_resource::AnyResourceHandle,
);

impl RenderGpuCanvasShader for TestGpuCanvasShader {
    fn ore_shader_entry(
        &self,
        _: GpuCanvasShaderStage,
        _: &str,
    ) -> Option<nuxie_ore_metal::gpu_resource::AnyResourceHandle> {
        Some(self.1.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct GpuRecordingFactory {
    inner: RecordingFactory,
    shader_occurrences: Rc<RefCell<Vec<Arc<dyn RenderGpuCanvasShader>>>>,
    calls: Rc<RefCell<Vec<(GpuCanvasShader, GpuCanvasPlan)>>>,
    gpu: recording_gpu::RecordingGpu,
}

impl GpuRecordingFactory {
    fn new() -> Self {
        Self {
            inner: RecordingFactory::new(),
            shader_occurrences: Rc::new(RefCell::new(Vec::new())),
            calls: Rc::new(RefCell::new(Vec::new())),
            gpu: recording_gpu::RecordingGpu::new(),
        }
    }
}

impl Factory for GpuRecordingFactory {
    fn is_render_context(&self) -> bool {
        true
    }
    fn ore(&mut self) -> Option<nuxie_render_api::OreContextHandle> {
        Some(self.gpu.context.clone())
    }
    fn make_render_canvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Box<dyn nuxie_render_api::RenderCanvas>, nuxie_render_api::RenderCanvasError> {
        Ok(recording_gpu::canvas(width, height))
    }
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.inner
            .make_render_buffer(buffer_type, flags, size_in_bytes)
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.inner.make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.inner.decode_image(data)
    }

    fn make_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let occurrence: Arc<dyn RenderGpuCanvasShader> = Arc::new(TestGpuCanvasShader(
            shader.clone(),
            self.gpu
                .shader(self.shader_occurrences.borrow().len() as u64 + 1, shader),
        ));
        self.shader_occurrences
            .borrow_mut()
            .push(Arc::clone(&occurrence));
        Ok(occurrence)
    }

    fn make_gpu_canvas_image(
        &mut self,
        vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
        fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        assert!(
            Arc::ptr_eq(vertex_shader, fragment_shader),
            "combined shader fixture must use the same handle for both stages"
        );
        let vertex_shader = vertex_shader
            .as_any()
            .downcast_ref::<TestGpuCanvasShader>()
            .expect("test vertex shader handle");
        fragment_shader
            .as_any()
            .downcast_ref::<TestGpuCanvasShader>()
            .expect("test fragment shader handle");
        self.calls
            .borrow_mut()
            .push((vertex_shader.0.clone(), plan.clone()));
        Ok(Box::new(TestImage {
            identity: Rc::new(()),
            width: plan.width,
            height: plan.height,
        }))
    }
}

fn import_default_artboard(
    bytes: &[u8],
    factory: &mut dyn Factory,
) -> (ScriptedFile, RuntimeArtboardInstanceHandle) {
    let scripted = import_unsigned_scripted(
        bytes,
        factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("trusted scripted fixture imports");
    let artboard = scripted
        .native_file()
        .with_file(File::artboard_default)
        .expect("fixture artboard");
    (scripted, artboard)
}

#[test]
fn foldered_shader_resolves_through_bare_and_qualified_aliases_as_distinct_occurrences() {
    let mut factory = PersistentFactory::new(GpuRecordingFactory::new());
    let (_file, artboard) = import_default_artboard(&foldered_shader_alias_file(), &mut factory);
    let mut renderer = factory.borrow().inner.make_renderer();

    artboard.advance_default(0.0);
    artboard.draw(&mut renderer);

    let factory = factory.borrow();
    let occurrences = factory.shader_occurrences.borrow();
    assert_eq!(
        occurrences.len(),
        2,
        "each successful alias lookup must create one shader occurrence"
    );
    assert!(
        !Arc::ptr_eq(&occurrences[0], &occurrences[1]),
        "bare and qualified lookups must create distinct shader occurrences"
    );
}

#[test]
fn unused_contentless_shader_asset_does_not_prevent_script_boot() {
    let mut factory = PersistentFactory::new(GpuRecordingFactory::new());
    let (_file, artboard) = import_default_artboard(
        &contentless_shader_file(UNUSED_CONTENTLESS_SHADER_SCRIPT),
        &mut factory,
    );
    let mut renderer = factory.borrow().inner.make_renderer();

    artboard.advance_default(0.0);
    artboard.draw(&mut renderer);

    assert!(factory.borrow().shader_occurrences.borrow().is_empty());
}

#[test]
fn requested_contentless_shader_asset_returns_nil_and_script_continues() {
    let mut factory = PersistentFactory::new(GpuRecordingFactory::new());
    let (_file, artboard) = import_default_artboard(
        &contentless_shader_file(REQUESTED_CONTENTLESS_SHADER_SCRIPT),
        &mut factory,
    );
    let mut renderer = factory.borrow().inner.make_renderer();

    artboard.advance_default(0.0);
    artboard.draw(&mut renderer);

    assert!(factory.borrow().shader_occurrences.borrow().is_empty());
}

#[test]
fn shader_alias_collisions_preserve_the_first_owner_per_alias() {
    let mut factory = PersistentFactory::new(GpuRecordingFactory::new());
    let (_file, artboard) = import_default_artboard(&colliding_shader_alias_file(), &mut factory);
    let mut renderer = factory.borrow().inner.make_renderer();

    artboard.advance_default(0.0);
    artboard.draw(&mut renderer);

    let factory = factory.borrow();
    let occurrences = factory.shader_occurrences.borrow();
    let sources = occurrences
        .iter()
        .map(|shader| {
            shader
                .as_any()
                .downcast_ref::<TestGpuCanvasShader>()
                .expect("test shader occurrence")
                .0
                .source
                .as_str()
        })
        .collect::<Vec<_>>();
    assert_eq!(sources.len(), 3);
    assert!(sources[0].contains("first-owner"));
    assert!(sources[1].contains("first-owner"));
    assert!(sources[2].contains("second-owner"));
}

#[test]
fn imported_shader_and_script_execute_and_composite_through_one_factory() {
    let mut factory = PersistentFactory::new(GpuRecordingFactory::new());
    let (_file, artboard) = import_default_artboard(&imported_file(), &mut factory);
    let mut renderer = factory.borrow().inner.make_renderer();

    artboard.advance_default(0.0);
    artboard.draw(&mut renderer);

    let factory_ref = factory.borrow();
    assert!(
        factory_ref.calls.borrow().is_empty(),
        "recorded draws must not use the retired immediate plan submission"
    );
    let occurrences = factory_ref.shader_occurrences.borrow();
    assert_eq!(occurrences.len(), 1);
    let shader = &occurrences[0]
        .as_any()
        .downcast_ref::<TestGpuCanvasShader>()
        .expect("authored shader occurrence")
        .0;
    assert_eq!(
        shader
            .entry(GpuCanvasShaderStage::Vertex, "chosen_vertex")
            .expect("vertex entry")
            .physical_entry_point,
        "physical_vertex_1",
    );
    assert_eq!(
        shader
            .entry(GpuCanvasShaderStage::Fragment, "chosen_fragment")
            .expect("fragment entry")
            .physical_entry_point,
        "physical_fragment_1",
    );
    assert_eq!(&*factory_ref.gpu.pipelines.borrow(), &[(1, 1, true)]);
    assert_eq!(
        &*factory_ref.gpu.pipeline_entries.borrow(),
        &[(
            "physical_vertex_1".to_owned(),
            "physical_fragment_1".to_owned()
        )]
    );
    assert_eq!(factory_ref.gpu.draws.get(), 1);
    assert_eq!(&*factory_ref.gpu.draw_vertices.borrow(), &[3]);
    assert_eq!(&*factory_ref.gpu.canvas_sizes.borrow(), &[(32, 24)]);
    drop(occurrences);
    let stream = factory_ref.inner.stream();
    assert!(stream.contains("drawImage image=0"), "{stream}");
    assert!(
        stream.contains("sampler={wrapX=0,wrapY=0,filter=1,key=9}"),
        "{stream}"
    );
    assert!(stream.contains("blendMode=3 opacity=1"), "{stream}");
}

#[test]
fn scripted_drawable_hydrates_authored_boolean_input_before_init() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let (file, artboard) = import_default_artboard(&authored_boolean_input_file(), &mut factory);
    let mut renderer = factory.borrow().make_renderer();

    artboard.advance_default(0.0);
    artboard.draw(&mut renderer);

    let second = file
        .native_file()
        .with_file(File::artboard_default)
        .expect("second fixture artboard occurrence");
    second.advance_default(0.0);
    second.draw(&mut renderer);
}

#[test]
fn default_factory_does_not_silently_draw_an_unsupported_gpu_canvas() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let (_file, artboard) = import_default_artboard(&imported_file(), &mut factory);
    let mut renderer = factory.borrow().make_renderer();

    // A factory without an ORE recorder cannot create a scripted GPU canvas;
    // script failure must not publish an immediate fallback image.
    artboard.advance_default(0.0);
    artboard.draw(&mut renderer);
    let stream = factory.borrow().stream();
    assert!(
        !stream.contains("drawImage"),
        "unsupported GPU canvas reached the renderer: {stream}"
    );
}
