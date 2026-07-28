//! Public Rust API facade for loading and drawing Rive files.
//!
//! This crate keeps the user-facing surface narrow while the lower-level
//! crates continue to carry the import, graph, runtime, and renderer details.

use std::cell::{Ref, RefMut};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "scripting")]
use std::{
    cell::RefCell,
    collections::{BTreeSet, VecDeque},
    rc::Rc,
};

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
    ArtboardInstance as RuntimeArtboardInstance, RuntimeGeometryHit, RuntimeImageDimensionConflict,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance, RuntimeSemanticTextHit,
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
    GpuCanvasShaderBinding, GpuCanvasShaderEntry, GpuCanvasShaderEntrySelection,
    GpuCanvasShaderResourceKind, GpuCanvasShaderStage, GpuCanvasShaderTextureSampleType,
    GpuCanvasShaderTextureViewDimension, ImageDecodeError, ImageFilter, ImageSampler, ImageWrap,
    Mat2D, PathVerb, RawPath, RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType,
    RenderGpuCanvasShader, RenderImage, RenderPaint, RenderPaintStyle, RenderPath, RenderShader,
    Renderer, StrokeCap, StrokeJoin, Vec2D,
};
#[cfg(all(feature = "renderer", any(target_os = "ios", target_os = "macos")))]
pub use nuxie_renderer::{
    ApplePresentationCompletion, AppleSurface, SurfaceDisposition, SurfaceError,
};
#[cfg(all(feature = "renderer", target_arch = "wasm32"))]
pub use nuxie_renderer::{
    BrowserFactory, BrowserFactory as DefaultRendererFactory, BrowserFrame,
    BrowserFrame as DefaultRendererFrame, BrowserResizeError,
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
    RuntimeStateMachineInput, ScriptCoreString, ScriptError, ScriptHost, ScriptInstance,
    ScriptMethod, ScriptModule, ScriptModuleFailure, ScriptValue, ScriptingVm,
    StateMachineInputInstance, StateMachineInputKind, StateMachineInstance,
    StateMachineReportedEvent,
};
use nuxie_runtime::{RuntimeFileStateMachineActionCatalog, RuntimeFileViewModelInstanceCatalog};

fn advance_and_apply_keep_going(
    changed: bool,
    elapsed_seconds: f32,
    state_machines: &[StateMachineInstance],
) -> bool {
    // C++ `advanceAndApply` forces zero-second frames to keep going and
    // includes reports created by this frame in the facade return
    // (`state_machine_instance.cpp:2608-2613,2663-2665`). Raw
    // `StateMachineInstance::advance` deliberately retains its own return
    // semantics for callers that do not run the full apply pipeline.
    changed
        || elapsed_seconds == 0.0
        || state_machines.iter().any(|instance| {
            instance.reported_event_count() != 0
                || instance.has_pending_listener_view_model_reports()
        })
}

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
    bare_name: String,
    name: String,
    scope: ScopeKey,
    is_module: bool,
    serialized_implemented_methods: u32,
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
                    bare_name: name.to_owned(),
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
                    serialized_implemented_methods: entry
                        .asset
                        .uint_property("serializedImplementedMethods")
                        .unwrap_or((1 << 21) - 1)
                        as u32,
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
        let mut registered_shader_aliases = BTreeSet::new();
        for asset in assets
            .iter()
            .filter(|asset| asset.type_name == "ShaderAsset")
        {
            let aliases = [&asset.bare_name, &asset.name]
                .into_iter()
                .filter(|alias| registered_shader_aliases.insert((*alias).clone()))
                .map(String::as_str)
                .collect::<Vec<_>>();
            if aliases.is_empty() {
                continue;
            }
            vm.register_gpu_canvas_shader_asset_aliases(
                &aliases,
                asset.payload.as_deref().unwrap_or_default(),
            )
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
    serialized_implemented_methods: u32,
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
    scripts: Vec<(ScriptMountTargetKind, u32, u32, Box<dyn ScriptInstance>)>,
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
            if nuxie_runtime::scripted_object_inits(target.serialized_implemented_methods) {
                // Pinned `ScriptedObject::tryLuaUserInit` resolves `init`
                // exactly once and calls that exact value. A metatable may
                // legally return a different value on a second lookup, so the
                // facade must not probe with `has_method` first
                // (`scripted_object.cpp:331-382`).
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
                        group.path, target.global_id, target.asset_ordinal, target.asset_name
                    )));
                }
            }
            scripts.push((
                target.kind,
                target.global_id,
                target.serialized_implemented_methods,
                script,
            ));
        }
        prepared.push(PreparedScriptMountGroup {
            graph_global_id: group.graph_global_id,
            scripts,
        });
    }
    Ok(prepared)
}

#[cfg(feature = "scripting")]
fn scripted_listener_action_or_inert<T>(
    result: std::result::Result<T, nuxie_runtime::ScriptError>,
) -> std::result::Result<Option<T>, nuxie_runtime::ScriptError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.resource_code().is_some() => Err(error),
        // Pinned C++ ignores ScriptAsset generator/init/hydration failure and
        // retains the stateful ScriptedListenerAction with no live Lua table.
        // Rust's typed resource-limit error remains the binding safety fence.
        Err(_) => Ok(None),
    }
}

#[cfg(feature = "scripting")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptedListenerDataContextPhase {
    /// `cloneScriptedObject()->reinit()` runs before the state-machine
    /// occurrence receives its live DataContext.
    Cold,
    /// Construction's later `initScriptedObjects()` pass and every explicit
    /// rebind see the occurrence-owned DataContext, with the facade root used
    /// only when no occurrence-owned context exists.
    Live,
}

#[cfg(feature = "scripting")]
fn instantiate_state_machine_data_converters(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    factory: Option<&mut dyn Factory>,
    root_view_model: Option<&ViewModelInstance>,
    explicit_rebind: bool,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let steps = machine.state_machine_data_converter_bind_steps();
    if steps.is_empty() {
        return Ok(());
    }
    let (context_view_model, context_parent_view_models) = machine
        .scripted_listener_data_context_view_models(
            file.runtime(),
            root_view_model.map(ViewModelInstance::handle),
        );
    let mut factory = factory;

    // Execute the complete C++ virtual call stack one outer DataBind at a
    // time. A Scripted converter may replace a nested ViewModel during init;
    // the next authored outer bind must resolve only after that callback.
    for step in steps {
        let (parent_data_bind_index, converter_path, converter_global_id, inits) = match step {
            nuxie_runtime::RuntimeStateMachineDataConverterBindStep::BindOuter {
                data_bind_index,
            } => {
                let _ = machine.bind_state_machine_data_bind_source(data_bind_index);
                continue;
            }
            nuxie_runtime::RuntimeStateMachineDataConverterBindStep::BindConverter {
                data_bind_index,
                converter_path,
            } => {
                // Do not retain a RefCell read guard across ScriptedDataConverter
                // callbacks. Its init may replace a nested ViewModel before the
                // next authored DataBind resolves, exactly as C++ permits.
                let raw_root_view_model = root_view_model.map(ViewModelInstance::raw);
                let result = machine.bind_state_machine_data_converter_own_sources(
                    file.runtime(),
                    raw_root_view_model.as_deref(),
                    data_bind_index,
                    &converter_path,
                    explicit_rebind,
                );
                drop(raw_root_view_model);
                let _ = result;
                continue;
            }
            nuxie_runtime::RuntimeStateMachineDataConverterBindStep::Rehydrate {
                data_bind_index,
                converter_path,
                converter_global_id,
                inits,
            } => (data_bind_index, converter_path, converter_global_id, inits),
            nuxie_runtime::RuntimeStateMachineDataConverterBindStep::RebindFinalInput {
                data_bind_index,
                converter_path,
                converter_input_index,
                inner_data_bind_index,
            } => {
                let raw_root_view_model = root_view_model.map(ViewModelInstance::raw);
                let result = machine.rebind_state_machine_data_converter_final_input(
                    file.runtime(),
                    raw_root_view_model.as_deref(),
                    data_bind_index,
                    &converter_path,
                    converter_input_index,
                    inner_data_bind_index,
                );
                drop(raw_root_view_model);
                let _ = result;
                continue;
            }
            nuxie_runtime::RuntimeStateMachineDataConverterBindStep::FinalizeOuter {
                data_bind_index,
            } => {
                let _ = machine.finalize_state_machine_data_bind_source(data_bind_index);
                continue;
            }
        };

        if !machine.has_scripted_data_converter_instance(parent_data_bind_index, &converter_path) {
            let scripts = file.scripts.borrow();
            let Some(ready) = scripts.ready.as_ref() else {
                continue;
            };
            if let Some(active_factory) = factory.as_deref_mut()
                && ready.factory_domain != render_factory_domain(active_factory)
            {
                return Err(nuxie_runtime::ScriptError::new(
                    "state-machine scripted data converters used a different renderer Factory domain",
                ));
            }
            let Some(converter) = file.runtime.object(converter_global_id as usize) else {
                continue;
            };
            let target = match script_mount_target(
                &file.runtime,
                &scripts,
                converter,
                ScriptMountTargetKind::DataConverter,
                &format!(
                    "state-machine DataBind {parent_data_bind_index} converter occurrence {converter_path:?}",
                ),
            ) {
                Ok(target) => target,
                Err(error) if error.resource_code().is_some() => return Err(error),
                Err(_) => continue,
            };
            let Some(program) = ready.programs.get(&target.asset_ordinal) else {
                continue;
            };
            let mut host = NoopScriptHost;
            let script = match factory.as_deref_mut() {
                Some(active_factory) => ready
                    .vm
                    .instantiate_registered_script_with_factory_and_context(
                        program,
                        &mut host,
                        active_factory,
                        context_view_model.clone(),
                        context_parent_view_models.clone(),
                    ),
                None => ready.vm.instantiate_registered_script_with_context(
                    program,
                    context_view_model.clone(),
                    context_parent_view_models.clone(),
                ),
            };
            let Some(script) = scripted_listener_action_or_inert(script)? else {
                drop(scripts);
                continue;
            };
            drop(scripts);
            machine.set_scripted_data_converter_instance(
                parent_data_bind_index,
                &converter_path,
                converter_global_id,
                script,
            )?;
        }

        let context = script_listener_context_hydration(
            file,
            machine,
            root_view_model,
            ScriptedListenerDataContextPhase::Live,
        );
        let result = machine.hydrate_and_initialize_scripted_data_converter_instance(
            parent_data_bind_index,
            &converter_path,
            context,
            inits,
            factory
                .as_deref_mut()
                .map(|factory| factory as &mut dyn Factory),
            |machine| {
                prepare_state_machine_data_converter_hydration(
                    file,
                    machine,
                    parent_data_bind_index,
                    &converter_path,
                    root_view_model,
                )
            },
        );
        let _ = scripted_listener_action_or_inert(result)?;
    }
    Ok(())
}

#[cfg(all(feature = "scripting", test))]
fn instantiate_script_listener_actions(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    factory: &mut dyn Factory,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    instantiate_script_listener_actions_with_optional_factory(
        file,
        machine,
        Some(factory),
        root_view_model,
    )?;
    // This test-only entry point represents the complete synchronous C++
    // constructor pass. A retained table may still have user `init` pending
    // because a live hydration prerequisite is absent, but low-level
    // callbacks may use that valid `m_self` occurrence
    // (`scripted_object.cpp:399-437`;
    // `state_machine_instance.cpp:2072-2082`).
    machine.mark_scripted_object_initialization_complete(
        root_view_model.map(ViewModelInstance::handle),
    );
    Ok(())
}

