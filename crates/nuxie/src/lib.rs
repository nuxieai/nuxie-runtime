//! Public Rust API facade for loading and drawing Rive files.
//!
//! This crate keeps the user-facing surface narrow while the lower-level
//! crates continue to carry the import, graph, runtime, and renderer details.

use std::{collections::BTreeMap, sync::Arc};

#[cfg(feature = "scripting")]
use std::{cell::RefCell, collections::VecDeque};

use anyhow::{Context, Result, bail};
use nuxie_binary::RuntimeFile;
#[cfg(not(feature = "scripting"))]
use nuxie_binary::read_runtime_file as read_runtime_file_for_facade;
#[cfg(feature = "scripting")]
use nuxie_binary::read_runtime_file_with_scripting as read_runtime_file_for_facade;
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_runtime::{
    ArtboardInstance as RuntimeArtboardInstance, RuntimeGeometryCache,
    RuntimeOwnedViewModelInstance, RuntimeRenderPaintCache, RuntimeRenderPathCache,
    preallocate_render_paint_cache_for_artboard_tree_with_external_images,
};

mod scene;

pub use scene::*;

pub use nuxie_render_api::{
    Aabb, BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageFilter, ImageSampler,
    ImageWrap, Mat2D, PathVerb, RawPath, RecordingFactory, RenderBuffer, RenderBufferFlags,
    RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath, RenderShader,
    Renderer, StrokeCap, StrokeJoin, Vec2D,
};
#[cfg(all(feature = "renderer", target_arch = "wasm32"))]
pub use nuxie_renderer::{
    BrowserBackend, BrowserBackendPreference, BrowserFactory,
    BrowserFactory as DefaultRendererFactory, BrowserFrame, BrowserFrame as DefaultRendererFrame,
    BrowserResizeError, WebGl2Factory, WebGl2Frame,
};
#[cfg(feature = "renderer")]
pub use nuxie_renderer::{
    GpuCanvasRenderPlan, GpuCanvasUniformBuffer, GpuCanvasVertexAttribute, GpuCanvasVertexBuffer,
    GpuCanvasVertexLayout, RenderMode, RendererError, WgpuAdapterInfo, WgpuFactory, WgpuFrame,
    WgpuFrameMetrics,
};
#[cfg(all(feature = "renderer", not(target_arch = "wasm32")))]
pub use nuxie_renderer::{
    WgpuFactory as DefaultRendererFactory, WgpuFrame as DefaultRendererFrame,
};
pub use nuxie_runtime::{
    ExternalFontAssetError, LinearAnimationInstance, NoopScriptHost, RuntimeLayerState,
    RuntimeStateMachineInput, ScriptError, ScriptHost, ScriptInstance, ScriptMethod, ScriptModule,
    ScriptModuleFailure, ScriptValue, ScriptingVm, StateMachineInputInstance,
    StateMachineInputKind, StateMachineInstance, StateMachineReportedEvent,
};

#[cfg(feature = "scripting")]
use nuxie_scripting::vm::ScriptProgram;
#[cfg(feature = "scripting")]
pub use nuxie_scripting::vm::{LuaScriptInstance, ScriptVm};

#[cfg(feature = "scripting")]
#[derive(Debug, Clone)]
struct FileScriptAsset {
    ordinal: usize,
    global_id: u32,
    type_name: &'static str,
    name: String,
    is_module: bool,
    payload: Option<Vec<u8>>,
}

#[cfg(feature = "scripting")]
struct ReadyFileScripts {
    // Programs contain VM-owned function handles, so they must drop before VM.
    programs: BTreeMap<usize, ScriptProgram>,
    vm: ScriptVm,
    factory_domain: usize,
}

#[cfg(feature = "scripting")]
struct FileScriptRuntime {
    assets: Option<Vec<FileScriptAsset>>,
    allow_unsigned_execution: bool,
    ready: Option<ReadyFileScripts>,
}

#[cfg(feature = "scripting")]
impl std::fmt::Debug for FileScriptRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileScriptRuntime")
            .field("assets", &self.assets)
            .field("allow_unsigned_execution", &self.allow_unsigned_execution)
            .field("ready", &self.ready.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "scripting")]
impl FileScriptRuntime {
    fn new(runtime: &RuntimeFile, allow_unsigned_execution: bool) -> Self {
        let assets = allow_unsigned_execution.then(|| {
            runtime
                .scripting_file_assets_with_contents()
                .into_iter()
                .map(|entry| FileScriptAsset {
                    ordinal: entry.ordinal,
                    global_id: entry.asset.id,
                    type_name: entry.asset.type_name,
                    name: entry
                        .asset
                        .string_property("name")
                        .unwrap_or_default()
                        .to_owned(),
                    is_module: entry.asset.bool_property("isModule").unwrap_or(false),
                    payload: entry.contents.map(ToOwned::to_owned),
                })
                .collect()
        });
        Self {
            assets,
            allow_unsigned_execution,
            ready: None,
        }
    }

    fn materialized_assets(
        &self,
    ) -> std::result::Result<&[FileScriptAsset], nuxie_runtime::ScriptError> {
        self.assets.as_deref().ok_or_else(|| {
            nuxie_runtime::ScriptError::new(
                "unsigned script asset catalog is unavailable for an inert File",
            )
        })
    }

    fn build_candidate(
        &self,
        factory: &mut dyn Factory,
    ) -> std::result::Result<ReadyFileScripts, nuxie_runtime::ScriptError> {
        let vm = ScriptVm::new();
        let assets = self.materialized_assets()?;
        let mut pending = assets
            .iter()
            .filter(|asset| asset.type_name == "ScriptAsset" && asset.is_module)
            .collect::<Vec<_>>();

        loop {
            let before = pending.len();
            let mut failures = Vec::new();
            for asset in pending {
                let payload = required_script_payload(asset, "module registration")?;
                if let Err(error) = vm.register_module_with_factory(&asset.name, payload, factory) {
                    failures.push((asset, error));
                }
            }
            if failures.is_empty() {
                break;
            }
            if failures.len() == before {
                let (asset, error) = failures.remove(0);
                return Err(asset_phase_error(asset, "module registration", error));
            }
            pending = failures.into_iter().map(|(asset, _)| asset).collect();
        }

        let mut programs = BTreeMap::new();
        for asset in assets
            .iter()
            .filter(|asset| asset.type_name == "ScriptAsset" && !asset.is_module)
        {
            let payload = required_script_payload(asset, "protocol registration")?;
            let program = vm
                .register_protocol_script_with_factory(&asset.name, payload, factory)
                .map_err(|error| asset_phase_error(asset, "protocol registration", error))?;
            programs.insert(asset.ordinal, program);
        }

        Ok(ReadyFileScripts {
            programs,
            vm,
            factory_domain: render_factory_domain(factory),
        })
    }

    fn prepare_mounts(
        &mut self,
        groups: &[ScriptMountGroup],
        factory: &mut dyn Factory,
    ) -> std::result::Result<PreparedFileScriptMounts, nuxie_runtime::ScriptError> {
        let domain = render_factory_domain(factory);
        if let Some(ready) = self.ready.as_ref() {
            if ready.factory_domain != domain {
                return Err(nuxie_runtime::ScriptError::new(
                    "scripted File was used with a different renderer Factory domain",
                ));
            }
            return Ok(PreparedFileScriptMounts {
                groups: instantiate_script_mounts(ready, groups, factory)?,
                candidate: None,
            });
        }

        // Keep the candidate cold until every concrete occurrence has a
        // generated table and successful init. Any error drops all tables and
        // the candidate VM, leaving this File retryable with zero attachments.
        let candidate = self.build_candidate(factory)?;
        let groups = instantiate_script_mounts(&candidate, groups, factory)?;
        Ok(PreparedFileScriptMounts {
            // Drop table handles before their candidate VM on a failed
            // topology validation.
            groups,
            candidate: Some(candidate),
        })
    }
}

#[cfg(feature = "scripting")]
#[derive(Debug)]
struct ScriptMountTarget {
    component_global_id: u32,
    asset_ordinal: usize,
    asset_name: String,
}

#[cfg(feature = "scripting")]
#[derive(Debug)]
struct ScriptMountGroup {
    path: String,
    graph_global_id: u32,
    targets: Vec<ScriptMountTarget>,
}

#[cfg(feature = "scripting")]
struct PreparedScriptMountGroup {
    graph_global_id: u32,
    scripts: Vec<(u32, Box<dyn ScriptInstance>)>,
}

#[cfg(feature = "scripting")]
struct PreparedFileScriptMounts {
    // Field order is intentional: Lua table handles drop before the cold VM.
    groups: Vec<PreparedScriptMountGroup>,
    candidate: Option<ReadyFileScripts>,
}

#[cfg(feature = "scripting")]
fn required_script_payload<'a>(
    asset: &'a FileScriptAsset,
    phase: &str,
) -> std::result::Result<&'a [u8], nuxie_runtime::ScriptError> {
    asset.payload.as_deref().ok_or_else(|| {
        nuxie_runtime::ScriptError::new(format!(
            "ScriptAsset ordinal {} global {} name '{}' phase {} has no imported FileAssetContents payload",
            asset.ordinal, asset.global_id, asset.name, phase
        ))
    })
}

