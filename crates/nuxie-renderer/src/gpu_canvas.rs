//! Arbitrary WGSL draw execution for editor GPU-canvas critique frames.
//!
//! Luau execution lives in `nuxie-scripting`; this module accepts only its
//! typed draw plan and owns shader modules, buffers, submission, and readback.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use nuxie_render_api::{GpuCanvasError, GpuCanvasPlan, GpuCanvasShader, RenderImage};
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

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WebGl2UniformBlock {
    group: u32,
    binding: u32,
    name: String,
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WebGl2TranslatedStage {
    source: String,
    uniform_blocks: Vec<WebGl2UniformBlock>,
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WebGl2TranslatedProgram {
    vertex: WebGl2TranslatedStage,
    fragment: WebGl2TranslatedStage,
    vertex_attribute_formats: std::collections::BTreeMap<u32, String>,
}

/// Parse and validate once, then emit the exact GLSL ES 3.00 stages consumed
/// by WebGL2. The typed draw plan intentionally has no texture, storage, or
/// compute resource variants, so shaders that escape that closed contract fail
/// before a browser allocation.
#[cfg(any(test, target_arch = "wasm32"))]
fn translate_gpu_canvas_wgsl_to_webgl2(
    source: &str,
) -> Result<WebGl2TranslatedProgram, RendererError> {
    let invalid = |message: String| RendererError::InvalidGpuCanvas(message);
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|error| invalid(error.emit_to_string(source)))?;
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| invalid(error.emit_to_string(source)))?;

    for (_, global) in module.global_variables.iter() {
        match global.space {
            naga::AddressSpace::Uniform => {
                if global.binding.is_none() {
                    return Err(invalid(
                        "WebGL2 GPU-canvas uniform buffers require @group and @binding".into(),
                    ));
                }
            }
            naga::AddressSpace::Private => {}
            _ => {
                return Err(invalid(
                    "WebGL2 GPU-canvas shaders support only private values and uniform buffers"
                        .into(),
                ));
            }
        }
    }

    let vertex_entry = module
        .entry_points
        .iter()
        .find(|entry| entry.stage == naga::ShaderStage::Vertex && entry.name == "vs_main")
        .ok_or_else(|| invalid("WGSL must define vs_main as a vertex entry point".into()))?;
    let mut vertex_inputs = Vec::new();
    let mut _vertex_builtin_inputs = 0;
    for argument in &vertex_entry.function.arguments {
        collect_io_locations(
            &module,
            argument.binding.as_ref(),
            argument.ty,
            &mut vertex_inputs,
            &mut _vertex_builtin_inputs,
        );
    }
    let mut vertex_attribute_formats = std::collections::BTreeMap::new();
    for (location, ty) in vertex_inputs {
        let format = naga_vertex_format(&module, ty)?;
        if vertex_attribute_formats
            .insert(location, format.into())
            .is_some()
        {
            return Err(invalid(format!(
                "vertex attribute location {location} is duplicated"
            )));
        }
    }

    let fragment_entry = module
        .entry_points
        .iter()
        .find(|entry| entry.stage == naga::ShaderStage::Fragment && entry.name == "fs_main")
        .ok_or_else(|| invalid("WGSL must define fs_main as a fragment entry point".into()))?;
    let fragment_result = fragment_entry
        .function
        .result
        .as_ref()
        .ok_or_else(|| invalid("fs_main must return one RGBA value at location 0".into()))?;
    let mut fragment_outputs = Vec::new();
    let mut fragment_builtin_outputs = 0;
    collect_io_locations(
        &module,
        fragment_result.binding.as_ref(),
        fragment_result.ty,
        &mut fragment_outputs,
        &mut fragment_builtin_outputs,
    );
    if fragment_builtin_outputs != 0
        || fragment_outputs.len() != 1
        || fragment_outputs[0].0 != 0
        || naga_vertex_format(&module, fragment_outputs[0].1)? != "float32x4"
    {
        return Err(invalid(
            "fs_main must return exactly one vec4<f32> RGBA value at location 0".into(),
        ));
    }

    Ok(WebGl2TranslatedProgram {
        vertex: translate_naga_stage_to_webgl2(
            &module,
            &info,
            naga::ShaderStage::Vertex,
            "vs_main",
        )?,
        fragment: translate_naga_stage_to_webgl2(
            &module,
            &info,
            naga::ShaderStage::Fragment,
            "fs_main",
        )?,
        vertex_attribute_formats,
    })
}

#[cfg(any(test, target_arch = "wasm32"))]
fn translate_naga_stage_to_webgl2(
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
    stage: naga::ShaderStage,
    entry_point: &str,
) -> Result<WebGl2TranslatedStage, RendererError> {
    let invalid = |message: String| RendererError::InvalidGpuCanvas(message);
    let options = naga::back::glsl::Options {
        version: naga::back::glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        writer_flags: naga::back::glsl::WriterFlags::ADJUST_COORDINATE_SPACE,
        binding_map: naga::back::glsl::BindingMap::default(),
        zero_initialize_workgroup_memory: true,
    };
    let pipeline_options = naga::back::glsl::PipelineOptions {
        shader_stage: stage,
        entry_point: entry_point.into(),
        multiview: None,
    };
    let mut translated = String::new();
    let mut writer = naga::back::glsl::Writer::new(
        &mut translated,
        module,
        info,
        &options,
        &pipeline_options,
        naga::proc::BoundsCheckPolicies::default(),
    )
    .map_err(|error| invalid(format!("WebGL2 {entry_point} translation failed: {error}")))?;
    let reflection = writer
        .write()
        .map_err(|error| invalid(format!("WebGL2 {entry_point} emission failed: {error}")))?;
    let mut uniform_blocks = reflection
        .uniforms
        .into_iter()
        .filter_map(|(handle, name)| {
            module.global_variables[handle]
                .binding
                .as_ref()
                .map(|binding| WebGl2UniformBlock {
                    group: binding.group,
                    binding: binding.binding,
                    name,
                })
        })
        .collect::<Vec<_>>();
    uniform_blocks.sort_by(|left, right| {
        (left.group, left.binding, &left.name).cmp(&(right.group, right.binding, &right.name))
    });
    Ok(WebGl2TranslatedStage {
        source: translated,
        uniform_blocks,
    })
}