#[cfg(feature = "scripting")]
fn instantiate_script_listener_actions_with_optional_factory(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    factory: Option<&mut dyn Factory>,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let definitions = machine.scripted_objects().to_vec();
    let mut factory = factory;

    // Clone/reinit every ScriptedListenerAction before the live DataContext is
    // attached. C++ completes this cold pass for the full authored collection
    // during StateMachineInstance construction. Missing, malformed, and
    // unauthenticated ScriptAssets leave the occurrence inert; they do not
    // remove its cloned ScriptInput/DataBind ownership.
    for definition in &definitions {
        // C++ cloneScriptedObject::reinit performs one cold generator /
        // authored-input / user-init attempt before the live DataContext is
        // assigned (`scripted_listener_action.cpp:154-160`).
        if instantiate_state_machine_scripted_object_table(
            file,
            machine,
            definition,
            factory
                .as_deref_mut()
                .map(|factory| factory as &mut dyn Factory),
            None,
            ScriptedListenerDataContextPhase::Cold,
        )? {
            let context = script_listener_context_hydration(
                file,
                machine,
                None,
                ScriptedListenerDataContextPhase::Cold,
            );
            let result = machine.hydrate_and_initialize_scripted_object_instance(
                definition.scripted_object_global_id(),
                context,
                definition.inits(),
                factory
                    .as_deref_mut()
                    .map(|factory| factory as &mut dyn Factory),
                |machine| {
                    prepare_script_listener_hydration(
                        file,
                        machine,
                        definition,
                        None,
                        None,
                        false,
                        false,
                        ScriptedListenerDataContextPhase::Cold,
                    )
                },
            );
            let _ = scripted_listener_action_or_inert(result)?;
        }
    }

    // `internalDataContext` binds the ordinary StateMachine container before
    // assigning the live context to the already-cloned ScriptedObjects. Core
    // DataBind/converter binding is not scripting-authority dependent; only
    // the Lua table generator above/below is allowed to remain inert.
    instantiate_state_machine_data_converters(
        file,
        machine,
        factory
            .as_deref_mut()
            .map(|factory| factory as &mut dyn Factory),
        root_view_model,
        false,
    )?;

    // `internalDataContext` walks outer DataBinds and converter occurrences in
    // authored order. A ScriptedDataConverter reinitializes before the next
    // occurrence binds, so this must remain one interleaved operation stream
    // rather than an eager recursive bind followed by a facade hydration pass.
    instantiate_script_listener_data_converters(
        file,
        machine,
        factory
            .as_deref_mut()
            .map(|factory| factory as &mut dyn Factory),
        root_view_model,
        false,
    )?;
    machine.finish_scripted_object_data_context_bind();

    let mut live_factory = factory;
    install_live_scripted_object_contexts(
        file,
        machine,
        &definitions,
        root_view_model,
        &mut live_factory,
    )?;

    for definition in definitions {
        // StateMachineInstance then hydrates every action from its cloned Core
        // target values. A fail-once generator/init can settle before
        // construction returns, while a genuinely unresolved prerequisite
        // retains the table for a later live-context retry.
        if !materialize_missing_scripted_object_after_context_barrier(
            file,
            machine,
            &definition,
            root_view_model,
            &mut live_factory,
        )? {
            continue;
        }
        let result = machine
            .hydrate_and_initialize_scripted_object_instance_after_context_install(
                definition.scripted_object_global_id(),
                definition.inits(),
                live_factory
                    .as_mut()
                    .map(|factory| &mut **factory as &mut dyn Factory),
                |machine| {
                    prepare_script_listener_hydration(
                        file,
                        machine,
                        &definition,
                        root_view_model,
                        None,
                        false,
                        false,
                        ScriptedListenerDataContextPhase::Live,
                    )
                },
            )
            .map_err(|error| {
                error.with_context(format!(
                    "state-machine ScriptedObject global {} asset ordinal {} name '{}' hydration/init failed",
                    definition.scripted_object_global_id(),
                    definition.asset_ordinal(),
                    definition.asset_name()
                ))
            });
        let _ = scripted_listener_action_or_inert(result)?;
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn install_live_scripted_object_contexts(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    definitions: &[nuxie_runtime::ScriptListenerActionDefinition],
    root_view_model: Option<&ViewModelInstance>,
    _factory: &mut Option<&mut dyn Factory>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    // C++ assigns `m_dataContext` to the complete cloned ScriptedObject map
    // before `initScriptedObjects` enters its first occurrence
    // (`state_machine_instance.cpp:2901-2913`). A missing table's generator is
    // already init-phase work, so it must not run until every retained table
    // has crossed this barrier.
    for definition in definitions {
        if !machine.has_scripted_object_instance(definition.scripted_object_global_id()) {
            continue;
        }
        let context = script_listener_context_hydration(
            file,
            machine,
            root_view_model,
            ScriptedListenerDataContextPhase::Live,
        );
        machine.install_scripted_object_data_context(
            definition.scripted_object_global_id(),
            &context,
        )?;
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn materialize_missing_scripted_object_after_context_barrier(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    definition: &nuxie_runtime::ScriptListenerActionDefinition,
    root_view_model: Option<&ViewModelInstance>,
    factory: &mut Option<&mut dyn Factory>,
) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
    if machine.has_scripted_object_instance(definition.scripted_object_global_id()) {
        return Ok(true);
    }
    if !instantiate_state_machine_scripted_object_table(
        file,
        machine,
        definition,
        factory
            .as_mut()
            .map(|factory| &mut **factory as &mut dyn Factory),
        root_view_model,
        ScriptedListenerDataContextPhase::Live,
    )? {
        return Ok(false);
    }
    let context = script_listener_context_hydration(
        file,
        machine,
        root_view_model,
        ScriptedListenerDataContextPhase::Live,
    );
    machine
        .install_scripted_object_data_context(definition.scripted_object_global_id(), &context)?;
    Ok(true)
}

#[cfg(feature = "scripting")]
fn initialize_state_machine_scripted_objects_impl(
    file: &Arc<File>,
    artboard: &RuntimeArtboardInstance,
    machine: &mut StateMachineInstance,
    factory: Option<&mut dyn Factory>,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let mut factory = factory;
    let waits_for_ready_file_vm =
        state_machine_script_lifecycle_waits_for_ready_file_vm(file, machine);
    if !machine.scripted_object_initialization_complete() && waits_for_ready_file_vm {
        // C++ performs this constructor lifecycle once with its scripting
        // context already available. An optional-Factory Rust call cannot
        // run a partial bind/finish pass and then replay the constructor when
        // the File VM appears, so leave the complete fixed occurrence
        // untouched and fail closed until the first Factory boundary.
        return Ok(());
    }
    let staged_context_bind = match root_view_model {
        Some(root) => machine.begin_scripted_object_data_context_bind(root.handle()),
        None => machine.begin_retained_scripted_object_data_context_rebind(),
    };
    if machine.scripted_object_initialization_complete() {
        if staged_context_bind {
            // C++ `internalDataContext` updates every retained ScriptedObject
            // and calls `initScriptedObjects` before the next advance. The
            // ordinary facade's repeated A->A argument is not a lifecycle
            // boundary, but A->B is and must complete in this same call
            // (`state_machine_instance.cpp:2880-2913`;
            // `scripted_data_converter.cpp:170-188`).
            rehydrate_script_listener_actions(file, machine, root_view_model, None, &mut factory)?;
        }
        return Ok(());
    }

    // Pinned C++ mounts ScriptedDrawables before the constructor scans them
    // for listener-less input groups, then clones/initializes the complete
    // state-machine ScriptedObject collection once
    // (`state_machine_instance.cpp:1969-2013,2072-2082,2141-2199`).
    machine.synchronize_scripted_input_groups(artboard);
    machine.set_scripted_listener_artboard_resolver(Box::new(FileScriptArtboardResolver {
        file: Arc::clone(file),
    }));
    instantiate_script_listener_actions_with_optional_factory(
        file,
        machine,
        factory,
        root_view_model,
    )?;
    machine.mark_scripted_object_initialization_complete(
        root_view_model.map(ViewModelInstance::handle),
    );
    Ok(())
}

#[cfg(feature = "scripting")]
fn state_machine_script_lifecycle_waits_for_ready_file_vm(
    file: &Arc<File>,
    machine: &StateMachineInstance,
) -> bool {
    let scripts = file.scripts.borrow();
    if scripts.ready.is_some() || !scripts.scripts_are_authenticated() {
        return false;
    }
    let executable_protocol_asset = |ordinal: usize| {
        scripts.assets.get(ordinal).is_some_and(|asset| {
            asset.type_name == "ScriptAsset"
                && !asset.is_module
                && !asset.is_project_data_converter
                && asset.payload.is_some()
        })
    };

    // A valid fixed ScriptedObject cannot be declared initialized merely
    // because Rust's optional-Factory facade has not built the File VM yet.
    // C++ constructs the VM-backed occurrence synchronously as part of the
    // StateMachineInstance lifecycle; Rust keeps that lifecycle pending until
    // the first Factory-bearing call can perform the same work.
    if machine.scripted_objects().iter().any(|definition| {
        definition.has_protocol_asset() && executable_protocol_asset(definition.asset_ordinal())
    }) {
        return true;
    }

    let mut converter_global_ids = machine
        .scripted_data_converter_occurrence_snapshots()
        .into_iter()
        .map(|occurrence| occurrence.converter_global_id)
        .collect::<BTreeSet<_>>();
    converter_global_ids.extend(
        machine
            .scripted_listener_data_converter_occurrences()
            .into_iter()
            .map(
                |(
                    _action_global_id,
                    _input_global_id,
                    _converter_path,
                    converter_global_id,
                    _inits,
                    _attached,
                )| converter_global_id,
            ),
    );
    converter_global_ids.into_iter().any(|global_id| {
        let Some(converter) = file.runtime.object(global_id as usize) else {
            return false;
        };
        let Ok(target) = script_mount_target(
            &file.runtime,
            &scripts,
            converter,
            ScriptMountTargetKind::DataConverter,
            "state-machine fixed ScriptedDataConverter",
        ) else {
            return false;
        };
        executable_protocol_asset(target.asset_ordinal)
    })
}

#[cfg(feature = "scripting")]
fn initialize_state_machine_scripted_objects(
    file: &Arc<File>,
    artboard: &RuntimeArtboardInstance,
    machine: &mut StateMachineInstance,
    factory: &mut dyn Factory,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    if let Some(error) = machine.script_error() {
        return Err(error.clone());
    }
    let result = initialize_state_machine_scripted_objects_impl(
        file,
        artboard,
        machine,
        Some(factory),
        root_view_model,
    );
    if let Err(error) = result.as_ref() {
        machine.retain_scripted_object_data_context_error(error.clone());
    }
    result
}

#[cfg(feature = "scripting")]
fn instantiate_script_listener_data_converters(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    factory: Option<&mut dyn Factory>,
    root_view_model: Option<&ViewModelInstance>,
    explicit_rebind: bool,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let steps = machine.scripted_listener_data_converter_bind_steps();
    let (generator_context_view_model, generator_context_parent_view_models) = machine
        .scripted_listener_data_context_view_models(
            file.runtime(),
            root_view_model.map(ViewModelInstance::handle),
        );
    let mut factory = factory;
    for step in steps {
        let (
            action_global_id,
            input_global_id,
            converter_path,
            converter_global_id,
            inits,
        ) = match step {
            nuxie_runtime::RuntimeScriptedListenerDataConverterBindStep::BindListenerInput {
                action_global_id,
                listener_input_global_id,
            } => {
                let raw_root_view_model = root_view_model.map(ViewModelInstance::raw);
                let result = machine.bind_scripted_listener_input_source(
                    file.runtime(),
                    raw_root_view_model.as_deref(),
                    action_global_id,
                    listener_input_global_id,
                    explicit_rebind,
                );
                drop(raw_root_view_model);
                let _ = result;
                continue;
            }
            nuxie_runtime::RuntimeScriptedListenerDataConverterBindStep::BindConverter {
                action_global_id,
                listener_input_global_id,
                converter_path,
            } => {
                let raw_root_view_model = root_view_model.map(ViewModelInstance::raw);
                let result = machine.bind_scripted_listener_converter_own_sources(
                    file.runtime(),
                    raw_root_view_model.as_deref(),
                    action_global_id,
                    listener_input_global_id,
                    &converter_path,
                    explicit_rebind,
                );
                drop(raw_root_view_model);
                let _ = result;
                continue;
            }
            nuxie_runtime::RuntimeScriptedListenerDataConverterBindStep::Rehydrate {
                action_global_id,
                listener_input_global_id,
                converter_path,
                converter_global_id,
                inits,
            } => (
                action_global_id,
                listener_input_global_id,
                converter_path,
                converter_global_id,
                inits,
            ),
            nuxie_runtime::RuntimeScriptedListenerDataConverterBindStep::RebindFinalInput {
                action_global_id,
                listener_input_global_id,
                converter_path,
                converter_input_index,
                data_bind_index,
            } => {
                let raw_root_view_model = root_view_model.map(ViewModelInstance::raw);
                let result = machine.rebind_scripted_listener_data_converter_final_input(
                    file.runtime(),
                    raw_root_view_model.as_deref(),
                    action_global_id,
                    listener_input_global_id,
                    &converter_path,
                    converter_input_index,
                    data_bind_index,
                );
                drop(raw_root_view_model);
                let _ = result;
                continue;
            }
            nuxie_runtime::RuntimeScriptedListenerDataConverterBindStep::FinalizeListenerInput {
                action_global_id,
                listener_input_global_id,
            } => {
                let _ = machine.finalize_scripted_listener_input_sources(
                    action_global_id,
                    listener_input_global_id,
                );
                continue;
            }
        };

        if !machine.has_scripted_listener_data_converter_instance(
            action_global_id,
            input_global_id,
            &converter_path,
        ) {
            let scripts = file.scripts.borrow();
            let Some(ready) = scripts.ready.as_ref() else {
                continue;
            };
            if let Some(active_factory) = factory.as_deref_mut()
                && ready.factory_domain != render_factory_domain(active_factory)
            {
                return Err(nuxie_runtime::ScriptError::new(
                    "scripted listener data converters used a different renderer Factory domain",
                ));
            }
            let Some(converter) = file.runtime.object(converter_global_id as usize) else {
                continue;
            };
            let target = match script_mount_target(
                &file.runtime,
                &scripts,
                converter,
                ScriptMountTargetKind::DataConverter,
                &format!(
                    "ScriptedListenerAction global {action_global_id} input global {input_global_id}",
                ),
            ) {
                Ok(target) => target,
                Err(error) if error.resource_code().is_some() => return Err(error),
                Err(_) => continue,
            };
            let Some(program) = ready.programs.get(&target.asset_ordinal) else {
                continue;
            };
            let mut host = NoopScriptHost;
            let script = match factory.as_deref_mut() {
                Some(active_factory) => ready
                    .vm
                    .instantiate_registered_script_with_factory_and_context(
                        program,
                        &mut host,
                        active_factory,
                        generator_context_view_model.clone(),
                        generator_context_parent_view_models.clone(),
                    ),
                None => ready.vm.instantiate_registered_script_with_context(
                    program,
                    generator_context_view_model.clone(),
                    generator_context_parent_view_models.clone(),
                ),
            };
            let Some(script) = scripted_listener_action_or_inert(script)? else {
                continue;
            };
            drop(scripts);
            // `ensureScriptInitialized` owns the generated table before
            // hydration validation. Keep that table attached if a live
            // prerequisite is temporarily absent so the next DataContext
            // bind retries the same occurrence.
            machine.set_scripted_listener_data_converter_instance(
                action_global_id,
                input_global_id,
                &converter_path,
                converter_global_id,
                script,
            )?;
        }

        let context = nuxie_runtime::ScriptListenerActionHydration::new_with_context_chain(
            generator_context_view_model.clone(),
            generator_context_parent_view_models.clone(),
            Vec::new(),
        );
        let result = machine.hydrate_and_initialize_scripted_listener_data_converter_instance(
            action_global_id,
            input_global_id,
            &converter_path,
            context,
            inits,
            factory
                .as_deref_mut()
                .map(|factory| factory as &mut dyn Factory),
            |machine| {
                prepare_script_listener_data_converter_hydration(
                    file,
                    machine,
                    action_global_id,
                    input_global_id,
                    &converter_path,
                    root_view_model,
                )
            },
        );
        let _ = scripted_listener_action_or_inert(result)?;
    }
    Ok(())
}

#[cfg(feature = "scripting")]
enum PreparedScriptListenerInput {
    Value {
        name: ScriptCoreString,
        value: ScriptValue,
    },
    Artboard {
        name: ScriptCoreString,
        artboard_id: usize,
    },
    ViewModel {
        input_global_id: u32,
        name: ScriptCoreString,
        path: nuxie_runtime::ScriptInputViewModelPropertyPath,
    },
}

#[cfg(feature = "scripting")]
fn script_listener_context_hydration(
    file: &File,
    machine: &StateMachineInstance,
    root_view_model: Option<&ViewModelInstance>,
    phase: ScriptedListenerDataContextPhase,
) -> nuxie_runtime::ScriptListenerActionHydration {
    if phase == ScriptedListenerDataContextPhase::Cold {
        return nuxie_runtime::ScriptListenerActionHydration::unresolved(Vec::new());
    }
    let (context_view_model, context_parent_view_models) = machine
        .scripted_listener_data_context_view_models(
            file.runtime(),
            root_view_model.map(ViewModelInstance::handle),
        );
    if machine.has_scripted_listener_data_context() || root_view_model.is_some() {
        nuxie_runtime::ScriptListenerActionHydration::new_with_context_chain(
            context_view_model,
            context_parent_view_models,
            Vec::new(),
        )
    } else {
        nuxie_runtime::ScriptListenerActionHydration::unresolved(Vec::new())
    }
}

#[cfg(feature = "scripting")]
fn prepare_script_listener_data_converter_hydration(
    file: &Arc<File>,
    machine: &StateMachineInstance,
    action_global_id: u32,
    input_global_id: u32,
    converter_path: &[usize],
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<nuxie_runtime::ScriptListenerActionHydration, nuxie_runtime::ScriptError> {
    let snapshots = machine
        .scripted_listener_data_converter_input_snapshots(
            action_global_id,
            input_global_id,
            converter_path,
        )
        .ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "ScriptedListenerAction global {action_global_id} input global {input_global_id} has no ScriptedDataConverter occurrence {converter_path:?}",
            ))
        })?;
    let owner = format!(
        "ScriptedListenerAction global {action_global_id} input global {input_global_id} ScriptedDataConverter occurrence {converter_path:?}",
    );
    prepare_scripted_data_converter_hydration_from_snapshots(
        file,
        machine,
        snapshots,
        &owner,
        root_view_model,
    )
}