#[cfg(feature = "scripting")]
fn asset_phase_error(
    asset: &FileScriptAsset,
    phase: &str,
    error: impl std::fmt::Display,
) -> nuxie_runtime::ScriptError {
    nuxie_runtime::ScriptError::new(format!(
        "ScriptAsset ordinal {} global {} name '{}' phase {} failed: {}",
        asset.ordinal, asset.global_id, asset.name, phase, error
    ))
}

#[cfg(feature = "scripting")]
fn render_factory_domain(factory: &mut dyn Factory) -> usize {
    let pointer: *mut dyn Factory = factory;
    pointer as *mut () as usize
}

#[cfg(feature = "scripting")]
fn instantiate_script_mounts(
    ready: &ReadyFileScripts,
    groups: &[ScriptMountGroup],
    factory: &mut dyn Factory,
) -> std::result::Result<Vec<PreparedScriptMountGroup>, nuxie_runtime::ScriptError> {
    let mut prepared = Vec::with_capacity(groups.len());
    for group in groups {
        let mut scripts = Vec::with_capacity(group.targets.len());
        for target in &group.targets {
            let program = ready.programs.get(&target.asset_ordinal).ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "{} ScriptedDrawable global {} references unregistered protocol ordinal {} name '{}'",
                    group.path,
                    target.component_global_id,
                    target.asset_ordinal,
                    target.asset_name
                ))
            })?;
            let mut host = NoopScriptHost;
            let mut script = ready
                .vm
                .instantiate_registered_script_with_factory(program, &mut host, factory)
                .map_err(|error| {
                    nuxie_runtime::ScriptError::new(format!(
                        "{} ScriptedDrawable global {} asset ordinal {} name '{}' phase generator failed: {}",
                        group.path,
                        target.component_global_id,
                        target.asset_ordinal,
                        target.asset_name,
                        error
                    ))
                })?;
            if script.has_method(ScriptMethod::Init).map_err(|error| {
                nuxie_runtime::ScriptError::new(format!(
                    "{} ScriptedDrawable global {} asset ordinal {} name '{}' phase init lookup failed: {}",
                    group.path,
                    target.component_global_id,
                    target.asset_ordinal,
                    target.asset_name,
                    error
                ))
            })? {
                let initialized = script
                    .call_init_with_factory(&mut host, factory)
                    .map_err(|error| {
                        nuxie_runtime::ScriptError::new(format!(
                            "{} ScriptedDrawable global {} asset ordinal {} name '{}' phase init failed: {}",
                            group.path,
                            target.component_global_id,
                            target.asset_ordinal,
                            target.asset_name,
                            error
                        ))
                    })?;
                if !initialized {
                    return Err(nuxie_runtime::ScriptError::new(format!(
                        "{} ScriptedDrawable global {} asset ordinal {} name '{}' phase init returned false or nil",
                        group.path,
                        target.component_global_id,
                        target.asset_ordinal,
                        target.asset_name
                    )));
                }
            }
            scripts.push((target.component_global_id, script));
        }
        prepared.push(PreparedScriptMountGroup {
            graph_global_id: group.graph_global_id,
            scripts,
        });
    }
    Ok(prepared)
}

#[cfg(feature = "scripting")]
fn script_mount_group(
    runtime: &RuntimeFile,
    scripts: &FileScriptRuntime,
    graph: &ArtboardGraph,
    instance: &RuntimeArtboardInstance,
    path: String,
) -> std::result::Result<(bool, ScriptMountGroup), nuxie_runtime::ScriptError> {
    let mut has_scripted_drawable = false;
    let mut targets = Vec::new();
    for component in &graph.components {
        if component.type_name != "ScriptedDrawable" {
            continue;
        }
        has_scripted_drawable = true;
        if !scripts.allow_unsigned_execution {
            return Err(nuxie_runtime::ScriptError::new(format!(
                "{path} contains ScriptedDrawable global {}, but arbitrary imported Files do not execute unsigned bytecode; use File::import_with_unsigned_scripts only for trusted content",
                component.global_id
            )));
        }
        let object = runtime
            .object(component.global_id as usize)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "{path} ScriptedDrawable global {} is absent from the runtime file",
                    component.global_id
                ))
            })?;
        let ordinal = object
            .uint_property("scriptAssetId")
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "{path} ScriptedDrawable global {} has no valid FileAsset ordinal",
                    component.global_id
                ))
            })?;
        let resolved = runtime
            .resolved_file_asset_for_referencer(object)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "{path} ScriptedDrawable global {} cannot resolve FileAsset ordinal {ordinal}",
                    component.global_id
                ))
            })?;
        let asset = scripts.materialized_assets()?.get(ordinal).ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "{path} ScriptedDrawable global {} references absent FileAsset ordinal {ordinal}",
                component.global_id
            ))
        })?;
        if asset.global_id != resolved.id {
            return Err(nuxie_runtime::ScriptError::new(format!(
                "{path} ScriptedDrawable global {} FileAsset ordinal {ordinal} resolved global {}, but catalog contains global {}",
                component.global_id, resolved.id, asset.global_id
            )));
        }
        if asset.type_name != "ScriptAsset" || resolved.type_name != "ScriptAsset" {
            return Err(nuxie_runtime::ScriptError::new(format!(
                "{path} ScriptedDrawable global {} FileAsset ordinal {ordinal} is {}, not ScriptAsset",
                component.global_id, resolved.type_name
            )));
        }
        if asset.is_module {
            return Err(nuxie_runtime::ScriptError::new(format!(
                "{path} ScriptedDrawable global {} references module ScriptAsset ordinal {ordinal} name '{}'",
                component.global_id, asset.name
            )));
        }
        if !instance.has_script_instance_for_global(component.global_id) {
            targets.push(ScriptMountTarget {
                component_global_id: component.global_id,
                asset_ordinal: ordinal,
                asset_name: asset.name.clone(),
            });
        }
    }
    Ok((
        has_scripted_drawable,
        ScriptMountGroup {
            path,
            graph_global_id: graph.global_id,
            targets,
        },
    ))
}

#[cfg(feature = "scripting")]
fn collect_script_mount_groups(
    file: &File,
    root_graph: &ArtboardGraph,
    instance: &mut RuntimeArtboardInstance,
) -> std::result::Result<(bool, Vec<ScriptMountGroup>), nuxie_runtime::ScriptError> {
    let scripts = file.scripts.borrow();
    let (mut has_scripted_drawable, root) = script_mount_group(
        &file.runtime,
        &scripts,
        root_graph,
        instance,
        format!("root graph {}", root_graph.global_id),
    )?;
    let mut groups = vec![root];
    let mut visitor = |depth: usize, graph_global_id: u32, nested: &mut RuntimeArtboardInstance| {
        let graph = file
            .graph
            .artboards
            .iter()
            .find(|candidate| candidate.global_id == graph_global_id)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "occurrence {} depth {depth} references unavailable artboard graph {graph_global_id}",
                    groups.len()
                ))
            })?;
        let path = format!(
            "occurrence {} depth {depth} graph {graph_global_id}",
            groups.len()
        );
        let (has_scripts, group) =
            script_mount_group(&file.runtime, &scripts, graph, nested, path)?;
        has_scripted_drawable |= has_scripts;
        groups.push(group);
        Ok::<(), nuxie_runtime::ScriptError>(())
    };
    instance.try_visit_artboard_tree_instances_mut(&mut visitor)?;
    Ok((has_scripted_drawable, groups))
}

#[cfg(feature = "scripting")]
fn artboard_tree_topology(instance: &mut RuntimeArtboardInstance) -> Vec<u32> {
    let mut topology = vec![instance.graph_global_id()];
    let mut visitor = |_: usize, graph_global_id: u32, _: &mut RuntimeArtboardInstance| {
        topology.push(graph_global_id);
        Ok::<(), std::convert::Infallible>(())
    };
    let result = instance.try_visit_artboard_tree_instances_mut(&mut visitor);
    match result {
        Ok(()) => topology,
        Err(error) => match error {},
    }
}

#[cfg(feature = "scripting")]
fn validate_prepared_script_mount_topology(
    instance: &mut RuntimeArtboardInstance,
    prepared: &[PreparedScriptMountGroup],
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let expected = prepared
        .iter()
        .map(|group| group.graph_global_id)
        .collect::<Vec<_>>();
    let actual = artboard_tree_topology(instance);
    if actual != expected {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "scripted artboard topology changed during atomic bootstrap: expected {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn attach_prepared_script_mounts(
    instance: &mut RuntimeArtboardInstance,
    prepared: Vec<PreparedScriptMountGroup>,
) {
    let mut groups = VecDeque::from(prepared);
    if let Some(root) = groups.pop_front() {
        for (global_id, script) in root.scripts {
            instance.set_script_instance_for_global(global_id, script);
        }
    }
    let mut visitor = |_: usize, _: u32, nested: &mut RuntimeArtboardInstance| {
        if let Some(group) = groups.pop_front() {
            for (global_id, script) in group.scripts {
                nested.set_script_instance_for_global(global_id, script);
            }
        }
        Ok::<(), std::convert::Infallible>(())
    };
    let result = instance.try_visit_artboard_tree_instances_mut(&mut visitor);
    match result {
        Ok(()) => {}
        Err(error) => match error {},
    }
}

#[cfg(feature = "scripting")]
fn flush_scripted_artboard_tree(
    instance: &mut RuntimeArtboardInstance,
    factory: &mut dyn Factory,
) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
    let mut changed = instance
        .flush_script_lifecycle_with_factory(factory)
        .map_err(|error| {
            nuxie_runtime::ScriptError::new(format!(
                "root graph {} script lifecycle failed: {error}",
                instance.graph_global_id()
            ))
        })?;
    let mut visitor = |depth: usize, graph_global_id: u32, nested: &mut RuntimeArtboardInstance| {
        changed |= nested
            .flush_script_lifecycle_with_factory(factory)
            .map_err(|error| {
                nuxie_runtime::ScriptError::new(format!(
                    "nested depth {depth} graph {graph_global_id} script lifecycle failed: {error}"
                ))
            })?;
        Ok::<(), nuxie_runtime::ScriptError>(())
    };
    instance.try_visit_artboard_tree_instances_mut(&mut visitor)?;
    // Each concrete child receives a regular component update after its
    // script update. The facade caller performs the root update immediately
    // after this helper returns.
    let mut visitor = |_: usize, _: u32, nested: &mut RuntimeArtboardInstance| {
        changed |= nested.update_pass();
        Ok::<(), std::convert::Infallible>(())
    };
    let result = instance.try_visit_artboard_tree_instances_mut(&mut visitor);
    match result {
        Ok(()) => {}
        Err(error) => match error {},
    }
    Ok(changed)
}

