//! Browser execution owner for the exact WebGL2 translation.
//!
//! The translated renderer intentionally talks in GLES-style numeric names.
//! WebGL exposes JavaScript objects instead, so this provider preserves the
//! same per-namespace name tables as Emscripten's GL bridge and executes every
//! command synchronously against one retained `WebGL2RenderingContext`.

#![allow(non_snake_case)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use js_sys::{
    Array, Float32Array, Function, Int16Array, Int32Array, Int8Array, Object, Promise, Reflect,
    Uint16Array, Uint32Array, Uint8Array,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlCanvasElement};

use super::gles3_decl::{
    GLCommand, GLContextLifecycleIngress, GLFinalReleaseIngress, GLObjectKind,
    GLExecutionProvider, WebGLShaderPixelLocalStorageEnableResult, GLenum, GLint, GLuint,
    GL_ARRAY_BUFFER_BINDING, GL_CURRENT_PROGRAM, GL_ELEMENT_ARRAY_BUFFER_BINDING,
    GL_FRAMEBUFFER_BINDING, GL_RENDERER, GL_RGBA, GL_UNIFORM_BUFFER_BINDING,
    GL_UNSIGNED_BYTE, GL_VERSION, GL_VERTEX_ARRAY_BINDING,
};

thread_local! {
    static FINAL_RELEASE_INGRESSES: RefCell<HashMap<u64, GLFinalReleaseIngress>> =
        RefCell::new(HashMap::new());
}

static NEXT_WAKE_ID: AtomicU64 = AtomicU64::new(1);

struct BrowserFinalReleaseWake {
    id: u64,
}

impl nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake for BrowserFinalReleaseWake {
    fn post(&self) {
        let id = self.id;
        let callback = Closure::once_into_js(move || {
            FINAL_RELEASE_INGRESSES.with(|ingresses| {
                if let Some(ingress) = ingresses.borrow().get(&id) {
                    ingress.drainFinalReleases();
                }
            });
        });
        let promise: JsValue = Promise::resolve(&JsValue::UNDEFINED).into();
        invoke(&promise, "then", &[callback]);
    }
}

#[derive(Default)]
struct NameTable {
    next: GLuint,
    values: HashMap<GLuint, JsValue>,
}

impl NameTable {
    fn insert(&mut self, value: JsValue) -> GLuint {
        if self.next == 0 {
            self.next = 1;
        }
        let name = self.next;
        self.next = self.next.checked_add(1).expect("WebGL object name overflow");
        self.values.insert(name, value);
        name
    }

    fn get(&self, name: GLuint) -> JsValue {
        if name == 0 {
            JsValue::NULL
        } else {
            self.values
                .get(&name)
                .cloned()
                .unwrap_or(JsValue::NULL)
        }
    }

    fn remove(&mut self, name: GLuint) -> JsValue {
        self.values.remove(&name).unwrap_or(JsValue::NULL)
    }

    fn name_of(&self, value: &JsValue) -> Option<GLuint> {
        self.values
            .iter()
            .find_map(|(name, candidate)| Object::is(candidate, value).then_some(*name))
    }

    fn clear(&mut self) {
        self.values.clear();
    }
}

#[derive(Default)]
struct BrowserNames {
    buffers: NameTable,
    framebuffers: NameTable,
    programs: NameTable,
    renderbuffers: NameTable,
    samplers: NameTable,
    shaders: NameTable,
    textures: NameTable,
    vertex_arrays: NameTable,
    uniform_locations: NameTable,
    query_values: HashMap<u64, JsValue>,
}

impl BrowserNames {
    fn table(&self, kind: GLObjectKind) -> &NameTable {
        match kind {
            GLObjectKind::Buffer => &self.buffers,
            GLObjectKind::Framebuffer => &self.framebuffers,
            GLObjectKind::Program => &self.programs,
            GLObjectKind::Renderbuffer => &self.renderbuffers,
            GLObjectKind::Sampler => &self.samplers,
            GLObjectKind::Texture => &self.textures,
            GLObjectKind::VertexArray => &self.vertex_arrays,
        }
    }

    fn table_mut(&mut self, kind: GLObjectKind) -> &mut NameTable {
        match kind {
            GLObjectKind::Buffer => &mut self.buffers,
            GLObjectKind::Framebuffer => &mut self.framebuffers,
            GLObjectKind::Program => &mut self.programs,
            GLObjectKind::Renderbuffer => &mut self.renderbuffers,
            GLObjectKind::Sampler => &mut self.samplers,
            GLObjectKind::Texture => &mut self.textures,
            GLObjectKind::VertexArray => &mut self.vertex_arrays,
        }
    }

    fn clear(&mut self) {
        self.buffers.clear();
        self.framebuffers.clear();
        self.programs.clear();
        self.renderbuffers.clear();
        self.samplers.clear();
        self.shaders.clear();
        self.textures.clear();
        self.vertex_arrays.clear();
        self.uniform_locations.clear();
        self.query_values.clear();
    }
}

pub(crate) struct BrowserWebGl2Provider {
    canvas: HtmlCanvasElement,
    gl: JsValue,
    names: BrowserNames,
    extensions: HashMap<String, JsValue>,
    pixel_local_storage: Option<JsValue>,
    provoking_vertex: Option<JsValue>,
    context_lost_listener: Option<Closure<dyn FnMut(Event)>>,
    context_restored_listener: Option<Closure<dyn FnMut(Event)>>,
    wake_id: Option<u64>,
}