#[cfg(feature = "scripting")]
fn prepare_state_machine_data_converter_hydration(
    file: &Arc<File>,
    machine: &StateMachineInstance,
    parent_data_bind_index: usize,
    converter_path: &[usize],
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<nuxie_runtime::ScriptListenerActionHydration, nuxie_runtime::ScriptError> {
    let snapshots = machine
        .scripted_data_converter_input_snapshots(parent_data_bind_index, converter_path)
        .ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "state-machine DataBind {parent_data_bind_index} has no ScriptedDataConverter occurrence {converter_path:?}",
            ))
        })?;
    let owner = format!(
        "state-machine DataBind {parent_data_bind_index} ScriptedDataConverter occurrence {converter_path:?}",
    );
    prepare_scripted_data_converter_hydration_from_snapshots(
        file,
        machine,
        snapshots,
        &owner,
        root_view_model,
    )
}

#[cfg(feature = "scripting")]
fn prepare_scripted_data_converter_hydration_from_snapshots(
    file: &Arc<File>,
    machine: &StateMachineInstance,
    snapshots: Vec<nuxie_runtime::ScriptListenerInputSnapshot>,
    owner: &str,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<nuxie_runtime::ScriptListenerActionHydration, nuxie_runtime::ScriptError> {
    let runtime = file.runtime();
    let root_context = root_view_model.map(|view_model| {
        nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
            runtime,
            view_model.handle().clone(),
        )
    });
    let (context_view_model, context_parent_view_models) = machine
        .scripted_listener_data_context_view_models(
            runtime,
            root_view_model.map(ViewModelInstance::handle),
        );
    let artboard_parent_context =
        machine.scripted_listener_artboard_parent_context(root_context.as_ref());
    // C++ validates every ScriptInput before hydrating any of them. In
    // particular, constructing an earlier ScriptInputArtboard must not leave
    // a child occurrence behind when a later input is malformed
    // (`scripted_object.cpp:399-426`).
    let mut prepared_inputs = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        let input = runtime
            .object(snapshot.input_global_id as usize)
            .ok_or_else(|| {
                scripted_data_converter_hydration_error(
                    owner,
                    snapshot.input_global_id,
                    "object is absent",
                )
            })?;
        let expected_type = match snapshot.kind {
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
            return Err(scripted_data_converter_hydration_error(
                owner,
                input.id,
                &format!("expected {expected_type}, found {}", input.type_name),
            ));
        }
        let name = snapshot.name;
        let view_model_path = snapshot.view_model_path;
        match snapshot.kind {
            nuxie_runtime::ScriptListenerInputKind::Boolean
            | nuxie_runtime::ScriptListenerInputKind::Number
            | nuxie_runtime::ScriptListenerInputKind::Color
            | nuxie_runtime::ScriptListenerInputKind::String => {
                let Some(nuxie_runtime::ScriptListenerInputSnapshotValue::Value(value)) =
                    snapshot.value
                else {
                    return Err(scripted_data_converter_hydration_error(
                        owner,
                        input.id,
                        "cloned scalar value is unavailable",
                    ));
                };
                prepared_inputs.push(PreparedScriptListenerInput::Value { name, value });
            }
            // Base hydration never invokes an authored trigger callback.
            nuxie_runtime::ScriptListenerInputKind::Trigger => {}
            nuxie_runtime::ScriptListenerInputKind::Artboard => {
                let artboard_id = match snapshot.value {
                    Some(nuxie_runtime::ScriptListenerInputSnapshotValue::Artboard(value)) => {
                        Some(value)
                    }
                    _ => None,
                }
                .filter(|id| *id != u64::from(u32::MAX))
                .and_then(|id| usize::try_from(id).ok())
                .ok_or_else(|| {
                    scripted_data_converter_hydration_error(
                        owner,
                        input.id,
                        "referenced artboard is unresolved",
                    )
                })?;
                if file.artboard(artboard_id).is_none() {
                    return Err(scripted_data_converter_hydration_error(
                        owner,
                        input.id,
                        &format!(
                            "referenced artboard {artboard_id} is unavailable: missing scripted artboard index {artboard_id}"
                        ),
                    ));
                }
                prepared_inputs.push(PreparedScriptListenerInput::Artboard { name, artboard_id });
            }
            nuxie_runtime::ScriptListenerInputKind::ViewModelProperty => {
                let path = view_model_path.ok_or_else(|| {
                    scripted_data_converter_hydration_error(
                        owner,
                        input.id,
                        "cloned view-model property path is absent",
                    )
                })?;
                machine
                    .scripted_listener_bound_view_model(runtime, &path, root_context.as_ref())
                    .ok_or_else(|| {
                        scripted_data_converter_hydration_error(
                            owner,
                            input.id,
                            "view-model property path is unresolved",
                        )
                    })?;
                prepared_inputs.push(PreparedScriptListenerInput::ViewModel {
                    input_global_id: input.id,
                    name,
                    path,
                });
            }
        }
    }
    let mut inputs = Vec::with_capacity(prepared_inputs.len());
    let artboard_resolver: Rc<dyn nuxie_runtime::ScriptArtboardResolver> =
        Rc::new(FileScriptArtboardResolver {
            file: Arc::clone(file),
        });
    let view_model_resolver = artboard_parent_context.clone().map(|context| {
        Rc::new(FileScriptViewModelInputResolver {
            file: Arc::clone(file),
            context,
        }) as Rc<dyn nuxie_runtime::ScriptViewModelInputResolver>
    });
    for prepared_input in prepared_inputs {
        match prepared_input {
            PreparedScriptListenerInput::Value { name, value, .. } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::Value { name, value });
            }
            PreparedScriptListenerInput::Artboard { name, artboard_id } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::Artboard {
                    name,
                    artboard_id: u64::try_from(artboard_id)
                        .expect("validated ScriptInputArtboard id originated as u64"),
                    resolver: Rc::clone(&artboard_resolver),
                    parent_context: artboard_parent_context.clone(),
                });
            }
            PreparedScriptListenerInput::ViewModel {
                input_global_id,
                name,
                path,
            } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::ViewModel {
                    name,
                    input_global_id,
                    path,
                    resolver: Rc::clone(
                        view_model_resolver
                            .as_ref()
                            .expect("validated ViewModel input retains its DataContext"),
                    ),
                });
            }
        }
    }
    Ok(
        nuxie_runtime::ScriptListenerActionHydration::new_with_context_chain(
            context_view_model,
            context_parent_view_models,
            inputs,
        ),
    )
}

