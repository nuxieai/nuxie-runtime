use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
    sync::mpsc,
};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const CATALOG_PATH: &str = "tools/apple-msl-catalog/catalog.json";
const GENERATED_DIR: &str = "crates/nuxie-renderer/apple-msl-catalog";
const GENERATED_MANIFEST: &str = "crates/nuxie-renderer/apple-msl-catalog/manifest.json";
const SURFACE_PRESENT_SOURCE: &str = "crates/nuxie-renderer/src/surface_present.wgsl";
const SUPPORTED_MSL_VERSIONS: [[u8; 2]; 5] = [[2, 4], [3, 0], [3, 1], [3, 2], [4, 0]];
const LIVE_MSL_VERSION: [u8; 2] = [4, 0];

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
    artifacts: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct Manifest {
    schema_version: u32,
    artifacts: Vec<ManifestArtifact>,
    aliases: Vec<ManifestAlias>,
}

#[derive(Clone, Deserialize)]
struct ManifestArtifact {
    key_sha256: String,
    msl_version: [u8; 2],
    msl_path: String,
    msl_sha256: String,
    translated_entry_point: String,
    workgroup_size: [u32; 3],
}

#[derive(Deserialize)]
struct ManifestAlias {
    id: String,
    source_pipeline_id: String,
    msl_version: [u8; 2],
    key_sha256: String,
}

struct PhysicalArtifact {
    label: String,
    path: String,
    entry_point: String,
    workgroup_size: [u32; 3],
    source: String,
}

fn main() {
    #[cfg(not(target_os = "macos"))]
    panic!("apple-msl-no-wgsl-probe requires macOS and a real Metal device");

    #[cfg(target_os = "macos")]
    if let Err(error) = pollster::block_on(run()) {
        panic!("Apple MSL catalog execution probe failed: {error:#}");
    }
}

#[cfg(target_os = "macos")]
async fn run() -> Result<()> {
    let root = env::current_dir().context("resolve repository root")?;
    let catalog: Catalog = serde_json::from_slice(
        &fs::read(root.join(CATALOG_PATH)).context("read Apple MSL input catalog")?,
    )
    .context("parse Apple MSL input catalog")?;
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(root.join(GENERATED_MANIFEST)).context("read generated Apple MSL manifest")?,
    )
    .context("parse generated Apple MSL manifest")?;
    let physical = verify_catalog(&root, &catalog, &manifest)?;

    let flags = wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION;
    ensure!(
        !flags.contains(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL),
        "probe enabled wgpu's indirect-call WGSL helper"
    );
    ensure!(
        !flags.contains(wgpu::InstanceFlags::AUTOMATIC_TIMESTAMP_NORMALIZATION),
        "probe enabled wgpu's timestamp-normalization WGSL helper"
    );

    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::METAL;
    // Retain ordinary validation while preventing wgpu-core from creating
    // either of its own WGSL-backed helper pipelines.
    descriptor.flags = flags;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .context("a macOS Metal adapter is required")?;
    ensure!(
        adapter.get_info().backend == wgpu::Backend::Metal,
        "probe selected a non-Metal adapter"
    );
    ensure!(
        adapter
            .features()
            .contains(wgpu::Features::PASSTHROUGH_SHADERS),
        "the Metal adapter does not advertise PASSTHROUGH_SHADERS"
    );

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("committed Apple MSL catalog probe"),
            required_features: wgpu::Features::PASSTHROUGH_SHADERS,
            ..Default::default()
        })
        .await
        .context("request a Metal device with passthrough shaders")?;

    let modules = compile_all(&device, &physical).await?;
    execute_surface_present(&device, &queue, &root, &catalog, &manifest, &modules).await?;

    println!(
        "PASS: validated {} logical MSL aliases, compiled {} committed physical MSL artifacts, and executed both SurfacePresent alpha paths on Metal without WGSL input support",
        manifest.aliases.len(),
        modules.len()
    );
    Ok(())
}

