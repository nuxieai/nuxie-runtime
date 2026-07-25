//! Arbitrary WGSL draw execution for editor GPU-canvas critique frames.
//!
//! Luau execution lives in `nuxie-scripting`; this module accepts only its
//! typed draw plan and owns shader modules, buffers, submission, and readback.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use nuxie_render_api::{
    GpuCanvasError, GpuCanvasPlan, GpuCanvasShader, GpuCanvasShaderEntry,
    GpuCanvasShaderEntrySelection, GpuCanvasShaderResourceKind, GpuCanvasShaderStage, RenderImage,
};
pub use nuxie_render_api::{
    GpuCanvasUniformBuffer, GpuCanvasVertexAttribute, GpuCanvasVertexBuffer, GpuCanvasVertexLayout,
};
use wgpu::util::DeviceExt;

use super::{align_to, map_buffer, RendererError, WgpuFactory, WgpuImage, WgpuImageTexture};

const MAX_GPU_CANVAS_DIMENSION: u32 = 2_048;
const MAX_UNIFORM_BUFFER_BYTES: usize = 64 * 1024;
const MAX_VERTEX_BUFFER_BYTES: usize = 16 * 1024 * 1024;
const MAX_DRAW_INVOCATIONS: u64 = 1_000_000;
const MAX_VERTEX_BUFFERS: usize = 8;
const MAX_VERTEX_ATTRIBUTES: usize = 16;
const MAX_BIND_GROUPS: u32 = 4;
const MAX_UNIFORM_BINDINGS_PER_GROUP: usize = 12;
const MAX_BINDING_INDEX: u32 = 255;
const MAX_IMPORTED_GPU_CANVAS_CACHE_ENTRIES: usize = 16;
// Retain at least one maximally valid public plan while bounding aggregate
// cached input buffers across distinct shader/layout keys.
const MAX_IMPORTED_GPU_CANVAS_CACHE_BYTES: usize = MAX_VERTEX_BUFFERS * MAX_VERTEX_BUFFER_BYTES
    + MAX_BIND_GROUPS as usize * MAX_UNIFORM_BINDINGS_PER_GROUP * MAX_UNIFORM_BUFFER_BYTES;
// GPU-canvas images are retained by script occurrences between frames. A
// per-canvas dimension limit alone would let one document pin an unbounded
// number of 2,048-square RGBA targets, so every factory lineage also owns one
// aggregate target budget.
const MAX_RETAINED_GPU_CANVAS_TARGETS: usize = 16;
const MAX_RETAINED_GPU_CANVAS_TARGET_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Default)]
pub(super) struct RetainedGpuCanvasTargetBudget {
    state: Mutex<RetainedGpuCanvasTargetBudgetState>,
}

#[derive(Debug, Default)]
struct RetainedGpuCanvasTargetBudgetState {
    targets: usize,
    bytes: usize,
}

impl RetainedGpuCanvasTargetBudget {
    pub(super) fn acquire(
        self: &Arc<Self>,
        width: u32,
        height: u32,
    ) -> Result<RetainedGpuCanvasTargetLease, GpuCanvasError> {
        let bytes = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| GpuCanvasError::new("GPU-canvas target byte length overflow"))?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let targets = state
            .targets
            .checked_add(1)
            .ok_or_else(|| GpuCanvasError::new("GPU-canvas target count overflow"))?;
        let retained_bytes = state
            .bytes
            .checked_add(bytes)
            .ok_or_else(|| GpuCanvasError::new("GPU-canvas target byte budget overflow"))?;
        if targets > MAX_RETAINED_GPU_CANVAS_TARGETS
            || retained_bytes > MAX_RETAINED_GPU_CANVAS_TARGET_BYTES
        {
            return Err(GpuCanvasError::new(format!(
                "retained GPU-canvas targets exceed the factory budget of {MAX_RETAINED_GPU_CANVAS_TARGETS} targets or {MAX_RETAINED_GPU_CANVAS_TARGET_BYTES} bytes"
            )));
        }
        state.targets = targets;
        state.bytes = retained_bytes;
        drop(state);
        Ok(RetainedGpuCanvasTargetLease {
            budget: Arc::clone(self),
            bytes,
        })
    }

    #[cfg(test)]
    fn retained(&self) -> (usize, usize) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (state.targets, state.bytes)
    }
}

#[derive(Debug)]
pub(super) struct RetainedGpuCanvasTargetLease {
    budget: Arc<RetainedGpuCanvasTargetBudget>,
    bytes: usize,
}

impl Drop for RetainedGpuCanvasTargetLease {
    fn drop(&mut self) {
        let mut state = self
            .budget
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.targets = state.targets.saturating_sub(1);
        state.bytes = state.bytes.saturating_sub(self.bytes);
    }
}

struct BudgetedGpuCanvasImage {
    image: Box<dyn RenderImage>,
    _target_lease: RetainedGpuCanvasTargetLease,
}

impl RenderImage for BudgetedGpuCanvasImage {
    fn as_any(&self) -> &dyn std::any::Any {
        self.image.as_any()
    }

    fn width(&self) -> u32 {
        self.image.width()
    }

    fn height(&self) -> u32 {
        self.image.height()
    }

    fn uv_transform(&self) -> nuxie_render_api::Mat2D {
        self.image.uv_transform()
    }
}

