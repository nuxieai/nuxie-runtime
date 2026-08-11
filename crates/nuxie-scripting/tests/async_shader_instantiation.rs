#![cfg(feature = "luau")]

//! UNIV-1764: native repro of the browser-only async instantiation path.
//!
//! Browser WebGPU validates imported shader assets asynchronously before the
//! synchronous script generator runs. These tests drive the exact production
//! entry point (`instantiate_registered_script_with_factory_async`) with a
//! factory whose `load_gpu_canvas_shader` stays `Pending` across polls,
//! matching the real Chrome preparation timeline without a browser.

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};

use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie_render_api::{
    ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasShader, GpuCanvasShaderLoad,
    ImageDecodeError, PersistentFactory, RawPath, RecordingFactory, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderGpuCanvasShader, RenderImage, RenderPaint,
    RenderPath, RenderShader,
};
use nuxie_runtime::NoopScriptHost;
use nuxie_scripting::vm::ScriptVm;

// Mirrors the browser smoke fixture: canvas created before the await, shader
// awaited through the asynchronous factory load, pipeline/sampler built after
// resume, and the returned closures method-index the captured userdata after
// the coroutine has completed.
const AWAITED_SHADER_SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local shader = context:shader("scene")
    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "first_vertex" },
        fragment = { module = shader, entryPoint = "first_fragment" },
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
        draw = function(self, renderer) end,
    }
end
"#;

// Same timeline for a device-rejected module: the awaited lookup must resolve
// to zero Lua values and execution must continue.
const AWAITED_REJECTED_SHADER_SCRIPT: &[u8] = br##"
return function(context)
    local shader = context:shader("scene")
    local returnCount = select("#", context:shader("scene"))
    return {
        evaluate = function(self)
            return shader == nil and returnCount == 0
        end,
    }
end
"##;

const LAZY_SHADER_SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local pipeline = nil
    return {
        drawCanvas = function(self)
            if pipeline == nil then
                local shader = context:shader("scene")
                pipeline = GPUPipeline.new {
                    vertex = { module = shader, entryPoint = "first_vertex" },
                    fragment = { module = shader, entryPoint = "first_fragment" },
                    vertexLayout = {},
                    colorTargets = { { format = "rgba8unorm" } },
                }
                canvas:resize(8, 8)
            end
            local pass = canvas:beginRenderPass {
                color = { { loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 } } },
            }
            pass:setPipeline(pipeline)
            pass:draw(3)
            pass:finish()
        end,
        draw = function(self, renderer) end,
    }
end
"#;

const TWO_LOOKUP_SHADER_SCRIPT: &[u8] = br#"
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
        draw = function(self, renderer) end,
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