#[cfg(feature = "scripting")]
fn prepare_scripted_artboard_tree(
    file: &File,
    root_graph: &ArtboardGraph,
    instance: &mut RuntimeArtboardInstance,
    factory: &mut dyn Factory,
) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
    let (has_scripted_drawable, groups) = collect_script_mount_groups(file, root_graph, instance)?;
    if !has_scripted_drawable {
        return Ok(false);
    }
    let mut prepared = file.scripts.borrow_mut().prepare_mounts(&groups, factory)?;
    validate_prepared_script_mount_topology(instance, &prepared.groups)?;

    // Validation is the final fallible step. Publish a cold candidate before
    // attaching its tables so every mounted handle always has a live owner;
    // attachment itself is now an infallible commit over the validated tree.
    if let Some(candidate) = prepared.candidate.take() {
        file.scripts.borrow_mut().ready = Some(candidate);
    }
    attach_prepared_script_mounts(instance, prepared.groups);

    // Facade execution fails closed: every concrete scripted drawable must
    // have an attached table before entering the lower runtime draw path.
    let (_, verified) = collect_script_mount_groups(file, root_graph, instance)?;
    if let Some(group) = verified.iter().find(|group| !group.targets.is_empty()) {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{} still has unattached scripted drawable instances",
            group.path
        )));
    }
    let changed = flush_scripted_artboard_tree(instance, factory)?;

    // Script update refreshes component-list occurrences. A newly materialized
    // child is mounted on the next preparation call, but must not slip through
    // this draw without a table in the meantime.
    let (_, after_lifecycle) = collect_script_mount_groups(file, root_graph, instance)?;
    if let Some(group) = after_lifecycle
        .iter()
        .find(|group| !group.targets.is_empty())
    {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{} materialized an unattached scripted drawable during script lifecycle",
            group.path
        )));
    }
    Ok(changed)
}

#[cfg(feature = "scripting")]
fn queue_root_script_advance(
    file: &File,
    root_graph: &ArtboardGraph,
    instance: &mut RuntimeArtboardInstance,
    elapsed_seconds: f32,
) -> bool {
    let mut has_scripted_drawable = root_graph
        .components
        .iter()
        .any(|component| component.type_name == "ScriptedDrawable");
    let mut visitor = |_: usize, graph_global_id: u32, _: &mut RuntimeArtboardInstance| {
        has_scripted_drawable |= file
            .graph
            .artboards
            .iter()
            .find(|candidate| candidate.global_id == graph_global_id)
            .is_some_and(|graph| {
                graph
                    .components
                    .iter()
                    .any(|component| component.type_name == "ScriptedDrawable")
            });
        Ok::<(), std::convert::Infallible>(())
    };
    let result = instance.try_visit_artboard_tree_instances_mut(&mut visitor);
    match result {
        Ok(()) => {}
        Err(error) => match error {},
    }
    if has_scripted_drawable {
        instance.queue_script_advance(elapsed_seconds);
    }
    has_scripted_drawable && elapsed_seconds != 0.0
}

/// Imported Rive file plus its runtime graph projection.
pub struct File {
    runtime: RuntimeFile,
    graph: GraphFile,
    external_image_assets: BTreeMap<u32, Arc<[u8]>>,
    external_font_assets: BTreeMap<u32, Arc<[u8]>>,
    #[cfg(feature = "scripting")]
    scripts: RefCell<FileScriptRuntime>,
}

/// Allocation limits applied after binary import and before the owned runtime
/// graph is constructed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FileImportLimits {
    max_imported_file_assets: Option<usize>,
}

impl FileImportLimits {
    pub const fn new() -> Self {
        Self {
            max_imported_file_assets: None,
        }
    }

    pub const fn with_max_imported_file_assets(mut self, maximum: usize) -> Self {
        self.max_imported_file_assets = Some(maximum);
        self
    }

    pub const fn max_imported_file_assets(self) -> Option<usize> {
        self.max_imported_file_assets
    }
}

/// Failure to attach host-supplied bytes to an external `ImageAsset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalImageAssetError {
    UnknownAsset { asset_id: u32 },
    WrongAssetKind { asset_id: u32, actual: &'static str },
}

impl std::fmt::Display for ExternalImageAssetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAsset { asset_id } => {
                write!(formatter, "unknown external image asset id {asset_id}")
            }
            Self::WrongAssetKind { asset_id, actual } => write!(
                formatter,
                "external image asset id {asset_id} resolves to {actual}, not ImageAsset"
            ),
        }
    }
}

impl std::error::Error for ExternalImageAssetError {}

impl std::fmt::Debug for File {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = formatter.debug_struct("File");
        debug
            .field("runtime", &self.runtime)
            .field("graph", &self.graph);
        #[cfg(feature = "scripting")]
        debug.field("scripts", &self.scripts);
        debug.finish()
    }
}

impl Clone for File {
    fn clone(&self) -> Self {
        let runtime = self.runtime.clone();
        let graph = self.graph.clone();
        #[cfg(feature = "scripting")]
        let scripts = RefCell::new(FileScriptRuntime::new(
            &runtime,
            self.scripts.borrow().allow_unsigned_execution,
        ));
        Self {
            runtime,
            graph,
            external_image_assets: self.external_image_assets.clone(),
            external_font_assets: self.external_font_assets.clone(),
            #[cfg(feature = "scripting")]
            scripts,
        }
    }
}

impl File {
    /// Import `.riv` bytes and build the runtime graph needed for instancing.
    ///
    /// With scripting enabled, arbitrary imported files remain inert by
    /// default. Use `File::import_with_unsigned_scripts` only after the host
    /// has established trust in the file bytes.
    pub fn import(bytes: &[u8]) -> Result<Self> {
        Self::import_with_limits(bytes, FileImportLimits::new())
    }

    /// Import `.riv` bytes while bounding allocations derived from the parsed
    /// file before constructing the owned runtime graph.
    pub fn import_with_limits(bytes: &[u8], limits: FileImportLimits) -> Result<Self> {
        let runtime = read_runtime_file_for_facade(bytes).context("failed to import Rive file")?;
        Self::from_runtime_with_script_policy_and_limits(runtime, false, limits)
    }

    /// Import `.riv` bytes and allow their unsigned ScriptAsset bytecode to
    /// execute lazily on first factory-bearing advance or draw.
    ///
    /// This is an explicit trust boundary for content the host authored or
    /// authenticated. Arbitrary uploaded or network-provided files should use
    /// [`Self::import`], which refuses to enter the script draw path.
    #[cfg(feature = "scripting")]
    pub fn import_with_unsigned_scripts(bytes: &[u8]) -> Result<Self> {
        let runtime = read_runtime_file_for_facade(bytes).context("failed to import Rive file")?;
        Self::from_runtime_with_script_policy(runtime, true)
    }

    pub(crate) fn from_runtime(runtime: RuntimeFile) -> Result<Self> {
        // RuntimeFile values constructed by Scene are authored in-process and
        // deliberately opt into unsigned editor bytecode execution.
        Self::from_runtime_with_script_policy(runtime, true)
    }

    fn from_runtime_with_script_policy(
        runtime: RuntimeFile,
        _allow_unsigned_execution: bool,
    ) -> Result<Self> {
        Self::from_runtime_with_script_policy_and_limits(
            runtime,
            _allow_unsigned_execution,
            FileImportLimits::new(),
        )
    }

    fn from_runtime_with_script_policy_and_limits(
        runtime: RuntimeFile,
        _allow_unsigned_execution: bool,
        limits: FileImportLimits,
    ) -> Result<Self> {
        if let Some(maximum) = limits.max_imported_file_assets()
            && runtime
                .imported_file_assets_with_contents_bounded(maximum)
                .is_none()
        {
            bail!("Rive file imports more than {maximum} FileAssets");
        }
        let graph = GraphFile::from_runtime_file(&runtime).context("failed to build Rive graph")?;
        Ok(Self {
            #[cfg(feature = "scripting")]
            scripts: RefCell::new(FileScriptRuntime::new(&runtime, _allow_unsigned_execution)),
            runtime,
            graph,
            external_image_assets: BTreeMap::new(),
            external_font_assets: BTreeMap::new(),
        })
    }

