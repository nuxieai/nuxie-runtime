//! Arbitrary WGSL draw execution for editor GPU-canvas critique frames.
//!
//! Luau execution lives in `nuxie-scripting`; this module accepts only its
//! typed draw plan and owns shader modules, buffers, submission, and readback.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub use nuxie_render_api::{
    GpuCanvasBlendState, GpuCanvasColorTarget, GpuCanvasDepthStencilState, GpuCanvasIndexBuffer,
    GpuCanvasIndexedDraw, GpuCanvasPassState, GpuCanvasPipelineState, GpuCanvasSamplerBinding,
    GpuCanvasStencilFace, GpuCanvasTextureBinding, GpuCanvasTextureUpload, GpuCanvasUniformBuffer,
    GpuCanvasVertexAttribute, GpuCanvasVertexBuffer, GpuCanvasVertexLayout,
};
use nuxie_render_api::{
    GpuCanvasError, GpuCanvasPlan, GpuCanvasShader, GpuCanvasShaderEntry,
    GpuCanvasShaderEntrySelection, GpuCanvasShaderResourceKind, GpuCanvasShaderStage,
    GpuCanvasShaderTextureSampleType, GpuCanvasShaderTextureViewDimension, RenderGpuCanvasShader,
    RenderImage,
};
use wgpu::util::DeviceExt;

use super::{RendererError, WgpuFactory, WgpuImage, WgpuImageTexture, align_to, map_buffer};

const MAX_GPU_CANVAS_DIMENSION: u32 = 2_048;
const MAX_UNIFORM_BUFFER_BYTES: usize = 64 * 1024;
const MAX_VERTEX_BUFFER_BYTES: usize = 16 * 1024 * 1024;
const MAX_DRAW_INVOCATIONS: u64 = 1_000_000;
const MAX_VERTEX_BUFFERS: usize = 8;
const MAX_VERTEX_ATTRIBUTES: usize = 16;
const MAX_BIND_GROUPS: u32 = 4;
const MAX_UNIFORM_BINDINGS_PER_GROUP: usize = 8;
const MAX_BINDING_INDEX: u32 = 7;
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
static NEXT_GPU_CANVAS_SHADER_OCCURRENCE_ID: AtomicU64 = AtomicU64::new(1);

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
    resource_requirements: BTreeMap<(u32, u32), ImportedResourceRequirement>,
}

#[derive(Clone)]
struct ImportedGpuCanvasPipelineKey {
    vertex_occurrence_id: u64,
    fragment_occurrence_id: u64,
    vertex_entry: Option<GpuCanvasShaderEntrySelection>,
    fragment_entry: Option<GpuCanvasShaderEntrySelection>,
    uniform_bindings: Vec<(u32, u32, usize)>,
    vertex_layouts: Vec<GpuCanvasVertexLayout>,
    vertex_buffers: Vec<(u32, usize)>,
    index_buffer: Option<(usize, String)>,
    texture_bindings: Vec<ImportedTextureBindingKey>,
    sampler_bindings: Vec<(
        u32,
        u32,
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        u32,
        u32,
        u16,
    )>,
    pipeline_state: nuxie_render_api::GpuCanvasPipelineState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ImportedTextureBindingKey {
    group: u32,
    binding: u32,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    format: String,
    texture_type: String,
    sample_count: u32,
    mip_level_count: u32,
    view_dimension: String,
    base_mip_level: u32,
    mip_level_count_in_view: u32,
    base_array_layer: u32,
    array_layer_count: u32,
}

impl ImportedGpuCanvasPipelineKey {
    fn new(
        vertex_shader: &WgpuGpuCanvasShader,
        fragment_shader: &WgpuGpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Self {
        Self::from_occurrence_ids(
            vertex_shader.occurrence_id,
            fragment_shader.occurrence_id,
            plan,
        )
    }

    fn from_occurrence_ids(
        vertex_occurrence_id: u64,
        fragment_occurrence_id: u64,
        plan: &GpuCanvasPlan,
    ) -> Self {
        let uniform_bindings = plan
            .uniform_buffers
            .iter()
            .map(|buffer| (buffer.group, buffer.binding, buffer.bytes.len()))
            .collect::<Vec<_>>();
        Self {
            vertex_occurrence_id,
            fragment_occurrence_id,
            vertex_entry: plan.vertex_entry.clone(),
            fragment_entry: plan.fragment_entry.clone(),
            uniform_bindings,
            vertex_layouts: plan.vertex_layouts.clone(),
            vertex_buffers: plan
                .vertex_buffers
                .iter()
                .map(|buffer| (buffer.slot, buffer.bytes.len()))
                .collect(),
            index_buffer: plan
                .index_buffer
                .as_ref()
                .map(|buffer| (buffer.bytes.len(), buffer.format.clone())),
            texture_bindings: plan
                .texture_bindings
                .iter()
                .map(|texture| ImportedTextureBindingKey {
                    group: texture.group,
                    binding: texture.binding,
                    width: texture.width,
                    height: texture.height,
                    depth_or_array_layers: texture.depth_or_array_layers,
                    format: texture.format.clone(),
                    texture_type: texture.texture_type.clone(),
                    sample_count: texture.sample_count,
                    mip_level_count: texture.mip_level_count,
                    view_dimension: texture.view_dimension.clone(),
                    base_mip_level: texture.base_mip_level,
                    mip_level_count_in_view: texture.mip_level_count_in_view,
                    base_array_layer: texture.base_array_layer,
                    array_layer_count: texture.array_layer_count,
                })
                .collect(),
            sampler_bindings: plan
                .sampler_bindings
                .iter()
                .map(|sampler| {
                    (
                        sampler.group,
                        sampler.binding,
                        sampler.min_filter.clone(),
                        sampler.mag_filter.clone(),
                        sampler.mipmap_filter.clone(),
                        sampler.address_mode_u.clone(),
                        sampler.address_mode_v.clone(),
                        sampler.address_mode_w.clone(),
                        sampler.compare.clone(),
                        sampler.lod_min_clamp.to_bits(),
                        sampler.lod_max_clamp.to_bits(),
                        sampler.max_anisotropy,
                    )
                })
                .collect(),
            pipeline_state: plan.pipeline_state.clone(),
        }
    }

    fn buffer_bytes(&self) -> usize {
        self.uniform_bindings
            .iter()
            .map(|(_, _, bytes)| *bytes)
            .chain(self.vertex_buffers.iter().map(|(_, bytes)| *bytes))
            .chain(self.index_buffer.iter().map(|(bytes, _)| *bytes))
            .chain(self.texture_bindings.iter().map(|texture| {
                (texture.width as usize)
                    .saturating_mul(texture.height as usize)
                    .saturating_mul(texture.depth_or_array_layers as usize)
                    .saturating_mul(4)
            }))
            .fold(0, usize::saturating_add)
    }
}

impl PartialEq for ImportedGpuCanvasPipelineKey {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_occurrence_id == other.vertex_occurrence_id
            && self.fragment_occurrence_id == other.fragment_occurrence_id
            && self.vertex_entry == other.vertex_entry
            && self.fragment_entry == other.fragment_entry
            && self.uniform_bindings == other.uniform_bindings
            && self.vertex_layouts == other.vertex_layouts
            && self.vertex_buffers == other.vertex_buffers
            && self.index_buffer == other.index_buffer
            && self.texture_bindings == other.texture_bindings
            && self.sampler_bindings == other.sampler_bindings
            && self.pipeline_state == other.pipeline_state
    }
}

impl Eq for ImportedGpuCanvasPipelineKey {}

impl std::fmt::Debug for ImportedGpuCanvasPipelineKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ImportedGpuCanvasPipelineKey")
            .field("vertex_occurrence_id", &self.vertex_occurrence_id)
            .field("fragment_occurrence_id", &self.fragment_occurrence_id)
            .field("vertex_entry", &self.vertex_entry)
            .field("fragment_entry", &self.fragment_entry)
            .field("uniform_bindings", &self.uniform_bindings)
            .field("vertex_layouts", &self.vertex_layouts)
            .field("vertex_buffers", &self.vertex_buffers)
            .finish_non_exhaustive()
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
    resource_requirements: BTreeMap<(u32, u32), ImportedResourceRequirement>,
    bind_groups: Vec<wgpu::BindGroup>,
    uniform_buffers: Vec<wgpu::Buffer>,
    textures: Vec<wgpu::Texture>,
    _texture_views: Vec<wgpu::TextureView>,
    _samplers: Vec<wgpu::Sampler>,
    vertex_buffers: Vec<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImportedResourceRequirement {
    Uniform(ImportedUniformRequirement),
    Texture {
        stage_mask: u8,
        view_dimension: GpuCanvasShaderTextureViewDimension,
        sample_type: GpuCanvasShaderTextureSampleType,
        multisampled: bool,
    },
    Sampler {
        stage_mask: u8,
        comparison: bool,
    },
}

impl ImportedResourceRequirement {
    fn stage_mask(self) -> u8 {
        match self {
            Self::Uniform(requirement) => requirement.stage_mask,
            Self::Texture { stage_mask, .. } | Self::Sampler { stage_mask, .. } => stage_mask,
        }
    }

    fn visibility(self) -> wgpu::ShaderStages {
        ImportedUniformRequirement {
            required_size: 0,
            stage_mask: self.stage_mask(),
        }
        .visibility()
    }
}

struct ParsedAuthoredWgsl {
    module: naga::Module,
    info: naga::valid::ModuleInfo,
}

struct WgpuGpuCanvasShader {
    occurrence_id: u64,
    owner: std::sync::Weak<super::Context>,
    shader: GpuCanvasShader,
    parsed: ParsedAuthoredWgsl,
    uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
    resource_requirements: BTreeMap<(u32, u32), ImportedResourceRequirement>,
    module: wgpu::ShaderModule,
}

#[derive(Clone, Copy)]
struct ImportedGpuCanvasShaderRef<'a> {
    shader: &'a GpuCanvasShader,
    parsed: &'a ParsedAuthoredWgsl,
    uniform_requirements: &'a BTreeMap<(u32, u32), ImportedUniformRequirement>,
    resource_requirements: &'a BTreeMap<(u32, u32), ImportedResourceRequirement>,
}

