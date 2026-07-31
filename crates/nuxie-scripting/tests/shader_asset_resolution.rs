#![cfg(feature = "luau")]

use std::any::Any;
use std::sync::{Arc, Weak};

use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie_render_api::{
    ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasPlan, GpuCanvasShader, ImageDecodeError,
    RawPath, RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType,
    RenderGpuCanvasShader, RenderImage, RenderPaint, RenderPath, RenderShader,
};
use nuxie_runtime::{NoopScriptHost, ScriptMethod, ScriptValue};
use nuxie_scripting::vm::ScriptVm;

const UNUSED_SHADER_SCRIPT: &[u8] = br#"
return function(_context)
    return {
        initialized = false,
        init = function(self)
            self.initialized = true
            return true
        end,
        evaluate = function(self)
            return self.initialized
        end,
    }
end
"#;

const REQUESTED_SHADER_SCRIPT: &[u8] = br##"
return function(context)
    local shader = context:shader("legacy")
    local returnCount = select("#", context:shader("legacy"))
    return {
        evaluate = function(self)
            return shader == nil and returnCount == 0
        end,
    }
end
"##;

const LOOKUP_ONLY_SCRIPT: &[u8] = br#"
return function(context)
    context:shader("scene")
    return {}
end
"#;

const NIL_SHADER_PIPELINE_SCRIPT: &[u8] = br#"
return function(context)
    local shader = context:shader("legacy")
    GPUPipeline.new {
        vertex = { module = shader },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    return {}
end
"#;

const TWO_SAME_NAME_STAGE_LOOKUPS_SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local vertexShader = context:shader("scene")
    local fragmentShader = context:shader("scene")
    local pipeline = GPUPipeline.new {
        vertex = { module = vertexShader, entryPoint = "first_vertex" },
        fragment = { module = fragmentShader, entryPoint = "first_fragment" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    canvas:resize(8, 8)
    return {
        drawCanvas = function(self)
            local pass = canvas:beginRenderPass {
                color = { { loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 } } },
            }
            pass:setPipeline(pipeline)
            pass:draw(3)
            pass:finish()
        end,
    }
end
"#;

const TWO_PIPELINES_ONE_LOOKUP_SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local shader = context:shader("scene")
    local first = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "first_vertex" },
        fragment = { module = shader, entryPoint = "first_fragment" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    local second = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "second_vertex" },
        fragment = { module = shader, entryPoint = "second_fragment" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    local frame = 0
    canvas:resize(8, 8)
    return {
        drawCanvas = function(self)
            frame = frame + 1
            local pass = canvas:beginRenderPass {
                color = { { loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 } } },
            }
            pass:setPipeline(if frame == 1 then first else second)
            pass:draw(3)
            pass:finish()
        end,
    }
end
"#;

const DISTINCT_STAGE_SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local vertexShader = context:shader("vertex_scene")
    local fragmentShader = context:shader("fragment_scene")
    local pipeline = GPUPipeline.new {
        vertex = { module = vertexShader, entryPoint = "vertex_entry" },
        fragment = { module = fragmentShader, entryPoint = "fragment_entry" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    canvas:resize(8, 8)
    return {
        drawCanvas = function(self)
            local pass = canvas:beginRenderPass {
                color = { { loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 } } },
            }
            pass:setPipeline(pipeline)
            pass:draw(3)
            pass:finish()
        end,
    }
end
"#;

const COMBINED_STAGE_SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local shader = context:shader("scene")
    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "first_vertex" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    canvas:resize(8, 8)
    return {
        drawCanvas = function(self)
            local pass = canvas:beginRenderPass {
                color = { { loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 } } },
            }
            pass:setPipeline(pipeline)
            pass:draw(3)
            pass:finish()
        end,
    }
end
"#;

const DRAW_CANVAS_FIRST_LOOKUP_SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    canvas:resize(8, 8)
    return {
        drawCanvas = function(self)
            local shader = context:shader("scene")
            local pipeline = GPUPipeline.new {
                vertex = { module = shader, entryPoint = "first_vertex" },
                vertexLayout = {},
                colorTargets = { { format = "rgba8unorm" } },
            }
            local pass = canvas:beginRenderPass {
                color = { { loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 } } },
            }
            pass:setPipeline(pipeline)
            pass:draw(3)
            pass:finish()
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
    assert!(!output.is_null(), "pinned Luau compiler returned null");
    // SAFETY: luaur returns a valid allocation containing output_size bytes.
    unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
}

fn script_payload(source: &[u8]) -> Vec<u8> {
    let mut payload = vec![0];
    payload.extend(compile_luau(source));
    payload
}