    /// Attach host-supplied bytes to an external `ImageAsset` in this file.
    ///
    /// This performs no I/O or decode. `asset_id` is the published, file-wide
    /// `FileAsset.assetId`, not the object id or asset-list ordinal. The bytes
    /// are decoded lazily by the renderer on first draw. Embedded image bytes
    /// remain authoritative if a file carries both forms.
    pub fn attach_image_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalImageAssetError> {
        let Some(asset) = self
            .runtime
            .file_assets()
            .into_iter()
            .find(|asset| asset.uint_property("assetId") == Some(u64::from(asset_id)))
        else {
            return Err(ExternalImageAssetError::UnknownAsset { asset_id });
        };
        if asset.type_name != "ImageAsset" {
            return Err(ExternalImageAssetError::WrongAssetKind {
                asset_id,
                actual: asset.type_name,
            });
        }
        self.external_image_assets
            .insert(asset.id, Arc::<[u8]>::from(bytes));
        Ok(())
    }

    /// Attach host-supplied bytes to an external `FontAsset` in this file.
    ///
    /// This performs no I/O. `asset_id` is the published, file-wide
    /// `FileAsset.assetId`, not the object id or asset-list ordinal. The
    /// identity, asset kind, and both runtime font parsing backends are
    /// validated before this file changes. Every root or nested artboard
    /// instantiated from the file receives the bytes, while embedded font
    /// contents remain authoritative when present.
    pub fn attach_font_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalFontAssetError> {
        let Some(asset) = self
            .runtime
            .file_assets()
            .into_iter()
            .find(|asset| asset.uint_property("assetId") == Some(u64::from(asset_id)))
        else {
            return Err(ExternalFontAssetError::UnknownAsset { asset_id });
        };
        if asset.type_name != "FontAsset" {
            return Err(ExternalFontAssetError::WrongAssetKind {
                asset_id,
                actual: asset.type_name,
            });
        }
        if !nuxie_runtime::embedded_font_is_parseable(&bytes) {
            return Err(ExternalFontAssetError::InvalidFont { asset_id });
        }
        self.external_font_assets
            .insert(asset_id, Arc::<[u8]>::from(bytes));
        Ok(())
    }

    /// Low-level imported file data for advanced integrations.
    pub fn runtime(&self) -> &RuntimeFile {
        &self.runtime
    }

    /// Low-level graph projection for advanced integrations.
    pub fn graph(&self) -> &GraphFile {
        &self.graph
    }

    pub fn artboard_count(&self) -> usize {
        self.graph.artboards.len()
    }

    pub fn artboards(&self) -> impl ExactSizeIterator<Item = Artboard<'_>> + '_ {
        (0..self.artboard_count()).map(|index| Artboard { file: self, index })
    }

    pub fn default_artboard(&self) -> Option<Artboard<'_>> {
        self.artboard(0)
    }

    pub fn artboard(&self, index: usize) -> Option<Artboard<'_>> {
        (index < self.artboard_count()).then_some(Artboard { file: self, index })
    }

    pub fn artboard_named(&self, name: &str) -> Option<Artboard<'_>> {
        let index = self
            .graph
            .artboards
            .iter()
            .position(|artboard| artboard.name.as_deref() == Some(name))?;
        Some(Artboard { file: self, index })
    }
}

fn attach_file_font_assets_to_instance(
    file: &File,
    instance: &mut RuntimeArtboardInstance,
) -> Result<()> {
    for (&asset_id, bytes) in &file.external_font_assets {
        instance
            .attach_external_font_asset_bytes(asset_id, Arc::clone(bytes))
            .with_context(|| format!("failed to attach external font asset {asset_id}"))?;
    }
    Ok(())
}

/// Borrowed handle to an artboard inside an imported [`File`].
#[derive(Debug, Clone, Copy)]
pub struct Artboard<'a> {
    file: &'a File,
    index: usize,
}

impl<'a> Artboard<'a> {
    pub fn index(self) -> usize {
        self.index
    }

    pub fn name(self) -> Option<&'a str> {
        self.graph().name.as_deref()
    }

    pub fn graph(self) -> &'a ArtboardGraph {
        // Safe by construction: every Artboard is created with an index bounds-
        // checked against this same vec (artboards()/artboard()/artboard_named()).
        #[allow(clippy::indexing_slicing)]
        &self.file.graph.artboards[self.index]
    }

    pub fn animation_count(self) -> usize {
        self.graph().animations.len()
    }

    pub fn state_machine_count(self) -> usize {
        self.graph().state_machines.len()
    }

    /// Name of the state machine at `index`, when it has one.
    pub fn state_machine_name(self, index: usize) -> Option<&'a str> {
        self.graph().state_machines.get(index)?.name.as_deref()
    }

    /// Index of the state machine flagged as the artboard default in the
    /// source file, validated against the artboard's state machine list.
    pub fn default_state_machine_index(self) -> Option<usize> {
        let artboard = self.file.runtime.artboard(self.index)?;
        artboard.property("defaultStateMachineId")?;
        let index = usize::try_from(artboard.uint_property("defaultStateMachineId")?).ok()?;
        (index < self.state_machine_count()).then_some(index)
    }

    pub fn instantiate(self) -> Result<ArtboardInstance<'a>> {
        let mut raw = RuntimeArtboardInstance::from_graph_with_artboards(
            &self.file.runtime,
            self.graph(),
            &self.file.graph.artboards,
        )
        .with_context(|| {
            format!(
                "failed to instantiate artboard {}",
                self.name().unwrap_or("<unnamed>")
            )
        })?;
        attach_file_font_assets_to_instance(self.file, &mut raw)?;
        Ok(ArtboardInstance {
            file: self.file,
            artboard_index: self.index,
            raw,
            geometry: RuntimeGeometryCache::default(),
        })
    }
}

/// User-facing artboard instance that keeps file and graph context available.
#[derive(Debug)]
pub struct ArtboardInstance<'a> {
    file: &'a File,
    artboard_index: usize,
    raw: RuntimeArtboardInstance,
    geometry: RuntimeGeometryCache,
}

/// Render resources retained across draws of one [`ArtboardInstance`].
///
/// Create this renderer-neutral handle from [`ArtboardInstance::new_render_cache`].
/// The first draw lazily allocates resources from its factory. A failed image
/// decode discards that candidate allocation so a later draw with the same
/// cache can retry; successfully created resources remain owned until drop.
pub struct ArtboardRenderCache {
    paint: Option<RuntimeRenderPaintCache>,
    path: RuntimeRenderPathCache,
}

fn ensure_render_paint_cache_for_draw(
    retained: &mut Option<RuntimeRenderPaintCache>,
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    external_image_assets: &BTreeMap<u32, Arc<[u8]>>,
    factory: &mut dyn Factory,
) -> std::result::Result<(), ImageDecodeError> {
    if retained.is_some() {
        return Ok(());
    }
    let candidate = preallocate_render_paint_cache_for_artboard_tree_with_external_images(
        runtime,
        artboard,
        artboards,
        factory,
        external_image_assets,
    );
    if let Some(error) = candidate.image_decode_error() {
        return Err(error);
    }
    *retained = Some(candidate);
    Ok(())
}

