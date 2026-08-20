#![cfg(all(
    feature = "ore-metal-authored-msl",
    any(target_os = "ios", target_os = "macos")
))]

use std::any::Any;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie::ore_metal_gpu_canvas::{OreMetalGpuCanvas, OreMetalGpuCanvasImage};
use nuxie::{
    ColorInt, Factory, File, FillRule, GpuCanvasError, GpuCanvasPipelineShaders, GpuCanvasPlan,
    GpuCanvasShaderArtifact, GpuCanvasShaderProfile, ImageDecodeError, PersistentFactory, RawPath,
    RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderGpuCanvasShader,
    RenderImage, RenderPaint, RenderPath, RenderShader, ScriptExecutionCapability,
    ScriptExecutionLimits, ScriptHostEffects, ScriptHostExtension, ScriptHostExtensionInstance,
    ScriptVm,
};
use nuxie_schema::definition_by_name;
use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};
use sha2::{Digest as _, Sha256};

const WIDTH: u32 = 16;
const HEIGHT: u32 = 16;
const EXPECTED_PIXEL: [u8; 4] = [64, 128, 191, 255];

const PROBE_MSL: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct VertexOutput {
    float4 position [[position]];
};

vertex VertexOutput vertex_main(uint vertex_index [[vertex_id]]) {
    const float2 positions[3] = {
        float2(-1.0, -1.0),
        float2(3.0, -1.0),
        float2(-1.0, 3.0),
    };
    VertexOutput output;
    output.position = float4(positions[vertex_index], 0.0, 1.0);
    return output;
}

fragment float4 fragment_main(
    constant float4& first [[buffer(0)]],
    constant float4& second [[buffer(1)]]) {
    return first + second;
}
"#;

const BINDING_MAP: &[u8] = &[
    2, 1, 14, 0, 2, 0, 0, 0, // v2 header, two 14-byte rows.
    0, 0, 0, 2, 0, 0xff, 0xff, 0, 0, 0xff, 0xff, 2, 1, 0, // group 0, binding 0.
    2, 3, 0, 2, 2, 0xff, 0xff, 1, 0, 0xff, 0xff, 2, 1, 0, // group 2, binding 3.
];

const SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local shader = context:shader("scene")

    local firstBytes = buffer.create(272)
    buffer.writef32(firstBytes, 256, 0.125)
    buffer.writef32(firstBytes, 260, 0.25)
    buffer.writef32(firstBytes, 264, 0.375)
    buffer.writef32(firstBytes, 268, 0.5)
    local secondBytes = buffer.create(16)
    buffer.writef32(secondBytes, 0, 0.125)
    buffer.writef32(secondBytes, 4, 0.25)
    buffer.writef32(secondBytes, 8, 0.375)
    buffer.writef32(secondBytes, 12, 0.5)

    local firstBuffer = GPUBuffer.new {
        size = 272, usage = "uniform", data = firstBytes, immutable = true,
    }
    local secondBuffer = GPUBuffer.new {
        size = 16, usage = "uniform", data = secondBytes, immutable = true,
    }
    local firstLayout = GPUBindGroupLayout.new {
        groupIndex = 0, shader = shader, dynamicUBOs = { 0 },
    }
    local secondLayout = GPUBindGroupLayout.new { groupIndex = 2, shader = shader }
    local firstGroup = GPUBindGroup.new {
        layout = firstLayout,
        ubos = { { slot = 0, buffer = firstBuffer, offset = 0, size = 16 } },
    }
    local secondGroup = GPUBindGroup.new {
        layout = secondLayout,
        ubos = { { slot = 3, buffer = secondBuffer, offset = 0, size = 16 } },
    }
    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "vertex_main" },
        fragment = { module = shader, entryPoint = "fragment_main" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
        bindGroupLayouts = { firstLayout, secondLayout },
    }
    local sampler = ImageSampler("clamp", "clamp", "nearest")
    canvas:resize(16, 16)
    return {
        drawCanvas = function(self)
            local pass = canvas:beginRenderPass {
                color = { {
                    loadOp = "clear",
                    storeOp = "store",
                    clearColor = { 0, 0, 0, 1 },
                } },
            }
            pass:setPipeline(pipeline)
            pass:setBindGroup(0, firstGroup, { 256 })
            pass:setBindGroup(2, secondGroup)
            pass:draw(3)
            pass:finish()
        end,
        draw = function(self, renderer)
            renderer:drawImage(canvas.image, sampler, "srcOver", 1.0)
        end,
    }
end
"#;

#[derive(Debug)]
struct NoopExtension;

#[derive(Debug)]
struct NoopExtensionInstance;

impl ScriptHostExtension for NoopExtension {
    fn install(
        &self,
        _vm: &ScriptVm,
    ) -> Result<Box<dyn ScriptHostExtensionInstance>, nuxie::ScriptError> {
        Ok(Box::new(NoopExtensionInstance))
    }
}