fn complete_shader_payload(color: &str) -> Vec<u8> {
    const EMPTY_BINDING_MAP: &[u8] = &[2, 1, 14, 0, 0, 0, 0, 0];
    let entries: &[(u8, &str, &str)] = &[
        (0, "first_vertex", "vs_first"),
        (1, "first_fragment", "fs_first"),
    ];
    let wgsl = format!("complete shader source {color}");
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

#[derive(Debug)]
struct ObservedShader {
    domain: Weak<()>,
    occurrence_id: u64,
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

/// Stays `Pending` for `pending_polls` polls before resolving, so pre-generator
/// preparation suspends and resumes exactly like the browser's validation
/// promise.
struct DeferredLoad {
    pending_polls: u32,
    result: Option<Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError>>,
}

impl Future for DeferredLoad {
    type Output = Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.pending_polls > 0 {
            self.pending_polls -= 1;
            context.waker().wake_by_ref();
            return Poll::Pending;
        }
        Poll::Ready(
            self.result
                .take()
                .expect("deferred shader load polled after completion"),
        )
    }
}

struct AsyncFactory {
    inner: RecordingFactory,
    domain: Arc<()>,
    reject_modules: bool,
    load_calls: u32,
    make_calls: u32,
    occurrence_calls: u32,
    next_occurrence_id: u64,
    image_occurrences: Vec<(u64, u64)>,
}

impl AsyncFactory {
    fn new(reject_modules: bool) -> Self {
        Self {
            inner: RecordingFactory::new(),
            domain: Arc::new(()),
            reject_modules,
            load_calls: 0,
            make_calls: 0,
            occurrence_calls: 0,
            next_occurrence_id: 1,
            image_occurrences: Vec::new(),
        }
    }

    fn fresh_observed_shader(&mut self) -> Arc<dyn RenderGpuCanvasShader> {
        let occurrence_id = self.next_occurrence_id;
        self.next_occurrence_id += 1;
        Arc::new(ObservedShader {
            domain: Arc::downgrade(&self.domain),
            occurrence_id,
        })
    }
}

impl Factory for AsyncFactory {
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
        _shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.make_calls += 1;
        if self.reject_modules {
            return Err(GpuCanvasError::new("device rejected the physical module"));
        }
        Ok(self.fresh_observed_shader())
    }

    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.occurrence_calls += 1;
        let observed = prepared
            .as_any()
            .downcast_ref::<ObservedShader>()
            .ok_or_else(|| GpuCanvasError::new("foreign shader backend"))?;
        let domain = observed
            .domain
            .upgrade()
            .ok_or_else(|| GpuCanvasError::new("shader domain expired"))?;
        if !Arc::ptr_eq(&domain, &self.domain) {
            return Err(GpuCanvasError::new(
                "shader belongs to another factory/device domain",
            ));
        }
        Ok(self.fresh_observed_shader())
    }

    fn load_gpu_canvas_shader(&mut self, shader: &GpuCanvasShader) -> GpuCanvasShaderLoad {
        self.load_calls += 1;
        let result = self.make_gpu_canvas_shader(shader);
        GpuCanvasShaderLoad::Pending(Box::pin(DeferredLoad {
            pending_polls: 3,
            result: Some(result),
        }))
    }

    fn make_gpu_canvas_image(
        &mut self,
        vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
        fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
        _plan: &nuxie_render_api::GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        let mut occurrence_ids = [0; 2];
        for (index, shader) in [vertex_shader, fragment_shader].into_iter().enumerate() {
            let observed = shader
                .as_any()
                .downcast_ref::<ObservedShader>()
                .ok_or_else(|| GpuCanvasError::new("foreign shader backend"))?;
            let domain = observed
                .domain
                .upgrade()
                .ok_or_else(|| GpuCanvasError::new("shader domain expired"))?;
            if !Arc::ptr_eq(&domain, &self.domain) {
                return Err(GpuCanvasError::new(
                    "shader belongs to another factory/device domain",
                ));
            }
            occurrence_ids[index] = observed.occurrence_id;
        }
        self.image_occurrences
            .push((occurrence_ids[0], occurrence_ids[1]));
        Ok(Box::new(TestImage))
    }
}

/// Single-future executor: the deferred loads self-wake, so re-polling in a
/// loop is exactly the browser's microtask cadence with no runtime dependency.
fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut future = Box::pin(future);
    let waker = std::task::Waker::noop();
    let mut context = Context::from_waker(waker);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
    }
}

#[test]
fn awaited_shader_closure_keeps_captured_canvas_after_coroutine_completes() {
    let vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("awaited"))
        .unwrap();
    let mut factory = PersistentFactory::new(AsyncFactory::new(false));
    let program = vm
        .register_protocol_script_with_factory(
            "awaited",
            &script_payload(AWAITED_SHADER_SCRIPT),
            &mut factory,
        )
        .unwrap();
    let mut instance =
        block_on(vm.instantiate_registered_script_with_factory_async(&program, &mut factory))
            .expect("generator completes across the pending shader load");
    assert_eq!(factory.borrow().load_calls, 1);

    let mut renderer = factory.borrow().inner.make_renderer();
    let mut host = NoopScriptHost;
    // The captured canvas/pipeline upvalues must still be the live userdata
    // once the coroutine is dead; UNIV-1764's browser abort surfaced here as
    // a Luau index error inside drawCanvas.
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .expect("drawCanvas method-indexes the captured canvas after the await");
}