fn target_1_only_shader_payload() -> Vec<u8> {
    let retired_glsl = b"retired target-1 GLSL";
    let mut payload = vec![0];
    payload.extend_from_slice(&0x5253_5442_u32.to_le_bytes());
    payload.extend_from_slice(&4_u16.to_le_bytes());
    payload.extend_from_slice(&[1, 0]);
    payload.push(1);
    payload.extend_from_slice(&0_u32.to_le_bytes());
    payload.extend_from_slice(&(retired_glsl.len() as u32).to_le_bytes());
    payload.extend_from_slice(retired_glsl);
    payload
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

fn webgpu_shader_payload(entries: &[(u8, &str, &str)], wgsl: &str) -> Vec<u8> {
    const EMPTY_BINDING_MAP: &[u8] = &[2, 1, 14, 0, 0, 0, 0, 0];
    let mut source = vec![entries.len() as u8];
    for (stage, logical, physical) in entries {
        source.push(*stage);
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

fn complete_shader_payload(color: &str) -> Vec<u8> {
    webgpu_shader_payload(
        &[
            (0, "first_vertex", "vs_first"),
            (0, "second_vertex", "vs_second"),
            (1, "first_fragment", "fs_first"),
            (1, "second_fragment", "fs_second"),
        ],
        &format!("complete shader source {color}"),
    )
}

fn empty_plan() -> GpuCanvasPlan {
    GpuCanvasPlan {
        vertex_entry: None,
        fragment_entry: None,
        width: 8,
        height: 8,
        clear_color: [0.0, 0.0, 0.0, 1.0],
        vertex_count: 3,
        instance_count: 1,
        first_vertex: 0,
        first_instance: 0,
        uniform_buffers: Vec::new(),
        vertex_layouts: Vec::new(),
        vertex_buffers: Vec::new(),
    }
}

#[derive(Debug)]
struct ObservedShader {
    id: u64,
    domain: Weak<()>,
}

impl RenderGpuCanvasShader for ObservedShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct TestImage;

impl RenderImage for TestImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        8
    }

    fn height(&self) -> u32 {
        8
    }
}

struct ObservingFactory {
    inner: RecordingFactory,
    domain: Arc<()>,
    module_error: Option<&'static str>,
    module_sources: Vec<String>,
    image_calls: Vec<(u64, u64, bool)>,
}

impl ObservingFactory {
    fn new() -> Self {
        Self {
            inner: RecordingFactory::new(),
            domain: Arc::new(()),
            module_error: None,
            module_sources: Vec::new(),
            image_calls: Vec::new(),
        }
    }

    fn fail_module_creation(&mut self, message: &'static str) {
        self.module_error = Some(message);
    }
}

impl Factory for ObservingFactory {
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
        if let Some(message) = self.module_error {
            return Err(GpuCanvasError::new(message));
        }
        let id = self.module_sources.len() as u64 + 1;
        self.module_sources.push(shader.source.clone());
        Ok(Arc::new(ObservedShader {
            id,
            domain: Arc::downgrade(&self.domain),
        }))
    }

    fn make_gpu_canvas_image(
        &mut self,
        vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
        fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
        _plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        let vertex = vertex_shader
            .as_any()
            .downcast_ref::<ObservedShader>()
            .ok_or_else(|| GpuCanvasError::new("foreign vertex shader backend"))?;
        let fragment = fragment_shader
            .as_any()
            .downcast_ref::<ObservedShader>()
            .ok_or_else(|| GpuCanvasError::new("foreign fragment shader backend"))?;
        for shader in [vertex, fragment] {
            let domain = shader
                .domain
                .upgrade()
                .ok_or_else(|| GpuCanvasError::new("shader domain expired"))?;
            if !Arc::ptr_eq(&domain, &self.domain) {
                return Err(GpuCanvasError::new(
                    "shader belongs to another factory/device domain",
                ));
            }
        }
        self.image_calls.push((
            vertex.id,
            fragment.id,
            Arc::ptr_eq(vertex_shader, fragment_shader),
        ));
        Ok(Box::new(TestImage))
    }
}

#[test]
fn unused_incompatible_shader_does_not_poison_unrelated_script_boot() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("legacy", &target_1_only_shader_payload())
        .expect("registration retains an unused shader without selecting a backend");

    let mut host = NoopScriptHost;
    let mut factory = RecordingFactory::new();
    let mut instance = vm
        .instantiate_script_with_factory(
            "unrelated",
            &script_payload(UNUSED_SHADER_SCRIPT),
            &mut host,
            &mut factory,
        )
        .expect("unrelated protocol script instantiates");

    assert!(
        instance
            .call_init_with_factory(&mut host, &mut factory)
            .expect("unrelated script initializes")
    );
    assert_eq!(
        instance
            .call_method(ScriptMethod::Evaluate, &[], &mut host)
            .expect("unrelated script executes"),
        ScriptValue::Bool(true)
    );
}

