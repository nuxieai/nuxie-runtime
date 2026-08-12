//! Opt-in capture of the exact Naga-to-MSL inputs derived by the Metal HAL.
//!
//! This module is compiled only for the repository's `apple-msl-capture`
//! tooling feature and is inert unless `NUXIE_APPLE_MSL_CAPTURE_DIR` is set.

use alloc::{
    borrow::ToOwned as _,
    boxed::Box,
    string::{String, ToString as _},
    vec::Vec,
};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::shader_translation::{TranslationInput, TranslationOutput};

static CAPTURE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Clone, Serialize)]
struct SourceRecord {
    path: String,
    sha256: String,
}

#[derive(Clone, Serialize)]
struct ArtifactRecord {
    id: String,
    source: SourceRecord,
    stage: String,
    entry_point: String,
    constants: BTreeMap<String, f64>,
    resources: ResourceMapRecord,
    binding_array_lengths: Vec<BindingArrayLengthRecord>,
    vertex_buffers: Vec<VertexBufferRecord>,
    primitive_topology: String,
    allow_and_force_point_size: bool,
    msl_version: [u8; 2],
    zero_initialize_workgroup_memory: bool,
    runtime_checks: RuntimeChecksRecord,
    task_dispatch_limits: TaskLimitsRecord,
    compile_options: CompileOptionsRecord,
}

#[derive(Clone, Serialize)]
struct ResourceMapRecord {
    bindings: Vec<ResourceBindingRecord>,
    immediates_buffer: Option<u8>,
    sizes_buffer: Option<u8>,
}

#[derive(Clone, Serialize)]
struct ResourceBindingRecord {
    group: u32,
    binding: u32,
    buffer: Option<u8>,
    texture: Option<u8>,
    sampler: Option<SamplerTargetRecord>,
    external_texture: Option<ExternalTextureTargetRecord>,
    mutable: bool,
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", content = "slot", rename_all = "snake_case")]
enum SamplerTargetRecord {
    Resource(u8),
    Inline(u8),
}

#[derive(Clone, Serialize)]
struct ExternalTextureTargetRecord {
    planes: [u8; 3],
    params: u8,
}

#[derive(Clone, Serialize)]
struct BindingArrayLengthRecord {
    group: u32,
    binding: u32,
    length: u32,
}

#[derive(Clone, Serialize)]
struct VertexBufferRecord {
    id: u32,
    stride: u32,
    step_mode: String,
    attributes: Vec<VertexAttributeRecord>,
}

#[derive(Clone, Serialize)]
struct VertexAttributeRecord {
    shader_location: u32,
    offset: u32,
    format: String,
}

#[derive(Clone, Serialize)]
struct RuntimeChecksRecord {
    bounds_checks: bool,
    force_loop_bounding: bool,
    ray_query_initialization_tracking: bool,
    task_shader_dispatch_tracking: bool,
    mesh_shader_primitive_indices_clamp: bool,
    int_div_checks: bool,
}

#[derive(Clone, Serialize)]
struct TaskLimitsRecord {
    max_mesh_workgroups_per_dim: u32,
    max_mesh_workgroups_total: u32,
}

#[derive(Clone, Serialize)]
struct CompileOptionsRecord {
    language_version: [u8; 2],
    preserve_invariance_expected: bool,
}

#[derive(Serialize)]
struct CaptureEnvelope {
    schema_version: u32,
    canonical_input_sha256: String,
    identity: IdentityRecord,
    artifact: ArtifactRecord,
    translation: TranslationRecord,
}

#[derive(Serialize)]
struct IdentityRecord {
    debug_name: Option<String>,
    source_sha256: Option<String>,
    module_sha256: String,
    source_code: Option<String>,
}

#[derive(Serialize)]
struct TranslationRecord {
    msl_source: String,
    msl_sha256: String,
    translated_entry_point: String,
    workgroup_size: [u32; 3],
    workgroup_memory_sizes: Vec<u32>,
    sized_bindings: Vec<SizedBindingRecord>,
    immutable_buffer_mask: usize,
    preserve_invariance: bool,
}

#[derive(Serialize)]
struct SizedBindingRecord {
    group: u32,
    binding: u32,
    array_index: u32,
}

pub(super) fn record(
    input: &TranslationInput<'_>,
    output: &TranslationOutput,
    primitive_topology: Option<wgt::PrimitiveTopology>,
) {
    let Some(directory) = env::var_os("NUXIE_APPLE_MSL_CAPTURE_DIR").map(PathBuf::from) else {
        return;
    };
    if let Err(error) = record_inner(&directory, input, output, primitive_topology) {
        log::error!("failed to capture Metal shader translation: {error}");
        let _ = fs::create_dir_all(&directory);
        let _ = fs::write(directory.join("_capture_error.txt"), error.to_string());
    }
}

fn record_inner(
    directory: &Path,
    input: &TranslationInput<'_>,
    output: &TranslationOutput,
    primitive_topology: Option<wgt::PrimitiveTopology>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = CAPTURE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "Apple MSL capture lock is poisoned")?;