impl<'a> ArtboardInstance<'a> {
    pub fn artboard(&self) -> Artboard<'a> {
        Artboard {
            file: self.file,
            index: self.artboard_index,
        }
    }

    pub fn raw(&self) -> &RuntimeArtboardInstance {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut RuntimeArtboardInstance {
        &mut self.raw
    }

    pub fn advance_nested_artboards(&mut self, elapsed_seconds: f32) -> bool {
        self.raw.advance_nested_artboards(elapsed_seconds)
    }

    pub fn advance(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = false;
        #[cfg(feature = "scripting")]
        {
            changed |= queue_root_script_advance(
                self.file,
                self.artboard().graph(),
                &mut self.raw,
                elapsed_seconds,
            );
        }
        changed |= self.raw.advance_nested_artboards(elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        changed |= self.raw.update_pass();
        changed
    }

    /// Advance with a renderer factory available to every script lifecycle
    /// phase. Legacy [`Self::advance`] queues exact script steps for the next
    /// draw; this method executes them before the regular update pass and
    /// surfaces script errors immediately. Once this File's scripts bootstrap,
    /// every factory-bearing advance and draw must use that same live Factory
    /// object; a different object is rejected before script execution.
    pub fn try_advance_with_factory(
        &mut self,
        factory: &mut dyn Factory,
        elapsed_seconds: f32,
    ) -> Result<bool> {
        #[cfg(feature = "scripting")]
        let _ = queue_root_script_advance(
            self.file,
            self.artboard().graph(),
            &mut self.raw,
            elapsed_seconds,
        );
        let mut changed = self.raw.advance_nested_artboards(elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        #[cfg(feature = "scripting")]
        {
            changed |= prepare_scripted_artboard_tree(
                self.file,
                self.artboard().graph(),
                &mut self.raw,
                factory,
            )
            .context("failed to advance scripted drawables")?;
        }
        #[cfg(not(feature = "scripting"))]
        let _ = factory;
        changed |= self.raw.update_pass();
        Ok(changed)
    }

    /// Return visible Shape and Text locals under `point`, front to back,
    /// including descendants reached through nested artboards.
    pub fn hit_test(&mut self, point: Vec2D) -> Vec<usize> {
        self.raw.geometry_hit_test(point, &mut self.geometry)
    }

    /// Return visible Shape and Text local-id paths under `point`, front to
    /// back. Direct hits contain one local id; nested hits are prefixed with
    /// their nested-host local ids.
    pub fn hit_test_paths(&mut self, point: Vec2D) -> Vec<Vec<usize>> {
        self.raw.geometry_hit_test_paths(point, &mut self.geometry)
    }

    /// Return exact logical world bounds for one runtime-local object.
    pub fn world_bounds(&mut self, local_id: usize) -> Option<Aabb> {
        self.raw.geometry_world_bounds(local_id, &mut self.geometry)
    }

    /// Return the settled, layout-aware world transform for one runtime-local object.
    pub fn world_transform(&mut self, local_id: usize) -> Option<Mat2D> {
        self.raw
            .geometry_world_transform(local_id, &mut self.geometry)
    }

    /// Return the canonical downstream shaped Text caret in source-artboard
    /// world space for one exact UTF-8 byte boundary.
    ///
    /// A boundary skipped with leading whitespace at a soft wrap snaps to the
    /// next visual line. Static Text does not synthesize a caret after a
    /// trailing newline or other static line separator. CRLF is one authored
    /// separator, so the boundary between its two scalars has no geometry.
    ///
    /// Returns `None` for an offset past the source or inside a UTF-8 scalar;
    /// an unknown local or non-Text object; missing or invalid font data for
    /// the base style or any participating nonempty run; nonfinite layout,
    /// transform, or modifier geometry; and unsupported or unknown overflow.
    /// Geometry v1 supports only `Visible`, `Fit`, and `FitFontSize`; `Hidden`,
    /// `Clipped`, and `Ellipsis` fail closed.
    pub fn text_caret(&mut self, local_id: usize, byte_offset: usize) -> Option<CaretGeometry> {
        let (top, bottom) =
            self.raw
                .geometry_text_caret(local_id, byte_offset, &mut self.geometry)?;
        Some(CaretGeometry { top, bottom })
    }

    /// Return the nearest valid UTF-8 byte caret for one source-artboard
    /// world-space point on shaped Text.
    ///
    /// Returns `None` for a nonfinite point; an unknown local or non-Text
    /// object; unshapeable text; nonfinite layout, transform, or modifier
    /// geometry; a singular/non-invertible world transform; and unsupported or
    /// unknown overflow. Geometry v1 supports only `Visible`, `Fit`, and
    /// `FitFontSize`.
    pub fn text_hit(&mut self, local_id: usize, point: Vec2D) -> Option<usize> {
        self.raw
            .geometry_text_hit(local_id, point, &mut self.geometry)
    }

    /// Return one source-artboard world-space selection rectangle per shaped
    /// line segment covered by an exact UTF-8 byte range.
    ///
    /// Returns an empty result when either endpoint is past the source or
    /// inside a UTF-8 scalar, the range is empty or reversed, the local is
    /// unknown or not Text, the text is unshapeable, layout geometry is
    /// nonfinite, or overflow is unsupported or unknown. Selection starts use
    /// downstream affinity and ends use upstream affinity, including source
    /// whitespace omitted at soft wraps. A trailing static line separator does
    /// not create a selectable final empty line. CRLF is treated as one
    /// authored separator; its internal scalar boundary is not selectable.
    pub fn text_selection_rects(
        &mut self,
        local_id: usize,
        range: std::ops::Range<usize>,
    ) -> Vec<Aabb> {
        self.raw
            .geometry_text_selection_rects(local_id, range, &mut self.geometry)
    }

    pub fn state_machine_instance(&self, index: usize) -> Option<StateMachineInstance> {
        self.raw.state_machine_instance(index)
    }

    /// Instantiate the artboard's default state machine: the one flagged in
    /// the source file when present, otherwise the first state machine.
    pub fn default_state_machine_instance(&self) -> Option<StateMachineInstance> {
        let index = self.artboard().default_state_machine_index().unwrap_or(0);
        self.state_machine_instance(index)
    }

    /// Index of the view model backing this artboard's data binds (the source
    /// `viewModelId`), when it declares one. Artboards with no view model carry
    /// the `0xFFFFFFFF` (-1) sentinel, reported here as `None`.
    pub fn view_model_index(&self) -> Option<usize> {
        let artboard = self.file.runtime.artboard(self.artboard_index)?;
        let view_model_id = artboard.uint_property("viewModelId")?;
        if view_model_id == u32::MAX as u64 {
            return None;
        }
        usize::try_from(view_model_id).ok()
    }

    /// Instantiate this artboard's view model with generated defaults, mirroring
    /// `file->createDefaultViewModelInstance(artboard)` in the C++ runtime.
    /// Returns `None` when the artboard has no view model. Bind the returned
    /// context with [`ArtboardInstance::bind_view_model`] before advancing.
    pub fn instantiate_view_model(&self) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::new(&self.file.runtime, view_model_index)?;
        Some(ViewModelInstance { raw })
    }

    /// Instantiate this artboard's view model from the source instance at
    /// `instance_index` (the order the instances appear in the file). Returns
    /// `None` when the artboard has no view model or the index is out of range.
    pub fn instantiate_view_model_instance(
        &self,
        instance_index: usize,
    ) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::from_instance(
            &self.file.runtime,
            view_model_index,
            instance_index,
        )?;
        Some(ViewModelInstance { raw })
    }

    /// Bind `view_model` to this artboard's own data binds and its nested
    /// artboard contexts, mirroring `artboard->bindViewModelInstance(...)` in
    /// the C++ runtime.
    ///
    /// The context is copied into the data-bind graph, so this must be called
    /// again after mutating `view_model` for the change to reach the next
    /// [`ArtboardInstance::advance`]. Returns whether the binding changed
    /// anything. State-machine-driven binds must additionally be bound through
    /// [`StateMachineInstance::bind_owned_view_model_context`] using
    /// [`ViewModelInstance::raw`].
    ///
    pub fn bind_view_model(&mut self, view_model: &ViewModelInstance) -> bool {
        let mut changed = self
            .raw
            .bind_default_view_model_artboard_list_context(&self.file.runtime);
        changed |= self
            .raw
            .bind_owned_view_model_artboard_context(&self.file.runtime, view_model.raw());
        changed
    }

    /// Advance the scene while driving `state_machine`, mirroring the golden
    /// runner's advance order (state machine, nested artboards, data binds,
    /// update pass). Returns whether anything changed.
    pub fn advance_with_state_machine(
        &mut self,
        state_machine: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_with_state_machines(std::slice::from_mut(state_machine), elapsed_seconds)
    }

    /// Batched state-machine advance for one retained artboard instance.
    ///
    /// Root scripts, nested artboards, data binds, and the update pass run
    /// once for the frame; only the authored machines themselves advance in
    /// caller order.
    pub fn advance_with_state_machines(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
    ) -> bool {
        if state_machines.is_empty() {
            return self.advance(elapsed_seconds);
        }
        let mut changed = false;
        #[cfg(feature = "scripting")]
        {
            changed |= queue_root_script_advance(
                self.file,
                self.artboard().graph(),
                &mut self.raw,
                elapsed_seconds,
            );
        }
        changed |= self
            .raw
            .advance_state_machine_instances_with_nested(state_machines, elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        changed |= self.raw.update_pass();
        changed
    }

    /// Factory-bearing mirror of [`Self::advance_with_state_machine`].
    pub fn try_advance_with_state_machine_and_factory(
        &mut self,
        state_machine: &mut StateMachineInstance,
        elapsed_seconds: f32,
        factory: &mut dyn Factory,
    ) -> Result<bool> {
        self.try_advance_with_state_machines_and_factory(
            std::slice::from_mut(state_machine),
            elapsed_seconds,
            factory,
        )
    }

    /// Factory-bearing mirror of [`Self::advance_with_state_machines`].
    pub fn try_advance_with_state_machines_and_factory(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        factory: &mut dyn Factory,
    ) -> Result<bool> {
        if state_machines.is_empty() {
            return self.try_advance_with_factory(factory, elapsed_seconds);
        }
        let mut changed = false;
        #[cfg(feature = "scripting")]
        {
            changed |= queue_root_script_advance(
                self.file,
                self.artboard().graph(),
                &mut self.raw,
                elapsed_seconds,
            );
        }
        changed |= self
            .raw
            .advance_state_machine_instances_with_nested(state_machines, elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        #[cfg(feature = "scripting")]
        {
            changed |= prepare_scripted_artboard_tree(
                self.file,
                self.artboard().graph(),
                &mut self.raw,
                factory,
            )
            .context("failed to advance scripted drawables")?;
        }
        #[cfg(not(feature = "scripting"))]
        let _ = factory;
        changed |= self.raw.update_pass();
        Ok(changed)
    }

    pub fn draw(&mut self, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<()> {
        let mut cache = self.new_render_cache();
        self.draw_with_render_cache(factory, renderer, &mut cache)
    }

    /// Create a renderer-neutral cache for [`Self::draw_with_render_cache`].
    ///
    /// Render resources and image decoding are deferred until draw. Failed
    /// candidate resources are discarded, leaving this cache reusable. After
    /// the first successful draw, subsequent draws must use a compatible
    /// factory and renderer for the retained resources.
    pub fn new_render_cache(&self) -> ArtboardRenderCache {
        ArtboardRenderCache {
            paint: None,
            path: RuntimeRenderPathCache::default(),
        }
    }

    /// Draw while retaining paint and path handles across frames.
    ///
    /// `cache` must have been created by this instance. Once a draw succeeds,
    /// later draws must use a factory and renderer backed by the same renderer
    /// integration as the retained resources. Scripted Files additionally pin
    /// the exact live Factory object used for their first successful bootstrap.
    pub fn draw_with_render_cache(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
        cache: &mut ArtboardRenderCache,
    ) -> Result<()> {
        let artboard = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .context("artboard instance graph is unavailable")?;
        #[cfg(feature = "scripting")]
        let had_retained_paint = cache.paint.is_some();
        ensure_render_paint_cache_for_draw(
            &mut cache.paint,
            &self.file.runtime,
            artboard,
            &self.file.graph.artboards,
            &self.file.external_image_assets,
            factory,
        )?;
        #[cfg(feature = "scripting")]
        if let Err(error) =
            prepare_scripted_artboard_tree(self.file, artboard, &mut self.raw, factory)
        {
            if !had_retained_paint {
                cache.paint = None;
            }
            return Err(error).context("failed to prepare scripted drawables");
        }
        let paint = cache
            .paint
            .as_mut()
            .context("render paint cache disappeared after successful allocation")?;
        self.raw.update_pass();
        self.raw
            .prepare_static_artboard_tree_paints(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                paint,
                &mut cache.path,
            )
            .context("failed to prepare Rive paints")?;
        self.raw
            .draw_prepared_static_artboard_with_render_cache(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                renderer,
                paint,
                &mut cache.path,
            )
            .context("failed to draw Rive artboard")
    }
}

/// Owning variant of [`ArtboardInstance`] for hosts that cannot hold a
/// borrow of the [`File`] — editors, long-lived embeddings, FFI surfaces.
/// Shares the file via [`Arc`] and owns the runtime instance; a
/// method-for-method mirror of [`ArtboardInstance`] (which stays the
/// zero-overhead choice when a borrow works).
pub struct OwnedArtboardInstance {
    // Raw script-table handles must drop before the final Arc<File> can drop
    // its VM. Field declaration order is drop order.
    raw: RuntimeArtboardInstance,
    geometry: RuntimeGeometryCache,
    file: Arc<File>,
    artboard_index: usize,
}

impl OwnedArtboardInstance {
    /// Instantiate `artboard_index` of `file` as an owning instance.
    pub fn instantiate(file: Arc<File>, artboard_index: usize) -> Result<Self> {
        let mut raw = {
            let artboard = file
                .artboard(artboard_index)
                .with_context(|| format!("artboard index {artboard_index} out of range"))?;
            RuntimeArtboardInstance::from_graph_with_artboards(
                &file.runtime,
                artboard.graph(),
                &file.graph.artboards,
            )
            .with_context(|| {
                format!(
                    "failed to instantiate artboard {}",
                    artboard.name().unwrap_or("<unnamed>")
                )
            })?
        };
        attach_file_font_assets_to_instance(&file, &mut raw)?;
        Ok(Self {
            raw,
            geometry: RuntimeGeometryCache::default(),
            file,
            artboard_index,
        })
    }

    /// Instantiate the file's default artboard as an owning instance.
    pub fn instantiate_default(file: Arc<File>) -> Result<Self> {
        let artboard_index = file
            .default_artboard()
            .context("file has no artboards")?
            .index();
        Self::instantiate(file, artboard_index)
    }

    pub fn file(&self) -> &Arc<File> {
        &self.file
    }

    pub fn artboard(&self) -> Artboard<'_> {
        Artboard {
            file: &self.file,
            index: self.artboard_index,
        }
    }

    pub fn raw(&self) -> &RuntimeArtboardInstance {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut RuntimeArtboardInstance {
        &mut self.raw
    }

    /// Attach host-supplied bytes to an external `FontAsset` in this instance.
    ///
    /// This performs no I/O. `asset_id` is the published, file-wide
    /// `FileAsset.assetId`, not the asset-list ordinal. The runtime validates
    /// the identity, asset kind, and both font parsing backends before changing
    /// the instance tree. Embedded font contents remain authoritative. Existing
    /// and subsequently remounted nested artboards inherit the attachment.
    pub fn attach_font_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalFontAssetError> {
        self.raw
            .attach_external_font_asset_bytes(asset_id, Arc::<[u8]>::from(bytes))
    }

    pub fn advance_nested_artboards(&mut self, elapsed_seconds: f32) -> bool {
        self.raw.advance_nested_artboards(elapsed_seconds)
    }

    pub fn advance(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = false;
        #[cfg(feature = "scripting")]
        if let Some(artboard) = self.file.graph.artboards.get(self.artboard_index) {
            changed |=
                queue_root_script_advance(&self.file, artboard, &mut self.raw, elapsed_seconds);
        }
        changed |= self.raw.advance_nested_artboards(elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        changed |= self.raw.update_pass();
        changed
    }

    /// Owning mirror of [`ArtboardInstance::try_advance_with_factory`],
    /// including its stable live-Factory identity precondition.
    pub fn try_advance_with_factory(
        &mut self,
        factory: &mut dyn Factory,
        elapsed_seconds: f32,
    ) -> Result<bool> {
        #[cfg(feature = "scripting")]
        let artboard = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .context("owned artboard instance graph is unavailable")?;
        #[cfg(feature = "scripting")]
        let _ = queue_root_script_advance(&self.file, artboard, &mut self.raw, elapsed_seconds);
        let mut changed = self.raw.advance_nested_artboards(elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        #[cfg(feature = "scripting")]
        {
            changed |= prepare_scripted_artboard_tree(&self.file, artboard, &mut self.raw, factory)
                .context("failed to advance scripted drawables")?;
        }
        #[cfg(not(feature = "scripting"))]
        let _ = factory;
        changed |= self.raw.update_pass();
        Ok(changed)
    }

    /// Return visible Shape and Text locals under `point`, front to back,
    /// including descendants reached through nested artboards.
    pub fn hit_test(&mut self, point: Vec2D) -> Vec<usize> {
        self.raw.geometry_hit_test(point, &mut self.geometry)
    }

    /// Return visible Shape and Text local-id paths under `point`, front to
    /// back. Direct hits contain one local id; nested hits are prefixed with
    /// their nested-host local ids.
    pub fn hit_test_paths(&mut self, point: Vec2D) -> Vec<Vec<usize>> {
        self.raw.geometry_hit_test_paths(point, &mut self.geometry)
    }

    /// Return exact logical world bounds for one runtime-local object.
    pub fn world_bounds(&mut self, local_id: usize) -> Option<Aabb> {
        self.raw.geometry_world_bounds(local_id, &mut self.geometry)
    }

    /// Return the settled, layout-aware world transform for one runtime-local object.
    pub fn world_transform(&mut self, local_id: usize) -> Option<Mat2D> {
        self.raw
            .geometry_world_transform(local_id, &mut self.geometry)
    }

    /// Return the canonical downstream shaped Text caret in source-artboard
    /// world space for one exact UTF-8 byte boundary.
    ///
    /// This is the owning mirror of [`ArtboardInstance::text_caret`] and has
    /// the same invalid-offset, target-kind, shaping, finite-geometry,
    /// overflow, soft-wrap, and trailing-static-separator behavior.
    pub fn text_caret(&mut self, local_id: usize, byte_offset: usize) -> Option<CaretGeometry> {
        let (top, bottom) =
            self.raw
                .geometry_text_caret(local_id, byte_offset, &mut self.geometry)?;
        Some(CaretGeometry { top, bottom })
    }

    /// Return the nearest valid UTF-8 byte caret for one source-artboard
    /// world-space point on shaped Text.
    ///
    /// This is the owning mirror of [`ArtboardInstance::text_hit`] and has the
    /// same nonfinite-point, target-kind, shaping, finite-geometry, and
    /// unsupported-or-unknown-overflow failure behavior.
    pub fn text_hit(&mut self, local_id: usize, point: Vec2D) -> Option<usize> {
        self.raw
            .geometry_text_hit(local_id, point, &mut self.geometry)
    }

    /// Return one source-artboard world-space selection rectangle per shaped
    /// line segment covered by an exact UTF-8 byte range.
    ///
    /// This is the owning mirror of [`ArtboardInstance::text_selection_rects`]
    /// and has the same invalid-range, target-kind, shaping, finite-geometry,
    /// overflow, soft-wrap, and trailing-static-separator behavior.
    pub fn text_selection_rects(
        &mut self,
        local_id: usize,
        range: std::ops::Range<usize>,
    ) -> Vec<Aabb> {
        self.raw
            .geometry_text_selection_rects(local_id, range, &mut self.geometry)
    }

    pub fn state_machine_instance(&self, index: usize) -> Option<StateMachineInstance> {
        self.raw.state_machine_instance(index)
    }

    /// See [`ArtboardInstance::default_state_machine_instance`].
    pub fn default_state_machine_instance(&self) -> Option<StateMachineInstance> {
        let index = self.artboard().default_state_machine_index().unwrap_or(0);
        self.state_machine_instance(index)
    }

    /// See [`ArtboardInstance::view_model_index`].
    pub fn view_model_index(&self) -> Option<usize> {
        let artboard = self.file.runtime.artboard(self.artboard_index)?;
        let view_model_id = artboard.uint_property("viewModelId")?;
        if view_model_id == u32::MAX as u64 {
            return None;
        }
        usize::try_from(view_model_id).ok()
    }

    /// See [`ArtboardInstance::instantiate_view_model`].
    pub fn instantiate_view_model(&self) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::new(&self.file.runtime, view_model_index)?;
        Some(ViewModelInstance { raw })
    }

    /// See [`ArtboardInstance::instantiate_view_model_instance`].
    pub fn instantiate_view_model_instance(
        &self,
        instance_index: usize,
    ) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::from_instance(
            &self.file.runtime,
            view_model_index,
            instance_index,
        )?;
        Some(ViewModelInstance { raw })
    }

    /// See [`ArtboardInstance::bind_view_model`].
    pub fn bind_view_model(&mut self, view_model: &ViewModelInstance) -> bool {
        let mut changed = self
            .raw
            .bind_default_view_model_artboard_list_context(&self.file.runtime);
        changed |= self
            .raw
            .bind_owned_view_model_artboard_context(&self.file.runtime, view_model.raw());
        changed
    }

    /// See [`ArtboardInstance::advance_with_state_machine`].
    pub fn advance_with_state_machine(
        &mut self,
        state_machine: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_with_state_machines(std::slice::from_mut(state_machine), elapsed_seconds)
    }

    /// Owning mirror of [`ArtboardInstance::advance_with_state_machines`].
    pub fn advance_with_state_machines(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
    ) -> bool {
        if state_machines.is_empty() {
            return self.advance(elapsed_seconds);
        }
        let mut changed = false;
        #[cfg(feature = "scripting")]
        if let Some(artboard) = self.file.graph.artboards.get(self.artboard_index) {
            changed |=
                queue_root_script_advance(&self.file, artboard, &mut self.raw, elapsed_seconds);
        }
        changed |= self
            .raw
            .advance_state_machine_instances_with_nested(state_machines, elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        changed |= self.raw.update_pass();
        changed
    }

    /// Owning mirror of
    /// [`ArtboardInstance::try_advance_with_state_machine_and_factory`].
    pub fn try_advance_with_state_machine_and_factory(
        &mut self,
        state_machine: &mut StateMachineInstance,
        elapsed_seconds: f32,
        factory: &mut dyn Factory,
    ) -> Result<bool> {
        self.try_advance_with_state_machines_and_factory(
            std::slice::from_mut(state_machine),
            elapsed_seconds,
            factory,
        )
    }

    /// Owning mirror of
    /// [`ArtboardInstance::try_advance_with_state_machines_and_factory`].
    pub fn try_advance_with_state_machines_and_factory(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        factory: &mut dyn Factory,
    ) -> Result<bool> {
        if state_machines.is_empty() {
            return self.try_advance_with_factory(factory, elapsed_seconds);
        }
        let mut changed = false;
        #[cfg(feature = "scripting")]
        let artboard = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .context("owned artboard instance graph is unavailable")?;
        #[cfg(feature = "scripting")]
        {
            changed |=
                queue_root_script_advance(&self.file, artboard, &mut self.raw, elapsed_seconds);
        }
        changed |= self
            .raw
            .advance_state_machine_instances_with_nested(state_machines, elapsed_seconds);
        changed |= self
            .raw
            .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        #[cfg(feature = "scripting")]
        {
            changed |= prepare_scripted_artboard_tree(&self.file, artboard, &mut self.raw, factory)
                .context("failed to advance scripted drawables")?;
        }
        #[cfg(not(feature = "scripting"))]
        let _ = factory;
        changed |= self.raw.update_pass();
        Ok(changed)
    }

    pub fn draw(&mut self, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<()> {
        let mut cache = self.new_render_cache();
        self.draw_with_render_cache(factory, renderer, &mut cache)
    }

    /// See [`ArtboardInstance::new_render_cache`].
    pub fn new_render_cache(&self) -> ArtboardRenderCache {
        ArtboardRenderCache {
            paint: None,
            path: RuntimeRenderPathCache::default(),
        }
    }

    /// See [`ArtboardInstance::draw_with_render_cache`].
    pub fn draw_with_render_cache(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
        cache: &mut ArtboardRenderCache,
    ) -> Result<()> {
        let artboard = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .context("owned artboard instance graph is unavailable")?;
        #[cfg(feature = "scripting")]
        let had_retained_paint = cache.paint.is_some();
        ensure_render_paint_cache_for_draw(
            &mut cache.paint,
            &self.file.runtime,
            artboard,
            &self.file.graph.artboards,
            &self.file.external_image_assets,
            factory,
        )?;
        #[cfg(feature = "scripting")]
        if let Err(error) =
            prepare_scripted_artboard_tree(&self.file, artboard, &mut self.raw, factory)
        {
            if !had_retained_paint {
                cache.paint = None;
            }
            return Err(error).context("failed to prepare scripted drawables");
        }
        let paint = cache
            .paint
            .as_mut()
            .context("render paint cache disappeared after successful allocation")?;
        self.raw.update_pass();
        self.raw
            .prepare_static_artboard_tree_paints(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                paint,
                &mut cache.path,
            )
            .context("failed to prepare Rive paints")?;
        self.raw
            .draw_prepared_static_artboard_with_render_cache(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                renderer,
                paint,
                &mut cache.path,
            )
            .context("failed to draw Rive artboard")
    }
}

/// Owned view-model context for driving an artboard's data binds.
///
/// Instantiate one from an [`ArtboardInstance`], set properties by name path,
/// bind it with [`ArtboardInstance::bind_view_model`], then advance and draw.
/// The context owns its own copy of the view model's values, so it does not
/// borrow the originating [`File`] and can outlive nothing in particular; it is
/// only meaningful when bound back to the artboard it came from.
///
/// Property paths address nested view models with `/` separators (for example
/// `"child/width"`); a single segment addresses a property on the root view
/// model. Every setter returns whether a matching, settable property existed
/// and its value changed.
#[derive(Debug, Clone)]
pub struct ViewModelInstance {
    raw: RuntimeOwnedViewModelInstance,
}

impl ViewModelInstance {
    /// Low-level owned context for advanced integrations (for example binding
    /// through [`StateMachineInstance::bind_owned_view_model_context`]).
    pub fn raw(&self) -> &RuntimeOwnedViewModelInstance {
        &self.raw
    }

    /// Low-level mutable owned context for advanced integrations.
    pub fn raw_mut(&mut self) -> &mut RuntimeOwnedViewModelInstance {
        &mut self.raw
    }

    /// Set a number property by name path. Returns whether the property existed
    /// and changed.
    pub fn set_number(&mut self, name_path: &str, value: f32) -> bool {
        self.raw.set_number_by_property_name_path(name_path, value)
    }

    /// Set a boolean property by name path. Returns whether the property existed
    /// and changed.
    pub fn set_bool(&mut self, name_path: &str, value: bool) -> bool {
        self.raw.set_boolean_by_property_name_path(name_path, value)
    }

    /// Set a string property by name path. The value is stored as its UTF-8
    /// bytes. Returns whether the property existed and changed.
    pub fn set_string(&mut self, name_path: &str, value: &str) -> bool {
        self.raw
            .set_string_by_property_name_path(name_path, value.as_bytes())
    }

    /// Set an enum property by its numeric value at the given name path. Returns
    /// whether the property existed and changed. (Enum-by-value-name is not
    /// exposed here because the owned context resolves enums by index.)
    pub fn set_enum(&mut self, name_path: &str, value: u64) -> bool {
        self.raw.set_enum_by_property_name_path(name_path, value)
    }
}

#[cfg(all(test, feature = "scripting"))]
mod inert_script_import_tests {
    use super::*;
    use nuxie_schema::definition_by_name;

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn property_key(type_name: &str, property_name: &str) -> u16 {
        let definition = definition_by_name(type_name).expect("fixture type exists");
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|ancestor| {
                definition_by_name(ancestor)
                    .expect("fixture ancestor exists")
                    .properties
                    .iter()
            }))
            .find(|property| property.name == property_name)
            .expect("fixture property exists")
            .key
            .int
    }

    fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
        push_var_uint(
            bytes,
            u64::from(
                definition_by_name(type_name)
                    .expect("fixture type exists")
                    .type_key
                    .int,
            ),
        );
        properties(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value);
    }

    fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    fn imported_script_asset_bytes() -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ScriptAsset", |bytes| {
            push_uint(bytes, "ScriptAsset", "assetId", 0);
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(bytes, "FileAssetContents", "bytes", &[0, 1, 2, 3]);
        });
        bytes
    }

    fn imported_image_asset_bytes(count: usize) -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 992);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        for asset_id in 0..count {
            push_object(&mut bytes, "ImageAsset", |bytes| {
                push_uint(bytes, "ImageAsset", "assetId", asset_id as u64);
            });
        }
        bytes
    }

    #[test]
    fn bounded_import_rejects_file_assets_before_owned_graph_construction() {
        let bytes = imported_image_asset_bytes(2);
        let limits = FileImportLimits::new().with_max_imported_file_assets(1);

        let error = File::import_with_limits(&bytes, limits)
            .expect_err("the parsed file exceeds its pre-graph asset limit");
        assert!(
            error.to_string().contains("imports more than 1 FileAssets"),
            "{error:#}"
        );
        File::import_with_limits(
            &bytes,
            FileImportLimits::new().with_max_imported_file_assets(2),
        )
        .expect("the exact bound admits graph construction");
    }

    #[test]
    fn ordinary_import_does_not_materialize_the_script_asset_catalog() {
        let bytes = imported_script_asset_bytes();

        let inert = File::import(&bytes).expect("ordinary import remains available");
        assert!(!inert.scripts.borrow().allow_unsigned_execution);
        assert!(
            inert.scripts.borrow().assets.is_none(),
            "untrusted bytes must not allocate or clone their script asset payload catalog"
        );
        assert!(
            inert.clone().scripts.borrow().assets.is_none(),
            "cloning an inert File must not materialize the catalog either"
        );

        let trusted =
            File::import_with_unsigned_scripts(&bytes).expect("explicitly trusted import succeeds");
        let scripts = trusted.scripts.borrow();
        assert!(scripts.allow_unsigned_execution);
        let assets = scripts
            .assets
            .as_ref()
            .expect("trusted imports retain the script catalog");
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].payload.as_deref(), Some([0, 1, 2, 3].as_slice()));
    }
}