impl BrowserWebGl2Provider {
    pub(crate) fn new(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<(Self, String, u32), JsValue> {
        canvas.set_width(width);
        canvas.set_height(height);
        let options = Object::new();
        for (name, value) in [
            ("alpha", JsValue::TRUE),
            ("depth", JsValue::FALSE),
            ("stencil", JsValue::FALSE),
            ("antialias", JsValue::FALSE),
            ("premultipliedAlpha", JsValue::TRUE),
            ("preserveDrawingBuffer", JsValue::TRUE),
            ("failIfMajorPerformanceCaveat", JsValue::FALSE),
        ] {
            Reflect::set(&options, &JsValue::from_str(name), &value)?;
        }
        Reflect::set(
            &options,
            &JsValue::from_str("powerPreference"),
            &JsValue::from_str("high-performance"),
        )?;
        let gl: JsValue = canvas
            .get_context_with_context_options("webgl2", &options)?
            .ok_or_else(|| JsValue::from_str("WebGL2 context is unavailable"))?
            .into();
        let adapter_name = invoke(&gl, "getParameter", &[number(GL_RENDERER)])
            .as_string()
            .unwrap_or_default();
        if adapter_name.is_empty() {
            return Err(JsValue::from_str("WebGL2 returned an empty renderer name"));
        }
        invoke(&gl, "bindFramebuffer", &[number(0x8D40_u32), JsValue::NULL]);
        invoke(
            &gl,
            "viewport",
            &[number(0_i32), number(0_i32), number(width), number(height)],
        );
        let sample_count = invoke(&gl, "getParameter", &[number(0x80A9_u32)])
            .as_f64()
            .unwrap_or(0.0) as u32;
        Ok((
            Self {
                canvas,
                gl,
                names: BrowserNames::default(),
                extensions: HashMap::new(),
                pixel_local_storage: None,
                provoking_vertex: None,
                context_lost_listener: None,
                context_restored_listener: None,
                wake_id: None,
            },
            adapter_name,
            sample_count,
        ))
    }

    fn object(&self, kind: GLObjectKind, name: GLuint) -> JsValue {
        self.names.table(kind).get(name)
    }

    fn create_named(&mut self, kind: GLObjectKind, method: &str) -> GLuint {
        let object = self.call(method, &[]);
        if object.is_null() || object.is_undefined() {
            return 0;
        }
        self.names.table_mut(kind).insert(object)
    }

    fn call(&self, method: &str, args: &[JsValue]) -> JsValue {
        invoke(&self.gl, method, args)
    }

    fn call_extension(extension: &JsValue, method: &str, args: &[JsValue]) -> JsValue {
        invoke(extension, method, args)
    }

    fn delete_named(&mut self, kind: GLObjectKind, name: GLuint, method: &str) {
        let object = self.names.table_mut(kind).remove(name);
        self.call(method, &[object]);
    }

    fn queried_name(&self, parameter: GLenum, value: &JsValue) -> GLint {
        if value.is_null() || value.is_undefined() {
            return 0;
        }
        let name = match parameter {
            GL_CURRENT_PROGRAM => self.names.programs.name_of(value),
            GL_ARRAY_BUFFER_BINDING | GL_ELEMENT_ARRAY_BUFFER_BINDING | GL_UNIFORM_BUFFER_BINDING => {
                self.names.buffers.name_of(value)
            }
            GL_FRAMEBUFFER_BINDING => self.names.framebuffers.name_of(value),
            GL_VERTEX_ARRAY_BINDING => self.names.vertex_arrays.name_of(value),
            _ => None,
        };
        name.map_or_else(|| value.as_f64().unwrap_or(0.0) as GLint, |name| name as GLint)
    }

    fn shader(&self, name: GLuint) -> JsValue {
        self.names.shaders.get(name)
    }

    fn program(&self, name: GLuint) -> JsValue {
        self.names.programs.get(name)
    }

    fn uniform_location(&self, name: GLint) -> JsValue {
        if name < 0 {
            JsValue::NULL
        } else {
            self.names.uniform_locations.get(name as GLuint)
        }
    }

    fn log_shader_error(&self, shader: GLuint) {
        let log = self
            .call("getShaderInfoLog", &[self.shader(shader)])
            .as_string()
            .unwrap_or_default();
        if !log.is_empty() {
            console_call("error", &format!("RIVE: Shader compilation error: {log}"));
        }
    }

    fn log_program_error(&self, program: GLuint) {
        let log = self
            .call("getProgramInfoLog", &[self.program(program)])
            .as_string()
            .unwrap_or_default();
        if !log.is_empty() {
            console_call("error", &format!("RIVE: Program link error: {log}"));
        }
    }
}

impl GLExecutionProvider for BrowserWebGl2Provider {
    fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
        let lost_ingress = ingress.clone();
        let lost = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            event.prevent_default();
            lost_ingress.contextLost();
        });
        self.canvas
            .add_event_listener_with_callback("webglcontextlost", lost.as_ref().unchecked_ref())
            .expect("install WebGL context-loss listener");

        let restored = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            let _ = ingress.contextRestored();
        });
        self.canvas
            .add_event_listener_with_callback(
                "webglcontextrestored",
                restored.as_ref().unchecked_ref(),
            )
            .expect("install WebGL context-restoration listener");
        self.context_lost_listener = Some(lost);
        self.context_restored_listener = Some(restored);
    }

    fn installFinalReleaseIngress(
        &mut self,
        ingress: GLFinalReleaseIngress,
    ) -> Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
        let id = NEXT_WAKE_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, 0, "WebGL final-release wake identity overflow");
        FINAL_RELEASE_INGRESSES.with(|ingresses| {
            assert!(ingresses.borrow_mut().insert(id, ingress).is_none());
        });
        self.wake_id = Some(id);
        Arc::new(BrowserFinalReleaseWake { id })
    }

    fn submit(&mut self, command: GLCommand) {
        match command {
            GLCommand::Clear(mask) => { self.call("clear", &[number(mask)]); }
            GLCommand::ClearColor(r, g, b, a) => { self.call("clearColor", &[number(r), number(g), number(b), number(a)]); }
            GLCommand::FrontFace(mode) => { self.call("frontFace", &[number(mode)]); }
            GLCommand::DepthRange(near, far) => { self.call("depthRange", &[number(near), number(far)]); }
            GLCommand::DepthFunc(function) => { self.call("depthFunc", &[number(function)]); }
            GLCommand::ClearDepth(depth) => { self.call("clearDepth", &[number(depth)]); }
            GLCommand::ClearStencil(stencil) => { self.call("clearStencil", &[number(stencil)]); }
            GLCommand::Enable(capability) => { self.call("enable", &[number(capability)]); }
            GLCommand::Disable(capability) => { self.call("disable", &[number(capability)]); }
            GLCommand::PixelStore(parameter, value) => { self.call("pixelStorei", &[number(parameter), number(value)]); }
            GLCommand::BindAttribLocation { program, index, name } => {
                self.call("bindAttribLocation", &[self.program(program), number(index), JsValue::from_str(bytes_string(&name))]);
            }
            GLCommand::BindBuffer(target, buffer) => { self.call("bindBuffer", &[number(target), self.object(GLObjectKind::Buffer, buffer)]); }
            GLCommand::BindBufferRange { target, index, buffer, offset, size } => {
                self.call("bindBufferRange", &[number(target), number(index), self.object(GLObjectKind::Buffer, buffer), number(offset), number(size)]);
            }
            GLCommand::BindFramebuffer(target, framebuffer) => { self.call("bindFramebuffer", &[number(target), self.object(GLObjectKind::Framebuffer, framebuffer)]); }
            GLCommand::BindFramebufferFromQuery(target, slot) => {
                let value = self.names.query_values.get(&slot).cloned().unwrap_or(JsValue::NULL);
                self.call("bindFramebuffer", &[number(target), value]);
            }
            GLCommand::BindRenderbuffer(target, renderbuffer) => { self.call("bindRenderbuffer", &[number(target), self.object(GLObjectKind::Renderbuffer, renderbuffer)]); }
            GLCommand::BindSampler(unit, sampler) => { self.call("bindSampler", &[number(unit), self.object(GLObjectKind::Sampler, sampler)]); }
            GLCommand::ProvokingVertex(mode) => {
                if let Some(extension) = self.provoking_vertex.as_ref() {
                    Self::call_extension(extension, "provokingVertexWEBGL", &[number(mode)]);
                }
            }
            GLCommand::Scissor(x, y, width, height) => { self.call("scissor", &[number(x), number(y), number(width), number(height)]); }
            GLCommand::Viewport(x, y, width, height) => { self.call("viewport", &[number(x), number(y), number(width), number(height)]); }
            GLCommand::PolygonOffset(factor, units) => { self.call("polygonOffset", &[number(factor), number(units)]); }
            GLCommand::CullFace(mode) => { self.call("cullFace", &[number(mode)]); }
            GLCommand::BlendEquation(mode) => { self.call("blendEquation", &[number(mode)]); }
            GLCommand::BlendEquationSeparate(rgb, alpha) => { self.call("blendEquationSeparate", &[number(rgb), number(alpha)]); }
            GLCommand::BlendFunc(source, destination) => { self.call("blendFunc", &[number(source), number(destination)]); }
            GLCommand::BlendFuncSeparate(src_rgb, dst_rgb, src_alpha, dst_alpha) => { self.call("blendFuncSeparate", &[number(src_rgb), number(dst_rgb), number(src_alpha), number(dst_alpha)]); }
            GLCommand::BlendColor(r, g, b, a) => { self.call("blendColor", &[number(r), number(g), number(b), number(a)]); }
            GLCommand::ColorMask(r, g, b, a) => { self.call("colorMask", &[JsValue::from_bool(r), JsValue::from_bool(g), JsValue::from_bool(b), JsValue::from_bool(a)]); }
            GLCommand::DepthMask(enabled) => { self.call("depthMask", &[JsValue::from_bool(enabled)]); }
            GLCommand::StencilMask(mask) => { self.call("stencilMask", &[number(mask)]); }
            GLCommand::StencilMaskSeparate(face, mask) => { self.call("stencilMaskSeparate", &[number(face), number(mask)]); }
            GLCommand::StencilFunc(function, reference, mask) => { self.call("stencilFunc", &[number(function), number(reference), number(mask)]); }
            GLCommand::StencilOp(fail, depth_fail, pass) => { self.call("stencilOp", &[number(fail), number(depth_fail), number(pass)]); }
            GLCommand::StencilFuncSeparate(face, function, reference, mask) => { self.call("stencilFuncSeparate", &[number(face), number(function), number(reference), number(mask)]); }
            GLCommand::StencilOpSeparate(face, fail, depth_fail, pass) => { self.call("stencilOpSeparate", &[number(face), number(fail), number(depth_fail), number(pass)]); }
            GLCommand::UseProgram(program) => { self.call("useProgram", &[self.program(program)]); }
            GLCommand::BindVertexArray(array) => { self.call("bindVertexArray", &[self.object(GLObjectKind::VertexArray, array)]); }
            GLCommand::BindVertexArrayFromQuery(slot) => {
                let value = self.names.query_values.get(&slot).cloned().unwrap_or(JsValue::NULL);
                self.call("bindVertexArray", &[value]);
            }
            GLCommand::ClearBufferDepthStencil { buffer, drawbuffer, depth, stencil } => { self.call("clearBufferfi", &[number(buffer), number(drawbuffer), number(depth), number(stencil)]); }
            GLCommand::ClearBufferFloat { buffer, drawbuffer, values, value_count } => {
                let array = Float32Array::from(&values[..usize::from(value_count)]);
                self.call("clearBufferfv", &[number(buffer), number(drawbuffer), array.into()]);
            }
            GLCommand::ClearBufferInt { buffer, drawbuffer, values, value_count } => {
                let array = Int32Array::from(&values[..usize::from(value_count)]);
                self.call("clearBufferiv", &[number(buffer), number(drawbuffer), array.into()]);
            }
            GLCommand::ClearBufferUInt { buffer, drawbuffer, values, value_count } => {
                let array = Uint32Array::from(&values[..usize::from(value_count)]);
                self.call("clearBufferuiv", &[number(buffer), number(drawbuffer), array.into()]);
            }
            GLCommand::EnableVertexAttribArray(index) => { self.call("enableVertexAttribArray", &[number(index)]); }
            GLCommand::DisableVertexAttribArray(index) => { self.call("disableVertexAttribArray", &[number(index)]); }
            GLCommand::VertexAttribIPointer { index, size, type_, stride, offset } => { self.call("vertexAttribIPointer", &[number(index), number(size), number(type_), number(stride), number(offset)]); }
            GLCommand::VertexAttribPointer { index, size, type_, normalized, stride, offset } => { self.call("vertexAttribPointer", &[number(index), number(size), number(type_), JsValue::from_bool(normalized != 0), number(stride), number(offset)]); }
            GLCommand::VertexAttribDivisor(index, divisor) => { self.call("vertexAttribDivisor", &[number(index), number(divisor)]); }
            GLCommand::DrawArrays { mode, first, count } => { self.call("drawArrays", &[number(mode), number(first), number(count)]); }
            GLCommand::DrawArraysInstanced { mode, first, count, instanceCount } => { self.call("drawArraysInstanced", &[number(mode), number(first), number(count), number(instanceCount)]); }
            GLCommand::DrawElements { mode, count, type_, offset } => { self.call("drawElements", &[number(mode), number(count), number(type_), number(offset)]); }
            GLCommand::DrawElementsInstanced { mode, count, type_, offset, instanceCount } => { self.call("drawElementsInstanced", &[number(mode), number(count), number(type_), number(offset), number(instanceCount)]); }
            GLCommand::DrawElementsInstancedBaseInstance { mode, count, type_, offset, instance_count, base_instance } => {
                let extension = self.extensions.get("WEBGL_draw_instanced_base_vertex_base_instance").expect("source admitted base-instance extension");
                Self::call_extension(extension, "drawElementsInstancedBaseVertexBaseInstanceWEBGL", &[number(mode), number(count), number(type_), number(offset), number(instance_count), number(0), number(base_instance)]);
            }
            GLCommand::BlendBarrierKHR => {
                if let Some(extension) = self.extensions.get("KHR_blend_equation_advanced") {
                    Self::call_extension(extension, "blendBarrierKHR", &[]);
                }
            }
            GLCommand::ReadBuffer(mode) => { self.call("readBuffer", &[number(mode)]); }
            GLCommand::FramebufferTexture2D { target, attachment, texture_target, texture, level } => { self.call("framebufferTexture2D", &[number(target), number(attachment), number(texture_target), self.object(GLObjectKind::Texture, texture), number(level)]); }
            GLCommand::FramebufferTextureLayer { target, attachment, texture, level, layer } => { self.call("framebufferTextureLayer", &[number(target), number(attachment), self.object(GLObjectKind::Texture, texture), number(level), number(layer)]); }
            GLCommand::FramebufferRenderbuffer { target, attachment, renderbuffer_target, renderbuffer } => { self.call("framebufferRenderbuffer", &[number(target), number(attachment), number(renderbuffer_target), self.object(GLObjectKind::Renderbuffer, renderbuffer)]); }
            GLCommand::DrawBuffers(buffers) => { self.call("drawBuffers", &[Uint32Array::from(buffers.as_slice()).into()]); }
            GLCommand::Flush => { self.call("flush", &[]); }
            GLCommand::GenerateMipmap(target) => { self.call("generateMipmap", &[number(target)]); }
            GLCommand::InvalidateFramebuffer { target, attachments } => { self.call("invalidateFramebuffer", &[number(target), Uint32Array::from(attachments.as_slice()).into()]); }
            GLCommand::LineWidth(width) => { self.call("lineWidth", &[number(width)]); }
            GLCommand::FramebufferTexturePixelLocalStorageANGLE { plane, backing_texture, level, layer, usage } => {
                if let Some(extension) = self.pixel_local_storage.as_ref() {
                    Self::call_extension(extension, "framebufferTexturePixelLocalStorageWEBGL", &[number(plane), self.object(GLObjectKind::Texture, backing_texture), number(level), number(layer), number(usage)]);
                }
            }
            GLCommand::FramebufferPixelLocalClearValuefvANGLE { plane, value } => {
                if let Some(extension) = self.pixel_local_storage.as_ref() {
                    Self::call_extension(extension, "framebufferPixelLocalClearValuefvWEBGL", &[number(plane), Float32Array::from(value.as_slice()).into()]);
                }
            }
            GLCommand::BeginPixelLocalStorageANGLE { load_ops } => {
                if let Some(extension) = self.pixel_local_storage.as_ref() {
                    Self::call_extension(extension, "beginPixelLocalStorageWEBGL", &[Uint32Array::from(load_ops.as_slice()).into()]);
                }
            }
            GLCommand::EndPixelLocalStorageANGLE { store_ops } => {
                if let Some(extension) = self.pixel_local_storage.as_ref() {
                    Self::call_extension(extension, "endPixelLocalStorageWEBGL", &[Uint32Array::from(store_ops.as_slice()).into()]);
                }
            }
            GLCommand::RenderbufferStorageMultisample { target, samples, internal_format, width, height } => { self.call("renderbufferStorageMultisample", &[number(target), number(samples), number(internal_format), number(width), number(height)]); }
            GLCommand::TexStorage2D { target, levels, internal_format, width, height } => { self.call("texStorage2D", &[number(target), number(levels), number(internal_format), number(width), number(height)]); }
            GLCommand::TexStorage3D { target, levels, internal_format, width, height, depth } => { self.call("texStorage3D", &[number(target), number(levels), number(internal_format), number(width), number(height), number(depth)]); }
            GLCommand::DeleteProgram(name) => self.delete_named(GLObjectKind::Program, name, "deleteProgram"),
            GLCommand::DeleteVertexArray(name) => self.delete_named(GLObjectKind::VertexArray, name, "deleteVertexArray"),
            GLCommand::DeleteBuffer(name) => self.delete_named(GLObjectKind::Buffer, name, "deleteBuffer"),
            GLCommand::DeleteTexture(name) => self.delete_named(GLObjectKind::Texture, name, "deleteTexture"),
            GLCommand::DeleteFramebuffer(name) => self.delete_named(GLObjectKind::Framebuffer, name, "deleteFramebuffer"),
            GLCommand::DeleteRenderbuffer(name) => self.delete_named(GLObjectKind::Renderbuffer, name, "deleteRenderbuffer"),
            GLCommand::DeleteSampler(name) => self.delete_named(GLObjectKind::Sampler, name, "deleteSampler"),
            GLCommand::GenerateBuffer(_) | GLCommand::GenerateTexture(_) | GLCommand::GenerateFramebuffer(_) | GLCommand::GenerateRenderbuffer(_) | GLCommand::GenerateSampler(_) | GLCommand::GenerateVertexArray(_) | GLCommand::CreateProgram(_) | GLCommand::CreateShader(_, _) => {
                panic!("production WebGL names are published synchronously, never replayed")
            }
            GLCommand::SamplerParameterFloat { sampler, parameter, value } => { self.call("samplerParameterf", &[self.object(GLObjectKind::Sampler, sampler), number(parameter), number(value)]); }
            GLCommand::SamplerParameterInt { sampler, parameter, value } => { self.call("samplerParameteri", &[self.object(GLObjectKind::Sampler, sampler), number(parameter), number(value)]); }
            GLCommand::ShaderSource(shader, source) => { self.call("shaderSource", &[self.shader(shader), JsValue::from_str(&source)]); }
            GLCommand::ShaderSourceBytes { shader, source } => { self.call("shaderSource", &[self.shader(shader), JsValue::from_str(source.as_deref().map(bytes_string).unwrap_or(""))]); }
            GLCommand::ShaderSourceBypassingEmscripten { shader, minimal_source: _, raw_source } => { self.call("shaderSource", &[self.shader(shader), JsValue::from_str(&raw_source)]); }
            GLCommand::CompileShader(shader) => { self.call("compileShader", &[self.shader(shader)]); }
            GLCommand::PrintShaderCompilationErrors(shader) => self.log_shader_error(shader),
            GLCommand::ValidateShaderCompilationAndAbort { shader, stderr_flush_delay_ms: _ } => {
                if self.shaderParameter(shader, 0x8B81) == 0 { self.log_shader_error(shader); panic!("exact WebGL shader compilation failed"); }
            }
            GLCommand::DeleteShader(name) => {
                let shader = self.names.shaders.remove(name);
                self.call("deleteShader", &[shader]);
            }
            GLCommand::AttachShader(program, shader) => { self.call("attachShader", &[self.program(program), self.shader(shader)]); }
            GLCommand::LinkProgram(program) => { self.call("linkProgram", &[self.program(program)]); }
            GLCommand::PrintLinkProgramErrors(program) => self.log_program_error(program),
            GLCommand::ValidateProgramLinkAndAbort(program) => {
                if self.programParameter(program, 0x8B82) == 0 { self.log_program_error(program); panic!("exact WebGL program link failed"); }
            }
            GLCommand::TextureParameter(target, parameter, value) => { self.call("texParameteri", &[number(target), number(parameter), number(value)]); }
            GLCommand::BlitFramebuffer(bounds, mask, filter) => { self.call("blitFramebuffer", &[number(bounds[0]), number(bounds[1]), number(bounds[2]), number(bounds[3]), number(bounds[4]), number(bounds[5]), number(bounds[6]), number(bounds[7]), number(mask), number(filter)]); }
            GLCommand::Uniform1iByName(program, name, value) => {
                let location = self.call("getUniformLocation", &[self.program(program), JsValue::from_str(&name)]);
                self.call("uniform1i", &[location, number(value)]);
            }
            GLCommand::GetInteger(parameter, slot) => {
                let value = self.call("getParameter", &[number(parameter)]);
                self.names.query_values.insert(slot, value);
            }
            GLCommand::BindBufferFromQuery(target, slot) => {
                let value = self.names.query_values.get(&slot).cloned().unwrap_or(JsValue::NULL);
                self.call("bindBuffer", &[number(target), value]);
            }
            GLCommand::BufferSubData { target, offset, data } => { self.call("bufferSubData", &[number(target), number(offset), Uint8Array::from(data.as_slice()).into()]); }
            GLCommand::BufferData { target, size, data, usage } => {
                let payload = data.map_or_else(|| number(size), |bytes| Uint8Array::from(bytes.as_slice()).into());
                self.call("bufferData", &[number(target), payload, number(usage)]);
            }
            GLCommand::ActiveTexture(texture) => { self.call("activeTexture", &[number(texture)]); }
            GLCommand::BindTexture(target, texture) => { self.call("bindTexture", &[number(target), self.object(GLObjectKind::Texture, texture)]); }
            GLCommand::CompressedTexSubImage2D { target, level, x, y, width, height, format, data } => { self.call("compressedTexSubImage2D", &[number(target), number(level), number(x), number(y), number(width), number(height), number(format), Uint8Array::from(data.as_slice()).into()]); }
            GLCommand::CompressedTexSubImage3D { target, level, x, y, z, width, height, depth, format, data } => { self.call("compressedTexSubImage3D", &[number(target), number(level), number(x), number(y), number(z), number(width), number(height), number(depth), number(format), Uint8Array::from(data.as_slice()).into()]); }
            GLCommand::TexSubImage2D { target, level, x, y, width, height, format, type_, data } => { self.call("texSubImage2D", &[number(target), number(level), number(x), number(y), number(width), number(height), number(format), number(type_), texture_pixel_data(type_, &data), number(0)]); }
            GLCommand::TexSubImage3D { target, level, x, y, z, width, height, depth, format, type_, data } => { self.call("texSubImage3D", &[number(target), number(level), number(x), number(y), number(z), number(width), number(height), number(depth), number(format), number(type_), texture_pixel_data(type_, &data), number(0)]); }
            GLCommand::PixelStoreFromQuery(parameter, slot) => {
                let value = self.names.query_values.get(&slot).cloned().unwrap_or_else(|| number(0));
                self.call("pixelStorei", &[number(parameter), value]);
            }
            GLCommand::Uniform1iLocation { location, value } => { self.call("uniform1i", &[self.uniform_location(location), number(value)]); }
            GLCommand::UniformBlockBinding { program, block_index, binding } => { self.call("uniformBlockBinding", &[self.program(program), number(block_index), number(binding)]); }
        }
    }

    fn generateObject(&mut self, kind: GLObjectKind) -> GLuint {
        let method = match kind {
            GLObjectKind::Buffer => "createBuffer",
            GLObjectKind::Framebuffer => "createFramebuffer",
            GLObjectKind::Program => return self.createProgram(),
            GLObjectKind::Renderbuffer => "createRenderbuffer",
            GLObjectKind::Sampler => "createSampler",
            GLObjectKind::Texture => "createTexture",
            GLObjectKind::VertexArray => "createVertexArray",
        };
        self.create_named(kind, method)
    }

    fn createProgram(&mut self) -> GLuint {
        self.create_named(GLObjectKind::Program, "createProgram")
    }

    fn createShader(&mut self, shaderType: GLenum) -> GLuint {
        let shader = self.call("createShader", &[number(shaderType)]);
        if shader.is_null() || shader.is_undefined() { 0 } else { self.names.shaders.insert(shader) }
    }

    fn getInteger(&mut self, parameter: GLenum) -> GLint {
        let value = self.call("getParameter", &[number(parameter)]);
        self.queried_name(parameter, &value)
    }

    fn getString(&mut self, parameter: GLenum) -> Option<Vec<u8>> {
        let mut value = self.call("getParameter", &[number(parameter)]).as_string()?;
        if parameter == GL_VERSION && !value.starts_with("OpenGL ES ") {
            value = format!("OpenGL ES 3.0 ({value})");
        }
        Some(value.into_bytes())
    }

    fn getExtension(&mut self, index: GLuint) -> Option<Vec<u8>> {
        let supported = self.call("getSupportedExtensions", &[]);
        let values = Array::from(&supported);
        values.get(index).as_string().map(String::into_bytes)
    }

    fn enableWebGLExtension(&mut self, name: &str) -> bool {
        let extension = self.call("getExtension", &[JsValue::from_str(name)]);
        if extension.is_null() || extension.is_undefined() { false } else {
            self.extensions.insert(name.to_owned(), extension);
            true
        }
    }

    fn enableWebGLShaderPixelLocalStorageCoherent(&mut self, warning: &'static str) -> WebGLShaderPixelLocalStorageEnableResult {
        let extension = self.call("getExtension", &[JsValue::from_str("WEBGL_shader_pixel_local_storage")]);
        if extension.is_null() || extension.is_undefined() { return WebGLShaderPixelLocalStorageEnableResult::ExtensionUnavailable; }
        if !Self::call_extension(&extension, "isCoherent", &[]).as_bool().unwrap_or(false) {
            return WebGLShaderPixelLocalStorageEnableResult::NonCoherent;
        }
        let function = Reflect::get(&extension, &JsValue::from_str("framebufferTexturePixelLocalStorageWEBGL")).expect("query WebGL PLS function");
        let arity = Reflect::get(&function, &JsValue::from_str("length")).ok().and_then(|value| value.as_f64()).unwrap_or(0.0) as u32;
        if arity != 5 {
            console_call("warn", warning);
            return WebGLShaderPixelLocalStorageEnableResult::DeprecatedVersion;
        }
        self.pixel_local_storage = Some(extension);
        WebGLShaderPixelLocalStorageEnableResult::Enabled
    }

    fn enableWebGLProvokingVertex(&mut self) -> bool {
        let extension = self.call("getExtension", &[JsValue::from_str("WEBGL_provoking_vertex")]);
        if extension.is_null() || extension.is_undefined() { false } else {
            self.provoking_vertex = Some(extension);
            true
        }
    }

    fn getFramebufferPixelLocalStorageParameter(&mut self, plane: GLint, parameter: GLenum) -> GLint {
        self.pixel_local_storage.as_ref().map(|extension| Self::call_extension(extension, "getFramebufferPixelLocalStorageParameterWEBGL", &[number(plane), number(parameter)]).as_f64().unwrap_or(0.0) as GLint).unwrap_or(0)
    }

    fn isObject(&mut self, kind: GLObjectKind, name: GLuint) -> bool {
        let method = match kind {
            GLObjectKind::Buffer => "isBuffer", GLObjectKind::Framebuffer => "isFramebuffer",
            GLObjectKind::Program => "isProgram", GLObjectKind::Renderbuffer => "isRenderbuffer",
            GLObjectKind::Sampler => "isSampler", GLObjectKind::Texture => "isTexture",
            GLObjectKind::VertexArray => "isVertexArray",
        };
        self.call(method, &[self.object(kind, name)]).as_bool().unwrap_or(false)
    }

    fn checkFramebufferStatus(&mut self, target: GLenum) -> GLenum { self.call("checkFramebufferStatus", &[number(target)]).as_f64().unwrap_or(0.0) as GLenum }
    fn shaderParameter(&mut self, shader: GLuint, parameter: GLenum) -> GLint { js_integer(self.call("getShaderParameter", &[self.shader(shader), number(parameter)])) }
    fn shaderInfoLog(&mut self, shader: GLuint, maxLength: usize) -> Vec<u8> { truncate_bytes(self.call("getShaderInfoLog", &[self.shader(shader)]).as_string().unwrap_or_default().into_bytes(), maxLength) }
    fn programParameter(&mut self, program: GLuint, parameter: GLenum) -> GLint { js_integer(self.call("getProgramParameter", &[self.program(program), number(parameter)])) }
    fn programInfoLog(&mut self, program: GLuint, maxLength: usize) -> Vec<u8> { truncate_bytes(self.call("getProgramInfoLog", &[self.program(program)]).as_string().unwrap_or_default().into_bytes(), maxLength) }
    fn uniformBlockIndex(&mut self, program: GLuint, name: &[u8]) -> GLuint { self.call("getUniformBlockIndex", &[self.program(program), JsValue::from_str(bytes_string(name))]).as_f64().unwrap_or(f64::from(u32::MAX)) as GLuint }

    fn uniformLocation(&mut self, program: GLuint, name: &[u8]) -> GLint {
        let location = self.call("getUniformLocation", &[self.program(program), JsValue::from_str(bytes_string(name))]);
        if location.is_null() || location.is_undefined() { -1 } else { self.names.uniform_locations.insert(location) as GLint }
    }

    fn readPixelsRGBA8(&mut self, x: i32, y: i32, width: u32, height: u32) -> Vec<u8> {
        let len = usize::try_from(u64::from(width) * u64::from(height) * 4).expect("WebGL readback size overflow");
        let pixels = Uint8Array::new_with_length(len as u32);
        self.call("readPixels", &[number(x), number(y), number(width), number(height), number(GL_RGBA), number(GL_UNSIGNED_BYTE), pixels.clone().into()]);
        let mut result = vec![0; len];
        pixels.copy_to(&mut result);
        result
    }

    fn finishAndGetError(&mut self) -> GLenum {
        self.call("finish", &[]);
        self.call("getError", &[]).as_f64().unwrap_or(0.0) as GLenum
    }

    fn contextLost(&mut self, _nextGeneration: u64) {
        self.names.clear();
        self.extensions.clear();
        self.pixel_local_storage = None;
        self.provoking_vertex = None;
    }
}

