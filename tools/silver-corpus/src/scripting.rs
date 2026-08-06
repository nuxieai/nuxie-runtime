//! Scripting attach flow for silver replays.
//!
//! Ports the rust-golden-runner's File VM registration and state-machine
//! scripted-object attachment (`tools/rust-golden-runner/src/main.rs`) so
//! silver action streams replay with a live nuxie-scripting VM when the
//! fixture carries `ScriptAsset` objects, matching the scripting-enabled
//! upstream test build that produced the pinned `.sriv` files.
//!
//! The port is bounded to what silver action streams exercise: scripted
//! drawables are not realized, scripted data-converter bind steps are not
//! bound, and a script that asks the harness to realize a child artboard
//! receives a `ScriptError` (leaving that occurrence inert, the same terminal
//! state C++ assigns a failed script).

use anyhow::{Result, anyhow};
use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;
use nuxie_render_api::Factory as RenderFactory;
use nuxie_runtime::{
    ArtboardInstance, NoopScriptHost, RuntimeOwnedViewModelContext,
    RuntimeOwnedViewModelContextHandle, ScriptArtboard, ScriptError, ScriptValue, ScriptViewModel,
    StateMachineInstance,
};
use nuxie_scripting::vm::{ScriptProgram, ScriptVm};
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone)]
struct ExtractedScriptAsset {
    asset_id: u64,
    name: String,
    is_module: bool,
    payload: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct RegisteredScriptFile {
    vm: Rc<ScriptVm>,
    script_programs: Rc<BTreeMap<u64, ScriptProgram>>,
}

fn extract_script_assets(runtime: &RuntimeFile) -> BTreeMap<u64, ExtractedScriptAsset> {
    runtime
        .scripting_file_assets_with_contents()
        .into_iter()
        .filter(|entry| entry.asset.type_name == "ScriptAsset")
        .filter_map(|entry| {
            let payload = entry.contents?;
            let name = entry.asset.string_property("name").unwrap_or("unnamed");
            let folder = entry
                .asset
                .string_property("folderPath")
                .unwrap_or_default();
            Some((
                entry.ordinal as u64,
                ExtractedScriptAsset {
                    asset_id: entry.ordinal as u64,
                    name: if folder.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{folder}/{name}")
                    },
                    is_module: entry.asset.bool_property("isModule").unwrap_or(false),
                    payload: payload.to_vec(),
                },
            ))
        })
        .collect()
}

fn prepare_script_vm(
    runtime: &RuntimeFile,
    script_assets: &BTreeMap<u64, ExtractedScriptAsset>,
    factory: &mut dyn RenderFactory,
) -> Result<ScriptVm> {
    let mut vm = ScriptVm::new();
    vm.install_render_factory(factory)?;
    vm.set_view_models(nuxie_runtime::script_view_models(runtime));

    // C++ retries module registration until the dependency graph converges.
    // Preserve the original FileAsset ordering within each pass.
    let mut pending = script_assets
        .values()
        .filter(|asset| asset.is_module)
        .collect::<Vec<_>>();
    loop {
        let before = pending.len();
        let mut failures = Vec::new();
        for asset in pending {
            if let Err(error) =
                vm.register_module_with_factory(&asset.name, &asset.payload, factory)
            {
                failures.push((asset, error));
            }
        }
        if failures.is_empty() {
            break;
        }
        if failures.len() == before {
            // C++ `ScriptingContext::performRegistration` reports unresolved
            // module errors but does not reject the File. Preserve the
            // partially registered VM so non-scripted artboard content still
            // renders and later protocol lookup remains safely absent.
            break;
        }
        pending = failures.into_iter().map(|(asset, _)| asset).collect();
    }

    Ok(vm)
}

