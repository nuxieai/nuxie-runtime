#![cfg(any(target_os = "ios", target_os = "macos"))]

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, ensure};
use naga::back::msl::{
    AttributeMapping, BindExternalTextureTarget, BindSamplerTarget, BindTarget,
    EntryPointResources, VertexBufferMapping, VertexBufferStepMode,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wgpu_hal::metal::shader_translation::{TranslationInput, translate};

const INPUT: &str = "tools/apple-msl-catalog/catalog.json";
const REVIEWED_INVENTORY: &str = "tools/apple-msl-catalog/reviewed-inventory.json";
const OUTPUT_DIR: &str = "crates/nuxie-renderer/apple-msl-catalog";
const MANIFEST: &str = "manifest.json";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub artifacts: Vec<Artifact>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub source: Source,
    pub stage: Stage,
    pub entry_point: String,
    pub constants: BTreeMap<String, f64>,
    pub resources: ResourceMap,
    pub binding_array_lengths: Vec<BindingArrayLength>,
    pub vertex_buffers: Vec<VertexBuffer>,
    pub primitive_topology: String,
    pub allow_and_force_point_size: bool,
    pub msl_version: [u8; 2],
    pub zero_initialize_workgroup_memory: bool,
    pub runtime_checks: RuntimeChecks,
    pub task_dispatch_limits: TaskLimits,
    pub compile_options: CompileOptions,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceMap {
    pub bindings: Vec<ResourceBinding>,
    pub immediates_buffer: Option<u8>,
    pub sizes_buffer: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBinding {
    pub group: u32,
    pub binding: u32,
    pub buffer: Option<u8>,
    pub texture: Option<u8>,
    pub sampler: Option<SamplerTarget>,
    pub external_texture: Option<ExternalTextureTarget>,
    pub mutable: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", content = "slot", rename_all = "snake_case")]
pub enum SamplerTarget {
    Resource(u8),
    Inline(u8),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalTextureTarget {
    pub planes: [u8; 3],
    pub params: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BindingArrayLength {
    pub group: u32,
    pub binding: u32,
    pub length: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VertexBuffer {
    pub id: u32,
    pub stride: u32,
    pub step_mode: StepMode,
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepMode {
    Constant,
    Vertex,
    Instance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VertexAttribute {
    pub shader_location: u32,
    pub offset: u32,
    pub format: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeChecks {
    pub bounds_checks: bool,
    pub force_loop_bounding: bool,
    pub ray_query_initialization_tracking: bool,
    pub task_shader_dispatch_tracking: bool,
    pub mesh_shader_primitive_indices_clamp: bool,
    pub int_div_checks: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskLimits {
    pub max_mesh_workgroups_per_dim: u32,
    pub max_mesh_workgroups_total: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompileOptions {
    pub language_version: [u8; 2],
    pub preserve_invariance_expected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct GeneratedManifest {
    schema_version: u32,
    artifacts: Vec<GeneratedArtifact>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct GeneratedArtifact {
    id: String,
    key_sha256: String,
    msl_path: String,
    msl_sha256: String,
    translated_entry_point: String,
    workgroup_size: [u32; 3],
    workgroup_memory_sizes: Vec<u32>,
    sized_bindings: Vec<GeneratedBinding>,
    immutable_buffer_mask: usize,
    preserve_invariance: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct GeneratedBinding {
    group: u32,
    binding: u32,
    array_index: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReviewedInventory {
    schema_version: u32,
    logical_permutations: InventoryFingerprint,
    compiler_artifacts: InventoryFingerprint,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct InventoryFingerprint {
    count: usize,
    sha256: String,
}

#[derive(Serialize)]
struct LogicalPermutation<'a> {
    id: &'a str,
    key_sha256: &'a str,
}

pub fn generate(root: &Path) -> Result<()> {
    let generated = render(root)?;
    let output = root.join(OUTPUT_DIR);
    fs::create_dir_all(&output)?;
    let expected: BTreeSet<_> = generated.msls.keys().cloned().collect();
    for entry in fs::read_dir(&output)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "metal")
            && !expected.contains(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default(),
            )
        {
            fs::remove_file(path)?;
        }
    }
    for (name, source) in generated.msls {
        fs::write(output.join(name), source)?;
    }
    fs::write(output.join(MANIFEST), generated.manifest)?;
    Ok(())
}

pub fn check(root: &Path) -> Result<()> {
    let generated = render(root)?;
    let output = root.join(OUTPUT_DIR);
    let manifest = fs::read_to_string(output.join(MANIFEST))
        .context("missing generated Apple MSL manifest")?;
    ensure!(
        manifest == generated.manifest,
        "stale generated Apple MSL manifest; run tools/generate-apple-msl-catalog.sh"
    );
    let actual: BTreeSet<_> = fs::read_dir(&output)
        .context("missing generated Apple MSL directory")?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            (path
                .extension()
                .is_some_and(|extension| extension == "metal"))
            .then(|| path.file_name()?.to_str().map(str::to_owned))
            .flatten()
        })
        .collect();
    let expected: BTreeSet<_> = generated.msls.keys().cloned().collect();
    ensure!(
        actual == expected,
        "missing or unreferenced generated MSL artifacts: expected {expected:?}, found {actual:?}"
    );
    for (name, expected_source) in generated.msls {
        let actual_source = fs::read_to_string(output.join(&name))
            .with_context(|| format!("missing generated artifact {name}"))?;
        ensure!(
            actual_source == expected_source,
            "stale generated artifact {name}"
        );
    }
    Ok(())
}

struct Rendered {
    manifest: String,
    msls: BTreeMap<String, String>,
}

fn render(root: &Path) -> Result<Rendered> {
    let bytes = fs::read(root.join(INPUT)).context("read Apple MSL catalog")?;
    let catalog: Catalog = serde_json::from_slice(&bytes).context("parse Apple MSL catalog")?;
    ensure!(
        catalog.schema_version == 1,
        "unsupported catalog schema version {}",
        catalog.schema_version
    );
    validate_reviewed_inventory(root, &catalog)?;
    let mut ids = BTreeSet::new();
    let mut key_sources: BTreeMap<String, String> = BTreeMap::new();
    let mut msls = BTreeMap::new();
    let mut records = Vec::new();
    for artifact in catalog.artifacts {
        ensure!(
            !artifact.id.is_empty()
                && artifact
                    .id
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')),
            "invalid artifact id {:?}",
            artifact.id
        );
        ensure!(
            ids.insert(artifact.id.clone()),
            "duplicate artifact id {:?}",
            artifact.id
        );
        ensure!(
            artifact.msl_version == artifact.compile_options.language_version,
            "{}: translation and Metal compile language versions differ",
            artifact.id
        );
        let topology = match artifact.primitive_topology.as_str() {
            "point_list" => wgpu_types::PrimitiveTopology::PointList,
            "line_list" => wgpu_types::PrimitiveTopology::LineList,
            "line_strip" => wgpu_types::PrimitiveTopology::LineStrip,
            "triangle_list" => wgpu_types::PrimitiveTopology::TriangleList,
            "triangle_strip" => wgpu_types::PrimitiveTopology::TriangleStrip,
            "compute" if matches!(artifact.stage, Stage::Compute) => {
                wgpu_types::PrimitiveTopology::TriangleList
            }
            unknown => anyhow::bail!("{}: unknown primitive topology {unknown:?}", artifact.id),
        };
        ensure!(
            artifact.allow_and_force_point_size
                == matches!(topology, wgpu_types::PrimitiveTopology::PointList),
            "{}: point-size setting does not match primitive topology",
            artifact.id
        );
        let source = fs::read(root.join(&artifact.source.path))
            .with_context(|| format!("read source for {}", artifact.id))?;
        ensure!(
            sha256(&source) == artifact.source.sha256,
            "{}: source digest is stale",
            artifact.id
        );
        // Human labels and repository paths are logical aliases, not compiler
        // inputs. Byte-identical built-ins share one committed MSL artifact.
        let key_sha256 = canonical_key_sha256(&artifact)?;
        if let Some(previous_path) = key_sources.get(&key_sha256) {
            ensure!(
                previous_path != &artifact.source.path,
                "duplicate canonical artifact key {key_sha256} for source {}",
                artifact.source.path
            );
        } else {
            key_sources.insert(key_sha256.clone(), artifact.source.path.clone());
        }
        let wgsl = std::str::from_utf8(&source)?;
        let module = naga::front::wgsl::parse_str(wgsl)
            .with_context(|| format!("parse WGSL for {}", artifact.id))?;
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .with_context(|| format!("validate WGSL for {}", artifact.id))?;
        let shader = wgpu_hal::NagaShader {
            module: Cow::Owned(module),
            info,
            debug_source: None,
        };
        let resources = resources(&artifact.resources)?;
        let arrays = artifact
            .binding_array_lengths
            .iter()
            .map(|binding| {
                (
                    naga::ResourceBinding {
                        group: binding.group,
                        binding: binding.binding,
                    },
                    binding.length,
                )
            })
            .collect();
        let vertices = artifact
            .vertex_buffers
            .iter()
            .map(vertex_buffer)
            .collect::<Result<Vec<_>>>()?;
        let output = translate(TranslationInput {
            shader: &shader,
            stage: artifact.stage.into(),
            entry_point: &artifact.entry_point,
            constants: &artifact
                .constants
                .iter()
                .map(|(key, value)| (key.clone(), *value))
                .collect(),
            resources: &resources,
            binding_array_length_map: &arrays,
            vertex_buffer_mappings: &vertices,
            allow_and_force_point_size: artifact.allow_and_force_point_size,
            msl_version: (artifact.msl_version[0], artifact.msl_version[1]),
            zero_initialize_workgroup_memory: artifact.zero_initialize_workgroup_memory,
            runtime_checks: artifact.runtime_checks.into(),
            task_dispatch_limits: naga::back::TaskDispatchLimits {
                max_mesh_workgroups_per_dim: artifact
                    .task_dispatch_limits
                    .max_mesh_workgroups_per_dim,
                max_mesh_workgroups_total: artifact.task_dispatch_limits.max_mesh_workgroups_total,
            },
        })
        .map_err(|error| anyhow::anyhow!("translate {}: {error:?}", artifact.id))?;
        ensure!(
            output.preserve_invariance == artifact.compile_options.preserve_invariance_expected,
            "{}: preserveInvariance expectation differs from translated MSL",
            artifact.id
        );
        let file = format!("{key_sha256}.metal");
        let msl_sha256 = sha256(output.source.as_bytes());
        let sized_bindings = output
            .sized_bindings
            .iter()
            .map(|(binding, array_index)| GeneratedBinding {
                group: binding.group,
                binding: binding.binding,
                array_index: *array_index,
            })
            .collect();
        records.push(GeneratedArtifact {
            id: artifact.id,
            key_sha256: key_sha256.clone(),
            msl_path: file.clone(),
            msl_sha256,
            translated_entry_point: output.translated_entry_point,
            workgroup_size: output.workgroup_size,
            workgroup_memory_sizes: output.workgroup_memory_sizes,
            sized_bindings,
            immutable_buffer_mask: output.immutable_buffer_mask,
            preserve_invariance: output.preserve_invariance,
        });
        if let Some(existing) = msls.insert(file.clone(), output.source.clone()) {
            ensure!(
                existing == output.source,
                "canonical key {key_sha256} generated divergent MSL"
            );
        }
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    let mut manifest = serde_json::to_string_pretty(&GeneratedManifest {
        schema_version: 1,
        artifacts: records,
    })?;
    manifest.push('\n');
    Ok(Rendered { manifest, msls })
}

fn resources(map: &ResourceMap) -> Result<EntryPointResources> {
    let mut resources = BTreeMap::new();
    for item in &map.bindings {
        let key = naga::ResourceBinding {
            group: item.group,
            binding: item.binding,
        };
        ensure!(
            !resources.contains_key(&key),
            "duplicate resource binding {}:{}",
            item.group,
            item.binding
        );
        resources.insert(
            key,
            BindTarget {
                buffer: item.buffer,
                texture: item.texture,
                sampler: item.sampler.as_ref().map(|target| match target {
                    SamplerTarget::Resource(slot) => BindSamplerTarget::Resource(*slot),
                    SamplerTarget::Inline(slot) => BindSamplerTarget::Inline(*slot),
                }),
                external_texture: item.external_texture.as_ref().map(|target| {
                    BindExternalTextureTarget {
                        planes: target.planes,
                        params: target.params,
                    }
                }),
                mutable: item.mutable,
            },
        );
    }
    Ok(EntryPointResources {
        resources,
        immediates_buffer: map.immediates_buffer,
        sizes_buffer: map.sizes_buffer,
    })
}

fn vertex_buffer(buffer: &VertexBuffer) -> Result<VertexBufferMapping> {
    Ok(VertexBufferMapping {
        id: buffer.id,
        stride: buffer.stride,
        step_mode: match buffer.step_mode {
            StepMode::Constant => VertexBufferStepMode::Constant,
            StepMode::Vertex => VertexBufferStepMode::ByVertex,
            StepMode::Instance => VertexBufferStepMode::ByInstance,
        },
        attributes: buffer
            .attributes
            .iter()
            .map(|attribute| {
                Ok(AttributeMapping {
                    shader_location: attribute.shader_location,
                    offset: attribute.offset,
                    format: serde_json::from_value(serde_json::Value::String(
                        attribute.format.clone(),
                    ))
                    .with_context(|| format!("unknown vertex format {:?}", attribute.format))?,
                })
            })
            .collect::<Result<_>>()?,
    })
}

impl From<Stage> for naga::ShaderStage {
    fn from(stage: Stage) -> Self {
        match stage {
            Stage::Vertex => Self::Vertex,
            Stage::Fragment => Self::Fragment,
            Stage::Compute => Self::Compute,
        }
    }
}
impl From<RuntimeChecks> for wgpu_types::ShaderRuntimeChecks {
    fn from(checks: RuntimeChecks) -> Self {
        Self {
            bounds_checks: checks.bounds_checks,
            force_loop_bounding: checks.force_loop_bounding,
            ray_query_initialization_tracking: checks.ray_query_initialization_tracking,
            task_shader_dispatch_tracking: checks.task_shader_dispatch_tracking,
            mesh_shader_primitive_indices_clamp: checks.mesh_shader_primitive_indices_clamp,
            int_div_checks: checks.int_div_checks,
        }
    }
}
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_key_sha256(artifact: &Artifact) -> Result<String> {
    let mut key = serde_json::to_value(artifact)?;
    let object = key
        .as_object_mut()
        .expect("Artifact always serializes as an object");
    object.remove("id");
    object
        .get_mut("source")
        .and_then(serde_json::Value::as_object_mut)
        .expect("Artifact source always serializes as an object")
        .remove("path");
    Ok(sha256(&serde_json::to_vec(&key)?))
}

fn reviewed_inventory(catalog: &Catalog) -> Result<ReviewedInventory> {
    let mut logical = catalog
        .artifacts
        .iter()
        .map(|artifact| Ok((artifact.id.as_str(), canonical_key_sha256(artifact)?)))
        .collect::<Result<Vec<_>>>()?;
    logical.sort();

    let compiler_keys: BTreeSet<_> = logical.iter().map(|(_, key)| key.as_str()).collect();
    let logical_records: Vec<_> = logical
        .iter()
        .map(|(id, key)| LogicalPermutation {
            id,
            key_sha256: key,
        })
        .collect();
    let compiler_keys: Vec<_> = compiler_keys.into_iter().collect();

    Ok(ReviewedInventory {
        schema_version: 1,
        logical_permutations: InventoryFingerprint {
            count: logical_records.len(),
            sha256: sha256(&serde_json::to_vec(&logical_records)?),
        },
        compiler_artifacts: InventoryFingerprint {
            count: compiler_keys.len(),
            sha256: sha256(&serde_json::to_vec(&compiler_keys)?),
        },
    })
}

fn validate_reviewed_inventory(root: &Path, catalog: &Catalog) -> Result<()> {
    let path = root.join(REVIEWED_INVENTORY);
    let expected: ReviewedInventory = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("read reviewed inventory {}", path.display()))?,
    )
    .with_context(|| format!("parse reviewed inventory {}", path.display()))?;
    ensure!(
        expected.schema_version == 1,
        "unsupported reviewed inventory schema version {}",
        expected.schema_version
    );
    let actual = reviewed_inventory(catalog)?;
    ensure!(
        actual.logical_permutations == expected.logical_permutations,
        "reviewed logical pipeline permutation inventory changed: expected {:?}, found {:?}; review the capture and deliberately update {REVIEWED_INVENTORY}",
        expected.logical_permutations,
        actual.logical_permutations
    );
    ensure!(
        actual.compiler_artifacts == expected.compiler_artifacts,
        "reviewed physical compiler artifact inventory changed: expected {:?}, found {:?}; review logical aliases and deliberately update {REVIEWED_INVENTORY}",
        expected.compiler_artifacts,
        actual.compiler_artifacts
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_catalog_and_reviewed_inventory(root: &Path, catalog: &Catalog) {
        let catalog_path = root.join(INPUT);
        fs::create_dir_all(catalog_path.parent().unwrap()).unwrap();
        fs::write(&catalog_path, serde_json::to_vec_pretty(catalog).unwrap()).unwrap();

        let inventory_path = root.join(REVIEWED_INVENTORY);
        fs::write(
            inventory_path,
            serde_json::to_vec_pretty(&reviewed_inventory(catalog).unwrap()).unwrap(),
        )
        .unwrap();
    }

    fn fixture_root() -> tempfile::TempDir {
        let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let temp = tempfile::tempdir().unwrap();
        let catalog_dir = temp.path().join("tools/apple-msl-catalog");
        fs::create_dir_all(&catalog_dir).unwrap();
        let repository: Catalog =
            serde_json::from_slice(&fs::read(source_root.join(INPUT)).unwrap()).unwrap();
        let artifact = repository.artifacts.into_iter().next().unwrap();
        let source = PathBuf::from(&artifact.source.path);
        let destination = temp.path().join(&source);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source_root.join(&source), destination).unwrap();
        write_catalog_and_reviewed_inventory(
            temp.path(),
            &Catalog {
                schema_version: 1,
                artifacts: vec![artifact],
            },
        );
        temp
    }

    #[test]
    fn repository_catalog_is_fresh_and_translates_real_layout() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        check(&root).unwrap();
        let manifest: GeneratedManifest = serde_json::from_str(
            &fs::read_to_string(root.join(OUTPUT_DIR).join(MANIFEST)).unwrap(),
        )
        .unwrap();
        assert!(!manifest.artifacts.is_empty());
        assert!(
            manifest
                .artifacts
                .iter()
                .all(|artifact| !artifact.translated_entry_point.is_empty())
        );
        let catalog: Catalog =
            serde_json::from_slice(&fs::read(root.join(INPUT)).unwrap()).unwrap();
        assert!(catalog.artifacts.iter().any(|artifact| {
            artifact
                .resources
                .bindings
                .iter()
                .any(|binding| binding.group > 0)
        }));
        assert!(
            catalog
                .artifacts
                .iter()
                .any(|artifact| !artifact.vertex_buffers.is_empty())
        );
    }

    #[test]
    fn duplicate_resource_bindings_are_rejected() {
        let binding = ResourceBinding {
            group: 1,
            binding: 2,
            buffer: Some(0),
            texture: None,
            sampler: None,
            external_texture: None,
            mutable: false,
        };
        let error = resources(&ResourceMap {
            bindings: vec![binding.clone(), binding],
            immediates_buffer: None,
            sizes_buffer: None,
        })
        .unwrap_err();
        assert!(error.to_string().contains("duplicate resource binding"));
    }

    #[test]
    fn constant_vertex_step_mode_round_trips_to_hal() {
        let mapping = vertex_buffer(&VertexBuffer {
            id: 30,
            stride: 0,
            step_mode: StepMode::Constant,
            attributes: Vec::new(),
        })
        .unwrap();

        assert!(matches!(mapping.step_mode, VertexBufferStepMode::Constant));
    }

    #[test]
    fn translation_key_covers_every_device_pipeline_and_compile_variant() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let catalog: Catalog =
            serde_json::from_slice(&fs::read(root.join(INPUT)).unwrap()).unwrap();
        let base = catalog.artifacts[0].clone();
        let mut variants = Vec::new();

        let mut source = base.clone();
        source.source.sha256 = "0".repeat(64);
        variants.push(source);

        let mut entry = base.clone();
        entry.entry_point.push_str("_variant");
        variants.push(entry);

        let mut stage = base.clone();
        stage.stage = match stage.stage {
            Stage::Vertex => Stage::Fragment,
            Stage::Fragment | Stage::Compute => Stage::Vertex,
        };
        variants.push(stage);

        let mut constants = base.clone();
        constants.constants.insert("__key_test".to_owned(), 1.0);
        variants.push(constants);

        let mut resource = base.clone();
        resource.resources.immediates_buffer = Some(
            resource
                .resources
                .immediates_buffer
                .unwrap_or_default()
                .saturating_add(1),
        );
        variants.push(resource);

        let mut array = base.clone();
        array.binding_array_lengths.push(BindingArrayLength {
            group: 7,
            binding: 9,
            length: 3,
        });
        variants.push(array);

        let mut vertex = base.clone();
        vertex.vertex_buffers.push(VertexBuffer {
            id: 29,
            stride: 4,
            step_mode: StepMode::Instance,
            attributes: vec![VertexAttribute {
                shader_location: 31,
                offset: 0,
                format: "float32".to_owned(),
            }],
        });
        variants.push(vertex);

        let mut topology = base.clone();
        topology.primitive_topology = "point_list".to_owned();
        topology.allow_and_force_point_size = true;
        variants.push(topology);

        let mut language = base.clone();
        language.msl_version = [2, 1];
        language.compile_options.language_version = [2, 1];
        variants.push(language);

        let mut workgroup = base.clone();
        workgroup.zero_initialize_workgroup_memory = !workgroup.zero_initialize_workgroup_memory;
        variants.push(workgroup);

        for mutate in [
            |checks: &mut RuntimeChecks| checks.bounds_checks = !checks.bounds_checks,
            |checks: &mut RuntimeChecks| checks.force_loop_bounding = !checks.force_loop_bounding,
            |checks: &mut RuntimeChecks| {
                checks.ray_query_initialization_tracking = !checks.ray_query_initialization_tracking
            },
            |checks: &mut RuntimeChecks| {
                checks.task_shader_dispatch_tracking = !checks.task_shader_dispatch_tracking
            },
            |checks: &mut RuntimeChecks| {
                checks.mesh_shader_primitive_indices_clamp =
                    !checks.mesh_shader_primitive_indices_clamp
            },
            |checks: &mut RuntimeChecks| checks.int_div_checks = !checks.int_div_checks,
        ] {
            let mut checks = base.clone();
            mutate(&mut checks.runtime_checks);
            variants.push(checks);
        }

        let mut compile = base.clone();
        compile.compile_options.preserve_invariance_expected =
            !compile.compile_options.preserve_invariance_expected;
        variants.push(compile);

        let mut limits = base.clone();
        limits.task_dispatch_limits.max_mesh_workgroups_total = limits
            .task_dispatch_limits
            .max_mesh_workgroups_total
            .saturating_add(1);
        variants.push(limits);

        let mut keys = BTreeSet::from([canonical_key_sha256(&base).unwrap()]);
        for variant in variants {
            assert!(keys.insert(canonical_key_sha256(&variant).unwrap()));
        }
    }

    #[test]
    fn duplicate_artifact_keys_are_rejected() {
        let root = fixture_root();
        let path = root.path().join(INPUT);
        let mut catalog: Catalog = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let mut duplicate = catalog.artifacts[0].clone();
        duplicate.id = "same-inputs-different-name".to_owned();
        catalog.artifacts.push(duplicate);
        write_catalog_and_reviewed_inventory(root.path(), &catalog);
        let error = match render(root.path()) {
            Ok(_) => panic!("duplicate artifact unexpectedly rendered"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("duplicate canonical artifact key")
        );
    }

    #[test]
    fn reviewed_inventory_rejects_removed_logical_permutation() {
        let root = fixture_root();
        let path = root.path().join(INPUT);
        let mut catalog: Catalog = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        catalog.artifacts.clear();
        fs::write(path, serde_json::to_vec_pretty(&catalog).unwrap()).unwrap();

        let error = match generate(root.path()) {
            Ok(_) => panic!("removed logical permutation unexpectedly generated"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("reviewed logical pipeline permutation inventory changed")
        );
    }

    #[test]
    fn reviewed_inventory_counts_logical_aliases_separately_from_compiler_artifacts() {
        let root = fixture_root();
        let path = root.path().join(INPUT);
        let mut catalog: Catalog = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let mut alias = catalog.artifacts[0].clone();
        alias.id = "reviewed-logical-alias".to_owned();
        let alias_source = PathBuf::from("tools/apple-msl-catalog/alias.wgsl");
        fs::copy(
            root.path().join(&alias.source.path),
            root.path().join(&alias_source),
        )
        .unwrap();
        alias.source.path = alias_source.to_string_lossy().into_owned();
        catalog.artifacts.push(alias);
        write_catalog_and_reviewed_inventory(root.path(), &catalog);

        let inventory = reviewed_inventory(&catalog).unwrap();
        assert_eq!(inventory.logical_permutations.count, 2);
        assert_eq!(inventory.compiler_artifacts.count, 1);
        generate(root.path()).unwrap();
        let generated_msl_count = fs::read_dir(root.path().join(OUTPUT_DIR))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "metal"))
            .count();
        assert_eq!(generated_msl_count, 1);

        catalog.artifacts.pop();
        fs::write(path, serde_json::to_vec_pretty(&catalog).unwrap()).unwrap();
        let error = match generate(root.path()) {
            Ok(_) => panic!("removed logical alias unexpectedly generated"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("reviewed logical pipeline permutation inventory changed")
        );
    }

    #[test]
    fn checker_rejects_missing_stale_and_unreferenced_outputs() {
        let root = fixture_root();
        generate(root.path()).unwrap();
        let output = root.path().join(OUTPUT_DIR);
        let manifest: GeneratedManifest =
            serde_json::from_str(&fs::read_to_string(output.join(MANIFEST)).unwrap()).unwrap();
        let metal = output.join(&manifest.artifacts[0].msl_path);

        fs::write(&metal, "stale").unwrap();
        assert!(
            check(root.path())
                .unwrap_err()
                .to_string()
                .contains("stale generated artifact")
        );

        generate(root.path()).unwrap();
        fs::remove_file(&metal).unwrap();
        assert!(
            check(root.path())
                .unwrap_err()
                .to_string()
                .contains("missing or unreferenced")
        );

        generate(root.path()).unwrap();
        fs::write(output.join("orphan.metal"), "orphan").unwrap();
        assert!(
            check(root.path())
                .unwrap_err()
                .to_string()
                .contains("missing or unreferenced")
        );
    }
}
