//! Pure-Rust execution boundary for editor-authored GPU-canvas Luau.
//!
//! The classes installed here are Rust userdata, not a second Lua
//! implementation of the GPU contract. Luau owns authored control flow while
//! Rust owns buffer bounds, pipeline layout, binding identity, pass lifecycle,
//! and the typed renderer handoff.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use luaur_rt::{
    AnyUserData, Function, Table, UserData, UserDataFields, UserDataMethods, Value, VmState,
};
use nuxie_render_api::{Factory as RenderFactory, GpuCanvasPlan, GpuCanvasShader, RenderImage};
pub use nuxie_render_api::{
    GpuCanvasUniformBuffer, GpuCanvasVertexAttribute, GpuCanvasVertexBuffer, GpuCanvasVertexLayout,
};

use crate::vm::{Error, Result, ScriptVm};

/// Product GPU-canvas resource fences. These are deliberately below WebGPU's
/// portable minimum limits so malformed authored scripts fail in Rust before
/// reaching a backend allocation or validation path.
pub const MAX_GPU_CANVAS_DIMENSION: u32 = 2_048;
pub const MAX_GPU_CANVAS_SCRIPT_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_CPU_BUFFER_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_UNIFORM_BUFFER_BYTES: usize = 64 * 1024;
pub const MAX_VERTEX_BUFFER_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TOTAL_BUFFER_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_LUAU_VM_MEMORY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_LUAU_INTERRUPTS_PER_CALL: u32 = 50_000;
pub const MAX_GPU_CANVAS_DRAW_INVOCATIONS: u64 = 1_000_000;
pub const MAX_GPU_CANVAS_VERTEX_BUFFERS: usize = 8;
pub const MAX_GPU_CANVAS_VERTEX_ATTRIBUTES: usize = 16;
pub const MAX_GPU_CANVAS_BIND_GROUPS: u32 = 4;
pub const MAX_GPU_CANVAS_UNIFORM_BINDINGS_PER_GROUP: usize = 12;
pub const MAX_GPU_CANVAS_BINDING_INDEX: u32 = 255;

#[derive(Debug, Default)]
struct GpuCanvasResourceBudget {
    allocated_buffer_bytes: usize,
}

impl GpuCanvasResourceBudget {
    fn reserve(&mut self, bytes: usize) -> Result<()> {
        let total = self
            .allocated_buffer_bytes
            .checked_add(bytes)
            .ok_or_else(|| Error::runtime("GPU-canvas buffer budget overflow"))?;
        if total > MAX_TOTAL_BUFFER_BYTES {
            return Err(Error::runtime(format!(
                "GPU-canvas scripts may allocate at most {MAX_TOTAL_BUFFER_BYTES} buffer bytes"
            )));
        }
        self.allocated_buffer_bytes = total;
        Ok(())
    }
}

/// Backwards-compatible name for the backend-neutral plan now owned by the
/// renderer API seam.
pub type GpuCanvasDrawPlan = GpuCanvasPlan;

#[derive(Debug, Clone)]
struct CpuBuffer {
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl CpuBuffer {
    fn new(size: usize) -> Self {
        Self {
            bytes: Rc::new(RefCell::new(vec![0; size])),
        }
    }
}

impl UserData for CpuBuffer {}

#[derive(Debug, Clone)]
struct GpuBuffer {
    usage: GpuBufferUsage,
    bytes: Rc<RefCell<Vec<u8>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GpuBufferUsage {
    Uniform,
    Vertex,
}

impl UserData for GpuBuffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "write",
            |_, this, (source, offset): (AnyUserData, Option<usize>)| {
                if this.usage != GpuBufferUsage::Uniform {
                    return Err(Error::runtime(
                        "GPUBuffer:write is supported only for uniform buffers",
                    ));
                }
                let source = source.borrow::<CpuBuffer>()?;
                let source = source.bytes.borrow();
                let offset = offset.unwrap_or(0);
                let end = offset
                    .checked_add(source.len())
                    .ok_or_else(|| Error::runtime("GPUBuffer:write byte range overflow"))?;
                let mut destination = this.bytes.borrow_mut();
                if end > destination.len() {
                    return Err(Error::runtime(format!(
                        "GPUBuffer:write range {offset}..{end} exceeds {} bytes",
                        destination.len()
                    )));
                }
                destination[offset..end].copy_from_slice(&source);
                Ok(())
            },
        );
    }
}

#[derive(Debug, Clone)]
struct GpuShader {
    name: String,
}

impl UserData for GpuShader {}