#[test]
fn prepared_shader_is_available_to_lazy_draw_canvas_lookup() {
    let vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("lazy"))
        .unwrap();
    let mut factory = PersistentFactory::new(AsyncFactory::new(false));
    let program = vm
        .register_protocol_script_with_factory(
            "lazy",
            &script_payload(LAZY_SHADER_SCRIPT),
            &mut factory,
        )
        .unwrap();
    let mut instance =
        block_on(vm.instantiate_registered_script_with_factory_async(&program, &mut factory))
            .expect("shader catalog preparation succeeds before a lazy lookup");
    assert_eq!(factory.borrow().load_calls, 1);

    let mut renderer = factory.borrow().inner.make_renderer();
    let mut host = NoopScriptHost;
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .expect("drawCanvas reuses the prepared physical module");
    assert_eq!(factory.borrow().load_calls, 1);
}

#[test]
fn prepared_same_name_lookups_publish_distinct_shader_occurrences() {
    let vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("two-lookups"))
        .unwrap();
    let mut factory = PersistentFactory::new(AsyncFactory::new(false));
    let program = vm
        .register_protocol_script_with_factory(
            "two-lookups",
            &script_payload(TWO_LOOKUP_SHADER_SCRIPT),
            &mut factory,
        )
        .unwrap();
    let mut instance =
        block_on(vm.instantiate_registered_script_with_factory_async(&program, &mut factory))
            .expect("prepared asset publishes a fresh occurrence for each lookup");

    assert_eq!(factory.borrow().load_calls, 1);
    assert_eq!(factory.borrow().occurrence_calls, 2);

    let mut renderer = factory.borrow().inner.make_renderer();
    let mut host = NoopScriptHost;
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .expect("distinct occurrences can be used in one pipeline");
    let image_occurrences = factory.borrow().image_occurrences.clone();
    assert_eq!(image_occurrences.len(), 1);
    assert_ne!(image_occurrences[0].0, image_occurrences[0].1);
}

#[test]
fn awaited_rejected_shader_returns_zero_values_and_execution_continues() {
    let vm = ScriptVm::new();
    vm.register_gpu_canvas_shader_asset("scene", &complete_shader_payload("rejected"))
        .unwrap();
    let mut factory = PersistentFactory::new(AsyncFactory::new(true));
    let program = vm
        .register_protocol_script_with_factory(
            "rejected",
            &script_payload(AWAITED_REJECTED_SHADER_SCRIPT),
            &mut factory,
        )
        .unwrap();
    let mut instance =
        block_on(vm.instantiate_registered_script_with_factory_async(&program, &mut factory))
            .expect("a rejected physical module is zero Lua values, not an error");
    assert_eq!(factory.borrow().load_calls, 1);
    assert_eq!(factory.borrow().make_calls, 3);

    let mut host = NoopScriptHost;
    assert_eq!(
        instance
            .call_method(nuxie_runtime::ScriptMethod::Evaluate, &[], &mut host)
            .unwrap(),
        nuxie_runtime::ScriptValue::Bool(true)
    );

    let mut retry =
        block_on(vm.instantiate_registered_script_with_factory_async(&program, &mut factory))
            .expect("a later instantiation retries a previously rejected module");
    assert_eq!(factory.borrow().load_calls, 2);
    assert_eq!(factory.borrow().make_calls, 6);
    assert_eq!(
        retry
            .call_method(nuxie_runtime::ScriptMethod::Evaluate, &[], &mut host)
            .unwrap(),
        nuxie_runtime::ScriptValue::Bool(true)
    );
}