    let module_bytes = serde_json::to_vec(input.shader.module.as_ref())?;
    let module_sha256 = sha256(&module_bytes);
    let debug_source = input.shader.debug_source.as_ref();
    let source_code = debug_source.map(|source| source.source_code.to_string());
    let source_sha256 = source_code
        .as_deref()
        .map(|source| sha256(source.as_bytes()));
    let debug_name = debug_source.map(|source| source.file_name.to_string());
    let source_digest = source_sha256
        .clone()
        .unwrap_or_else(|| module_sha256.clone());

    let mut artifact = ArtifactRecord {
        id: String::new(),
        source: SourceRecord {
            // The capture driver replaces this with the repository-relative WGSL
            // path after matching the exact source digest.
            path: String::new(),
            sha256: source_digest,
        },
        stage: shader_stage(input.stage).to_owned(),
        entry_point: input.entry_point.to_owned(),
        constants: input
            .constants
            .iter()
            .map(|(name, value)| (name.clone(), *value))
            .collect(),
        resources: resource_map(input.resources),
        binding_array_lengths: binding_array_lengths(input.binding_array_length_map),
        vertex_buffers: input
            .vertex_buffer_mappings
            .iter()
            .map(vertex_buffer)
            .collect::<Result<_, _>>()?,
        primitive_topology: topology_name(primitive_topology).to_owned(),
        allow_and_force_point_size: input.allow_and_force_point_size,
        msl_version: [input.msl_version.0, input.msl_version.1],
        zero_initialize_workgroup_memory: input.zero_initialize_workgroup_memory,
        runtime_checks: RuntimeChecksRecord {
            bounds_checks: input.runtime_checks.bounds_checks,
            force_loop_bounding: input.runtime_checks.force_loop_bounding,
            ray_query_initialization_tracking: input
                .runtime_checks
                .ray_query_initialization_tracking,
            task_shader_dispatch_tracking: input.runtime_checks.task_shader_dispatch_tracking,
            mesh_shader_primitive_indices_clamp: input
                .runtime_checks
                .mesh_shader_primitive_indices_clamp,
            int_div_checks: input.runtime_checks.int_div_checks,
        },
        task_dispatch_limits: TaskLimitsRecord {
            max_mesh_workgroups_per_dim: input.task_dispatch_limits.max_mesh_workgroups_per_dim,
            max_mesh_workgroups_total: input.task_dispatch_limits.max_mesh_workgroups_total,
        },
        compile_options: CompileOptionsRecord {
            language_version: [input.msl_version.0, input.msl_version.1],
            preserve_invariance_expected: output.preserve_invariance,
        },
    };

    let canonical_bytes = serde_json::to_vec(&artifact)?;
    let canonical_input_sha256 = sha256(&canonical_bytes);
    let name = debug_name.as_deref().unwrap_or("naga-module");
    let identity_prefix = sanitize(name);
    artifact.id = format!(
        "{}-{}-{}-{}",
        sanitize(name),
        shader_stage(input.stage),
        sanitize(input.entry_point),
        &canonical_input_sha256[..12]
    );

    let envelope = CaptureEnvelope {
        schema_version: 1,
        canonical_input_sha256: canonical_input_sha256.clone(),
        identity: IdentityRecord {
            debug_name,
            source_sha256,
            module_sha256,
            source_code,
        },
        artifact,
        translation: TranslationRecord {
            msl_source: output.source.clone(),
            msl_sha256: sha256(output.source.as_bytes()),
            translated_entry_point: output.translated_entry_point.clone(),
            workgroup_size: output.workgroup_size,
            workgroup_memory_sizes: output.workgroup_memory_sizes.clone(),
            sized_bindings: output
                .sized_bindings
                .iter()
                .map(|(binding, array_index)| SizedBindingRecord {
                    group: binding.group,
                    binding: binding.binding,
                    array_index: *array_index,
                })
                .collect(),
            immutable_buffer_mask: output.immutable_buffer_mask,
            preserve_invariance: output.preserve_invariance,
        },
    };