impl WgpuGpuCanvasShader {
    fn imported(&self) -> ImportedGpuCanvasShaderRef<'_> {
        ImportedGpuCanvasShaderRef {
            shader: &self.shader,
            parsed: &self.parsed,
            uniform_requirements: &self.uniform_requirements,
            resource_requirements: &self.resource_requirements,
        }
    }
}

impl RenderGpuCanvasShader for WgpuGpuCanvasShader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
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
    vertex_shader: &WgpuGpuCanvasShader,
    fragment_shader: &WgpuGpuCanvasShader,
    plan: &GpuCanvasPlan,
) -> Result<PreparedImportedGpuCanvas, GpuCanvasError> {
    prepare_imported_gpu_canvas_modules(vertex_shader.imported(), fragment_shader.imported(), plan)
}

fn prepare_imported_gpu_canvas_modules(
    vertex_shader: ImportedGpuCanvasShaderRef<'_>,
    fragment_shader: ImportedGpuCanvasShaderRef<'_>,
    plan: &GpuCanvasPlan,
) -> Result<PreparedImportedGpuCanvas, GpuCanvasError> {
    validate_imported_gpu_canvas_plan(plan)
        .map_err(|error| GpuCanvasError::new(error.to_string()))?;
    let vertex_record = resolve_imported_entry(
        vertex_shader.shader,
        GpuCanvasShaderStage::Vertex,
        plan.vertex_entry.as_ref(),
        "vertex",
    )?;
    let fragment_record = resolve_imported_entry(
        fragment_shader.shader,
        GpuCanvasShaderStage::Fragment,
        plan.fragment_entry.as_ref(),
        "fragment",
    )?;
    validate_imported_interface(
        vertex_shader,
        fragment_shader,
        plan,
        vertex_record,
        fragment_record,
    )?;
    let resource_requirements =
        merge_imported_resource_requirements(vertex_shader, fragment_shader)?;
    validate_imported_resource_plan(&resource_requirements, plan)?;
    Ok(PreparedImportedGpuCanvas {
        vertex_entry_point: vertex_record.physical_entry_point.clone(),
        fragment_entry_point: fragment_record.physical_entry_point.clone(),
        resource_requirements,
    })
}

fn validate_imported_resource_plan(
    requirements: &BTreeMap<(u32, u32), ImportedResourceRequirement>,
    plan: &GpuCanvasPlan,
) -> Result<(), GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let planned = plan
        .uniform_buffers
        .iter()
        .map(|resource| {
            (
                (resource.group, resource.binding),
                GpuCanvasShaderResourceKind::UniformBuffer,
            )
        })
        .chain(plan.texture_bindings.iter().map(|resource| {
            (
                (resource.group, resource.binding),
                GpuCanvasShaderResourceKind::SampledTexture,
            )
        }))
        .chain(plan.sampler_bindings.iter().map(|resource| {
            (
                (resource.group, resource.binding),
                if resource.compare.is_some() {
                    GpuCanvasShaderResourceKind::ComparisonSampler
                } else {
                    GpuCanvasShaderResourceKind::Sampler
                },
            )
        }))
        .collect::<BTreeMap<_, _>>();
    if requirements.keys().ne(planned.keys()) {
        return Err(invalid(format!(
            "shader resource bindings {:?} do not exactly match planned bindings {:?}",
            requirements.keys().collect::<Vec<_>>(),
            planned.keys().collect::<Vec<_>>()
        )));
    }
    for (&binding, requirement) in requirements {
        let planned_kind = planned[&binding];
        let expected_kind = match requirement {
            ImportedResourceRequirement::Uniform(_) => GpuCanvasShaderResourceKind::UniformBuffer,
            ImportedResourceRequirement::Texture { .. } => {
                GpuCanvasShaderResourceKind::SampledTexture
            }
            ImportedResourceRequirement::Sampler {
                comparison: false, ..
            } => GpuCanvasShaderResourceKind::Sampler,
            ImportedResourceRequirement::Sampler {
                comparison: true, ..
            } => GpuCanvasShaderResourceKind::ComparisonSampler,
        };
        if planned_kind != expected_kind {
            return Err(invalid(format!(
                "planned resource group {} binding {} has kind {planned_kind:?}; shader requires {expected_kind:?}",
                binding.0, binding.1
            )));
        }
    }
    Ok(())
}

fn merge_imported_resource_requirements(
    vertex_shader: ImportedGpuCanvasShaderRef<'_>,
    fragment_shader: ImportedGpuCanvasShaderRef<'_>,
) -> Result<BTreeMap<(u32, u32), ImportedResourceRequirement>, GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let mut requirements = vertex_shader.resource_requirements.clone();
    let fragment_stage_mask = 1 << GpuCanvasShaderStage::Fragment as u8;
    for (&binding, fragment_requirement) in fragment_shader
        .resource_requirements
        .iter()
        .filter(|(_, requirement)| requirement.stage_mask() & fragment_stage_mask != 0)
    {
        let Some(vertex_requirement) = requirements.get_mut(&binding) else {
            return Err(invalid(format!(
                "fragment resource group {} binding {} is absent from the vertex-authoritative target-16 map",
                binding.0, binding.1
            )));
        };
        if vertex_requirement.stage_mask() & fragment_stage_mask == 0 {
            return Err(invalid(format!(
                "fragment resource group {} binding {} is not fragment-visible in the vertex-authoritative target-16 map",
                binding.0, binding.1
            )));
        }
        match (&mut *vertex_requirement, fragment_requirement) {
            (
                ImportedResourceRequirement::Uniform(vertex),
                ImportedResourceRequirement::Uniform(fragment),
            ) => {
                vertex.required_size = vertex.required_size.max(fragment.required_size);
            }
            (vertex, fragment) if *vertex == *fragment => {}
            _ => {
                return Err(invalid(format!(
                    "resource group {} binding {} differs between shader modules",
                    binding.0, binding.1
                )));
            }
        }
    }
    Ok(requirements)
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
    vertex_shader: ImportedGpuCanvasShaderRef<'_>,
    fragment_shader: ImportedGpuCanvasShaderRef<'_>,
    plan: &GpuCanvasPlan,
    vertex_record: &GpuCanvasShaderEntry,
    fragment_record: &GpuCanvasShaderEntry,
) -> Result<BTreeMap<(u32, u32), ImportedUniformRequirement>, GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let vertex_entry = imported_entry_point(
        &vertex_shader.parsed.module,
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
        &fragment_shader.parsed.module,
        naga::ShaderStage::Fragment,
        &fragment_record.physical_entry_point,
    )
    .ok_or_else(|| {
        invalid(format!(
            "fragment stage has no physical entry point '{}'",
            fragment_record.physical_entry_point
        ))
    })?;

    let vertex_inputs =
        imported_function_inputs(&vertex_shader.parsed.module, &vertex_entry.function)?;
    let vertex_outputs =
        imported_function_output(&vertex_shader.parsed.module, &vertex_entry.function)?;
    let fragment_inputs =
        imported_function_inputs(&fragment_shader.parsed.module, &fragment_entry.function)?;
    let fragment_outputs =
        imported_function_output(&fragment_shader.parsed.module, &fragment_entry.function)?;

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

    // Preserve the pinned native stale assumption: whenever a vertex stage is
    // present, pipeline auto-layout is built exclusively from that module's
    // target-16 map. Do not union the fragment module's resources into it.
    let mut required_uniforms = vertex_shader.uniform_requirements.clone();
    let fragment_stage_mask = 1 << GpuCanvasShaderStage::Fragment as u8;
    for (&binding, fragment_requirement) in fragment_shader
        .uniform_requirements
        .iter()
        .filter(|(_, requirement)| requirement.stage_mask & fragment_stage_mask != 0)
    {
        let Some(vertex_requirement) = required_uniforms.get_mut(&binding) else {
            return Err(invalid(format!(
                "fragment resource group {} binding {} is absent from the vertex-authoritative target-16 map",
                binding.0, binding.1
            )));
        };
        if vertex_requirement.stage_mask & fragment_stage_mask == 0 {
            return Err(invalid(format!(
                "fragment resource group {} binding {} is not fragment-visible in the vertex-authoritative target-16 map",
                binding.0, binding.1
            )));
        }
        // Target 16 carries no uniform size. Keep its vertex-authored identity
        // and visibility, but preflight enough bytes for either explicit WGSL.
        vertex_requirement.required_size = vertex_requirement
            .required_size
            .max(fragment_requirement.required_size);
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

fn imported_resource_requirements(
    shader: &GpuCanvasShader,
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
) -> Result<BTreeMap<(u32, u32), ImportedResourceRequirement>, GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!("invalid imported GPU-canvas interface: {message}"))
    };
    let mut layouter = naga::proc::Layouter::default();
    layouter
        .update(module.to_ctx())
        .map_err(|error| invalid(format!("uniform layout failed: {error}")))?;
    let mut authored_resources = BTreeMap::new();
    for (handle, global) in module.global_variables.iter() {
        match global.space {
            naga::AddressSpace::Private => {}
            naga::AddressSpace::Uniform | naga::AddressSpace::Handle => {
                let binding = global
                    .binding
                    .as_ref()
                    .ok_or_else(|| invalid("GPU resource has no group and binding".into()))?;
                let key = (binding.group, binding.binding);
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
                let uniform_size = (global.space == naga::AddressSpace::Uniform)
                    .then_some(layouter[global.ty].size);
                if authored_resources
                    .insert(key, (global.space, uniform_size, stage_mask))
                    .is_some()
                {
                    return Err(invalid(format!(
                        "resource group {} binding {} appears more than once",
                        key.0, key.1
                    )));
                }
            }
            ref unsupported => {
                return Err(invalid(format!(
                    "global address space {unsupported:?} is outside the Lua GPU binding contract"
                )));
            }
        }
    }

    let mut requirements = BTreeMap::new();
    for binding in &shader.bindings {
        if matches!(
            binding.kind,
            GpuCanvasShaderResourceKind::StorageBufferReadOnly
                | GpuCanvasShaderResourceKind::StorageBufferReadWrite
                | GpuCanvasShaderResourceKind::StorageTexture
        ) {
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
        let (address_space, uniform_size, actual_stage_mask) =
            authored_resources.get(&key).copied().ok_or_else(|| {
                invalid(format!(
                    "binding map contains group {} binding {} absent from authored WGSL",
                    binding.group, binding.binding
                ))
            })?;
        let expected_space = match binding.kind {
            GpuCanvasShaderResourceKind::UniformBuffer => naga::AddressSpace::Uniform,
            GpuCanvasShaderResourceKind::SampledTexture
            | GpuCanvasShaderResourceKind::Sampler
            | GpuCanvasShaderResourceKind::ComparisonSampler => naga::AddressSpace::Handle,
            unsupported => {
                return Err(invalid(format!(
                    "binding group {} binding {} uses unsupported resource kind {unsupported:?}",
                    binding.group, binding.binding
                )));
            }
        };
        if address_space != expected_space {
            return Err(invalid(format!(
                "binding group {} binding {} kind {:?} disagrees with authored WGSL address space {address_space:?}",
                binding.group, binding.binding, binding.kind
            )));
        }
        if actual_stage_mask & !binding.stage_mask != 0 {
            return Err(invalid(format!(
                "binding group {} binding {} target-16 visibility {:#x} underdeclares authored WGSL usage {:#x}",
                binding.group, binding.binding, binding.stage_mask, actual_stage_mask
            )));
        }
        let requirement = match binding.kind {
            GpuCanvasShaderResourceKind::UniformBuffer => {
                ImportedResourceRequirement::Uniform(ImportedUniformRequirement {
                    required_size: uniform_size.ok_or_else(|| {
                        invalid(format!(
                            "uniform group {} binding {} has no layout size",
                            binding.group, binding.binding
                        ))
                    })?,
                    stage_mask: binding.stage_mask,
                })
            }
            GpuCanvasShaderResourceKind::SampledTexture => ImportedResourceRequirement::Texture {
                stage_mask: binding.stage_mask,
                view_dimension: binding.texture_view_dimension,
                sample_type: binding.texture_sample_type,
                multisampled: binding.texture_multisampled,
            },
            GpuCanvasShaderResourceKind::Sampler => ImportedResourceRequirement::Sampler {
                stage_mask: binding.stage_mask,
                comparison: false,
            },
            GpuCanvasShaderResourceKind::ComparisonSampler => {
                ImportedResourceRequirement::Sampler {
                    stage_mask: binding.stage_mask,
                    comparison: true,
                }
            }
            unsupported => {
                return Err(invalid(format!(
                    "binding group {} binding {} uses unsupported resource kind {unsupported:?}",
                    binding.group, binding.binding
                )));
            }
        };
        if requirements.insert(key, requirement).is_some() {
            return Err(invalid(format!(
                "binding map contains duplicate group {} binding {}",
                binding.group, binding.binding
            )));
        }
    }
    if requirements.keys().ne(authored_resources.keys()) {
        return Err(invalid(format!(
            "binding-map resources {:?} do not exactly match authored WGSL resources {:?}",
            requirements.keys().collect::<Vec<_>>(),
            authored_resources.keys().collect::<Vec<_>>()
        )));
    }
    Ok(requirements)
}