impl ScriptHostExtensionInstance for NoopExtensionInstance {
    fn effects_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<()>()
    }

    fn begin_cycle(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn rollback_cycle(&self, _checkpoint: Box<dyn Any>) -> Result<(), nuxie::ScriptError> {
        Ok(())
    }

    fn checkpoint_effects(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn rollback_effects(&self, _checkpoint: Box<dyn Any>) -> Result<(), nuxie::ScriptError> {
        Ok(())
    }

    fn drain_effects(&self) -> ScriptHostEffects {
        ScriptHostEffects::new(())
    }
}

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

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
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

fn source_container() -> Vec<u8> {
    let entries = [
        (0_u8, "vertex_main", "vertex_main"),
        (1, "fragment_main", "fragment_main"),
    ];
    let mut source = vec![entries.len() as u8];
    for (stage, logical, physical) in entries {
        source.push(stage);
        put_string(&mut source, logical);
        put_string(&mut source, physical);
    }
    put_u32(&mut source, PROBE_MSL.len() as u32);
    source.extend_from_slice(PROBE_MSL.as_bytes());
    source
}

fn put_interface(
    bytes: &mut Vec<u8>,
    kind: u8,
    value: u16,
    interface_type: u8,
    interpolation: u8,
    sampling: u8,
) {
    bytes.push(kind);
    put_u16(bytes, value);
    bytes.extend_from_slice(&[interface_type, interpolation, sampling]);
}

fn supplemental_reflection(source: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1];
    bytes.extend_from_slice(&Sha256::digest(source));
    bytes.extend_from_slice(&Sha256::digest(BINDING_MAP));
    bytes.push(2);

    bytes.push(0);
    put_string(&mut bytes, "vertex_main");
    put_string(&mut bytes, "vertex_main");
    for dimension in [1_u32; 3] {
        put_u32(&mut bytes, dimension);
    }
    bytes.extend_from_slice(&[1, 1]);
    put_interface(&mut bytes, 1, 0, 8, 0xff, 0xff);
    put_interface(&mut bytes, 1, 2, 3, 0xff, 0xff);

    bytes.push(1);
    put_string(&mut bytes, "fragment_main");
    put_string(&mut bytes, "fragment_main");
    for dimension in [1_u32; 3] {
        put_u32(&mut bytes, dimension);
    }
    bytes.extend_from_slice(&[0, 1]);
    put_interface(&mut bytes, 0, 0, 3, 0xff, 0xff);

    put_u16(&mut bytes, 2);
    for (group, binding) in [(0_u8, 0_u8), (2, 3)] {
        bytes.extend_from_slice(&[group, binding]);
        put_u16(&mut bytes, 1);
        bytes.extend_from_slice(&16_u64.to_le_bytes());
    }
    bytes
}

fn shader_payload() -> Vec<u8> {
    let source = source_container();
    let reflection = supplemental_reflection(&source);
    let variants = [(2_u8, source), (10, BINDING_MAP.to_vec())];
    let mut offset = 0_u32;
    let mut descriptors = Vec::new();
    for (target, blob) in &variants {
        descriptors.push((*target, offset, blob.len()));
        offset = offset
            .checked_add(u32::try_from(blob.len()).expect("small fixture"))
            .expect("small fixture offset");
    }
    let mut payload = vec![0];
    put_u32(&mut payload, 0x5253_5442);
    put_u16(&mut payload, 4);
    payload.extend_from_slice(&[2, 1]);
    for (target, offset, size) in descriptors {
        payload.push(target);
        put_u32(&mut payload, offset);
        put_u32(&mut payload, u32::try_from(size).expect("small fixture"));
    }
    payload.push(2);
    put_u16(
        &mut payload,
        u16::try_from(reflection.len()).expect("small fixture reflection"),
    );
    payload.extend_from_slice(&reflection);
    for (_, blob) in variants {
        payload.extend_from_slice(&blob);
    }
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
        push_string(bytes, "ScriptAsset", "name", "OreMetalProbe");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", WIDTH as f32);
        push_f32(bytes, "Artboard", "height", HEIGHT as f32);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
    });
    bytes
}

struct OreProbeFactory {
    inner: RecordingFactory,
    ore: OreMetalGpuCanvas,
    artifact_calls: usize,
    occurrence_calls: usize,
    image_calls: usize,
    module_identities: Vec<usize>,
    errors: Vec<String>,
    pixels: Rc<RefCell<Vec<Arc<[u8]>>>>,
}

impl OreProbeFactory {
    fn new() -> Result<Self, GpuCanvasError> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| GpuCanvasError::new("Metal device is unavailable"))?;
        let queue = device
            .newCommandQueue()
            .ok_or_else(|| GpuCanvasError::new("Metal command queue creation failed"))?;
        Ok(Self {
            inner: RecordingFactory::new(),
            ore: OreMetalGpuCanvas::from_device_queue(device, queue),
            artifact_calls: 0,
            occurrence_calls: 0,
            image_calls: 0,
            module_identities: Vec::new(),
            errors: Vec::new(),
            pixels: Rc::new(RefCell::new(Vec::new())),
        })
    }
}

