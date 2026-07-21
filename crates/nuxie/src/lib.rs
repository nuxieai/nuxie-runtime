//! Public Rust API facade for loading and drawing Rive files.
//!
//! This crate keeps the user-facing surface narrow while the lower-level
//! crates continue to carry the import, graph, runtime, and renderer details.

use std::cell::{Ref, RefMut};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "scripting")]
use std::{cell::RefCell, collections::VecDeque};

use anyhow::{Context, Result, bail};
// The facade always retains ScriptAsset contents because pure-Rust ProjectDO
// converter envelopes use that standard Rive asset carrier. Retention does
// not grant script execution: arbitrary bytecode remains gated by the
// `scripting` feature and an explicitly bounded trusted-script import.
use nuxie_binary::{
    RuntimeFile, RuntimeImportStatus,
    read_runtime_file_with_scripting as read_runtime_file_for_facade,
    read_runtime_file_with_scripting_with_limits as read_runtime_file_for_facade_with_parser_limits,
};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_runtime::{
    ArtboardInstance as RuntimeArtboardInstance, RuntimeGeometryCache, RuntimeGeometryHit,
    RuntimeImageDimensionConflict, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    RuntimeRenderPaintCache, RuntimeRenderPathCache, RuntimeSemanticTextHit,
    StateMachineEventContext, embedded_fonts_are_parseable,
};

pub mod flow_session;
mod scene;
#[cfg(feature = "scripting")]
mod script_import;

#[cfg(all(test, feature = "scripting"))]
mod scripted_listener_action_lifecycle_tests;

pub use scene::*;
#[cfg(feature = "scripting")]
pub use script_import::{ScriptAuthenticationError, ScriptImportCapability};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptExecutionAuthorization {
    VisualOnly,
    Authenticated,
}

pub use nuxie_render_api::{
    Aabb, BlendMode, ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasPlan, GpuCanvasShader,
    GpuCanvasShaderStage, ImageDecodeError, ImageFilter, ImageSampler, ImageWrap, Mat2D, PathVerb,
    RawPath, RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage,
    RenderPaint, RenderPaintStyle, RenderPath, RenderShader, Renderer, StrokeCap, StrokeJoin,
    Vec2D,
};
#[cfg(all(feature = "renderer", any(target_os = "ios", target_os = "macos")))]
pub use nuxie_renderer::{
    ApplePresentationCompletion, AppleSurface, SurfaceDisposition, SurfaceError,
};
#[cfg(all(feature = "renderer", target_arch = "wasm32"))]
pub use nuxie_renderer::{
    BrowserBackend, BrowserBackendPreference, BrowserFactory,
    BrowserFactory as DefaultRendererFactory, BrowserFrame, BrowserFrame as DefaultRendererFrame,
    BrowserResizeError, WebGl2Factory, WebGl2Frame, WebGl2GpuCanvasRenderer,
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
    ExternalFontAssetError, LinearAnimationInstance, NoopScriptHost, ProjectDataConverterCatalog,
    ProjectDataConverterCompileError, ProjectDataConverterContext, ProjectDataConverterDefinition,
    ProjectDataConverterEasing, ProjectDataConverterFormat, ProjectDataConverterKind,
    ProjectDataConverterMathOperation, ProjectDataConverterOutputType, ProjectDataConverterProgram,
    ProjectDataConverterProgramError, ProjectDataConverterRangeClamp, ProjectDataConverterResolver,
    ProjectDataConverterReverseResult, ProjectDataConverterRuntimeError, ProjectDataConverterSpec,
    ProjectDataConverterState, ProjectDataConverterStringPadSide,
    ProjectDataConverterStringTrimMode, ProjectDataConverterValidationRule, ProjectDataValue,
    ProjectDataValuePath, RuntimeLayerState, RuntimeOwnedViewModelContext,
    RuntimeStateMachineInput, ScriptError, ScriptHost, ScriptInstance, ScriptMethod, ScriptModule,
    ScriptModuleFailure, ScriptValue, ScriptingVm, StateMachineInputInstance,
    StateMachineInputKind, StateMachineInstance, StateMachineReportedEvent,
};

#[cfg(feature = "scripting")]
use nuxie_scripting::vm::{
    HostCommand as LuaHostCommand, HostCycleCheckpoint, HostValue as LuaHostValue, ScriptProgram,
};
#[cfg(feature = "scripting")]
pub use nuxie_scripting::vm::{LuaScriptInstance, ScopeKey, ScriptExecutionLimits, ScriptVm};

#[cfg(feature = "scripting")]
type FileScriptPolicy = Option<ScriptExecutionLimits>;
#[cfg(not(feature = "scripting"))]
type FileScriptPolicy = ();

#[cfg(feature = "scripting")]
fn inert_script_policy() -> FileScriptPolicy {
    None
}
#[cfg(not(feature = "scripting"))]
fn inert_script_policy() -> FileScriptPolicy {}

#[cfg(feature = "scripting")]
fn trusted_script_policy(enabled: bool) -> FileScriptPolicy {
    enabled.then(ScriptExecutionLimits::new)
}
#[cfg(not(feature = "scripting"))]
fn trusted_script_policy(_enabled: bool) -> FileScriptPolicy {}

#[cfg(feature = "scripting")]
#[derive(Debug, Clone)]
struct FileScriptAsset {
    ordinal: usize,
    global_id: u32,
    type_name: &'static str,
    name: String,
    scope: ScopeKey,
    is_module: bool,
    payload: Option<Vec<u8>>,
    is_project_data_converter: bool,
}

#[cfg(feature = "scripting")]
#[derive(Debug, Clone)]
struct FileScriptLibraryImport {
    caller: ScopeKey,
    name: String,
    target: ScopeKey,
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
    assets: Arc<[FileScriptAsset]>,
    imports: Arc<[FileScriptLibraryImport]>,
    authorization: ScriptExecutionAuthorization,
    execution_limits: Option<ScriptExecutionLimits>,
    ready: Option<ReadyFileScripts>,
}

#[cfg(feature = "scripting")]
impl std::fmt::Debug for FileScriptRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileScriptRuntime")
            .field("assets", &self.assets)
            .field("imports", &self.imports)
            .field("authorization", &self.authorization)
            .field("execution_limits", &self.execution_limits)
            .field("ready", &self.ready.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "scripting")]
impl FileScriptRuntime {
    fn import(
        runtime: &RuntimeFile,
        authorization: ScriptExecutionAuthorization,
        execution_limits: Option<ScriptExecutionLimits>,
    ) -> Self {
        let entries = runtime.scripting_file_assets_with_contents();
        let assets = entries
            .iter()
            .copied()
            .map(|entry| {
                let name = entry.asset.string_property("name").unwrap_or_default();
                let folder = entry
                    .asset
                    .string_property("folderPath")
                    .unwrap_or_default();
                let payload = entry.contents.map(ToOwned::to_owned);
                FileScriptAsset {
                    ordinal: entry.ordinal,
                    global_id: entry.asset.id,
                    type_name: entry.asset.type_name,
                    name: if folder.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{folder}/{name}")
                    },
                    scope: ScopeKey::new(
                        entry.asset.uint_property("scopeLibraryId").unwrap_or(0),
                        entry
                            .asset
                            .uint_property("scopeLibraryVersionId")
                            .unwrap_or(0),
                    ),
                    is_module: entry.asset.bool_property("isModule").unwrap_or(false),
                    is_project_data_converter: entry.asset.type_name == "ScriptAsset"
                        && payload
                            .as_deref()
                            .is_some_and(ProjectDataConverterProgram::is_envelope),
                    payload,
                }
            })
            .collect::<Vec<_>>()
            .into();
        let imports = entries
            .into_iter()
            .filter(|entry| entry.asset.type_name == "LibraryAsset")
            .map(|entry| FileScriptLibraryImport {
                caller: ScopeKey::new(
                    entry.asset.uint_property("scopeLibraryId").unwrap_or(0),
                    entry
                        .asset
                        .uint_property("scopeLibraryVersionId")
                        .unwrap_or(0),
                ),
                name: entry
                    .asset
                    .string_property("name")
                    .unwrap_or_default()
                    .to_owned(),
                target: ScopeKey::new(
                    entry.asset.uint_property("libraryId").unwrap_or(0),
                    entry.asset.uint_property("libraryVersionId").unwrap_or(0),
                ),
            })
            .collect::<Vec<_>>()
            .into();
        Self::new(assets, imports, authorization, execution_limits)
    }

    fn new(
        assets: Arc<[FileScriptAsset]>,
        imports: Arc<[FileScriptLibraryImport]>,
        authorization: ScriptExecutionAuthorization,
        execution_limits: Option<ScriptExecutionLimits>,
    ) -> Self {
        Self {
            assets,
            imports,
            authorization,
            execution_limits,
            ready: None,
        }
    }

    fn scripts_are_authenticated(&self) -> bool {
        self.authorization == ScriptExecutionAuthorization::Authenticated
            && self.execution_limits.is_some()
    }

    fn is_project_data_converter_asset(&self, ordinal: usize) -> bool {
        self.assets
            .get(ordinal)
            .is_some_and(|asset| asset.is_project_data_converter)
    }

    fn build_candidate(
        &self,
        runtime: &RuntimeFile,
        factory: &mut dyn Factory,
    ) -> std::result::Result<ReadyFileScripts, nuxie_runtime::ScriptError> {
        let execution_limits = self.execution_limits.ok_or_else(|| {
            nuxie_runtime::ScriptError::new(
                "trusted script execution limits are unavailable for a visual-only File",
            )
        })?;
        let mut vm = ScriptVm::new_with_execution_limits(execution_limits)
            .map_err(|error| nuxie_runtime::ScriptError::new(error.to_string()))?;
        vm.set_view_models(nuxie_runtime::script_view_models(runtime));
        // LibraryAsset records are serialized import edges. Seed every pin
        // before executing any module so both eager and lazy requires observe
        // the exact per-caller dependency graph from the file.
        for import in self.imports.iter() {
            vm.add_import(import.caller, &import.name, import.target);
        }
        let assets = self.assets.as_ref();
        for asset in assets
            .iter()
            .filter(|asset| asset.type_name == "ShaderAsset")
        {
            let payload = required_script_payload(asset, "shader registration")?;
            vm.register_gpu_canvas_shader_asset(&asset.name, payload)
                .map_err(|error| asset_phase_error(asset, "shader registration", error))?;
        }
        let mut pending = assets
            .iter()
            .filter(|asset| {
                asset.type_name == "ScriptAsset"
                    && asset.is_module
                    && !asset.is_project_data_converter
            })
            .collect::<Vec<_>>();

        loop {
            let before = pending.len();
            let mut failures = Vec::new();
            for asset in pending {
                let payload = required_script_payload(asset, "module registration")?;
                let effect_checkpoint = vm.checkpoint_host_effects();
                if let Err(error) = vm.register_module_with_factory_scoped(
                    &asset.name,
                    asset.scope,
                    payload,
                    factory,
                ) {
                    vm.rollback_host_effects(effect_checkpoint);
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
        for asset in assets.iter().filter(|asset| {
            asset.type_name == "ScriptAsset" && !asset.is_module && !asset.is_project_data_converter
        }) {
            let payload = required_script_payload(asset, "protocol registration")?;
            let program = vm
                .register_protocol_script_with_factory_scoped(
                    &asset.name,
                    asset.scope,
                    payload,
                    factory,
                )
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
        runtime: &RuntimeFile,
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
        let candidate = self.build_candidate(runtime, factory)?;
        let groups = instantiate_script_mounts(&candidate, groups, factory)?;
        Ok(PreparedFileScriptMounts {
            // Drop table handles before their candidate VM on a failed
            // topology validation.
            groups,
            candidate: Some(candidate),
        })
    }

    fn begin_host_cycle(&self) -> Option<HostCycleCheckpoint> {
        self.ready.as_ref().map(|ready| ready.vm.begin_host_cycle())
    }

    fn rollback_host_cycle(&self, checkpoint: HostCycleCheckpoint) {
        if let Some(ready) = self.ready.as_ref() {
            ready.vm.rollback_host_cycle(checkpoint);
        }
    }

    fn drain_host_commands(&self) -> Vec<LuaHostCommand> {
        self.ready
            .as_ref()
            .map(|ready| ready.vm.drain_host_commands())
            .unwrap_or_default()
    }
}

#[cfg(feature = "scripting")]
#[derive(Debug, Clone, Copy)]
enum ScriptMountTargetKind {
    Drawable,
    DataConverter,
}

#[cfg(feature = "scripting")]
impl ScriptMountTargetKind {
    fn label(self) -> &'static str {
        match self {
            Self::Drawable => "ScriptedDrawable",
            Self::DataConverter => "ScriptedDataConverter",
        }
    }
}

#[cfg(feature = "scripting")]
#[derive(Debug)]
struct ScriptMountTarget {
    kind: ScriptMountTargetKind,
    global_id: u32,
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
    scripts: Vec<(ScriptMountTargetKind, u32, Box<dyn ScriptInstance>)>,
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
            "{} ordinal {} global {} name '{}' phase {} has no imported FileAssetContents payload",
            asset.type_name, asset.ordinal, asset.global_id, asset.name, phase
        ))
    })
}