impl Drop for BrowserWebGl2Provider {
    fn drop(&mut self) {
        if let Some(listener) = self.context_lost_listener.as_ref() {
            let _ = self.canvas.remove_event_listener_with_callback("webglcontextlost", listener.as_ref().unchecked_ref());
        }
        if let Some(listener) = self.context_restored_listener.as_ref() {
            let _ = self.canvas.remove_event_listener_with_callback("webglcontextrestored", listener.as_ref().unchecked_ref());
        }
        if let Some(id) = self.wake_id.take() {
            FINAL_RELEASE_INGRESSES.with(|ingresses| { ingresses.borrow_mut().remove(&id); });
        }
    }
}

fn invoke(receiver: &JsValue, method: &str, args: &[JsValue]) -> JsValue {
    let function: Function = Reflect::get(receiver, &JsValue::from_str(method))
        .unwrap_or_else(|_| panic!("WebGL method `{method}` is unavailable"))
        .dyn_into()
        .unwrap_or_else(|_| panic!("WebGL property `{method}` is not callable"));
    let arguments = Array::new();
    for argument in args { arguments.push(argument); }
    function.apply(receiver, &arguments).unwrap_or_else(|error| {
        panic!("WebGL method `{method}` threw: {:?}", error.as_string())
    })
}

