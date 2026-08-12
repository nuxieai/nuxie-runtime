use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use nuxie_renderer::{RenderMode, WgpuFactory, builtin_shader_capture_inventory};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const CATALOG: &str = "tools/apple-msl-catalog/catalog.json";

#[derive(Deserialize, Serialize)]
struct Catalog {
    schema_version: u32,
    artifacts: Vec<Value>,
}

#[derive(Deserialize)]
struct CommittedCatalog {
    schema_version: u32,
    artifacts: Vec<Value>,
}

#[derive(Deserialize)]
struct CaptureEnvelope {
    schema_version: u32,
    identity: Identity,
    artifact: Value,
    translation: Translation,
}

#[derive(Deserialize)]
struct Identity {
    debug_name: Option<String>,
    source_sha256: Option<String>,
}

#[derive(Deserialize)]
struct Translation {
    msl_source: String,
    msl_sha256: String,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let first = args.next();
    let (check, root_arg) = if first.as_deref() == Some("--check") {
        (true, args.next())
    } else {
        (false, first)
    };
    let root = root_arg
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let captures = args.next().map(PathBuf::from).unwrap_or_else(|| {
        env::var_os("NUXIE_APPLE_MSL_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target/apple-msl-capture"))
    });
    if args.next().is_some() {
        bail!("usage: apple-msl-capture [--check] [repository-root] [capture-directory]");
    }

    ensure!(
        cfg!(target_os = "macos"),
        "Apple MSL capture must run on macOS with a Metal adapter"
    );
    fs::create_dir_all(&captures)?;
    clear_captures(&captures)?;
    // SAFETY: this process sets the variable before creating any thread or
    // initializing wgpu. No concurrent environment access occurs here.
    unsafe { env::set_var("NUXIE_APPLE_MSL_CAPTURE_DIR", &captures) };

    // Construction eagerly creates all built-in pipeline families. Running
    // both modes covers the conditional atomic and MSAA permutations.
    let factory = WgpuFactory::new_with_mode(8, 8, RenderMode::Msaa)
        .context("create MSAA pipeline catalog on Metal")?;
    factory.capture_builtin_present_pipeline_variants();
    WgpuFactory::new_with_mode(8, 8, RenderMode::ClockwiseAtomic)
        .context("create clockwise-atomic pipeline catalog on Metal")?;
    WgpuFactory::new_with_forced_vertex_storage_polyfill(8, 8, RenderMode::Msaa)
        .context("create vertex-storage-polyfill MSAA pipeline catalog on Metal")?;

    let builtin_identities: BTreeMap<_, _> = builtin_shader_capture_inventory()
        .into_iter()
        .map(|identity| {
            (
                (identity.label.to_owned(), sha256(identity.wgsl.as_bytes())),
                identity.source_path,
            )
        })
        .collect();
    let mut artifacts = Vec::new();
    let mut ids = BTreeSet::new();
    let mut captured_source_paths = BTreeSet::new();
    for path in capture_files(&captures)? {
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let capture: CaptureEnvelope =
            serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
        ensure!(capture.schema_version == 1, "unsupported capture schema");
        ensure!(
            sha256(capture.translation.msl_source.as_bytes()) == capture.translation.msl_sha256,
            "{}: captured MSL digest mismatch",
            path.display()
        );
        let digest = capture.identity.source_sha256.with_context(|| {
            format!(
                "{} ({:?}) lacks debug WGSL identity; capture must use a DEBUG wgpu instance",
                path.display(),
                capture.identity.debug_name
            )
        })?;
        let debug_name = capture
            .identity
            .debug_name
            .as_deref()
            .context("captured translation has no renderer shader label")?;
        let source_path = builtin_identities
            .get(&(debug_name.to_owned(), digest.clone()))
            .with_context(|| {
                format!(
                    "capture contains a shader not owned by the built-in catalog: {debug_name:?} ({digest})"
                )
            })?;
        ensure!(
            sha256(&fs::read(root.join(source_path))?) == digest,
            "typed source path {source_path} does not match {debug_name:?}"
        );
        let mut artifact = capture.artifact;
        let object = artifact
            .as_object_mut()
            .context("captured artifact is not an object")?;
        let source = object
            .get_mut("source")
            .and_then(Value::as_object_mut)
            .context("captured source identity is not an object")?;
        source.insert("path".to_owned(), Value::String((*source_path).to_owned()));
        captured_source_paths.insert(*source_path);
        source.insert("sha256".to_owned(), Value::String(digest));
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .context("captured artifact has no id")?;
        ensure!(ids.insert(id.to_owned()), "duplicate artifact id {id}");
        artifacts.push(artifact);
    }
    ensure!(
        !artifacts.is_empty(),
        "Metal run captured no shader translations"
    );
    let inventory_source_paths: BTreeSet<_> = builtin_identities.values().copied().collect();
    let uncaptured: BTreeSet<_> = inventory_source_paths
        .difference(&captured_source_paths)
        .copied()
        .collect();
    let expected_uncaptured: BTreeSet<_> = [
        // This renderer family is retained for parity work but is not runtime
        // reachable while WEBGPU_SUPPORTS_CLOCKWISE_ATOMIC_MODE is false.
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_clip.webgpu_fixedcolor_frag.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_clip_interior_triangles.webgpu_fixedcolor_frag.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles.webgpu_vert.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles_borrowed.webgpu_frag.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_interior_triangles_sampled_clip.webgpu_fixedcolor_frag.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path.webgpu_fixedcolor_frag.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path.webgpu_vert.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path_borrowed.webgpu_frag.wgsl",
        "crates/nuxie-renderer/src/generated/clockwise_atomic_draw_path_sampled_clip.webgpu_fixedcolor_frag.wgsl",
    ]
    .into_iter()
    .collect();
    ensure!(
        uncaptured == expected_uncaptured,
        "built-in capture coverage changed: expected uncaptured {expected_uncaptured:?}, found {uncaptured:?}"
    );
    artifacts.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));

    let catalog_path = root.join(CATALOG);
    let mut bytes = serde_json::to_vec_pretty(&Catalog {
        schema_version: 1,
        artifacts,
    })?;
    bytes.push(b'\n');
    if check {
        let committed: CommittedCatalog = serde_json::from_slice(&fs::read(&catalog_path)?)?;
        ensure!(
            committed.schema_version == 1,
            "unsupported committed catalog schema"
        );
        let committed_keys = version_neutral_artifact_keys(&committed.artifacts)?;
        let captured: Catalog = serde_json::from_slice(&bytes)?;
        let captured_keys = version_neutral_artifact_keys(&captured.artifacts)?;
        ensure!(
            captured_keys == committed_keys,
            "live Metal pipeline capture differs from committed catalog input"
        );
        println!("live Apple MSL capture matches the committed pipeline inventory");
        return Ok(());
    }
    let temporary = catalog_path.with_extension("json.tmp");
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, &catalog_path)?;
    println!("captured Apple MSL catalog at {}", catalog_path.display());
    Ok(())
}