    fs::create_dir_all(directory)?;
    let destination = directory.join(format!("{identity_prefix}-{canonical_input_sha256}.json"));
    let temporary = directory.join(format!(".{identity_prefix}-{canonical_input_sha256}.tmp"));
    let mut bytes = serde_json::to_vec_pretty(&envelope)?;
    bytes.push(b'\n');
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, destination)?;
    Ok(())
}

fn resource_map(resources: &naga::back::msl::EntryPointResources) -> ResourceMapRecord {
    ResourceMapRecord {
        bindings: resources
            .resources
            .iter()
            .map(|(binding, target)| ResourceBindingRecord {
                group: binding.group,
                binding: binding.binding,
                buffer: target.buffer,
                texture: target.texture,
                sampler: target.sampler.as_ref().map(|sampler| match sampler {
                    naga::back::msl::BindSamplerTarget::Resource(slot) => {
                        SamplerTargetRecord::Resource(*slot)
                    }
                    naga::back::msl::BindSamplerTarget::Inline(slot) => {
                        SamplerTargetRecord::Inline(*slot)
                    }
                }),
                external_texture: target.external_texture.as_ref().map(|target| {
                    ExternalTextureTargetRecord {
                        planes: target.planes,
                        params: target.params,
                    }
                }),
                mutable: target.mutable,
            })
            .collect(),
        immediates_buffer: resources.immediates_buffer,
        sizes_buffer: resources.sizes_buffer,
    }
}

fn binding_array_lengths(
    lengths: &naga::FastHashMap<naga::ResourceBinding, u32>,
) -> Vec<BindingArrayLengthRecord> {
    let mut records: Vec<_> = lengths
        .iter()
        .map(|(binding, length)| BindingArrayLengthRecord {
            group: binding.group,
            binding: binding.binding,
            length: *length,
        })
        .collect();
    records.sort_by_key(|record| (record.group, record.binding));
    records
}

fn vertex_buffer(
    buffer: &naga::back::msl::VertexBufferMapping,
) -> Result<VertexBufferRecord, serde_json::Error> {
    Ok(VertexBufferRecord {
        id: buffer.id,
        stride: buffer.stride,
        step_mode: match buffer.step_mode {
            naga::back::msl::VertexBufferStepMode::Constant => "constant",
            naga::back::msl::VertexBufferStepMode::ByVertex => "vertex",
            naga::back::msl::VertexBufferStepMode::ByInstance => "instance",
        }
        .to_owned(),
        attributes: buffer
            .attributes
            .iter()
            .map(|attribute| {
                serde_json::to_value(attribute.format).map(|format| VertexAttributeRecord {
                    shader_location: attribute.shader_location,
                    offset: attribute.offset,
                    format: format.as_str().unwrap_or("unknown").to_owned(),
                })
            })
            .collect::<Result<_, _>>()?,
    })
}

fn shader_stage(stage: naga::ShaderStage) -> &'static str {
    match stage {
        naga::ShaderStage::Vertex => "vertex",
        naga::ShaderStage::Fragment => "fragment",
        naga::ShaderStage::Compute => "compute",
        naga::ShaderStage::Task => "task",
        naga::ShaderStage::Mesh => "mesh",
        naga::ShaderStage::RayGeneration => "ray_generation",
        naga::ShaderStage::Miss => "miss",
        naga::ShaderStage::AnyHit => "any_hit",
        naga::ShaderStage::ClosestHit => "closest_hit",
    }
}

fn topology_name(topology: Option<wgt::PrimitiveTopology>) -> &'static str {
    match topology {
        Some(wgt::PrimitiveTopology::PointList) => "point_list",
        Some(wgt::PrimitiveTopology::LineList) => "line_list",
        Some(wgt::PrimitiveTopology::LineStrip) => "line_strip",
        Some(wgt::PrimitiveTopology::TriangleList) => "triangle_list",
        Some(wgt::PrimitiveTopology::TriangleStrip) => "triangle_strip",
        None => "compute",
    }
}

fn sanitize(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !output.is_empty() {
            output.push('-');
            separator = true;
        }
    }
    while output.ends_with('-') {
        output.pop();
    }
    if output.is_empty() {
        "shader".to_owned()
    } else {
        output
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