fn verify_catalog(
    root: &Path,
    catalog: &Catalog,
    manifest: &Manifest,
) -> Result<BTreeMap<String, PhysicalArtifact>> {
    ensure!(
        catalog.schema_version == 1 && manifest.schema_version == 2,
        "unsupported Apple MSL catalog or manifest schema"
    );
    let catalog_ids: BTreeSet<_> = catalog
        .artifacts
        .iter()
        .map(stable_source_pipeline_id)
        .collect::<Result<_>>()?;
    let alias_ids: BTreeSet<_> = manifest
        .aliases
        .iter()
        .map(|alias| alias.id.as_str())
        .collect();
    ensure!(
        catalog_ids.len() == catalog.artifacts.len(),
        "input catalog contains duplicate logical artifact ids"
    );
    ensure!(
        alias_ids.len() == manifest.aliases.len(),
        "generated manifest contains duplicate alias ids"
    );

    let expected_aliases: BTreeSet<_> = catalog_ids
        .iter()
        .flat_map(|id| {
            SUPPORTED_MSL_VERSIONS
                .into_iter()
                .map(move |version| (id.clone(), version))
        })
        .collect();
    let actual_aliases: BTreeSet<_> = manifest
        .aliases
        .iter()
        .map(|alias| (alias.source_pipeline_id.clone(), alias.msl_version))
        .collect();
    ensure!(
        actual_aliases.len() == manifest.aliases.len(),
        "generated manifest contains duplicate source-pipeline/MSL-version aliases"
    );
    ensure!(
        expected_aliases == actual_aliases,
        "generated aliases are not the exact catalog pipeline × supported MSL version cross-product"
    );

    let artifact_keys: BTreeSet<_> = manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.key_sha256.as_str())
        .collect();
    ensure!(
        artifact_keys.len() == manifest.artifacts.len(),
        "generated manifest contains duplicate physical artifact keys"
    );
    let referenced_keys: BTreeSet<_> = manifest
        .aliases
        .iter()
        .map(|alias| alias.key_sha256.as_str())
        .collect();
    ensure!(
        artifact_keys == referenced_keys,
        "generated manifest contains missing or unreferenced physical artifacts"
    );
    let artifacts_by_key: BTreeMap<_, _> = manifest
        .artifacts
        .iter()
        .map(|artifact| (artifact.key_sha256.as_str(), artifact))
        .collect();
    for alias in &manifest.aliases {
        let artifact = artifacts_by_key
            .get(alias.key_sha256.as_str())
            .with_context(|| {
                format!(
                    "alias {} references missing physical key {}",
                    alias.id, alias.key_sha256
                )
            })?;
        ensure!(
            artifact.msl_version == alias.msl_version,
            "alias {} MSL version differs from physical artifact {}",
            alias.id,
            alias.key_sha256
        );
        ensure!(
            alias.id
                == format!(
                    "{}-msl-{}-{}",
                    alias.source_pipeline_id, alias.msl_version[0], alias.msl_version[1]
                ),
            "alias {} does not encode its source pipeline and MSL version",
            alias.id
        );
    }

    let mut physical = BTreeMap::<String, PhysicalArtifact>::new();
    for artifact in &manifest.artifacts {
        ensure!(
            artifact.msl_path == format!("{}.metal", artifact.key_sha256),
            "{}: MSL path does not match its canonical key",
            artifact.key_sha256
        );
        let path = root.join(GENERATED_DIR).join(&artifact.msl_path);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("read committed MSL artifact {}", path.display()))?;
        ensure!(
            sha256(source.as_bytes()) == artifact.msl_sha256,
            "{}: committed MSL digest differs from the manifest",
            artifact.key_sha256
        );
        ensure!(
            !artifact.translated_entry_point.is_empty()
                && source.contains(&format!(" {}(", artifact.translated_entry_point)),
            "{}: translated entry point {:?} is absent from committed MSL",
            artifact.key_sha256,
            artifact.translated_entry_point
        );
        ensure!(
            physical
                .insert(
                    artifact.key_sha256.clone(),
                    PhysicalArtifact {
                        label: artifact.key_sha256.clone(),
                        path: artifact.msl_path.clone(),
                        entry_point: artifact.translated_entry_point.clone(),
                        workgroup_size: artifact.workgroup_size,
                        source,
                    },
                )
                .is_none(),
            "duplicate physical artifact key {}",
            artifact.key_sha256
        );
    }

    let committed: BTreeSet<_> = fs::read_dir(root.join(GENERATED_DIR))
        .context("read committed Apple MSL directory")?
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
    let referenced: BTreeSet<_> = physical
        .values()
        .map(|artifact| artifact.path.clone())
        .collect();
    ensure!(
        committed == referenced,
        "committed and manifest-referenced physical MSL artifacts differ"
    );
    Ok(physical)
}