#[cfg(test)]
mod owned_instance_tests {
    use super::*;
    use nuxie_render_api::RecordingFactory;

    const FIXTURE: &[u8] = include_bytes!("../../../fixtures/graph/dependency_test.riv");

    fn stream_of(draw: impl FnOnce(&mut RecordingFactory) -> Result<()>) -> String {
        let mut factory = RecordingFactory::new();
        draw(&mut factory).expect("draw succeeds");
        factory.stream()
    }

    #[test]
    fn owned_instance_draws_identically_to_borrowed() {
        let borrowed_stream = stream_of(|factory| {
            let file = File::import(FIXTURE)?;
            let artboard = file.default_artboard().context("default artboard")?;
            let mut instance = artboard.instantiate()?;
            instance.advance(0.0);
            let mut renderer = factory.make_renderer();
            instance.draw(factory, &mut renderer)
        });

        let owned_stream = stream_of(|factory| {
            let file = Arc::new(File::import(FIXTURE)?);
            let mut instance = OwnedArtboardInstance::instantiate_default(file)?;
            instance.advance(0.0);
            let mut renderer = factory.make_renderer();
            instance.draw(factory, &mut renderer)
        });

        assert_eq!(
            owned_stream, borrowed_stream,
            "owned and borrowed instances must draw the identical stream"
        );
    }