fn validate_imported_wgpu_limits(
    plan: &GpuCanvasPlan,
    resource_requirements: &BTreeMap<(u32, u32), ImportedResourceRequirement>,
    limits: &wgpu::Limits,
) -> Result<(), GpuCanvasError> {
    let invalid = |message: String| {
        GpuCanvasError::new(format!(
            "invalid imported GPU-canvas device limits: {message}"
        ))
    };
    let required_bind_groups = resource_requirements
        .keys()
        .map(|(group, _)| group.saturating_add(1))
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
        let uniform_count = resource_requirements
            .values()
            .filter(|requirement| {
                matches!(requirement, ImportedResourceRequirement::Uniform(_))
                    && requirement.stage_mask() & stage_bit != 0
            })
            .count();
        if uniform_count > limits.max_uniform_buffers_per_shader_stage as usize {
            return Err(invalid(format!(
                "{label} stage requires {uniform_count} uniform buffers across bind groups but the device supports {} per stage",
                limits.max_uniform_buffers_per_shader_stage
            )));
        }
        let texture_count = resource_requirements
            .values()
            .filter(|requirement| {
                matches!(requirement, ImportedResourceRequirement::Texture { .. })
                    && requirement.stage_mask() & stage_bit != 0
            })
            .count();
        if texture_count > limits.max_sampled_textures_per_shader_stage as usize {
            return Err(invalid(format!(
                "{label} stage requires {texture_count} sampled textures but the device supports {} per stage",
                limits.max_sampled_textures_per_shader_stage
            )));
        }
        let sampler_count = resource_requirements
            .values()
            .filter(|requirement| {
                matches!(requirement, ImportedResourceRequirement::Sampler { .. })
                    && requirement.stage_mask() & stage_bit != 0
            })
            .count();
        if sampler_count > limits.max_samplers_per_shader_stage as usize {
            return Err(invalid(format!(
                "{label} stage requires {sampler_count} samplers but the device supports {} per stage",
                limits.max_samplers_per_shader_stage
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

fn imported_shader_for_context<'a>(
    context: &Arc<super::Context>,
    shader: &'a Arc<dyn RenderGpuCanvasShader>,
    stage: &str,
) -> Result<&'a WgpuGpuCanvasShader, GpuCanvasError> {
    let shader = shader
        .as_any()
        .downcast_ref::<WgpuGpuCanvasShader>()
        .ok_or_else(|| {
            GpuCanvasError::new(format!(
                "GPU-canvas {stage} shader belongs to a different backend domain"
            ))
        })?;
    let owner = shader.owner.upgrade().ok_or_else(|| {
        GpuCanvasError::new(format!(
            "GPU-canvas {stage} shader's backend domain has expired"
        ))
    })?;
    if !Arc::ptr_eq(&owner, context) {
        return Err(GpuCanvasError::new(format!(
            "GPU-canvas {stage} shader belongs to a different factory/device domain"
        )));
    }
    Ok(shader)
}

impl WgpuFactory {
    pub(super) fn make_imported_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let parsed = parse_authored_wgsl(&shader.source)?;
        let resource_requirements =
            imported_resource_requirements(shader, &parsed.module, &parsed.info)?;
        let uniform_requirements = resource_requirements
            .iter()
            .filter_map(|(&binding, requirement)| match requirement {
                ImportedResourceRequirement::Uniform(requirement) => Some((binding, *requirement)),
                _ => None,
            })
            .collect();
        let module = self
            .context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nuxie-imported-gpu-canvas"),
                source: wgpu::ShaderSource::Wgsl(shader.source.clone().into()),
            });
        Ok(Arc::new(WgpuGpuCanvasShader {
            occurrence_id: NEXT_GPU_CANVAS_SHADER_OCCURRENCE_ID.fetch_add(1, Ordering::Relaxed),
            owner: Arc::downgrade(&self.context),
            shader: shader.clone(),
            parsed,
            uniform_requirements,
            resource_requirements,
            module,
        }))
    }

    /// Execute authored RSTB WGSL on the retained device and return the
    /// offscreen texture as a normal image owned by this factory domain.
    pub(super) fn make_imported_gpu_canvas_image(
        &mut self,
        vertex_shader_handle: &Arc<dyn RenderGpuCanvasShader>,
        fragment_shader_handle: &Arc<dyn RenderGpuCanvasShader>,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        let vertex_shader =
            imported_shader_for_context(&self.context, vertex_shader_handle, "vertex")?;
        let fragment_shader =
            imported_shader_for_context(&self.context, fragment_shader_handle, "fragment")?;
        validate_imported_gpu_canvas_plan(plan)
            .map_err(|error| GpuCanvasError::new(error.to_string()))?;
        let target_lease = self.gpu_canvas_targets.acquire(plan.width, plan.height)?;
        let device = &self.context.device;
        let queue = &self.context.queue;
        let key = ImportedGpuCanvasPipelineKey::new(vertex_shader, fragment_shader, plan);
        let pipeline_index = self
            .imported_gpu_canvas
            .pipelines
            .iter()
            .position(|pipeline| pipeline.key == key);
        let prepared_pipeline = if pipeline_index.is_none() {
            let prepared = prepare_imported_gpu_canvas(vertex_shader, fragment_shader, plan)?;
            validate_imported_wgpu_limits(plan, &prepared.resource_requirements, &device.limits())?;
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
            validate_imported_wgpu_limits(plan, &cached.resource_requirements, &device.limits())?;
            None
        };
        #[cfg(not(target_arch = "wasm32"))]
        let validation_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut built_pipeline = None;
        if let Some((prepared, vertex_attributes)) = prepared_pipeline {
            let mut bind_group_layouts = Vec::new();
            if let Some(max_group) = prepared
                .resource_requirements
                .keys()
                .map(|(group, _)| *group)
                .max()
            {
                for group in 0..=max_group {
                    let entries = prepared
                        .resource_requirements
                        .iter()
                        .filter(|((resource_group, _), _)| *resource_group == group)
                        .map(|(&(resource_group, binding), &requirement)| {
                            let uniform_size = plan
                                .uniform_buffers
                                .iter()
                                .find(|buffer| {
                                    buffer.group == resource_group && buffer.binding == binding
                                })
                                .map(|buffer| buffer.bytes.len());
                            Ok(wgpu::BindGroupLayoutEntry {
                                binding,
                                visibility: requirement.visibility(),
                                ty: imported_binding_type(requirement, uniform_size)?,
                                count: None,
                            })
                        })
                        .collect::<Result<Vec<_>, GpuCanvasError>>()?;
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
                    Ok(Some(wgpu::VertexBufferLayout {
                        array_stride: layout.stride,
                        step_mode: vertex_step_mode(&layout.step_mode)?,
                        attributes,
                    }))
                })
                .collect::<Result<Vec<_>, RendererError>>()
                .map_err(|error| GpuCanvasError::new(error.to_string()))?;
            let color_target =
                plan.pipeline_state.color_targets.first().ok_or_else(|| {
                    GpuCanvasError::new("GPU-canvas pipeline has no color target")
                })?;
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("nuxie-imported-gpu-canvas-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader.module,
                    entry_point: Some(&prepared.vertex_entry_point),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &vertex_layouts,
                },
                primitive: primitive_state(&plan.pipeline_state)
                    .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                depth_stencil: plan
                    .pipeline_state
                    .depth_stencil
                    .as_ref()
                    .map(depth_stencil_state)
                    .transpose()
                    .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                multisample: wgpu::MultisampleState {
                    count: plan.pipeline_state.sample_count,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader.module,
                    entry_point: Some(&prepared.fragment_entry_point),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: texture_format(&color_target.format)
                            .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                        blend: color_target
                            .blend
                            .as_ref()
                            .map(blend_state)
                            .transpose()
                            .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                        write_mask: color_writes(&color_target.write_mask)
                            .map_err(|error| GpuCanvasError::new(error.to_string()))?,
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
            let textures = plan
                .texture_bindings
                .iter()
                .map(|texture| {
                    let mut usage = wgpu::TextureUsages::TEXTURE_BINDING;
                    if texture.sample_count == 1 {
                        usage |= wgpu::TextureUsages::COPY_DST;
                    }
                    if texture.render_target {
                        usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
                    }
                    Ok(device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("nuxie-imported-gpu-canvas-texture"),
                        size: wgpu::Extent3d {
                            width: texture.width,
                            height: texture.height,
                            depth_or_array_layers: texture.depth_or_array_layers,
                        },
                        mip_level_count: texture.mip_level_count,
                        sample_count: texture.sample_count,
                        dimension: texture_dimension(&texture.texture_type)
                            .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                        format: texture_format(&texture.format)
                            .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                        usage,
                        view_formats: &[],
                    }))
                })
                .collect::<Result<Vec<_>, GpuCanvasError>>()?;
            let texture_views = plan
                .texture_bindings
                .iter()
                .zip(&textures)
                .map(|(texture, gpu_texture)| {
                    Ok(gpu_texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("nuxie-imported-gpu-canvas-texture-view"),
                        format: None,
                        dimension: Some(
                            texture_view_dimension(&texture.view_dimension)
                                .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                        ),
                        usage: None,
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: texture.base_mip_level,
                        mip_level_count: Some(texture.mip_level_count_in_view),
                        base_array_layer: texture.base_array_layer,
                        array_layer_count: Some(texture.array_layer_count),
                    }))
                })
                .collect::<Result<Vec<_>, GpuCanvasError>>()?;
            let samplers = plan
                .sampler_bindings
                .iter()
                .map(|sampler| {
                    Ok(device.create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("nuxie-imported-gpu-canvas-sampler"),
                        address_mode_u: sampler_address_mode(&sampler.address_mode_u)?,
                        address_mode_v: sampler_address_mode(&sampler.address_mode_v)?,
                        address_mode_w: sampler_address_mode(&sampler.address_mode_w)?,
                        mag_filter: sampler_filter_mode(&sampler.mag_filter)?,
                        min_filter: sampler_filter_mode(&sampler.min_filter)?,
                        mipmap_filter: sampler_mipmap_filter_mode(&sampler.mipmap_filter)?,
                        lod_min_clamp: sampler.lod_min_clamp,
                        lod_max_clamp: sampler.lod_max_clamp,
                        compare: sampler
                            .compare
                            .as_deref()
                            .map(compare_function)
                            .transpose()
                            .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                        anisotropy_clamp: sampler.max_anisotropy,
                        border_color: None,
                    }))
                })
                .collect::<Result<Vec<_>, GpuCanvasError>>()?;
            let bind_groups = bind_group_layouts
                .iter()
                .enumerate()
                .map(|(group, layout)| {
                    let entries = prepared
                        .resource_requirements
                        .iter()
                        .filter(|((resource_group, _), _)| *resource_group == group as u32)
                        .map(|(&(resource_group, binding), requirement)| {
                            let resource = match requirement {
                                ImportedResourceRequirement::Uniform(_) => {
                                    let index = plan
                                        .uniform_buffers
                                        .iter()
                                        .position(|buffer| {
                                            buffer.group == resource_group
                                                && buffer.binding == binding
                                        })
                                        .ok_or_else(|| {
                                            GpuCanvasError::new("uniform resource disappeared")
                                        })?;
                                    uniform_buffers[index].as_entire_binding()
                                }
                                ImportedResourceRequirement::Texture { .. } => {
                                    let index = plan
                                        .texture_bindings
                                        .iter()
                                        .position(|texture| {
                                            texture.group == resource_group
                                                && texture.binding == binding
                                        })
                                        .ok_or_else(|| {
                                            GpuCanvasError::new("texture resource disappeared")
                                        })?;
                                    wgpu::BindingResource::TextureView(&texture_views[index])
                                }
                                ImportedResourceRequirement::Sampler { .. } => {
                                    let index = plan
                                        .sampler_bindings
                                        .iter()
                                        .position(|sampler| {
                                            sampler.group == resource_group
                                                && sampler.binding == binding
                                        })
                                        .ok_or_else(|| {
                                            GpuCanvasError::new("sampler resource disappeared")
                                        })?;
                                    wgpu::BindingResource::Sampler(&samplers[index])
                                }
                            };
                            Ok(wgpu::BindGroupEntry { binding, resource })
                        })
                        .collect::<Result<Vec<_>, GpuCanvasError>>()?;
                    Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("nuxie-imported-gpu-canvas-bind-group"),
                        layout,
                        entries: &entries,
                    }))
                })
                .collect::<Result<Vec<_>, GpuCanvasError>>()?;
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
            let index_buffer = plan.index_buffer.as_ref().map(|buffer| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("nuxie-imported-gpu-canvas-index-buffer"),
                    contents: &buffer.bytes,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                })
            });
            built_pipeline = Some(ImportedWgpuGpuCanvasPipeline {
                key,
                resource_requirements: prepared.resource_requirements,
                bind_groups,
                uniform_buffers,
                textures,
                _texture_views: texture_views,
                _samplers: samplers,
                vertex_buffers,
                index_buffer,
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
        if let (Some(buffer), Some(gpu_buffer)) = (&plan.index_buffer, &cached.index_buffer) {
            queue.write_buffer(gpu_buffer, 0, &buffer.bytes);
        }
        for (texture, gpu_texture) in plan.texture_bindings.iter().zip(&cached.textures) {
            for upload in &texture.uploads {
                let origin_z = if texture.texture_type == "3d" {
                    upload.z
                } else {
                    upload.array_layer
                };
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: gpu_texture,
                        mip_level: upload.mip_level,
                        origin: wgpu::Origin3d {
                            x: upload.x,
                            y: upload.y,
                            z: origin_z,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &upload.bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(upload.bytes_per_row),
                        rows_per_image: Some(upload.rows_per_image),
                    },
                    wgpu::Extent3d {
                        width: upload.width,
                        height: upload.height,
                        depth_or_array_layers: upload.depth,
                    },
                );
            }
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
        let multisample_target = (plan.pipeline_state.sample_count > 1).then(|| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-imported-gpu-canvas-multisample-target"),
                size: wgpu::Extent3d {
                    width: plan.width,
                    height: plan.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: plan.pipeline_state.sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        });
        let multisample_view = multisample_target
            .as_ref()
            .map(|texture| texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let depth_texture = plan
            .pipeline_state
            .depth_stencil
            .as_ref()
            .map(|depth| {
                Ok(device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("nuxie-imported-gpu-canvas-depth-stencil"),
                    size: wgpu::Extent3d {
                        width: plan.width,
                        height: plan.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: plan.pipeline_state.sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: texture_format(&depth.format)
                        .map_err(|error| GpuCanvasError::new(error.to_string()))?,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                }))
            })
            .transpose()?;
        let depth_view = depth_texture
            .as_ref()
            .map(|texture| texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let depth_stencil_attachment = depth_view
            .as_ref()
            .zip(plan.pipeline_state.depth_stencil.as_ref())
            .map(
                |(depth_view, depth)| wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: depth
                        .format
                        .contains("stencil")
                        .then_some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0),
                            store: wgpu::StoreOp::Store,
                        }),
                },
            );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nuxie-imported-gpu-canvas-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nuxie-imported-gpu-canvas-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: multisample_view.as_ref().unwrap_or(&view),
                    resolve_target: multisample_view.as_ref().map(|_| &view),
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
                depth_stencil_attachment,
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
            if let Some(viewport) = plan.pass_state.viewport {
                pass.set_viewport(viewport[0], viewport[1], viewport[2], viewport[3], 0.0, 1.0);
            }
            if let Some(scissor) = plan.pass_state.scissor_rect {
                pass.set_scissor_rect(scissor[0], scissor[1], scissor[2], scissor[3]);
            }
            pass.set_stencil_reference(plan.pass_state.stencil_reference);
            pass.set_blend_constant(wgpu::Color {
                r: plan.pass_state.blend_color[0],
                g: plan.pass_state.blend_color[1],
                b: plan.pass_state.blend_color[2],
                a: plan.pass_state.blend_color[3],
            });
            if let Some(draw) = &plan.indexed_draw {
                let index_buffer = cached.index_buffer.as_ref().ok_or_else(|| {
                    GpuCanvasError::new("indexed GPU-canvas draw has no wgpu index buffer")
                })?;
                let format = index_format(
                    &plan
                        .index_buffer
                        .as_ref()
                        .ok_or_else(|| GpuCanvasError::new("indexed draw has no index buffer"))?
                        .format,
                )
                .map_err(|error| GpuCanvasError::new(error.to_string()))?;
                pass.set_index_buffer(index_buffer.slice(..), format);
                pass.draw_indexed(
                    draw.first_index..draw.first_index.saturating_add(draw.index_count),
                    draw.base_vertex,
                    draw.first_instance..draw.first_instance.saturating_add(draw.instance_count),
                );
            } else {
                pass.draw(
                    plan.first_vertex..plan.first_vertex.saturating_add(plan.vertex_count),
                    plan.first_instance..plan.first_instance.saturating_add(plan.instance_count),
                );
            }
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
                Ok(Some(wgpu::VertexBufferLayout {
                    array_stride: layout.stride,
                    step_mode: vertex_step_mode(&layout.step_mode)?,
                    attributes,
                }))
            })
            .collect::<Result<Vec<_>, RendererError>>()?;
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
    index_buffer: Option<&'a GpuCanvasIndexBuffer>,
    indexed_draw: Option<&'a GpuCanvasIndexedDraw>,
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
            index_buffer: None,
            indexed_draw: None,
        }
    }
}