#[derive(Debug, Clone)]
struct GpuPipeline {
    shader_name: String,
    vertex_layouts: Vec<GpuCanvasVertexLayout>,
}

impl UserData for GpuPipeline {}

#[derive(Debug, Clone)]
struct GpuBindGroupLayout {
    group: u32,
}

impl UserData for GpuBindGroupLayout {}

#[derive(Debug, Clone)]
struct GpuUniformBinding {
    binding: u32,
    buffer: GpuBuffer,
}

#[derive(Debug, Clone)]
struct GpuBindGroup {
    group: u32,
    uniforms: Vec<GpuUniformBinding>,
}

impl UserData for GpuBindGroup {}

#[derive(Debug)]
struct CompletedGpuCanvasPass {
    shader_name: String,
    plan: GpuCanvasDrawPlan,
}

#[derive(Default)]
struct GpuCanvasState {
    width: u32,
    height: u32,
    completed: Option<CompletedGpuCanvasPass>,
    image: Option<Box<dyn RenderImage>>,
}

impl std::fmt::Debug for GpuCanvasState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GpuCanvasState")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("completed", &self.completed)
            .field("has_image", &self.image.is_some())
            .finish()
    }
}

#[derive(Debug, Clone)]
struct GpuCanvas {
    state: Rc<RefCell<GpuCanvasState>>,
}