fn register_script_file(
    runtime: &RuntimeFile,
    script_assets: &BTreeMap<u64, ExtractedScriptAsset>,
    factory: &mut dyn RenderFactory,
) -> Result<RegisteredScriptFile> {
    let vm = prepare_script_vm(runtime, script_assets, factory)?;
    let mut script_programs = BTreeMap::new();
    for script in script_assets.values().filter(|asset| !asset.is_module) {
        // The C++ file-registration pass reports unresolved ScriptAsset
        // dependencies but keeps the File usable. A protocol is only required
        // if an artboard actually references it, so preserve the background
        // render instead of rejecting unrelated test/library scripts.
        if let Ok(program) =
            vm.register_protocol_script_with_factory(&script.name, &script.payload, factory)
        {
            script_programs.insert(script.asset_id, program);
        }
    }
    Ok(RegisteredScriptFile {
        vm: Rc::new(vm),
        script_programs: Rc::new(script_programs),
    })
}

/// Registers the fixture's File VM while the artboard renderer initializes,
/// mirroring the C++ import-time registration point inside render-paint
/// preallocation (`tools/rust-golden-runner/src/main.rs` run(): the
/// non-scripted-layout branch).
pub(crate) fn initialize_renderer_and_register_scripts(
    instance: &ArtboardInstance,
    runtime: &RuntimeFile,
    artboard: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> Result<Option<RegisteredScriptFile>> {
    let script_assets = extract_script_assets(runtime);
    let mut registration_result = None;
    instance.initialize_scripted_artboard_renderer_with_file_registration(
        runtime,
        artboard,
        artboards,
        factory,
        |factory| {
            if !script_assets.is_empty() {
                registration_result = Some(register_script_file(runtime, &script_assets, factory));
            }
        },
        None,
    )?;
    let registered = registration_result.transpose()?;
    if let (Some(registered), Some(image_assets)) = (
        registered.as_ref(),
        instance.scripted_runtime_image_assets(),
    ) {
        registered.vm.set_image_asset_owners(image_assets);
    }
    Ok(registered)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum HydrationPhase {
    Cold,
    Live,
}

enum PreparedScriptInput {
    Value {
        name: nuxie_runtime::ScriptCoreString,
        value: ScriptValue,
    },
    Artboard {
        name: nuxie_runtime::ScriptCoreString,
        artboard_id: usize,
    },
    ViewModel {
        input_global_id: u32,
        name: nuxie_runtime::ScriptCoreString,
        path: nuxie_runtime::ScriptInputViewModelPropertyPath,
    },
}

/// The silver harness does not realize child artboards for scripts. A script
/// that requests one fails with a `ScriptError`, which the caller maps to the
/// same inert terminal state C++ assigns a failed script.
#[derive(Debug)]
struct SilverScriptArtboardResolver;

impl nuxie_runtime::ScriptArtboardResolver for SilverScriptArtboardResolver {
    fn resolve_script_artboard(
        &self,
        artboard_id: u64,
        _parent_context: Option<&nuxie_runtime::ScriptArtboardParentContext>,
    ) -> std::result::Result<Box<dyn ScriptArtboard>, ScriptError> {
        Err(ScriptError::new(format!(
            "silver harness does not realize script artboard {artboard_id}"
        )))
    }
}

#[derive(Clone)]
struct SilverScriptViewModelInputResolver {
    runtime: RuntimeFile,
    context: nuxie_runtime::ScriptArtboardParentContext,
}

impl std::fmt::Debug for SilverScriptViewModelInputResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SilverScriptViewModelInputResolver")
            .finish_non_exhaustive()
    }
}

impl nuxie_runtime::ScriptViewModelInputResolver for SilverScriptViewModelInputResolver {
    fn resolve_script_view_model(
        &self,
        input_global_id: u32,
        path: &nuxie_runtime::ScriptInputViewModelPropertyPath,
    ) -> std::result::Result<Option<ScriptViewModel>, ScriptError> {
        self.context
            .resolve_script_view_model_input(&self.runtime, path)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "ScriptInputViewModelProperty global {input_global_id} became unresolved during authored hydration"
                ))
            })
    }
}

/// C++ leaves a scripted occurrence inert when its script fails
/// (`scripted_object.cpp:532-540`); only harness-level errors abort.
fn script_or_inert<T>(result: std::result::Result<T, ScriptError>) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(_) => Ok(None),
    }
}

