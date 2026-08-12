//! Test-only replay of committed MSL at the Metal pipeline permutation seam.
//!
//! Enabling the feature is inert unless `NUXIE_APPLE_MSL_REPLAY_DIR` names a
//! generated catalog directory. In replay mode every Naga pipeline stage must
//! have an exact compiler-input key in the manifest. The committed MSL and
//! committed reflection are verified before they replace the freshly derived
//! translation output.

use alloc::{borrow::ToOwned as _, boxed::Box, string::String, vec::Vec};
use std::{collections::BTreeMap, env, fs, io::Write as _, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::shader_translation::{TranslationInput, TranslationOutput};

#[derive(Serialize)]
struct CompilerInput<'a> {
    source: Source<'a>,
    stage: &'static str,
    entry_point: &'a str,
    constants: BTreeMap<&'a str, f64>,
    resources: ResourceMap,
    binding_array_lengths: Vec<BindingArrayLength>,
    vertex_buffers: Vec<VertexBuffer>,
    primitive_topology: &'static str,
    allow_and_force_point_size: bool,
    msl_version: [u8; 2],
    zero_initialize_workgroup_memory: bool,
    runtime_checks: RuntimeChecks,
    task_dispatch_limits: TaskLimits,
    compile_options: CompileOptions,
}

#[derive(Serialize)]
struct Source<'a> {
    sha256: &'a str,
}

#[derive(Serialize)]
struct ResourceMap {
    bindings: Vec<ResourceBinding>,
    immediates_buffer: Option<u8>,
    sizes_buffer: Option<u8>,
}

#[derive(Serialize)]
struct ResourceBinding {
    group: u32,
    binding: u32,
    buffer: Option<u8>,
    texture: Option<u8>,
    sampler: Option<SamplerTarget>,
    external_texture: Option<ExternalTextureTarget>,
    mutable: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "slot", rename_all = "snake_case")]
enum SamplerTarget {
    Resource(u8),
    Inline(u8),
}

#[derive(Serialize)]
struct ExternalTextureTarget {
    planes: [u8; 3],
    params: u8,
}

#[derive(Serialize)]
struct BindingArrayLength {
    group: u32,
    binding: u32,
    length: u32,
}

#[derive(Serialize)]
struct VertexBuffer {
    id: u32,
    stride: u32,
    step_mode: &'static str,
    attributes: Vec<VertexAttribute>,
}

#[derive(Serialize)]
struct VertexAttribute {
    shader_location: u32,
    offset: u32,
    format: String,
}

#[derive(Serialize)]
struct RuntimeChecks {
    bounds_checks: bool,
    force_loop_bounding: bool,
    ray_query_initialization_tracking: bool,
    task_shader_dispatch_tracking: bool,
    mesh_shader_primitive_indices_clamp: bool,
    int_div_checks: bool,
}

#[derive(Serialize)]
struct TaskLimits {
    max_mesh_workgroups_per_dim: u32,
    max_mesh_workgroups_total: u32,
}

#[derive(Serialize)]
struct CompileOptions {
    language_version: [u8; 2],
    preserve_invariance_expected: bool,
}