    #[test]
    fn owned_instance_outlives_the_importing_scope() {
        let mut instance = {
            let file = Arc::new(File::import(FIXTURE).expect("import"));
            OwnedArtboardInstance::instantiate_default(file).expect("instantiate")
        };
        assert!(!instance.advance(0.016) || instance.raw().components().len() > 0);
    }

    #[test]
    fn borrowed_and_owned_instances_expose_the_same_geometry_queries() {
        let borrowed_file = File::import(FIXTURE).expect("import borrowed fixture");
        let mut borrowed = borrowed_file
            .default_artboard()
            .expect("default artboard")
            .instantiate()
            .expect("instantiate borrowed artboard");
        let mut owned = OwnedArtboardInstance::instantiate_default(Arc::new(
            File::import(FIXTURE).expect("import owned fixture"),
        ))
        .expect("instantiate owned artboard");

        assert_eq!(
            borrowed.hit_test(Vec2D::new(0.0, 0.0)),
            owned.hit_test(Vec2D::new(0.0, 0.0))
        );
        assert_eq!(borrowed.world_bounds(0), owned.world_bounds(0));
        assert_eq!(borrowed.world_transform(0), owned.world_transform(0));
        assert_eq!(borrowed.text_caret(0, 0), owned.text_caret(0, 0));
        assert_eq!(
            borrowed.text_hit(0, Vec2D::new(0.0, 0.0)),
            owned.text_hit(0, Vec2D::new(0.0, 0.0))
        );
        assert_eq!(
            borrowed.text_selection_rects(0, 0..1),
            owned.text_selection_rects(0, 0..1)
        );
    }