pub(super) fn retain_gpu_canvas_target(
    image: Box<dyn RenderImage>,
    lease: RetainedGpuCanvasTargetLease,
) -> Box<dyn RenderImage> {
    Box::new(BudgetedGpuCanvasImage {
        image,
        _target_lease: lease,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuCanvasRenderPlan {
    pub shader_wgsl: String,
    pub width: u32,
    pub height: u32,
    pub clear_color: [f64; 4],
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub uniform_buffers: Vec<GpuCanvasUniformBuffer>,
    pub vertex_layouts: Vec<GpuCanvasVertexLayout>,
    pub vertex_buffers: Vec<GpuCanvasVertexBuffer>,
}

struct PreparedImportedGpuCanvas {
    vertex_entry_point: String,
    fragment_entry_point: String,
    uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ImportedGpuCanvasPipelineKey {
    shader: GpuCanvasShader,
    vertex_entry: Option<GpuCanvasShaderEntrySelection>,
    fragment_entry: Option<GpuCanvasShaderEntrySelection>,
    uniform_bindings: Vec<(u32, u32, usize)>,
    vertex_layouts: Vec<GpuCanvasVertexLayout>,
    vertex_buffers: Vec<(u32, usize)>,
}

impl ImportedGpuCanvasPipelineKey {
    fn new(shader: &GpuCanvasShader, plan: &GpuCanvasPlan) -> Self {
        let uniform_bindings = plan
            .uniform_buffers
            .iter()
            .map(|buffer| (buffer.group, buffer.binding, buffer.bytes.len()))
            .collect::<Vec<_>>();
        Self {
            shader: shader.clone(),
            vertex_entry: plan.vertex_entry.clone(),
            fragment_entry: plan.fragment_entry.clone(),
            uniform_bindings,
            vertex_layouts: plan.vertex_layouts.clone(),
            vertex_buffers: plan
                .vertex_buffers
                .iter()
                .map(|buffer| (buffer.slot, buffer.bytes.len()))
                .collect(),
        }
    }

    fn buffer_bytes(&self) -> usize {
        self.uniform_bindings
            .iter()
            .map(|(_, _, bytes)| *bytes)
            .chain(self.vertex_buffers.iter().map(|(_, bytes)| *bytes))
            .fold(0, usize::saturating_add)
    }
}

pub(super) struct ImportedWgpuGpuCanvasCache {
    pipelines: Vec<ImportedWgpuGpuCanvasPipeline>,
    buffer_bytes: usize,
    pipeline_builds: u64,
}

impl Default for ImportedWgpuGpuCanvasCache {
    fn default() -> Self {
        Self {
            pipelines: Vec::new(),
            buffer_bytes: 0,
            pipeline_builds: 0,
        }
    }
}

impl ImportedWgpuGpuCanvasCache {
    fn insert(&mut self, pipeline: ImportedWgpuGpuCanvasPipeline) -> usize {
        let buffer_bytes = pipeline.key.buffer_bytes();
        while !self.pipelines.is_empty()
            && (self.pipelines.len() >= MAX_IMPORTED_GPU_CANVAS_CACHE_ENTRIES
                || self.buffer_bytes.saturating_add(buffer_bytes)
                    > MAX_IMPORTED_GPU_CANVAS_CACHE_BYTES)
        {
            let evicted = self.pipelines.remove(0);
            self.buffer_bytes = self.buffer_bytes.saturating_sub(evicted.key.buffer_bytes());
        }
        self.buffer_bytes = self.buffer_bytes.saturating_add(buffer_bytes);
        self.pipelines.push(pipeline);
        self.pipelines.len() - 1
    }
}

struct ImportedWgpuGpuCanvasPipeline {
    key: ImportedGpuCanvasPipelineKey,
    uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
    bind_groups: Vec<wgpu::BindGroup>,
    uniform_buffers: Vec<wgpu::Buffer>,
    vertex_buffers: Vec<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ImportedUniformRequirement {
    required_size: u32,
    stage_mask: u8,
}

impl ImportedUniformRequirement {
    fn visibility(self) -> wgpu::ShaderStages {
        let mut stages = wgpu::ShaderStages::empty();
        if self.stage_mask & (1 << GpuCanvasShaderStage::Vertex as u8) != 0 {
            stages |= wgpu::ShaderStages::VERTEX;
        }
        if self.stage_mask & (1 << GpuCanvasShaderStage::Fragment as u8) != 0 {
            stages |= wgpu::ShaderStages::FRAGMENT;
        }
        if self.stage_mask & (1 << GpuCanvasShaderStage::Compute as u8) != 0 {
            stages |= wgpu::ShaderStages::COMPUTE;
        }
        stages
    }
}

struct ParsedAuthoredWgsl {
    module: naga::Module,
    info: naga::valid::ModuleInfo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ImportedLocation {
    format: &'static str,
    interpolation: Option<naga::Interpolation>,
    sampling: Option<naga::Sampling>,
}

#[derive(Default)]
struct ImportedStageInterface {
    locations: BTreeMap<u32, ImportedLocation>,
    builtins: Vec<naga::BuiltIn>,
}

fn prepare_imported_gpu_canvas(
    shader: &GpuCanvasShader,
    plan: &GpuCanvasPlan,
) -> Result<PreparedImportedGpuCanvas, GpuCanvasError> {
    validate_imported_gpu_canvas_plan(plan)
        .map_err(|error| GpuCanvasError::new(error.to_string()))?;
    let parsed = parse_authored_wgsl(&shader.source)?;
    let vertex_record = resolve_imported_entry(
        shader,
        GpuCanvasShaderStage::Vertex,
        plan.vertex_entry.as_ref(),
        "vertex",
    )?;
    let fragment_record = resolve_imported_entry(
        shader,
        GpuCanvasShaderStage::Fragment,
        plan.fragment_entry.as_ref(),
        "fragment",
    )?;
    let uniform_requirements = validate_imported_interface(
        shader,
        plan,
        &parsed.module,
        &parsed.info,
        vertex_record,
        fragment_record,
    )?;
    Ok(PreparedImportedGpuCanvas {
        vertex_entry_point: vertex_record.physical_entry_point.clone(),
        fragment_entry_point: fragment_record.physical_entry_point.clone(),
        uniform_requirements,
    })
}

fn resolve_imported_entry<'a>(
    shader: &'a GpuCanvasShader,
    stage: GpuCanvasShaderStage,
    selection: Option<&GpuCanvasShaderEntrySelection>,
    stage_name: &str,
) -> Result<&'a GpuCanvasShaderEntry, GpuCanvasError> {
    let entry = match selection {
        Some(selection) => shader.entries.iter().find(|entry| {
            entry.stage == stage
                && entry.logical_entry_point == selection.logical_entry_point
                && entry.physical_entry_point == selection.physical_entry_point
        }),
        None => shader.entries.iter().find(|entry| entry.stage == stage),
    };
    entry.ok_or_else(|| {
        let requested = selection
            .map(|selection| {
                format!(
                    " logical '{}' / physical '{}'",
                    selection.logical_entry_point, selection.physical_entry_point
                )
            })
            .unwrap_or_default();
        GpuCanvasError::new(format!(
            "authored WGSL has no matching {stage_name}{requested} entry"
        ))
    })
}

fn validate_imported_interface(
    shader: &GpuCanvasShader,
    plan: &GpuCanvasPlan,
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
    vertex_record: &GpuCanvasShaderEntry,
    fragment_record: &GpuCanvasShaderEntry,
) -> Result<BTreeMap<(u32, u32), ImportedUniformRequirement>, GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let vertex_entry = imported_entry_point(
        module,
        naga::ShaderStage::Vertex,
        &vertex_record.physical_entry_point,
    )
    .ok_or_else(|| {
        invalid(format!(
            "vertex stage has no physical entry point '{}'",
            vertex_record.physical_entry_point
        ))
    })?;
    let fragment_entry = imported_entry_point(
        module,
        naga::ShaderStage::Fragment,
        &fragment_record.physical_entry_point,
    )
    .ok_or_else(|| {
        invalid(format!(
            "fragment stage has no physical entry point '{}'",
            fragment_record.physical_entry_point
        ))
    })?;

    let vertex_inputs = imported_function_inputs(module, &vertex_entry.function)?;
    let vertex_outputs = imported_function_output(module, &vertex_entry.function)?;
    let fragment_inputs = imported_function_inputs(module, &fragment_entry.function)?;
    let fragment_outputs = imported_function_output(module, &fragment_entry.function)?;

    if vertex_inputs.builtins.iter().any(|builtin| {
        !matches!(
            builtin,
            naga::BuiltIn::VertexIndex | naga::BuiltIn::InstanceIndex
        )
    }) {
        return Err(invalid(format!(
            "vertex inputs contain unsupported built-ins {:?}",
            vertex_inputs.builtins
        )));
    }
    if !vertex_outputs
        .builtins
        .iter()
        .any(|builtin| matches!(builtin, naga::BuiltIn::Position { .. }))
        || vertex_outputs
            .builtins
            .iter()
            .any(|builtin| !matches!(builtin, naga::BuiltIn::Position { .. }))
    {
        return Err(invalid(format!(
            "vertex output must contain only the position built-in plus user locations; found {:?}",
            vertex_outputs.builtins
        )));
    }
    if fragment_inputs.builtins.iter().any(|builtin| {
        !matches!(
            builtin,
            naga::BuiltIn::Position { .. } | naga::BuiltIn::FrontFacing
        )
    }) {
        return Err(invalid(format!(
            "fragment inputs contain unsupported built-ins {:?}",
            fragment_inputs.builtins
        )));
    }
    if !fragment_outputs.builtins.is_empty()
        || fragment_outputs.locations.len() != 1
        || fragment_outputs
            .locations
            .get(&0)
            .map(|output| output.format)
            != Some("float32x4")
    {
        return Err(invalid(
            "fragment output must be exactly one vec4<f32> at location 0".into(),
        ));
    }

    let planned_attributes = plan
        .vertex_layouts
        .iter()
        .flat_map(|layout| &layout.attributes)
        .map(|attribute| {
            imported_format(&attribute.format).map(|format| (attribute.shader_location, format))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let shader_attributes = vertex_inputs
        .locations
        .iter()
        .map(|(&location, interface)| (location, interface.format))
        .collect::<BTreeMap<_, _>>();
    if shader_attributes != planned_attributes {
        return Err(invalid(format!(
            "vertex inputs {shader_attributes:?} do not exactly match planned attributes {planned_attributes:?}"
        )));
    }

    for (&location, fragment_input) in &fragment_inputs.locations {
        let Some(vertex_output) = vertex_outputs.locations.get(&location) else {
            return Err(invalid(format!(
                "fragment input location {location} has no vertex output"
            )));
        };
        if vertex_output != fragment_input {
            return Err(invalid(format!(
                "inter-stage location {location} differs: vertex {vertex_output:?}, fragment {fragment_input:?}"
            )));
        }
    }

    let required_uniforms = imported_uniform_requirements(shader, module, info)?;
    let planned_uniforms = plan
        .uniform_buffers
        .iter()
        .map(|buffer| ((buffer.group, buffer.binding), buffer.bytes.len()))
        .collect::<BTreeMap<_, _>>();
    if required_uniforms.keys().ne(planned_uniforms.keys()) {
        return Err(invalid(format!(
            "shader uniform bindings {:?} do not exactly match planned bindings {:?}",
            required_uniforms.keys().collect::<Vec<_>>(),
            planned_uniforms.keys().collect::<Vec<_>>()
        )));
    }
    for (&binding, requirement) in &required_uniforms {
        let supplied_size = planned_uniforms[&binding];
        if supplied_size < requirement.required_size as usize {
            return Err(invalid(format!(
                "uniform group {} binding {} supplies {supplied_size} bytes but its layout requires {}",
                binding.0, binding.1, requirement.required_size
            )));
        }
    }
    Ok(required_uniforms)
}

fn imported_entry_point<'a>(
    module: &'a naga::Module,
    stage: naga::ShaderStage,
    name: &str,
) -> Option<&'a naga::EntryPoint> {
    module
        .entry_points
        .iter()
        .find(|entry| entry.stage == stage && entry.name == name)
}

fn imported_function_inputs(
    module: &naga::Module,
    function: &naga::Function,
) -> Result<ImportedStageInterface, GpuCanvasError> {
    let mut interface = ImportedStageInterface::default();
    for argument in &function.arguments {
        collect_imported_interface(
            module,
            argument.binding.as_ref(),
            argument.ty,
            &mut interface,
        )?;
    }
    Ok(interface)
}

fn imported_function_output(
    module: &naga::Module,
    function: &naga::Function,
) -> Result<ImportedStageInterface, GpuCanvasError> {
    let mut interface = ImportedStageInterface::default();
    if let Some(result) = &function.result {
        collect_imported_interface(module, result.binding.as_ref(), result.ty, &mut interface)?;
    }
    Ok(interface)
}

fn collect_imported_interface(
    module: &naga::Module,
    binding: Option<&naga::Binding>,
    ty: naga::Handle<naga::Type>,
    interface: &mut ImportedStageInterface,
) -> Result<(), GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    match binding {
        Some(naga::Binding::BuiltIn(builtin)) => interface.builtins.push(*builtin),
        Some(naga::Binding::Location {
            location,
            interpolation,
            sampling,
            blend_src,
            per_primitive,
        }) => {
            if blend_src.is_some() || *per_primitive {
                return Err(invalid(format!(
                    "location {location} uses dual-source or per-primitive IO"
                )));
            }
            let value = ImportedLocation {
                format: imported_naga_format(module, ty)?,
                interpolation: *interpolation,
                sampling: *sampling,
            };
            if interface.locations.insert(*location, value).is_some() {
                return Err(invalid(format!(
                    "location {location} appears more than once"
                )));
            }
        }
        None => match &module.types[ty].inner {
            naga::TypeInner::Struct { members, .. } => {
                for member in members {
                    collect_imported_interface(
                        module,
                        member.binding.as_ref(),
                        member.ty,
                        interface,
                    )?;
                }
            }
            _ => {
                return Err(invalid(
                    "entry-point IO value has neither a binding nor bound struct members".into(),
                ));
            }
        },
    }
    Ok(())
}

fn imported_naga_format(
    module: &naga::Module,
    ty: naga::Handle<naga::Type>,
) -> Result<&'static str, GpuCanvasError> {
    let scalar = |scalar: naga::Scalar, scalar_name, vector_names: [&'static str; 3]| {
        if scalar.kind != naga::ScalarKind::Float || scalar.width != 4 {
            return Err(GpuCanvasError::new(
                "invalid imported GPU-canvas interface: stage IO must use float32 scalars or vectors",
            ));
        }
        Ok((scalar_name, vector_names))
    };
    match module.types[ty].inner {
        naga::TypeInner::Scalar(value) => {
            scalar(value, "float32", ["float32x2", "float32x3", "float32x4"]).map(|(name, _)| name)
        }
        naga::TypeInner::Vector {
            size,
            scalar: value,
        } => {
            let (_, names) = scalar(value, "float32", ["float32x2", "float32x3", "float32x4"])?;
            Ok(match size {
                naga::VectorSize::Bi => names[0],
                naga::VectorSize::Tri => names[1],
                naga::VectorSize::Quad => names[2],
            })
        }
        _ => Err(GpuCanvasError::new(
            "invalid imported GPU-canvas interface: stage IO must use float32 scalars or vectors",
        )),
    }
}

fn imported_format(value: &str) -> Result<&'static str, GpuCanvasError> {
    match value {
        "float32" => Ok("float32"),
        "float32x2" => Ok("float32x2"),
        "float32x3" => Ok("float32x3"),
        "float32x4" => Ok("float32x4"),
        _ => Err(GpuCanvasError::new(format!(
            "invalid imported GPU-canvas interface: unsupported planned vertex format '{value}'"
        ))),
    }
}