#[test]
fn requested_incompatible_shader_returns_nil_and_execution_continues() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("legacy", &target_1_only_shader_payload())
        .expect("registration retains the shader without selecting a backend");

    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();
    let mut instance = vm
        .instantiate_script_with_factory(
            "requesting",
            &script_payload(REQUESTED_SHADER_SCRIPT),
            &mut host,
            &mut factory,
        )
        .expect("context:shader returns no values for the incompatible shader");

    assert_eq!(
        instance
            .call_method(ScriptMethod::Evaluate, &[], &mut host)
            .expect("script continues after failed shader lookup"),
        ScriptValue::Bool(true)
    );
    assert!(factory.module_sources.is_empty());
}

#[test]
fn malformed_neutral_shader_registers_then_returns_nil_when_requested() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("legacy", &[0, 1, 2, 3])
        .expect("public file registration retains failed neutral decode state");

    let mut host = NoopScriptHost;
    let mut unused_factory = ObservingFactory::new();
    vm.instantiate_script_with_factory(
        "unrelated",
        &script_payload(UNUSED_SHADER_SCRIPT),
        &mut host,
        &mut unused_factory,
    )
    .expect("malformed unused shader does not poison boot");
    assert!(unused_factory.module_sources.is_empty());

    let mut requested_factory = ObservingFactory::new();
    let mut instance = vm
        .instantiate_script_with_factory(
            "requesting",
            &script_payload(REQUESTED_SHADER_SCRIPT),
            &mut host,
            &mut requested_factory,
        )
        .expect("the retained neutral decode failure returns no values at exact lookup");
    assert_eq!(
        instance
            .call_method(ScriptMethod::Evaluate, &[], &mut host)
            .expect("script continues after malformed shader lookup"),
        ScriptValue::Bool(true)
    );
    assert!(requested_factory.module_sources.is_empty());
}

#[test]
fn missing_shader_returns_nil_and_execution_continues() {
    let mut vm = ScriptVm::new();
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();
    let mut instance = vm
        .instantiate_script_with_factory(
            "requesting",
            &script_payload(REQUESTED_SHADER_SCRIPT),
            &mut host,
            &mut factory,
        )
        .expect("a missing shader returns no values");

    assert_eq!(
        instance
            .call_method(ScriptMethod::Evaluate, &[], &mut host)
            .expect("script continues after missing shader lookup"),
        ScriptValue::Bool(true)
    );
    assert!(factory.module_sources.is_empty());
}

#[test]
fn backend_module_creation_failure_returns_nil_and_execution_continues() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("legacy", &complete_shader_payload("backend-rejected"))
        .expect("valid shader registers");
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();
    factory.fail_module_creation("backend rejected module");
    let mut instance = vm
        .instantiate_script_with_factory(
            "requesting",
            &script_payload(REQUESTED_SHADER_SCRIPT),
            &mut host,
            &mut factory,
        )
        .expect("backend module failure returns no values");

    assert_eq!(
        instance
            .call_method(ScriptMethod::Evaluate, &[], &mut host)
            .expect("script continues after backend module failure"),
        ScriptValue::Bool(true)
    );
    assert!(factory.module_sources.is_empty());
}

#[test]
fn pipeline_construction_still_fails_closed_when_given_a_nil_shader() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("legacy", &target_1_only_shader_payload())
        .expect("target-incompatible shader registers");
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();

    vm.instantiate_script_with_factory(
        "nil-pipeline",
        &script_payload(NIL_SHADER_PIPELINE_SCRIPT),
        &mut host,
        &mut factory,
    )
    .err()
    .expect("GPUPipeline must reject a nil module");
    assert!(factory.module_sources.is_empty());
}

#[test]
fn duplicate_registration_rejects_the_second_source_and_preserves_the_first() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("first-source"))
        .expect("first source registers");
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("second-source"))
        .expect_err("the existing stronger Rust duplicate policy rejects replacement");

    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();
    vm.instantiate_script_with_factory(
        "lookup",
        &script_payload(LOOKUP_ONLY_SCRIPT),
        &mut host,
        &mut factory,
    )
    .expect("the first source remains resolvable");

    assert_eq!(factory.module_sources.len(), 1);
    assert!(factory.module_sources[0].contains("first-source"));
    assert!(!factory.module_sources[0].contains("second-source"));
}

#[test]
fn successful_lookup_without_a_pipeline_creates_one_module() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("lookup-only"))
        .unwrap();
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();

    vm.instantiate_script_with_factory(
        "lookup",
        &script_payload(LOOKUP_ONLY_SCRIPT),
        &mut host,
        &mut factory,
    )
    .unwrap();

    assert_eq!(factory.module_sources.len(), 1);
    assert!(factory.image_calls.is_empty());
}