impl UserData for GpuCanvas {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("image", |lua, this| {
            if this.state.borrow().image.is_none() {
                return Ok(Value::Nil);
            }
            lua.create_userdata(GpuCanvasImage {
                state: Rc::clone(&this.state),
            })
            .map(Value::UserData)
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("resize", |_, this, (width, height): (u32, u32)| {
            if width == 0
                || height == 0
                || width > MAX_GPU_CANVAS_DIMENSION
                || height > MAX_GPU_CANVAS_DIMENSION
            {
                return Err(Error::runtime(format!(
                    "GPUCanvas:resize dimensions must be between 1 and {MAX_GPU_CANVAS_DIMENSION}"
                )));
            }
            let mut state = this.state.borrow_mut();
            state.width = width;
            state.height = height;
            Ok(())
        });
        methods.add_method("beginRenderPass", |lua, this, descriptor: Table| {
            reject_unknown_fields(&descriptor, &["color"], "GPU render-pass descriptor")?;
            let colors: Table = descriptor.get("color")?;
            if colors.raw_len() != 1 {
                return Err(Error::runtime(
                    "GPU-canvas product snapshots require exactly one color attachment",
                ));
            }
            let color: Table = colors.get(1)?;
            reject_unknown_fields(
                &color,
                &["loadOp", "storeOp", "clearColor"],
                "GPU color attachment",
            )?;
            let load_op: String = color.get("loadOp")?;
            let store_op: String = color.get("storeOp")?;
            if load_op != "clear" || store_op != "store" {
                return Err(Error::runtime(
                    "GPUCanvas render pass requires loadOp='clear' and storeOp='store'",
                ));
            }
            let clear: Table = color.get("clearColor")?;
            if clear.raw_len() != 4 {
                return Err(Error::runtime(
                    "GPU clearColor must contain exactly four components",
                ));
            }
            lua.create_userdata(GpuRenderPass {
                state: Rc::clone(&this.state),
                clear_color: [clear.get(1)?, clear.get(2)?, clear.get(3)?, clear.get(4)?],
                pipeline: None,
                bind_groups: BTreeMap::new(),
                vertex_buffers: BTreeMap::new(),
                draw: None,
                finished: false,
            })
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GpuCanvasImage {
    state: Rc<RefCell<GpuCanvasState>>,
}

impl UserData for GpuCanvasImage {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScriptedImageSampler(pub(crate) nuxie_render_api::ImageSampler);

impl UserData for ScriptedImageSampler {}

#[derive(Debug, Clone)]
pub(crate) struct GpuCanvasContextBindings {
    canvas: GpuCanvas,
    shader_names: Rc<BTreeSet<String>>,
}

impl GpuCanvasContextBindings {
    pub(crate) fn canvas_userdata(&self, lua: &luaur_rt::Lua) -> Result<AnyUserData> {
        lua.create_userdata(self.canvas.clone())
    }

    pub(crate) fn shader_userdata(&self, lua: &luaur_rt::Lua, name: String) -> Result<AnyUserData> {
        if !self.shader_names.contains(&name) {
            return Err(Error::runtime(format!(
                "GPU-canvas shader '{name}' is unavailable"
            )));
        }
        lua.create_userdata(GpuShader { name })
    }
}

impl UserData for GpuCanvasContextBindings {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("gpuCanvas", |lua, this, ()| this.canvas_userdata(lua));
        methods.add_method("shader", |lua, this, name: String| {
            this.shader_userdata(lua, name)
        });
    }
}

/// Per-script retained GPU-canvas state. The Lua context and image userdata
/// share this state, while canonical shader bytes remain VM-owned.
pub(crate) struct ImportedGpuCanvasInstance {
    state: Rc<RefCell<GpuCanvasState>>,
    shaders: Rc<RefCell<BTreeMap<String, GpuCanvasShader>>>,
}

impl ImportedGpuCanvasInstance {
    pub(crate) fn new(
        shaders: Rc<RefCell<BTreeMap<String, GpuCanvasShader>>>,
    ) -> (Self, GpuCanvasContextBindings) {
        let state = Rc::new(RefCell::new(GpuCanvasState::default()));
        let shader_names = Rc::new(shaders.borrow().keys().cloned().collect());
        let bindings = GpuCanvasContextBindings {
            canvas: GpuCanvas {
                state: Rc::clone(&state),
            },
            shader_names,
        };
        (Self { state, shaders }, bindings)
    }

    pub(crate) fn execute_draw_canvas(
        &self,
        table: &Table,
        factory: &mut dyn RenderFactory,
    ) -> Result<()> {
        let value: Value = table.get("drawCanvas")?;
        let Value::Function(function) = value else {
            return Ok(());
        };
        {
            let mut state = self.state.borrow_mut();
            state.completed = None;
            state.image = None;
        }
        function.call::<()>((table.clone(),))?;
        let completed =
            self.state.borrow_mut().completed.take().ok_or_else(|| {
                Error::runtime("gpu-canvas drawCanvas did not finish a render pass")
            })?;
        let shaders = self.shaders.borrow();
        let shader = shaders.get(&completed.shader_name).ok_or_else(|| {
            Error::runtime(format!(
                "GPU-canvas shader '{}' is unavailable",
                completed.shader_name
            ))
        })?;
        let image = factory
            .make_gpu_canvas_image(shader, &completed.plan)
            .map_err(|error| Error::runtime(format!("GPU-canvas render failed: {error}")))?;
        drop(shaders);
        self.state.borrow_mut().image = Some(image);
        Ok(())
    }
}

pub(crate) fn with_gpu_canvas_image<R>(
    image: &GpuCanvasImage,
    callback: impl FnOnce(&dyn RenderImage) -> R,
) -> Result<R> {
    let state = image.state.borrow();
    let image = state
        .image
        .as_deref()
        .ok_or_else(|| Error::runtime("GPU-canvas image is unavailable before drawCanvas"))?;
    Ok(callback(image))
}

#[derive(Debug, Clone, Copy)]
struct GpuDrawCall {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

#[derive(Debug)]
struct GpuRenderPass {
    state: Rc<RefCell<GpuCanvasState>>,
    clear_color: [f64; 4],
    pipeline: Option<GpuPipeline>,
    bind_groups: BTreeMap<u32, GpuBindGroup>,
    vertex_buffers: BTreeMap<u32, GpuBuffer>,
    draw: Option<GpuDrawCall>,
    finished: bool,
}

impl UserData for GpuRenderPass {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("setPipeline", |_, this, pipeline: AnyUserData| {
            this.ensure_open()?;
            if this.pipeline.is_some() {
                return Err(Error::runtime(
                    "GPU render pass pipeline has already been set",
                ));
            }
            this.pipeline = Some(pipeline.borrow::<GpuPipeline>()?.clone());
            Ok(())
        });
        methods.add_method_mut(
            "setBindGroup",
            |_, this, (index, bind_group): (u32, AnyUserData)| {
                this.ensure_open()?;
                if this.draw.is_some() {
                    return Err(Error::runtime(
                        "GPU render pass supports exactly one draw call",
                    ));
                }
                if index >= MAX_GPU_CANVAS_BIND_GROUPS {
                    return Err(Error::runtime(format!(
                        "setBindGroup index must be less than {MAX_GPU_CANVAS_BIND_GROUPS}"
                    )));
                }
                if this.bind_groups.contains_key(&index) {
                    return Err(Error::runtime(format!(
                        "setBindGroup index {index} is already bound"
                    )));
                }
                let bind_group = bind_group.borrow::<GpuBindGroup>()?.clone();
                if index != bind_group.group {
                    return Err(Error::runtime(format!(
                        "setBindGroup index {index} does not match layout group {}",
                        bind_group.group
                    )));
                }
                this.bind_groups.insert(index, bind_group);
                Ok(())
            },
        );
        methods.add_method_mut(
            "setVertexBuffer",
            |_, this, (slot, buffer): (u32, AnyUserData)| {
                this.ensure_open()?;
                if usize::try_from(slot).unwrap_or(usize::MAX) >= MAX_GPU_CANVAS_VERTEX_BUFFERS {
                    return Err(Error::runtime(format!(
                        "setVertexBuffer slot must be less than {MAX_GPU_CANVAS_VERTEX_BUFFERS}"
                    )));
                }
                if this.vertex_buffers.contains_key(&slot) {
                    return Err(Error::runtime(format!(
                        "setVertexBuffer slot {slot} is already bound"
                    )));
                }
                let buffer = buffer.borrow::<GpuBuffer>()?.clone();
                if buffer.usage != GpuBufferUsage::Vertex {
                    return Err(Error::runtime(
                        "setVertexBuffer requires a vertex GPUBuffer",
                    ));
                }
                this.vertex_buffers.insert(slot, buffer);
                Ok(())
            },
        );
        methods.add_method_mut(
            "draw",
            |_,
             this,
             (vertex_count, instance_count, first_vertex, first_instance): (
                u32,
                Option<u32>,
                Option<u32>,
                Option<u32>,
            )| {
                this.ensure_open()?;
                let instance_count = instance_count.unwrap_or(1);
                let first_vertex = first_vertex.unwrap_or(0);
                let first_instance = first_instance.unwrap_or(0);
                let vertex_end = first_vertex
                    .checked_add(vertex_count)
                    .ok_or_else(|| Error::runtime("GPU draw vertex range overflow"))?;
                let instance_end = first_instance
                    .checked_add(instance_count)
                    .ok_or_else(|| Error::runtime("GPU draw instance range overflow"))?;
                let invocation_count = u64::from(vertex_count)
                    .checked_mul(u64::from(instance_count))
                    .ok_or_else(|| Error::runtime("GPU draw invocation count overflow"))?;
                if vertex_count == 0 || instance_count == 0 {
                    return Err(Error::runtime(
                        "GPU render pass vertex and instance counts must be positive",
                    ));
                }
                if invocation_count > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                    || u64::from(vertex_end) > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                    || u64::from(instance_end) > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                {
                    return Err(Error::runtime(format!(
                        "GPU render pass draw ranges may cover at most {MAX_GPU_CANVAS_DRAW_INVOCATIONS} invocations"
                    )));
                }
                this.draw = Some(GpuDrawCall {
                    vertex_count,
                    instance_count,
                    first_vertex,
                    first_instance,
                });
                Ok(())
            },
        );
        methods.add_method_mut("finish", |_, this, ()| {
            this.ensure_open()?;
            let pipeline = this.pipeline.as_ref().ok_or_else(|| {
                Error::runtime("GPU render pass must set a pipeline before finish")
            })?;
            let draw = this
                .draw
                .ok_or_else(|| Error::runtime("GPU render pass must issue a draw before finish"))?;
            if pipeline.vertex_layouts.len() != this.vertex_buffers.len() {
                return Err(Error::runtime(format!(
                    "GPU render pass has {} vertex layouts but {} bound vertex buffers",
                    pipeline.vertex_layouts.len(),
                    this.vertex_buffers.len()
                )));
            }
            for (slot, layout) in pipeline.vertex_layouts.iter().enumerate() {
                let slot = u32::try_from(slot)
                    .map_err(|_| Error::runtime("GPU vertex slot conversion overflow"))?;
                let buffer = this.vertex_buffers.get(&slot).ok_or_else(|| {
                    Error::runtime(format!("GPU vertex buffer slot {slot} is not bound"))
                })?;
                let vertex_end = u64::from(draw.first_vertex)
                    .checked_add(u64::from(draw.vertex_count))
                    .ok_or_else(|| Error::runtime("GPU vertex byte range overflow"))?;
                let required_bytes = vertex_end
                    .checked_mul(layout.stride)
                    .ok_or_else(|| Error::runtime("GPU vertex byte range overflow"))?;
                if required_bytes > buffer.bytes.borrow().len() as u64 {
                    return Err(Error::runtime(format!(
                        "GPU vertex buffer slot {slot} requires {required_bytes} bytes"
                    )));
                }
            }
            let state = this.state.borrow();
            if state.width == 0 || state.height == 0 {
                return Err(Error::runtime(
                    "GPU canvas must be resized before its first render pass",
                ));
            }
            if state.completed.is_some() {
                return Err(Error::runtime(
                    "gpu-canvas drawCanvas supports exactly one completed render pass",
                ));
            }
            let mut uniform_buffers = Vec::new();
            for bind_group in this.bind_groups.values() {
                for uniform in &bind_group.uniforms {
                    uniform_buffers.push(GpuCanvasUniformBuffer {
                        group: bind_group.group,
                        binding: uniform.binding,
                        bytes: uniform.buffer.bytes.borrow().clone(),
                    });
                }
            }
            let vertex_buffers = this
                .vertex_buffers
                .iter()
                .map(|(slot, buffer)| GpuCanvasVertexBuffer {
                    slot: *slot,
                    bytes: buffer.bytes.borrow().clone(),
                })
                .collect();
            let plan = GpuCanvasDrawPlan {
                width: state.width,
                height: state.height,
                clear_color: this.clear_color,
                vertex_count: draw.vertex_count,
                instance_count: draw.instance_count,
                first_vertex: draw.first_vertex,
                first_instance: draw.first_instance,
                uniform_buffers,
                vertex_layouts: pipeline.vertex_layouts.clone(),
                vertex_buffers,
            };
            drop(state);
            this.state.borrow_mut().completed = Some(CompletedGpuCanvasPass {
                shader_name: pipeline.shader_name.clone(),
                plan,
            });
            this.finished = true;
            Ok(())
        });
    }
}

impl GpuRenderPass {
    fn ensure_open(&self) -> Result<()> {
        if self.finished {
            Err(Error::runtime("GPU render pass has already finished"))
        } else {
            Ok(())
        }
    }
}

/// Retained pure-Rust Luau program used for deterministic temporal sampling.
pub struct GpuCanvasProgram {
    vm: ScriptVm,
    instance: Table,
    state: Rc<RefCell<GpuCanvasState>>,
    execution_budget: Rc<Cell<u32>>,
}

impl std::fmt::Debug for GpuCanvasProgram {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GpuCanvasProgram")
            .finish_non_exhaustive()
    }
}

impl GpuCanvasProgram {
    /// Compile source, run its protocol generator, and retain the returned
    /// script instance. Syntax and generator-shape failures are returned with
    /// the VM diagnostic intact.
    pub fn compile(source: &str) -> Result<Self> {
        if source.len() > MAX_GPU_CANVAS_SCRIPT_SOURCE_BYTES {
            return Err(Error::runtime(format!(
                "GPU-canvas Luau source exceeds {MAX_GPU_CANVAS_SCRIPT_SOURCE_BYTES} bytes"
            )));
        }
        let vm = ScriptVm::new();
        vm.lua().set_memory_limit(MAX_LUAU_VM_MEMORY_BYTES)?;
        let execution_budget = Rc::new(Cell::new(MAX_LUAU_INTERRUPTS_PER_CALL));
        let interrupt_budget = Rc::clone(&execution_budget);
        vm.lua().set_interrupt(move |_| {
            let remaining = interrupt_budget.get();
            if remaining == 0 {
                return Err(Error::runtime(format!(
                    "GPU-canvas Luau execution exceeded {MAX_LUAU_INTERRUPTS_PER_CALL} interrupt safepoints"
                )));
            }
            interrupt_budget.set(remaining - 1);
            Ok(VmState::Continue)
        });
        let resource_budget = Rc::new(RefCell::new(GpuCanvasResourceBudget::default()));
        install_gpu_canvas_globals_with_budget(&vm, resource_budget)?;
        let state = Rc::new(RefCell::new(GpuCanvasState::default()));
        let bindings = GpuCanvasContextBindings {
            canvas: GpuCanvas {
                state: Rc::clone(&state),
            },
            shader_names: Rc::new(BTreeSet::from(["scene".into()])),
        };
        let context = vm.lua().create_userdata(bindings)?;
        let chunk = vm
            .load("editor-gpu-canvas", source)
            .map_err(|error| Error::runtime(format!("gpu-canvas Luau syntax error: {error}")))?;
        execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        let generator: Function = chunk.call(()).map_err(|error| {
            Error::runtime(format!(
                "gpu-canvas script must return a generator: {error}"
            ))
        })?;
        execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        let instance: Table = generator
            .call(context)
            .map_err(|error| Error::runtime(format!("gpu-canvas generator failed: {error}")))?;
        Ok(Self {
            vm,
            instance,
            state,
            execution_budget,
        })
    }

    /// Advance the retained script by an exact fixed-step delta. A missing
    /// `advance` method represents a static scene and is accepted.
    pub fn advance(&mut self, elapsed_seconds: f64) -> Result<bool> {
        let method: Value = self.instance.get("advance")?;
        let Value::Function(method) = method else {
            return Ok(false);
        };
        self.execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        let value: Value = method.call((self.instance.clone(), elapsed_seconds))?;
        Ok(matches!(value, Value::Boolean(true)))
    }

    /// Execute `drawCanvas` and return the exact Rust-owned completed pass.
    pub fn draw(&mut self) -> Result<GpuCanvasDrawPlan> {
        self.state.borrow_mut().completed = None;
        let method: Value = self.instance.get("drawCanvas")?;
        let Value::Function(method) = method else {
            return Err(Error::runtime(
                "gpu-canvas script instance has no drawCanvas function",
            ));
        };
        self.execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        method.call::<()>((self.instance.clone(),))?;
        self.state
            .borrow_mut()
            .completed
            .take()
            .map(|completed| completed.plan)
            .ok_or_else(|| Error::runtime("gpu-canvas drawCanvas did not finish a render pass"))
    }

    pub fn vm(&self) -> &ScriptVm {
        &self.vm
    }
}

pub(crate) fn install_gpu_canvas_globals(vm: &ScriptVm) -> Result<()> {
    install_gpu_canvas_globals_with_budget(
        vm,
        Rc::new(RefCell::new(GpuCanvasResourceBudget::default())),
    )
}

fn install_gpu_canvas_globals_with_budget(
    vm: &ScriptVm,
    resource_budget: Rc<RefCell<GpuCanvasResourceBudget>>,
) -> Result<()> {
    let lua = vm.lua();
    let buffer = lua.create_table();
    let cpu_buffer_budget = Rc::clone(&resource_budget);
    buffer.set(
        "create",
        lua.create_function(move |lua, size: usize| {
            if size == 0 || size > MAX_CPU_BUFFER_BYTES {
                return Err(Error::runtime(format!(
                    "buffer.create size must be between 1 and {MAX_CPU_BUFFER_BYTES} bytes"
                )));
            }
            cpu_buffer_budget.borrow_mut().reserve(size)?;
            lua.create_userdata(CpuBuffer::new(size))
        })?,
    )?;
    buffer.set(
        "writef32",
        lua.create_function(|_, (target, offset, value): (AnyUserData, usize, f64)| {
            let target = target.borrow::<CpuBuffer>()?;
            let end = offset
                .checked_add(4)
                .ok_or_else(|| Error::runtime("buffer.writef32 byte range overflow"))?;
            let mut bytes = target.bytes.borrow_mut();
            if offset % 4 != 0 || end > bytes.len() {
                return Err(Error::runtime(format!(
                    "buffer.writef32 offset {offset} is outside {} bytes",
                    bytes.len()
                )));
            }
            bytes[offset..end].copy_from_slice(&(value as f32).to_le_bytes());
            Ok(())
        })?,
    )?;
    lua.globals().set("buffer", buffer)?;

    let gpu_buffer_budget = Rc::clone(&resource_budget);
    install_constructor(lua, "GPUBuffer", move |lua, descriptor| {
        reject_unknown_fields(&descriptor, &["size", "usage", "data"], "GPUBuffer")?;
        let size: usize = descriptor.get("size")?;
        let usage = match descriptor.get::<String>("usage")?.as_str() {
            "uniform" => GpuBufferUsage::Uniform,
            "vertex" => GpuBufferUsage::Vertex,
            usage => {
                return Err(Error::runtime(format!(
                    "unsupported GPUBuffer usage '{usage}'"
                )));
            }
        };
        let max_size = match usage {
            GpuBufferUsage::Uniform => MAX_UNIFORM_BUFFER_BYTES,
            GpuBufferUsage::Vertex => MAX_VERTEX_BUFFER_BYTES,
        };
        if size == 0 || size > max_size {
            return Err(Error::runtime(format!(
                "GPUBuffer {usage:?} size must be between 1 and {max_size} bytes"
            )));
        }
        let source: AnyUserData = descriptor.get("data")?;
        let source = source.borrow::<CpuBuffer>()?;
        let source = source.bytes.borrow();
        if size == 0 || source.len() != size {
            return Err(Error::runtime(format!(
                "GPUBuffer size {size} does not match {} source bytes",
                source.len()
            )));
        }
        gpu_buffer_budget.borrow_mut().reserve(size)?;
        lua.create_userdata(GpuBuffer {
            usage,
            bytes: Rc::new(RefCell::new(source.clone())),
        })
    })?;
    install_constructor(lua, "GPUPipeline", |lua, descriptor| {
        reject_unknown_fields(
            &descriptor,
            &["vertex", "fragment", "vertexLayout", "colorTargets"],
            "GPUPipeline",
        )?;
        let vertex: AnyUserData = descriptor.get("vertex")?;
        let fragment: AnyUserData = descriptor.get("fragment")?;
        let vertex = vertex.borrow::<GpuShader>()?;
        let fragment = fragment.borrow::<GpuShader>()?;
        if vertex.name != fragment.name {
            return Err(Error::runtime(
                "GPU pipeline vertex and fragment shaders must share one module",
            ));
        }
        let targets: Table = descriptor.get("colorTargets")?;
        if targets.raw_len() != 1 {
            return Err(Error::runtime(
                "GPU-canvas product snapshots require exactly one color target",
            ));
        }
        let target: Table = targets.get(1)?;
        reject_unknown_fields(&target, &["format"], "GPU color target")?;
        if target.get::<String>("format")? != "rgba8unorm" {
            return Err(Error::runtime(
                "GPU-canvas product snapshots require rgba8unorm",
            ));
        }
        lua.create_userdata(GpuPipeline {
            shader_name: vertex.name.clone(),
            vertex_layouts: decode_vertex_layouts(&descriptor)?,
        })
    })?;
    install_constructor(lua, "GPUBindGroupLayout", |lua, descriptor| {
        reject_unknown_fields(&descriptor, &["groupIndex", "shader"], "GPUBindGroupLayout")?;
        let shader: AnyUserData = descriptor.get("shader")?;
        let _shader = shader.borrow::<GpuShader>()?;
        let group: u32 = descriptor.get("groupIndex")?;
        if group >= MAX_GPU_CANVAS_BIND_GROUPS {
            return Err(Error::runtime(format!(
                "GPUBindGroupLayout groupIndex must be less than {MAX_GPU_CANVAS_BIND_GROUPS}"
            )));
        }
        lua.create_userdata(GpuBindGroupLayout { group })
    })?;
    install_constructor(lua, "GPUBindGroup", |lua, descriptor| {
        reject_unknown_fields(&descriptor, &["layout", "ubos"], "GPUBindGroup")?;
        let layout: AnyUserData = descriptor.get("layout")?;
        let layout = layout.borrow::<GpuBindGroupLayout>()?;
        let ubos: Table = descriptor.get("ubos")?;
        let mut uniforms = Vec::new();
        let mut bindings = BTreeSet::new();
        for entry in ubos.sequence_values::<Table>() {
            if uniforms.len() >= MAX_GPU_CANVAS_UNIFORM_BINDINGS_PER_GROUP {
                return Err(Error::runtime(format!(
                    "GPUBindGroup supports at most {MAX_GPU_CANVAS_UNIFORM_BINDINGS_PER_GROUP} uniform bindings"
                )));
            }
            let entry = entry?;
            reject_unknown_fields(&entry, &["slot", "buffer"], "GPUBindGroup UBO")?;
            let binding: u32 = entry.get("slot")?;
            if binding > MAX_GPU_CANVAS_BINDING_INDEX {
                return Err(Error::runtime(format!(
                    "GPUBindGroup binding must be at most {MAX_GPU_CANVAS_BINDING_INDEX}"
                )));
            }
            if !bindings.insert(binding) {
                return Err(Error::runtime(format!(
                    "GPUBindGroup binding {binding} is duplicated"
                )));
            }
            let buffer: AnyUserData = entry.get("buffer")?;
            let buffer = buffer.borrow::<GpuBuffer>()?.clone();
            if buffer.usage != GpuBufferUsage::Uniform {
                return Err(Error::runtime(
                    "GPUBindGroup UBO entry requires a uniform GPUBuffer",
                ));
            }
            uniforms.push(GpuUniformBinding { binding, buffer });
        }
        lua.create_userdata(GpuBindGroup {
            group: layout.group,
            uniforms,
        })
    })?;
    lua.globals().set(
        "ImageSampler",
        lua.create_function(|lua, (wrap_x, wrap_y, filter): (String, String, String)| {
            use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap};
            let parse_wrap = |value: &str| match value {
                "clamp" => Ok(ImageWrap::Clamp),
                "repeat" => Ok(ImageWrap::Repeat),
                "mirror" => Ok(ImageWrap::Mirror),
                other => Err(Error::runtime(format!(
                    "unsupported image sampler wrap '{other}'"
                ))),
            };
            let filter = match filter.as_str() {
                "linear" => ImageFilter::Bilinear,
                "nearest" => ImageFilter::Nearest,
                other => {
                    return Err(Error::runtime(format!(
                        "unsupported image sampler filter '{other}'"
                    )));
                }
            };
            lua.create_userdata(ScriptedImageSampler(ImageSampler {
                wrap_x: parse_wrap(&wrap_x)?,
                wrap_y: parse_wrap(&wrap_y)?,
                filter,
            }))
        })?,
    )?;
    Ok(())
}

fn install_constructor(
    lua: &luaur_rt::Lua,
    name: &str,
    constructor: impl Fn(&luaur_rt::Lua, Table) -> Result<AnyUserData> + 'static,
) -> Result<()> {
    let table = lua.create_table();
    table.set("new", lua.create_function(constructor)?)?;
    lua.globals().set(name, table)
}

fn reject_unknown_fields(table: &Table, allowed: &[&str], label: &str) -> Result<()> {
    for pair in table.clone().pairs::<String, Value>() {
        let (key, _) = pair?;
        if !allowed.contains(&key.as_str()) {
            return Err(Error::runtime(format!(
                "{label} field '{key}' is unsupported"
            )));
        }
    }
    Ok(())
}

fn decode_vertex_layouts(descriptor: &Table) -> Result<Vec<GpuCanvasVertexLayout>> {
    let layouts: Table = descriptor.get("vertexLayout")?;
    let mut decoded = Vec::new();
    let mut shader_locations = BTreeSet::new();
    let mut attribute_count = 0;
    for layout in layouts.sequence_values::<Table>() {
        if decoded.len() >= MAX_GPU_CANVAS_VERTEX_BUFFERS {
            return Err(Error::runtime(format!(
                "GPU pipeline supports at most {MAX_GPU_CANVAS_VERTEX_BUFFERS} vertex layouts"
            )));
        }
        let layout = layout?;
        reject_unknown_fields(&layout, &["stride", "attributes"], "GPU vertex layout")?;
        let stride: u64 = layout.get("stride")?;
        if stride == 0 || stride > 2_048 {
            return Err(Error::runtime(
                "GPU vertex layout stride must be between 1 and 2048 bytes",
            ));
        }
        let source_attributes: Table = layout.get("attributes")?;
        let mut attributes = Vec::new();
        for attribute in source_attributes.sequence_values::<Table>() {
            attribute_count += 1;
            if attribute_count > MAX_GPU_CANVAS_VERTEX_ATTRIBUTES {
                return Err(Error::runtime(format!(
                    "GPU pipeline supports at most {MAX_GPU_CANVAS_VERTEX_ATTRIBUTES} vertex attributes"
                )));
            }
            let attribute = attribute?;
            reject_unknown_fields(
                &attribute,
                &["format", "slot", "offset"],
                "GPU vertex attribute",
            )?;
            let format: String = attribute.get("format")?;
            let format_size = match format.as_str() {
                "float32" => 4,
                "float32x2" => 8,
                "float32x3" => 12,
                "float32x4" => 16,
                _ => {
                    return Err(Error::runtime(format!(
                        "unsupported GPU vertex format '{format}'"
                    )));
                }
            };
            let shader_location: u32 = attribute.get("slot")?;
            if shader_location >= MAX_GPU_CANVAS_VERTEX_ATTRIBUTES as u32 {
                return Err(Error::runtime(format!(
                    "GPU vertex attribute slot must be less than {MAX_GPU_CANVAS_VERTEX_ATTRIBUTES}"
                )));
            }
            if !shader_locations.insert(shader_location) {
                return Err(Error::runtime(format!(
                    "GPU vertex attribute slot {shader_location} is duplicated"
                )));
            }
            let offset: u64 = attribute.get("offset")?;
            if offset
                .checked_add(format_size)
                .is_none_or(|end| end > stride)
            {
                return Err(Error::runtime(format!(
                    "GPU vertex attribute at offset {offset} exceeds stride {stride}"
                )));
            }
            attributes.push(GpuCanvasVertexAttribute {
                shader_location,
                offset,
                format,
            });
        }
        if attributes.is_empty() {
            return Err(Error::runtime(
                "GPU vertex layouts must contain at least one attribute",
            ));
        }
        decoded.push(GpuCanvasVertexLayout { stride, attributes });
    }
    Ok(decoded)
}