fn imported_uniform_requirements(
    shader: &GpuCanvasShader,
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
) -> Result<BTreeMap<(u32, u32), ImportedUniformRequirement>, GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let mut layouter = naga::proc::Layouter::default();
    layouter
        .update(module.to_ctx())
        .map_err(|error| invalid(format!("uniform layout failed: {error}")))?;
    let mut uniform_sizes = BTreeMap::new();
    let mut uniform_stage_masks = BTreeMap::new();
    for (handle, global) in module.global_variables.iter() {
        match global.space {
            naga::AddressSpace::Private => {}
            naga::AddressSpace::Uniform => {
                let binding = global
                    .binding
                    .as_ref()
                    .ok_or_else(|| invalid("uniform global has no group and binding".into()))?;
                let key = (binding.group, binding.binding);
                if uniform_sizes
                    .insert(key, layouter[global.ty].size)
                    .is_some()
                {
                    return Err(invalid(format!(
                        "uniform group {} binding {} appears more than once",
                        key.0, key.1
                    )));
                }
                let mut stage_mask = 0;
                for (index, entry) in module.entry_points.iter().enumerate() {
                    if info.get_entry_point(index)[handle].is_empty() {
                        continue;
                    }
                    stage_mask |= match entry.stage {
                        naga::ShaderStage::Vertex => 1 << GpuCanvasShaderStage::Vertex as u8,
                        naga::ShaderStage::Fragment => 1 << GpuCanvasShaderStage::Fragment as u8,
                        naga::ShaderStage::Compute => 1 << GpuCanvasShaderStage::Compute as u8,
                        unsupported => {
                            return Err(invalid(format!(
                                "entry point '{}' uses unsupported shader stage {unsupported:?}",
                                entry.name
                            )));
                        }
                    };
                }
                uniform_stage_masks.insert(key, stage_mask);
            }
            ref unsupported => {
                return Err(invalid(format!(
                    "global address space {unsupported:?} is outside the uniform-only imported contract"
                )));
            }
        }
    }

    let mut requirements = BTreeMap::new();
    for binding in &shader.bindings {
        if binding.kind != GpuCanvasShaderResourceKind::UniformBuffer {
            return Err(invalid(format!(
                "binding group {} binding {} has unsupported resource kind {:?}",
                binding.group, binding.binding, binding.kind
            )));
        }
        if binding.stage_mask & !0b111 != 0 {
            return Err(invalid(format!(
                "binding group {} binding {} has unknown stage mask {:#x}",
                binding.group, binding.binding, binding.stage_mask
            )));
        }
        let expected_slots = [
            (binding.stage_mask & (1 << GpuCanvasShaderStage::Vertex as u8) != 0)
                .then_some(u16::from(binding.binding)),
            (binding.stage_mask & (1 << GpuCanvasShaderStage::Fragment as u8) != 0)
                .then_some(u16::from(binding.binding)),
            (binding.stage_mask & (1 << GpuCanvasShaderStage::Compute as u8) != 0)
                .then_some(u16::from(binding.binding)),
        ];
        if binding.backend_space != binding.group || binding.backend_slots != expected_slots {
            return Err(invalid(format!(
                "binding group {} binding {} is not the WebGPU identity mapping",
                binding.group, binding.binding
            )));
        }

        let key = (u32::from(binding.group), u32::from(binding.binding));
        let required_size = uniform_sizes.get(&key).copied().ok_or_else(|| {
            invalid(format!(
                "binding map contains group {} binding {} absent from authored WGSL",
                binding.group, binding.binding
            ))
        })?;
        let actual_stage_mask = uniform_stage_masks[&key];
        if actual_stage_mask & !binding.stage_mask != 0 {
            return Err(invalid(format!(
                "binding group {} binding {} target-16 visibility {:#x} underdeclares authored WGSL usage {:#x}",
                binding.group, binding.binding, binding.stage_mask, actual_stage_mask
            )));
        }
        let requirement = ImportedUniformRequirement {
            required_size,
            stage_mask: binding.stage_mask,
        };
        if requirements.insert(key, requirement).is_some() {
            return Err(invalid(format!(
                "binding map contains duplicate group {} binding {}",
                binding.group, binding.binding
            )));
        }
    }
    if requirements.keys().ne(uniform_sizes.keys()) {
        return Err(invalid(format!(
            "binding-map uniforms {:?} do not exactly match authored WGSL uniforms {:?}",
            requirements.keys().collect::<Vec<_>>(),
            uniform_sizes.keys().collect::<Vec<_>>()
        )));
    }
    Ok(requirements)
}

fn validate_imported_wgpu_limits(
    plan: &GpuCanvasPlan,
    uniform_requirements: &BTreeMap<(u32, u32), ImportedUniformRequirement>,
    limits: &wgpu::Limits,
) -> Result<(), GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!(
            "invalid imported GPU-canvas device limits: {message}"
        ))
    };
    let required_bind_groups = plan
        .uniform_buffers
        .iter()
        .map(|buffer| buffer.group.saturating_add(1))
        .max()
        .unwrap_or(0);
    if required_bind_groups > limits.max_bind_groups {
        return Err(invalid(format!(
            "draw requires {required_bind_groups} bind groups but the device supports {}",
            limits.max_bind_groups
        )));
    }

    for (label, stage) in [
        ("vertex", GpuCanvasShaderStage::Vertex),
        ("fragment", GpuCanvasShaderStage::Fragment),
        ("compute", GpuCanvasShaderStage::Compute),
    ] {
        let stage_bit = 1 << stage as u8;
        let count = uniform_requirements
            .values()
            .filter(|requirement| requirement.stage_mask & stage_bit != 0)
            .count();
        if count > limits.max_uniform_buffers_per_shader_stage as usize {
            return Err(invalid(format!(
                "{label} stage requires {count} uniform buffers across bind groups but the device supports {} per stage",
                limits.max_uniform_buffers_per_shader_stage
            )));
        }
    }

    if let Some(buffer) = plan
        .uniform_buffers
        .iter()
        .find(|buffer| buffer.bytes.len() > limits.max_uniform_buffer_binding_size as usize)
    {
        return Err(invalid(format!(
            "uniform group {} binding {} contains {} bytes but the device supports {} per binding",
            buffer.group,
            buffer.binding,
            buffer.bytes.len(),
            limits.max_uniform_buffer_binding_size
        )));
    }
    Ok(())
}