#[cfg(target_os = "macos")]
async fn compile_all(
    device: &wgpu::Device,
    physical: &BTreeMap<String, PhysicalArtifact>,
) -> Result<BTreeMap<String, wgpu::ShaderModule>> {
    let mut modules = BTreeMap::new();
    for artifact in physical.values() {
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let entry_points = [wgpu::PassthroughShaderEntryPoint {
            name: Cow::Borrowed(&artifact.entry_point),
            workgroup_size: artifact.workgroup_size.into(),
        }];
        // SAFETY: Each source and translated entry point comes from the pinned
        // generator manifest, whose digest was checked immediately above. The
        // probe does not accept customer-authored source or infer an interface.
        let module = unsafe {
            device.create_shader_module_passthrough(wgpu::ShaderModuleDescriptorPassthrough {
                label: Some(&artifact.label),
                entry_points: Cow::Borrowed(&entry_points),
                msl: Some(Cow::Borrowed(&artifact.source)),
                ..Default::default()
            })
        };
        if let Some(error) = scope.pop().await {
            anyhow::bail!(
                "Metal rejected committed artifact {} ({}): {error}",
                artifact.label,
                artifact.path
            );
        }
        modules.insert(artifact.label.clone(), module);
    }
    ensure!(
        modules.len() == physical.len(),
        "not every physical MSL artifact was compiled"
    );
    Ok(modules)
}

#[cfg(target_os = "macos")]
async fn execute_surface_present(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    root: &Path,
    catalog: &Catalog,
    manifest: &Manifest,
    modules: &BTreeMap<String, wgpu::ShaderModule>,
) -> Result<()> {
    let vertex = surface_artifact(catalog, manifest, "vertex", "vertex_main")?;
    let straight = surface_artifact(catalog, manifest, "fragment", "fragment_straight_alpha")?;
    let premultiplied = surface_artifact(
        catalog,
        manifest,
        "fragment",
        "fragment_premultiplied_alpha",
    )?;
    // Re-read through the joined records so this execution proof is coupled to
    // the committed catalog, not to a synthetic fallback shader.
    for artifact in [vertex, straight, premultiplied] {
        ensure!(
            root.join(GENERATED_DIR).join(&artifact.msl_path).is_file(),
            "SurfacePresent artifact {} is not committed",
            artifact.msl_path
        );
    }

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("committed SurfacePresent bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("committed SurfacePresent pipeline layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let vertex_module = modules
        .get(&vertex.key_sha256)
        .context("compiled SurfacePresent vertex module")?;
    let straight_pipeline = create_present_pipeline(
        device,
        &pipeline_layout,
        vertex_module,
        &vertex.translated_entry_point,
        modules
            .get(&straight.key_sha256)
            .context("compiled straight-alpha fragment module")?,
        &straight.translated_entry_point,
        "committed SurfacePresent straight-alpha pipeline",
    );
    let premultiplied_pipeline = create_present_pipeline(
        device,
        &pipeline_layout,
        vertex_module,
        &vertex.translated_entry_point,
        modules
            .get(&premultiplied.key_sha256)
            .context("compiled premultiplied-alpha fragment module")?,
        &premultiplied.translated_entry_point,
        "committed SurfacePresent premultiplied-alpha pipeline",
    );

    let extent = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    let source = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("committed SurfacePresent half-alpha red source"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        source.as_image_copy(),
        &[0x00, 0x00, 0x80, 0x80],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        extent,
    );
    let source_view = source.create_view(&Default::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("committed SurfacePresent nearest sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("committed SurfacePresent bind group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&source_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let straight_pixel = render_present_pixel(device, queue, &straight_pipeline, &group)?;
    let premultiplied_pixel = render_present_pixel(device, queue, &premultiplied_pipeline, &group)?;
    if let Some(error) = scope.pop().await {
        anyhow::bail!("Metal rejected committed SurfacePresent execution: {error}");
    }
    ensure!(
        straight_pixel == [0x00, 0x00, 0xff, 0x80],
        "straight-alpha committed MSL produced {straight_pixel:02x?}"
    );
    ensure!(
        premultiplied_pixel == [0x00, 0x00, 0x80, 0x80],
        "premultiplied-alpha committed MSL produced {premultiplied_pixel:02x?}"
    );
    Ok(())
}

fn surface_artifact<'a>(
    catalog: &Catalog,
    manifest: &'a Manifest,
    stage: &str,
    entry_point: &str,
) -> Result<&'a ManifestArtifact> {
    let matches: Vec<_> = catalog
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact
                .pointer("/source/path")
                .and_then(serde_json::Value::as_str)
                == Some(SURFACE_PRESENT_SOURCE)
                && artifact.get("stage").and_then(serde_json::Value::as_str) == Some(stage)
                && artifact
                    .get("entry_point")
                    .and_then(serde_json::Value::as_str)
                    == Some(entry_point)
        })
        .collect();
    ensure!(
        matches.len() == 1,
        "expected exactly one SurfacePresent {stage} {entry_point:?} catalog artifact, found {}",
        matches.len()
    );
    let source_pipeline_id = stable_source_pipeline_id(matches[0])?;
    let aliases: Vec<_> = manifest
        .aliases
        .iter()
        .filter(|alias| {
            alias.source_pipeline_id == source_pipeline_id && alias.msl_version == LIVE_MSL_VERSION
        })
        .collect();
    ensure!(
        aliases.len() == 1,
        "expected exactly one SurfacePresent {source_pipeline_id} MSL 4.0 alias, found {}",
        aliases.len()
    );
    let key = &aliases[0].key_sha256;
    manifest
        .artifacts
        .iter()
        .find(|artifact| &artifact.key_sha256 == key)
        .with_context(|| format!("SurfacePresent alias references missing physical key {key}"))
}