fn version_neutral_artifact_keys(artifacts: &[Value]) -> Result<BTreeSet<Vec<u8>>> {
    let keys: BTreeSet<_> = artifacts
        .iter()
        .map(|artifact| {
            let mut artifact = artifact.clone();
            let object = artifact
                .as_object_mut()
                .context("captured artifact is not an object")?;
            object.remove("id");
            object.remove("msl_version");
            object
                .get_mut("compile_options")
                .and_then(Value::as_object_mut)
                .context("captured compile options are not an object")?
                .remove("language_version");
            serde_json::to_vec(&artifact).context("serialize version-neutral capture key")
        })
        .collect::<Result<_>>()?;
    ensure!(
        keys.len() == artifacts.len(),
        "capture contains duplicate version-neutral pipeline inputs"
    );
    Ok(keys)
}

fn clear_captures(directory: &Path) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
            || path
                .file_name()
                .is_some_and(|name| name == "_capture_error.txt")
        {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn capture_files(directory: &Path) -> Result<Vec<PathBuf>> {
    let error = directory.join("_capture_error.txt");
    ensure!(
        !error.exists(),
        "HAL capture failed: {}",
        fs::read_to_string(&error).unwrap_or_default()
    );
    let mut paths: Vec<_> = fs::read_dir(directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    Ok(paths)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