impl<'a> From<&'a GpuCanvasPlan> for GpuCanvasPlanRef<'a> {
    fn from(plan: &'a GpuCanvasPlan) -> Self {
        let indexed_draw = plan.indexed_draw.as_ref();
        Self {
            width: plan.width,
            height: plan.height,
            clear_color: &plan.clear_color,
            vertex_count: indexed_draw.map_or(plan.vertex_count, |draw| draw.index_count),
            instance_count: indexed_draw.map_or(plan.instance_count, |draw| draw.instance_count),
            first_vertex: indexed_draw.map_or(plan.first_vertex, |draw| draw.first_index),
            first_instance: indexed_draw.map_or(plan.first_instance, |draw| draw.first_instance),
            uniform_buffers: &plan.uniform_buffers,
            vertex_layouts: &plan.vertex_layouts,
            vertex_buffers: &plan.vertex_buffers,
            index_buffer: plan.index_buffer.as_ref(),
            indexed_draw,
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
    if let Some(draw) = plan.indexed_draw {
        let buffer = plan
            .index_buffer
            .ok_or_else(|| invalid("indexed draw requires an index buffer".into()))?;
        let index_size = match buffer.format.as_str() {
            "uint16" => 2_u64,
            "uint32" => 4_u64,
            value => return Err(invalid(format!("invalid index format '{value}'"))),
        };
        let required_bytes = u64::from(draw.first_index)
            .checked_add(u64::from(draw.index_count))
            .and_then(|indices| indices.checked_mul(index_size))
            .ok_or_else(|| invalid("index buffer byte range overflow".into()))?;
        if required_bytes > buffer.bytes.len() as u64 {
            return Err(invalid(format!(
                "index buffer requires {required_bytes} bytes but contains {}",
                buffer.bytes.len()
            )));
        }
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
        if plan.indexed_draw.is_none() {
            let required_bytes = u64::from(vertex_end)
                .checked_mul(layout.stride)
                .ok_or_else(|| invalid("vertex buffer byte range overflow".into()))?;
            if required_bytes > buffer.bytes.len() as u64 {
                return Err(invalid(format!(
                    "vertex buffer slot {slot} requires {required_bytes} bytes"
                )));
            }
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
        "uint8x4" | "unorm8x4" | "snorm8x4" | "float16x2" => Ok(4),
        "float16x4" => Ok(8),
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
        "uint8x4" => Ok(wgpu::VertexFormat::Uint8x4),
        "unorm8x4" => Ok(wgpu::VertexFormat::Unorm8x4),
        "snorm8x4" => Ok(wgpu::VertexFormat::Snorm8x4),
        "float16x2" => Ok(wgpu::VertexFormat::Float16x2),
        "float16x4" => Ok(wgpu::VertexFormat::Float16x4),
        _ => Err(RendererError::Unsupported(
            "GPU-canvas vertex format is not implemented",
        )),
    }
}

fn vertex_step_mode(value: &str) -> Result<wgpu::VertexStepMode, RendererError> {
    match value {
        "vertex" => Ok(wgpu::VertexStepMode::Vertex),
        "instance" => Ok(wgpu::VertexStepMode::Instance),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid vertex step mode '{value}'"
        ))),
    }
}