#[cfg(target_os = "macos")]
fn create_present_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vertex: &wgpu::ShaderModule,
    vertex_entry: &str,
    fragment: &wgpu::ShaderModule,
    fragment_entry: &str,
    label: &str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: vertex,
            entry_point: Some(vertex_entry),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: fragment,
            entry_point: Some(fragment_entry),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(target_os = "macos")]
fn render_present_pixel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    group: &wgpu::BindGroup,
) -> Result<[u8; 4]> {
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("committed SurfacePresent execution target"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("committed SurfacePresent execution readback"),
        size: wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("committed SurfacePresent execution encoder"),
    });
    {
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &target_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("committed SurfacePresent execution pass"),
            color_attachments: &attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
                rows_per_image: Some(1),
            },
        },
        target.size(),
    );
    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("wait for committed SurfacePresent execution")?;
    receiver
        .recv()
        .context("receive committed SurfacePresent readback result")?
        .context("map committed SurfacePresent readback")?;
    let mapped = slice
        .get_mapped_range()
        .context("read committed SurfacePresent result")?;
    Ok(mapped[..4]
        .try_into()
        .expect("the mapped readback is at least four bytes"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn stable_source_pipeline_id(artifact: &serde_json::Value) -> Result<String> {
    let id = artifact
        .get("id")
        .and_then(serde_json::Value::as_str)
        .context("catalog artifact has no string id")?;
    let stem = id.rsplit_once('-').map_or(id, |(stem, _)| stem);
    let mut key = artifact.clone();
    let object = key
        .as_object_mut()
        .context("catalog artifact is not an object")?;
    object.remove("id");
    object.remove("msl_version");
    object
        .get_mut("compile_options")
        .and_then(serde_json::Value::as_object_mut)
        .context("catalog artifact compile_options is not an object")?
        .remove("language_version");
    let digest = sha256(&serde_json::to_vec(&key)?);
    Ok(format!("{stem}-{}", &digest[..12]))
}