#[cfg(any(test, target_arch = "wasm32"))]
fn collect_io_locations(
    module: &naga::Module,
    binding: Option<&naga::Binding>,
    ty: naga::Handle<naga::Type>,
    locations: &mut Vec<(u32, naga::Handle<naga::Type>)>,
    builtin_count: &mut usize,
) {
    match binding {
        Some(naga::Binding::Location { location, .. }) => locations.push((*location, ty)),
        Some(naga::Binding::BuiltIn(_)) => *builtin_count += 1,
        None => {
            if let naga::TypeInner::Struct { members, .. } = &module.types[ty].inner {
                for member in members {
                    collect_io_locations(
                        module,
                        member.binding.as_ref(),
                        member.ty,
                        locations,
                        builtin_count,
                    );
                }
            }
        }
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn naga_vertex_format(
    module: &naga::Module,
    ty: naga::Handle<naga::Type>,
) -> Result<&'static str, RendererError> {
    let scalar_format = |scalar: naga::Scalar, suffix: &'static str| {
        if scalar.kind == naga::ScalarKind::Float && scalar.width == 4 {
            Ok(suffix)
        } else {
            Err(RendererError::InvalidGpuCanvas(
                "WebGL2 GPU-canvas vertex attributes and fragment output must use float32".into(),
            ))
        }
    };
    match module.types[ty].inner {
        naga::TypeInner::Scalar(scalar) => scalar_format(scalar, "float32"),
        naga::TypeInner::Vector { size, scalar } => scalar_format(
            scalar,
            match size {
                naga::VectorSize::Bi => "float32x2",
                naga::VectorSize::Tri => "float32x3",
                naga::VectorSize::Quad => "float32x4",
            },
        ),
        _ => Err(RendererError::InvalidGpuCanvas(
            "WebGL2 GPU-canvas vertex attributes and fragment output must be float32 scalars or vectors"
                .into(),
        )),
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn validate_webgl2_vertex_attributes(
    expected: &std::collections::BTreeMap<u32, String>,
    plan: &GpuCanvasRenderPlan,
) -> Result<(), RendererError> {
    let provided = plan
        .vertex_layouts
        .iter()
        .flat_map(|layout| &layout.attributes)
        .map(|attribute| (attribute.shader_location, attribute.format.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (location, format) in expected {
        match provided.get(location) {
            Some(provided_format) if *provided_format == format.as_str() => {}
            Some(provided_format) => {
                return Err(RendererError::InvalidGpuCanvas(format!(
                    "vertex attribute location {location} requires {format}, but the draw plan supplies {provided_format}"
                )));
            }
            None => {
                return Err(RendererError::InvalidGpuCanvas(format!(
                    "vertex attribute location {location} ({format}) is missing from the draw plan"
                )));
            }
        }
    }
    Ok(())
}

struct CanonicalWgpuStage {
    source: String,
    module: naga::Module,
    #[cfg(any(test, target_arch = "wasm32"))]
    info: naga::valid::ModuleInfo,
}

struct PreparedImportedGpuCanvas {
    vertex: CanonicalWgpuStage,
    fragment: CanonicalWgpuStage,
    uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ImportedGpuCanvasPipelineKey {
    shader: GpuCanvasShader,
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
    vertex: bool,
    fragment: bool,
}

impl ImportedUniformRequirement {
    fn visibility(self) -> wgpu::ShaderStages {
        let mut stages = wgpu::ShaderStages::empty();
        if self.vertex {
            stages |= wgpu::ShaderStages::VERTEX;
        }
        if self.fragment {
            stages |= wgpu::ShaderStages::FRAGMENT;
        }
        stages
    }
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
    let vertex = canonical_glsl_es300_to_wgsl(&shader.vertex.source, true)?;
    let fragment = canonical_glsl_es300_to_wgsl(&shader.fragment.source, false)?;
    let uniform_requirements =
        validate_imported_interface(shader, plan, &vertex.module, &fragment.module)?;
    Ok(PreparedImportedGpuCanvas {
        vertex,
        fragment,
        uniform_requirements,
    })
}

fn validate_imported_interface(
    shader: &GpuCanvasShader,
    plan: &GpuCanvasPlan,
    vertex_module: &naga::Module,
    fragment_module: &naga::Module,
) -> Result<BTreeMap<(u32, u32), ImportedUniformRequirement>, GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let vertex_entry = imported_entry_point(
        vertex_module,
        naga::ShaderStage::Vertex,
        &shader.vertex.physical_entry_point,
    )
    .ok_or_else(|| {
        invalid(format!(
            "vertex stage has no physical entry point '{}'",
            shader.vertex.physical_entry_point
        ))
    })?;
    let fragment_entry = imported_entry_point(
        fragment_module,
        naga::ShaderStage::Fragment,
        &shader.fragment.physical_entry_point,
    )
    .ok_or_else(|| {
        invalid(format!(
            "fragment stage has no physical entry point '{}'",
            shader.fragment.physical_entry_point
        ))
    })?;

    let vertex_inputs = imported_function_inputs(vertex_module, &vertex_entry.function)?;
    let vertex_outputs = imported_function_output(vertex_module, &vertex_entry.function)?;
    let fragment_inputs = imported_function_inputs(fragment_module, &fragment_entry.function)?;
    let fragment_outputs = imported_function_output(fragment_module, &fragment_entry.function)?;

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

    let mut required_uniforms = imported_uniform_requirements(vertex_module)?
        .into_iter()
        .map(|(binding, required_size)| {
            (
                binding,
                ImportedUniformRequirement {
                    required_size,
                    vertex: true,
                    fragment: false,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (binding, required_size) in imported_uniform_requirements(fragment_module)? {
        required_uniforms
            .entry(binding)
            .and_modify(|requirement| {
                requirement.required_size = requirement.required_size.max(required_size);
                requirement.fragment = true;
            })
            .or_insert(ImportedUniformRequirement {
                required_size,
                vertex: false,
                fragment: true,
            });
    }
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
    module: &naga::Module,
) -> Result<BTreeMap<(u32, u32), u32>, GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let mut layouter = naga::proc::Layouter::default();
    layouter
        .update(module.to_ctx())
        .map_err(|error| invalid(format!("uniform layout failed: {error}")))?;
    let mut uniforms = BTreeMap::new();
    for (_, global) in module.global_variables.iter() {
        match global.space {
            naga::AddressSpace::Private => {}
            naga::AddressSpace::Uniform => {
                let binding = global
                    .binding
                    .as_ref()
                    .ok_or_else(|| invalid("uniform global has no group and binding".into()))?;
                let key = (binding.group, binding.binding);
                if uniforms.insert(key, layouter[global.ty].size).is_some() {
                    return Err(invalid(format!(
                        "uniform group {} binding {} appears more than once",
                        key.0, key.1
                    )));
                }
            }
            ref unsupported => {
                return Err(invalid(format!(
                    "global address space {unsupported:?} is outside the uniform-only imported contract"
                )));
            }
        }
    }
    Ok(uniforms)
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

    for (label, count) in [
        (
            "vertex",
            uniform_requirements
                .values()
                .filter(|requirement| requirement.vertex)
                .count(),
        ),
        (
            "fragment",
            uniform_requirements
                .values()
                .filter(|requirement| requirement.fragment)
                .count(),
        ),
    ] {
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
    /// Execute canonical RSTB GLSL on the retained device and return the
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
            let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nuxie-imported-gpu-canvas-vertex"),
                source: wgpu::ShaderSource::Wgsl(prepared.vertex.source.into()),
            });
            let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nuxie-imported-gpu-canvas-fragment"),
                source: wgpu::ShaderSource::Wgsl(prepared.fragment.source.into()),
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
                    module: &vertex_shader,
                    entry_point: Some(&shader.vertex.physical_entry_point),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &vertex_layouts,
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: Some(&shader.fragment.physical_entry_point),
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
                    "wgpu rejected imported GPU-canvas GLSL: {error}"
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

/// Retained WebGL2 context isolated from femtovg's renderer state.
///
/// One instance belongs to one [`WebGl2Factory`](super::WebGl2Factory) and is
/// reused for every imported GPU-canvas image update. Shader programs and draw
/// resources remain retained between updates alongside the scarce browser
/// context/device.
#[cfg(any(test, target_arch = "wasm32"))]
fn prepare_imported_gpu_canvas_webgl2(
    shader: &GpuCanvasShader,
    plan: &GpuCanvasPlan,
) -> Result<WebGl2TranslatedProgram, GpuCanvasError> {
    let prepared = prepare_imported_gpu_canvas(shader, plan)?;
    let translate = |stage: &CanonicalWgpuStage, shader_stage, entry_point: &str| {
        translate_naga_stage_to_webgl2(&stage.module, &stage.info, shader_stage, entry_point)
            .map_err(|error| GpuCanvasError::new(error.to_string()))
    };
    let vertex_attribute_formats = plan
        .vertex_layouts
        .iter()
        .flat_map(|layout| &layout.attributes)
        .map(|attribute| (attribute.shader_location, attribute.format.clone()))
        .collect();
    Ok(WebGl2TranslatedProgram {
        vertex: translate(
            &prepared.vertex,
            naga::ShaderStage::Vertex,
            &shader.vertex.physical_entry_point,
        )?,
        fragment: translate(
            &prepared.fragment,
            naga::ShaderStage::Fragment,
            &shader.fragment.physical_entry_point,
        )?,
        vertex_attribute_formats,
    })
}

#[cfg(target_arch = "wasm32")]
pub(super) struct ImportedWebGl2GpuCanvasRenderer {
    element: web_sys::HtmlCanvasElement,
    gl: web_sys::WebGl2RenderingContext,
    pipelines: Vec<ImportedWebGl2GpuCanvasPipeline>,
    buffer_bytes: usize,
}

#[cfg(target_arch = "wasm32")]
impl ImportedWebGl2GpuCanvasRenderer {
    pub(super) fn new(owner: &web_sys::HtmlCanvasElement) -> Result<Self, GpuCanvasError> {
        use wasm_bindgen::JsCast as _;

        let document = owner
            .owner_document()
            .ok_or_else(|| GpuCanvasError::new("browser canvas has no owner document"))?;
        let element = document
            .create_element("canvas")
            .map_err(|error| {
                GpuCanvasError::new(webgl2_js_error("offscreen canvas", error).to_string())
            })?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| GpuCanvasError::new("browser created a non-canvas element"))?;
        let attributes = js_sys::Object::new();
        for (name, value) in [
            ("alpha", wasm_bindgen::JsValue::TRUE),
            ("antialias", wasm_bindgen::JsValue::FALSE),
            ("depth", wasm_bindgen::JsValue::FALSE),
            ("premultipliedAlpha", wasm_bindgen::JsValue::FALSE),
            ("preserveDrawingBuffer", wasm_bindgen::JsValue::TRUE),
            ("stencil", wasm_bindgen::JsValue::FALSE),
        ] {
            js_sys::Reflect::set(&attributes, &wasm_bindgen::JsValue::from_str(name), &value)
                .map_err(|error| {
                    GpuCanvasError::new(webgl2_js_error("context attributes", error).to_string())
                })?;
        }
        let context = element
            .get_context_with_context_options("webgl2", attributes.as_ref())
            .map_err(|error| {
                GpuCanvasError::new(webgl2_js_error("context creation", error).to_string())
            })?
            .ok_or_else(|| GpuCanvasError::new("WebGL2 is unavailable"))?;
        let gl = context
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|_| GpuCanvasError::new("browser returned a non-WebGL2 context"))?;
        Ok(Self {
            element,
            gl,
            pipelines: Vec::new(),
            buffer_bytes: 0,
        })
    }

    fn insert_pipeline(&mut self, pipeline: ImportedWebGl2GpuCanvasPipeline) -> usize {
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

    /// Execute one canonical RSTB draw and return top-left-origin RGBA for
    /// upload into the factory's ordinary image domain. Shader translation,
    /// program linkage, VAO setup, and buffer allocation are retained while
    /// animated uniform/vertex bytes are updated in place.
    pub(super) fn render(
        &mut self,
        shader: &GpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Result<Vec<u8>, GpuCanvasError> {
        use web_sys::WebGl2RenderingContext as Gl;

        let fail = |error: RendererError| GpuCanvasError::new(error.to_string());
        validate_imported_gpu_canvas_plan(plan).map_err(fail)?;
        let key = ImportedGpuCanvasPipelineKey::new(shader, plan);
        let mut pipeline_index = self
            .pipelines
            .iter()
            .position(|pipeline| pipeline.key == key);
        if pipeline_index.is_none() {
            let pipeline = build_imported_webgl2_pipeline(&self.gl, shader, plan, key)?;
            pipeline_index = Some(self.insert_pipeline(pipeline));
        }
        if (self.element.width(), self.element.height()) != (plan.width, plan.height) {
            self.element.set_width(plan.width);
            self.element.set_height(plan.height);
        }
        let gl = &self.gl;
        let pipeline = self
            .pipelines
            .get(pipeline_index.expect("imported WebGL2 pipeline index exists"))
            .expect("imported WebGL2 pipeline was initialized");

        for _ in 0..16 {
            if gl.get_error() == Gl::NO_ERROR {
                break;
            }
        }
        gl.use_program(Some(&pipeline.program));
        gl.viewport(0, 0, plan.width as i32, plan.height as i32);
        gl.disable(Gl::BLEND);
        gl.disable(Gl::CULL_FACE);
        gl.disable(Gl::DEPTH_TEST);
        gl.disable(Gl::SCISSOR_TEST);
        gl.disable(Gl::STENCIL_TEST);
        gl.color_mask(true, true, true, true);
        gl.bind_vertex_array(Some(&pipeline.vertex_array));
        for (binding_point, (buffer, gpu_buffer)) in plan
            .uniform_buffers
            .iter()
            .zip(&pipeline.uniform_buffers)
            .enumerate()
        {
            gl.bind_buffer(Gl::UNIFORM_BUFFER, Some(gpu_buffer));
            gl.buffer_sub_data_with_i32_and_u8_array(Gl::UNIFORM_BUFFER, 0, &buffer.bytes);
            gl.bind_buffer_base(Gl::UNIFORM_BUFFER, binding_point as u32, Some(gpu_buffer));
        }
        for (slot, gpu_buffer) in &pipeline.vertex_buffers {
            let buffer = plan
                .vertex_buffers
                .iter()
                .find(|buffer| buffer.slot == *slot)
                .expect("validated imported GPU-canvas vertex slot remains present");
            gl.bind_buffer(Gl::ARRAY_BUFFER, Some(gpu_buffer));
            gl.buffer_sub_data_with_i32_and_u8_array(Gl::ARRAY_BUFFER, 0, &buffer.bytes);
        }

        if let Some(location) = gl.get_uniform_location(&pipeline.program, "naga_vs_first_instance")
        {
            gl.uniform1ui(Some(&location), plan.first_instance);
        }

        gl.clear_color(
            plan.clear_color[0] as f32,
            plan.clear_color[1] as f32,
            plan.clear_color[2] as f32,
            plan.clear_color[3] as f32,
        );
        gl.clear(Gl::COLOR_BUFFER_BIT);
        gl.draw_arrays_instanced(
            Gl::TRIANGLES,
            plan.first_vertex as i32,
            plan.vertex_count as i32,
            plan.instance_count as i32,
        );
        check_webgl2_error(gl, "imported draw submission").map_err(fail)?;

        let byte_len = usize::try_from(plan.width)
            .ok()
            .and_then(|width| {
                usize::try_from(plan.height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| GpuCanvasError::new("WebGL2 readback byte length overflow"))?;
        let mut pixels = vec![0; byte_len];
        gl.pixel_storei(Gl::PACK_ALIGNMENT, 1);
        gl.finish();
        gl.read_pixels_with_opt_u8_array(
            0,
            0,
            plan.width as i32,
            plan.height as i32,
            Gl::RGBA,
            Gl::UNSIGNED_BYTE,
            Some(&mut pixels),
        )
        .map_err(|error| {
            GpuCanvasError::new(webgl2_js_error("imported RGBA readback", error).to_string())
        })?;
        check_webgl2_error(gl, "imported RGBA readback").map_err(fail)?;
        flip_rgba_rows(&mut pixels, plan.width, plan.height);
        Ok(pixels)
    }
}

#[cfg(target_arch = "wasm32")]
struct ImportedWebGl2GpuCanvasPipeline {
    gl: web_sys::WebGl2RenderingContext,
    key: ImportedGpuCanvasPipelineKey,
    program: web_sys::WebGlProgram,
    vertex_array: web_sys::WebGlVertexArrayObject,
    uniform_buffers: Vec<web_sys::WebGlBuffer>,
    vertex_buffers: Vec<(u32, web_sys::WebGlBuffer)>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for ImportedWebGl2GpuCanvasPipeline {
    fn drop(&mut self) {
        self.gl.use_program(None);
        self.gl.bind_vertex_array(None);
        self.gl
            .bind_buffer(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, None);
        self.gl
            .bind_buffer(web_sys::WebGl2RenderingContext::UNIFORM_BUFFER, None);
        for binding_point in 0..self.uniform_buffers.len() {
            self.gl.bind_buffer_base(
                web_sys::WebGl2RenderingContext::UNIFORM_BUFFER,
                binding_point as u32,
                None,
            );
        }
        for buffer in &self.uniform_buffers {
            self.gl.delete_buffer(Some(buffer));
        }
        for (_, buffer) in &self.vertex_buffers {
            self.gl.delete_buffer(Some(buffer));
        }
        self.gl.delete_vertex_array(Some(&self.vertex_array));
        self.gl.delete_program(Some(&self.program));
    }
}

#[cfg(target_arch = "wasm32")]
struct ImportedWebGl2BuildGuard {
    gl: web_sys::WebGl2RenderingContext,
    program: Option<web_sys::WebGlProgram>,
    vertex_array: Option<web_sys::WebGlVertexArrayObject>,
    uniform_buffers: Vec<web_sys::WebGlBuffer>,
    vertex_buffers: Vec<(u32, web_sys::WebGlBuffer)>,
}

#[cfg(target_arch = "wasm32")]
impl ImportedWebGl2BuildGuard {
    fn new(gl: &web_sys::WebGl2RenderingContext) -> Self {
        Self {
            gl: gl.clone(),
            program: None,
            vertex_array: None,
            uniform_buffers: Vec::new(),
            vertex_buffers: Vec::new(),
        }
    }

    fn finish(mut self, key: ImportedGpuCanvasPipelineKey) -> ImportedWebGl2GpuCanvasPipeline {
        ImportedWebGl2GpuCanvasPipeline {
            gl: self.gl.clone(),
            key,
            program: self
                .program
                .take()
                .expect("linked imported WebGL2 program exists"),
            vertex_array: self
                .vertex_array
                .take()
                .expect("imported WebGL2 vertex array exists"),
            uniform_buffers: std::mem::take(&mut self.uniform_buffers),
            vertex_buffers: std::mem::take(&mut self.vertex_buffers),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for ImportedWebGl2BuildGuard {
    fn drop(&mut self) {
        self.gl.use_program(None);
        self.gl.bind_vertex_array(None);
        self.gl
            .bind_buffer(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, None);
        self.gl
            .bind_buffer(web_sys::WebGl2RenderingContext::UNIFORM_BUFFER, None);
        for binding_point in 0..self.uniform_buffers.len() {
            self.gl.bind_buffer_base(
                web_sys::WebGl2RenderingContext::UNIFORM_BUFFER,
                binding_point as u32,
                None,
            );
        }
        for buffer in &self.uniform_buffers {
            self.gl.delete_buffer(Some(buffer));
        }
        for (_, buffer) in &self.vertex_buffers {
            self.gl.delete_buffer(Some(buffer));
        }
        if let Some(vertex_array) = self.vertex_array.as_ref() {
            self.gl.delete_vertex_array(Some(vertex_array));
        }
        if let Some(program) = self.program.as_ref() {
            self.gl.delete_program(Some(program));
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn build_imported_webgl2_pipeline(
    gl: &web_sys::WebGl2RenderingContext,
    shader: &GpuCanvasShader,
    plan: &GpuCanvasPlan,
    key: ImportedGpuCanvasPipelineKey,
) -> Result<ImportedWebGl2GpuCanvasPipeline, GpuCanvasError> {
    use web_sys::WebGl2RenderingContext as Gl;

    let fail = |error: RendererError| GpuCanvasError::new(error.to_string());
    let translated = prepare_imported_gpu_canvas_webgl2(shader, plan)?;
    let max_uniform_bindings = webgl2_u32_parameter(
        gl,
        Gl::MAX_UNIFORM_BUFFER_BINDINGS,
        "MAX_UNIFORM_BUFFER_BINDINGS",
    )
    .map_err(fail)?;
    if plan.uniform_buffers.len() > max_uniform_bindings as usize {
        return Err(GpuCanvasError::new(format!(
            "draw requires {} uniform bindings but this WebGL2 context supports {max_uniform_bindings}",
            plan.uniform_buffers.len()
        )));
    }
    let max_uniform_block_size =
        webgl2_u32_parameter(gl, Gl::MAX_UNIFORM_BLOCK_SIZE, "MAX_UNIFORM_BLOCK_SIZE")
            .map_err(fail)? as usize;
    if let Some(buffer) = plan
        .uniform_buffers
        .iter()
        .find(|buffer| buffer.bytes.len() > max_uniform_block_size)
    {
        return Err(GpuCanvasError::new(format!(
            "uniform binding {} in group {} contains {} bytes but this WebGL2 context supports {max_uniform_block_size}",
            buffer.binding,
            buffer.group,
            buffer.bytes.len()
        )));
    }

    let vertex = compile_webgl2_shader(
        gl,
        Gl::VERTEX_SHADER,
        &translated.vertex.source,
        "RSTB vertex",
    )
    .map_err(fail)?;
    let fragment = match compile_webgl2_shader(
        gl,
        Gl::FRAGMENT_SHADER,
        &translated.fragment.source,
        "RSTB fragment",
    ) {
        Ok(fragment) => fragment,
        Err(error) => {
            gl.delete_shader(Some(&vertex));
            return Err(fail(error));
        }
    };
    let Some(program) = gl.create_program() else {
        gl.delete_shader(Some(&vertex));
        gl.delete_shader(Some(&fragment));
        return Err(GpuCanvasError::new(
            "failed to allocate RSTB shader program",
        ));
    };
    gl.attach_shader(&program, &vertex);
    gl.attach_shader(&program, &fragment);
    gl.link_program(&program);
    let linked = gl
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false);
    let link_log = gl.get_program_info_log(&program).unwrap_or_default();
    gl.detach_shader(&program, &vertex);
    gl.detach_shader(&program, &fragment);
    gl.delete_shader(Some(&vertex));
    gl.delete_shader(Some(&fragment));
    if !linked {
        gl.delete_program(Some(&program));
        return Err(GpuCanvasError::new(format!(
            "RSTB shader program failed to link: {link_log}"
        )));
    }

    let mut guard = ImportedWebGl2BuildGuard::new(gl);
    guard.program = Some(program);
    let program = guard
        .program
        .as_ref()
        .expect("linked imported WebGL2 program exists");
    gl.use_program(Some(program));
    let vertex_array = gl
        .create_vertex_array()
        .ok_or_else(|| GpuCanvasError::new("failed to allocate vertex array"))?;
    gl.bind_vertex_array(Some(&vertex_array));
    guard.vertex_array = Some(vertex_array);

    let binding_points = plan
        .uniform_buffers
        .iter()
        .enumerate()
        .map(|(index, buffer)| ((buffer.group, buffer.binding), index as u32))
        .collect::<BTreeMap<_, _>>();
    let mut uniform_blocks = translated.vertex.uniform_blocks;
    uniform_blocks.extend(translated.fragment.uniform_blocks);
    uniform_blocks.sort_by(|left, right| {
        (left.group, left.binding, &left.name).cmp(&(right.group, right.binding, &right.name))
    });
    uniform_blocks.dedup();
    for block in &uniform_blocks {
        let binding_point = binding_points
            .get(&(block.group, block.binding))
            .ok_or_else(|| {
                GpuCanvasError::new(format!(
                    "shader uniform binding {} in group {} is not supplied by the draw plan",
                    block.binding, block.group
                ))
            })?;
        let block_index = gl.get_uniform_block_index(program, &block.name);
        if block_index == Gl::INVALID_INDEX {
            return Err(GpuCanvasError::new(format!(
                "linked program omitted reflected uniform block '{}'",
                block.name
            )));
        }
        gl.uniform_block_binding(program, block_index, *binding_point);
    }
    for (binding_point, buffer) in plan.uniform_buffers.iter().enumerate() {
        let gpu_buffer = gl
            .create_buffer()
            .ok_or_else(|| GpuCanvasError::new("failed to allocate uniform buffer"))?;
        gl.bind_buffer(Gl::UNIFORM_BUFFER, Some(&gpu_buffer));
        gl.buffer_data_with_u8_array(Gl::UNIFORM_BUFFER, &buffer.bytes, Gl::DYNAMIC_DRAW);
        gl.bind_buffer_base(Gl::UNIFORM_BUFFER, binding_point as u32, Some(&gpu_buffer));
        guard.uniform_buffers.push(gpu_buffer);
    }
    for (slot, layout) in plan.vertex_layouts.iter().enumerate() {
        let buffer = plan
            .vertex_buffers
            .iter()
            .find(|buffer| buffer.slot == slot as u32)
            .ok_or_else(|| {
                GpuCanvasError::new(format!("vertex buffer slot {slot} is not bound"))
            })?;
        let gpu_buffer = gl
            .create_buffer()
            .ok_or_else(|| GpuCanvasError::new("failed to allocate vertex buffer"))?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&gpu_buffer));
        gl.buffer_data_with_u8_array(Gl::ARRAY_BUFFER, &buffer.bytes, Gl::DYNAMIC_DRAW);
        for attribute in &layout.attributes {
            let components = webgl2_vertex_component_count(&attribute.format).map_err(fail)?;
            gl.enable_vertex_attrib_array(attribute.shader_location);
            gl.vertex_attrib_pointer_with_i32(
                attribute.shader_location,
                components,
                Gl::FLOAT,
                false,
                layout.stride as i32,
                attribute.offset as i32,
            );
            gl.vertex_attrib_divisor(attribute.shader_location, 0);
        }
        guard.vertex_buffers.push((slot as u32, gpu_buffer));
    }
    check_webgl2_error(gl, "imported pipeline preparation").map_err(fail)?;
    Ok(guard.finish(key))
}

/// RSTB target 1 contains the producer's canonical GLSL ES 3.00. Naga's
/// native GLSL frontend accepts Vulkan-flavoured desktop GLSL, so the WGPU
/// backend performs this narrow, generated-source normalization internally.
/// No scripting/runtime layer sees a backend shader dialect.
fn canonical_glsl_es300_to_wgsl(
    source: &str,
    vertex: bool,
) -> Result<CanonicalWgpuStage, GpuCanvasError> {
    let mut lines = source.lines();
    let version = lines.next().unwrap_or_default().trim();
    if version != "#version 300 es" {
        return Err(GpuCanvasError::new(
            "canonical RSTB GLSL must begin with '#version 300 es'",
        ));
    }
    let mut normalized = String::from("#version 450 core\n");
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("precision ")
            || (vertex
                && trimmed
                    == "gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);")
        {
            continue;
        }
        normalized.push_str(line);
        normalized.push('\n');
    }
    if vertex {
        normalized = normalized
            .replace("gl_VertexID", "int(gl_VertexIndex)")
            .replace("gl_InstanceID", "int(gl_InstanceIndex)");
    }
    let stage = if vertex {
        naga::ShaderStage::Vertex
    } else {
        naga::ShaderStage::Fragment
    };
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&naga::front::glsl::Options::from(stage), &normalized)
        .map_err(|error| GpuCanvasError::new(error.emit_to_string(&normalized)))?;
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| GpuCanvasError::new(error.emit_to_string(&normalized)))?;
    let source = naga::back::wgsl::write_string(
        &module,
        &info,
        naga::back::wgsl::WriterFlags::EXPLICIT_TYPES,
    )
    .map_err(|error| GpuCanvasError::new(format!("RSTB GLSL to WGSL emission failed: {error}")))?;
    let module = naga::front::wgsl::parse_str(&source)
        .map_err(|error| GpuCanvasError::new(error.emit_to_string(&source)))?;
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| GpuCanvasError::new(error.emit_to_string(&source)))?;
    #[cfg(not(any(test, target_arch = "wasm32")))]
    let _ = info;
    Ok(CanonicalWgpuStage {
        source,
        module,
        #[cfg(any(test, target_arch = "wasm32"))]
        info,
    })
}

/// Retained arbitrary-WGSL renderer for browsers without WebGPU.
///
/// WGSL translation, reflection, resource uploads, draw submission, and
/// readback all remain in Rust/Wasm. The HTML canvas is only the WebGL2 render
/// target; JavaScript never compiles shaders or interprets the draw plan.
#[cfg(target_arch = "wasm32")]
pub struct WebGl2GpuCanvasRenderer {
    element: web_sys::HtmlCanvasElement,
    gl: web_sys::WebGl2RenderingContext,
    program: web_sys::WebGlProgram,
    shader_wgsl: String,
    uniform_blocks: Vec<WebGl2UniformBlock>,
    vertex_attribute_formats: std::collections::BTreeMap<u32, String>,
}

#[cfg(target_arch = "wasm32")]
impl WebGl2GpuCanvasRenderer {
    pub fn new(
        element: &web_sys::HtmlCanvasElement,
        shader_wgsl: &str,
    ) -> Result<Self, RendererError> {
        use wasm_bindgen::{JsCast as _, JsValue};

        let translated = translate_gpu_canvas_wgsl_to_webgl2(shader_wgsl)?;
        let attributes = js_sys::Object::new();
        for (name, value) in [
            ("alpha", JsValue::TRUE),
            ("antialias", JsValue::FALSE),
            ("depth", JsValue::FALSE),
            ("premultipliedAlpha", JsValue::FALSE),
            ("preserveDrawingBuffer", JsValue::TRUE),
            ("stencil", JsValue::FALSE),
        ] {
            js_sys::Reflect::set(&attributes, &JsValue::from_str(name), &value)
                .map_err(|error| webgl2_js_error("context attributes", error))?;
        }
        let context = element
            .get_context_with_context_options("webgl2", attributes.as_ref())
            .map_err(|error| webgl2_js_error("context creation", error))?
            .ok_or_else(|| RendererError::WebGl2("WebGL2 is unavailable".into()))?;
        let gl = context
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|_| RendererError::WebGl2("browser returned a non-WebGL2 context".into()))?;

        let vertex = compile_webgl2_shader(
            &gl,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            &translated.vertex.source,
            "vertex",
        )?;
        let fragment = match compile_webgl2_shader(
            &gl,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            &translated.fragment.source,
            "fragment",
        ) {
            Ok(fragment) => fragment,
            Err(error) => {
                gl.delete_shader(Some(&vertex));
                return Err(error);
            }
        };
        let program = gl
            .create_program()
            .ok_or_else(|| RendererError::WebGl2("failed to allocate shader program".into()))?;
        gl.attach_shader(&program, &vertex);
        gl.attach_shader(&program, &fragment);
        gl.link_program(&program);
        let linked = gl
            .get_program_parameter(&program, web_sys::WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false);
        let link_log = gl.get_program_info_log(&program).unwrap_or_default();
        gl.detach_shader(&program, &vertex);
        gl.detach_shader(&program, &fragment);
        gl.delete_shader(Some(&vertex));
        gl.delete_shader(Some(&fragment));
        if !linked {
            gl.delete_program(Some(&program));
            return Err(RendererError::WebGl2(format!(
                "WGSL-translated shader program failed to link: {link_log}"
            )));
        }

        let mut uniform_blocks = translated.vertex.uniform_blocks;
        uniform_blocks.extend(translated.fragment.uniform_blocks);
        uniform_blocks.sort_by(|left, right| {
            (left.group, left.binding, &left.name).cmp(&(right.group, right.binding, &right.name))
        });
        Ok(Self {
            element: element.clone(),
            gl,
            program,
            shader_wgsl: shader_wgsl.into(),
            uniform_blocks,
            vertex_attribute_formats: translated.vertex_attribute_formats,
        })
    }

    pub fn render_gpu_canvas(&self, plan: &GpuCanvasRenderPlan) -> Result<Vec<u8>, RendererError> {
        use web_sys::WebGl2RenderingContext as Gl;

        validate_gpu_canvas_plan(plan)?;
        validate_webgl2_vertex_attributes(&self.vertex_attribute_formats, plan)?;
        if plan.shader_wgsl != self.shader_wgsl {
            return Err(RendererError::InvalidGpuCanvas(
                "WebGL2 renderer cannot switch shader source after creation".into(),
            ));
        }
        self.element.set_width(plan.width);
        self.element.set_height(plan.height);
        let gl = &self.gl;
        for _ in 0..16 {
            if gl.get_error() == Gl::NO_ERROR {
                break;
            }
        }
        gl.use_program(Some(&self.program));
        gl.viewport(0, 0, plan.width as i32, plan.height as i32);
        gl.disable(Gl::BLEND);
        gl.disable(Gl::CULL_FACE);
        gl.disable(Gl::DEPTH_TEST);
        gl.disable(Gl::SCISSOR_TEST);
        gl.disable(Gl::STENCIL_TEST);
        gl.color_mask(true, true, true, true);

        let max_uniform_bindings = webgl2_u32_parameter(
            gl,
            Gl::MAX_UNIFORM_BUFFER_BINDINGS,
            "MAX_UNIFORM_BUFFER_BINDINGS",
        )?;
        if plan.uniform_buffers.len() > max_uniform_bindings as usize {
            return Err(RendererError::WebGl2(format!(
                "draw requires {} uniform bindings but this WebGL2 context supports {max_uniform_bindings}",
                plan.uniform_buffers.len()
            )));
        }
        let max_uniform_block_size =
            webgl2_u32_parameter(gl, Gl::MAX_UNIFORM_BLOCK_SIZE, "MAX_UNIFORM_BLOCK_SIZE")?
                as usize;
        if let Some(buffer) = plan
            .uniform_buffers
            .iter()
            .find(|buffer| buffer.bytes.len() > max_uniform_block_size)
        {
            return Err(RendererError::WebGl2(format!(
                "uniform binding {} in group {} contains {} bytes but this WebGL2 context supports {max_uniform_block_size}",
                buffer.binding,
                buffer.group,
                buffer.bytes.len()
            )));
        }

        let mut resources = WebGl2GpuCanvasFrameResources::new(gl)?;
        let binding_points = plan
            .uniform_buffers
            .iter()
            .enumerate()
            .map(|(index, buffer)| ((buffer.group, buffer.binding), index as u32))
            .collect::<std::collections::BTreeMap<_, _>>();
        for block in &self.uniform_blocks {
            let binding_point = binding_points
                .get(&(block.group, block.binding))
                .ok_or_else(|| {
                    RendererError::InvalidGpuCanvas(format!(
                        "shader uniform binding {} in group {} is not supplied by the draw plan",
                        block.binding, block.group
                    ))
                })?;
            let block_index = gl.get_uniform_block_index(&self.program, &block.name);
            if block_index == Gl::INVALID_INDEX {
                return Err(RendererError::WebGl2(format!(
                    "linked program omitted reflected uniform block '{}'",
                    block.name
                )));
            }
            gl.uniform_block_binding(&self.program, block_index, *binding_point);
        }
        for (binding_point, buffer) in plan.uniform_buffers.iter().enumerate() {
            let gpu_buffer = gl
                .create_buffer()
                .ok_or_else(|| RendererError::WebGl2("failed to allocate uniform buffer".into()))?;
            gl.bind_buffer(Gl::UNIFORM_BUFFER, Some(&gpu_buffer));
            gl.buffer_data_with_u8_array(Gl::UNIFORM_BUFFER, &buffer.bytes, Gl::DYNAMIC_DRAW);
            gl.bind_buffer_base(Gl::UNIFORM_BUFFER, binding_point as u32, Some(&gpu_buffer));
            resources.uniform_binding_points.push(binding_point as u32);
            resources.buffers.push(gpu_buffer);
        }

        gl.bind_vertex_array(Some(&resources.vertex_array));
        for (slot, layout) in plan.vertex_layouts.iter().enumerate() {
            let buffer = plan
                .vertex_buffers
                .iter()
                .find(|buffer| buffer.slot == slot as u32)
                .ok_or_else(|| {
                    RendererError::InvalidGpuCanvas(format!(
                        "vertex buffer slot {slot} is not bound"
                    ))
                })?;
            let gpu_buffer = gl
                .create_buffer()
                .ok_or_else(|| RendererError::WebGl2("failed to allocate vertex buffer".into()))?;
            gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&gpu_buffer));
            gl.buffer_data_with_u8_array(Gl::ARRAY_BUFFER, &buffer.bytes, Gl::STATIC_DRAW);
            for attribute in &layout.attributes {
                let component_count = webgl2_vertex_component_count(&attribute.format)?;
                gl.enable_vertex_attrib_array(attribute.shader_location);
                gl.vertex_attrib_pointer_with_i32(
                    attribute.shader_location,
                    component_count,
                    Gl::FLOAT,
                    false,
                    layout.stride as i32,
                    attribute.offset as i32,
                );
                gl.vertex_attrib_divisor(attribute.shader_location, 0);
            }
            resources.buffers.push(gpu_buffer);
        }

        if let Some(location) = gl.get_uniform_location(&self.program, "naga_vs_first_instance") {
            gl.uniform1ui(Some(&location), plan.first_instance);
        }

        gl.clear_color(
            plan.clear_color[0] as f32,
            plan.clear_color[1] as f32,
            plan.clear_color[2] as f32,
            plan.clear_color[3] as f32,
        );
        gl.clear(Gl::COLOR_BUFFER_BIT);
        gl.draw_arrays_instanced(
            Gl::TRIANGLES,
            plan.first_vertex as i32,
            plan.vertex_count as i32,
            plan.instance_count as i32,
        );
        check_webgl2_error(gl, "draw submission")?;

        let pixel_bytes = usize::try_from(plan.width)
            .ok()
            .and_then(|width| {
                usize::try_from(plan.height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| RendererError::WebGl2("readback byte length overflow".into()))?;
        let mut pixels = vec![0; pixel_bytes];
        gl.pixel_storei(Gl::PACK_ALIGNMENT, 1);
        gl.finish();
        gl.read_pixels_with_opt_u8_array(
            0,
            0,
            plan.width as i32,
            plan.height as i32,
            Gl::RGBA,
            Gl::UNSIGNED_BYTE,
            Some(&mut pixels),
        )
        .map_err(|error| webgl2_js_error("RGBA readback", error))?;
        check_webgl2_error(gl, "RGBA readback")?;
        flip_rgba_rows(&mut pixels, plan.width, plan.height);
        Ok(pixels)
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for WebGl2GpuCanvasRenderer {
    fn drop(&mut self) {
        self.gl.delete_program(Some(&self.program));
    }
}

#[cfg(target_arch = "wasm32")]
struct WebGl2GpuCanvasFrameResources<'a> {
    gl: &'a web_sys::WebGl2RenderingContext,
    vertex_array: web_sys::WebGlVertexArrayObject,
    buffers: Vec<web_sys::WebGlBuffer>,
    uniform_binding_points: Vec<u32>,
}

#[cfg(target_arch = "wasm32")]
impl<'a> WebGl2GpuCanvasFrameResources<'a> {
    fn new(gl: &'a web_sys::WebGl2RenderingContext) -> Result<Self, RendererError> {
        let vertex_array = gl
            .create_vertex_array()
            .ok_or_else(|| RendererError::WebGl2("failed to allocate vertex array".into()))?;
        Ok(Self {
            gl,
            vertex_array,
            buffers: Vec::new(),
            uniform_binding_points: Vec::new(),
        })
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for WebGl2GpuCanvasFrameResources<'_> {
    fn drop(&mut self) {
        use web_sys::WebGl2RenderingContext as Gl;
        self.gl.bind_vertex_array(None);
        self.gl.bind_buffer(Gl::ARRAY_BUFFER, None);
        self.gl.bind_buffer(Gl::UNIFORM_BUFFER, None);
        for binding_point in self.uniform_binding_points.drain(..) {
            self.gl
                .bind_buffer_base(Gl::UNIFORM_BUFFER, binding_point, None);
        }
        for buffer in self.buffers.drain(..) {
            self.gl.delete_buffer(Some(&buffer));
        }
        self.gl.delete_vertex_array(Some(&self.vertex_array));
    }
}

#[cfg(target_arch = "wasm32")]
fn compile_webgl2_shader(
    gl: &web_sys::WebGl2RenderingContext,
    kind: u32,
    source: &str,
    label: &str,
) -> Result<web_sys::WebGlShader, RendererError> {
    let shader = gl
        .create_shader(kind)
        .ok_or_else(|| RendererError::WebGl2(format!("failed to allocate {label} shader")))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    let compiled = gl
        .get_shader_parameter(&shader, web_sys::WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false);
    if compiled {
        Ok(shader)
    } else {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        gl.delete_shader(Some(&shader));
        Err(RendererError::WebGl2(format!(
            "WGSL-translated {label} shader failed to compile: {log}"
        )))
    }
}

#[cfg(target_arch = "wasm32")]
fn webgl2_u32_parameter(
    gl: &web_sys::WebGl2RenderingContext,
    parameter: u32,
    label: &str,
) -> Result<u32, RendererError> {
    let value = gl
        .get_parameter(parameter)
        .map_err(|error| webgl2_js_error(label, error))?
        .as_f64()
        .filter(|value| value.is_finite() && *value >= 0.0 && *value <= u32::MAX as f64)
        .ok_or_else(|| RendererError::WebGl2(format!("{label} is not a finite u32")))?;
    Ok(value as u32)
}

#[cfg(target_arch = "wasm32")]
fn webgl2_vertex_component_count(format: &str) -> Result<i32, RendererError> {
    match format {
        "float32" => Ok(1),
        "float32x2" => Ok(2),
        "float32x3" => Ok(3),
        "float32x4" => Ok(4),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "unsupported WebGL2 vertex format '{format}'"
        ))),
    }
}

#[cfg(target_arch = "wasm32")]
fn check_webgl2_error(
    gl: &web_sys::WebGl2RenderingContext,
    operation: &str,
) -> Result<(), RendererError> {
    let code = gl.get_error();
    if code == web_sys::WebGl2RenderingContext::NO_ERROR {
        Ok(())
    } else {
        Err(RendererError::WebGl2(format!(
            "{operation} produced GL error 0x{code:04x}"
        )))
    }
}

#[cfg(target_arch = "wasm32")]
fn webgl2_js_error(operation: &str, value: wasm_bindgen::JsValue) -> RendererError {
    RendererError::WebGl2(format!(
        "{operation} failed: {}",
        value.as_string().unwrap_or_else(|| format!("{value:?}"))
    ))
}

#[cfg(any(test, target_arch = "wasm32"))]
fn flip_rgba_rows(pixels: &mut [u8], width: u32, height: u32) {
    let row_bytes = width as usize * 4;
    for row in 0..height as usize / 2 {
        let opposite = height as usize - row - 1;
        let (before, after) = pixels.split_at_mut(opposite * row_bytes);
        before[row * row_bytes..(row + 1) * row_bytes].swap_with_slice(&mut after[..row_bytes]);
    }
}

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

    const IMPORTED_VERTEX: &str = r#"#version 300 es
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

    const IMPORTED_FRAGMENT: &str = r#"#version 300 es
precision highp float;
layout(location = 0) out vec4 color;
void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }
"#;

    fn imported_shader(vertex: &str, fragment: &str) -> GpuCanvasShader {
        nuxie_render_api::GpuCanvasShader {
            vertex: nuxie_render_api::GpuCanvasShaderStage {
                source: vertex.into(),
                logical_entry_point: "vs_main".into(),
                physical_entry_point: "main".into(),
            },
            fragment: nuxie_render_api::GpuCanvasShaderStage {
                source: fragment.into(),
                logical_entry_point: "fs_main".into(),
                physical_entry_point: "main".into(),
            },
        }
    }

    fn imported_plan() -> GpuCanvasPlan {
        GpuCanvasPlan {
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
        let shader = imported_shader(IMPORTED_VERTEX, IMPORTED_FRAGMENT);
        prepare_imported_gpu_canvas(&shader, &imported_plan()).unwrap();

        let vertex_with_input = IMPORTED_VERTEX.replace(
            "void main() {",
            "layout(location = 0) in vec2 position;\nvoid main() {",
        );
        let error = prepare_imported_gpu_canvas(
            &imported_shader(&vertex_with_input, IMPORTED_FRAGMENT),
            &imported_plan(),
        )
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
        prepare_imported_gpu_canvas(
            &imported_shader(&vertex_with_input, IMPORTED_FRAGMENT),
            &matching_plan,
        )
        .unwrap();
        matching_plan.vertex_layouts[0].attributes[0].format = "float32x3".into();
        matching_plan.vertex_layouts[0].stride = 12;
        matching_plan.vertex_buffers[0].bytes.resize(36, 0);
        let error = prepare_imported_gpu_canvas(
            &imported_shader(&vertex_with_input, IMPORTED_FRAGMENT),
            &matching_plan,
        )
        .err()
        .expect("wrong vertex format must fail");
        assert!(error.to_string().contains("vertex inputs"), "{error}");

        let fragment_vec3 = IMPORTED_FRAGMENT
            .replace("out vec4 color", "out vec3 color")
            .replace("vec4(1.0, 0.0, 0.0, 1.0)", "vec3(1.0, 0.0, 0.0)");
        let error = prepare_imported_gpu_canvas(
            &imported_shader(IMPORTED_VERTEX, &fragment_vec3),
            &imported_plan(),
        )
        .err()
        .expect("non-RGBA fragment output must fail");
        assert!(error.to_string().contains("fragment output"), "{error}");

        let mut wrong_entry = shader;
        wrong_entry.vertex.physical_entry_point = "vs_main".into();
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
        let varying_vertex = IMPORTED_VERTEX.replace(
            "void main() {",
            "layout(location = 0) out vec2 varying_value;\nvoid main() {\n    varying_value = vec2(1.0);",
        );
        let varying_fragment = IMPORTED_FRAGMENT.replace(
            "void main() {",
            "layout(location = 0) in vec3 varying_value;\nvoid main() {",
        );
        let error = prepare_imported_gpu_canvas(
            &imported_shader(&varying_vertex, &varying_fragment),
            &imported_plan(),
        )
        .err()
        .expect("inter-stage type mismatch must fail");
        assert!(error.to_string().contains("inter-stage"), "{error}");

        let uniform_fragment = IMPORTED_FRAGMENT.replace(
            "void main() {",
            "layout(std140, binding = 0) uniform Tint { vec4 value; } tint;\nvoid main() {",
        );
        let error = prepare_imported_gpu_canvas(
            &imported_shader(IMPORTED_VERTEX, &uniform_fragment),
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
            &imported_shader(IMPORTED_VERTEX, &uniform_fragment),
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
        prepare_imported_gpu_canvas(&imported_shader(IMPORTED_VERTEX, &uniform_fragment), &exact)
            .unwrap();
    }

    #[test]
    fn imported_webgl2_translation_supports_uniforms_and_base_instance() {
        let vertex = IMPORTED_VERTEX
            .replace(
                "uint index = uint(gl_VertexID);",
                "uint index = uint(gl_VertexID);\n    uint instance = uint(gl_InstanceID);",
            )
            .replace(
                "float x = float(int(index) - 1);",
                "float x = float(int(index) - 1) + float(instance) * 0.01;",
            );
        let fragment = IMPORTED_FRAGMENT.replace(
            "void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }",
            "layout(std140, binding = 0) uniform Tint { vec4 value; } tint;\nvoid main() { color = tint.value; }",
        );
        let mut plan = imported_plan();
        plan.instance_count = 2;
        plan.first_instance = 4;
        plan.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: 1.0_f32
                .to_le_bytes()
                .into_iter()
                .chain(0.0_f32.to_le_bytes())
                .chain(0.0_f32.to_le_bytes())
                .chain(1.0_f32.to_le_bytes())
                .collect(),
        });

        let translated =
            prepare_imported_gpu_canvas_webgl2(&imported_shader(&vertex, &fragment), &plan)
                .expect("valid imported uniforms and base instance translate to WebGL2");

        assert!(
            translated.vertex.source.contains("naga_vs_first_instance"),
            "Naga must preserve WebGPU firstInstance semantics through a WebGL2 base-instance uniform"
        );
        assert!(
            translated
                .fragment
                .uniform_blocks
                .iter()
                .any(|block| block.group == 0 && block.binding == 0),
            "the imported uniform block remains reflected for WebGL2 binding"
        );
    }

    #[test]
    fn imported_pipeline_key_reuses_resources_for_animated_buffer_bytes() {
        let shader = imported_shader(IMPORTED_VERTEX, IMPORTED_FRAGMENT);
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
            "temporal byte updates must retain translated shaders, pipelines, and buffers"
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
                        vertex: false,
                        fragment: true,
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
                requirement.vertex = index < 7;
                requirement.fragment = index >= 7;
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

    #[test]
    fn webgl2_translation_reflects_cookbook_uniforms_for_both_stages() {
        let shader = r#"
            struct Uniforms { color: vec4<f32> };
            @group(0) @binding(0) var<uniform> u: Uniforms;

            @vertex
            fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
                let x = f32(i32(index) - 1);
                let y = f32(i32(index & 1u) * 2 - 1);
                return vec4<f32>(x, y, 0.0, 1.0);
            }

            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                return u.color;
            }
        "#;

        let translated = translate_gpu_canvas_wgsl_to_webgl2(shader)
            .expect("portable WGSL translates to WebGL2 GLSL ES 3.00");
        assert!(translated.vertex.source.starts_with("#version 300 es"));
        assert!(translated.fragment.source.starts_with("#version 300 es"));
        assert!(translated.vertex_attribute_formats.is_empty());
        assert!(translated.vertex.uniform_blocks.is_empty());
        assert_eq!(
            translated.fragment.uniform_blocks,
            vec![WebGl2UniformBlock {
                group: 0,
                binding: 0,
                name: translated.fragment.uniform_blocks[0].name.clone(),
            }]
        );
        assert!(
            translated
                .fragment
                .source
                .contains(&translated.fragment.uniform_blocks[0].name),
            "reflected block name must be the exact linked-program lookup key"
        );
    }

    #[test]
    fn webgl2_translation_preserves_mesh_vertex_attribute_locations() {
        let shader = r#"
            struct Uniforms { mvp: mat4x4<f32> };
            @group(0) @binding(0) var<uniform> u: Uniforms;
            struct VertexIn {
                @location(0) position: vec3<f32>,
                @location(1) color: vec4<f32>,
            };
            struct VertexOut {
                @builtin(position) position: vec4<f32>,
                @location(0) color: vec4<f32>,
            };
            @vertex fn vs_main(input: VertexIn) -> VertexOut {
                var output: VertexOut;
                output.position = u.mvp * vec4<f32>(input.position, 1.0);
                output.color = input.color;
                return output;
            }
            @fragment fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
                return input.color;
            }
        "#;

        let translated =
            translate_gpu_canvas_wgsl_to_webgl2(shader).expect("mesh WGSL translates to WebGL2");
        assert!(translated.vertex.source.contains("layout(location = 0)"));
        assert!(translated.vertex.source.contains("layout(location = 1)"));
        assert_eq!(translated.vertex.uniform_blocks.len(), 1);
        assert_eq!(translated.vertex.uniform_blocks[0].group, 0);
        assert_eq!(translated.vertex.uniform_blocks[0].binding, 0);
        assert_eq!(
            translated.vertex_attribute_formats,
            std::collections::BTreeMap::from([(0, "float32x3".into()), (1, "float32x4".into()),])
        );

        let mut missing_color = valid_plan();
        missing_color.vertex_layouts = vec![GpuCanvasVertexLayout {
            stride: 12,
            attributes: vec![GpuCanvasVertexAttribute {
                shader_location: 0,
                offset: 0,
                format: "float32x3".into(),
            }],
        }];
        assert!(validate_webgl2_vertex_attributes(
            &translated.vertex_attribute_formats,
            &missing_color,
        )
        .unwrap_err()
        .to_string()
        .contains("location 1"));
    }

    #[test]
    fn webgl2_translation_rejects_resources_outside_the_closed_draw_plan() {
        let shader = r#"
            @group(0) @binding(0) var<storage, read> values: array<f32>;
            @vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
                return vec4<f32>(f32(index), values[0], 0.0, 1.0);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0);
            }
        "#;

        let error = translate_gpu_canvas_wgsl_to_webgl2(shader).unwrap_err();
        assert!(error.to_string().contains("uniform buffers"), "{error}");
    }

    #[test]
    fn webgl2_translation_rejects_shader_io_outside_the_rgba_float_plan() {
        let integer_vertex = r#"
            @vertex fn vs_main(@location(0) position: vec2<u32>) -> @builtin(position) vec4<f32> {
                return vec4<f32>(vec2<f32>(position), 0.0, 1.0);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0);
            }
        "#;
        let error = translate_gpu_canvas_wgsl_to_webgl2(integer_vertex).unwrap_err();
        assert!(error.to_string().contains("float32"), "{error}");

        let multiple_targets = r#"
            @vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
                return vec4<f32>(f32(index), 0.0, 0.0, 1.0);
            }
            struct FragmentOut {
                @location(0) color: vec4<f32>,
                @location(1) glow: vec4<f32>,
            };
            @fragment fn fs_main() -> FragmentOut {
                return FragmentOut(vec4<f32>(1.0), vec4<f32>(0.0));
            }
        "#;
        let error = translate_gpu_canvas_wgsl_to_webgl2(multiple_targets).unwrap_err();
        assert!(error.to_string().contains("exactly one"), "{error}");
    }

    #[test]
    fn webgl2_readback_rows_are_normalized_to_webgpu_order() {
        let mut pixels = vec![
            1, 2, 3, 4, 5, 6, 7, 8, // WebGL bottom row
            9, 10, 11, 12, 13, 14, 15, 16, // WebGL top row
        ];
        flip_rgba_rows(&mut pixels, 2, 2);
        assert_eq!(
            pixels,
            vec![9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8]
        );
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
        use nuxie_render_api::{BlendMode, GpuCanvasShaderStage, ImageSampler, Renderer as _};

        let shader = GpuCanvasShader {
            vertex: GpuCanvasShaderStage {
                source: r#"#version 300 es
                    precision highp float;
                    layout(location = 0) in vec2 position;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                        gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
                    }
                "#
                .into(),
                logical_entry_point: "vs_main".into(),
                physical_entry_point: "main".into(),
            },
            fragment: GpuCanvasShaderStage {
                source: r#"#version 300 es
                    precision highp float;
                    layout(std140, binding = 0) uniform Tint { vec4 value; } tint;
                    layout(location = 0) out vec4 color;
                    void main() { color = tint.value; }
                "#
                .into(),
                logical_entry_point: "fs_main".into(),
                physical_entry_point: "main".into(),
            },
        };
        let encode_f32s = |values: &[f32]| {
            values
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<_>>()
        };
        let mut plan = GpuCanvasPlan {
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
            alternate.fragment.source = alternate
                .fragment
                .source
                .replace("color = tint.value;", "color = tint.value.bgra;");
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
    fn canonical_rstb_glsl_becomes_a_composited_render_image() {
        use nuxie_render_api::{BlendMode, GpuCanvasShaderStage, ImageSampler, Renderer as _};

        let shader = GpuCanvasShader {
            vertex: GpuCanvasShaderStage {
                source: r#"#version 300 es
                    precision highp float;
                    precision highp int;
                    void main() {
                        uint index = uint(gl_VertexID);
                        float x = float(int(index) - 1);
                        float y = float(int(index & 1u) * 2 - 1);
                        gl_Position = vec4(x, y, 0.0, 1.0);
                        gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
                    }
                "#
                .into(),
                logical_entry_point: "vs_main".into(),
                physical_entry_point: "main".into(),
            },
            fragment: GpuCanvasShaderStage {
                source: r#"#version 300 es
                    precision highp float;
                    layout(location = 0) out vec4 color;
                    void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }
                "#
                .into(),
                logical_entry_point: "fs_main".into(),
                physical_entry_point: "main".into(),
            },
        };
        let plan = GpuCanvasPlan {
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
                .expect("canonical GLSL renders to a retained image");
            let image = factory
                .make_imported_gpu_canvas_image(&shader, &plan)
                .expect("the retained imported pipeline renders a second image");
            assert_eq!(
                factory.imported_gpu_canvas.pipeline_builds, 1,
                "identical temporal frames must not retranslate shaders or rebuild the WGPU pipeline"
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