fn index_format(value: &str) -> Result<wgpu::IndexFormat, RendererError> {
    match value {
        "uint16" => Ok(wgpu::IndexFormat::Uint16),
        "uint32" => Ok(wgpu::IndexFormat::Uint32),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid index format '{value}'"
        ))),
    }
}

fn texture_format(value: &str) -> Result<wgpu::TextureFormat, RendererError> {
    match value {
        "r8unorm" => Ok(wgpu::TextureFormat::R8Unorm),
        "rg8unorm" => Ok(wgpu::TextureFormat::Rg8Unorm),
        "rgba8unorm" => Ok(wgpu::TextureFormat::Rgba8Unorm),
        "bgra8unorm" => Ok(wgpu::TextureFormat::Bgra8Unorm),
        "r16float" => Ok(wgpu::TextureFormat::R16Float),
        "rg16float" => Ok(wgpu::TextureFormat::Rg16Float),
        "rgba16float" => Ok(wgpu::TextureFormat::Rgba16Float),
        "r32float" => Ok(wgpu::TextureFormat::R32Float),
        "rg32float" => Ok(wgpu::TextureFormat::Rg32Float),
        "rgba32float" => Ok(wgpu::TextureFormat::Rgba32Float),
        "rgb10a2unorm" => Ok(wgpu::TextureFormat::Rgb10a2Unorm),
        "rg11b10ufloat" => Ok(wgpu::TextureFormat::Rg11b10Ufloat),
        "depth16unorm" => Ok(wgpu::TextureFormat::Depth16Unorm),
        "depth24plus-stencil8" => Ok(wgpu::TextureFormat::Depth24PlusStencil8),
        "depth32float" => Ok(wgpu::TextureFormat::Depth32Float),
        "depth32float-stencil8" => Ok(wgpu::TextureFormat::Depth32FloatStencil8),
        "bc1-rgba-unorm" => Ok(wgpu::TextureFormat::Bc1RgbaUnorm),
        "bc3-rgba-unorm" => Ok(wgpu::TextureFormat::Bc3RgbaUnorm),
        "bc7-rgba-unorm" => Ok(wgpu::TextureFormat::Bc7RgbaUnorm),
        "etc2-rgb8unorm" => Ok(wgpu::TextureFormat::Etc2Rgb8Unorm),
        "etc2-rgba8unorm" => Ok(wgpu::TextureFormat::Etc2Rgba8Unorm),
        "astc-4x4-unorm" => Ok(wgpu::TextureFormat::Astc {
            block: wgpu::AstcBlock::B4x4,
            channel: wgpu::AstcChannel::Unorm,
        }),
        "astc-6x6-unorm" => Ok(wgpu::TextureFormat::Astc {
            block: wgpu::AstcBlock::B6x6,
            channel: wgpu::AstcChannel::Unorm,
        }),
        "astc-8x8-unorm" => Ok(wgpu::TextureFormat::Astc {
            block: wgpu::AstcBlock::B8x8,
            channel: wgpu::AstcChannel::Unorm,
        }),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid texture format '{value}'"
        ))),
    }
}

fn texture_dimension(value: &str) -> Result<wgpu::TextureDimension, RendererError> {
    match value {
        "2d" | "2d-array" | "cube" => Ok(wgpu::TextureDimension::D2),
        "3d" => Ok(wgpu::TextureDimension::D3),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid texture type '{value}'"
        ))),
    }
}

fn texture_view_dimension(value: &str) -> Result<wgpu::TextureViewDimension, RendererError> {
    match value {
        "2d" => Ok(wgpu::TextureViewDimension::D2),
        "2d-array" => Ok(wgpu::TextureViewDimension::D2Array),
        "cube" => Ok(wgpu::TextureViewDimension::Cube),
        "cube-array" => Ok(wgpu::TextureViewDimension::CubeArray),
        "3d" => Ok(wgpu::TextureViewDimension::D3),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid texture view dimension '{value}'"
        ))),
    }
}

fn reflected_texture_view_dimension(
    value: GpuCanvasShaderTextureViewDimension,
) -> Result<wgpu::TextureViewDimension, GpuCanvasError> {
    match value {
        GpuCanvasShaderTextureViewDimension::D1 => Ok(wgpu::TextureViewDimension::D1),
        GpuCanvasShaderTextureViewDimension::D2 => Ok(wgpu::TextureViewDimension::D2),
        GpuCanvasShaderTextureViewDimension::D2Array => Ok(wgpu::TextureViewDimension::D2Array),
        GpuCanvasShaderTextureViewDimension::Cube => Ok(wgpu::TextureViewDimension::Cube),
        GpuCanvasShaderTextureViewDimension::CubeArray => Ok(wgpu::TextureViewDimension::CubeArray),
        GpuCanvasShaderTextureViewDimension::D3 => Ok(wgpu::TextureViewDimension::D3),
        GpuCanvasShaderTextureViewDimension::Undefined => Err(GpuCanvasError::new(
            "sampled texture binding has no reflected view dimension",
        )),
    }
}

fn reflected_texture_sample_type(
    value: GpuCanvasShaderTextureSampleType,
) -> Result<wgpu::TextureSampleType, GpuCanvasError> {
    match value {
        GpuCanvasShaderTextureSampleType::Float => {
            Ok(wgpu::TextureSampleType::Float { filterable: true })
        }
        GpuCanvasShaderTextureSampleType::UnfilterableFloat => {
            Ok(wgpu::TextureSampleType::Float { filterable: false })
        }
        GpuCanvasShaderTextureSampleType::Depth => Ok(wgpu::TextureSampleType::Depth),
        GpuCanvasShaderTextureSampleType::Sint => Ok(wgpu::TextureSampleType::Sint),
        GpuCanvasShaderTextureSampleType::Uint => Ok(wgpu::TextureSampleType::Uint),
        GpuCanvasShaderTextureSampleType::Undefined => Err(GpuCanvasError::new(
            "sampled texture binding has no reflected sample type",
        )),
    }
}

fn sampler_address_mode(value: &str) -> Result<wgpu::AddressMode, GpuCanvasError> {
    match value {
        "repeat" => Ok(wgpu::AddressMode::Repeat),
        "mirror-repeat" => Ok(wgpu::AddressMode::MirrorRepeat),
        "clamp-to-edge" => Ok(wgpu::AddressMode::ClampToEdge),
        _ => Err(GpuCanvasError::new(format!(
            "invalid sampler address mode '{value}'"
        ))),
    }
}

fn sampler_filter_mode(value: &str) -> Result<wgpu::FilterMode, GpuCanvasError> {
    match value {
        "nearest" => Ok(wgpu::FilterMode::Nearest),
        "linear" => Ok(wgpu::FilterMode::Linear),
        _ => Err(GpuCanvasError::new(format!(
            "invalid sampler filter mode '{value}'"
        ))),
    }
}