#[cfg(feature = "scripting")]
fn asset_phase_error(
    asset: &FileScriptAsset,
    phase: &str,
    error: nuxie_runtime::ScriptError,
) -> nuxie_runtime::ScriptError {
    error.with_context(format!(
        "{} ordinal {} global {} name '{}' phase {} failed",
        asset.type_name, asset.ordinal, asset.global_id, asset.name, phase
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
            let target_label = target.kind.label();
            let program = ready.programs.get(&target.asset_ordinal).ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "{} {target_label} global {} references unregistered protocol ordinal {} name '{}'",
                    group.path,
                    target.global_id,
                    target.asset_ordinal,
                    target.asset_name
                ))
            })?;
            let mut host = NoopScriptHost;
            let mut script = ready
                .vm
                .instantiate_registered_script_with_factory(program, &mut host, factory)
                .map_err(|error| {
                    error.with_context(format!(
                        "{} {target_label} global {} asset ordinal {} name '{}' phase generator failed",
                        group.path,
                        target.global_id,
                        target.asset_ordinal,
                        target.asset_name
                    ))
                })?;
            if script.has_method(ScriptMethod::Init).map_err(|error| {
                error.with_context(format!(
                    "{} {target_label} global {} asset ordinal {} name '{}' phase init lookup failed",
                    group.path,
                    target.global_id,
                    target.asset_ordinal,
                    target.asset_name
                ))
            })? {
                let initialized = script
                    .call_init_with_factory(&mut host, factory)
                    .map_err(|error| {
                        error.with_context(format!(
                            "{} {target_label} global {} asset ordinal {} name '{}' phase init failed",
                            group.path,
                            target.global_id,
                            target.asset_ordinal,
                            target.asset_name
                        ))
                    })?;
                if !initialized {
                    return Err(nuxie_runtime::ScriptError::new(format!(
                        "{} {target_label} global {} asset ordinal {} name '{}' phase init returned false or nil",
                        group.path,
                        target.global_id,
                        target.asset_ordinal,
                        target.asset_name
                    )));
                }
            }
            scripts.push((target.kind, target.global_id, script));
        }
        prepared.push(PreparedScriptMountGroup {
            graph_global_id: group.graph_global_id,
            scripts,
        });
    }
    Ok(prepared)
}

#[cfg(feature = "scripting")]
fn instantiate_script_listener_actions(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    factory: &mut dyn Factory,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let definitions = machine.scripted_listener_actions().to_vec();
    if definitions.is_empty() || !file.scripts.borrow().scripts_are_authenticated() {
        return Ok(());
    }
    let hydrations =
        prepare_script_listener_hydrations(file, &definitions, root_view_model, None, false)?;
    let scripts = file.scripts.borrow();
    let ready = scripts.ready.as_ref().ok_or_else(|| {
        nuxie_runtime::ScriptError::new(
            "scripted listener actions require a bootstrapped script VM",
        )
    })?;
    if ready.factory_domain != render_factory_domain(factory) {
        return Err(nuxie_runtime::ScriptError::new(
            "scripted listener actions used a different renderer Factory domain",
        ));
    }
    let mut prepared = Vec::with_capacity(definitions.len());
    for (definition, hydration) in definitions.into_iter().zip(hydrations) {
        let ordinal = definition.asset_ordinal();
        let asset = scripts.assets.get(ordinal).ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "ScriptedListenerAction global {} references absent FileAsset ordinal {ordinal}",
                definition.action_global_id()
            ))
        })?;
        if asset.type_name != "ScriptAsset"
            || asset.is_module
            || asset.name != definition.asset_name()
        {
            return Err(nuxie_runtime::ScriptError::new(format!(
                "ScriptedListenerAction global {} does not resolve its non-module ScriptAsset ordinal {ordinal} name '{}'",
                definition.action_global_id(),
                definition.asset_name()
            )));
        }
        let program = ready.programs.get(&ordinal).ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "ScriptedListenerAction global {} references unregistered protocol ordinal {ordinal} name '{}'",
                definition.action_global_id(),
                definition.asset_name()
            ))
        })?;
        let mut host = NoopScriptHost;
        let mut instance = ready
            .vm
            .instantiate_registered_script_with_factory(program, &mut host, factory)
            .map_err(|error| {
                error.with_context(format!(
                    "ScriptedListenerAction global {} asset ordinal {ordinal} name '{}' generator failed",
                    definition.action_global_id(),
                    definition.asset_name()
                ))
            })?;
        hydration.apply(&mut *instance, &mut host).map_err(|error| {
            error.with_context(format!(
                "ScriptedListenerAction global {} asset ordinal {ordinal} name '{}' input hydration failed",
                definition.action_global_id(),
                definition.asset_name()
            ))
        })?;
        if instance.has_method(ScriptMethod::Init).map_err(|error| {
            error.with_context(format!(
                "ScriptedListenerAction global {} asset ordinal {ordinal} name '{}' init lookup failed",
                definition.action_global_id(),
                definition.asset_name()
            ))
        })? {
            let initialized = instance
                .call_init_with_factory(&mut host, factory)
                .map_err(|error| {
                    error.with_context(format!(
                        "ScriptedListenerAction global {} asset ordinal {ordinal} name '{}' init failed",
                        definition.action_global_id(),
                        definition.asset_name()
                    ))
                })?;
            if !initialized {
                return Err(nuxie_runtime::ScriptError::new(format!(
                    "ScriptedListenerAction global {} asset ordinal {ordinal} name '{}' init returned false or nil",
                    definition.action_global_id(),
                    definition.asset_name()
                )));
            }
        }
        prepared.push((definition.action_global_id(), instance));
    }
    drop(scripts);
    for (action_global_id, instance) in prepared {
        machine.set_scripted_listener_action_instance(action_global_id, instance)?;
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn rehydrate_script_listener_actions(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    root_view_model: Option<&ViewModelInstance>,
    previous_root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let definitions = machine.scripted_listener_actions().to_vec();
    if definitions.is_empty() || !file.scripts.borrow().scripts_are_authenticated() {
        return Ok(());
    }
    let hydrations = prepare_script_listener_hydrations(
        file,
        &definitions,
        root_view_model,
        previous_root_view_model,
        true,
    )?;
    for (definition, hydration) in definitions.into_iter().zip(hydrations) {
        machine
            .hydrate_scripted_listener_action_instance(definition.action_global_id(), hydration)
            .map_err(|error| {
                error.with_context(format!(
                    "ScriptedListenerAction global {} asset ordinal {} name '{}' data-context rehydration failed",
                    definition.action_global_id(),
                    definition.asset_ordinal(),
                    definition.asset_name()
                ))
            })?;
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn prepare_script_listener_hydrations(
    file: &Arc<File>,
    definitions: &[nuxie_runtime::ScriptListenerActionDefinition],
    root_view_model: Option<&ViewModelInstance>,
    previous_root_view_model: Option<&ViewModelInstance>,
    rebind: bool,
) -> std::result::Result<
    Vec<nuxie_runtime::ScriptListenerActionHydration>,
    nuxie_runtime::ScriptError,
> {
    let runtime = file.runtime();
    let root_context = root_view_model.map(|view_model| {
        nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
            runtime,
            view_model.handle().clone(),
        )
    });
    let context_view_model = root_view_model.and_then(|view_model| {
        nuxie_runtime::script_view_model_from_owned(runtime, view_model.handle())
    });
    let mut prepared = Vec::with_capacity(definitions.len());

    for definition in definitions {
        let mut inputs = Vec::with_capacity(definition.inputs().len());
        for input_definition in definition.inputs() {
            let input = runtime
                .object(input_definition.input_global_id() as usize)
                .ok_or_else(|| {
                    listener_input_hydration_error(
                        definition,
                        input_definition.input_global_id(),
                        "object is absent",
                    )
                })?;
            let expected_type = match input_definition.kind() {
                nuxie_runtime::ScriptListenerInputKind::Boolean => "ScriptInputBoolean",
                nuxie_runtime::ScriptListenerInputKind::Number => "ScriptInputNumber",
                nuxie_runtime::ScriptListenerInputKind::Color => "ScriptInputColor",
                nuxie_runtime::ScriptListenerInputKind::String => "ScriptInputString",
                nuxie_runtime::ScriptListenerInputKind::Trigger => "ScriptInputTrigger",
                nuxie_runtime::ScriptListenerInputKind::Artboard => "ScriptInputArtboard",
                nuxie_runtime::ScriptListenerInputKind::ViewModelProperty => {
                    "ScriptInputViewModelProperty"
                }
            };
            if input.type_name != expected_type {
                return Err(listener_input_hydration_error(
                    definition,
                    input.id,
                    &format!("expected {expected_type}, found {}", input.type_name),
                ));
            }
            let name = input.string_property("name").unwrap_or_default().to_owned();
            match input_definition.kind() {
                nuxie_runtime::ScriptListenerInputKind::Boolean
                | nuxie_runtime::ScriptListenerInputKind::Number
                | nuxie_runtime::ScriptListenerInputKind::Color
                | nuxie_runtime::ScriptListenerInputKind::String => {
                    let bound = match root_view_model {
                        Some(view_model) => nuxie_runtime::bound_script_input_value(
                            runtime,
                            &view_model.raw(),
                            input,
                        )?,
                        None => None,
                    };
                    if rebind && bound.is_none() {
                        // Unbound authored inputs are installed once before
                        // init. Rive only refreshes retained DataBind inputs at
                        // the advance-time bind flush; resetting this field on
                        // every cycle would erase script-owned table state.
                        continue;
                    }
                    let value = bound.unwrap_or_else(|| match input_definition.kind() {
                        nuxie_runtime::ScriptListenerInputKind::Boolean => {
                            nuxie_runtime::ScriptValue::Bool(
                                input.bool_property("propertyValue").unwrap_or(false),
                            )
                        }
                        nuxie_runtime::ScriptListenerInputKind::Number => {
                            nuxie_runtime::ScriptValue::Number(f64::from(
                                input.double_property("propertyValue").unwrap_or(0.0),
                            ))
                        }
                        nuxie_runtime::ScriptListenerInputKind::Color => {
                            nuxie_runtime::ScriptValue::Color(
                                input.color_property("propertyValue").unwrap_or(0),
                            )
                        }
                        nuxie_runtime::ScriptListenerInputKind::String => {
                            nuxie_runtime::ScriptValue::String(
                                input
                                    .string_property("propertyValue")
                                    .unwrap_or_default()
                                    .to_owned(),
                            )
                        }
                        _ => unreachable!("scalar listener input kind"),
                    });
                    inputs.push(nuxie_runtime::ScriptListenerInputHydration::Value { name, value });
                }
                nuxie_runtime::ScriptListenerInputKind::Trigger => {
                    let current = match root_view_model {
                        Some(view_model) => nuxie_runtime::bound_script_trigger_input(
                            runtime,
                            &view_model.raw(),
                            input,
                        )?,
                        None => None,
                    };
                    if rebind {
                        let previous = match previous_root_view_model {
                            Some(view_model) => nuxie_runtime::bound_script_trigger_input(
                                runtime,
                                &view_model.raw(),
                                input,
                            )?,
                            None => None,
                        };
                        if current.is_some_and(|current| current != 0 && Some(current) != previous)
                        {
                            inputs.push(nuxie_runtime::ScriptListenerInputHydration::Trigger {
                                name,
                            });
                        }
                    }
                }
                nuxie_runtime::ScriptListenerInputKind::Artboard => {
                    let bound = match root_view_model {
                        Some(view_model) => nuxie_runtime::bound_script_artboard_input(
                            runtime,
                            &view_model.raw(),
                            input,
                        )?,
                        None => None,
                    };
                    if rebind && bound.is_none() {
                        continue;
                    }
                    let artboard_id = bound
                        .or_else(|| input.uint_property("artboardId"))
                        .filter(|id| *id != u64::from(u32::MAX))
                        .and_then(|id| usize::try_from(id).ok())
                        .ok_or_else(|| {
                            listener_input_hydration_error(
                                definition,
                                input.id,
                                "referenced artboard is unresolved",
                            )
                        })?;
                    let artboard = FileScriptArtboard::new(Arc::clone(file), artboard_id).map_err(
                        |error| {
                            listener_input_hydration_error(
                                definition,
                                input.id,
                                &format!(
                                    "referenced artboard {artboard_id} is unavailable: {error}"
                                ),
                            )
                        },
                    )?;
                    inputs.push(nuxie_runtime::ScriptListenerInputHydration::Artboard {
                        name,
                        artboard: Box::new(artboard),
                    });
                }
                nuxie_runtime::ScriptListenerInputKind::ViewModelProperty => {
                    let context = root_context.as_ref().ok_or_else(|| {
                        listener_input_hydration_error(
                            definition,
                            input.id,
                            "requires an active root view-model context",
                        )
                    })?;
                    let view_model = nuxie_runtime::bound_script_view_model_from_owned_context(
                        runtime, context, input,
                    )
                    .ok_or_else(|| {
                        listener_input_hydration_error(
                            definition,
                            input.id,
                            "view-model property path is unresolved",
                        )
                    })?;
                    inputs.push(nuxie_runtime::ScriptListenerInputHydration::ViewModel {
                        name,
                        view_model,
                    });
                }
            }
        }
        prepared.push(nuxie_runtime::ScriptListenerActionHydration::new(
            context_view_model.clone(),
            inputs,
        ));
    }
    Ok(prepared)
}

#[cfg(feature = "scripting")]
fn listener_input_hydration_error(
    definition: &nuxie_runtime::ScriptListenerActionDefinition,
    input_global_id: u32,
    detail: &str,
) -> nuxie_runtime::ScriptError {
    nuxie_runtime::ScriptError::new(format!(
        "ScriptedListenerAction global {} asset ordinal {} name '{}' input global {input_global_id}: {detail}",
        definition.action_global_id(),
        definition.asset_ordinal(),
        definition.asset_name()
    ))
}

/// File-backed artboard userdata used by `ScriptInputArtboard`.
///
/// This deliberately drives the low-level runtime instance instead of calling
/// the owning facade's script bootstrap recursively: listener hydration and
/// `init` execute while the File VM is already borrowed.
#[cfg(feature = "scripting")]
struct FileScriptArtboard {
    file: Arc<File>,
    artboard_index: usize,
    instance: RuntimeArtboardInstance,
    state_machine: Option<StateMachineInstance>,
    view_model: Option<nuxie_runtime::ScriptViewModel>,
    width: f32,
    height: f32,
    frame_origin: bool,
    paint_cache: Option<RuntimeRenderPaintCache>,
    path_cache: RuntimeRenderPathCache,
}

#[cfg(feature = "scripting")]
impl FileScriptArtboard {
    fn new(
        file: Arc<File>,
        artboard_index: usize,
    ) -> std::result::Result<Self, nuxie_runtime::ScriptError> {
        let graph = file.graph.artboards.get(artboard_index).ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "missing scripted artboard index {artboard_index}",
            ))
        })?;
        let external_font_assets = file.external_font_assets.snapshot();
        let instance = RuntimeArtboardInstance::from_graph_with_artboards_and_external_fonts(
            &file.runtime,
            graph,
            &file.graph.artboards,
            &external_font_assets,
        )
        .map_err(|error| nuxie_runtime::ScriptError::new(error.to_string()))?;
        let state_machine_index = file
            .artboard(artboard_index)
            .and_then(|artboard| artboard.default_state_machine_index())
            .unwrap_or(0);
        let state_machine = instance.state_machine_instance(state_machine_index);
        let owned_view_model = file
            .runtime
            .artboard(artboard_index)
            .and_then(|artboard| artboard.uint_property("viewModelId"))
            .filter(|index| *index != u64::from(u32::MAX))
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|view_model_index| {
                file.runtime
                    .view_model_default_instance(view_model_index)
                    .and_then(|instance| {
                        RuntimeOwnedViewModelInstance::from_instance(
                            &file.runtime,
                            view_model_index,
                            instance.instance_index,
                        )
                    })
                    .or_else(|| RuntimeOwnedViewModelInstance::new(&file.runtime, view_model_index))
                    .map(RuntimeOwnedViewModelHandle::new)
            });
        let view_model = owned_view_model.as_ref().and_then(|instance| {
            nuxie_runtime::script_view_model_from_owned(&file.runtime, instance)
        });
        let (width, height) = instance.artboard_dimensions();
        let mut scripted = Self {
            file,
            artboard_index,
            instance,
            state_machine,
            view_model,
            width,
            height,
            frame_origin: false,
            paint_cache: None,
            path_cache: RuntimeRenderPathCache::default(),
        };
        scripted.bind_view_model();
        Ok(scripted)
    }

    fn bind_view_model(&mut self) {
        let Some(view_model) = self.view_model.as_ref() else {
            return;
        };
        let context = view_model.owned_handle();
        if let Some(state_machine) = self.state_machine.as_mut() {
            state_machine.bind_owned_view_model_context_handle(&context);
            state_machine.advance_data_context();
        }
        self.instance
            .bind_owned_view_model_artboard_context_handle(&self.file.runtime, &context);
    }
}