fn state_machine_script_context_hydration(
    runtime: &RuntimeFile,
    state_machine: &StateMachineInstance,
    owned_context: Option<&RuntimeOwnedViewModelContext>,
    phase: HydrationPhase,
) -> nuxie_runtime::ScriptListenerActionHydration {
    if phase == HydrationPhase::Cold {
        return nuxie_runtime::ScriptListenerActionHydration::unresolved(Vec::new());
    }
    let fallback_root = owned_context.and_then(RuntimeOwnedViewModelContext::main_handle);
    let (context_view_model, context_parent_view_models) =
        state_machine.scripted_listener_data_context_view_models(runtime, fallback_root);
    if state_machine.has_scripted_listener_data_context() || fallback_root.is_some() {
        nuxie_runtime::ScriptListenerActionHydration::new_with_context_chain(
            context_view_model,
            context_parent_view_models,
            Vec::new(),
        )
    } else {
        nuxie_runtime::ScriptListenerActionHydration::unresolved(Vec::new())
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_state_machine_script_hydration_from_snapshots(
    runtime: &RuntimeFile,
    artboards: &[ArtboardGraph],
    state_machine: &StateMachineInstance,
    snapshots: Vec<nuxie_runtime::ScriptListenerInputSnapshot>,
    owner: &str,
    owned_context: Option<&RuntimeOwnedViewModelContext>,
    phase: HydrationPhase,
) -> std::result::Result<nuxie_runtime::ScriptListenerActionHydration, ScriptError> {
    let fallback_root = (phase == HydrationPhase::Live)
        .then(|| owned_context.and_then(RuntimeOwnedViewModelContext::main_handle))
        .flatten();
    let root_context =
        fallback_root.map(|root| RuntimeOwnedViewModelContextHandle::root(runtime, root.clone()));
    let (context_view_model, context_parent_view_models) = if phase == HydrationPhase::Live {
        state_machine.scripted_listener_data_context_view_models(runtime, fallback_root)
    } else {
        (None, Vec::new())
    };
    let context_resolved = phase == HydrationPhase::Live
        && (state_machine.has_scripted_listener_data_context() || fallback_root.is_some());
    let artboard_parent_context = if phase == HydrationPhase::Live {
        state_machine.scripted_listener_artboard_parent_context(root_context.as_ref())
    } else {
        None
    };

    // Validate the complete cloned input collection before touching the Lua
    // table (`scripted_object.cpp:399-426`).
    let mut prepared = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        let input = runtime
            .object(snapshot.input_global_id as usize)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "{owner} input global {}: object is absent",
                    snapshot.input_global_id
                ))
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
            return Err(ScriptError::new(format!(
                "{owner} input global {}: expected {expected_type}, found {}",
                input.id, input.type_name
            )));
        }
        let name = snapshot.name;
        match snapshot.kind {
            nuxie_runtime::ScriptListenerInputKind::Boolean
            | nuxie_runtime::ScriptListenerInputKind::Number
            | nuxie_runtime::ScriptListenerInputKind::Color
            | nuxie_runtime::ScriptListenerInputKind::String => {
                let Some(nuxie_runtime::ScriptListenerInputSnapshotValue::Value(value)) =
                    snapshot.value
                else {
                    return Err(ScriptError::new(format!(
                        "{owner} input global {}: cloned scalar value is unavailable",
                        input.id
                    )));
                };
                prepared.push(PreparedScriptInput::Value { name, value });
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
                    ScriptError::new(format!(
                        "{owner} input global {}: referenced artboard is unresolved",
                        input.id
                    ))
                })?;
                if artboards.get(artboard_id).is_none() {
                    return Err(ScriptError::new(format!(
                        "{owner} input global {}: referenced artboard {artboard_id} is unavailable",
                        input.id
                    )));
                }
                prepared.push(PreparedScriptInput::Artboard { name, artboard_id });
            }
            nuxie_runtime::ScriptListenerInputKind::ViewModelProperty => {
                if phase == HydrationPhase::Cold {
                    return Err(ScriptError::new(format!(
                        "{owner} input global {}: view-model property path is unresolved during cold initialization",
                        input.id
                    )));
                }
                let path = snapshot.view_model_path.ok_or_else(|| {
                    ScriptError::new(format!(
                        "{owner} input global {}: cloned view-model property path is absent",
                        input.id
                    ))
                })?;
                state_machine
                    .scripted_listener_bound_view_model(runtime, &path, root_context.as_ref())
                    .ok_or_else(|| {
                        ScriptError::new(format!(
                            "{owner} input global {}: view-model property path is unresolved",
                            input.id
                        ))
                    })?;
                prepared.push(PreparedScriptInput::ViewModel {
                    input_global_id: input.id,
                    name,
                    path,
                });
            }
        }
    }

    let artboard_resolver: Rc<dyn nuxie_runtime::ScriptArtboardResolver> =
        Rc::new(SilverScriptArtboardResolver);
    let view_model_resolver = artboard_parent_context.clone().map(|context| {
        Rc::new(SilverScriptViewModelInputResolver {
            runtime: runtime.clone(),
            context,
        }) as Rc<dyn nuxie_runtime::ScriptViewModelInputResolver>
    });
    let mut inputs = Vec::with_capacity(prepared.len());
    for input in prepared {
        match input {
            PreparedScriptInput::Value { name, value } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::Value { name, value });
            }
            PreparedScriptInput::Artboard { name, artboard_id } => {
                inputs.push(nuxie_runtime::ScriptListenerInputHydration::Artboard {
                    name,
                    artboard_id: u64::try_from(artboard_id)
                        .expect("validated ScriptInputArtboard id originated as u64"),
                    resolver: Rc::clone(&artboard_resolver),
                    parent_context: artboard_parent_context.clone(),
                });
            }
            PreparedScriptInput::ViewModel {
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

fn instantiate_state_machine_scripted_object_table(
    runtime: &RuntimeFile,
    state_machine: &mut StateMachineInstance,
    definition: &nuxie_runtime::ScriptListenerActionDefinition,
    factory: &mut dyn RenderFactory,
    owned_context: Option<&RuntimeOwnedViewModelContext>,
    registered_file: &RegisteredScriptFile,
    phase: HydrationPhase,
) -> Result<bool> {
    if state_machine.has_scripted_object_instance(definition.scripted_object_global_id()) {
        return Ok(true);
    }
    if !definition.has_protocol_asset() {
        return Ok(false);
    }
    let Some(program) = registered_file
        .script_programs
        .get(&(definition.asset_ordinal() as u64))
    else {
        return Ok(false);
    };
    let (context_view_model, context_parent_view_models) = if phase == HydrationPhase::Live {
        state_machine.scripted_listener_data_context_view_models(
            runtime,
            owned_context.and_then(RuntimeOwnedViewModelContext::main_handle),
        )
    } else {
        (None, Vec::new())
    };
    let mut host = NoopScriptHost;
    let Some(instance) = script_or_inert(
        registered_file
            .vm
            .instantiate_registered_script_with_factory_and_context(
                program,
                &mut host,
                factory,
                context_view_model,
                context_parent_view_models,
            ),
    )?
    else {
        return Ok(false);
    };
    state_machine
        .set_scripted_object_instance(definition.scripted_object_global_id(), instance)
        .map_err(|error| anyhow!(error))?;
    Ok(true)
}

fn hydrate_live_state_machine_scripted_objects(
    runtime: &RuntimeFile,
    artboards: &[ArtboardGraph],
    state_machine: &mut StateMachineInstance,
    definitions: &[nuxie_runtime::ScriptListenerActionDefinition],
    factory: &mut dyn RenderFactory,
    owned_context: Option<&RuntimeOwnedViewModelContext>,
    registered_file: &RegisteredScriptFile,
) -> Result<()> {
    // `internalDataContext` assigns the live context to every retained table
    // before `initScriptedObjects` enters its first occurrence
    // (`state_machine_instance.cpp:2901-2913`).
    let live_context = state_machine_script_context_hydration(
        runtime,
        state_machine,
        owned_context,
        HydrationPhase::Live,
    );
    for definition in definitions {
        if state_machine.has_scripted_object_instance(definition.scripted_object_global_id()) {
            state_machine
                .install_scripted_object_data_context(
                    definition.scripted_object_global_id(),
                    &live_context,
                )
                .map_err(|error| anyhow!(error))?;
        }
    }
    for definition in definitions {
        let already_attached =
            state_machine.has_scripted_object_instance(definition.scripted_object_global_id());
        if !instantiate_state_machine_scripted_object_table(
            runtime,
            state_machine,
            definition,
            factory,
            owned_context,
            registered_file,
            HydrationPhase::Live,
        )? {
            continue;
        }
        if !already_attached {
            state_machine
                .install_scripted_object_data_context(
                    definition.scripted_object_global_id(),
                    &live_context,
                )
                .map_err(|error| anyhow!(error))?;
        }
        let owner = format!(
            "state-machine ScriptedObject global {}",
            definition.scripted_object_global_id()
        );
        let result = state_machine
            .hydrate_and_initialize_scripted_object_instance_after_context_install(
                definition.scripted_object_global_id(),
                definition.inits(),
                Some(factory),
                |state_machine| {
                    let snapshots = state_machine
                        .scripted_listener_action_input_snapshots(
                            definition.scripted_object_global_id(),
                        )
                        .ok_or_else(|| {
                            ScriptError::new(format!("{owner}: cloned input occurrence is absent"))
                        })?;
                    prepare_state_machine_script_hydration_from_snapshots(
                        runtime,
                        artboards,
                        state_machine,
                        snapshots,
                        &owner,
                        owned_context,
                        HydrationPhase::Live,
                    )
                },
            );
        let _ = script_or_inert(result)?;
    }
    Ok(())
}

fn retry_cold_state_machine_scripted_objects_during_constructor(
    runtime: &RuntimeFile,
    artboards: &[ArtboardGraph],
    state_machine: &mut StateMachineInstance,
    definitions: &[nuxie_runtime::ScriptListenerActionDefinition],
    factory: &mut dyn RenderFactory,
    registered_file: &RegisteredScriptFile,
) -> Result<()> {
    // C++ follows clone/reinit with a second `initScriptedObjects` loop even
    // when the owning Artboard's DataContext is null. Failed cold
    // generator/init attempts therefore retry before any later context bind
    // (`state_machine_instance.cpp:2072-2082`; `scripted_object.cpp:532-540`).
    for definition in definitions {
        if !instantiate_state_machine_scripted_object_table(
            runtime,
            state_machine,
            definition,
            factory,
            None,
            registered_file,
            HydrationPhase::Cold,
        )? {
            continue;
        }
        let context = state_machine_script_context_hydration(
            runtime,
            state_machine,
            None,
            HydrationPhase::Cold,
        );
        let owner = format!(
            "state-machine ScriptedObject global {}",
            definition.scripted_object_global_id()
        );
        let result = state_machine.hydrate_and_initialize_scripted_object_instance(
            definition.scripted_object_global_id(),
            context,
            definition.inits(),
            Some(factory),
            |state_machine| {
                let snapshots = state_machine
                    .scripted_listener_action_input_snapshots(
                        definition.scripted_object_global_id(),
                    )
                    .ok_or_else(|| {
                        ScriptError::new(format!("{owner}: cloned input occurrence is absent"))
                    })?;
                prepare_state_machine_script_hydration_from_snapshots(
                    runtime,
                    artboards,
                    state_machine,
                    snapshots,
                    &owner,
                    None,
                    HydrationPhase::Cold,
                )
            },
        );
        let _ = script_or_inert(result)?;
    }
    Ok(())
}

/// Clones, hydrates, and initializes the state machine's fixed
/// `ScriptedObject` collection before any silver action runs, mirroring the
/// golden runner's construction-time attach. The silver harness always
/// constructs the state machine before its view-model bind action, so the
/// constructor context is never prebound and the cold retry branch applies
/// (`lua_artboards.cpp:20-50`).
pub(crate) fn initialize_state_machine_scripted_objects(
    runtime: &RuntimeFile,
    artboards: &[ArtboardGraph],
    state_machine: &mut StateMachineInstance,
    factory: &mut dyn RenderFactory,
    registered_file: &RegisteredScriptFile,
) -> Result<()> {
    let definitions = state_machine.scripted_objects().to_vec();
    if definitions.is_empty()
        && state_machine
            .state_machine_data_converter_bind_steps()
            .is_empty()
        && state_machine
            .scripted_listener_data_converter_bind_steps()
            .is_empty()
    {
        return Ok(());
    }

    state_machine.set_scripted_listener_artboard_resolver(Box::new(SilverScriptArtboardResolver));

    // Clone/reinit the complete fixed ScriptedObject collection before the
    // live DataContext is assigned (`state_machine_instance.cpp:2072-2082`).
    for definition in &definitions {
        if !instantiate_state_machine_scripted_object_table(
            runtime,
            state_machine,
            definition,
            factory,
            None,
            registered_file,
            HydrationPhase::Cold,
        )? {
            continue;
        }
        let context = state_machine_script_context_hydration(
            runtime,
            state_machine,
            None,
            HydrationPhase::Cold,
        );
        let owner = format!(
            "state-machine ScriptedObject global {}",
            definition.scripted_object_global_id()
        );
        let result = state_machine.hydrate_and_initialize_scripted_object_instance(
            definition.scripted_object_global_id(),
            context,
            definition.inits(),
            Some(factory),
            |state_machine| {
                let snapshots = state_machine
                    .scripted_listener_action_input_snapshots(
                        definition.scripted_object_global_id(),
                    )
                    .ok_or_else(|| {
                        ScriptError::new(format!("{owner}: cloned input occurrence is absent"))
                    })?;
                prepare_state_machine_script_hydration_from_snapshots(
                    runtime,
                    artboards,
                    state_machine,
                    snapshots,
                    &owner,
                    None,
                    HydrationPhase::Cold,
                )
            },
        );
        let _ = script_or_inert(result)?;
    }

    retry_cold_state_machine_scripted_objects_during_constructor(
        runtime,
        artboards,
        state_machine,
        &definitions,
        factory,
        registered_file,
    )?;

    state_machine.begin_retained_scripted_object_data_context_rebind();
    // Bounded port: scripted data-converter bind steps are not replayed by
    // the silver harness; a fixture that carries them keeps its recorded
    // divergence.
    state_machine.finish_scripted_object_data_context_bind();

    hydrate_live_state_machine_scripted_objects(
        runtime,
        artboards,
        state_machine,
        &definitions,
        factory,
        None,
        registered_file,
    )?;
    state_machine.mark_scripted_object_initialization_complete(None);
    Ok(())
}

/// Applies the state-machine half of a Scene view-model bind for a machine
/// whose DataContext owner is a fixed scripted object
/// (`state_machine_instance.cpp:2776-2790`).
pub(crate) fn rebind_state_machine_scripted_objects_after_artboard(
    runtime: &RuntimeFile,
    artboards: &[ArtboardGraph],
    state_machine: &mut StateMachineInstance,
    factory: &mut dyn RenderFactory,
    owned_context: &RuntimeOwnedViewModelContext,
    registered_file: &RegisteredScriptFile,
) -> Result<()> {
    let definitions = state_machine.scripted_objects().to_vec();
    state_machine.require_scripted_object_data_context_rebind();
    let Some(root) = owned_context.main_handle() else {
        return Ok(());
    };
    if !state_machine.begin_scripted_object_data_context_bind(root) {
        return Ok(());
    }
    // Bounded port: scripted data-converter bind steps are not replayed.
    state_machine.finish_scripted_object_data_context_bind();
    hydrate_live_state_machine_scripted_objects(
        runtime,
        artboards,
        state_machine,
        &definitions,
        factory,
        Some(owned_context),
        registered_file,
    )?;
    state_machine.mark_scripted_facade_root_hydrated(Some(root));
    Ok(())
}