#[cfg(feature = "scripting")]
fn scripted_data_converter_hydration_error(
    owner: &str,
    converter_input_global_id: u32,
    detail: &str,
) -> nuxie_runtime::ScriptError {
    nuxie_runtime::ScriptError::new(format!(
        "{owner} input global {converter_input_global_id}: {detail}",
    ))
}

#[cfg(feature = "scripting")]
fn instantiate_state_machine_scripted_object_table(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    definition: &nuxie_runtime::ScriptListenerActionDefinition,
    factory: Option<&mut dyn Factory>,
    root_view_model: Option<&ViewModelInstance>,
    phase: ScriptedListenerDataContextPhase,
) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
    if machine.has_scripted_object_instance(definition.scripted_object_global_id()) {
        return Ok(true);
    }
    let scripts = file.scripts.borrow();
    let Some(ready) = scripts.ready.as_ref() else {
        return Ok(false);
    };
    let mut factory = factory;
    if let Some(active_factory) = factory.as_deref_mut()
        && ready.factory_domain != render_factory_domain(active_factory)
    {
        return Err(nuxie_runtime::ScriptError::new(
            "state-machine scripted objects used a different renderer Factory domain",
        ));
    }
    let ordinal = definition.asset_ordinal();
    let Some(asset) = scripts.assets.get(ordinal) else {
        return Ok(false);
    };
    // Pinned C++ retains the resolved ScriptAsset pointer on ScriptedObject;
    // the serialized name is not part of occurrence identity
    // (`scripted_object.cpp:548-555`; `backboard_importer.cpp:84-101`).
    // Rust's `ordinal` is that file-asset identity. `asset.name` may include
    // `folderPath`, while the generated ScriptedObject definition retains the
    // asset's bare name, so comparing either spelling would incorrectly make
    // a valid foldered protocol occurrence inert.
    if asset.type_name != "ScriptAsset" || asset.is_module {
        return Ok(false);
    }
    let Some(program) = ready.programs.get(&ordinal) else {
        return Ok(false);
    };
    let mut host = NoopScriptHost;
    let (context_view_model, context_parent_view_models) = match phase {
        ScriptedListenerDataContextPhase::Cold => (None, Vec::new()),
        ScriptedListenerDataContextPhase::Live => machine
            .scripted_listener_data_context_view_models(
                file.runtime(),
                root_view_model.map(ViewModelInstance::handle),
            ),
    };
    let instance = match factory {
        Some(active_factory) => ready
            .vm
            .instantiate_registered_script_with_factory_and_context(
                program,
                &mut host,
                active_factory,
                context_view_model,
                context_parent_view_models,
            ),
        None => ready.vm.instantiate_registered_script_with_context(
            program,
            context_view_model,
            context_parent_view_models,
        ),
    }
    .map_err(|error| {
        error.with_context(format!(
            "state-machine ScriptedObject global {} asset ordinal {ordinal} name '{}' generator failed",
            definition.scripted_object_global_id(),
            definition.asset_name()
        ))
    });
    let instance = scripted_listener_action_or_inert(instance)?;
    drop(scripts);
    let Some(instance) = instance else {
        return Ok(false);
    };
    machine.set_scripted_object_instance(definition.scripted_object_global_id(), instance)?;
    Ok(true)
}

#[cfg(feature = "scripting")]
fn rehydrate_script_listener_actions(
    file: &Arc<File>,
    machine: &mut StateMachineInstance,
    root_view_model: Option<&ViewModelInstance>,
    previous_root_view_model: Option<&ViewModelInstance>,
    factory: &mut Option<&mut dyn Factory>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    let definitions = machine.scripted_objects().to_vec();

    // Rebinding the live DataContext is a core runtime lifecycle boundary,
    // even when no ScriptAsset can execute. Preserve the same ordinary-then-
    // cloned-ScriptedObject DataBind order as pinned `internalDataContext`.
    instantiate_state_machine_data_converters(
        file,
        machine,
        factory
            .as_mut()
            .map(|factory| &mut **factory as &mut dyn Factory),
        root_view_model,
        true,
    )?;
    instantiate_script_listener_data_converters(
        file,
        machine,
        factory
            .as_mut()
            .map(|factory| &mut **factory as &mut dyn Factory),
        root_view_model,
        true,
    )?;
    machine.finish_scripted_object_data_context_bind();

    install_live_scripted_object_contexts(file, machine, &definitions, root_view_model, factory)?;
    for definition in definitions {
        if !materialize_missing_scripted_object_after_context_barrier(
            file,
            machine,
            &definition,
            root_view_model,
            factory,
        )? {
            continue;
        }
        let result = machine
            .hydrate_and_initialize_scripted_object_instance_after_context_install(
                definition.scripted_object_global_id(),
                definition.inits(),
                factory
                    .as_mut()
                    .map(|factory| &mut **factory as &mut dyn Factory),
                |machine| {
                    prepare_script_listener_hydration(
                        file,
                        machine,
                        &definition,
                        root_view_model,
                        previous_root_view_model,
                        true,
                        false,
                        ScriptedListenerDataContextPhase::Live,
                    )
                },
            )
            .map_err(|error| {
                error.with_context(format!(
                    "state-machine ScriptedObject global {} asset ordinal {} name '{}' data-context rehydration failed",
                    definition.scripted_object_global_id(),
                    definition.asset_ordinal(),
                    definition.asset_name()
                ))
            });
        let _ = scripted_listener_action_or_inert(result)?;
    }
    if let Some(root_view_model) = root_view_model {
        machine.mark_scripted_facade_root_hydrated(Some(root_view_model.handle()));
    }
    Ok(())
}

#[cfg(feature = "scripting")]
fn try_prepare_state_machine_scripted_data_context_without_factory(
    file: &Arc<File>,
    artboard: &RuntimeArtboardInstance,
    machine: &mut StateMachineInstance,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
    if let Some(error) = machine.script_error() {
        return Err(error.clone());
    }
    if !machine.scripted_object_initialization_complete() {
        let result = initialize_state_machine_scripted_objects_impl(
            file,
            artboard,
            machine,
            None,
            root_view_model,
        );
        return match result {
            Ok(()) => Ok(()),
            Err(error) => {
                machine.retain_scripted_object_data_context_error(error.clone());
                Err(error)
            }
        };
    }
    let staged_context_bind = match root_view_model {
        Some(root) => machine.begin_scripted_object_data_context_bind(root.handle()),
        None => machine.begin_retained_scripted_object_data_context_rebind(),
    };
    if !staged_context_bind {
        return Ok(());
    }

    let mut factory = None;
    let result =
        rehydrate_script_listener_actions(file, machine, root_view_model, None, &mut factory);

    match result {
        Ok(()) => Ok(()),
        Err(error) => {
            // This compatibility API cannot return `Result`. Rust's typed
            // resource fence is terminal once the staged bind has entered
            // callbacks, so retain the first failure and never replay it as a
            // fresh C++ lifecycle boundary.
            machine.retain_scripted_object_data_context_error(error.clone());
            Err(error)
        }
    }
}

#[cfg(feature = "scripting")]
fn prepare_script_listener_hydration(
    file: &Arc<File>,
    machine: &StateMachineInstance,
    definition: &nuxie_runtime::ScriptListenerActionDefinition,
    root_view_model: Option<&ViewModelInstance>,
    _previous_root_view_model: Option<&ViewModelInstance>,
    _rebind: bool,
    _resolve_data_binds: bool,
    phase: ScriptedListenerDataContextPhase,
) -> std::result::Result<nuxie_runtime::ScriptListenerActionHydration, nuxie_runtime::ScriptError> {
    let runtime = file.runtime();
    let root_context = match phase {
        ScriptedListenerDataContextPhase::Cold => None,
        ScriptedListenerDataContextPhase::Live => root_view_model.map(|view_model| {
            nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
                runtime,
                view_model.handle().clone(),
            )
        }),
    };
    let (context_view_model, context_parent_view_models) = match phase {
        ScriptedListenerDataContextPhase::Cold => (None, Vec::new()),
        ScriptedListenerDataContextPhase::Live => machine
            .scripted_listener_data_context_view_models(
                runtime,
                root_view_model.map(ViewModelInstance::handle),
            ),
    };
    let context_resolved = phase == ScriptedListenerDataContextPhase::Live
        && (machine.has_scripted_listener_data_context() || root_view_model.is_some());
    let artboard_parent_context = match phase {
        ScriptedListenerDataContextPhase::Cold => None,
        ScriptedListenerDataContextPhase::Live => {
            machine.scripted_listener_artboard_parent_context(root_context.as_ref())
        }
    };

    // C++ gives each ScriptedObject a write-free prerequisite preflight:
    // validate all of this occurrence's properties before constructing any
    // child artboard or touching its script table
    // (`scripted_object.cpp:399-426`). Phase-two writes remain authored-order
    // operations, and a different ScriptedObject is a separate attempt.
    let snapshots = machine
        .scripted_listener_action_input_snapshots(definition.action_global_id())
        .ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "ScriptedListenerAction global {} has no cloned input occurrence",
                definition.action_global_id(),
            ))
        })?;
    let mut prepared_inputs = Vec::with_capacity(definition.inputs().len());
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
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.input_global_id == input.id)
            .ok_or_else(|| {
                listener_input_hydration_error(
                    definition,
                    input.id,
                    "cloned input occurrence is absent",
                )
            })?;
        let name = snapshot.name.clone();
        match input_definition.kind() {
            nuxie_runtime::ScriptListenerInputKind::Boolean
            | nuxie_runtime::ScriptListenerInputKind::Number
            | nuxie_runtime::ScriptListenerInputKind::Color
            | nuxie_runtime::ScriptListenerInputKind::String => {
                let Some(nuxie_runtime::ScriptListenerInputSnapshotValue::Value(value)) =
                    snapshot.value.clone()
                else {
                    return Err(listener_input_hydration_error(
                        definition,
                        input.id,
                        "cloned scalar value is unavailable",
                    ));
                };
                prepared_inputs.push(PreparedScriptListenerInput::Value { name, value });
            }
            // ScriptInputTrigger's base hydration is intentionally a no-op.
            // Only the later retained DataBind update can invoke the authored
            // callback (`script_input_trigger.cpp:49-55`).
            nuxie_runtime::ScriptListenerInputKind::Trigger => {}
            nuxie_runtime::ScriptListenerInputKind::Artboard => {
                let artboard_id = match snapshot.value {
                    Some(nuxie_runtime::ScriptListenerInputSnapshotValue::Artboard(value)) => {
                        Some(value)
                    }
                    _ => None,
                }
                .filter(|id| *id != u64::from(u32::MAX))
                .and_then(|id| usize::try_from(id).ok())
                .ok_or_else(|| {
                    listener_input_hydration_error(
                        definition,
                        input.id,
                        "referenced artboard is unresolved",
                    )
                })?;
                if file.artboard(artboard_id).is_none() {
                    return Err(listener_input_hydration_error(
                        definition,
                        input.id,
                        &format!(
                            "referenced artboard {artboard_id} is unavailable: missing scripted artboard index {artboard_id}"
                        ),
                    ));
                }
                prepared_inputs.push(PreparedScriptListenerInput::Artboard { name, artboard_id });
            }
            nuxie_runtime::ScriptListenerInputKind::ViewModelProperty => {
                if phase == ScriptedListenerDataContextPhase::Cold {
                    // `validateForColdScriptInit()` accepts the definition,
                    // but the later whole-object hydration preflight cannot
                    // resolve a ViewModel ScriptInput before DataContext
                    // attachment (`script_input_viewmodel_property.cpp:46-81`;
                    // `scripted_object.cpp:399-426`).
                    return Err(listener_input_hydration_error(
                        definition,
                        input.id,
                        "view-model property path is unresolved during cold initialization",
                    ));
                }
                let path = snapshot.view_model_path.clone().ok_or_else(|| {
                    listener_input_hydration_error(
                        definition,
                        input.id,
                        "cloned view-model property path is absent",
                    )
                })?;
                machine
                    .scripted_listener_bound_view_model(runtime, &path, root_context.as_ref())
                    .ok_or_else(|| {
                        listener_input_hydration_error(
                            definition,
                            input.id,
                            "view-model property path is unresolved",
                        )
                    })?;
                prepared_inputs.push(PreparedScriptListenerInput::ViewModel {
                    input_global_id: input.id,
                    name,
                    path,
                });
            }
        }
    }

    let artboard_resolver: Rc<dyn nuxie_runtime::ScriptArtboardResolver> =
        Rc::new(FileScriptArtboardResolver {
            file: Arc::clone(file),
        });
    let view_model_resolver = artboard_parent_context.clone().map(|context| {
        Rc::new(FileScriptViewModelInputResolver {
            file: Arc::clone(file),
            context,
        }) as Rc<dyn nuxie_runtime::ScriptViewModelInputResolver>
    });
    let mut inputs = Vec::with_capacity(prepared_inputs.len());
    for prepared_input in prepared_inputs {
        match prepared_input {
            PreparedScriptListenerInput::Value { name, value, .. } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::Value { name, value });
            }
            PreparedScriptListenerInput::Artboard { name, artboard_id } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::Artboard {
                    name,
                    artboard_id: u64::try_from(artboard_id)
                        .expect("validated ScriptInputArtboard id originated as u64"),
                    resolver: Rc::clone(&artboard_resolver),
                    parent_context: artboard_parent_context.clone(),
                });
            }
            PreparedScriptListenerInput::ViewModel {
                input_global_id,
                name,
                path,
            } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::ViewModel {
                    name,
                    input_global_id,
                    path,
                    resolver: Rc::clone(
                        view_model_resolver
                            .as_ref()
                            .expect("validated ViewModel input retains its DataContext"),
                    ),
                });
            }
        }
    }
    Ok(if context_resolved {
        nuxie_runtime::ScriptListenerActionHydration::new_with_context_chain(
            context_view_model,
            context_parent_view_models,
            inputs,
        )
    } else {
        nuxie_runtime::ScriptListenerActionHydration::unresolved(inputs)
    })
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
    parent_context: Option<nuxie_runtime::ScriptArtboardParentContext>,
    // C++ ScriptReffedArtboard constructs and binds this identity once. It is
    // not rebuilt on every ScriptedArtboard::advance
    // (`lua_artboards.cpp:20-50,103-115`).
    _data_context: Option<nuxie_runtime::ScriptArtboardDataContext>,
    width: f32,
    height: f32,
    frame_origin: bool,
}