#[cfg(feature = "scripting")]
impl nuxie_runtime::ScriptArtboard for FileScriptArtboard {
    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn frame_origin(&self) -> bool {
        self.frame_origin
    }

    fn set_width(&mut self, width: f32) {
        self.width = width;
        self.instance.set_artboard_dimensions(width, self.height);
    }

    fn set_height(&mut self, height: f32) {
        self.height = height;
        self.instance.set_artboard_dimensions(self.width, height);
    }

    fn set_frame_origin(&mut self, frame_origin: bool) {
        self.frame_origin = frame_origin;
    }

    fn data(&self) -> Option<nuxie_runtime::ScriptViewModel> {
        self.view_model.clone()
    }

    fn instance(
        &self,
        view_model: Option<nuxie_runtime::ScriptViewModel>,
    ) -> std::result::Result<Box<dyn nuxie_runtime::ScriptArtboard>, nuxie_runtime::ScriptError>
    {
        let mut instance = Self::new(Arc::clone(&self.file), self.artboard_index)?;
        instance.view_model = view_model;
        instance.bind_view_model();
        Ok(Box::new(instance))
    }

    fn advance(&mut self, seconds: f32) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
        self.bind_view_model();
        let mut changed = if let Some(state_machine) = self.state_machine.as_mut() {
            self.instance
                .advance_state_machine_instance(state_machine, seconds)
        } else {
            self.instance.advance_nested_artboards(seconds)
        };
        changed |= self
            .instance
            .advance_artboard_data_binds_with_elapsed(seconds);
        changed |= self.instance.update_pass();
        Ok(changed)
    }

    fn animation(
        &self,
        name: &str,
    ) -> std::result::Result<Option<nuxie_runtime::ScriptAnimation>, nuxie_runtime::ScriptError>
    {
        Ok(nuxie_runtime::ScriptAnimation::named(&self.instance, name))
    }

    fn advance_animation(
        &mut self,
        animation: &mut nuxie_runtime::ScriptAnimation,
        seconds: f32,
    ) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
        Ok(animation.advance(&mut self.instance, seconds))
    }

    fn set_animation_time(
        &mut self,
        animation: &mut nuxie_runtime::ScriptAnimation,
        value: f32,
        mode: nuxie_runtime::ScriptAnimationTime,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        animation.set_time(&mut self.instance, value, mode);
        Ok(())
    }

    fn node(
        &self,
        name: &str,
    ) -> std::result::Result<Option<nuxie_runtime::ScriptNode>, nuxie_runtime::ScriptError> {
        let graph = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "missing scripted artboard index {}",
                    self.artboard_index
                ))
            })?;
        Ok(nuxie_runtime::script_node_for_artboard(
            &self.instance,
            graph,
            name,
        ))
    }

    fn draw(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        let graph = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "missing scripted artboard index {}",
                    self.artboard_index
                ))
            })?;
        if self.paint_cache.is_none() {
            let candidate =
                RuntimeRenderPaintCache::preallocate_for_artboard_tree_with_external_images(
                    &self.file.runtime,
                    graph,
                    &self.file.graph.artboards,
                    &self.file.external_image_assets,
                    factory,
                );
            if let Some(error) = candidate.image_decode_error() {
                return Err(nuxie_runtime::ScriptError::new(error.to_string()));
            }
            self.paint_cache = Some(candidate);
        }
        let Some(paint_cache) = self.paint_cache.as_mut() else {
            return Err(nuxie_runtime::ScriptError::new(
                "scripted artboard paint cache initialization produced no cache",
            ));
        };
        self.instance
            .prepare_static_artboard_tree_paints(
                &self.file.runtime,
                graph,
                &self.file.graph.artboards,
                factory,
                paint_cache,
                &mut self.path_cache,
            )
            .map_err(|error| nuxie_runtime::ScriptError::new(error.to_string()))?;
        self.instance
            .draw_prepared_static_artboard_with_render_cache_and_origin(
                &self.file.runtime,
                graph,
                &self.file.graph.artboards,
                factory,
                renderer,
                paint_cache,
                &mut self.path_cache,
                self.frame_origin,
            )
            .map_err(|error| nuxie_runtime::ScriptError::new(error.to_string()))
    }
}

#[cfg(feature = "scripting")]
fn script_mount_target(
    runtime: &RuntimeFile,
    scripts: &FileScriptRuntime,
    object: &nuxie_binary::RuntimeObject,
    kind: ScriptMountTargetKind,
    path: &str,
) -> std::result::Result<ScriptMountTarget, nuxie_runtime::ScriptError> {
    let label = kind.label();
    if !scripts.scripts_are_authenticated() {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{path} contains {label} global {}, but this File has no authenticated script authority",
            object.id
        )));
    }
    let ordinal = object
        .uint_property("scriptAssetId")
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "{path} {label} global {} has no valid FileAsset ordinal",
                object.id
            ))
        })?;
    let resolved = runtime
        .resolved_file_asset_for_referencer(object)
        .ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "{path} {label} global {} cannot resolve FileAsset ordinal {ordinal}",
                object.id
            ))
        })?;
    let asset = scripts.assets.get(ordinal).ok_or_else(|| {
        nuxie_runtime::ScriptError::new(format!(
            "{path} {label} global {} references absent FileAsset ordinal {ordinal}",
            object.id
        ))
    })?;
    if asset.global_id != resolved.id {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{path} {label} global {} FileAsset ordinal {ordinal} resolved global {}, but catalog contains global {}",
            object.id, resolved.id, asset.global_id
        )));
    }
    if asset.type_name != "ScriptAsset" || resolved.type_name != "ScriptAsset" {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{path} {label} global {} FileAsset ordinal {ordinal} is {}, not ScriptAsset",
            object.id, resolved.type_name
        )));
    }
    if asset.is_module {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{path} {label} global {} references module ScriptAsset ordinal {ordinal} name '{}'",
            object.id, asset.name
        )));
    }
    Ok(ScriptMountTarget {
        kind,
        global_id: object.id,
        asset_ordinal: ordinal,
        asset_name: asset.name.clone(),
    })
}