fn sampler_mipmap_filter_mode(value: &str) -> Result<wgpu::MipmapFilterMode, GpuCanvasError> {
    match value {
        "nearest" => Ok(wgpu::MipmapFilterMode::Nearest),
        "linear" => Ok(wgpu::MipmapFilterMode::Linear),
        _ => Err(GpuCanvasError::new(format!(
            "invalid sampler mipmap filter mode '{value}'"
        ))),
    }
}

fn imported_binding_type(
    requirement: ImportedResourceRequirement,
    uniform_size: Option<usize>,
) -> Result<wgpu::BindingType, GpuCanvasError> {
    match requirement {
        ImportedResourceRequirement::Uniform(_) => Ok(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: uniform_size.and_then(|size| NonZeroU64::new(size as u64)),
        }),
        ImportedResourceRequirement::Texture {
            view_dimension,
            sample_type,
            multisampled,
            ..
        } => Ok(wgpu::BindingType::Texture {
            sample_type: reflected_texture_sample_type(sample_type)?,
            view_dimension: reflected_texture_view_dimension(view_dimension)?,
            multisampled,
        }),
        ImportedResourceRequirement::Sampler { comparison, .. } => {
            Ok(wgpu::BindingType::Sampler(if comparison {
                wgpu::SamplerBindingType::Comparison
            } else {
                wgpu::SamplerBindingType::Filtering
            }))
        }
    }
}

fn primitive_state(
    state: &nuxie_render_api::GpuCanvasPipelineState,
) -> Result<wgpu::PrimitiveState, RendererError> {
    let topology = match state.topology.as_str() {
        "triangle-list" => wgpu::PrimitiveTopology::TriangleList,
        "triangle-strip" => wgpu::PrimitiveTopology::TriangleStrip,
        "line-list" => wgpu::PrimitiveTopology::LineList,
        "line-strip" => wgpu::PrimitiveTopology::LineStrip,
        "point-list" => wgpu::PrimitiveTopology::PointList,
        value => {
            return Err(RendererError::InvalidGpuCanvas(format!(
                "invalid primitive topology '{value}'"
            )));
        }
    };
    let front_face = match state.winding.as_str() {
        "cw" => wgpu::FrontFace::Cw,
        "ccw" => wgpu::FrontFace::Ccw,
        value => {
            return Err(RendererError::InvalidGpuCanvas(format!(
                "invalid front-face winding '{value}'"
            )));
        }
    };
    let cull_mode = match state.cull_mode.as_str() {
        "none" => None,
        "front" => Some(wgpu::Face::Front),
        "back" => Some(wgpu::Face::Back),
        value => {
            return Err(RendererError::InvalidGpuCanvas(format!(
                "invalid cull mode '{value}'"
            )));
        }
    };
    Ok(wgpu::PrimitiveState {
        topology,
        strip_index_format: None,
        front_face,
        cull_mode,
        ..Default::default()
    })
}

fn blend_state(
    state: &nuxie_render_api::GpuCanvasBlendState,
) -> Result<wgpu::BlendState, RendererError> {
    Ok(wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: blend_factor(&state.src_color)?,
            dst_factor: blend_factor(&state.dst_color)?,
            operation: blend_operation(&state.color_op)?,
        },
        alpha: wgpu::BlendComponent {
            src_factor: blend_factor(&state.src_alpha)?,
            dst_factor: blend_factor(&state.dst_alpha)?,
            operation: blend_operation(&state.alpha_op)?,
        },
    })
}

fn blend_factor(value: &str) -> Result<wgpu::BlendFactor, RendererError> {
    match value {
        "zero" => Ok(wgpu::BlendFactor::Zero),
        "one" => Ok(wgpu::BlendFactor::One),
        "src" => Ok(wgpu::BlendFactor::Src),
        "one-minus-src" => Ok(wgpu::BlendFactor::OneMinusSrc),
        "src-alpha" => Ok(wgpu::BlendFactor::SrcAlpha),
        "one-minus-src-alpha" => Ok(wgpu::BlendFactor::OneMinusSrcAlpha),
        "dst" => Ok(wgpu::BlendFactor::Dst),
        "one-minus-dst" => Ok(wgpu::BlendFactor::OneMinusDst),
        "dst-alpha" => Ok(wgpu::BlendFactor::DstAlpha),
        "one-minus-dst-alpha" => Ok(wgpu::BlendFactor::OneMinusDstAlpha),
        "src-alpha-saturated" => Ok(wgpu::BlendFactor::SrcAlphaSaturated),
        "constant" => Ok(wgpu::BlendFactor::Constant),
        "one-minus-constant" => Ok(wgpu::BlendFactor::OneMinusConstant),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid blend factor '{value}'"
        ))),
    }
}

fn blend_operation(value: &str) -> Result<wgpu::BlendOperation, RendererError> {
    match value {
        "add" => Ok(wgpu::BlendOperation::Add),
        "subtract" => Ok(wgpu::BlendOperation::Subtract),
        "reverse-subtract" => Ok(wgpu::BlendOperation::ReverseSubtract),
        "min" => Ok(wgpu::BlendOperation::Min),
        "max" => Ok(wgpu::BlendOperation::Max),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid blend operation '{value}'"
        ))),
    }
}

fn color_writes(value: &str) -> Result<wgpu::ColorWrites, RendererError> {
    if matches!(value, "all" | "rgba") {
        return Ok(wgpu::ColorWrites::ALL);
    }
    if matches!(value, "" | "none") {
        return Ok(wgpu::ColorWrites::empty());
    }
    let mut writes = wgpu::ColorWrites::empty();
    for channel in value.chars().map(|value| value.to_ascii_lowercase()) {
        writes |= match channel {
            'r' => wgpu::ColorWrites::RED,
            'g' => wgpu::ColorWrites::GREEN,
            'b' => wgpu::ColorWrites::BLUE,
            'a' => wgpu::ColorWrites::ALPHA,
            _ => {
                return Err(RendererError::InvalidGpuCanvas(format!(
                    "invalid color write mask '{value}'"
                )));
            }
        };
    }
    Ok(writes)
}

fn depth_stencil_state(
    state: &nuxie_render_api::GpuCanvasDepthStencilState,
) -> Result<wgpu::DepthStencilState, RendererError> {
    Ok(wgpu::DepthStencilState {
        format: texture_format(&state.format)?,
        depth_write_enabled: Some(state.depth_write_enabled),
        depth_compare: Some(compare_function(&state.depth_compare)?),
        stencil: wgpu::StencilState {
            front: stencil_face_state(&state.stencil_front)?,
            back: stencil_face_state(&state.stencil_back)?,
            read_mask: state.stencil_read_mask,
            write_mask: state.stencil_write_mask,
        },
        bias: wgpu::DepthBiasState {
            constant: state.depth_bias,
            slope_scale: state.depth_bias_slope_scale,
            clamp: state.depth_bias_clamp,
        },
    })
}

fn compare_function(value: &str) -> Result<wgpu::CompareFunction, RendererError> {
    match value {
        "never" => Ok(wgpu::CompareFunction::Never),
        "less" => Ok(wgpu::CompareFunction::Less),
        "equal" => Ok(wgpu::CompareFunction::Equal),
        "less-equal" => Ok(wgpu::CompareFunction::LessEqual),
        "greater" => Ok(wgpu::CompareFunction::Greater),
        "not-equal" => Ok(wgpu::CompareFunction::NotEqual),
        "greater-equal" => Ok(wgpu::CompareFunction::GreaterEqual),
        "always" => Ok(wgpu::CompareFunction::Always),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid compare function '{value}'"
        ))),
    }
}

fn stencil_face_state(
    state: &nuxie_render_api::GpuCanvasStencilFace,
) -> Result<wgpu::StencilFaceState, RendererError> {
    Ok(wgpu::StencilFaceState {
        compare: compare_function(&state.compare)?,
        fail_op: stencil_operation(&state.fail_op)?,
        depth_fail_op: stencil_operation(&state.depth_fail_op)?,
        pass_op: stencil_operation(&state.pass_op)?,
    })
}