impl WgpuFactory {
    /// Execute authored RSTB WGSL on the retained device and return the
    /// offscreen texture as a normal image owned by this factory domain.
    pub(super) fn make_imported_gpu_canvas_image(
        &mut self,
        shader: &GpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        validate_imported_gpu_canvas_plan(plan)
            .map_err(|error| GpuCanvasError::new(error.to_string()))?;
        let target_lease = self.gpu_canvas_targets.acquire(plan.width, plan.height)?;
        let device = &self.context.device;
        let queue = &self.context.queue;
        let key = ImportedGpuCanvasPipelineKey::new(shader, plan);
        let pipeline_index = self
            .imported_gpu_canvas
            .pipelines
            .iter()
            .position(|pipeline| pipeline.key == key);
        let prepared_pipeline = if pipeline_index.is_none() {
            let prepared = prepare_imported_gpu_canvas(shader, plan)?;
            validate_imported_wgpu_limits(plan, &prepared.uniform_requirements, &device.limits())?;
            let vertex_attributes = plan
                .vertex_layouts
                .iter()
                .map(|layout| {
                    layout
                        .attributes
                        .iter()
                        .map(|attribute| {
                            Ok(wgpu::VertexAttribute {
                                format: vertex_format(&attribute.format)?,
                                offset: attribute.offset,
                                shader_location: attribute.shader_location,
                            })
                        })
                        .collect::<Result<Vec<_>, RendererError>>()
                })
                .collect::<Result<Vec<_>, RendererError>>()
                .map_err(|error| GpuCanvasError::new(error.to_string()))?;
            Some((prepared, vertex_attributes))
        } else {
            let cached = self
                .imported_gpu_canvas
                .pipelines
                .get(pipeline_index.expect("cached imported pipeline index exists"))
                .expect("cached imported GPU-canvas pipeline exists");
            validate_imported_wgpu_limits(plan, &cached.uniform_requirements, &device.limits())?;
            None
        };
        #[cfg(not(target_arch = "wasm32"))]
        let validation_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut built_pipeline = None;
        if let Some((prepared, vertex_attributes)) = prepared_pipeline {
            // C++ `src/lua/renderer/lua_gpu.cpp::buildShaderEntries` creates
            // one ShaderModule for whole-module WGSL and points every entry
            // record at it. WebGPU selects each authored physical entry name
            // from this single shared module.
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nuxie-imported-gpu-canvas"),
                source: wgpu::ShaderSource::Wgsl(shader.source.clone().into()),
            });
            let mut bind_group_layouts = Vec::new();
            if let Some(max_group) = plan.uniform_buffers.iter().map(|buffer| buffer.group).max() {
                for group in 0..=max_group {
                    let entries = plan
                        .uniform_buffers
                        .iter()
                        .filter(|buffer| buffer.group == group)
                        .map(|buffer| wgpu::BindGroupLayoutEntry {
                            binding: buffer.binding,
                            visibility: prepared.uniform_requirements
                                [&(buffer.group, buffer.binding)]
                                .visibility(),
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: NonZeroU64::new(buffer.bytes.len() as u64),
                            },
                            count: None,
                        })
                        .collect::<Vec<_>>();
                    bind_group_layouts.push(device.create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            label: Some("nuxie-imported-gpu-canvas-bind-group-layout"),
                            entries: &entries,
                        },
                    ));
                }
            }
            let layout_refs = bind_group_layouts.iter().map(Some).collect::<Vec<_>>();
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("nuxie-imported-gpu-canvas-pipeline-layout"),
                bind_group_layouts: &layout_refs,
                immediate_size: 0,
            });
            let vertex_layouts = plan
                .vertex_layouts
                .iter()
                .zip(&vertex_attributes)
                .map(|(layout, attributes)| {
                    Some(wgpu::VertexBufferLayout {
                        array_stride: layout.stride,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes,
                    })
                })
                .collect::<Vec<_>>();
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("nuxie-imported-gpu-canvas-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some(&prepared.vertex_entry_point),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &vertex_layouts,
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some(&prepared.fragment_entry_point),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });
            let uniform_buffers = plan
                .uniform_buffers
                .iter()
                .map(|buffer| {
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("nuxie-imported-gpu-canvas-uniform"),
                        contents: &buffer.bytes,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    })
                })
                .collect::<Vec<_>>();
            let bind_groups = bind_group_layouts
                .iter()
                .enumerate()
                .map(|(group, layout)| {
                    let entries = plan
                        .uniform_buffers
                        .iter()
                        .enumerate()
                        .filter(|(_, buffer)| buffer.group == group as u32)
                        .map(|(index, buffer)| wgpu::BindGroupEntry {
                            binding: buffer.binding,
                            resource: uniform_buffers[index].as_entire_binding(),
                        })
                        .collect::<Vec<_>>();
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("nuxie-imported-gpu-canvas-bind-group"),
                        layout,
                        entries: &entries,
                    })
                })
                .collect::<Vec<_>>();
            let vertex_buffers = plan
                .vertex_buffers
                .iter()
                .map(|buffer| {
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("nuxie-imported-gpu-canvas-vertex-buffer"),
                        contents: &buffer.bytes,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    })
                })
                .collect::<Vec<_>>();
            built_pipeline = Some(ImportedWgpuGpuCanvasPipeline {
                key,
                uniform_requirements: prepared.uniform_requirements,
                bind_groups,
                uniform_buffers,
                vertex_buffers,
                pipeline,
            });
        }
        let cached = built_pipeline.as_ref().unwrap_or_else(|| {
            self.imported_gpu_canvas
                .pipelines
                .get(pipeline_index.expect("imported GPU-canvas pipeline index exists"))
                .expect("imported GPU-canvas pipeline was initialized")
        });
        for (buffer, gpu_buffer) in plan.uniform_buffers.iter().zip(&cached.uniform_buffers) {
            queue.write_buffer(gpu_buffer, 0, &buffer.bytes);
        }
        for (buffer, gpu_buffer) in plan.vertex_buffers.iter().zip(&cached.vertex_buffers) {
            queue.write_buffer(gpu_buffer, 0, &buffer.bytes);
        }
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-imported-gpu-canvas-target"),
            size: wgpu::Extent3d {
                width: plan.width,
                height: plan.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nuxie-imported-gpu-canvas-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nuxie-imported-gpu-canvas-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: plan.clear_color[0],
                            g: plan.clear_color[1],
                            b: plan.clear_color[2],
                            a: plan.clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&cached.pipeline);
            for (index, bind_group) in cached.bind_groups.iter().enumerate() {
                pass.set_bind_group(index as u32, bind_group, &[]);
            }
            for (buffer, gpu_buffer) in plan.vertex_buffers.iter().zip(&cached.vertex_buffers) {
                pass.set_vertex_buffer(buffer.slot, gpu_buffer.slice(..));
            }
            pass.draw(
                plan.first_vertex..plan.first_vertex.saturating_add(plan.vertex_count),
                plan.first_instance..plan.first_instance.saturating_add(plan.instance_count),
            );
        }
        queue.submit(Some(encoder.finish()));
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(error) = pollster::block_on(validation_scope.pop()) {
                return Err(GpuCanvasError::new(format!(
                    "wgpu rejected imported GPU-canvas WGSL: {error}"
                )));
            }
        }
        if let Some(pipeline) = built_pipeline {
            self.imported_gpu_canvas.insert(pipeline);
            self.imported_gpu_canvas.pipeline_builds =
                self.imported_gpu_canvas.pipeline_builds.saturating_add(1);
        }
        let image = Box::new(WgpuImage {
            width: plan.width,
            height: plan.height,
            texture: Some(Arc::new(WgpuImageTexture {
                texture: target,
                view,
            })),
            owner: Arc::downgrade(&self.context),
        });
        Ok(retain_gpu_canvas_target(image, target_lease))
    }

    /// Execute one validated script-authored WGSL pass and return tightly
    /// packed RGBA pixels. The caller retains the factory across temporal
    /// samples so device selection and shader behavior stay fixed.
    pub async fn render_gpu_canvas(
        &self,
        plan: &GpuCanvasRenderPlan,
    ) -> Result<Vec<u8>, RendererError> {
        validate_gpu_canvas_plan(plan)?;
        let device = &self.context.device;
        let queue = &self.context.queue;
        let validation_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-gpu-canvas-shader"),
            source: wgpu::ShaderSource::Wgsl(plan.shader_wgsl.clone().into()),
        });

        let max_group = plan.uniform_buffers.iter().map(|buffer| buffer.group).max();
        let mut bind_group_layouts = Vec::new();
        let mut bind_groups = Vec::new();
        let mut uniform_gpu_buffers = Vec::new();
        if let Some(max_group) = max_group {
            for group in 0..=max_group {
                let group_buffers = plan
                    .uniform_buffers
                    .iter()
                    .filter(|buffer| buffer.group == group)
                    .collect::<Vec<_>>();
                let entries = group_buffers
                    .iter()
                    .map(|buffer| wgpu::BindGroupLayoutEntry {
                        binding: buffer.binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(buffer.bytes.len() as u64),
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>();
                let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("nuxie-gpu-canvas-bind-group-layout"),
                    entries: &entries,
                });
                let first_buffer = uniform_gpu_buffers.len();
                for buffer in &group_buffers {
                    uniform_gpu_buffers.push(device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("nuxie-gpu-canvas-uniform"),
                            contents: &buffer.bytes,
                            usage: wgpu::BufferUsages::UNIFORM,
                        },
                    ));
                }
                let binding_entries = group_buffers
                    .iter()
                    .enumerate()
                    .map(|(index, buffer)| wgpu::BindGroupEntry {
                        binding: buffer.binding,
                        resource: uniform_gpu_buffers[first_buffer + index].as_entire_binding(),
                    })
                    .collect::<Vec<_>>();
                bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("nuxie-gpu-canvas-bind-group"),
                    layout: &layout,
                    entries: &binding_entries,
                }));
                bind_group_layouts.push(layout);
            }
        }
        let layout_refs = bind_group_layouts.iter().map(Some).collect::<Vec<_>>();
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-gpu-canvas-pipeline-layout"),
            bind_group_layouts: &layout_refs,
            immediate_size: 0,
        });

        let vertex_attributes = plan
            .vertex_layouts
            .iter()
            .map(|layout| {
                layout
                    .attributes
                    .iter()
                    .map(|attribute| {
                        Ok(wgpu::VertexAttribute {
                            format: vertex_format(&attribute.format)?,
                            offset: attribute.offset,
                            shader_location: attribute.shader_location,
                        })
                    })
                    .collect::<Result<Vec<_>, RendererError>>()
            })
            .collect::<Result<Vec<_>, RendererError>>()?;
        let vertex_layouts = plan
            .vertex_layouts
            .iter()
            .zip(&vertex_attributes)
            .map(|(layout, attributes)| {
                Some(wgpu::VertexBufferLayout {
                    array_stride: layout.stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes,
                })
            })
            .collect::<Vec<_>>();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-gpu-canvas-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &vertex_layouts,
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_gpu_buffers = plan
            .vertex_buffers
            .iter()
            .map(|buffer| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("nuxie-gpu-canvas-vertex-buffer"),
                    contents: &buffer.bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                })
            })
            .collect::<Vec<_>>();
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-gpu-canvas-target"),
            size: wgpu::Extent3d {
                width: plan.width,
                height: plan.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let unpadded_bytes_per_row = plan.width.saturating_mul(4);
        let padded_bytes_per_row =
            align_to(unpadded_bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nuxie-gpu-canvas-readback"),
            size: u64::from(padded_bytes_per_row).saturating_mul(u64::from(plan.height)),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nuxie-gpu-canvas-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nuxie-gpu-canvas-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: plan.clear_color[0],
                            g: plan.clear_color[1],
                            b: plan.clear_color[2],
                            a: plan.clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipeline);
            for (index, bind_group) in bind_groups.iter().enumerate() {
                pass.set_bind_group(index as u32, bind_group, &[]);
            }
            for (buffer, gpu_buffer) in plan.vertex_buffers.iter().zip(&vertex_gpu_buffers) {
                pass.set_vertex_buffer(buffer.slot, gpu_buffer.slice(..));
            }
            pass.draw(
                plan.first_vertex..plan.first_vertex.saturating_add(plan.vertex_count),
                plan.first_instance..plan.first_instance.saturating_add(plan.instance_count),
            );
        }
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(plan.height),
                },
            },
            target.size(),
        );
        queue.submit(Some(encoder.finish()));
        if let Some(error) = validation_scope.pop().await {
            return Err(RendererError::InvalidGpuCanvas(format!(
                "wgpu rejected the validated plan: {error}"
            )));
        }

        let slice = readback.slice(..);
        map_buffer(&self.context, &slice).await?;
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| RendererError::Map(error.to_string()))?;
        let mut pixels = Vec::with_capacity(unpadded_bytes_per_row as usize * plan.height as usize);
        for row in mapped.chunks_exact(padded_bytes_per_row as usize) {
            pixels.extend_from_slice(&row[..unpadded_bytes_per_row as usize]);
        }
        drop(mapped);
        readback.unmap();
        Ok(pixels)
    }
}