#[cfg(feature = "scripting")]
struct FileScriptArtboardResolver {
    file: Arc<File>,
}

#[cfg(feature = "scripting")]
struct FileScriptViewModelInputResolver {
    file: Arc<File>,
    context: nuxie_runtime::ScriptArtboardParentContext,
}

#[cfg(feature = "scripting")]
impl std::fmt::Debug for FileScriptArtboardResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileScriptArtboardResolver")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "scripting")]
impl std::fmt::Debug for FileScriptViewModelInputResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileScriptViewModelInputResolver")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "scripting")]
impl nuxie_runtime::ScriptViewModelInputResolver for FileScriptViewModelInputResolver {
    fn resolve_script_view_model(
        &self,
        input_global_id: u32,
        path: &nuxie_runtime::ScriptInputViewModelPropertyPath,
    ) -> std::result::Result<Option<nuxie_runtime::ScriptViewModel>, nuxie_runtime::ScriptError>
    {
        self.context
            .resolve_script_view_model_input(self.file.runtime(), path)
            .ok_or_else(|| {
                nuxie_runtime::ScriptError::new(format!(
                    "ScriptInputViewModelProperty global {input_global_id} became unresolved during authored hydration",
                ))
            })
    }
}

#[cfg(feature = "scripting")]
impl nuxie_runtime::ScriptArtboardResolver for FileScriptArtboardResolver {
    fn resolve_script_artboard(
        &self,
        artboard_id: u64,
        parent_context: Option<&nuxie_runtime::ScriptArtboardParentContext>,
    ) -> std::result::Result<Box<dyn nuxie_runtime::ScriptArtboard>, nuxie_runtime::ScriptError>
    {
        let artboard_index = usize::try_from(artboard_id).map_err(|_| {
            nuxie_runtime::ScriptError::new(format!(
                "script artboard id {artboard_id} does not fit this platform",
            ))
        })?;
        FileScriptArtboard::new(Arc::clone(&self.file), artboard_index, parent_context)
            .map(|artboard| Box::new(artboard) as Box<dyn nuxie_runtime::ScriptArtboard>)
    }
}

#[cfg(feature = "scripting")]
impl FileScriptArtboard {
    fn new(
        file: Arc<File>,
        artboard_index: usize,
        parent_context: Option<&nuxie_runtime::ScriptArtboardParentContext>,
    ) -> std::result::Result<Self, nuxie_runtime::ScriptError> {
        Self::new_with_view_model(file, artboard_index, parent_context, None)
    }