#[test]
fn first_shader_lookup_inside_draw_canvas_uses_active_factory_and_combined_handle() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("draw-lazy"))
        .unwrap();
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();
    let mut instance = vm
        .instantiate_script_with_factory(
            "draw-lazy",
            &script_payload(DRAW_CANVAS_FIRST_LOOKUP_SCRIPT),
            &mut host,
            &mut factory,
        )
        .unwrap();

    assert!(
        factory.module_sources.is_empty(),
        "generator construction must not resolve the shader"
    );

    let mut renderer = factory.inner.make_renderer();
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();

    assert_eq!(factory.module_sources.len(), 1);
    assert_eq!(factory.image_calls, vec![(1, 1, true)]);
}

#[test]
fn one_occurrence_keeps_one_identity_across_two_pipeline_keys() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("two-pipelines"))
        .unwrap();
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();
    let mut instance = vm
        .instantiate_script_with_factory(
            "two-pipelines",
            &script_payload(TWO_PIPELINES_ONE_LOOKUP_SCRIPT),
            &mut host,
            &mut factory,
        )
        .unwrap();
    let mut renderer = factory.inner.make_renderer();

    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();

    assert_eq!(factory.module_sources.len(), 1);
    assert_eq!(
        factory.image_calls,
        vec![(1, 1, true), (1, 1, true)],
        "both pipeline descriptors retain the exact lookup occurrence"
    );
}

#[test]
fn two_same_name_lookups_create_distinct_module_identities() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("same-name"))
        .unwrap();
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();

    let mut instance = vm
        .instantiate_script_with_factory(
            "two-lookups",
            &script_payload(TWO_SAME_NAME_STAGE_LOOKUPS_SCRIPT),
            &mut host,
            &mut factory,
        )
        .unwrap();
    let mut renderer = factory.inner.make_renderer();
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();

    assert_eq!(factory.module_sources.len(), 2);
    assert_eq!(factory.module_sources[0], factory.module_sources[1]);
    assert_eq!(factory.image_calls, vec![(1, 2, false)]);
}

#[test]
fn explicit_different_name_stages_and_combined_fallback_keep_exact_handles() {
    let mut vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset(
        "vertex_scene",
        &webgpu_shader_payload(&[(0, "vertex_entry", "vs")], "vertex-only-source"),
    )
    .unwrap();
    vm.register_gpu_canvas_shader_asset(
        "fragment_scene",
        &webgpu_shader_payload(&[(1, "fragment_entry", "fs")], "fragment-only-source"),
    )
    .unwrap();
    let mut host = NoopScriptHost;
    let mut factory = ObservingFactory::new();
    let mut instance = vm
        .instantiate_script_with_factory(
            "distinct-stages",
            &script_payload(DISTINCT_STAGE_SCRIPT),
            &mut host,
            &mut factory,
        )
        .unwrap();
    let mut renderer = factory.inner.make_renderer();
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();

    assert_eq!(
        factory.module_sources,
        vec!["vertex-only-source", "fragment-only-source"]
    );
    assert_eq!(factory.image_calls, vec![(1, 2, false)]);

    let mut combined_vm = ScriptVm::new();
    combined_vm
        .register_gpu_canvas_shader_asset("scene", &complete_shader_payload("combined"))
        .unwrap();
    let mut combined_factory = ObservingFactory::new();
    let mut combined = combined_vm
        .instantiate_script_with_factory(
            "combined",
            &script_payload(COMBINED_STAGE_SCRIPT),
            &mut host,
            &mut combined_factory,
        )
        .unwrap();
    let mut combined_renderer = combined_factory.inner.make_renderer();
    combined
        .call_draw(&mut combined_factory, &mut combined_renderer, &mut host)
        .unwrap();

    assert_eq!(combined_factory.module_sources.len(), 1);
    assert_eq!(combined_factory.image_calls, vec![(1, 1, true)]);
}

#[test]
fn opaque_shader_handles_are_rejected_by_another_factory_domain() {
    let shader = GpuCanvasShader {
        source: "domain-source".into(),
        entries: Vec::new(),
        bindings: Vec::new(),
    };
    let mut first = ObservingFactory::new();
    let handle = first.make_gpu_canvas_shader(&shader).unwrap();
    let mut second = ObservingFactory::new();
    let error = second
        .make_gpu_canvas_image(&handle, &handle, &empty_plan())
        .err()
        .expect("a foreign occurrence must not be rehomed");

    assert!(error.to_string().contains("another factory/device domain"));
}