fn stencil_operation(value: &str) -> Result<wgpu::StencilOperation, RendererError> {
    match value {
        "keep" => Ok(wgpu::StencilOperation::Keep),
        "zero" => Ok(wgpu::StencilOperation::Zero),
        "replace" => Ok(wgpu::StencilOperation::Replace),
        "increment-clamp" => Ok(wgpu::StencilOperation::IncrementClamp),
        "decrement-clamp" => Ok(wgpu::StencilOperation::DecrementClamp),
        "invert" => Ok(wgpu::StencilOperation::Invert),
        "increment-wrap" => Ok(wgpu::StencilOperation::IncrementWrap),
        "decrement-wrap" => Ok(wgpu::StencilOperation::DecrementWrap),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "invalid stencil operation '{value}'"
        ))),
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
            index_buffer: None,
            indexed_draw: None,
            texture_bindings: Vec::new(),
            sampler_bindings: Vec::new(),
            pipeline_state: nuxie_render_api::GpuCanvasPipelineState::default(),
            pass_state: nuxie_render_api::GpuCanvasPassState::default(),
        }
    }

    fn prepare_test_gpu_canvas(
        shader: &GpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Result<PreparedImportedGpuCanvas, GpuCanvasError> {
        prepare_test_gpu_canvas_stages(shader, shader, plan)
    }

    fn prepare_test_gpu_canvas_stages(
        vertex_shader: &GpuCanvasShader,
        fragment_shader: &GpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Result<PreparedImportedGpuCanvas, GpuCanvasError> {
        let vertex_parsed = parse_authored_wgsl(&vertex_shader.source)?;
        let vertex_resources = imported_resource_requirements(
            vertex_shader,
            &vertex_parsed.module,
            &vertex_parsed.info,
        )?;
        let vertex_requirements = vertex_resources
            .iter()
            .filter_map(|(&binding, requirement)| match requirement {
                ImportedResourceRequirement::Uniform(requirement) => Some((binding, *requirement)),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let fragment_parsed = parse_authored_wgsl(&fragment_shader.source)?;
        let fragment_resources = imported_resource_requirements(
            fragment_shader,
            &fragment_parsed.module,
            &fragment_parsed.info,
        )?;
        let fragment_requirements = fragment_resources
            .iter()
            .filter_map(|(&binding, requirement)| match requirement {
                ImportedResourceRequirement::Uniform(requirement) => Some((binding, *requirement)),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        prepare_imported_gpu_canvas_modules(
            ImportedGpuCanvasShaderRef {
                shader: vertex_shader,
                parsed: &vertex_parsed,
                uniform_requirements: &vertex_requirements,
                resource_requirements: &vertex_resources,
            },
            ImportedGpuCanvasShaderRef {
                shader: fragment_shader,
                parsed: &fragment_parsed,
                uniform_requirements: &fragment_requirements,
                resource_requirements: &fragment_resources,
            },
            plan,
        )
    }

    #[test]
    fn imported_interface_preflight_accepts_only_the_exact_physical_stage_interface() {
        let shader = imported_shader(IMPORTED_WGSL);
        prepare_test_gpu_canvas(&shader, &imported_plan()).unwrap();

        let vertex_with_input = IMPORTED_WGSL.replace(
            "fn vs_main(@builtin(vertex_index) index: u32)",
            "fn vs_main(@builtin(vertex_index) index: u32, @location(0) position: vec2<f32>)",
        );
        let error = prepare_test_gpu_canvas(&imported_shader(&vertex_with_input), &imported_plan())
            .err()
            .expect("missing vertex plan must fail");
        assert!(error.to_string().contains("vertex inputs"), "{error}");

        let mut matching_plan = imported_plan();
        matching_plan.vertex_layouts.push(GpuCanvasVertexLayout {
            stride: 8,
            step_mode: "vertex".into(),
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
        prepare_test_gpu_canvas(&imported_shader(&vertex_with_input), &matching_plan).unwrap();
        matching_plan.vertex_layouts[0].attributes[0].format = "float32x3".into();
        matching_plan.vertex_layouts[0].stride = 12;
        matching_plan.vertex_buffers[0].bytes.resize(36, 0);
        let error = prepare_test_gpu_canvas(&imported_shader(&vertex_with_input), &matching_plan)
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
        let error = prepare_test_gpu_canvas(&imported_shader(&fragment_vec3), &imported_plan())
            .err()
            .expect("non-RGBA fragment output must fail");
        assert!(error.to_string().contains("fragment output"), "{error}");

        let wrong_entry = imported_shader_with_entries(IMPORTED_WGSL, "missing", "fs_main");
        let error = prepare_test_gpu_canvas(&wrong_entry, &imported_plan())
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
        let error = prepare_test_gpu_canvas(&imported_shader(varying), &imported_plan())
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
        let error = prepare_test_gpu_canvas(
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
        let error = prepare_test_gpu_canvas(
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
        prepare_test_gpu_canvas(
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
        prepare_test_gpu_canvas(
            &imported_uniform_shader(&uniform_wgsl, fragment_mask),
            &plan,
        )
        .expect("canonical target-16 WebGPU identity metadata is valid");

        let vertex_mask = 1 << GpuCanvasShaderStage::Vertex as u8;
        let error =
            prepare_test_gpu_canvas(&imported_uniform_shader(&uniform_wgsl, vertex_mask), &plan)
                .err()
                .expect("target-16 visibility may not omit a stage that actually uses the binding");
        assert!(error.to_string().contains("underdeclares"), "{error}");

        let broader_mask = fragment_mask | (1 << GpuCanvasShaderStage::Compute as u8);
        prepare_test_gpu_canvas(&imported_uniform_shader(&uniform_wgsl, broader_mask), &plan)
            .expect("C++ and WebGPU allow layout visibility broader than actual use");

        let mut unknown_stage = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        unknown_stage.bindings[0].stage_mask |= 0x80;
        let error = prepare_test_gpu_canvas(&unknown_stage, &plan)
            .err()
            .expect("unknown target-16 stage bits must fail closed");
        assert!(error.to_string().contains("unknown stage mask"), "{error}");

        let mut absent_visible_slot = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        absent_visible_slot.bindings[0].backend_slots[1] = None;
        let error = prepare_test_gpu_canvas(&absent_visible_slot, &plan)
            .err()
            .expect("a visible stage must retain its target-16 native slot");
        assert!(error.to_string().contains("identity mapping"), "{error}");

        let mut populated_absent_slot = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        populated_absent_slot.bindings[0].backend_slots[0] = Some(0);
        let error = prepare_test_gpu_canvas(&populated_absent_slot, &plan)
            .err()
            .expect("an invisible stage must preserve BindingMap::kAbsent");
        assert!(error.to_string().contains("identity mapping"), "{error}");

        let mut remapped_space = imported_uniform_shader(&uniform_wgsl, fragment_mask);
        remapped_space.bindings[0].backend_space = 1;
        let error = prepare_test_gpu_canvas(&remapped_space, &plan)
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

        let prepared = prepare_test_gpu_canvas(&shader, &plan)
            .expect("one authored module contains both physical entries");
        assert_eq!(prepared.vertex_entry_point, "vs_main");
        assert_eq!(prepared.fragment_entry_point, "fs_main");
        assert_eq!(
            prepared.resource_requirements[&(0, 0)].visibility(),
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );
    }

    #[test]
    fn distinct_stage_modules_do_not_union_fragment_resources_into_vertex_layout() {
        let fragment_source = IMPORTED_WGSL
            .replace(
                "@fragment",
                "struct Tint { value: vec4<f32>, }\n@group(0) @binding(0) var<uniform> tint: Tint;\n\n@fragment",
            )
            .replace(
                "return vec4<f32>(1.0, 0.0, 0.0, 1.0);",
                "return tint.value;",
            );
        let vertex_shader = imported_shader(IMPORTED_WGSL);
        let fragment_shader =
            imported_uniform_shader(&fragment_source, 1 << GpuCanvasShaderStage::Fragment as u8);
        let mut plan = imported_plan();
        plan.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 16],
        });

        let error = prepare_test_gpu_canvas_stages(&vertex_shader, &fragment_shader, &plan)
            .err()
            .expect("fragment resources cannot deepen the vertex-authoritative layout");

        assert!(
            error.to_string().contains(
                "fragment resource group 0 binding 0 is absent from the vertex-authoritative"
            ),
            "{error}"
        );
    }

    #[test]
    fn explicit_fragment_uniform_size_deepens_vertex_authoritative_preflight() {
        let vertex_source = IMPORTED_WGSL
            .replace(
                "@vertex",
                "struct Params { value: vec4<f32>, }\n@group(0) @binding(0) var<uniform> params: Params;\n\n@vertex",
            )
            .replace(
                "return vec4<f32>(x, y, 0.0, 1.0);",
                "return vec4<f32>(x, y, params.value.x * 0.0, 1.0);",
            );
        let fragment_source = IMPORTED_WGSL
            .replace(
                "@fragment",
                "struct Params { first: vec4<f32>, second: vec4<f32>, }\n@group(0) @binding(0) var<uniform> params: Params;\n\n@fragment",
            )
            .replace(
                "return vec4<f32>(1.0, 0.0, 0.0, 1.0);",
                "return params.second;",
            );
        let vertex_mask =
            (1 << GpuCanvasShaderStage::Vertex as u8) | (1 << GpuCanvasShaderStage::Fragment as u8);
        let fragment_mask = 1 << GpuCanvasShaderStage::Fragment as u8;
        let vertex_shader = imported_uniform_shader(&vertex_source, vertex_mask);
        let fragment_shader = imported_uniform_shader(&fragment_source, fragment_mask);

        let mut max_sized = imported_plan();
        max_sized.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 32],
        });
        let prepared = prepare_test_gpu_canvas_stages(&vertex_shader, &fragment_shader, &max_sized)
            .expect("the larger explicit fragment uniform determines the preflight size");
        assert_eq!(
            prepared.resource_requirements[&(0, 0)],
            ImportedResourceRequirement::Uniform(ImportedUniformRequirement {
                required_size: 32,
                stage_mask: vertex_mask,
            }),
            "layout identity and visibility remain vertex target-16 authoritative"
        );

        let mut too_small = max_sized;
        too_small.uniform_buffers[0].bytes.truncate(16);
        let error = prepare_test_gpu_canvas_stages(&vertex_shader, &fragment_shader, &too_small)
            .err()
            .expect("a buffer sized only for the vertex module must fail");
        assert!(error.to_string().contains("requires 32"), "{error}");
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

        let prepared = prepare_test_gpu_canvas(&shader, &plan)
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
        let prepared = prepare_test_gpu_canvas(&shader, &plan)
            .expect("the pipeline carries its resolved logical/physical pair");
        assert_eq!(prepared.vertex_entry_point, "physical_vertex_1");
        assert_eq!(prepared.fragment_entry_point, "physical_fragment_1");

        plan.fragment_entry.as_mut().unwrap().physical_entry_point = "physical_fragment_0".into();
        let error = prepare_test_gpu_canvas(&shader, &plan)
            .err()
            .expect("a stale logical/physical pair fails before device allocation");
        assert!(
            error.to_string().contains("no matching fragment"),
            "{error}"
        );
    }

    #[test]
    fn imported_pipeline_key_reuses_resources_for_animated_buffer_bytes() {
        let mut first = imported_plan();
        first.uniform_buffers.push(GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; 16],
        });
        let mut second = first.clone();
        second.uniform_buffers[0].bytes.fill(0xff);

        assert_eq!(
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(7, 7, &first),
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(7, 7, &second),
            "temporal byte updates must retain authored modules, pipelines, and buffers"
        );

        second.uniform_buffers[0].bytes.push(0);
        assert_ne!(
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(7, 7, &first),
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(7, 7, &second),
            "resource-size changes require a fresh backend allocation"
        );
    }

    #[test]
    fn imported_pipeline_key_orders_occurrences_and_distinguishes_entry_pairs() {
        let mut first = imported_plan();
        first.vertex_entry = Some(GpuCanvasShaderEntrySelection {
            logical_entry_point: "first_vertex".into(),
            physical_entry_point: "vs_first".into(),
        });
        first.fragment_entry = Some(GpuCanvasShaderEntrySelection {
            logical_entry_point: "first_fragment".into(),
            physical_entry_point: "fs_first".into(),
        });
        let mut second = first.clone();
        second.vertex_entry.as_mut().unwrap().logical_entry_point = "second_vertex".into();
        second.vertex_entry.as_mut().unwrap().physical_entry_point = "vs_second".into();

        assert_ne!(
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(11, 29, &first),
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(29, 11, &first),
            "vertex and fragment occurrence identity is ordered"
        );
        assert_ne!(
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(11, 29, &first),
            ImportedGpuCanvasPipelineKey::from_occurrence_ids(11, 29, &second),
            "one occurrence pair may materialize more than one pipeline key"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn wgpu_occurrences_are_fresh_and_reject_another_device_domain() {
        let Ok(mut first_factory) = WgpuFactory::new(8, 8) else {
            eprintln!("GPU adapter unavailable; domain validation is covered by the pure seam");
            return;
        };
        let Ok(mut second_factory) = WgpuFactory::new(8, 8) else {
            eprintln!(
                "second GPU adapter unavailable; domain validation is covered by the pure seam"
            );
            return;
        };
        let shader = imported_shader(IMPORTED_WGSL);
        let first = first_factory
            .make_imported_gpu_canvas_shader(&shader)
            .expect("first lookup occurrence");
        let second = first_factory
            .make_imported_gpu_canvas_shader(&shader)
            .expect("second same-source lookup occurrence");
        let first_id = first
            .as_any()
            .downcast_ref::<WgpuGpuCanvasShader>()
            .unwrap()
            .occurrence_id;
        let second_id = second
            .as_any()
            .downcast_ref::<WgpuGpuCanvasShader>()
            .unwrap()
            .occurrence_id;
        assert_ne!(first_id, second_id);

        let error = second_factory
            .make_imported_gpu_canvas_image(&first, &first, &imported_plan())
            .err()
            .expect("a WGPU occurrence cannot be rehomed to another device");
        assert!(
            error
                .to_string()
                .contains("different factory/device domain"),
            "{error}"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn imported_wgpu_executes_sampled_texture_sampler_and_indexed_draw() {
        let source = r#"
@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return textureSample(color_texture, color_sampler, vec2<f32>(0.5));
}
"#;
        let fragment_mask = 1 << GpuCanvasShaderStage::Fragment as u8;
        let mut shader = imported_shader(source);
        shader.bindings = vec![
            GpuCanvasShaderBinding {
                group: 0,
                binding: 0,
                kind: GpuCanvasShaderResourceKind::SampledTexture,
                stage_mask: fragment_mask,
                backend_space: 0,
                backend_slots: [None, Some(0), None],
                texture_view_dimension: GpuCanvasShaderTextureViewDimension::D2,
                texture_sample_type: GpuCanvasShaderTextureSampleType::Float,
                texture_multisampled: false,
            },
            GpuCanvasShaderBinding {
                group: 0,
                binding: 1,
                kind: GpuCanvasShaderResourceKind::Sampler,
                stage_mask: fragment_mask,
                backend_space: 0,
                backend_slots: [None, Some(1), None],
                texture_view_dimension: GpuCanvasShaderTextureViewDimension::Undefined,
                texture_sample_type: GpuCanvasShaderTextureSampleType::Undefined,
                texture_multisampled: false,
            },
        ];
        let mut plan = imported_plan();
        plan.vertex_count = 0;
        plan.index_buffer = Some(nuxie_render_api::GpuCanvasIndexBuffer {
            bytes: [0_u16, 1, 2]
                .into_iter()
                .flat_map(u16::to_le_bytes)
                .collect(),
            format: "uint16".into(),
        });
        plan.indexed_draw = Some(nuxie_render_api::GpuCanvasIndexedDraw {
            index_count: 3,
            instance_count: 1,
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        });
        plan.texture_bindings = vec![nuxie_render_api::GpuCanvasTextureBinding {
            group: 0,
            binding: 0,
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
            format: "rgba8unorm".into(),
            texture_type: "2d".into(),
            render_target: false,
            sample_count: 1,
            mip_level_count: 1,
            view_dimension: "2d".into(),
            base_mip_level: 0,
            mip_level_count_in_view: 1,
            base_array_layer: 0,
            array_layer_count: 1,
            uploads: vec![nuxie_render_api::GpuCanvasTextureUpload {
                bytes: vec![255, 0, 0, 255],
                width: 1,
                height: 1,
                depth: 1,
                x: 0,
                y: 0,
                z: 0,
                mip_level: 0,
                array_layer: 0,
                bytes_per_row: 4,
                rows_per_image: 1,
            }],
        }];
        plan.sampler_bindings = vec![nuxie_render_api::GpuCanvasSamplerBinding {
            group: 0,
            binding: 1,
            min_filter: "nearest".into(),
            mag_filter: "nearest".into(),
            mipmap_filter: "nearest".into(),
            address_mode_u: "clamp-to-edge".into(),
            address_mode_v: "clamp-to-edge".into(),
            address_mode_w: "clamp-to-edge".into(),
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            max_anisotropy: 1,
        }];
        let stencil_face = nuxie_render_api::GpuCanvasStencilFace {
            compare: "always".into(),
            fail_op: "keep".into(),
            depth_fail_op: "keep".into(),
            pass_op: "keep".into(),
        };
        plan.pipeline_state.sample_count = 4;
        plan.pipeline_state.depth_stencil = Some(nuxie_render_api::GpuCanvasDepthStencilState {
            format: "depth32float".into(),
            depth_compare: "always".into(),
            depth_write_enabled: true,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            stencil_front: stencil_face.clone(),
            stencil_back: stencil_face,
            stencil_read_mask: u32::MAX,
            stencil_write_mask: u32::MAX,
        });
        plan.pass_state.viewport = Some([0.0, 0.0, 8.0, 8.0]);
        plan.pass_state.scissor_rect = Some([0, 0, 8, 8]);

        let prepared = prepare_test_gpu_canvas(&shader, &plan)
            .expect("reflected texture and sampler bindings match the indexed plan");
        assert!(matches!(
            prepared.resource_requirements[&(0, 0)],
            ImportedResourceRequirement::Texture { .. }
        ));
        assert!(matches!(
            prepared.resource_requirements[&(0, 1)],
            ImportedResourceRequirement::Sampler {
                comparison: false,
                ..
            }
        ));

        let Ok(mut factory) = WgpuFactory::new(8, 8) else {
            eprintln!("GPU adapter unavailable; reflected resource validation remains covered");
            return;
        };

        let handle = factory
            .make_imported_gpu_canvas_shader(&shader)
            .expect("sampled shader imports");
        let image = factory
            .make_imported_gpu_canvas_image(&handle, &handle, &plan)
            .expect("texture, sampler, upload, and indexed draw execute on wgpu");
        assert_eq!((image.width(), image.height()), (8, 8));
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
                    ImportedResourceRequirement::Uniform(ImportedUniformRequirement {
                        required_size: 16,
                        stage_mask: 1 << GpuCanvasShaderStage::Fragment as u8,
                    }),
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
                if let ImportedResourceRequirement::Uniform(uniform) = &mut requirement {
                    uniform.stage_mask = if index < 7 {
                        1 << GpuCanvasShaderStage::Vertex as u8
                    } else {
                        1 << GpuCanvasShaderStage::Fragment as u8
                    };
                }
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
                step_mode: "vertex".into(),
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: "float32".into(),
                }],
            },
            GpuCanvasVertexLayout {
                stride: 4,
                step_mode: "vertex".into(),
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
                step_mode: "vertex".into(),
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
            index_buffer: None,
            indexed_draw: None,
            texture_bindings: Vec::new(),
            sampler_bindings: Vec::new(),
            pipeline_state: nuxie_render_api::GpuCanvasPipelineState::default(),
            pass_state: nuxie_render_api::GpuCanvasPassState::default(),
        };

        for mode in [crate::RenderMode::ClockwiseAtomic, crate::RenderMode::Msaa] {
            let Ok(mut factory) = WgpuFactory::new_with_mode(64, 48, mode) else {
                eprintln!(
                    "GPU adapter unavailable; exact iOS execution remains the required proof"
                );
                return;
            };
            let shader_handle = factory
                .make_imported_gpu_canvas_shader(&shader)
                .expect("lookup materializes one shader occurrence");
            let first = factory
                .make_imported_gpu_canvas_image(&shader_handle, &shader_handle, &plan)
                .expect("first byte set creates the imported pipeline");
            plan.uniform_buffers[0].bytes = encode_f32s(&[0.0, 1.0, 0.0, 1.0]);
            plan.vertex_buffers[0].bytes = encode_f32s(&[-1.0, -1.0, 3.0, -1.0, -1.0, 3.0]);
            let green = factory
                .make_imported_gpu_canvas_image(&shader_handle, &shader_handle, &plan)
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
            let alternate_handle = factory
                .make_imported_gpu_canvas_shader(&alternate)
                .expect("second lookup materializes a distinct occurrence");
            let alternate_image = factory
                .make_imported_gpu_canvas_image(&alternate_handle, &alternate_handle, &plan)
                .expect("a second shader key builds independently");
            plan.uniform_buffers[0].bytes = encode_f32s(&[1.0, 0.0, 0.0, 1.0]);
            let red = factory
                .make_imported_gpu_canvas_image(&shader_handle, &shader_handle, &plan)
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
            index_buffer: None,
            indexed_draw: None,
            texture_bindings: Vec::new(),
            sampler_bindings: Vec::new(),
            pipeline_state: nuxie_render_api::GpuCanvasPipelineState::default(),
            pass_state: nuxie_render_api::GpuCanvasPassState::default(),
        };
        for mode in [crate::RenderMode::ClockwiseAtomic, crate::RenderMode::Msaa] {
            let Ok(mut factory) = WgpuFactory::new_with_mode(160, 100, mode) else {
                eprintln!(
                    "GPU adapter unavailable; exact iOS execution remains the required proof"
                );
                return;
            };
            let shader_handle = factory
                .make_imported_gpu_canvas_shader(&shader)
                .expect("lookup materializes the canonical shader occurrence");
            let first_image = factory
                .make_imported_gpu_canvas_image(&shader_handle, &shader_handle, &plan)
                .expect("authored WGSL renders to a retained image");
            let image = factory
                .make_imported_gpu_canvas_image(&shader_handle, &shader_handle, &plan)
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