    fn new_with_view_model(
        file: Arc<File>,
        artboard_index: usize,
        parent_context: Option<&nuxie_runtime::ScriptArtboardParentContext>,
        supplied_view_model: Option<nuxie_runtime::ScriptViewModel>,
    ) -> std::result::Result<Self, nuxie_runtime::ScriptError> {
        let graph = file.graph.artboards.get(artboard_index).ok_or_else(|| {
            nuxie_runtime::ScriptError::new(format!(
                "missing scripted artboard index {artboard_index}",
            ))
        })?;
        let external_font_assets = file.external_font_assets.snapshot();
        let mut instance =
            RuntimeArtboardInstance::from_graph_with_artboards_external_fonts_and_file_catalogs(
                &file.runtime,
                graph,
                &file.graph.artboards,
                &external_font_assets,
                file.file_view_model_instances.clone(),
                file.state_machine_actions.clone(),
            )
            .map_err(|error| nuxie_runtime::ScriptError::new(error.to_string()))?;
        instance.set_frame_origin(false);
        let state_machine = file
            .artboard(artboard_index)
            .and_then(|artboard| artboard.default_state_machine_index())
            .and_then(|state_machine_index| instance.state_machine_instance(state_machine_index));
        let view_model = supplied_view_model.or_else(|| {
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
                        .or_else(|| {
                            RuntimeOwnedViewModelInstance::new(&file.runtime, view_model_index)
                        })
                        .map(RuntimeOwnedViewModelHandle::new)
                });
            owned_view_model.as_ref().and_then(|instance| {
                nuxie_runtime::script_view_model_from_owned(&file.runtime, instance)
            })
        });
        let (width, height) = instance.artboard_dimensions();
        let mut scripted = Self {
            file,
            artboard_index,
            instance,
            state_machine,
            view_model,
            parent_context: parent_context.cloned(),
            _data_context: None,
            width,
            height,
            frame_origin: false,
        };
        scripted.bind_view_model_once();
        scripted.prepare_state_machine_once()?;
        Ok(scripted)
    }

    fn bind_view_model_once(&mut self) {
        let Some(state_machine) = self.state_machine.as_mut() else {
            return;
        };
        let Some(view_model) = self.view_model.as_ref() else {
            return;
        };
        let local = view_model.owned_handle();
        let context = self
            .parent_context
            .as_ref()
            .map(|parent| parent.with_local_view_model(&local))
            .unwrap_or_else(|| nuxie_runtime::ScriptArtboardDataContext::root(&local));
        state_machine.bind_script_artboard_data_context(&context);
        self.instance
            .bind_script_artboard_data_context(&self.file.runtime, &context);
        self._data_context = Some(context);
    }

    fn prepare_state_machine_once(
        &mut self,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        let Some(state_machine) = self.state_machine.as_mut() else {
            return Ok(());
        };
        let result = initialize_state_machine_scripted_objects_impl(
            &self.file,
            &self.instance,
            state_machine,
            None,
            None,
        );
        if let Err(error) = result.as_ref() {
            state_machine.retain_scripted_object_data_context_error(error.clone());
        }
        result
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
        self.instance.set_frame_origin(frame_origin);
    }

    fn data(&self) -> Option<nuxie_runtime::ScriptViewModel> {
        self.view_model.clone()
    }

    fn instance(
        &self,
        view_model: Option<nuxie_runtime::ScriptViewModel>,
    ) -> std::result::Result<Box<dyn nuxie_runtime::ScriptArtboard>, nuxie_runtime::ScriptError>
    {
        Self::new_with_view_model(
            Arc::clone(&self.file),
            self.artboard_index,
            self.parent_context.as_ref(),
            view_model,
        )
        .map(|instance| Box::new(instance) as Box<dyn nuxie_runtime::ScriptArtboard>)
    }

    fn advance(&mut self, seconds: f32) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
        if let Some(state_machine) = self.state_machine.as_mut() {
            try_prepare_state_machine_scripted_data_context_without_factory(
                &self.file,
                &self.instance,
                state_machine,
                None,
            )?;
            let state_machines = std::slice::from_mut(state_machine);
            let mut changed = self
                .instance
                .advance_frame_components_with_state_machines(state_machines, seconds)?;
            changed |= self
                .instance
                .settle_state_machine_update_passes_after_main_advance_without_root_view_model_reset_with_script_errors(
                    state_machines,
                )?;
            Ok(advance_and_apply_keep_going(
                changed,
                seconds,
                state_machines,
            ))
        } else {
            let mut changed = self.instance.advance_frame_components(seconds)?;
            changed |= self.instance.update_pass_with_script_errors()?;
            Ok(changed)
        }
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
        self.instance
            .draw_artboard(
                &self.file.runtime,
                graph,
                &self.file.graph.artboards,
                factory,
                renderer,
                &self.file.external_image_assets,
                self.file.max_retained_decoded_image_bytes,
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
        serialized_implemented_methods: asset.serialized_implemented_methods,
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
        if !nuxie_schema::definition_by_name(component.type_name)
            .is_some_and(|definition| definition.is_a("ScriptedDrawable"))
        {
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
    for global_id in instance.scripted_data_converter_global_ids() {
        let Some(converter) = runtime.object(global_id as usize) else {
            continue;
        };
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
        serialized_implemented_methods: u32,
        script: Box<dyn ScriptInstance>,
    ) {
        match kind {
            ScriptMountTargetKind::Drawable => {
                instance.set_script_instance_for_global_with_implemented_methods(
                    global_id,
                    script,
                    serialized_implemented_methods,
                );
            }
            ScriptMountTargetKind::DataConverter => {
                instance.set_scripted_data_converter_instance_for_global(global_id, script);
            }
        }
    }

    let mut groups = VecDeque::from(prepared);
    if let Some(root) = groups.pop_front() {
        for (kind, global_id, serialized_implemented_methods, script) in root.scripts {
            attach(
                instance,
                kind,
                global_id,
                serialized_implemented_methods,
                script,
            );
        }
    }
    let mut visitor = |_: usize, _: u32, nested: &mut RuntimeArtboardInstance| {
        if let Some(group) = groups.pop_front() {
            for (kind, global_id, serialized_implemented_methods, script) in group.scripts {
                attach(
                    nested,
                    kind,
                    global_id,
                    serialized_implemented_methods,
                    script,
                );
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
    // A draw-only bootstrap has no elapsed frame to replay. Walk the same
    // retained advancing list at zero seconds so nested child dirt bubbles to
    // its host, then consume ScriptUpdate at each concrete dependency slot.
    // This replaces the old Artboard-wide script-table sweep with the pinned
    // Component lifecycle (`artboard.cpp:1463-1480`;
    // `scripted_drawable.cpp:347-397`).
    let mut changed = instance.advance_frame_components_with_factory(0.0, factory)?;
    changed |= instance.update_pass_with_factory(factory)?;
    Ok(changed)
}

#[cfg(feature = "scripting")]
fn mount_scripted_artboard_tree(
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
    Ok(has_script_target)
}

#[cfg(feature = "scripting")]
fn verify_scripted_artboard_tree_attached(
    file: &File,
    root_graph: &ArtboardGraph,
    instance: &mut RuntimeArtboardInstance,
) -> std::result::Result<(), nuxie_runtime::ScriptError> {
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
    Ok(())
}

#[cfg(feature = "scripting")]
fn prepare_scripted_artboard_tree(
    file: &File,
    root_graph: &ArtboardGraph,
    instance: &mut RuntimeArtboardInstance,
    factory: &mut dyn Factory,
) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
    let has_script_target = mount_scripted_artboard_tree(file, root_graph, instance, factory)?;
    let changed = if has_script_target {
        flush_scripted_artboard_tree(instance, factory)?
    } else {
        false
    };

    // Script update refreshes component-list occurrences. A newly materialized
    // child is mounted on the next preparation call, but must not slip through
    // this draw without a table in the meantime.
    verify_scripted_artboard_tree_attached(file, root_graph, instance)?;
    Ok(changed)
}

#[cfg(feature = "scripting")]
fn advance_scripted_artboard_frame_with_factory(
    file: &Arc<File>,
    root_graph: &ArtboardGraph,
    instance: &mut RuntimeArtboardInstance,
    state_machines: &mut [StateMachineInstance],
    elapsed_seconds: f32,
    factory: &mut dyn Factory,
    root_view_model: Option<&ViewModelInstance>,
) -> std::result::Result<bool, nuxie_runtime::ScriptError> {
    let _ = mount_scripted_artboard_tree(file, root_graph, instance, factory)?;
    for machine in state_machines.iter_mut() {
        initialize_state_machine_scripted_objects(
            file,
            instance,
            machine,
            factory,
            root_view_model,
        )?;
    }
    let mut changed = if state_machines.is_empty() {
        instance.advance_frame_components_with_factory(elapsed_seconds, factory)?
    } else {
        instance.advance_frame_components_with_state_machines_and_factory(
            state_machines,
            elapsed_seconds,
            factory,
        )?
    };
    changed |= if state_machines.is_empty() {
        instance.update_pass_with_factory(factory)?
    } else {
        instance.settle_state_machine_update_passes_after_main_advance_with_factory(
            state_machines,
            factory,
        )?
    };
    verify_scripted_artboard_tree_attached(file, root_graph, instance)?;
    Ok(changed)
}

/// Imported Rive file plus its runtime graph projection.
pub struct File {
    runtime: Arc<RuntimeFile>,
    file_view_model_instances: RuntimeFileViewModelInstanceCatalog,
    state_machine_actions: RuntimeFileStateMachineActionCatalog,
    graph: Arc<GraphFile>,
    external_image_assets: BTreeMap<u32, Arc<[u8]>>,
    external_font_assets: ExternalFontAssetStore,
    // The decoded-image admission policy chosen at import time. Unlike the
    // other `FileImportLimits` knobs this one applies at draw time, when the
    // artboard-tree render cache decodes retained images.
    max_retained_decoded_image_bytes: Option<usize>,
    #[cfg(feature = "scripting")]
    scripts: Rc<RefCell<FileScriptRuntime>>,
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
    max_retained_decoded_image_bytes: Option<usize>,
}

impl FileImportLimits {
    const DEFAULT_MAX_INPUT_BYTES: usize = 128 * 1024 * 1024;
    const DEFAULT_MAX_RUNTIME_OBJECTS: usize = 1_000_000;
    const DEFAULT_MAX_RUNTIME_PROPERTIES: usize = 1_000_000;
    const DEFAULT_MAX_IMPORTED_FILE_ASSETS: usize = 16_384;
    const DEFAULT_MAX_FILE_ASSET_CONTENT_BYTES: usize = 64 * 1024 * 1024;
    const DEFAULT_MAX_TOTAL_FILE_ASSET_CONTENT_BYTES: usize = 128 * 1024 * 1024;
    const DEFAULT_MAX_RETAINED_DECODED_IMAGE_BYTES: usize = 64 * 1024 * 1024;

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
            max_retained_decoded_image_bytes: Some(Self::DEFAULT_MAX_RETAINED_DECODED_IMAGE_BYTES),
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
            max_retained_decoded_image_bytes: None,
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

    /// Bound the aggregate decoded RGBA bytes retained by one artboard-tree
    /// render cache. Images past the budget are not decoded and drawing
    /// reports an image decode error.
    ///
    /// Pinned C++ has no aggregate decoded-image ceiling; the bounded default
    /// here is a deliberate resource-policy divergence of the high-level host
    /// import path (register D-row). [`Self::unbounded`] restores the C++
    /// behavior.
    pub const fn with_max_retained_decoded_image_bytes(mut self, maximum: usize) -> Self {
        self.max_retained_decoded_image_bytes = Some(maximum);
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

    pub const fn max_retained_decoded_image_bytes(self) -> Option<usize> {
        self.max_retained_decoded_image_bytes
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
            Rc::new(RefCell::new(FileScriptRuntime::new(
                Arc::clone(&scripts.assets),
                Arc::clone(&scripts.imports),
                scripts.authorization,
                scripts.execution_limits,
            )))
        };
        Self {
            runtime,
            file_view_model_instances: self.file_view_model_instances.clone(),
            state_machine_actions: self.state_machine_actions.clone(),
            graph,
            external_image_assets: self.external_image_assets.clone(),
            external_font_assets: self.external_font_assets.clone(),
            max_retained_decoded_image_bytes: self.max_retained_decoded_image_bytes,
            #[cfg(feature = "scripting")]
            scripts,
        }
    }
}

impl File {
    /// Retain this File's exact live script VM while giving occurrence-owned
    /// ScriptInput resolvers an owning handle.
    ///
    /// Public `File::clone` deliberately starts a fresh VM. This private
    /// handle is different: C++ child artboards and ViewModel projections
    /// stay inside the same File/ScriptingContext as their owning
    /// ScriptedObject (`lua_artboards.cpp:20-50`;
    /// `rive_lua_libs.hpp:1530-1543`).
    #[cfg(feature = "scripting")]
    fn shared_script_runtime_handle(&self) -> Arc<Self> {
        Arc::new(Self {
            runtime: Arc::clone(&self.runtime),
            file_view_model_instances: self.file_view_model_instances.clone(),
            state_machine_actions: self.state_machine_actions.clone(),
            graph: Arc::clone(&self.graph),
            external_image_assets: self.external_image_assets.clone(),
            external_font_assets: self.external_font_assets.clone(),
            max_retained_decoded_image_bytes: self.max_retained_decoded_image_bytes,
            scripts: Rc::clone(&self.scripts),
        })
    }

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
        let file_view_model_instances = RuntimeFileViewModelInstanceCatalog::new(&runtime);
        let state_machine_actions = RuntimeFileStateMachineActionCatalog::new(&runtime);
        Ok(Self {
            #[cfg(feature = "scripting")]
            scripts: Rc::new(RefCell::new(FileScriptRuntime::import(
                &runtime,
                authorization,
                execution_limits,
            ))),
            runtime: Arc::new(runtime),
            file_view_model_instances,
            state_machine_actions,
            graph: Arc::new(graph),
            external_image_assets: BTreeMap::new(),
            external_font_assets: ExternalFontAssetStore::default(),
            max_retained_decoded_image_bytes: limits.max_retained_decoded_image_bytes(),
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

    /// Name of the linear animation at `index`, when it has one.
    pub fn animation_name(self, index: usize) -> Option<&'a str> {
        self.graph().animations.get(index)?.name.as_deref()
    }

    /// First authored linear-animation index with an exact matching name.
    /// Mirrors C++ `Artboard::animation(const std::string&)`.
    pub fn animation_index_named(self, name: &str) -> Option<usize> {
        (0..self.animation_count()).find(|index| self.animation_name(*index) == Some(name))
    }

    pub fn state_machine_count(self) -> usize {
        self.graph().state_machines.len()
    }

    /// Name of the state machine at `index`, when it has one.
    pub fn state_machine_name(self, index: usize) -> Option<&'a str> {
        self.graph().state_machines.get(index)?.name.as_deref()
    }

    /// First authored state-machine index with an exact matching name.
    /// Mirrors C++ `Artboard::stateMachine(const std::string&)`.
    pub fn state_machine_index_named(self, name: &str) -> Option<usize> {
        (0..self.state_machine_count()).find(|index| self.state_machine_name(*index) == Some(name))
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
        let raw =
            RuntimeArtboardInstance::from_graph_with_artboards_external_fonts_and_file_catalogs(
                &self.file.runtime,
                self.graph(),
                &self.file.graph.artboards,
                &external_font_assets,
                self.file.file_view_model_instances.clone(),
                self.file.state_machine_actions.clone(),
            )
            .with_context(|| {
                format!(
                    "failed to instantiate artboard {}",
                    self.name().unwrap_or("<unnamed>")
                )
            })?;
        Ok(ArtboardInstance {
            file: self.file,
            #[cfg(feature = "scripting")]
            script_file: self.file.shared_script_runtime_handle(),
            artboard_index: self.index,
            raw,
        })
    }
}

/// User-facing artboard instance that keeps file and graph context available.
#[derive(Debug)]
pub struct ArtboardInstance<'a> {
    file: &'a File,
    #[cfg(feature = "scripting")]
    script_file: Arc<File>,
    artboard_index: usize,
    raw: RuntimeArtboardInstance,
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
        let mut changed = self
            .raw
            .advance_frame_components(elapsed_seconds)
            .unwrap_or(false);
        changed |= self.raw.update_pass_with_script_errors().unwrap_or(false);
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        changed
    }

    /// Advance with a renderer factory available to every script lifecycle
    /// phase. [`Self::advance`] advances only lifecycle work that does not
    /// require a renderer factory; this method also advances retained scripted
    /// drawables before the regular update pass and surfaces script errors
    /// immediately. Once this File's scripts bootstrap, every factory-bearing
    /// advance and draw must use that same live Factory object; a different
    /// object is rejected before script execution.
    pub fn try_advance_with_factory(
        &mut self,
        factory: &mut dyn Factory,
        elapsed_seconds: f32,
    ) -> Result<bool> {
        #[cfg(feature = "scripting")]
        {
            let mut changed = advance_scripted_artboard_frame_with_factory(
                &self.script_file,
                self.artboard().graph(),
                &mut self.raw,
                &mut [],
                elapsed_seconds,
                factory,
                None,
            )
            .context("failed to advance scripted drawables")?;
            changed |= self.file.advance_detached_view_models();
            return Ok(changed);
        }
        #[cfg(not(feature = "scripting"))]
        {
            let _ = factory;
            let mut changed = self
                .raw
                .advance_frame_components(elapsed_seconds)
                .context("failed to advance retained artboard components")?;
            changed |= self.raw.update_pass();
            Ok(changed)
        }
    }

    /// Return visible Shape and Text locals under `point`, front to back,
    /// including descendants reached through nested artboards and
    /// component-list items.
    pub fn hit_test(&mut self, point: Vec2D) -> Vec<usize> {
        self.raw.hit_test(point)
    }

    /// Return visible Shape and Text local-id paths under `point`, front to
    /// back. Direct hits contain one local id; child-artboard hits are prefixed
    /// with their nested or component-list host local ids.
    pub fn hit_test_paths(&mut self, point: Vec2D) -> Vec<Vec<usize>> {
        self.raw.hit_test_paths(point)
    }

    /// Return exact logical world bounds for one runtime-local object.
    pub fn world_bounds(&mut self, local_id: usize) -> Option<Aabb> {
        self.raw.object_world_bounds(local_id)
    }

    /// Return the settled, layout-aware world transform for one runtime-local object.
    pub fn world_transform(&mut self, local_id: usize) -> Option<Mat2D> {
        self.raw.object_world_transform(local_id)
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
        let (top, bottom) = self.raw.text_caret(local_id, byte_offset)?;
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
        self.raw.text_hit(local_id, point)
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
        self.raw.text_selection_rects(local_id, range)
    }

    pub fn linear_animation_instance(&self, index: usize) -> Option<LinearAnimationInstance> {
        self.raw.linear_animation_instance(index)
    }

    /// Instantiate the first exact-name linear animation, mirroring C++
    /// `ArtboardInstance::animationNamed` without cross-kind fallback.
    pub fn linear_animation_instance_named(&self, name: &str) -> Option<LinearAnimationInstance> {
        let index = self.artboard().animation_index_named(name)?;
        self.linear_animation_instance(index)
    }

    pub fn state_machine_instance(&mut self, index: usize) -> Option<StateMachineInstance> {
        self.raw.state_machine_instance(index)
    }

    /// Instantiate the first exact-name state machine, mirroring C++
    /// `ArtboardInstance::stateMachineNamed` without cross-kind fallback.
    pub fn state_machine_instance_named(&mut self, name: &str) -> Option<StateMachineInstance> {
        let index = self.artboard().state_machine_index_named(name)?;
        self.state_machine_instance(index)
    }

    /// Instantiate the artboard's default state machine: the one flagged in
    /// the source file when present, otherwise the first state machine.
    pub fn default_state_machine_instance(&mut self) -> Option<StateMachineInstance> {
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
    /// `instance_index` (the order the instances appear in the file), cloning
    /// and completing nested ViewModel/list references like C++
    /// `ViewModelRuntime::createInstanceFromIndex`. Returns `None` when the
    /// artboard has no view model or the index is out of range.
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
        for state_machine in state_machines.iter_mut() {
            let prepared = try_prepare_state_machine_scripted_data_context_without_factory(
                &self.script_file,
                &self.raw,
                state_machine,
                None,
            );
            if prepared.is_err() {
                return false;
            }
        }
        changed |= self
            .raw
            .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
            .unwrap_or(false);
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance_with_script_errors(
                state_machines,
            )
            .unwrap_or(false);
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        advance_and_apply_keep_going(changed, elapsed_seconds, state_machines)
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
        #[cfg(not(feature = "scripting"))]
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        #[cfg(feature = "scripting")]
        for state_machine in state_machines.iter_mut() {
            let prepared = try_prepare_state_machine_scripted_data_context_without_factory(
                &self.script_file,
                &self.raw,
                state_machine,
                Some(view_model),
            );
            if prepared.is_err() {
                return false;
            }
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
        changed |= self
            .raw
            .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
            .unwrap_or(false);
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance_with_script_errors(
                state_machines,
            )
            .unwrap_or(false);
        advance_and_apply_keep_going(changed, elapsed_seconds, state_machines)
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
            changed |= advance_scripted_artboard_frame_with_factory(
                &self.script_file,
                self.artboard().graph(),
                &mut self.raw,
                state_machines,
                elapsed_seconds,
                factory,
                None,
            )
            .context("failed to advance scripted drawables")?;
            changed |= self.file.advance_detached_view_models();
        }
        #[cfg(not(feature = "scripting"))]
        {
            let _ = factory;
            changed |= self
                .raw
                .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
                .context("failed to advance retained artboard components")?;
            changed |= self
                .raw
                .settle_state_machine_update_passes_after_main_advance(state_machines);
        }
        Ok(advance_and_apply_keep_going(
            changed,
            elapsed_seconds,
            state_machines,
        ))
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
        #[cfg(not(feature = "scripting"))]
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
        #[cfg(feature = "scripting")]
        {
            changed |= advance_scripted_artboard_frame_with_factory(
                &self.script_file,
                self.artboard().graph(),
                &mut self.raw,
                state_machines,
                elapsed_seconds,
                factory,
                Some(view_model),
            )
            .context("failed to advance scripted drawables")?;
        }
        #[cfg(not(feature = "scripting"))]
        {
            let _ = factory;
            changed |= self
                .raw
                .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
                .context("failed to advance retained artboard components")?;
            changed |= self
                .raw
                .settle_state_machine_update_passes_after_main_advance(state_machines);
        }
        Ok(advance_and_apply_keep_going(
            changed,
            elapsed_seconds,
            state_machines,
        ))
    }

    pub fn draw(&mut self, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<()> {
        let artboard = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .context("artboard instance graph is unavailable")?;
        #[cfg(feature = "scripting")]
        prepare_scripted_artboard_tree(self.file, artboard, &mut self.raw, factory)
            .context("failed to prepare scripted drawables")?;
        self.raw.update_pass();
        self.raw
            .draw_artboard(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                renderer,
                &self.file.external_image_assets,
                self.file.max_retained_decoded_image_bytes,
                true,
            )
            .context("failed to draw Rive artboard")?;
        self.raw.observe_owned_images()?;
        Ok(())
    }

    /// Drop renderer-owned members before switching this occurrence to a
    /// replacement backend. The next draw rebuilds them from live state.
    pub fn reset_renderer(&self) {
        self.raw.reset_backend_resources();
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
            RuntimeArtboardInstance::from_graph_with_artboards_external_fonts_and_file_catalogs(
                &file.runtime,
                artboard.graph(),
                &file.graph.artboards,
                &external_font_assets,
                file.file_view_model_instances.clone(),
                file.state_machine_actions.clone(),
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
        let mut changed = self
            .raw
            .advance_frame_components(elapsed_seconds)
            .unwrap_or(false);
        changed |= self.raw.update_pass_with_script_errors().unwrap_or(false);
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
        {
            let mut changed = advance_scripted_artboard_frame_with_factory(
                &self.file,
                artboard,
                &mut self.raw,
                &mut [],
                elapsed_seconds,
                factory,
                None,
            )
            .context("failed to advance scripted drawables")?;
            changed |= self.file.advance_detached_view_models();
            return Ok(changed);
        }
        #[cfg(not(feature = "scripting"))]
        {
            let _ = factory;
            let mut changed = self
                .raw
                .advance_frame_components(elapsed_seconds)
                .context("failed to advance retained artboard components")?;
            changed |= self.raw.update_pass();
            Ok(changed)
        }
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
        initialize_state_machine_scripted_objects(
            &self.file,
            &self.raw,
            machine,
            factory,
            root_view_model,
        )
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn rehydrate_flow_listener_actions(
        &self,
        machine: &mut StateMachineInstance,
        root_view_model: Option<&ViewModelInstance>,
        previous_root_view_model: Option<&ViewModelInstance>,
        factory: &mut Option<&mut dyn Factory>,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        rehydrate_script_listener_actions(
            &self.file,
            machine,
            root_view_model,
            previous_root_view_model,
            factory,
        )?;
        Ok(())
    }

    #[cfg(feature = "scripting")]
    pub(crate) fn apply_flow_listener_action_source_updates(
        &self,
        machine: &mut StateMachineInstance,
    ) -> std::result::Result<(), nuxie_runtime::ScriptError> {
        machine.apply_scripted_listener_action_source_updates(
            &self.raw,
            None,
            &mut NoopScriptHost,
        )?;
        Ok(())
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
        self.raw.hit_test(point)
    }

    /// Return visible Shape and Text local-id paths under `point`, front to
    /// back. Direct hits contain one local id; child-artboard hits are prefixed
    /// with their nested or component-list host local ids.
    pub fn hit_test_paths(&mut self, point: Vec2D) -> Vec<Vec<usize>> {
        self.raw.hit_test_paths(point)
    }

    /// Resolve the frontmost concrete geometry occurrence for a native
    /// pointer dispatch. Pass this value to a state-machine instance's
    /// `pointer_*_with_event_context` methods so list/component occurrence
    /// identity survives even when the listener targets implementation-only
    /// hit geometry.
    pub fn pointer_event_context(&mut self, point: Vec2D) -> Option<StateMachineEventContext> {
        self.raw
            .hit_test_segments_with_bounds(point)
            .first()
            .map(StateMachineEventContext::from_geometry_hit)
    }

    pub(crate) fn hit_test_path_segments_with_bounds(
        &mut self,
        point: Vec2D,
    ) -> Vec<RuntimeGeometryHit> {
        self.raw.hit_test_segments_with_bounds(point)
    }

    pub(crate) fn geometry_path_segments_with_bounds(&mut self) -> Vec<RuntimeGeometryHit> {
        self.raw.visible_geometry_with_bounds()
    }

    pub(crate) fn retained_geometry_path_segments_with_bounds(
        &mut self,
    ) -> Vec<RuntimeGeometryHit> {
        self.raw.retained_geometry_with_bounds()
    }

    pub(crate) fn semantic_text_path_segments_with_bounds(
        &mut self,
    ) -> Vec<RuntimeSemanticTextHit> {
        self.raw.semantic_text_with_bounds()
    }

    /// Return exact logical world bounds for one runtime-local object.
    pub fn world_bounds(&mut self, local_id: usize) -> Option<Aabb> {
        self.raw.object_world_bounds(local_id)
    }

    pub(crate) fn register_intrinsic_image_dimensions(
        &mut self,
        asset_global: u32,
        width: u32,
        height: u32,
    ) -> std::result::Result<(), RuntimeImageDimensionConflict> {
        self.raw
            .register_image_dimensions(asset_global, width, height)
    }

    /// Return the settled, layout-aware world transform for one runtime-local object.
    pub fn world_transform(&mut self, local_id: usize) -> Option<Mat2D> {
        self.raw.object_world_transform(local_id)
    }

    /// Return the canonical downstream shaped Text caret in source-artboard
    /// world space for one exact UTF-8 byte boundary.
    ///
    /// This is the owning mirror of [`ArtboardInstance::text_caret`] and has
    /// the same invalid-offset, target-kind, shaping, finite-geometry,
    /// overflow, soft-wrap, and trailing-static-separator behavior.
    pub fn text_caret(&mut self, local_id: usize, byte_offset: usize) -> Option<CaretGeometry> {
        let (top, bottom) = self.raw.text_caret(local_id, byte_offset)?;
        Some(CaretGeometry { top, bottom })
    }

    /// Return the nearest valid UTF-8 byte caret for one source-artboard
    /// world-space point on shaped Text.
    ///
    /// This is the owning mirror of [`ArtboardInstance::text_hit`] and has the
    /// same nonfinite-point, target-kind, shaping, finite-geometry, and
    /// unsupported-or-unknown-overflow failure behavior.
    pub fn text_hit(&mut self, local_id: usize, point: Vec2D) -> Option<usize> {
        self.raw.text_hit(local_id, point)
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
        self.raw.text_selection_rects(local_id, range)
    }

    pub fn linear_animation_instance(&self, index: usize) -> Option<LinearAnimationInstance> {
        self.raw.linear_animation_instance(index)
    }

    /// Owning mirror of [`ArtboardInstance::linear_animation_instance_named`].
    pub fn linear_animation_instance_named(&self, name: &str) -> Option<LinearAnimationInstance> {
        let index = self.artboard().animation_index_named(name)?;
        self.linear_animation_instance(index)
    }

    pub fn state_machine_instance(&mut self, index: usize) -> Option<StateMachineInstance> {
        self.raw.state_machine_instance(index)
    }

    /// Owning mirror of [`ArtboardInstance::state_machine_instance_named`].
    pub fn state_machine_instance_named(&mut self, name: &str) -> Option<StateMachineInstance> {
        let index = self.artboard().state_machine_index_named(name)?;
        self.state_machine_instance(index)
    }

    /// See [`ArtboardInstance::default_state_machine_instance`].
    pub fn default_state_machine_instance(&mut self) -> Option<StateMachineInstance> {
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
        for state_machine in state_machines.iter_mut() {
            let prepared = try_prepare_state_machine_scripted_data_context_without_factory(
                &self.file,
                &self.raw,
                state_machine,
                None,
            );
            if prepared.is_err() {
                return false;
            }
        }
        changed |= self
            .raw
            .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
            .unwrap_or(false);
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance_with_script_errors(
                state_machines,
            )
            .unwrap_or(false);
        #[cfg(feature = "scripting")]
        {
            changed |= self.file.advance_detached_view_models();
        }
        advance_and_apply_keep_going(changed, elapsed_seconds, state_machines)
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
        #[cfg(not(feature = "scripting"))]
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        #[cfg(feature = "scripting")]
        for state_machine in state_machines.iter_mut() {
            let prepared = try_prepare_state_machine_scripted_data_context_without_factory(
                &self.file,
                &self.raw,
                state_machine,
                Some(view_model),
            );
            if prepared.is_err() {
                return false;
            }
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
        changed |= self
            .raw
            .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
            .unwrap_or(false);
        changed |= self
            .raw
            .settle_state_machine_update_passes_after_main_advance_with_script_errors(
                state_machines,
            )
            .unwrap_or(false);
        advance_and_apply_keep_going(changed, elapsed_seconds, state_machines)
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
            changed |= advance_scripted_artboard_frame_with_factory(
                &self.file,
                artboard,
                &mut self.raw,
                state_machines,
                elapsed_seconds,
                factory,
                None,
            )
            .context("failed to advance scripted drawables")?;
            changed |= self.file.advance_detached_view_models();
        }
        #[cfg(not(feature = "scripting"))]
        {
            let _ = factory;
            changed |= self
                .raw
                .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
                .context("failed to advance retained artboard components")?;
            changed |= self
                .raw
                .settle_state_machine_update_passes_after_main_advance(state_machines);
        }
        Ok(advance_and_apply_keep_going(
            changed,
            elapsed_seconds,
            state_machines,
        ))
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
        #[cfg(not(feature = "scripting"))]
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.bind_owned_view_model_handle(view_model.handle());
        }
        changed |= self
            .raw
            .bind_owned_view_model_artboard_handle(&self.file.runtime, view_model.handle());
        #[cfg(feature = "scripting")]
        {
            changed |= advance_scripted_artboard_frame_with_factory(
                &self.file,
                artboard,
                &mut self.raw,
                state_machines,
                elapsed_seconds,
                factory,
                Some(view_model),
            )
            .context("failed to advance scripted drawables")?;
        }
        #[cfg(not(feature = "scripting"))]
        {
            let _ = factory;
            changed |= self
                .raw
                .advance_frame_components_with_state_machines(state_machines, elapsed_seconds)
                .context("failed to advance retained artboard components")?;
            changed |= self
                .raw
                .settle_state_machine_update_passes_after_main_advance(state_machines);
        }
        Ok(advance_and_apply_keep_going(
            changed,
            elapsed_seconds,
            state_machines,
        ))
    }

    pub fn draw(&mut self, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<()> {
        let artboard = self
            .file
            .graph
            .artboards
            .get(self.artboard_index)
            .context("owned artboard instance graph is unavailable")?;
        #[cfg(feature = "scripting")]
        prepare_scripted_artboard_tree(&self.file, artboard, &mut self.raw, factory)
            .context("failed to prepare scripted drawables")?;
        self.raw.update_pass();
        self.raw
            .draw_artboard(
                &self.file.runtime,
                artboard,
                &self.file.graph.artboards,
                factory,
                renderer,
                &self.file.external_image_assets,
                self.file.max_retained_decoded_image_bytes,
                true,
            )
            .context("failed to draw Rive artboard")?;
        self.raw.observe_owned_images()?;
        Ok(())
    }

    /// Drop renderer-owned members before switching this occurrence to a
    /// replacement backend. The next draw rebuilds them from live state.
    pub fn reset_renderer(&self) {
        self.raw.reset_backend_resources();
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
    #[cfg(not(feature = "scripting"))]
    use nuxie_schema::definition_by_name;

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

    #[cfg(not(feature = "scripting"))]
    fn schema_property_key(type_name: &str, property_name: &str) -> u16 {
        definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .properties
            .iter()
            .find(|property| property.name == property_name)
            .unwrap_or_else(|| panic!("missing schema property {type_name}.{property_name}"))
            .key
            .int
    }

    #[cfg(not(feature = "scripting"))]
    fn start_clamped_scroll_physics(
        instance: &mut RuntimeArtboardInstance,
        state_machine: &mut StateMachineInstance,
    ) {
        instance
            .advance_frame_components(0.0)
            .expect("scroll fixture initializes its retained advancing owners");
        instance.update_pass();
        let mut host = NoopScriptHost;
        assert!(
            state_machine
                .try_pointer_down_with_timestamp_and_script_host(
                    instance, 40.0, 500.0, 0, 1.2, &mut host,
                )
                .expect("pointer down succeeds")
        );
        assert!(
            state_machine
                .try_pointer_move_with_timestamp_and_script_host(
                    instance, 30.0, 440.0, 0, 1.232, &mut host,
                )
                .expect("pointer move succeeds")
        );
        assert!(
            state_machine
                .try_pointer_up_with_timestamp_and_script_host(
                    instance, 30.0, 440.0, 0, 1.248, &mut host,
                )
                .expect("pointer up succeeds")
        );
    }

    #[cfg(not(feature = "scripting"))]
    fn detached_test_view_model() -> (File, ViewModelInstance) {
        let file = File::import(&external_fixture("custom_property_trigger.riv"))
            .expect("custom-property fixture imports");
        let instance = file
            .artboard_named("Main")
            .expect("custom-property fixture has Main")
            .instantiate()
            .expect("custom-property fixture instantiates");
        let view_model = instance
            .instantiate_view_model()
            .expect("custom-property fixture has a view model");
        drop(instance);
        (file, view_model)
    }

    #[cfg(not(feature = "scripting"))]
    #[test]
    fn borrowed_factory_view_model_advance_uses_mixed_schedule_and_bounded_settlement() {
        let scroll_file =
            File::import(&external_fixture("scroll_test.riv")).expect("scroll fixture imports");
        let mut scroll = scroll_file
            .artboard_named("Artboard 2")
            .expect("scroll fixture has Artboard 2")
            .instantiate()
            .expect("scroll artboard instantiates");
        let mut scroll_machine = scroll
            .state_machine_instance(0)
            .expect("scroll artboard has a state machine");
        start_clamped_scroll_physics(scroll.raw_mut(), &mut scroll_machine);
        let (_view_model_file, mut view_model) = detached_test_view_model();
        let mut factory = RecordingFactory::new();

        assert!(
            scroll
                .try_advance_with_state_machines_and_view_model_and_factory(
                    std::slice::from_mut(&mut scroll_machine),
                    0.1,
                    &mut view_model,
                    &mut factory,
                )
                .expect("borrowed facade advance succeeds")
        );
        assert!(
            !scroll
                .raw_mut()
                .advance_frame_components(0.1)
                .expect("mixed-family follow-up succeeds"),
            "the facade consumes the one-frame clamped ScrollPhysics run from \
             C++ m_advancingComponents; a direct follow-up is idle \
             (`artboard.cpp:1463-1480`; `scroll_constraint.cpp:299-336`; \
             `clamped_scroll_physics.cpp:6-9`)"
        );

        let trigger_file = File::import(&external_fixture("custom_property_trigger.riv"))
            .expect("custom-property fixture imports");
        let mut trigger = trigger_file
            .artboard_named("Main")
            .expect("custom-property fixture has Main")
            .instantiate()
            .expect("custom-property artboard instantiates");
        let mut trigger_machine = trigger
            .state_machine_instance(0)
            .expect("custom-property artboard has a state machine");
        let mut trigger_view_model = trigger
            .instantiate_view_model()
            .expect("custom-property artboard has a view model");
        let trigger_value = schema_property_key("CustomPropertyTrigger", "propertyValue");
        assert!(trigger.raw_mut().set_uint_property(7, trigger_value, 1));

        trigger
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut trigger_machine),
                0.1,
                &mut trigger_view_model,
                &mut factory,
            )
            .expect("borrowed trigger settlement succeeds");
        assert!(
            trigger.raw_mut().set_uint_property(7, trigger_value, 1),
            "advanceAndApply runs the bounded update/reset loop, so \
             Artboard::reset consumes CustomPropertyTrigger after the pass \
             (`state_machine_instance.cpp:2622-2654`; `artboard.cpp:1483-1493`)"
        );
    }

    #[cfg(not(feature = "scripting"))]
    #[test]
    fn owned_factory_view_model_advance_uses_mixed_schedule_and_bounded_settlement() {
        let scroll_file =
            Arc::new(File::import(&external_fixture("scroll_test.riv")).expect("scroll fixture"));
        let scroll_index = scroll_file
            .artboard_named("Artboard 2")
            .expect("scroll fixture has Artboard 2")
            .index();
        let mut scroll = OwnedArtboardInstance::instantiate(scroll_file, scroll_index)
            .expect("owned scroll artboard instantiates");
        let mut scroll_machine = scroll
            .state_machine_instance(0)
            .expect("scroll artboard has a state machine");
        start_clamped_scroll_physics(scroll.raw_mut(), &mut scroll_machine);
        let (_view_model_file, mut view_model) = detached_test_view_model();
        let mut factory = RecordingFactory::new();

        assert!(
            scroll
                .try_advance_with_state_machines_and_view_model_and_factory(
                    std::slice::from_mut(&mut scroll_machine),
                    0.1,
                    &mut view_model,
                    &mut factory,
                )
                .expect("owned facade advance succeeds")
        );
        assert!(
            !scroll
                .raw_mut()
                .advance_frame_components(0.1)
                .expect("mixed-family follow-up succeeds"),
            "the owning facade consumes the one-frame clamped ScrollPhysics \
             run in the C++ advancing-component slot \
             (`artboard.cpp:1463-1480`; `scroll_constraint.cpp:299-336`; \
             `clamped_scroll_physics.cpp:6-9`)"
        );

        let trigger_file = Arc::new(
            File::import(&external_fixture("custom_property_trigger.riv"))
                .expect("custom-property fixture imports"),
        );
        let trigger_index = trigger_file
            .artboard_named("Main")
            .expect("custom-property fixture has Main")
            .index();
        let mut trigger = OwnedArtboardInstance::instantiate(trigger_file, trigger_index)
            .expect("owned custom-property artboard instantiates");
        let mut trigger_machine = trigger
            .state_machine_instance(0)
            .expect("custom-property artboard has a state machine");
        let mut trigger_view_model = trigger
            .instantiate_view_model()
            .expect("custom-property artboard has a view model");
        let trigger_value = schema_property_key("CustomPropertyTrigger", "propertyValue");
        assert!(trigger.raw_mut().set_uint_property(7, trigger_value, 1));

        trigger
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut trigger_machine),
                0.1,
                &mut trigger_view_model,
                &mut factory,
            )
            .expect("owned trigger settlement succeeds");
        assert!(
            trigger.raw_mut().set_uint_property(7, trigger_value, 1),
            "the owning facade must run advanceAndApply's bounded update/reset loop \
             (`state_machine_instance.cpp:2622-2654`; `artboard.cpp:1483-1493`)"
        );
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