/// Parse and validate the exact authored WGSL selected from RSTB target 0.
/// Retain Naga's module analysis so target-16 visibility can be checked before
/// any WebGPU object or error scope is needed.
fn parse_authored_wgsl(source: &str) -> Result<ParsedAuthoredWgsl, GpuCanvasError> {
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|error| GpuCanvasError::new(error.emit_to_string(source)))?;
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| GpuCanvasError::new(error.emit_to_string(source)))?;
    Ok(ParsedAuthoredWgsl { module, info })
}

/// Shared borrowed view of either public GPU-canvas plan shape.
struct GpuCanvasPlanRef<'a> {
    width: u32,
    height: u32,
    clear_color: &'a [f64; 4],
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
    uniform_buffers: &'a [GpuCanvasUniformBuffer],
    vertex_layouts: &'a [GpuCanvasVertexLayout],
    vertex_buffers: &'a [GpuCanvasVertexBuffer],
}

impl<'a> From<&'a GpuCanvasRenderPlan> for GpuCanvasPlanRef<'a> {
    fn from(plan: &'a GpuCanvasRenderPlan) -> Self {
        Self {
            width: plan.width,
            height: plan.height,
            clear_color: &plan.clear_color,
            vertex_count: plan.vertex_count,
            instance_count: plan.instance_count,
            first_vertex: plan.first_vertex,
            first_instance: plan.first_instance,
            uniform_buffers: &plan.uniform_buffers,
            vertex_layouts: &plan.vertex_layouts,
            vertex_buffers: &plan.vertex_buffers,
        }
    }
}

impl<'a> From<&'a GpuCanvasPlan> for GpuCanvasPlanRef<'a> {
    fn from(plan: &'a GpuCanvasPlan) -> Self {
        Self {
            width: plan.width,
            height: plan.height,
            clear_color: &plan.clear_color,
            vertex_count: plan.vertex_count,
            instance_count: plan.instance_count,
            first_vertex: plan.first_vertex,
            first_instance: plan.first_instance,
            uniform_buffers: &plan.uniform_buffers,
            vertex_layouts: &plan.vertex_layouts,
            vertex_buffers: &plan.vertex_buffers,
        }
    }
}

fn validate_gpu_canvas_plan(plan: &GpuCanvasRenderPlan) -> Result<(), RendererError> {
    validate_gpu_canvas_plan_ref(plan.into())
}

pub(super) fn validate_imported_gpu_canvas_plan(plan: &GpuCanvasPlan) -> Result<(), RendererError> {
    validate_gpu_canvas_plan_ref(plan.into())
}

fn validate_gpu_canvas_plan_ref(plan: GpuCanvasPlanRef<'_>) -> Result<(), RendererError> {
    let invalid = |message: String| RendererError::InvalidGpuCanvas(message);
    if plan.width == 0
        || plan.height == 0
        || plan.width > MAX_GPU_CANVAS_DIMENSION
        || plan.height > MAX_GPU_CANVAS_DIMENSION
    {
        return Err(invalid(format!(
            "dimensions must be between 1 and {MAX_GPU_CANVAS_DIMENSION}"
        )));
    }
    if plan
        .clear_color
        .iter()
        .any(|component| !component.is_finite() || !(0.0..=1.0).contains(component))
    {
        return Err(invalid(
            "clear color components must be finite values from 0 through 1".into(),
        ));
    }
    if plan.vertex_count == 0 || plan.instance_count == 0 {
        return Err(invalid(
            "vertex and instance counts must be positive".into(),
        ));
    }
    let vertex_end = plan
        .first_vertex
        .checked_add(plan.vertex_count)
        .ok_or_else(|| invalid("vertex range overflow".into()))?;
    let instance_end = plan
        .first_instance
        .checked_add(plan.instance_count)
        .ok_or_else(|| invalid("instance range overflow".into()))?;
    let invocations = u64::from(plan.vertex_count)
        .checked_mul(u64::from(plan.instance_count))
        .ok_or_else(|| invalid("draw invocation count overflow".into()))?;
    if invocations > MAX_DRAW_INVOCATIONS
        || u64::from(vertex_end) > MAX_DRAW_INVOCATIONS
        || u64::from(instance_end) > MAX_DRAW_INVOCATIONS
    {
        return Err(invalid(format!(
            "draw ranges may cover at most {MAX_DRAW_INVOCATIONS} invocations"
        )));
    }

    let mut bindings = BTreeSet::new();
    let mut group_counts = [0_usize; MAX_BIND_GROUPS as usize];
    for buffer in plan.uniform_buffers {
        if buffer.group >= MAX_BIND_GROUPS {
            return Err(invalid(format!(
                "bind group must be less than {MAX_BIND_GROUPS}"
            )));
        }
        if buffer.binding > MAX_BINDING_INDEX {
            return Err(invalid(format!(
                "uniform binding must be at most {MAX_BINDING_INDEX}"
            )));
        }
        if buffer.bytes.is_empty() || buffer.bytes.len() > MAX_UNIFORM_BUFFER_BYTES {
            return Err(invalid(format!(
                "uniform buffers must contain between 1 and {MAX_UNIFORM_BUFFER_BYTES} bytes"
            )));
        }
        if buffer.bytes.len() % 4 != 0 {
            return Err(invalid(
                "uniform buffer byte lengths must be four-byte aligned".into(),
            ));
        }
        if !bindings.insert((buffer.group, buffer.binding)) {
            return Err(invalid(format!(
                "uniform binding {} in group {} is duplicated",
                buffer.binding, buffer.group
            )));
        }
        group_counts[buffer.group as usize] += 1;
        if group_counts[buffer.group as usize] > MAX_UNIFORM_BINDINGS_PER_GROUP {
            return Err(invalid(format!(
                "bind group {} exceeds {MAX_UNIFORM_BINDINGS_PER_GROUP} uniform bindings",
                buffer.group
            )));
        }
    }
    if plan.vertex_layouts.len() > MAX_VERTEX_BUFFERS
        || plan.vertex_buffers.len() > MAX_VERTEX_BUFFERS
        || plan.vertex_layouts.len() != plan.vertex_buffers.len()
    {
        return Err(invalid(format!(
            "vertex layout and buffer counts must match and be at most {MAX_VERTEX_BUFFERS}"
        )));
    }
    let mut locations = BTreeSet::new();
    let mut buffer_slots = BTreeSet::new();
    let mut attribute_count = 0;
    for buffer in plan.vertex_buffers {
        if buffer.slot as usize >= MAX_VERTEX_BUFFERS {
            return Err(invalid(format!(
                "vertex buffer slot must be less than {MAX_VERTEX_BUFFERS}"
            )));
        }
        if !buffer_slots.insert(buffer.slot) {
            return Err(invalid(format!(
                "vertex buffer slot {} is duplicated",
                buffer.slot
            )));
        }
        if buffer.bytes.is_empty() || buffer.bytes.len() > MAX_VERTEX_BUFFER_BYTES {
            return Err(invalid(format!(
                "vertex buffers must contain between 1 and {MAX_VERTEX_BUFFER_BYTES} bytes"
            )));
        }
    }
    for (slot, layout) in plan.vertex_layouts.iter().enumerate() {
        if layout.stride == 0 || layout.stride > 2_048 {
            return Err(invalid(
                "vertex layout stride must be between 1 and 2048 bytes".into(),
            ));
        }
        if layout.attributes.is_empty() {
            return Err(invalid(
                "vertex layouts must contain at least one attribute".into(),
            ));
        }
        let slot = u32::try_from(slot).map_err(|_| invalid("vertex slot overflow".into()))?;
        let buffer = plan
            .vertex_buffers
            .iter()
            .find(|buffer| buffer.slot == slot)
            .ok_or_else(|| invalid(format!("vertex buffer slot {slot} is not bound")))?;
        let required_bytes = u64::from(vertex_end)
            .checked_mul(layout.stride)
            .ok_or_else(|| invalid("vertex buffer byte range overflow".into()))?;
        if required_bytes > buffer.bytes.len() as u64 {
            return Err(invalid(format!(
                "vertex buffer slot {slot} requires {required_bytes} bytes"
            )));
        }
        for attribute in &layout.attributes {
            attribute_count += 1;
            if attribute_count > MAX_VERTEX_ATTRIBUTES {
                return Err(invalid(format!(
                    "pipelines support at most {MAX_VERTEX_ATTRIBUTES} vertex attributes"
                )));
            }
            if attribute.shader_location >= MAX_VERTEX_ATTRIBUTES as u32
                || !locations.insert(attribute.shader_location)
            {
                return Err(invalid(format!(
                    "vertex attribute location {} is out of range or duplicated",
                    attribute.shader_location
                )));
            }
            let size = vertex_format_size(&attribute.format)?;
            if attribute
                .offset
                .checked_add(size)
                .is_none_or(|end| end > layout.stride)
            {
                return Err(invalid(format!(
                    "vertex attribute at offset {} exceeds stride {}",
                    attribute.offset, layout.stride
                )));
            }
        }
    }
    Ok(())
}