trait JsNumber { fn to_f64(self) -> f64; }
macro_rules! impl_js_number {
    ($($ty:ty),* $(,)?) => { $(impl JsNumber for $ty { fn to_f64(self) -> f64 { self as f64 } })* };
}
impl_js_number!(u8, u32, u64, usize, i32, i64, f32, f64);
fn number(value: impl JsNumber) -> JsValue { JsValue::from_f64(value.to_f64()) }
fn bytes_string(bytes: &[u8]) -> &str { std::str::from_utf8(bytes).unwrap_or("").trim_end_matches('\0') }
fn js_integer(value: JsValue) -> GLint { value.as_bool().map_or_else(|| value.as_f64().unwrap_or(0.0) as GLint, i32::from) }
fn truncate_bytes(mut bytes: Vec<u8>, max_length: usize) -> Vec<u8> { bytes.truncate(max_length); bytes }

// Exact port of Emscripten 3.1.61's `heapObjectForWebGLType`, which owns the
// GLES-pointer to WebGL-typed-array boundary used by the pinned C++ oracle.
fn texture_pixel_data(type_: GLenum, data: &[u8]) -> JsValue {
    let bytes = Uint8Array::from(data);
    let buffer = bytes.buffer();
    match type_ {
        0x1400 /* GL_BYTE */ => Int8Array::new(&buffer).into(),
        0x1401 /* GL_UNSIGNED_BYTE */ => bytes.into(),
        0x1402 /* GL_SHORT */ => {
            assert_eq!(data.len() % 2, 0, "GL_SHORT texture data is misaligned");
            Int16Array::new(&buffer).into()
        }
        0x1404 /* GL_INT */ => {
            assert_eq!(data.len() % 4, 0, "GL_INT texture data is misaligned");
            Int32Array::new(&buffer).into()
        }
        0x1406 /* GL_FLOAT */ => {
            assert_eq!(data.len() % 4, 0, "GL_FLOAT texture data is misaligned");
            Float32Array::new(&buffer).into()
        }
        0x1405 /* GL_UNSIGNED_INT */
        | 0x84FA /* GL_UNSIGNED_INT_24_8 */
        | 0x8368 /* GL_UNSIGNED_INT_2_10_10_10_REV */
        | 0x8C3B /* GL_UNSIGNED_INT_10F_11F_11F_REV */
        | 0x8C3E /* GL_UNSIGNED_INT_5_9_9_9_REV */ => {
            assert_eq!(data.len() % 4, 0, "GL_UNSIGNED_INT texture data is misaligned");
            Uint32Array::new(&buffer).into()
        }
        // Emscripten's authoritative bridge defaults the remaining accepted
        // GLES texture types (including GL_HALF_FLOAT) to HEAPU16.
        _ => {
            assert_eq!(data.len() % 2, 0, "GL_UNSIGNED_SHORT texture data is misaligned");
            Uint16Array::new(&buffer).into()
        }
    }
}

fn console_call(method: &str, message: &str) {
    let console = Reflect::get(&js_sys::global(), &JsValue::from_str("console")).unwrap_or(JsValue::UNDEFINED);
    if !console.is_undefined() { let _ = invoke(&console, method, &[JsValue::from_str(message)]); }
}