#[cfg(feature = "scripting")]
fn script_mount_group(
    runtime: &RuntimeFile,
    scripts: &FileScriptRuntime,
    graph: &ArtboardGraph,
    instance: &RuntimeArtboardInstance,
    path: String,
) -> std::result::Result<(bool, ScriptMountGroup), nuxie_runtime::ScriptError> {
    let mut has_script_target = false;
    let mut targets = Vec::new();
    for component in &graph.components {
        if component.type_name != "ScriptedDrawable" {
            continue;
        }
        has_script_target = true;
        let object = runtime
            .object(component.global_id as usize)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "{path} ScriptedDrawable global {} is absent from the runtime file",
                    component.global_id
                ))
            })?;
        if !instance.has_script_instance_for_global(component.global_id) {
            targets.push(script_mount_target(
                runtime,
                scripts,
                object,
                ScriptMountTargetKind::Drawable,
                &path,
            )?);
        }
    }
    for converter in runtime.data_converters() {
        if converter.type_name != "ScriptedDataConverter" {
            continue;
        }
        let is_project_converter = converter
            .uint_property("scriptAssetId")
            .and_then(|value| usize::try_from(value).ok())
            .is_some_and(|ordinal| scripts.is_project_data_converter_asset(ordinal));
        if is_project_converter {
            continue;
        }
        has_script_target = true;
        if !instance.has_scripted_data_converter_instance_for_global(converter.id) {
            targets.push(script_mount_target(
                runtime,
                scripts,
                converter,
                ScriptMountTargetKind::DataConverter,
                &path,
            )?);
        }
    }
    Ok((
        has_script_target,
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
    if !scripts.scripts_are_authenticated() {
        return Ok((false, Vec::new()));
    }
    let (mut has_script_target, root) = script_mount_group(
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
        has_script_target |= has_scripts;
        groups.push(group);
        Ok::<(), nuxie_runtime::ScriptError>(())
    };
    instance.try_visit_artboard_tree_instances_mut(&mut visitor)?;
    Ok((has_script_target, groups))
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
    fn attach(
        instance: &mut RuntimeArtboardInstance,
        kind: ScriptMountTargetKind,
        global_id: u32,
        script: Box<dyn ScriptInstance>,
    ) {
        match kind {
            ScriptMountTargetKind::Drawable => {
                instance.set_script_instance_for_global(global_id, script);
            }
            ScriptMountTargetKind::DataConverter => {
                instance.set_scripted_data_converter_instance_for_global(global_id, script);
            }
        }
    }

    let mut groups = VecDeque::from(prepared);
    if let Some(root) = groups.pop_front() {
        for (kind, global_id, script) in root.scripts {
            attach(instance, kind, global_id, script);
        }
    }
    let mut visitor = |_: usize, _: u32, nested: &mut RuntimeArtboardInstance| {
        if let Some(group) = groups.pop_front() {
            for (kind, global_id, script) in group.scripts {
                attach(nested, kind, global_id, script);
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
    // Converter tables are attached after the caller's ordinary data-bind
    // pass. Re-run the zero-time bind phase so the first factory-bearing
    // frame observes scripted conversion instead of the inert pass-through
    // placeholder used during instance construction.
    let mut changed = instance.advance_artboard_data_binds();
    let mut visitor = |_: usize, _: u32, nested: &mut RuntimeArtboardInstance| {
        changed |= nested.advance_artboard_data_binds();
        Ok::<(), std::convert::Infallible>(())
    };
    let result = instance.try_visit_artboard_tree_instances_mut(&mut visitor);
    match result {
        Ok(()) => {}
        Err(error) => match error {},
    }

    changed |= instance
        .flush_script_lifecycle_with_factory(factory)
        .map_err(|error| {
            error.with_context(format!(
                "root graph {} script lifecycle failed",
                instance.graph_global_id()
            ))
        })?;
    let mut visitor = |depth: usize, graph_global_id: u32, nested: &mut RuntimeArtboardInstance| {
        changed |= nested
            .flush_script_lifecycle_with_factory(factory)
            .map_err(|error| {
                error.with_context(format!(
                    "nested depth {depth} graph {graph_global_id} script lifecycle failed"
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
    {
        let scripts = file.scripts.borrow();
        if !scripts.scripts_are_authenticated()
            || !scripts
                .assets
                .iter()
                .any(|asset| asset.type_name == "ScriptAsset")
        {
            return Ok(false);
        }
    }
    let (has_script_target, groups) = collect_script_mount_groups(file, root_graph, instance)?;
    let mut prepared = file
        .scripts
        .borrow_mut()
        .prepare_mounts(&file.runtime, &groups, factory)?;
    validate_prepared_script_mount_topology(instance, &prepared.groups)?;

    // Validation is the final fallible step. Publish a cold candidate before
    // attaching its tables so every mounted handle always has a live owner;
    // attachment itself is now an infallible commit over the validated tree.
    if let Some(candidate) = prepared.candidate.take() {
        file.scripts.borrow_mut().ready = Some(candidate);
    }
    attach_prepared_script_mounts(instance, prepared.groups);

    // Facade execution fails closed: every concrete scripted target must
    // have an attached table before entering the lower runtime draw path.
    let (_, verified) = collect_script_mount_groups(file, root_graph, instance)?;
    if let Some(group) = verified.iter().find(|group| !group.targets.is_empty()) {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{} still has unattached scripted runtime instances",
            group.path
        )));
    }
    let changed = if has_script_target {
        flush_scripted_artboard_tree(instance, factory)?
    } else {
        false
    };

    // Script update refreshes component-list occurrences. A newly materialized
    // child is mounted on the next preparation call, but must not slip through
    // this draw without a table in the meantime.
    let (_, after_lifecycle) = collect_script_mount_groups(file, root_graph, instance)?;
    if let Some(group) = after_lifecycle
        .iter()
        .find(|group| !group.targets.is_empty())
    {
        return Err(nuxie_runtime::ScriptError::new(format!(
            "{} materialized an unattached scripted runtime instance during script lifecycle",
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
    if !file.scripts.borrow().scripts_are_authenticated() {
        return false;
    }
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
    runtime: Arc<RuntimeFile>,
    graph: Arc<GraphFile>,
    external_image_assets: BTreeMap<u32, Arc<[u8]>>,
    external_font_assets: ExternalFontAssetStore,
    #[cfg(feature = "scripting")]
    scripts: RefCell<FileScriptRuntime>,
}

#[derive(Default)]
struct ExternalFontAssetStore {
    assets: RwLock<BTreeMap<u32, Arc<[u8]>>>,
}

impl ExternalFontAssetStore {
    fn read(&self) -> RwLockReadGuard<'_, BTreeMap<u32, Arc<[u8]>>> {
        match self.assets.read() {
            Ok(assets) => assets,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn write(&self) -> RwLockWriteGuard<'_, BTreeMap<u32, Arc<[u8]>>> {
        match self.assets.write() {
            Ok(assets) => assets,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn snapshot(&self) -> BTreeMap<u32, Arc<[u8]>> {
        self.read().clone()
    }

    fn len(&self) -> usize {
        self.read().len()
    }

    fn insert_if_changed(&self, asset_id: u32, bytes: Arc<[u8]>) -> bool {
        let mut assets = self.write();
        if assets
            .get(&asset_id)
            .is_some_and(|current| current.as_ref() == bytes.as_ref())
        {
            return false;
        }
        assets.insert(asset_id, bytes);
        true
    }

    #[cfg(test)]
    fn get(&self, asset_id: &u32) -> Option<Arc<[u8]>> {
        self.read().get(asset_id).cloned()
    }

    #[cfg(test)]
    fn contains_key(&self, asset_id: &u32) -> bool {
        self.read().contains_key(asset_id)
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.read().is_empty()
    }
}

impl Clone for ExternalFontAssetStore {
    fn clone(&self) -> Self {
        Self {
            assets: RwLock::new(self.snapshot()),
        }
    }
}

/// Rejection from attaching host-provided bytes to a semantic file asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalAssetError {
    UnknownAsset {
        asset_id: u32,
    },
    WrongAssetKind {
        asset_id: u32,
        expected: &'static str,
        actual: &'static str,
    },
    InvalidFont {
        asset_id: u32,
    },
}

impl std::fmt::Display for ExternalAssetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAsset { asset_id } => {
                write!(formatter, "file has no asset with semantic id {asset_id}")
            }
            Self::WrongAssetKind {
                asset_id,
                expected,
                actual,
            } => write!(formatter, "asset {asset_id} is {actual}, not {expected}"),
            Self::InvalidFont { asset_id } => {
                write!(formatter, "asset {asset_id} bytes are not a valid font")
            }
        }
    }
}

impl std::error::Error for ExternalAssetError {}

/// Resource limits applied before and after binary import, but always before
/// the owned graph and script catalog are constructed.
///
/// [`Self::new`] and [`Default::default`] are deliberately bounded. Hosts that
/// accept larger trusted artifacts can raise individual ceilings explicitly;
/// [`Self::unbounded`] is reserved for already-authenticated, host-controlled
/// inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileImportLimits {
    max_input_bytes: Option<usize>,
    max_runtime_objects: Option<usize>,
    max_runtime_properties: Option<usize>,
    max_imported_file_assets: Option<usize>,
    max_file_asset_content_bytes: Option<usize>,
    max_total_file_asset_content_bytes: Option<usize>,
}

impl FileImportLimits {
    const DEFAULT_MAX_INPUT_BYTES: usize = 128 * 1024 * 1024;
    const DEFAULT_MAX_RUNTIME_OBJECTS: usize = 1_000_000;
    const DEFAULT_MAX_RUNTIME_PROPERTIES: usize = 1_000_000;
    const DEFAULT_MAX_IMPORTED_FILE_ASSETS: usize = 16_384;
    const DEFAULT_MAX_FILE_ASSET_CONTENT_BYTES: usize = 64 * 1024 * 1024;
    const DEFAULT_MAX_TOTAL_FILE_ASSET_CONTENT_BYTES: usize = 128 * 1024 * 1024;

    pub const fn new() -> Self {
        Self {
            max_input_bytes: Some(Self::DEFAULT_MAX_INPUT_BYTES),
            max_runtime_objects: Some(Self::DEFAULT_MAX_RUNTIME_OBJECTS),
            max_runtime_properties: Some(Self::DEFAULT_MAX_RUNTIME_PROPERTIES),
            max_imported_file_assets: Some(Self::DEFAULT_MAX_IMPORTED_FILE_ASSETS),
            max_file_asset_content_bytes: Some(Self::DEFAULT_MAX_FILE_ASSET_CONTENT_BYTES),
            max_total_file_asset_content_bytes: Some(
                Self::DEFAULT_MAX_TOTAL_FILE_ASSET_CONTENT_BYTES,
            ),
        }
    }

    pub const fn unbounded() -> Self {
        Self {
            max_input_bytes: None,
            max_runtime_objects: None,
            max_runtime_properties: None,
            max_imported_file_assets: None,
            max_file_asset_content_bytes: None,
            max_total_file_asset_content_bytes: None,
        }
    }

    pub const fn with_max_input_bytes(mut self, maximum: usize) -> Self {
        self.max_input_bytes = Some(maximum);
        self
    }

    pub const fn with_max_runtime_objects(mut self, maximum: usize) -> Self {
        self.max_runtime_objects = Some(maximum);
        self
    }

    /// Bound every serialized property occurrence decoded by the binary
    /// parser, including skipped/unknown/duplicate properties and properties
    /// on objects that ultimately become null slots. The same aggregate also
    /// covers header property-table entries and manifest name, path-entry, and
    /// path-component declarations.
    pub const fn with_max_runtime_properties(mut self, maximum: usize) -> Self {
        self.max_runtime_properties = Some(maximum);
        self
    }

    pub const fn with_max_imported_file_assets(mut self, maximum: usize) -> Self {
        self.max_imported_file_assets = Some(maximum);
        self
    }

    /// Bound each retained `FileAssetContents` payload occurrence. Repeated
    /// records are validated independently even though the runtime selects the
    /// final record as the asset's active contents.
    pub const fn with_max_file_asset_content_bytes(mut self, maximum: usize) -> Self {
        self.max_file_asset_content_bytes = Some(maximum);
        self
    }

    pub const fn with_max_total_file_asset_content_bytes(mut self, maximum: usize) -> Self {
        self.max_total_file_asset_content_bytes = Some(maximum);
        self
    }

    pub const fn max_input_bytes(self) -> Option<usize> {
        self.max_input_bytes
    }

    pub const fn max_runtime_objects(self) -> Option<usize> {
        self.max_runtime_objects
    }

    pub const fn max_runtime_properties(self) -> Option<usize> {
        self.max_runtime_properties
    }

    pub const fn max_imported_file_assets(self) -> Option<usize> {
        self.max_imported_file_assets
    }

    pub const fn max_file_asset_content_bytes(self) -> Option<usize> {
        self.max_file_asset_content_bytes
    }

    pub const fn max_total_file_asset_content_bytes(self) -> Option<usize> {
        self.max_total_file_asset_content_bytes
    }

    fn validate_input(self, bytes: &[u8]) -> Result<()> {
        if let Some(maximum) = self.max_input_bytes()
            && bytes.len() > maximum
        {
            bail!(
                "Rive file is {} bytes; the import limit is {maximum} bytes",
                bytes.len()
            );
        }
        Ok(())
    }
}

impl Default for FileImportLimits {
    fn default() -> Self {
        Self::new()
    }
}

fn read_runtime_file_for_facade_with_limits(
    bytes: &[u8],
    limits: FileImportLimits,
) -> Result<RuntimeFile> {
    if limits.max_runtime_objects().is_none() && limits.max_runtime_properties().is_none() {
        read_runtime_file_for_facade(bytes)
    } else {
        read_runtime_file_for_facade_with_parser_limits(
            bytes,
            limits.max_runtime_objects(),
            limits.max_runtime_properties(),
        )
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
            .field("graph", &self.graph)
            .field(
                "external_image_asset_count",
                &self.external_image_assets.len(),
            )
            .field(
                "external_font_asset_count",
                &self.external_font_assets.len(),
            );
        #[cfg(feature = "scripting")]
        debug.field("scripts", &self.scripts);
        debug.finish()
    }
}

impl Clone for File {
    fn clone(&self) -> Self {
        let runtime = Arc::clone(&self.runtime);
        let graph = Arc::clone(&self.graph);
        #[cfg(feature = "scripting")]
        let scripts = {
            let scripts = self.scripts.borrow();
            RefCell::new(FileScriptRuntime::new(
                Arc::clone(&scripts.assets),
                Arc::clone(&scripts.imports),
                scripts.authorization,
                scripts.execution_limits,
            ))
        };
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
    /// With scripting enabled, arbitrary imported files remain visual-only by
    /// default. Use `File::import_with_trusted_scripts` only after the host has
    /// authenticated the file bytes and selected explicit execution limits.
    pub fn import(bytes: &[u8]) -> Result<Self> {
        Self::import_with_limits(bytes, FileImportLimits::new())
    }

    /// Import `.riv` bytes while bounding allocations derived from the parsed
    /// file before constructing the owned runtime graph.
    pub fn import_with_limits(bytes: &[u8], limits: FileImportLimits) -> Result<Self> {
        limits.validate_input(bytes)?;
        let runtime = read_runtime_file_for_facade_with_limits(bytes, limits)
            .context("failed to import Rive file")?;
        Self::from_runtime_with_script_policy_and_limits(
            runtime,
            ScriptExecutionAuthorization::VisualOnly,
            inert_script_policy(),
            limits,
        )
    }

    /// Import host-authenticated `.riv` bytes with explicit, non-zero Luau
    /// memory and per-callback interrupt ceilings.
    #[cfg(feature = "scripting")]
    pub fn import_with_trusted_scripts(
        bytes: &[u8],
        execution_limits: ScriptExecutionLimits,
    ) -> Result<Self> {
        Self::import_with_trusted_scripts_and_limits(
            bytes,
            FileImportLimits::new(),
            execution_limits,
        )
    }

    /// Apply both binary-import allocation limits and trusted Luau execution
    /// limits before the File can enter its lazy script bootstrap path.
    #[cfg(feature = "scripting")]
    pub fn import_with_trusted_scripts_and_limits(
        bytes: &[u8],
        import_limits: FileImportLimits,
        execution_limits: ScriptExecutionLimits,
    ) -> Result<Self> {
        execution_limits
            .validate()
            .context("invalid trusted script execution limits")?;
        import_limits.validate_input(bytes)?;
        let runtime = read_runtime_file_for_facade_with_limits(bytes, import_limits)
            .context("failed to import Rive file")?;
        Self::from_runtime_with_script_policy_and_limits(
            runtime,
            ScriptExecutionAuthorization::Authenticated,
            Some(execution_limits),
            import_limits,
        )
    }

    /// Import `.riv` bytes using cryptographically bound script authority.
    ///
    /// [`ScriptImportCapability::visual_only`] keeps scripts inert. Executable
    /// authority can only be minted by authenticating a signed manifest that
    /// binds these exact artifact bytes.
    #[cfg(feature = "scripting")]
    pub fn import_with_script_capability(
        bytes: &[u8],
        capability: ScriptImportCapability,
    ) -> Result<Self> {
        let import_limits = FileImportLimits::new();
        import_limits.validate_input(bytes)?;
        let authorization = capability
            .execution_authorization_for(bytes)
            .context("script import capability does not match the artifact")?;
        let runtime = read_runtime_file_for_facade_with_limits(bytes, import_limits)
            .context("failed to import Rive file")?;
        Self::from_runtime_with_script_policy_and_limits(
            runtime,
            authorization,
            trusted_script_policy(authorization == ScriptExecutionAuthorization::Authenticated),
            import_limits,
        )
    }

    /// This is an explicit trust boundary for content the host authored or
    /// authenticated. Arbitrary uploaded or network-provided files should use
    /// [`Self::import`], which refuses to enter the script draw path.
    ///
    /// This compatibility wrapper applies [`ScriptExecutionLimits::new`]. New
    /// callers should prefer [`Self::import_with_trusted_scripts`] so the trust
    /// decision and resource policy remain visible at the call site.
    #[cfg(feature = "scripting")]
    pub fn import_with_unsigned_scripts(bytes: &[u8]) -> Result<Self> {
        Self::import_with_trusted_scripts(bytes, ScriptExecutionLimits::new())
    }

    pub(crate) fn from_runtime(runtime: RuntimeFile) -> Result<Self> {
        // RuntimeFile values constructed by Scene are authored in-process and
        // deliberately opt into unsigned editor bytecode execution.
        Self::from_runtime_with_script_authorization(
            runtime,
            ScriptExecutionAuthorization::Authenticated,
        )
    }

    fn from_runtime_with_script_authorization(
        runtime: RuntimeFile,
        authorization: ScriptExecutionAuthorization,
    ) -> Result<Self> {
        Self::from_runtime_with_script_policy_and_limits(
            runtime,
            authorization,
            trusted_script_policy(authorization == ScriptExecutionAuthorization::Authenticated),
            FileImportLimits::new(),
        )
    }

    fn from_runtime_with_script_policy_and_limits(
        runtime: RuntimeFile,
        authorization: ScriptExecutionAuthorization,
        execution_limits: FileScriptPolicy,
        limits: FileImportLimits,
    ) -> Result<Self> {
        #[cfg(not(feature = "scripting"))]
        let _ = (authorization, execution_limits);
        if let Some(maximum) = limits.max_runtime_objects()
            && runtime.objects.len() > maximum
        {
            bail!(
                "Rive file contains {} runtime objects; the import limit is {maximum}",
                runtime.objects.len()
            );
        }
        if let Some(maximum) = limits.max_imported_file_assets() {
            runtime
                .imported_file_assets_with_contents_bounded(maximum)
                .ok_or_else(|| {
                    anyhow::anyhow!("Rive file imports more than {maximum} FileAssets")
                })?;
        }
        let mut total_content_bytes = 0usize;
        for (object_id, object) in runtime.objects.iter().enumerate() {
            if runtime.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(object) = object.as_ref() else {
                continue;
            };
            if object.type_name != "FileAssetContents" {
                continue;
            }
            let content_bytes = object.bytes_property("bytes").map_or(0, <[u8]>::len);
            if let Some(maximum) = limits.max_file_asset_content_bytes()
                && content_bytes > maximum
            {
                bail!(
                    "Rive FileAssetContents object {object_id} contains {content_bytes} bytes; the per-content import limit is {maximum} bytes"
                );
            }
            total_content_bytes = total_content_bytes
                .checked_add(content_bytes)
                .context("Rive FileAsset content byte total overflowed usize")?;
            if let Some(maximum) = limits.max_total_file_asset_content_bytes()
                && total_content_bytes > maximum
            {
                bail!("Rive FileAssets contain more than {maximum} aggregate content bytes");
            }
        }
        anyhow::ensure!(
            embedded_fonts_are_parseable(&runtime),
            "embedded FontAsset bytes are not a valid font"
        );
        let graph = GraphFile::from_runtime_file(&runtime).context("failed to build Rive graph")?;
        Ok(Self {
            #[cfg(feature = "scripting")]
            scripts: RefCell::new(FileScriptRuntime::import(
                &runtime,
                authorization,
                execution_limits,
            )),
            runtime: Arc::new(runtime),
            graph: Arc::new(graph),
            external_image_assets: BTreeMap::new(),
            external_font_assets: ExternalFontAssetStore::default(),
        })
    }

    /// Attach already-loaded bytes to an external `ImageAsset` identity.
    ///
    /// This performs no I/O or decoding. `asset_id` is the serialized
    /// `FileAsset.assetId`, not an asset-list ordinal. Decode remains lazy
    /// until the first draw, and embedded file contents remain authoritative.
    pub fn attach_external_image_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalAssetError> {
        self.validate_external_asset_kind(asset_id, "ImageAsset")?;
        let bytes = Arc::<[u8]>::from(bytes);
        if self
            .external_image_assets
            .get(&asset_id)
            .is_some_and(|current| current.as_ref() == bytes.as_ref())
        {
            return Ok(());
        }
        self.external_image_assets.insert(asset_id, bytes);
        Ok(())
    }

    /// Attach validated bytes to an external `FontAsset` identity.
    ///
    /// This performs no I/O. `asset_id` is the serialized
    /// `FileAsset.assetId`, not an asset-list ordinal. Call this before sharing
    /// the file through [`Arc`]; every subsequently instantiated root and
    /// child artboard receives the same immutable snapshot. Embedded file
    /// contents remain authoritative during text layout.
    pub fn attach_external_font_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalAssetError> {
        self.attach_external_font_asset_bytes_shared(asset_id, bytes)
            .map(|_| ())
    }

    fn attach_external_font_asset_bytes_shared(
        &self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<bool, ExternalAssetError> {
        self.validate_external_asset_kind(asset_id, "FontAsset")?;
        if !RuntimeArtboardInstance::external_font_bytes_are_parseable(&bytes) {
            return Err(ExternalAssetError::InvalidFont { asset_id });
        }
        let bytes = Arc::<[u8]>::from(bytes);
        Ok(self.external_font_assets.insert_if_changed(asset_id, bytes))
    }

    fn validate_external_asset_kind(
        &self,
        asset_id: u32,
        expected: &'static str,
    ) -> std::result::Result<(), ExternalAssetError> {
        let Some(actual) = self
            .runtime
            .file_assets()
            .into_iter()
            .find(|asset| asset.uint_property("assetId") == Some(u64::from(asset_id)))
            .map(|asset| asset.type_name)
        else {
            return Err(ExternalAssetError::UnknownAsset { asset_id });
        };
        if actual != expected {
            return Err(ExternalAssetError::WrongAssetKind {
                asset_id,
                expected,
                actual,
            });
        }
        Ok(())
    }

    /// Compatibility spelling for attaching bytes to an external image asset.
    pub fn attach_image_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalImageAssetError> {
        self.attach_external_image_asset_bytes(asset_id, bytes)
            .map_err(|error| match error {
                ExternalAssetError::UnknownAsset { asset_id } => {
                    ExternalImageAssetError::UnknownAsset { asset_id }
                }
                ExternalAssetError::WrongAssetKind {
                    asset_id, actual, ..
                } => ExternalImageAssetError::WrongAssetKind { asset_id, actual },
                ExternalAssetError::InvalidFont { asset_id } => {
                    ExternalImageAssetError::WrongAssetKind {
                        asset_id,
                        actual: "FontAsset",
                    }
                }
            })
    }

    /// Compatibility spelling for attaching validated external font bytes.
    pub fn attach_font_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalFontAssetError> {
        self.attach_external_font_asset_bytes(asset_id, bytes)
            .map_err(|error| match error {
                ExternalAssetError::UnknownAsset { asset_id } => {
                    ExternalFontAssetError::UnknownAsset { asset_id }
                }
                ExternalAssetError::WrongAssetKind {
                    asset_id, actual, ..
                } => ExternalFontAssetError::WrongAssetKind { asset_id, actual },
                ExternalAssetError::InvalidFont { asset_id } => {
                    ExternalFontAssetError::InvalidFont { asset_id }
                }
            })
    }

    /// Low-level imported file data for advanced integrations.
    pub fn runtime(&self) -> &RuntimeFile {
        self.runtime.as_ref()
    }

    /// Low-level graph projection for advanced integrations.
    pub fn graph(&self) -> &GraphFile {
        self.graph.as_ref()
    }

    #[cfg(feature = "scripting")]
    fn advance_detached_view_models(&self) -> bool {
        self.scripts
            .borrow()
            .ready
            .as_ref()
            .is_some_and(|ready| ready.vm.advance_detached_view_models())
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

    /// Authored artboard width and height in artboard coordinates.
    pub fn dimensions(self) -> Option<(f32, f32)> {
        let artboard = self.file.runtime.artboard(self.index)?;
        Some((
            artboard.double_property("width")?,
            artboard.double_property("height")?,
        ))
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
        let external_font_assets = self.file.external_font_assets.snapshot();
        let raw = RuntimeArtboardInstance::from_graph_with_artboards_and_external_fonts(
            &self.file.runtime,
            self.graph(),
            &self.file.graph.artboards,
            &external_font_assets,
        )
        .with_context(|| {
            format!(
                "failed to instantiate artboard {}",
                self.name().unwrap_or("<unnamed>")
            )
        })?;
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
    let candidate = RuntimeRenderPaintCache::preallocate_for_artboard_tree_with_external_images(
        runtime,
        artboard,
        artboards,
        external_image_assets,
        factory,
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

    pub fn artboard_dimensions(&self) -> (f32, f32) {
        self.raw.artboard_dimensions()
    }

    pub fn artboard_bounds(&self) -> (f32, f32, f32, f32) {
        self.raw.artboard_bounds()
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
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
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
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        Ok(changed)
    }

    /// Return visible Shape and Text locals under `point`, front to back,
    /// including descendants reached through nested artboards and
    /// component-list items.
    pub fn hit_test(&mut self, point: Vec2D) -> Vec<usize> {
        self.raw.geometry_hit_test(point, &mut self.geometry)
    }

    /// Return visible Shape and Text local-id paths under `point`, front to
    /// back. Direct hits contain one local id; child-artboard hits are prefixed
    /// with their nested or component-list host local ids.
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
        Some(ViewModelInstance {
            raw: RuntimeOwnedViewModelHandle::new(raw),
        })
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
        Some(ViewModelInstance {
            raw: RuntimeOwnedViewModelHandle::new(raw),
        })
    }

    /// Instantiate a serialized ViewModel default as a fully owned mutable
    /// tree, preserving nested scalars and lists while allowing hot writes at
    /// every generated child path.
    pub fn instantiate_mutable_view_model_instance(
        &self,
        instance_index: usize,
    ) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::from_instance_mutable(
            &self.file.runtime,
            view_model_index,
            instance_index,
        )?;
        Some(ViewModelInstance {
            raw: RuntimeOwnedViewModelHandle::new(raw),
        })
    }

    /// Bind `view_model` to this artboard's own data binds and its nested
    /// artboard contexts, mirroring `artboard->bindViewModelInstance(...)` in
    /// the C++ runtime.
    ///
    /// The exact mutable view-model graph is retained by the artboard, so later
    /// mutations are visible on the next [`ArtboardInstance::advance`] without
    /// rebinding. State-machine instances created afterward inherit the same
    /// handle; an already-created machine must be bound explicitly through
    /// [`StateMachineInstance::bind_owned_view_model_handle`]. Returns whether
    /// the binding changed anything.
    ///
    pub fn bind_view_model(&mut self, view_model: &ViewModelInstance) -> bool {
        let mut changed = self
            .raw
            .bind_default_view_model_artboard_list_context(&self.file.runtime);
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
        changed
    }

    /// Return the main-only context retained by the most recent
    /// [`ArtboardInstance::bind_view_model`] call.
    pub fn owned_view_model_context(&self) -> Option<&RuntimeOwnedViewModelContext> {
        self.raw.owned_view_model_context()
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
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance(state_machines);
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        changed
    }

    /// Advance retained machines while allowing listener actions to mutate
    /// the same owned ViewModel context bound to this artboard.
    pub fn advance_with_state_machines_and_view_model(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        view_model: &mut ViewModelInstance,
    ) -> bool {
        if state_machines.is_empty() {
            return self.bind_view_model(view_model) | self.advance(elapsed_seconds);
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
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
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
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance(state_machines);
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        Ok(changed)
    }

    /// Factory-bearing mirror of
    /// [`Self::advance_with_state_machines_and_view_model`].
    pub fn try_advance_with_state_machines_and_view_model_and_factory(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        view_model: &mut ViewModelInstance,
        factory: &mut dyn Factory,
    ) -> Result<bool> {
        if state_machines.is_empty() {
            let changed = self.bind_view_model(view_model);
            return self
                .try_advance_with_factory(factory, elapsed_seconds)
                .map(|advanced| changed | advanced);
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
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
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
        if paint.needs_paint_preparation(&self.raw, artboard) {
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
        }
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
            .context("failed to draw Rive artboard")?;
        self.geometry.observe_presented_images(&self.raw, paint)?;
        Ok(())
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
        let external_font_assets = file.external_font_assets.snapshot();
        let raw = {
            let artboard = file
                .artboard(artboard_index)
                .with_context(|| format!("artboard index {artboard_index} out of range"))?;
            RuntimeArtboardInstance::from_graph_with_artboards_and_external_fonts(
                &file.runtime,
                artboard.graph(),
                &file.graph.artboards,
                &external_font_assets,
            )
            .with_context(|| {
                format!(
                    "failed to instantiate artboard {}",
                    artboard.name().unwrap_or("<unnamed>")
                )
            })?
        };
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

    pub fn artboard_dimensions(&self) -> (f32, f32) {
        self.raw.artboard_dimensions()
    }

    pub fn artboard_bounds(&self) -> (f32, f32, f32, f32) {
        self.raw.artboard_bounds()
    }

    /// Compatibility attachment path for an already-owned instance.
    ///
    /// New integrations should attach bytes to [`File`] before wrapping it in
    /// [`Arc`]. This method updates the exact shared File without replacing its
    /// live script VM, then refreshes the complete current tree plus its future
    /// child-build contexts.
    pub fn attach_font_asset_bytes(
        &mut self,
        asset_id: u32,
        bytes: Vec<u8>,
    ) -> std::result::Result<(), ExternalFontAssetError> {
        self.file
            .attach_external_font_asset_bytes_shared(asset_id, bytes)
            .map_err(|error| match error {
                ExternalAssetError::UnknownAsset { asset_id } => {
                    ExternalFontAssetError::UnknownAsset { asset_id }
                }
                ExternalAssetError::WrongAssetKind {
                    asset_id, actual, ..
                } => ExternalFontAssetError::WrongAssetKind { asset_id, actual },
                ExternalAssetError::InvalidFont { asset_id } => {
                    ExternalFontAssetError::InvalidFont { asset_id }
                }
            })?;
        let external_font_assets = self.file.external_font_assets.snapshot();
        self.raw
            .replace_external_font_asset_snapshot(&external_font_assets);
        Ok(())
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
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
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
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        Ok(changed)
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn prepare_flow_scripts(
        &mut self,
        factory: &mut dyn Factory,
    ) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
        let artboard = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(
                    "owned artboard instance graph is unavailable during script bootstrap",
                )
            })?;
        prepare_scripted_artboard_tree(&self.file, artboard, &mut self.raw, factory)
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn prepare_flow_listener_actions(
        &self,
        machine: &mut StateMachineInstance,
        factory: &mut dyn Factory,
        root_view_model: Option<&ViewModelInstance>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        instantiate_script_listener_actions(&self.file, machine, factory, root_view_model)
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn rehydrate_flow_listener_actions(
        &self,
        machine: &mut StateMachineInstance,
        root_view_model: Option<&ViewModelInstance>,
        previous_root_view_model: Option<&ViewModelInstance>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        rehydrate_script_listener_actions(
            &self.file,
            machine,
            root_view_model,
            previous_root_view_model,
        )
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn begin_flow_host_cycle(&self) -> Option<HostCycleCheckpoint> {
        self.file.scripts.borrow().begin_host_cycle()
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn rollback_flow_host_cycle(&self, checkpoint: HostCycleCheckpoint) {
        self.file.scripts.borrow().rollback_host_cycle(checkpoint);
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn drain_flow_host_commands(&self) -> Vec<LuaHostCommand> {
        self.file.scripts.borrow().drain_host_commands()
    }

    /// Return visible Shape and Text locals under `point`, front to back,
    /// including descendants reached through nested artboards and
    /// component-list items.
    pub fn hit_test(&mut self, point: Vec2D) -> Vec<usize> {
        self.raw.geometry_hit_test(point, &mut self.geometry)
    }

    /// Return visible Shape and Text local-id paths under `point`, front to
    /// back. Direct hits contain one local id; child-artboard hits are prefixed
    /// with their nested or component-list host local ids.
    pub fn hit_test_paths(&mut self, point: Vec2D) -> Vec<Vec<usize>> {
        self.raw.geometry_hit_test_paths(point, &mut self.geometry)
    }

    /// Resolve the frontmost concrete geometry occurrence for a native
    /// pointer dispatch. Pass this value to a state-machine instance's
    /// `pointer_*_with_event_context` methods so list/component occurrence
    /// identity survives even when the listener targets implementation-only
    /// hit geometry.
    pub fn pointer_event_context(&mut self, point: Vec2D) -> Option<StateMachineEventContext> {
        self.raw
            .geometry_hit_test_path_segments_with_bounds(point, &mut self.geometry)
            .first()
            .map(StateMachineEventContext::from_geometry_hit)
    }

    pub(crate) fn hit_test_path_segments_with_bounds(
        &mut self,
        point: Vec2D,
    ) -> Vec<RuntimeGeometryHit> {
        self.raw
            .geometry_hit_test_path_segments_with_bounds(point, &mut self.geometry)
    }

    pub(crate) fn geometry_path_segments_with_bounds(&mut self) -> Vec<RuntimeGeometryHit> {
        self.raw
            .geometry_path_segments_with_bounds(&mut self.geometry)
    }

    pub(crate) fn retained_geometry_path_segments_with_bounds(
        &mut self,
    ) -> Vec<RuntimeGeometryHit> {
        self.raw
            .retained_geometry_path_segments_with_bounds(&mut self.geometry)
    }

    pub(crate) fn semantic_text_path_segments_with_bounds(
        &mut self,
    ) -> Vec<RuntimeSemanticTextHit> {
        self.raw
            .semantic_text_path_segments_with_bounds(&mut self.geometry)
    }

    /// Return exact logical world bounds for one runtime-local object.
    pub fn world_bounds(&mut self, local_id: usize) -> Option<Aabb> {
        self.raw.geometry_world_bounds(local_id, &mut self.geometry)
    }

    pub(crate) fn register_intrinsic_image_dimensions(
        &mut self,
        asset_global: u32,
        width: u32,
        height: u32,
    ) -> std::result::Result<(), RuntimeImageDimensionConflict> {
        self.geometry
            .register_image_dimensions(&self.raw, asset_global, width, height)
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
        Some(ViewModelInstance {
            raw: RuntimeOwnedViewModelHandle::new(raw),
        })
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
        Some(ViewModelInstance {
            raw: RuntimeOwnedViewModelHandle::new(raw),
        })
    }

    /// See [`ArtboardInstance::instantiate_mutable_view_model_instance`].
    pub fn instantiate_mutable_view_model_instance(
        &self,
        instance_index: usize,
    ) -> Option<ViewModelInstance> {
        let view_model_index = self.view_model_index()?;
        let raw = RuntimeOwnedViewModelInstance::from_instance_mutable(
            &self.file.runtime,
            view_model_index,
            instance_index,
        )?;
        Some(ViewModelInstance {
            raw: RuntimeOwnedViewModelHandle::new(raw),
        })
    }

    /// See [`ArtboardInstance::bind_view_model`].
    pub fn bind_view_model(&mut self, view_model: &ViewModelInstance) -> bool {
        let mut changed = self
            .raw
            .bind_default_view_model_artboard_list_context(&self.file.runtime);
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
        changed
    }

    /// See [`ArtboardInstance::owned_view_model_context`].
    pub fn owned_view_model_context(&self) -> Option<&RuntimeOwnedViewModelContext> {
        self.raw.owned_view_model_context()
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
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance(state_machines);
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        changed
    }

    /// Owning mirror of
    /// [`ArtboardInstance::advance_with_state_machines_and_view_model`].
    pub fn advance_with_state_machines_and_view_model(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        view_model: &mut ViewModelInstance,
    ) -> bool {
        if state_machines.is_empty() {
            return self.bind_view_model(view_model) | self.advance(elapsed_seconds);
        }
        let mut changed = false;
        #[cfg(feature = "scripting")]
        if let Some(artboard) = self.file.graph.artboards.get(self.artboard_index) {
            changed |=
                queue_root_script_advance(&self.file, artboard, &mut self.raw, elapsed_seconds);
        }
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
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
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance(state_machines);
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        Ok(changed)
    }

    /// Owning mirror of
    /// [`ArtboardInstance::try_advance_with_state_machines_and_view_model_and_factory`].
    pub fn try_advance_with_state_machines_and_view_model_and_factory(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        view_model: &mut ViewModelInstance,
        factory: &mut dyn Factory,
    ) -> Result<bool> {
        if state_machines.is_empty() {
            let changed = self.bind_view_model(view_model);
            return self
                .try_advance_with_factory(factory, elapsed_seconds)
                .map(|advanced| changed | advanced);
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
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
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
        if paint.needs_paint_preparation(&self.raw, artboard) {
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
        }
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
            .context("failed to draw Rive artboard")?;
        self.geometry.observe_presented_images(&self.raw, paint)?;
        Ok(())
    }
}

/// Owned view-model context for driving an artboard's data binds.
///
/// Instantiate one from an [`ArtboardInstance`], set properties by name path,
/// bind it with [`ArtboardInstance::bind_view_model`], then advance and draw.
/// The context owns a shared handle to the view model's values. Clones retain
/// the same mutable graph, and bindings observe later mutations on their next
/// advance. It does not borrow the originating [`File`] and is only meaningful
/// when bound back to the artboard it came from.
///
/// Property paths address nested view models with `/` separators (for example
/// `"child/width"`); a single segment addresses a property on the root view
/// model. Every setter returns whether a matching, settable property existed
/// and its value changed.
#[derive(Debug, Clone)]
pub struct ViewModelInstance {
    raw: RuntimeOwnedViewModelHandle,
}

impl ViewModelInstance {
    /// Low-level immutable access to the owned context.
    pub fn raw(&self) -> Ref<'_, RuntimeOwnedViewModelInstance> {
        self.raw.borrow()
    }

    /// Low-level mutable owned context for advanced integrations.
    pub fn raw_mut(&self) -> RefMut<'_, RuntimeOwnedViewModelInstance> {
        self.raw.borrow_mut()
    }

    /// Shared low-level handle for binding this exact mutable graph.
    pub fn handle(&self) -> &RuntimeOwnedViewModelHandle {
        &self.raw
    }

    /// Set a number property by name path. Returns whether the property existed
    /// and changed.
    pub fn set_number(&mut self, name_path: &str, value: f32) -> bool {
        self.raw
            .borrow_mut()
            .set_number_by_property_name_path(name_path, value)
    }

    /// Set a boolean property by name path. Returns whether the property existed
    /// and changed.
    pub fn set_bool(&mut self, name_path: &str, value: bool) -> bool {
        self.raw
            .borrow_mut()
            .set_boolean_by_property_name_path(name_path, value)
    }

    /// Set a string property by name path. The value is stored as its UTF-8
    /// bytes. Returns whether the property existed and changed.
    pub fn set_string(&mut self, name_path: &str, value: &str) -> bool {
        self.raw
            .borrow_mut()
            .set_string_by_property_name_path(name_path, value.as_bytes())
    }

    /// Set an enum property by its numeric value at the given name path. Returns
    /// whether the property existed and changed. (Enum-by-value-name is not
    /// exposed here because the owned context resolves enums by index.)
    pub fn set_enum(&mut self, name_path: &str, value: u64) -> bool {
        self.raw
            .borrow_mut()
            .set_enum_by_property_name_path(name_path, value)
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

    fn imported_script_assets_bytes(payloads: &[&[u8]]) -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        for (ordinal, payload) in payloads.iter().enumerate() {
            push_object(&mut bytes, "ScriptAsset", |bytes| {
                push_uint(bytes, "ScriptAsset", "assetId", ordinal as u64);
            });
            push_object(&mut bytes, "FileAssetContents", |bytes| {
                push_blob(bytes, "FileAssetContents", "bytes", payload);
            });
        }
        bytes
    }

    fn imported_script_asset_bytes() -> Vec<u8> {
        imported_script_assets_bytes(&[&[0, 1, 2, 3]])
    }

    fn imported_script_asset_with_repeated_contents_bytes(payloads: &[&[u8]]) -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ScriptAsset", |bytes| {
            push_uint(bytes, "ScriptAsset", "assetId", 0);
        });
        for payload in payloads {
            push_object(&mut bytes, "FileAssetContents", |bytes| {
                push_blob(bytes, "FileAssetContents", "bytes", payload);
            });
        }
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

    fn imported_manifest_asset_bytes() -> Vec<u8> {
        // One name entry: section=0, section bytes=[count=1, id=7,
        // string-length=1, 'a']. The parser budget charges the ManifestAsset
        // and FileAssetContents properties plus this declared entry.
        let manifest = [0, 4, 1, 7, 1, b'a'];
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 993);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ManifestAsset", |bytes| {
            push_uint(bytes, "ManifestAsset", "assetId", 0);
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(bytes, "FileAssetContents", "bytes", &manifest);
        });
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
    fn bounded_import_rejects_input_before_binary_parser_allocation() {
        let bytes = imported_script_asset_bytes();
        let error = File::import_with_limits(
            &bytes,
            FileImportLimits::new().with_max_input_bytes(bytes.len() - 1),
        )
        .expect_err("an oversized input must be rejected before parsing");
        assert!(error.to_string().contains("import limit"), "{error:#}");

        File::import_with_limits(
            &bytes,
            FileImportLimits::new().with_max_input_bytes(bytes.len()),
        )
        .expect("the exact input-byte bound admits parsing");
    }

    #[test]
    fn bounded_import_rejects_runtime_object_and_asset_content_growth() {
        let bytes = imported_script_asset_bytes();

        let object_error =
            File::import_with_limits(&bytes, FileImportLimits::new().with_max_runtime_objects(2))
                .expect_err("the fixture has three runtime objects");
        assert!(
            format!("{object_error:#}").contains("runtime objects"),
            "{object_error:#}"
        );

        let per_content_error = File::import_with_limits(
            &bytes,
            FileImportLimits::new().with_max_file_asset_content_bytes(3),
        )
        .expect_err("the script payload is four bytes");
        assert!(
            per_content_error
                .to_string()
                .contains("per-content import limit"),
            "{per_content_error:#}"
        );

        let aggregate_error = File::import_with_limits(
            &bytes,
            FileImportLimits::new().with_max_total_file_asset_content_bytes(3),
        )
        .expect_err("the aggregate payload is four bytes");
        assert!(
            aggregate_error
                .to_string()
                .contains("aggregate content bytes"),
            "{aggregate_error:#}"
        );
    }

    #[test]
    fn bounded_import_counts_every_retained_file_asset_contents_payload() {
        let bytes = imported_script_asset_with_repeated_contents_bytes(&[&[0, 1, 2, 3], &[4, 5]]);

        let per_content_error = File::import_with_limits(
            &bytes,
            FileImportLimits::new().with_max_file_asset_content_bytes(3),
        )
        .expect_err("an earlier four-byte contents record must not be hidden by its replacement");
        assert!(
            per_content_error
                .to_string()
                .contains("per-content import limit"),
            "{per_content_error:#}"
        );

        let aggregate_error = File::import_with_limits(
            &bytes,
            FileImportLimits::new().with_max_total_file_asset_content_bytes(5),
        )
        .expect_err("both retained contents payloads must contribute to the aggregate limit");
        assert!(
            aggregate_error
                .to_string()
                .contains("aggregate content bytes"),
            "{aggregate_error:#}"
        );

        File::import_with_limits(
            &bytes,
            FileImportLimits::new()
                .with_max_file_asset_content_bytes(4)
                .with_max_total_file_asset_content_bytes(6),
        )
        .expect("the exact per-content and aggregate bounds admit the repeated contents records");
    }

    #[test]
    fn facade_object_limit_precedes_next_record_decode_and_unbounded_stays_available() {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        bytes.push(0x80);

        let bounded =
            File::import_with_limits(&bytes, FileImportLimits::new().with_max_runtime_objects(1))
                .expect_err("the parser must reject before decoding compact object two");
        assert!(
            format!("{bounded:#}").contains("more than 1 runtime objects"),
            "{bounded:#}"
        );

        let unbounded = File::import_with_limits(&bytes, FileImportLimits::unbounded())
            .expect_err("the explicit unbounded reader reaches the malformed second record");
        assert!(
            !format!("{unbounded:#}").contains("more than 1 runtime objects"),
            "{unbounded:#}"
        );
    }

    #[test]
    fn facade_property_limit_covers_values_and_manifest_declared_work() {
        let node_type = definition_by_name("Node")
            .expect("Node schema")
            .type_key
            .int;
        let x_key = property_key("Node", "x");
        let mut malformed_value = b"RIVE".to_vec();
        push_var_uint(&mut malformed_value, 7);
        push_var_uint(&mut malformed_value, 0);
        push_var_uint(&mut malformed_value, 994);
        push_var_uint(&mut malformed_value, 0);
        push_object(&mut malformed_value, "Backboard", |_| {});
        push_var_uint(&mut malformed_value, u64::from(node_type));
        for _ in 0..2 {
            push_var_uint(&mut malformed_value, u64::from(x_key));
            malformed_value.extend_from_slice(&1.0f32.to_le_bytes());
        }
        push_var_uint(&mut malformed_value, u64::from(x_key));

        let error = File::import_with_limits(
            &malformed_value,
            FileImportLimits::new().with_max_runtime_properties(2),
        )
        .expect_err("property N+1 must be rejected before its missing value is decoded");
        assert!(
            format!("{error:#}").contains("runtime object properties"),
            "{error:#}"
        );

        let manifest = imported_manifest_asset_bytes();
        File::import_with_limits(
            &manifest,
            FileImportLimits::new().with_max_runtime_properties(3),
        )
        .expect("two object properties plus one manifest name fit the exact boundary");
        let error = File::import_with_limits(
            &manifest,
            FileImportLimits::new().with_max_runtime_properties(2),
        )
        .expect_err("manifest declarations share the facade property budget");
        assert!(
            format!("{error:#}").contains("manifest name entries"),
            "{error:#}"
        );
    }

    #[test]
    fn ordinary_import_keeps_the_bounded_script_catalog_inert_and_shared() {
        let bytes = imported_script_asset_bytes();

        let inert = File::import(&bytes).expect("ordinary import remains available");
        let inert_assets = {
            let scripts = inert.scripts.borrow();
            assert_eq!(
                scripts.authorization,
                ScriptExecutionAuthorization::VisualOnly
            );
            assert!(scripts.execution_limits.is_none());
            assert!(scripts.ready.is_none());
            assert_eq!(scripts.assets.len(), 1);
            Arc::clone(&scripts.assets)
        };
        let cloned = inert.clone();
        assert!(Arc::ptr_eq(&inert_assets, &cloned.scripts.borrow().assets));

        let trusted =
            File::import_with_unsigned_scripts(&bytes).expect("explicitly trusted import succeeds");
        let scripts = trusted.scripts.borrow();
        assert_eq!(
            scripts.authorization,
            ScriptExecutionAuthorization::Authenticated
        );
        assert!(scripts.execution_limits.is_some());
        let assets = scripts.assets.as_ref();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].payload.as_deref(), Some([0, 1, 2, 3].as_slice()));
    }

    #[test]
    fn project_converter_classification_is_retained_by_dense_asset_ordinal() {
        let bytes = imported_script_assets_bytes(&[b"ordinary script", b"NUXPCV1\0{}"]);
        let file = File::import(&bytes).expect("script asset catalog imports");
        let scripts = file.scripts.borrow();

        assert!(!scripts.is_project_data_converter_asset(0));
        assert!(scripts.is_project_data_converter_asset(1));
        assert!(!scripts.is_project_data_converter_asset(2));
    }
}

#[cfg(test)]
mod owned_instance_tests {
    use super::*;
    #[cfg(feature = "scripting")]
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
    use nuxie_render_api::RecordingFactory;

    const FIXTURE: &[u8] = include_bytes!("../../../fixtures/graph/dependency_test.riv");

    fn external_fixture(relative: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets")
        .join(relative);
        std::fs::read(path).expect("read external fixture")
    }

    fn first_semantic_asset_id(file: &File, kind: &str) -> u32 {
        let value = file
            .runtime()
            .file_assets()
            .into_iter()
            .find(|asset| asset.type_name == kind)
            .and_then(|asset| asset.uint_property("assetId"))
            .unwrap_or_else(|| panic!("fixture has no semantic {kind} id"));
        u32::try_from(value).expect("semantic asset id fits u32")
    }

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

    #[test]
    fn file_external_asset_store_validates_ids_and_replaces_deterministically() {
        let mut image_file =
            File::import(&external_fixture("hosted_image_file.riv")).expect("import image file");
        let image_id = first_semantic_asset_id(&image_file, "ImageAsset");
        assert_eq!(
            image_file.attach_external_image_asset_bytes(u32::MAX, vec![1]),
            Err(ExternalAssetError::UnknownAsset { asset_id: u32::MAX })
        );
        assert_eq!(
            image_file.attach_external_font_asset_bytes(image_id, vec![1]),
            Err(ExternalAssetError::WrongAssetKind {
                asset_id: image_id,
                expected: "FontAsset",
                actual: "ImageAsset",
            })
        );

        image_file
            .attach_external_image_asset_bytes(image_id, vec![1, 2, 3])
            .expect("attach image bytes");
        let first_image = Arc::clone(
            image_file
                .external_image_assets
                .get(&image_id)
                .expect("stored image"),
        );
        image_file
            .attach_external_image_asset_bytes(image_id, vec![1, 2, 3])
            .expect("repeat identical image bytes");
        assert!(Arc::ptr_eq(
            &first_image,
            image_file
                .external_image_assets
                .get(&image_id)
                .expect("same stored image")
        ));
        image_file
            .attach_external_image_asset_bytes(image_id, vec![4, 5])
            .expect("replace image bytes");
        assert_eq!(
            image_file
                .external_image_assets
                .get(&image_id)
                .map(AsRef::as_ref),
            Some(&[4, 5][..])
        );

        let mut font_file =
            File::import(&external_fixture("hosted_font_file.riv")).expect("import font file");
        let font_id = first_semantic_asset_id(&font_file, "FontAsset");
        assert_eq!(
            font_file.attach_external_font_asset_bytes(font_id, b"not a font".to_vec()),
            Err(ExternalAssetError::InvalidFont { asset_id: font_id })
        );
        assert!(!font_file.external_font_assets.contains_key(&font_id));

        let first_font_bytes = external_fixture("fonts/Inter_18pt-Regular.ttf");
        font_file
            .attach_external_font_asset_bytes(font_id, first_font_bytes.clone())
            .expect("attach valid font");
        let first_font = font_file
            .external_font_assets
            .get(&font_id)
            .expect("stored font");
        font_file
            .attach_external_font_asset_bytes(font_id, first_font_bytes)
            .expect("repeat identical font bytes");
        assert!(Arc::ptr_eq(
            &first_font,
            &font_file
                .external_font_assets
                .get(&font_id)
                .expect("same stored font")
        ));

        let replacement_font = external_fixture("Montserrat.ttf");
        font_file
            .attach_external_font_asset_bytes(font_id, replacement_font.clone())
            .expect("replace valid font");
        assert_eq!(
            font_file.external_font_assets.get(&font_id).as_deref(),
            Some(replacement_font.as_slice())
        );
        let cloned_file = font_file.clone();
        assert!(Arc::ptr_eq(
            &font_file
                .external_font_assets
                .get(&font_id)
                .expect("source font"),
            &cloned_file
                .external_font_assets
                .get(&font_id)
                .expect("cloned font")
        ));
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn file_script_runtime_extracts_scopes_and_nested_library_edges() {
        let property = |key, value| AuthoringProperty { key, value };
        let runtime = RuntimeFile::from_authoring_records(vec![
            AuthoringRecord {
                type_key: 23,
                properties: vec![],
            },
            AuthoringRecord {
                type_key: 558,
                properties: vec![
                    property(203, AuthoringValue::String("InnerLib".to_owned())),
                    property(798, AuthoringValue::Uint(21)),
                    property(799, AuthoringValue::Uint(4)),
                    property(1037, AuthoringValue::Uint(20)),
                    property(1038, AuthoringValue::Uint(6)),
                ],
            },
            AuthoringRecord {
                type_key: 529,
                properties: vec![
                    property(203, AuthoringValue::String("mesh".to_owned())),
                    property(926, AuthoringValue::String("config".to_owned())),
                    property(914, AuthoringValue::Bool(true)),
                    property(1037, AuthoringValue::Uint(21)),
                    property(1038, AuthoringValue::Uint(4)),
                ],
            },
        ])
        .expect("authored scripting asset graph imports");

        let scripts = FileScriptRuntime::import(
            &runtime,
            ScriptExecutionAuthorization::Authenticated,
            Some(ScriptExecutionLimits::new()),
        );
        let mesh = scripts
            .assets
            .iter()
            .find(|asset| asset.type_name == "ScriptAsset")
            .expect("script asset extracted");
        assert_eq!(mesh.name, "config/mesh");
        assert_eq!(mesh.scope, ScopeKey::new(21, 4));

        assert_eq!(scripts.imports.len(), 1);
        let import = &scripts.imports[0];
        assert_eq!(import.name, "InnerLib");
        assert_eq!(import.caller, ScopeKey::new(20, 6));
        assert_eq!(import.target, ScopeKey::new(21, 4));
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn file_script_runtime_resolves_exported_library_scope_probe() {
        let Some(rive_runtime_dir) = std::env::var_os("RIVE_RUNTIME_DIR") else {
            // The candidate-only fixture is exercised by the upstream-sync
            // gates; hermetic scope behavior is covered by nuxie-scripting.
            return;
        };
        let fixture = std::path::PathBuf::from(rive_runtime_dir)
            .join("tests/unit_tests/assets/scope_probe.riv");
        if !fixture.exists() {
            return;
        }
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", fixture.display()));
        let runtime = read_runtime_file_for_facade(&bytes).expect("scope probe imports");
        let scripts = FileScriptRuntime::import(
            &runtime,
            ScriptExecutionAuthorization::Authenticated,
            Some(ScriptExecutionLimits::new()),
        );
        let mut factory = RecordingFactory::new();

        let ready = scripts
            .build_candidate(&runtime, &mut factory)
            .expect("scoped modules register through serialized library pins");
        let (lib, has_decode, cached, bare_leaked): (i64, i64, i64, bool) = ready
            .vm
            .eval(
                "local probe = require('scope_probe')\n\
                 local bareLeaked = pcall(require, 'draco')\n\
                 return probe.lib, probe.hasDecode, probe.cached, bareLeaked",
            )
            .expect("root scope probe reads registered results");
        assert_eq!((lib, has_decode, cached), (1, 1, 1));
        assert!(
            !bare_leaked,
            "a root bare require leaked into library scope"
        );
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn file_script_bootstrap_seeds_data_before_registration() {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/script_create_viewmodel_instance.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let runtime = read_runtime_file_for_facade(&bytes).expect("fixture imports");
        let scripts = FileScriptRuntime::import(
            &runtime,
            ScriptExecutionAuthorization::Authenticated,
            Some(ScriptExecutionLimits::new()),
        );
        let model_name = nuxie_runtime::script_view_models(&runtime)
            .keys()
            .next()
            .cloned()
            .expect("fixture contains a view-model definition");
        let mut factory = RecordingFactory::new();

        let ready = scripts
            .build_candidate(&runtime, &mut factory)
            .expect("scripts register with Data initialized");
        let has_constructor: bool = ready
            .vm
            .eval(&format!(
                "return Data[{model_name:?}] ~= nil and type(Data[{model_name:?}].new) == 'function'"
            ))
            .expect("Data constructor probe runs");
        assert!(has_constructor);
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
        let image_asset_id = file
            .runtime
            .file_assets()
            .into_iter()
            .find(|asset| asset.type_name == "ImageAsset")
            .expect("image asset")
            .uint_property("assetId")
            .and_then(|value| u32::try_from(value).ok())
            .expect("semantic image asset id");
        assert_eq!(
            file.external_image_assets
                .get(&image_asset_id)
                .map(AsRef::as_ref),
            Some([4, 5, 6].as_slice())
        );
        assert_eq!(
            file.clone()
                .external_image_assets
                .get(&image_asset_id)
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
        let attached = file
            .external_font_assets
            .get(&8)
            .expect("valid attachment is retained by semantic id");
        assert_eq!(
            file.attach_font_asset_bytes(8, b"invalid replacement".to_vec()),
            Err(ExternalFontAssetError::InvalidFont { asset_id: 8 })
        );
        assert!(Arc::ptr_eq(
            &attached,
            &file
                .external_font_assets
                .get(&8)
                .expect("rejected replacement preserves the prior bytes")
        ));
        assert!(Arc::ptr_eq(
            &attached,
            &file
                .clone()
                .external_font_assets
                .get(&8)
                .expect("File::clone retains the exact attachment")
        ));
    }
}