#[derive(Deserialize)]
struct Manifest {
    schema_version: u32,
    artifacts: Vec<ManifestArtifact>,
    aliases: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct ManifestArtifact {
    key_sha256: String,
    msl_version: [u8; 2],
    msl_path: String,
    msl_sha256: String,
    translated_entry_point: String,
    workgroup_size: [u32; 3],
    workgroup_memory_sizes: Vec<u32>,
    sized_bindings: Vec<SizedBinding>,
    immutable_buffer_mask: usize,
    preserve_invariance: bool,
}

#[derive(Deserialize)]
struct SizedBinding {
    group: u32,
    binding: u32,
    array_index: u32,
}

pub(super) fn load(
    input: TranslationInput<'_>,
    generated: &TranslationOutput,
    primitive_topology: Option<wgt::PrimitiveTopology>,
) -> Result<Option<TranslationOutput>, Box<dyn std::error::Error>> {
    let Some(directory) = env::var_os("NUXIE_APPLE_MSL_REPLAY_DIR") else {
        return Ok(None);
    };
    let evidence = env::var_os("NUXIE_APPLE_MSL_REPLAY_EVIDENCE").ok_or(
        "MSL replay is fail-closed: NUXIE_APPLE_MSL_REPLAY_EVIDENCE must name an evidence file",
    )?;
    let directory = Path::new(&directory);
    let source = input
        .shader
        .debug_source
        .as_ref()
        .ok_or("MSL replay requires the DEBUG wgpu instance flag")?
        .source_code
        .as_bytes();
    let source_sha256 = sha256(source);
    let compiler_input = compiler_input(&input, generated, primitive_topology, &source_sha256)?;
    // Match the generator's canonical `serde_json::Value` representation. Its
    // map serializer orders object keys independently of Rust struct layout.
    let key_sha256 = sha256(&serde_json::to_vec(&serde_json::to_value(
        &compiler_input,
    )?)?);

    let manifest: Manifest = serde_json::from_slice(&fs::read(directory.join("manifest.json"))?)?;
    if manifest.schema_version != 2 {
        return Err(format!(
            "unsupported Apple MSL replay manifest schema {}",
            manifest.schema_version
        )
        .into());
    }
    if manifest.aliases.is_empty() {
        return Err("Apple MSL replay manifest has no logical aliases".into());
    }
    let matches: Vec<_> = manifest
        .artifacts
        .into_iter()
        .filter(|artifact| artifact.key_sha256 == key_sha256)
        .collect();
    let [artifact] = matches.as_slice() else {
        return Err(format!(
            "Apple MSL replay key {key_sha256} matched {} physical artifacts for {} {:?} {}; compiler input: {}",
            matches.len(),
            input.shader.debug_source.as_ref().unwrap().file_name,
            input.stage,
            input.entry_point,
            serde_json::to_string(&compiler_input)?,
        )
        .into());
    };
    if artifact.msl_version != [input.msl_version.0, input.msl_version.1] {
        return Err(format!(
            "Apple MSL replay selected {:?} for device language version {:?}",
            artifact.msl_version,
            [input.msl_version.0, input.msl_version.1]
        )
        .into());
    }

    let sized_bindings: Vec<_> = artifact
        .sized_bindings
        .iter()
        .map(|binding| {
            (
                naga::ResourceBinding {
                    group: binding.group,
                    binding: binding.binding,
                },
                binding.array_index,
            )
        })
        .collect();
    if artifact.translated_entry_point != generated.translated_entry_point
        || artifact.workgroup_size != generated.workgroup_size
        || artifact.workgroup_memory_sizes != generated.workgroup_memory_sizes
        || sized_bindings != generated.sized_bindings
        || artifact.immutable_buffer_mask != generated.immutable_buffer_mask
        || artifact.preserve_invariance != generated.preserve_invariance
    {
        return Err(format!(
            "committed reflection for {key_sha256} does not match the live HAL pipeline inputs"
        )
        .into());
    }

    let expected_path = format!("{key_sha256}.metal");
    if artifact.msl_path != expected_path {
        return Err(format!(
            "Apple MSL replay path {:?} does not equal content-keyed path {expected_path:?}",
            artifact.msl_path
        )
        .into());
    }
    let source = fs::read_to_string(directory.join(&expected_path))?;
    if sha256(source.as_bytes()) != artifact.msl_sha256 {
        return Err(format!("committed MSL digest mismatch for {key_sha256}").into());
    }
    let mut evidence = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(evidence)?;
    writeln!(evidence, "{key_sha256}")?;
    Ok(Some(TranslationOutput {
        source,
        translated_entry_point: artifact.translated_entry_point.to_owned(),
        workgroup_size: artifact.workgroup_size,
        workgroup_memory_sizes: artifact.workgroup_memory_sizes.clone(),
        sized_bindings,
        immutable_buffer_mask: artifact.immutable_buffer_mask,
        preserve_invariance: artifact.preserve_invariance,
    }))
}

fn compiler_input<'a>(
    input: &TranslationInput<'a>,
    output: &TranslationOutput,
    primitive_topology: Option<wgt::PrimitiveTopology>,
    source_sha256: &'a str,
) -> Result<CompilerInput<'a>, serde_json::Error> {
    let resources = ResourceMap {
        bindings: input
            .resources
            .resources
            .iter()
            .map(|(binding, target)| ResourceBinding {
                group: binding.group,
                binding: binding.binding,
                buffer: target.buffer,
                texture: target.texture,
                sampler: target.sampler.as_ref().map(|sampler| match sampler {
                    naga::back::msl::BindSamplerTarget::Resource(slot) => {
                        SamplerTarget::Resource(*slot)
                    }
                    naga::back::msl::BindSamplerTarget::Inline(slot) => {
                        SamplerTarget::Inline(*slot)
                    }
                }),
                external_texture: target.external_texture.as_ref().map(|target| {
                    ExternalTextureTarget {
                        planes: target.planes,
                        params: target.params,
                    }
                }),
                mutable: target.mutable,
            })
            .collect(),
        immediates_buffer: input.resources.immediates_buffer,
        sizes_buffer: input.resources.sizes_buffer,
    };
    let mut binding_array_lengths: Vec<_> = input
        .binding_array_length_map
        .iter()
        .map(|(binding, length)| BindingArrayLength {
            group: binding.group,
            binding: binding.binding,
            length: *length,
        })
        .collect();
    binding_array_lengths.sort_by_key(|item| (item.group, item.binding));
    let vertex_buffers = input
        .vertex_buffer_mappings
        .iter()
        .map(|buffer| {
            Ok(VertexBuffer {
                id: buffer.id,
                stride: buffer.stride,
                step_mode: match buffer.step_mode {
                    naga::back::msl::VertexBufferStepMode::Constant => "constant",
                    naga::back::msl::VertexBufferStepMode::ByVertex => "vertex",
                    naga::back::msl::VertexBufferStepMode::ByInstance => "instance",
                },
                attributes: buffer
                    .attributes
                    .iter()
                    .map(|attribute| {
                        serde_json::to_value(attribute.format).map(|format| VertexAttribute {
                            shader_location: attribute.shader_location,
                            offset: attribute.offset,
                            format: format.as_str().unwrap_or("unknown").to_owned(),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<_, serde_json::Error>>()?;
    Ok(CompilerInput {
        source: Source {
            sha256: source_sha256,
        },
        stage: stage(input.stage),
        entry_point: input.entry_point,
        constants: input
            .constants
            .iter()
            .map(|(name, value)| (name.as_str(), *value))
            .collect(),
        resources,
        binding_array_lengths,
        vertex_buffers,
        primitive_topology: topology(primitive_topology),
        allow_and_force_point_size: input.allow_and_force_point_size,
        msl_version: [input.msl_version.0, input.msl_version.1],
        zero_initialize_workgroup_memory: input.zero_initialize_workgroup_memory,
        runtime_checks: RuntimeChecks {
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
        task_dispatch_limits: TaskLimits {
            max_mesh_workgroups_per_dim: input.task_dispatch_limits.max_mesh_workgroups_per_dim,
            max_mesh_workgroups_total: input.task_dispatch_limits.max_mesh_workgroups_total,
        },
        compile_options: CompileOptions {
            language_version: [input.msl_version.0, input.msl_version.1],
            preserve_invariance_expected: output.preserve_invariance,
        },
    })
}

fn stage(stage: naga::ShaderStage) -> &'static str {
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

fn topology(topology: Option<wgt::PrimitiveTopology>) -> &'static str {
    match topology {
        Some(wgt::PrimitiveTopology::PointList) => "point_list",
        Some(wgt::PrimitiveTopology::LineList) => "line_list",
        Some(wgt::PrimitiveTopology::LineStrip) => "line_strip",
        Some(wgt::PrimitiveTopology::TriangleList) => "triangle_list",
        Some(wgt::PrimitiveTopology::TriangleStrip) => "triangle_strip",
        None => "compute",
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