fn vertex_format_size(name: &str) -> Result<u64, RendererError> {
    match name {
        "float32" => Ok(4),
        "float32x2" => Ok(8),
        "float32x3" => Ok(12),
        "float32x4" => Ok(16),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "unsupported vertex format '{name}'"
        ))),
    }
}

fn vertex_format(name: &str) -> Result<wgpu::VertexFormat, RendererError> {
    match name {
        "float32" => Ok(wgpu::VertexFormat::Float32),
        "float32x2" => Ok(wgpu::VertexFormat::Float32x2),
        "float32x3" => Ok(wgpu::VertexFormat::Float32x3),
        "float32x4" => Ok(wgpu::VertexFormat::Float32x4),
        _ => Err(RendererError::Unsupported(
            "GPU-canvas vertex format is not implemented",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_render_api::{
        GpuCanvasShaderBinding, GpuCanvasShaderEntry, GpuCanvasShaderTextureSampleType,
        GpuCanvasShaderTextureViewDimension,
    };

    #[test]
    fn multi_occurrence_retained_targets_share_budget_and_release_on_replacement() {
        let budget = Arc::new(RetainedGpuCanvasTargetBudget::default());
        let target_bytes =
            MAX_GPU_CANVAS_DIMENSION as usize * MAX_GPU_CANVAS_DIMENSION as usize * 4;
        let mut occurrences = (0..4)
            .map(|_| {
                budget
                    .acquire(MAX_GPU_CANVAS_DIMENSION, MAX_GPU_CANVAS_DIMENSION)
                    .expect("four maximally sized occurrences fit the aggregate byte budget")
            })
            .collect::<Vec<_>>();
        assert_eq!(budget.retained(), (4, MAX_RETAINED_GPU_CANVAS_TARGET_BYTES));

        let error = budget
            .acquire(MAX_GPU_CANVAS_DIMENSION, MAX_GPU_CANVAS_DIMENSION)
            .expect_err("a fifth maximally sized occurrence must fail closed");
        assert!(error.to_string().contains("factory budget"), "{error}");
        assert_eq!(
            budget.retained(),
            (4, MAX_RETAINED_GPU_CANVAS_TARGET_BYTES),
            "a failed reservation must not consume aggregate capacity"
        );

        drop(occurrences.pop());
        let replacement = budget
            .acquire(MAX_GPU_CANVAS_DIMENSION, MAX_GPU_CANVAS_DIMENSION)
            .expect("dropping an occurrence releases its retained target lease");
        assert_eq!(budget.retained(), (4, target_bytes * 4));
        drop(replacement);
        drop(occurrences);
        assert_eq!(budget.retained(), (0, 0));

        let tiny_occurrences = (0..MAX_RETAINED_GPU_CANVAS_TARGETS)
            .map(|_| {
                budget
                    .acquire(1, 1)
                    .expect("tiny occurrences fit until the target-count fence")
            })
            .collect::<Vec<_>>();
        budget
            .acquire(1, 1)
            .expect_err("the target-count fence applies even below the byte limit");
        drop(tiny_occurrences);
        assert_eq!(budget.retained(), (0, 0));
    }

    fn valid_plan() -> GpuCanvasRenderPlan {
        GpuCanvasRenderPlan {
            shader_wgsl: String::new(),
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

    const IMPORTED_WGSL: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

    fn imported_shader(source: &str) -> GpuCanvasShader {
        GpuCanvasShader {
            source: source.into(),
            entries: vec![
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Vertex,
                    logical_entry_point: "vs_main".into(),
                    physical_entry_point: "vs_main".into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Fragment,
                    logical_entry_point: "fs_main".into(),
                    physical_entry_point: "fs_main".into(),
                },
            ],
            bindings: Vec::new(),
        }
    }

    fn imported_uniform_shader(source: &str, stage_mask: u8) -> GpuCanvasShader {
        let mut shader = imported_shader(source);
        shader.bindings.push(GpuCanvasShaderBinding {
            group: 0,
            binding: 0,
            kind: GpuCanvasShaderResourceKind::UniformBuffer,
            stage_mask,
            backend_space: 0,
            backend_slots: [
                (stage_mask & (1 << GpuCanvasShaderStage::Vertex as u8) != 0).then_some(0),
                (stage_mask & (1 << GpuCanvasShaderStage::Fragment as u8) != 0).then_some(0),
                (stage_mask & (1 << GpuCanvasShaderStage::Compute as u8) != 0).then_some(0),
            ],
            texture_view_dimension: GpuCanvasShaderTextureViewDimension::Undefined,
            texture_sample_type: GpuCanvasShaderTextureSampleType::Undefined,
            texture_multisampled: false,
        });
        shader
    }

    fn imported_shader_with_entries(
        source: &str,
        vertex_physical: &str,
        fragment_physical: &str,
    ) -> GpuCanvasShader {
        GpuCanvasShader {
            source: source.into(),
            entries: vec![
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Vertex,
                    logical_entry_point: "vs_main".into(),
                    physical_entry_point: vertex_physical.into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Fragment,
                    logical_entry_point: "fs_main".into(),
                    physical_entry_point: fragment_physical.into(),
                },
            ],
            bindings: Vec::new(),
        }
    }

    fn imported_plan() -> GpuCanvasPlan {
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

    #[test]
    fn imported_interface_preflight_accepts_only_the_exact_physical_stage_interface() {
        let shader = imported_shader(IMPORTED_WGSL);
        prepare_imported_gpu_canvas(&shader, &imported_plan()).unwrap();

        let vertex_with_input = IMPORTED_WGSL.replace(
            "fn vs_main(@builtin(vertex_index) index: u32)",
            "fn vs_main(@builtin(vertex_index) index: u32, @location(0) position: vec2<f32>)",
        );
        let error =
            prepare_imported_gpu_canvas(&imported_shader(&vertex_with_input), &imported_plan())
                .err()
                .expect("missing vertex plan must fail");
        assert!(error.to_string().contains("vertex inputs"), "{error}");

        let mut matching_plan = imported_plan();
        matching_plan.vertex_layouts.push(GpuCanvasVertexLayout {
            stride: 8,
            attributes: vec![GpuCanvasVertexAttribute {
                shader_location: 0,
                offset: 0,
                format: "float32x2".into(),
            }],
        });
        matching_plan.vertex_buffers.push(GpuCanvasVertexBuffer {
            slot: 0,
            bytes: vec![0; 24],
        });
        prepare_imported_gpu_canvas(&imported_shader(&vertex_with_input), &matching_plan).unwrap();
        matching_plan.vertex_layouts[0].attributes[0].format = "float32x3".into();
        matching_plan.vertex_layouts[0].stride = 12;
        matching_plan.vertex_buffers[0].bytes.resize(36, 0);
        let error =
            prepare_imported_gpu_canvas(&imported_shader(&vertex_with_input), &matching_plan)
                .err()
                .expect("wrong vertex format must fail");
        assert!(error.to_string().contains("vertex inputs"), "{error}");

        let fragment_vec3 = IMPORTED_WGSL
            .replace(
                "fn fs_main() -> @location(0) vec4<f32>",
                "fn fs_main() -> @location(0) vec3<f32>",
            )
            .replace(
                "return vec4<f32>(1.0, 0.0, 0.0, 1.0);",
                "return vec3<f32>(1.0, 0.0, 0.0);",
            );
        let error = prepare_imported_gpu_canvas(&imported_shader(&fragment_vec3), &imported_plan())
            .err()
            .expect("non-RGBA fragment output must fail");
        assert!(error.to_string().contains("fragment output"), "{error}");

        let wrong_entry = imported_shader_with_entries(IMPORTED_WGSL, "missing", "fs_main");
        let error = prepare_imported_gpu_canvas(&wrong_entry, &imported_plan())
            .err()
            .expect("missing physical entry point must fail");
        assert!(
            error.to_string().contains("physical entry point"),
            "{error}"
        );
    }

    #[test]
    fn imported_interface_preflight_validates_interstage_and_uniform_layouts() {
        let varying = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) varying_value: vec2<f32>,
}

@vertex
fn vs_main() -> VertexOutput {
    return VertexOutput(vec4<f32>(0.0), vec2<f32>(1.0));
}

@fragment
fn fs_main(@location(0) varying_value: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(varying_value, 1.0);
}
"#;
        let error = prepare_imported_gpu_canvas(&imported_shader(varying), &imported_plan())
            .err()
            .expect("inter-stage type mismatch must fail");
        assert!(error.to_string().contains("inter-stage"), "{error}");

        let uniform_wgsl = IMPORTED_WGSL
            .replace(
                "@fragment",
                "struct Tint { value: vec4<f32>, }\n@group(0) @binding(0) var<uniform> tint: Tint;\n\n@fragment",
            )
            .replace(
                "return vec4<f32>(1.0, 0.0, 0.0, 1.0);",
                "return tint.value;",
            );
        let error = prepare_imported_gpu_canvas(
            &imported_uniform_shader(&uniform_wgsl, 1 << GpuCanvasShaderStage::Fragment as u8),
            &imported_plan(),
        )
        .err()
        .expect("missing uniform binding must fail");
        assert!(error.to_string().contains("uniform bindings"), "{error}");

        let mut undersized = imported_plan();
        undersized.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 8],
        });
        let error = prepare_imported_gpu_canvas(
            &imported_uniform_shader(&uniform_wgsl, 1 << GpuCanvasShaderStage::Fragment as u8),
            &undersized,
        )
        .err()
        .expect("undersized uniform binding must fail");
        assert!(error.to_string().contains("requires 16"), "{error}");

        let mut exact = imported_plan();
        exact.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 16],
        });
        prepare_imported_gpu_canvas(
            &imported_uniform_shader(&uniform_wgsl, 1 << GpuCanvasShaderStage::Fragment as u8),
            &exact,
        )
        .unwrap();
    }

    #[test]
    fn binding_map_identity_and_stage_visibility_fail_closed() {
        let uniform_wgsl = IMPORTED_WGSL
            .replace(
                "@fragment",
                "struct Tint { value: vec4<f32>, }\n@group(0) @binding(0) var<uniform> tint: Tint;\n\n@fragment",
            )
            .replace(
                "return vec4<f32>(1.0, 0.0, 0.0, 1.0);",
                "return tint.value;",
            );
        let fragment_mask = 1 << GpuCanvasShaderStage::Fragment as u8;
        let mut plan = imported_plan();
        plan.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 16],
        });
        prepare_imported_gpu_canvas(
            &imported_uniform_shader(&uniform_wgsl, fragment_mask),
            &plan,
        )
        .expect("canonical target-16 WebGPU identity metadata is valid");

        let vertex_mask = 1 << GpuCanvasShaderStage::Vertex as u8;
        let error = prepare_imported_gpu_canvas(
            &imported_uniform_shader(&uniform_wgsl, vertex_mask),
            &plan,
        )
        .err()
        .expect("target-16 visibility may not omit a stage that actually uses the binding");
        assert!(error.to_string().contains("underdeclares"), "{error}");

        let broader_mask = fragment_mask | (1 << GpuCanvasShaderStage::Compute as u8);
        prepare_imported_gpu_canvas(&imported_uniform_shader(&uniform_wgsl, broader_mask), &plan)
            .expect("C++ and WebGPU allow layout visibility broader than actual use");

        let mut unknown_stage = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        unknown_stage.bindings[0].stage_mask |= 0x80;
        let error = prepare_imported_gpu_canvas(&unknown_stage, &plan)
            .err()
            .expect("unknown target-16 stage bits must fail closed");
        assert!(error.to_string().contains("unknown stage mask"), "{error}");

        let mut absent_visible_slot = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        absent_visible_slot.bindings[0].backend_slots[1] = None;
        let error = prepare_imported_gpu_canvas(&absent_visible_slot, &plan)
            .err()
            .expect("a visible stage must retain its target-16 native slot");
        assert!(error.to_string().contains("identity mapping"), "{error}");

        let mut populated_absent_slot = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        populated_absent_slot.bindings[0].backend_slots[0] = Some(0);
        let error = prepare_imported_gpu_canvas(&populated_absent_slot, &plan)
            .err()
            .expect("an invisible stage must preserve BindingMap::kAbsent");
        assert!(error.to_string().contains("identity mapping"), "{error}");

        let mut remapped_space = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        remapped_space.bindings[0].backend_space = 1;
        let error = prepare_imported_gpu_canvas(&remapped_space, &plan)
            .err()
            .expect("WebGPU consumes the authored group without cross-backend remapping");
        assert!(error.to_string().contains("identity mapping"), "{error}");
    }

    #[test]
    fn shared_authored_module_drives_both_entries_and_binding_visibility() {
        let shared_wgsl = r#"
struct Tint {
    value: vec4<f32>,
}
@group(0) @binding(0) var<uniform> tint: Tint;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, tint.value.x * 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return tint.value;
}
"#;
        let stage_mask =
            (1 << GpuCanvasShaderStage::Vertex as u8) | (1 << GpuCanvasShaderStage::Fragment as u8);
        let shader = imported_uniform_shader(shared_wgsl, stage_mask);
        let mut plan = imported_plan();
        plan.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 16],
        });

        let prepared = prepare_imported_gpu_canvas(&shader, &plan)
            .expect("one authored module contains both physical entries");
        assert_eq!(prepared.vertex_entry_point, "vs_main");
        assert_eq!(prepared.fragment_entry_point, "fs_main");
        assert_eq!(
            prepared.uniform_requirements[&(0, 0)].visibility(),
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );
    }

    #[test]
    fn arbitrary_entries_use_declaration_order_defaults_and_exact_pipeline_selection() {
        let source = r#"
@vertex
fn physical_vertex_0(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(f32(i32(index) - 1), f32(i32(index & 1u) * 2 - 1), 0.0, 1.0);
}

@vertex
fn physical_vertex_1(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(f32(i32(index) - 1), f32(i32(index & 1u) * 2 - 1), 0.0, 1.0);
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
        let shader = GpuCanvasShader {
            source: source.into(),
            entries: vec![
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Vertex,
                    logical_entry_point: "default_vertex".into(),
                    physical_entry_point: "physical_vertex_0".into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Vertex,
                    logical_entry_point: "chosen_vertex".into(),
                    physical_entry_point: "physical_vertex_1".into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Fragment,
                    logical_entry_point: "default_fragment".into(),
                    physical_entry_point: "physical_fragment_0".into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Fragment,
                    logical_entry_point: "chosen_fragment".into(),
                    physical_entry_point: "physical_fragment_1".into(),
                },
            ],
            bindings: Vec::new(),
        };
        let mut plan = imported_plan();

        let prepared = prepare_imported_gpu_canvas(&shader, &plan)
            .expect("bare modules select the first declaration of each stage");
        assert_eq!(prepared.vertex_entry_point, "physical_vertex_0");
        assert_eq!(prepared.fragment_entry_point, "physical_fragment_0");

        plan.vertex_entry = Some(GpuCanvasShaderEntrySelection {
            logical_entry_point: "chosen_vertex".into(),
            physical_entry_point: "physical_vertex_1".into(),
        });
        plan.fragment_entry = Some(GpuCanvasShaderEntrySelection {
            logical_entry_point: "chosen_fragment".into(),
            physical_entry_point: "physical_fragment_1".into(),
        });
        let prepared = prepare_imported_gpu_canvas(&shader, &plan)
            .expect("the pipeline carries its resolved logical/physical pair");
        assert_eq!(prepared.vertex_entry_point, "physical_vertex_1");
        assert_eq!(prepared.fragment_entry_point, "physical_fragment_1");

        plan.fragment_entry.as_mut().unwrap().physical_entry_point = "physical_fragment_0".into();
        let error = prepare_imported_gpu_canvas(&shader, &plan)
            .err()
            .expect("a stale logical/physical pair fails before device allocation");
        assert!(
            error.to_string().contains("no matching fragment"),
            "{error}"
        );
    }

    #[test]
    fn imported_path_has_no_glsl_cross_translation_surface() {
        let source = include_str!("gpu_canvas.rs");
        let legacy_helper = ["canonical_glsl", "_es300_to_wgsl"].concat();
        let glsl_frontend = ["naga::front::", "glsl"].concat();
        let wgsl_backend = ["naga::back::", "wgsl"].concat();
        let imported_module_creation = ["label: Some(\"nuxie-imported", "-gpu-canvas\"),"].concat();
        assert!(!source.contains(&legacy_helper));
        assert!(!source.contains(&glsl_frontend));
        assert!(!source.contains(&wgsl_backend));
        assert_eq!(
            source.matches(&imported_module_creation).count(),
            1,
            "the imported pipeline must create one module shared by both stages",
        );

        let manifest = include_str!("../Cargo.toml");
        assert!(!manifest.contains("\"glsl-in\""));
        assert!(!manifest.contains("\"wgsl-out\""));
        assert!(!manifest.contains("features = [\"glsl\"]"));
    }

    #[test]
    fn imported_pipeline_key_reuses_resources_for_animated_buffer_bytes() {
        let shader = imported_shader(IMPORTED_WGSL);
        let mut first = imported_plan();
        first.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 16],
        });
        let mut second = first.clone();
        second.uniform_buffers[0].bytes.fill(0xff);

        assert_eq!(
            ImportedGpuCanvasPipelineKey::new(&shader, &first),
            ImportedGpuCanvasPipelineKey::new(&shader, &second),
            "temporal byte updates must retain authored modules, pipelines, and buffers"
        );

        second.uniform_buffers[0].bytes.push(0);
        assert_ne!(
            ImportedGpuCanvasPipelineKey::new(&shader, &first),
            ImportedGpuCanvasPipelineKey::new(&shader, &second),
            "resource-size changes require a fresh backend allocation"
        );
    }

    #[test]
    fn imported_webgpu_limits_count_reflected_uniforms_per_stage_across_groups() {
        let mut plan = imported_plan();
        plan.uniform_buffers = (0..13)
            .map(|index| GpuCanvasUniformBuffer {
                group: index / 7,
                binding: index % 7,
                bytes: vec![0; 16],
            })
            .collect();
        let fragment_only = plan
            .uniform_buffers
            .iter()
            .map(|buffer| {
                (
                    (buffer.group, buffer.binding),
                    ImportedUniformRequirement {
                        required_size: 16,
                        stage_mask: 1 << GpuCanvasShaderStage::Fragment as u8,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let limits = wgpu::Limits {
            max_uniform_buffers_per_shader_stage: 12,
            ..wgpu::Limits::downlevel_defaults()
        };
        let error = validate_imported_wgpu_limits(&plan, &fragment_only, &limits)
            .expect_err("13 fragment uniforms must fail before device allocation");
        assert!(
            error
                .to_string()
                .contains("fragment stage requires 13 uniform buffers"),
            "{error}"
        );

        let split_stages = fragment_only
            .into_iter()
            .enumerate()
            .map(|(index, (binding, mut requirement))| {
                requirement.stage_mask = if index < 7 {
                    1 << GpuCanvasShaderStage::Vertex as u8
                } else {
                    1 << GpuCanvasShaderStage::Fragment as u8
                };
                (binding, requirement)
            })
            .collect::<BTreeMap<_, _>>();
        validate_imported_wgpu_limits(&plan, &split_stages, &limits)
            .expect("13 total bindings remain valid when neither stage exceeds its device limit");
        assert_eq!(
            split_stages[&(0, 0)].visibility(),
            wgpu::ShaderStages::VERTEX
        );
        assert_eq!(
            split_stages[&(1, 5)].visibility(),
            wgpu::ShaderStages::FRAGMENT
        );
    }

    #[test]
    fn product_vertex_formats_are_explicit_and_fail_closed() {
        assert_eq!(
            vertex_format("float32x3").unwrap(),
            wgpu::VertexFormat::Float32x3
        );
        assert!(vertex_format("snorm10x3").is_err());
    }

    #[test]
    fn product_plan_limits_fail_before_backend_allocation() {
        let mut plan = valid_plan();
        plan.width = MAX_GPU_CANVAS_DIMENSION + 1;
        assert!(validate_gpu_canvas_plan(&plan).is_err());

        let mut plan = valid_plan();
        plan.vertex_count = MAX_DRAW_INVOCATIONS as u32;
        plan.instance_count = 2;
        assert!(validate_gpu_canvas_plan(&plan).is_err());

        let mut plan = valid_plan();
        plan.uniform_buffers = vec![GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; MAX_UNIFORM_BUFFER_BYTES + 4],
        }];
        assert!(validate_gpu_canvas_plan(&plan).is_err());
    }

    #[test]
    fn product_plan_rejects_duplicate_bindings_and_vertex_slots() {
        let mut plan = valid_plan();
        plan.uniform_buffers = vec![
            GpuCanvasUniformBuffer {
                group: 0,
                binding: 0,
                bytes: vec![0; 16],
            },
            GpuCanvasUniformBuffer {
                group: 0,
                binding: 0,
                bytes: vec![0; 16],
            },
        ];
        assert!(validate_gpu_canvas_plan(&plan).is_err());

        let mut plan = valid_plan();
        plan.vertex_layouts = vec![
            GpuCanvasVertexLayout {
                stride: 4,
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: "float32".into(),
                }],
            },
            GpuCanvasVertexLayout {
                stride: 4,
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 1,
                    offset: 0,
                    format: "float32".into(),
                }],
            },
        ];
        plan.vertex_buffers = vec![
            GpuCanvasVertexBuffer {
                slot: 0,
                bytes: vec![0; 12],
            },
            GpuCanvasVertexBuffer {
                slot: 0,
                bytes: vec![0; 12],
            },
        ];
        assert!(validate_gpu_canvas_plan(&plan).is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn executes_validated_wgsl_and_reads_real_pixels() {
        let Ok(factory) = WgpuFactory::new(8, 8) else {
            eprintln!("GPU adapter unavailable; browser execution remains a separate proof");
            return;
        };
        let plan = GpuCanvasRenderPlan {
            shader_wgsl: r#"
                @vertex
                fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
                    let x = f32(i32(index) - 1);
                    let y = f32(i32(index & 1u) * 2 - 1);
                    return vec4<f32>(x, y, 0.0, 1.0);
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                }
            "#
            .into(),
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
        };
        let pixels =
            pollster::block_on(factory.render_gpu_canvas(&plan)).expect("WGSL draw completes");
        assert_eq!(pixels.len(), 8 * 8 * 4);
        assert!(
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10),
            "fullscreen triangle produces red pixels"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn cached_wgpu_imported_pipelines_update_bytes_and_retain_multiple_keys() {
        use nuxie_render_api::{BlendMode, ImageSampler, Renderer as _};

        let shader = imported_uniform_shader(
            r#"
struct Tint {
    value: vec4<f32>,
}
@group(0) @binding(0) var<uniform> tint: Tint;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return tint.value;
}
"#,
            1 << GpuCanvasShaderStage::Fragment as u8,
        );
        let encode_f32s = |values: &[f32]| {
            values
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<_>>()
        };
        let mut plan = GpuCanvasPlan {
            vertex_entry: None,
            fragment_entry: None,
            width: 32,
            height: 24,
            clear_color: [0.0, 0.0, 1.0, 1.0],
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            uniform_buffers: vec![GpuCanvasUniformBuffer {
                group: 0,
                binding: 0,
                bytes: encode_f32s(&[1.0, 0.0, 0.0, 1.0]),
            }],
            vertex_layouts: vec![GpuCanvasVertexLayout {
                stride: 8,
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: "float32x2".into(),
                }],
            }],
            vertex_buffers: vec![GpuCanvasVertexBuffer {
                slot: 0,
                bytes: encode_f32s(&[0.0; 6]),
            }],
        };

        for mode in [crate::RenderMode::ClockwiseAtomic, crate::RenderMode::Msaa] {
            let Ok(mut factory) = WgpuFactory::new_with_mode(64, 48, mode) else {
                eprintln!(
                    "GPU adapter unavailable; exact iOS execution remains the required proof"
                );
                return;
            };
            let first = factory
                .make_imported_gpu_canvas_image(&shader, &plan)
                .expect("first byte set creates the imported pipeline");
            plan.uniform_buffers[0].bytes = encode_f32s(&[0.0, 1.0, 0.0, 1.0]);
            plan.vertex_buffers[0].bytes = encode_f32s(&[-1.0, -1.0, 3.0, -1.0, -1.0, 3.0]);
            let green = factory
                .make_imported_gpu_canvas_image(&shader, &plan)
                .expect("same pipeline accepts changed uniform and vertex bytes");
            assert_eq!(factory.imported_gpu_canvas.pipeline_builds, 1);
            drop(first);
            let mut frame = factory.begin_frame(0xff00_0000);
            frame.draw_image(
                Some(green.as_ref()),
                ImageSampler::default(),
                BlendMode::SrcOver,
                1.0,
            );
            let pixels = frame.finish().expect("updated image composites");
            assert!(
                pixels
                    .chunks_exact(4)
                    .filter(|pixel| pixel[0] < 10 && pixel[1] > 240 && pixel[2] < 10)
                    .count()
                    > 300,
                "{mode:?} must upload both changed vertex and uniform bytes"
            );

            let mut alternate = shader.clone();
            alternate.source = alternate
                .source
                .replace("return tint.value;", "return tint.value.bgra;");
            let alternate_image = factory
                .make_imported_gpu_canvas_image(&alternate, &plan)
                .expect("a second shader key builds independently");
            plan.uniform_buffers[0].bytes = encode_f32s(&[1.0, 0.0, 0.0, 1.0]);
            let red = factory
                .make_imported_gpu_canvas_image(&shader, &plan)
                .expect("returning to the first key reuses its retained pipeline");
            assert_eq!(
                factory.imported_gpu_canvas.pipeline_builds, 2,
                "alternating two shader keys must build each pipeline exactly once"
            );
            drop(alternate_image);
            let mut frame = factory.begin_frame(0xff00_0000);
            frame.draw_image(
                Some(red.as_ref()),
                ImageSampler::default(),
                BlendMode::SrcOver,
                1.0,
            );
            let pixels = frame.finish().expect("reused first-key image composites");
            assert!(
                pixels
                    .chunks_exact(4)
                    .filter(|pixel| pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10)
                    .count()
                    > 300,
                "{mode:?} must update the retained first-key uniform after alternating keys"
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn canonical_rstb_wgsl_becomes_a_composited_render_image() {
        use nuxie_render_api::{BlendMode, ImageSampler, Renderer as _};

        let shader = imported_shader(IMPORTED_WGSL);
        let plan = GpuCanvasPlan {
            vertex_entry: None,
            fragment_entry: None,
            width: 255,
            height: 255,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            uniform_buffers: Vec::new(),
            vertex_layouts: Vec::new(),
            vertex_buffers: Vec::new(),
        };
        for mode in [crate::RenderMode::ClockwiseAtomic, crate::RenderMode::Msaa] {
            let Ok(mut factory) = WgpuFactory::new_with_mode(160, 100, mode) else {
                eprintln!(
                    "GPU adapter unavailable; exact iOS execution remains the required proof"
                );
                return;
            };
            let first_image = factory
                .make_imported_gpu_canvas_image(&shader, &plan)
                .expect("authored WGSL renders to a retained image");
            let image = factory
                .make_imported_gpu_canvas_image(&shader, &plan)
                .expect("the retained imported pipeline renders a second image");
            assert_eq!(
                factory.imported_gpu_canvas.pipeline_builds, 1,
                "identical temporal frames must not rebuild the shared WGPU shader pipeline"
            );
            drop(first_image);
            let mut frame = factory.begin_frame(0xff00_0000);
            frame.draw_image(
                Some(image.as_ref()),
                ImageSampler::default(),
                BlendMode::SrcOver,
                1.0,
            );
            let pixels = frame.finish().expect("image composite completes");
            let red_pixel_count = pixels
                .chunks_exact(4)
                .filter(|pixel| pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10)
                .count();
            assert!(
                red_pixel_count > 1_000,
                "{mode:?} produced only {red_pixel_count} red pixels"
            );
        }
    }
}