impl Factory for OreProbeFactory {
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

    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        self.ore.shader_profile()
    }

    fn make_gpu_canvas_shader_artifact(
        &mut self,
        shader: &GpuCanvasShaderArtifact,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.artifact_calls += 1;
        let GpuCanvasShaderArtifact::TrustedAppleMetal(shader_bytes) = shader else {
            panic!("the ORE product probe must never request a WebGPU artifact");
        };
        assert_eq!(shader_bytes.source(), PROBE_MSL);
        assert_eq!(shader_bytes.binding_map_bytes(), BINDING_MAP);
        assert_eq!(shader_bytes.entries().len(), 2);
        assert_eq!(shader_bytes.bindings().len(), 2);
        let result = self.ore.make_shader_artifact(shader);
        if let Ok(prepared) = &result {
            self.module_identities
                .push(self.ore.shader_module_identity(prepared)?);
        }
        if let Err(error) = &result {
            self.errors.push(error.to_string());
        }
        result
    }

    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.occurrence_calls += 1;
        let occurrence = self.ore.make_shader_occurrence(prepared)?;
        self.module_identities
            .push(self.ore.shader_module_identity(&occurrence)?);
        Ok(occurrence)
    }

    fn make_gpu_canvas_image_with_pipelines(
        &mut self,
        pipelines: &[GpuCanvasPipelineShaders],
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        self.image_calls += 1;
        let image = self.ore.make_image_with_pipelines(pipelines, plan)?;
        let image_ref = image
            .as_any()
            .downcast_ref::<OreMetalGpuCanvasImage>()
            .expect("ORE adapter returns its retained image type");
        self.pixels.borrow_mut().push(Arc::from(image_ref.pixels()));
        Ok(image)
    }
}

#[test]
fn authenticated_rive_shader_flows_through_factory_into_ore_metal_pixels() {
    let bytes = imported_file();
    // SAFETY: this test module is the trusted exporter boundary for these
    // fixed, reviewed MSL bytes. It binds the exact target-2 source, target-10
    // map, supplemental reflection, script, and enclosing Rive bytes, and the
    // rooted gate compiles and executes that source under Metal validation.
    let capability = unsafe {
        ScriptExecutionCapability::for_verified_native_shader_artifact_unchecked(
            &bytes,
            Arc::new(NoopExtension),
        )
        .expect("fixture authority")
    };
    let file =
        File::import_with_execution_capability(&bytes, capability, ScriptExecutionLimits::new())
            .expect("authenticated fixture imports");
    let mut instance = file
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()
        .expect("fixture instance");
    let mut factory = PersistentFactory::new(OreProbeFactory::new().expect("Metal adapter"));
    assert!(
        block_on(instance.mount_scripted_drawables_async(&mut factory))
            .expect("authenticated scripted drawable mounts")
    );
    let mut renderer = factory.borrow().inner.make_renderer();

    if let Err(error) = instance.draw(&mut factory, &mut renderer) {
        let factory = factory.borrow();
        panic!(
            "authenticated GPUCanvas draw failed: {error:#}; backend errors={:?}; calls={}/{}/{}",
            factory.errors, factory.artifact_calls, factory.occurrence_calls, factory.image_calls,
        );
    }

    let factory = factory.borrow();
    assert_eq!(factory.artifact_calls, 1, "one prepared module per asset");
    assert_eq!(factory.occurrence_calls, 1, "one fresh lookup occurrence");
    assert_eq!(factory.image_calls, 1, "one explicit pipeline submission");
    assert_eq!(factory.module_identities.len(), 2);
    assert_ne!(
        factory.module_identities[0], factory.module_identities[1],
        "the lookup occurrence must own a freshly compiled MTLLibrary"
    );
    let pixels = factory.pixels.borrow();
    let [pixels] = pixels.as_slice() else {
        panic!("expected one retained readback");
    };
    assert_eq!(pixels.len(), usize::try_from(WIDTH * HEIGHT * 4).unwrap());
    assert!(
        pixels.chunks_exact(4).all(|pixel| pixel == EXPECTED_PIXEL),
        "every pixel must be the exact sum of the two authenticated UBOs"
    );
    assert!(factory.inner.stream().contains("drawImage image=0"));
}

#[test]
fn generic_script_trust_never_reaches_native_shader_compilation() {
    let bytes = imported_file();
    let file = File::import_with_unsigned_scripts(&bytes).expect("generic script trust imports");
    let mut instance = file
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()
        .expect("fixture instance");
    let mut factory = PersistentFactory::new(OreProbeFactory::new().expect("Metal adapter"));
    let mut renderer = factory.borrow().inner.make_renderer();

    let error = instance
        .draw(&mut factory, &mut renderer)
        .expect_err("native shader authority is required");
    assert!(
        format!("{error:#}").contains("error converting Lua nil to AnyUserData"),
        "{error:#}"
    );
    let factory = factory.borrow();
    assert_eq!(factory.artifact_calls, 0);
    assert_eq!(factory.occurrence_calls, 0);
    assert_eq!(factory.image_calls, 0);
}