    #[test]
    fn promoted_property_writes_report_missing_targets() {
        let file = Arc::new(File::import(FIXTURE).expect("import"));
        let mut instance = OwnedArtboardInstance::instantiate_default(file).expect("instantiate");
        // Nonexistent property key: the typed write path must report false
        // (no match), never panic.
        assert!(!instance.raw_mut().set_double_property(0, u16::MAX, 1.0));
    }
}

#[cfg(test)]
mod external_image_asset_tests {
    use super::*;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};

    fn file_with_image_and_font_assets() -> File {
        let image_asset_type = nuxie_schema::definition_by_name("ImageAsset")
            .expect("ImageAsset schema definition")
            .type_key
            .int;
        let font_asset_type = nuxie_schema::definition_by_name("FontAsset")
            .expect("FontAsset schema definition")
            .type_key
            .int;
        let runtime = RuntimeFile::from_authoring_records(vec![
            AuthoringRecord {
                type_key: crate::scene::TYPE_BACKBOARD,
                properties: Vec::new(),
            },
            AuthoringRecord {
                type_key: image_asset_type,
                properties: vec![AuthoringProperty {
                    key: crate::scene::PROPERTY_FILE_ASSET_ID,
                    value: AuthoringValue::Uint(7),
                }],
            },
            AuthoringRecord {
                type_key: font_asset_type,
                properties: vec![AuthoringProperty {
                    key: crate::scene::PROPERTY_FILE_ASSET_ID,
                    value: AuthoringValue::Uint(8),
                }],
            },
        ])
        .expect("asset-only runtime file");
        File::from_runtime(runtime).expect("asset-only file graph")
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn fixture_font_bytes() -> Vec<u8> {
        let mut accumulator = 0u32;
        let mut bit_count = 0u8;
        let mut decoded = Vec::new();
        for byte in include_bytes!("../tests/fixtures/roboto-a.ttf.base64")
            .iter()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
        {
            if byte == b'=' {
                break;
            }
            let value = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => panic!("invalid base64 font fixture"),
            };
            accumulator = (accumulator << 6) | u32::from(value);
            bit_count += 6;
            if bit_count >= 8 {
                bit_count -= 8;
                decoded.push((accumulator >> bit_count) as u8);
                accumulator &= (1u32 << bit_count) - 1;
            }
        }
        decoded
    }

    #[test]
    fn image_attachment_validates_semantic_identity_and_asset_kind() {
        let mut file = file_with_image_and_font_assets();

        assert_eq!(
            file.attach_image_asset_bytes(99, vec![1, 2, 3]),
            Err(ExternalImageAssetError::UnknownAsset { asset_id: 99 })
        );
        assert_eq!(
            file.attach_image_asset_bytes(8, vec![1, 2, 3]),
            Err(ExternalImageAssetError::WrongAssetKind {
                asset_id: 8,
                actual: "FontAsset",
            })
        );

        file.attach_image_asset_bytes(7, vec![4, 5, 6])
            .expect("ImageAsset accepts host bytes by FileAsset.assetId");
        let image_global_id = file
            .runtime
            .file_assets()
            .into_iter()
            .find(|asset| asset.type_name == "ImageAsset")
            .expect("image asset")
            .id;
        assert_eq!(
            file.external_image_assets
                .get(&image_global_id)
                .map(AsRef::as_ref),
            Some([4, 5, 6].as_slice())
        );
        assert_eq!(
            file.clone()
                .external_image_assets
                .get(&image_global_id)
                .map(AsRef::as_ref),
            Some([4, 5, 6].as_slice()),
            "cloned files retain the exact external asset envelope"
        );
    }

    #[test]
    fn file_font_attachment_rejects_atomically_and_clones_exact_bytes() {
        let mut file = file_with_image_and_font_assets();

        assert_eq!(
            file.attach_font_asset_bytes(99, fixture_font_bytes()),
            Err(ExternalFontAssetError::UnknownAsset { asset_id: 99 })
        );
        assert_eq!(
            file.attach_font_asset_bytes(7, fixture_font_bytes()),
            Err(ExternalFontAssetError::WrongAssetKind {
                asset_id: 7,
                actual: "ImageAsset",
            })
        );
        assert_eq!(
            file.attach_font_asset_bytes(8, b"not a font".to_vec()),
            Err(ExternalFontAssetError::InvalidFont { asset_id: 8 })
        );
        assert!(
            file.external_font_assets.is_empty(),
            "all validation failures happen before the file changes"
        );

        file.attach_font_asset_bytes(8, fixture_font_bytes())
            .expect("valid FontAsset bytes attach by FileAsset.assetId");
        let attached = Arc::clone(
            file.external_font_assets
                .get(&8)
                .expect("valid attachment is retained by semantic id"),
        );
        assert_eq!(
            file.attach_font_asset_bytes(8, b"invalid replacement".to_vec()),
            Err(ExternalFontAssetError::InvalidFont { asset_id: 8 })
        );
        assert!(Arc::ptr_eq(
            &attached,
            file.external_font_assets
                .get(&8)
                .expect("rejected replacement preserves the prior bytes")
        ));
        assert!(Arc::ptr_eq(
            &attached,
            file.clone()
                .external_font_assets
                .get(&8)
                .expect("File::clone retains the exact attachment")
        ));
    }
}
