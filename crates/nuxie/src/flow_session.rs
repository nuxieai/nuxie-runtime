//! High-level, renderer-neutral execution seam for one remote UI flow.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

use crate::{
    Factory, File, LinearAnimationInstance, NoopScriptHost, OwnedArtboardInstance, Renderer,
    StateMachineInstance, ViewModelInstance,
};
#[cfg(feature = "scripting")]
use crate::{LuaHostCommand, LuaHostValue};
use nuxie_runtime::{
    RuntimeEventPropertyValue, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    RuntimeViewModelLinkError, StateMachineReportedEvent,
};

/// Maximum UTF-8 byte length accepted for an identifier or property path.
pub const MAX_ID_PATH_BYTES: usize = 4 * 1024;
/// Maximum number of instances retained by one session.
pub const MAX_INSTANCES: usize = 4 * 1024;
pub const MAX_BATCH_ITEMS: usize = 4 * 1024;
pub const MAX_POINTERS_PER_BATCH: usize = 32;
pub const MAX_STRING_BYTES: usize = 1024 * 1024;
pub const MAX_LIST_ITEMS: usize = 4 * 1024;
pub const MAX_VALUE_NODES: usize = 4 * 1024;
pub const MAX_EVENT_PROPERTIES: usize = 256;
pub const MAX_VALUE_EDGES: usize = 16 * 1024;
pub const MAX_VALUE_DEPTH: usize = 32;
pub const MAX_ENCODED_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

/// Explicit named-player operation requested when creating a flow session.
///
/// These variants deliberately mirror C++ `ArtboardInstance::stateMachineNamed`
/// and `ArtboardInstance::animationNamed`. The namespaces stay separate even
/// when a state machine and a linear animation have the same authored name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowPlayerSelector {
    StateMachine(String),
    LinearAnimation(String),
}

/// Selection requested when creating a flow session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowSessionConfig {
    pub artboard_name: Option<String>,
    /// `None` mirrors C++ `defaultScene`; `Some` invokes exactly one typed
    /// named-player operation without cross-kind fallback.
    pub player: Option<FlowPlayerSelector>,
}

/// Machine-readable category for a rejected flow operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowSessionErrorKind {
    NotFound,
    InvalidArgument,
    LimitExceeded,
    /// A successfully-authored runtime result cannot fit the stable host ABI.
    ResultLimitExceeded,
    /// Authenticated script work exhausted a fixed VM/host-effect resource.
    ScriptResourceExceeded,
    Conflict,
    Runtime,
}

/// Typed failure at the flow-session seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowSessionError {
    kind: FlowSessionErrorKind,
    message: String,
}

impl FlowSessionError {
    fn new(kind: FlowSessionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub const fn kind(&self) -> FlowSessionErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for FlowSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FlowSessionError {}

fn flow_script_error(error: crate::ScriptError) -> FlowSessionError {
    let kind = if error.resource_code().is_some() {
        FlowSessionErrorKind::ScriptResourceExceeded
    } else {
        FlowSessionErrorKind::Runtime
    };
    FlowSessionError::new(kind, format!("script execution failed: {error}"))
}

fn flow_anyhow_error(error: anyhow::Error) -> FlowSessionError {
    let kind = if error.chain().any(|cause| {
        cause
            .downcast_ref::<crate::ScriptError>()
            .is_some_and(|error| error.resource_code().is_some())
    }) {
        FlowSessionErrorKind::ScriptResourceExceeded
    } else {
        FlowSessionErrorKind::Runtime
    };
    FlowSessionError::new(kind, error.to_string())
}

/// Stable, session-scoped identity exposed to hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlowInstanceId(u64);

impl FlowInstanceId {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Runtime player selected for the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPlayerKind {
    StateMachine,
    LinearAnimation,
    Static,
}

/// Exact selection branch used during deterministic creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPlayerSelection {
    ExplicitStateMachine,
    AuthoredDefaultStateMachine,
    FirstStateMachine,
    FirstAnimation,
    Static,
    ExplicitLinearAnimation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowPlayerMetadata {
    pub kind: FlowPlayerKind,
    pub selection: FlowPlayerSelection,
    pub index: Option<usize>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowArtboardBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowValueType {
    Null,
    String,
    Number,
    Bool,
    Enum,
    /// Authored component-list item index. This is an ordinal, not an enum
    /// identity, even though both are represented by unsigned integers.
    ListIndex,
    Color,
    Image,
    Object,
    ViewModel,
    List,
    /// Semantic trigger property. Its monotonic count is represented by an
    /// `Enum` node because the canonical recursive arena has no trigger node.
    Trigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowPropertySchema {
    pub name: String,
    pub value_type: FlowValueType,
    /// Authored enum labels in their stable numeric-value order.
    pub enum_labels: Vec<String>,
    /// Schema accepted by a nested view-model property.
    pub referenced_schema_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowSchema {
    pub name: String,
    pub properties: Vec<FlowPropertySchema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowInstanceMetadata {
    pub id: FlowInstanceId,
    pub schema_name: String,
    pub authored_name: Option<String>,
    pub is_root: bool,
}

/// Authored immutable recipe. Templates are not addressable mutable instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowInstanceTemplate {
    pub schema_name: String,
    pub authored_name: Option<String>,
    pub authored_index: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowCatalog {
    pub schemas: Vec<FlowSchema>,
    pub templates: Vec<FlowInstanceTemplate>,
    pub instances: Vec<FlowInstanceMetadata>,
    pub root_instance_id: Option<FlowInstanceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlowValueId(u32);

impl FlowValueId {
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowValue {
    Null,
    String(String),
    Number(f32),
    Bool(bool),
    Enum(u64),
    ListIndex(u64),
    Color(u32),
    Image(u64),
    Object(Vec<(String, FlowValueId)>),
    ViewModel(Vec<(String, FlowValueId)>),
    List(Vec<FlowValueId>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowValueNode {
    pub id: FlowValueId,
    pub value: FlowValue,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlowValueArena {
    pub roots: Vec<(FlowInstanceId, FlowValueId)>,
    pub nodes: Vec<FlowValueNode>,
}

/// Immutable creation result. Building it does not advance a player or drain
/// any runtime queue.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowBootstrap {
    pub artboard_index: usize,
    pub artboard_name: Option<String>,
    pub player: FlowPlayerMetadata,
    pub bounds: FlowArtboardBounds,
    pub catalog: FlowCatalog,
    pub values: FlowValueArena,
}

/// Result produced while creating a factory-bound session. Script modules,
/// protocol generators, and init hooks may enqueue host work, but creation
/// never advances the selected player.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowSessionCreation {
    pub bootstrap: FlowBootstrap,
    pub outputs: Vec<FlowOutput>,
    pub dirty: bool,
    pub settled: bool,
    pub wake_after_seconds: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlowOutputPhase {
    /// Reserved for lifecycle callbacks queued by a host-facing session API.
    /// No current operation produces this phase.
    DelayedEventCallbacks,
    ReportedEvents,
    RuntimeAdvance,
    ViewModelChanges,
    HostWork,
    Render,
}

/// Typed payload carried by a state-change output. View-model references are
/// output-only structural values: host scalar mutations continue to accept
/// [`FlowScalarValue`] and structural replacement uses [`FlowStateMutation::SetViewModel`].
#[derive(Debug, Clone, PartialEq)]
pub enum FlowStateChangeValue {
    Scalar(FlowScalarValue),
    ViewModelReference {
        instance_id: FlowInstanceId,
        schema_name: String,
    },
}

/// Renderer-neutral value emitted by the private Nuxie Luau host module.
/// Objects use a sorted map so equivalent script tables have one canonical
/// representation across the Rust and Apple seams.
#[derive(Debug, Clone, PartialEq)]
pub enum FlowHostValue {
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<FlowHostValue>),
    Object(BTreeMap<String, FlowHostValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowOutputPayload {
    ReportedEvent {
        name: Option<String>,
        event_type: u32,
        /// Present only for an authored OpenURL event. Paired with `target`;
        /// an empty string remains distinguishable from ordinary-event absence.
        url: Option<String>,
        /// Exact Rive target spelling (`_blank`, `_parent`, `_self`, `_top`,
        /// or empty for an unknown authored target value).
        target: Option<String>,
        /// Time between the authored event instant and the end of the runtime
        /// advance that produced it. This is overshoot metadata, not a future
        /// delivery deadline.
        delay_seconds: f32,
        properties: Vec<FlowEventProperty>,
    },
    StateChanged {
        instance_id: Option<FlowInstanceId>,
        path: String,
        value: Option<FlowStateChangeValue>,
        origin_mutation_id: Option<u64>,
    },
    HostCommand {
        name: String,
        payload: FlowHostValue,
    },
    RenderRequested {
        artboard_index: usize,
    },
    RuntimeAdvanced {
        delta_seconds: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowEventProperty {
    pub name: Option<String>,
    pub value: FlowScalarValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowOutput {
    pub sequence: u64,
    pub cycle: u64,
    pub phase: FlowOutputPhase,
    pub payload: FlowOutputPayload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowScalarValue {
    Null,
    String(String),
    Number(f32),
    Bool(bool),
    Enum(u64),
    ListIndex(u64),
    Color(u32),
    Image(u64),
    Trigger(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowInstanceRef {
    Existing(FlowInstanceId),
    New(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowNewInstance {
    pub local_id: u32,
    pub schema_name: String,
    pub authored_instance_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowStateMutation {
    SetInputBool {
        name: String,
        value: bool,
    },
    SetInputNumber {
        name: String,
        value: f32,
    },
    FireInputTrigger {
        name: String,
    },
    SetValue {
        instance: FlowInstanceRef,
        path: String,
        value: FlowScalarValue,
    },
    /// Replaces one outer nested view-model property with an existing shared
    /// instance. Inner property paths are intentionally unsupported.
    SetViewModel {
        instance: FlowInstanceRef,
        path: String,
        value: FlowInstanceRef,
    },
    FireTrigger {
        instance: FlowInstanceRef,
        path: String,
    },
    ListInsert {
        instance: FlowInstanceRef,
        path: String,
        index: usize,
        item: FlowInstanceRef,
    },
    ListRemove {
        instance: FlowInstanceRef,
        path: String,
        index: usize,
    },
    ListSwap {
        instance: FlowInstanceRef,
        path: String,
        first: usize,
        second: usize,
    },
    ListMove {
        instance: FlowInstanceRef,
        path: String,
        from: usize,
        to: usize,
    },
    ListSet {
        instance: FlowInstanceRef,
        path: String,
        index: usize,
        item: FlowInstanceRef,
    },
    ListClear {
        instance: FlowInstanceRef,
        path: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlowStateBatch {
    pub host_mutation_id: Option<u64>,
    pub mutations: Vec<FlowStateMutation>,
    pub new_instances: Vec<FlowNewInstance>,
}

/// One semantic write to a named `TextValueRun` on the root artboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowTextRunMutation {
    pub name: String,
    pub text: String,
}

/// One all-or-nothing set of root-artboard text-run writes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowTextRunBatch {
    pub mutations: Vec<FlowTextRunMutation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPointerKind {
    Down,
    Move,
    Up,
    Cancel,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowPointerEvent {
    pub kind: FlowPointerKind,
    pub pointer_id: i32,
    pub x: f32,
    pub y: f32,
    pub timestamp_seconds: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlowPointerBatch {
    pub events: Vec<FlowPointerEvent>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FlowAdvance {
    pub timestamp_seconds: f64,
    pub delta_seconds: f32,
    pub render: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowQuery {
    Bootstrap,
    Values,
    Catalog,
    PlayerInputs,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowInputSnapshot {
    pub name: Option<String>,
    pub kind: crate::StateMachineInputKind,
    pub value: FlowScalarValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowOperation {
    StateBatch(FlowStateBatch),
    PointerBatch(FlowPointerBatch),
    Advance(FlowAdvance),
    Query(FlowQuery),
    TextRunBatch(FlowTextRunBatch),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowCreatedInstance {
    pub local_id: u32,
    pub id: FlowInstanceId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowResult {
    pub outputs: Vec<FlowOutput>,
    pub dirty: bool,
    pub settled: bool,
    /// `Some(0)` requests an immediate follow-up operation to drain work
    /// produced by the preceding runtime advance. Future deadlines are not
    /// inferred from reported-event overshoot metadata.
    pub wake_after_seconds: Option<f32>,
    pub snapshot: Option<FlowBootstrap>,
    pub values: Option<FlowValueArena>,
    pub catalog: Option<FlowCatalog>,
    pub player_inputs: Option<Vec<FlowInputSnapshot>>,
    pub created_instances: Vec<FlowCreatedInstance>,
}

impl FlowResult {
    fn idle(settled: bool) -> Self {
        Self {
            outputs: Vec::new(),
            dirty: false,
            settled,
            wake_after_seconds: None,
            snapshot: None,
            values: None,
            catalog: None,
            player_inputs: None,
            created_instances: Vec::new(),
        }
    }
}

enum FlowPlayer {
    StateMachine(Box<StateMachineInstance>),
    Animation(LinearAnimationInstance),
    Static,
}

#[cfg(feature = "scripting")]
fn detached_view_model_snapshot(instance: Option<&ViewModelInstance>) -> Option<ViewModelInstance> {
    let instance = instance?;
    let raw = RuntimeOwnedViewModelHandle::detached_graph(std::slice::from_ref(instance.handle()))
        .into_iter()
        .next()?;
    Some(ViewModelInstance { raw })
}

/// Deep, renderer-neutral module owning one live flow.
pub struct FlowSession {
    artboard: OwnedArtboardInstance,
    player: FlowPlayer,
    instances: BTreeMap<FlowInstanceId, ViewModelInstance>,
    root_instance_id: Option<FlowInstanceId>,
    next_instance_id: u64,
    creation_bootstrap: FlowBootstrap,
    bootstrap: FlowBootstrap,
    next_sequence: u64,
    next_cycle: u64,
    last_timestamp_seconds: Option<f64>,
    pending_animation_events: Vec<StateMachineReportedEvent>,
    active_pointer_ids: BTreeSet<i32>,
    terminal_failure: Option<FlowSessionError>,
    #[cfg(feature = "scripting")]
    listener_binding_baseline: Option<ViewModelInstance>,
}

impl fmt::Debug for FlowSession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FlowSession")
            .field("artboard_index", &self.bootstrap.artboard_index)
            .field("player", &self.bootstrap.player)
            .finish_non_exhaustive()
    }
}

impl FlowSession {
    // `File` deliberately becomes !Send/!Sync when scripting is enabled: its
    // Luau VM is confined to the runtime worker. `Arc` is still the ownership
    // type used by artboard instances inside that one thread; it is never used
    // to move a scripted File between threads.
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn create(
        file: Arc<File>,
        config: FlowSessionConfig,
    ) -> Result<(Self, FlowBootstrap), FlowSessionError> {
        validate_optional_selector(config.artboard_name.as_deref(), "artboard name")?;
        if let Some(selector) = config.player.as_ref() {
            let (name, label) = match selector {
                FlowPlayerSelector::StateMachine(name) => (name.as_str(), "state machine name"),
                FlowPlayerSelector::LinearAnimation(name) => {
                    (name.as_str(), "linear animation name")
                }
            };
            validate_optional_selector(Some(name), label)?;
        }
        // A File owns lazy script-module registration and its VM. Cloning at
        // this deep-module boundary makes session isolation unconditional for
        // every host, not merely a convention that individual facades must
        // remember to uphold.
        let file = Arc::new(file.as_ref().clone());

        let artboard_index = match config.artboard_name.as_deref() {
            Some(name) => file
                .artboard_named(name)
                .map(|artboard| artboard.index())
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("artboard '{name}' was not found"),
                    )
                })?,
            None => file
                .default_artboard()
                .map(|artboard| artboard.index())
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        "file contains no artboards",
                    )
                })?,
        };

        let artboard = file.artboard(artboard_index).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "selected artboard disappeared",
            )
        })?;
        let artboard_name = artboard.name().map(ToOwned::to_owned);
        let mut instance = OwnedArtboardInstance::instantiate(Arc::clone(&file), artboard_index)
            .map_err(|error| {
                FlowSessionError::new(FlowSessionErrorKind::Runtime, error.to_string())
            })?;

        let root_view_model_selection = instance.view_model_index().map(|view_model_index| {
            let authored_index = file
                .runtime()
                .view_model_default_instance(view_model_index)
                .map(|default| default.instance_index);
            (view_model_index, authored_index)
        });
        let root_view_model = root_view_model_selection.and_then(|(_, authored_index)| {
            authored_index
                .and_then(|index| instance.instantiate_view_model_instance(index))
                .or_else(|| instance.instantiate_view_model())
        });
        if let Some(view_model) = root_view_model.as_ref() {
            let _ = instance.bind_view_model(view_model);
        }
        let (player_metadata, player) = select_player(artboard, &instance, config.player.as_ref())?;

        let (x, y, width, height) = instance.artboard_bounds();
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || !(x + width).is_finite()
            || !(y + height).is_finite()
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "artboard bounds are non-finite",
            ));
        }
        let catalog = build_catalog(&file, root_view_model_selection)?;
        let bootstrap = FlowBootstrap {
            artboard_index,
            artboard_name,
            player: player_metadata,
            bounds: FlowArtboardBounds {
                x,
                y,
                width,
                height,
            },
            catalog,
            values: FlowValueArena::default(),
        };
        let root_instance_id = bootstrap.catalog.root_instance_id;
        let mut instances = BTreeMap::new();
        if let (Some(id), Some(view_model)) = (root_instance_id, root_view_model) {
            instances.insert(id, view_model);
        }
        let mut session = Self {
            artboard: instance,
            player,
            instances,
            root_instance_id,
            next_instance_id: 2,
            creation_bootstrap: bootstrap.clone(),
            bootstrap: bootstrap.clone(),
            next_sequence: 1,
            next_cycle: 1,
            last_timestamp_seconds: None,
            pending_animation_events: Vec::new(),
            active_pointer_ids: BTreeSet::new(),
            terminal_failure: None,
            #[cfg(feature = "scripting")]
            listener_binding_baseline: None,
        };
        session.refresh_values()?;
        let bootstrap = session.bootstrap.clone();
        session.creation_bootstrap = bootstrap.clone();
        Ok((session, bootstrap))
    }

    /// Create a session in its stable renderer domain and synchronously
    /// bootstrap authenticated scripts. Any host work emitted by module load,
    /// protocol generation, or init is returned at cycle zero.
    pub fn create_with_factory(
        file: Arc<File>,
        config: FlowSessionConfig,
        factory: &mut dyn Factory,
    ) -> Result<(Self, FlowSessionCreation), FlowSessionError> {
        let (mut session, _) = Self::create(file, config)?;
        #[cfg(feature = "scripting")]
        let (outputs, dirty) = {
            let mut outputs = Vec::new();
            let mut dirty = false;
            dirty |= session
                .artboard
                .prepare_flow_scripts(factory)
                .map_err(|error| {
                    flow_script_error(error.with_context("script bootstrap failed"))
                })?;
            let root_view_model = session
                .root_instance_id
                .and_then(|id| session.instances.get(&id))
                .cloned();
            // Retain the exact source read by initial listener hydration. Init
            // callbacks can mutate the live root; those writes intentionally
            // remain pending for the first advance-time binding flush.
            let listener_binding_baseline = detached_view_model_snapshot(root_view_model.as_ref());
            if let FlowPlayer::StateMachine(machine) = &mut session.player {
                session
                    .artboard
                    .prepare_flow_listener_actions(machine, factory, root_view_model.as_ref())
                    .map_err(|error| {
                        flow_script_error(error.with_context("scripted listener bootstrap failed"))
                    })?;
                session.listener_binding_baseline = listener_binding_baseline;
            }
            let commands = session.artboard.drain_flow_host_commands();
            session.append_lua_host_commands(&mut outputs, 0, commands)?;
            (outputs, dirty)
        };
        #[cfg(not(feature = "scripting"))]
        let (outputs, dirty) = {
            let _ = factory;
            (Vec::new(), false)
        };
        session.refresh_values()?;
        let bootstrap = session.bootstrap.clone();
        session.creation_bootstrap = bootstrap.clone();
        let creation = FlowSessionCreation {
            bootstrap,
            outputs,
            dirty,
            settled: session.is_settled(),
            wake_after_seconds: None,
        };
        validate_creation_value_arena_bounds(&creation)?;
        Ok((session, creation))
    }

    /// Perform one bounded operation against the live flow.
    pub fn perform(&mut self, operation: FlowOperation) -> Result<FlowResult, FlowSessionError> {
        let mutates_runtime = !matches!(&operation, FlowOperation::Query(_));
        let result = self.perform_inner(operation, None)?;
        if let Err(error) = validate_result_value_arena_bounds(&result) {
            return if mutates_runtime {
                Err(self.poison_after_mutation(error))
            } else {
                Err(error)
            };
        }
        Ok(result)
    }

    /// Perform one bounded operation with the session's stable renderer
    /// factory available to authenticated script lifecycle hooks. Nuxie host
    /// effects commit only after the whole operation succeeds.
    pub fn perform_with_factory(
        &mut self,
        operation: FlowOperation,
        factory: &mut dyn Factory,
    ) -> Result<FlowResult, FlowSessionError> {
        let mutates_runtime = !matches!(&operation, FlowOperation::Query(_));
        #[cfg(feature = "scripting")]
        {
            if let Some(failure) = self.terminal_failure.as_ref() {
                return Err(failure.clone());
            }
            let checkpoint = self.artboard.begin_flow_host_cycle();
            let sequence_before = self.next_sequence;
            let operation_result = self.perform_inner(operation, Some(factory));
            return match operation_result {
                Ok(mut result) => {
                    let commands = self.artboard.drain_flow_host_commands();
                    if let Err(error) =
                        self.integrate_lua_host_commands(&mut result, sequence_before, commands)
                    {
                        Err(self.poison_after_mutation(error))
                    } else if let Err(error) = validate_result_value_arena_bounds(&result) {
                        if mutates_runtime {
                            Err(self.poison_after_mutation(error))
                        } else {
                            Err(error)
                        }
                    } else {
                        Ok(result)
                    }
                }
                Err(error) => {
                    if let Some(checkpoint) = checkpoint {
                        self.artboard.rollback_flow_host_cycle(checkpoint);
                    }
                    Err(error)
                }
            };
        }
        #[cfg(not(feature = "scripting"))]
        {
            let result = self.perform_inner(operation, Some(factory))?;
            if let Err(error) = validate_result_value_arena_bounds(&result) {
                return if mutates_runtime {
                    Err(self.poison_after_mutation(error))
                } else {
                    Err(error)
                };
            }
            Ok(result)
        }
    }

    fn perform_inner(
        &mut self,
        operation: FlowOperation,
        factory: Option<&mut dyn Factory>,
    ) -> Result<FlowResult, FlowSessionError> {
        if let Some(failure) = self.terminal_failure.as_ref() {
            return Err(failure.clone());
        }
        match operation {
            FlowOperation::Query(query) => {
                let mut result = FlowResult::idle(self.is_settled());
                match query {
                    FlowQuery::Bootstrap => result.snapshot = Some(self.creation_bootstrap.clone()),
                    FlowQuery::Values => result.values = Some(self.bootstrap.values.clone()),
                    FlowQuery::Catalog => result.catalog = Some(self.bootstrap.catalog.clone()),
                    FlowQuery::PlayerInputs => {
                        let inputs = match &self.player {
                            FlowPlayer::StateMachine(machine) => {
                                if machine.input_count() > MAX_BATCH_ITEMS {
                                    return Err(FlowSessionError::new(
                                        FlowSessionErrorKind::LimitExceeded,
                                        "player input query item limit exceeded",
                                    ));
                                }
                                (0..machine.input_count())
                                    .filter_map(|index| {
                                        let input = machine.input(index)?;
                                        let value = match input.kind() {
                                            crate::StateMachineInputKind::Bool => {
                                                FlowScalarValue::Bool(input.bool_value()?)
                                            }
                                            crate::StateMachineInputKind::Number => {
                                                FlowScalarValue::Number(input.number_value()?)
                                            }
                                            crate::StateMachineInputKind::Trigger => {
                                                FlowScalarValue::Bool(input.trigger_fired()?)
                                            }
                                        };
                                        Some(FlowInputSnapshot {
                                            name: input.name().map(ToOwned::to_owned),
                                            kind: input.kind(),
                                            value,
                                        })
                                    })
                                    .collect()
                            }
                            FlowPlayer::Animation(_) | FlowPlayer::Static => Vec::new(),
                        };
                        validate_player_input_snapshot(&inputs)?;
                        result.player_inputs = Some(inputs);
                    }
                }
                Ok(result)
            }
            FlowOperation::StateBatch(batch) => self.perform_state_batch(batch),
            FlowOperation::PointerBatch(batch) => self.perform_pointer_batch(batch, factory),
            FlowOperation::Advance(advance) => self.perform_advance(advance, factory),
            FlowOperation::TextRunBatch(batch) => self.perform_text_run_batch(batch),
        }
    }

    /// Draw the current settled session state through renderer-neutral traits.
    pub fn draw(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
    ) -> Result<(), FlowSessionError> {
        self.artboard
            .draw(factory, renderer)
            .map_err(flow_anyhow_error)
    }

    /// Drop renderer-owned members before switching this session to a
    /// replacement backend. The next draw rebuilds them from live state.
    pub fn reset_renderer(&self) {
        self.artboard.reset_renderer();
    }

    /// Draw the render requested by `result` before that operation result is
    /// exposed to the host. Script calls made while drawing remain in the same
    /// resource cycle and are inserted into HostWork before Render.
    pub fn draw_into_result(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
        result: &mut FlowResult,
    ) -> Result<(), FlowSessionError> {
        if let Some(failure) = self.terminal_failure.as_ref() {
            return Err(failure.clone());
        }
        if !result
            .outputs
            .iter()
            .any(|output| matches!(&output.payload, FlowOutputPayload::RenderRequested { .. }))
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::InvalidArgument,
                "draw requires the operation result that requested rendering",
            ));
        }
        #[cfg(feature = "scripting")]
        let sequence_before = result
            .outputs
            .first()
            .map(|output| output.sequence)
            .unwrap_or(self.next_sequence);
        let draw_result = self.draw(factory, renderer);
        #[cfg(feature = "scripting")]
        {
            if let Err(error) = draw_result {
                let _ = self.artboard.drain_flow_host_commands();
                return Err(self.poison_after_mutation(error));
            }
            let commands = self.artboard.drain_flow_host_commands();
            if let Err(error) = self.integrate_lua_host_commands(result, sequence_before, commands)
            {
                return Err(self.poison_after_mutation(error));
            }
            if let Err(error) = validate_result_value_arena_bounds(result) {
                return Err(self.poison_after_mutation(error));
            }
            Ok(())
        }
        #[cfg(not(feature = "scripting"))]
        {
            draw_result
        }
    }

    pub fn artboard_bounds(&self) -> FlowArtboardBounds {
        let (x, y, width, height) = self.artboard.artboard_bounds();
        FlowArtboardBounds {
            x,
            y,
            width,
            height,
        }
    }

    /// Whether the current player has no queued or continuing work.
    ///
    /// This is a read-only creation/result fact: it never advances the
    /// player or drains reported events.
    pub fn is_settled(&self) -> bool {
        let root_player_settled = match &self.player {
            FlowPlayer::StateMachine(machine) => {
                !machine.needs_advance() && machine.reported_event_count() == 0
            }
            FlowPlayer::Animation(animation) => {
                !self
                    .artboard
                    .raw()
                    .linear_animation_instance_keep_going(animation)
                    && self.pending_animation_events.is_empty()
            }
            FlowPlayer::Static => true,
        };
        root_player_settled && !self.artboard.raw().has_ongoing_nested_work()
    }

    fn has_pending_reports(&self) -> bool {
        match &self.player {
            FlowPlayer::StateMachine(machine) => machine.reported_event_count() > 0,
            FlowPlayer::Animation(_) => !self.pending_animation_events.is_empty(),
            FlowPlayer::Static => false,
        }
    }

    fn perform_state_batch(
        &mut self,
        batch: FlowStateBatch,
    ) -> Result<FlowResult, FlowSessionError> {
        let item_count = batch
            .mutations
            .len()
            .checked_add(batch.new_instances.len())
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "state batch item count overflow",
                )
            })?;
        if item_count > MAX_BATCH_ITEMS {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "state batch item limit exceeded",
            ));
        }
        if self
            .instances
            .len()
            .saturating_add(batch.new_instances.len())
            > MAX_INSTANCES
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "instance limit exceeded",
            ));
        }
        if state_batch_payload_bytes(&batch)? > MAX_ENCODED_PAYLOAD_BYTES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded state batch exceeds 4 MiB",
            ));
        }

        let mut new_ids = BTreeMap::new();
        let mut prepared_new = BTreeMap::new();
        let mut next_instance_id = self.next_instance_id;
        for new_instance in &batch.new_instances {
            validate_required_id_path(&new_instance.schema_name, "schema name")?;
            validate_optional_selector(
                new_instance.authored_instance_name.as_deref(),
                "authored instance name",
            )?;
            if new_ids.contains_key(&new_instance.local_id) {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Conflict,
                    "duplicate transaction-local instance id",
                ));
            }
            let id = FlowInstanceId(next_instance_id);
            next_instance_id = next_instance_id.checked_add(1).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "instance identity overflow",
                )
            })?;
            let view_model = instantiate_named_view_model(
                self.artboard.file(),
                &new_instance.schema_name,
                new_instance.authored_instance_name.as_deref(),
            )?;
            new_ids.insert(new_instance.local_id, id);
            prepared_new.insert(id, view_model);
        }

        let resolved = batch
            .mutations
            .iter()
            .map(|mutation| resolve_mutation(mutation, &new_ids))
            .collect::<Result<Vec<_>, _>>()?;

        for mutation in &resolved {
            for id in mutation_instance_ids(mutation) {
                if !prepared_new.contains_key(&id) && !self.instances.contains_key(&id) {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("instance {} was not found", id.get()),
                    ));
                }
            }
        }
        let graph_sources = self
            .instances
            .iter()
            .chain(prepared_new.iter())
            .map(|(id, instance)| (*id, instance.clone()))
            .collect::<Vec<_>>();
        let source_handles = graph_sources
            .iter()
            .map(|(_, instance)| instance.handle().clone())
            .collect::<Vec<_>>();
        let detached_handles = RuntimeOwnedViewModelHandle::detached_graph(&source_handles);
        let candidates = graph_sources
            .iter()
            .zip(detached_handles)
            .map(|((id, _), raw)| (*id, ViewModelInstance { raw }))
            .collect::<BTreeMap<_, _>>();

        let mut machine_candidate = match &self.player {
            FlowPlayer::StateMachine(machine) => Some(machine.clone()),
            FlowPlayer::Animation(_) | FlowPlayer::Static => None,
        };
        for mutation in &resolved {
            prevalidate_and_apply_mutation(
                machine_candidate.as_deref_mut(),
                &candidates,
                mutation,
            )?;
        }
        let cycle = self.next_cycle;
        let next_cycle = self.next_cycle.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "cycle counter overflow",
            )
        })?;
        let mut result = FlowResult::idle(self.is_settled());
        result.dirty = !resolved.is_empty() || !prepared_new.is_empty();
        result.created_instances = batch
            .new_instances
            .iter()
            .filter_map(|item| {
                new_ids.get(&item.local_id).map(|id| FlowCreatedInstance {
                    local_id: item.local_id,
                    id: *id,
                })
            })
            .collect();
        let mut candidate_catalog = self.bootstrap.catalog.clone();
        for created in &result.created_instances {
            let source = batch
                .new_instances
                .iter()
                .find(|item| item.local_id == created.local_id)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "created instance metadata disappeared",
                    )
                })?;
            candidate_catalog.instances.push(FlowInstanceMetadata {
                id: created.id,
                schema_name: source.schema_name.clone(),
                authored_name: source.authored_instance_name.clone(),
                is_root: false,
            });
        }
        let mut candidate_instances = candidates;
        let candidate_values = prepare_value_snapshot(
            self.artboard.file(),
            &mut candidate_instances,
            &mut candidate_catalog,
            &mut next_instance_id,
        )?;
        let mut candidate_bootstrap = self.bootstrap.clone();
        candidate_bootstrap.catalog = candidate_catalog;
        candidate_bootstrap.values = candidate_values;
        validate_bootstrap_payload(&candidate_bootstrap)?;

        let mut next_sequence = self.next_sequence;
        for mutation in &resolved {
            if let Some((instance_id, path, value)) =
                mutation_echo(mutation, &candidate_bootstrap.catalog)?
            {
                append_output(
                    &mut result.outputs,
                    &mut next_sequence,
                    cycle,
                    FlowOutputPhase::ViewModelChanges,
                    FlowOutputPayload::StateChanged {
                        instance_id,
                        path,
                        value,
                        origin_mutation_id: batch.host_mutation_id,
                    },
                )?;
            }
        }

        #[cfg(feature = "scripting")]
        let previous_listener_binding_baseline = self.listener_binding_baseline.clone();
        #[cfg(feature = "scripting")]
        let mut staged_listener_binding_baseline = None;
        if let (FlowPlayer::StateMachine(machine), Some(candidate)) =
            (&self.player, machine_candidate.as_mut())
        {
            candidate
                .adopt_scripted_listener_action_state_from(machine)
                .map_err(flow_script_error)?;
            #[cfg(feature = "scripting")]
            if let Some(previous_root) = previous_listener_binding_baseline.as_ref() {
                let candidate_root = self
                    .root_instance_id
                    .and_then(|id| candidate_instances.get(&id))
                    .cloned();
                // Stage the pre-callback candidate. Hydration callbacks may
                // write through Context.viewModel; committing this detached
                // source snapshot leaves those writes pending for the next
                // cycle instead of silently consuming them.
                staged_listener_binding_baseline =
                    detached_view_model_snapshot(candidate_root.as_ref());
                if let Err(error) = self.artboard.rehydrate_flow_listener_actions(
                    candidate,
                    candidate_root.as_ref(),
                    Some(previous_root),
                ) {
                    return Err(self.poison_after_mutation(flow_script_error(error)));
                }
            }
        }
        if let Some(root) = self
            .root_instance_id
            .and_then(|id| candidate_instances.get(&id))
        {
            let _ = self.artboard.bind_view_model(root);
            if let Some(machine) = machine_candidate.as_mut() {
                let _ = machine.bind_owned_view_model_handle(root.handle());
            }
        }
        if let (FlowPlayer::StateMachine(machine), Some(candidate)) =
            (&mut self.player, machine_candidate)
        {
            *machine = candidate;
        }
        self.instances = candidate_instances;
        self.bootstrap = candidate_bootstrap;
        self.next_instance_id = next_instance_id;
        self.next_cycle = next_cycle;
        self.next_sequence = next_sequence;
        #[cfg(feature = "scripting")]
        if previous_listener_binding_baseline.is_some() {
            self.listener_binding_baseline = staged_listener_binding_baseline;
        }
        result.settled = self.is_settled();
        include_reconciliation_values(&mut result, &self.bootstrap.values);
        Ok(result)
    }

    fn perform_text_run_batch(
        &mut self,
        batch: FlowTextRunBatch,
    ) -> Result<FlowResult, FlowSessionError> {
        if batch.mutations.len() > MAX_BATCH_ITEMS {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "text-run batch item limit exceeded",
            ));
        }

        let mut aggregate_text_bytes = 0_usize;
        let mut encoded_payload_bytes = 0_usize;
        for mutation in &batch.mutations {
            validate_required_text_run_name(&mutation.name)?;
            if mutation.text.len() > MAX_STRING_BYTES {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "text-run value exceeds 1 MiB",
                ));
            }
            checked_payload_add(&mut aggregate_text_bytes, mutation.text.len())?;
            checked_payload_add(&mut encoded_payload_bytes, mutation.name.len())?;
            checked_payload_add(&mut encoded_payload_bytes, mutation.text.len())?;
        }
        if aggregate_text_bytes > MAX_ENCODED_PAYLOAD_BYTES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "text-run batch text exceeds 4 MiB",
            ));
        }
        if encoded_payload_bytes > MAX_ENCODED_PAYLOAD_BYTES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded text-run batch exceeds 4 MiB",
            ));
        }

        for mutation in &batch.mutations {
            if !self.artboard.raw().has_root_text_value_run(&mutation.name) {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("root TextValueRun '{}' was not found", mutation.name),
                ));
            }
        }

        let mut changed = false;
        for mutation in batch.mutations {
            changed |= self
                .artboard
                .raw_mut()
                .set_root_text_value_run(&mutation.name, mutation.text.into_bytes())
                .unwrap_or(false);
        }
        let mut result = FlowResult::idle(self.is_settled());
        result.dirty = changed;
        result.wake_after_seconds = changed.then_some(0.0);
        Ok(result)
    }

    fn perform_pointer_batch(
        &mut self,
        batch: FlowPointerBatch,
        mut factory: Option<&mut dyn Factory>,
    ) -> Result<FlowResult, FlowSessionError> {
        if batch.events.len() > MAX_POINTERS_PER_BATCH {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                format!("pointer batch exceeds {MAX_POINTERS_PER_BATCH} events"),
            ));
        }
        let mut active_pointer_ids = self.active_pointer_ids.clone();
        for event in &batch.events {
            if event.pointer_id <= 0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "pointer ids must be positive",
                ));
            }
            if !event.x.is_finite() || !event.y.is_finite() {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "pointer coordinates must be finite",
                ));
            }
            if !event.timestamp_seconds.is_finite() || event.timestamp_seconds < 0.0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "pointer timestamps must be finite and nonnegative",
                ));
            }
            match event.kind {
                FlowPointerKind::Down | FlowPointerKind::Move => {
                    active_pointer_ids.insert(event.pointer_id);
                    if active_pointer_ids.len() > MAX_POINTERS_PER_BATCH {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::LimitExceeded,
                            format!("session exceeds {MAX_POINTERS_PER_BATCH} active pointers"),
                        ));
                    }
                }
                FlowPointerKind::Up | FlowPointerKind::Cancel | FlowPointerKind::Exit => {
                    active_pointer_ids.remove(&event.pointer_id);
                }
            }
        }

        let operation_result = (|| {
            let mut result = FlowResult::idle(self.is_settled());
            for event in batch.events {
                #[cfg(feature = "scripting")]
                let sequence_before = {
                    // One pointer batch is an atomic host operation, but each
                    // event advances the runtime independently. Reset script
                    // work budgets at that exact cycle boundary while the
                    // outer operation checkpoint remains responsible for
                    // rolling every effect back on failure.
                    let _ = self.artboard.begin_flow_host_cycle();
                    self.next_sequence
                };
                let before = self.bootstrap.values.clone();
                let changed = self.apply_pointer_event(event)?;
                result.dirty |= changed;
                let cycle_result = match event.kind {
                    FlowPointerKind::Down | FlowPointerKind::Up | FlowPointerKind::Cancel => {
                        self.run_player_cycle(0.0, false, None)?
                    }
                    FlowPointerKind::Move | FlowPointerKind::Exit => {
                        self.finish_nonadvance_pointer_cycle(before, changed)?
                    }
                };
                #[cfg(feature = "scripting")]
                let cycle_result = {
                    let mut cycle_result = cycle_result;
                    let commands = self.artboard.drain_flow_host_commands();
                    self.integrate_lua_host_commands(&mut cycle_result, sequence_before, commands)?;
                    cycle_result
                };
                merge_results(&mut result, cycle_result)?;
            }
            #[cfg(feature = "scripting")]
            if let Some(factory) = factory.take() {
                result.dirty |= self
                    .artboard
                    .prepare_flow_scripts(factory)
                    .map_err(|error| {
                        flow_script_error(error.with_context("scripted pointer cycle failed"))
                    })?;
            }
            #[cfg(not(feature = "scripting"))]
            let _ = factory.take();
            result.settled = self.is_settled();
            include_reconciliation_values(&mut result, &self.bootstrap.values);
            Ok(result)
        })();
        match operation_result {
            Ok(result) => {
                self.active_pointer_ids = active_pointer_ids;
                Ok(result)
            }
            Err(error) => Err(self.poison_after_mutation(error)),
        }
    }

    fn perform_advance(
        &mut self,
        advance: FlowAdvance,
        factory: Option<&mut dyn Factory>,
    ) -> Result<FlowResult, FlowSessionError> {
        if !advance.timestamp_seconds.is_finite() || !advance.delta_seconds.is_finite() {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::InvalidArgument,
                "advance timestamp and delta must be finite",
            ));
        }
        if advance.delta_seconds < 0.0 {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::InvalidArgument,
                "advance delta must not be negative",
            ));
        }
        if self
            .last_timestamp_seconds
            .is_some_and(|last| advance.timestamp_seconds < last)
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::Conflict,
                "advance timestamp must be nondecreasing",
            ));
        }
        self.last_timestamp_seconds = Some(advance.timestamp_seconds);
        let mut result = self
            .run_player_cycle(advance.delta_seconds, advance.render, factory)
            .map_err(|error| self.poison_after_mutation(error))?;
        include_reconciliation_values(&mut result, &self.bootstrap.values);
        Ok(result)
    }

    fn poison_after_mutation(&mut self, error: FlowSessionError) -> FlowSessionError {
        self.terminal_failure = Some(FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            format!("flow session is terminal after a failed mutation: {error}"),
        ));
        error
    }

    fn apply_pointer_event(&mut self, event: FlowPointerEvent) -> Result<bool, FlowSessionError> {
        let FlowPlayer::StateMachine(machine) = &mut self.player else {
            return Ok(false);
        };
        let mut host = NoopScriptHost;
        // Flow sessions bind the root handle onto the state machine when the
        // session is created or a StateBatch commits. Let each authored
        // ViewModelChange borrow that handle only for the individual action.
        // Holding one outer mutable borrow across the whole listener FIFO
        // would prevent a following Luau action from reading Context.viewModel.
        let changed = match event.kind {
            FlowPointerKind::Down => machine
                .try_pointer_down_with_timestamp_and_script_host(
                    self.artboard.raw(),
                    event.x,
                    event.y,
                    event.pointer_id,
                    event.timestamp_seconds,
                    &mut host,
                )
                .map_err(flow_script_error)?,
            FlowPointerKind::Move => machine
                .try_pointer_move_with_timestamp_and_script_host(
                    self.artboard.raw(),
                    event.x,
                    event.y,
                    event.pointer_id,
                    event.timestamp_seconds,
                    &mut host,
                )
                .map_err(flow_script_error)?,
            FlowPointerKind::Up | FlowPointerKind::Cancel => {
                let mut changed = machine
                    .try_pointer_up_with_timestamp_and_script_host(
                        self.artboard.raw(),
                        event.x,
                        event.y,
                        event.pointer_id,
                        event.timestamp_seconds,
                        &mut host,
                    )
                    .map_err(flow_script_error)?;
                changed |= machine
                    .try_pointer_exit_with_timestamp_and_script_host(
                        self.artboard.raw(),
                        event.x,
                        event.y,
                        event.pointer_id,
                        event.timestamp_seconds,
                        &mut host,
                    )
                    .map_err(flow_script_error)?;
                changed
            }
            FlowPointerKind::Exit => machine
                .try_pointer_exit_with_timestamp_and_script_host(
                    self.artboard.raw(),
                    event.x,
                    event.y,
                    event.pointer_id,
                    event.timestamp_seconds,
                    &mut host,
                )
                .map_err(flow_script_error)?,
        };
        Ok(changed)
    }

    fn run_player_cycle(
        &mut self,
        delta_seconds: f32,
        render: bool,
        mut factory: Option<&mut dyn Factory>,
    ) -> Result<FlowResult, FlowSessionError> {
        let before_values = self.bootstrap.values.clone();
        let cycle = self.next_cycle;
        self.next_cycle = self.next_cycle.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "cycle counter overflow",
            )
        })?;

        // Pointer callbacks have already completed when they enter this
        // method. Flush retained listener bindings once at this boundary,
        // before runtime advance; never interleave a refresh into the authored
        // FIFO of listener actions and never refresh again after advance.
        #[cfg(feature = "scripting")]
        self.flush_flow_listener_bindings()?;

        // Pointer callbacks can report events synchronously before the
        // ordinary new-frame advance. Capture only the host-visible suffix
        // now; the runtime keeps its independent listener cursor so C++
        // `applyEvents` semantics still consume this queue during advance.
        let mut pending_events = match &mut self.player {
            FlowPlayer::StateMachine(machine) => machine.take_reported_events(),
            FlowPlayer::Animation(_) => std::mem::take(&mut self.pending_animation_events),
            FlowPlayer::Static => Vec::new(),
        };
        let changed = match &mut self.player {
            FlowPlayer::StateMachine(machine) => {
                let changed = if let Some(factory) = factory.as_deref_mut() {
                    self.artboard
                        .try_advance_with_state_machine_and_factory(machine, delta_seconds, factory)
                        .map_err(flow_anyhow_error)?
                } else {
                    self.artboard
                        .advance_with_state_machine(machine, delta_seconds)
                };
                if let Some(error) = machine.script_error().cloned() {
                    return Err(flow_script_error(error));
                }
                changed
            }
            FlowPlayer::Animation(animation) => {
                let mut events = Vec::new();
                let mut changed = self
                    .artboard
                    .raw_mut()
                    .advance_linear_animation_instance_with_events(
                        animation,
                        delta_seconds,
                        &mut events,
                    );
                changed |= self
                    .artboard
                    .raw_mut()
                    .apply_linear_animation_instance(animation, 1.0);
                changed |= if let Some(factory) = factory.as_deref_mut() {
                    self.artboard
                        .try_advance_with_factory(factory, 0.0)
                        .map_err(flow_anyhow_error)?
                } else {
                    self.artboard.advance(0.0)
                };
                self.pending_animation_events = events;
                changed
            }
            FlowPlayer::Static => {
                if let Some(factory) = factory.as_deref_mut() {
                    self.artboard
                        .try_advance_with_factory(factory, delta_seconds)
                        .map_err(flow_anyhow_error)?
                } else {
                    self.artboard.advance(delta_seconds)
                }
            }
        };

        // Match C++ `StateMachineInstance::advance`: listener work retained
        // from the prior frame is applied at the start of advance, while
        // reports produced by this advance are visible when it returns.
        // Reference: rive-runtime
        // `d788e8ec6e8b598526607d6a1e8818e8b637b60c`,
        // `src/animation/state_machine_instance.cpp:2320-2335,2546-2584`.
        let produced_events = match &mut self.player {
            FlowPlayer::StateMachine(machine) => machine.take_reported_events(),
            FlowPlayer::Animation(_) => std::mem::take(&mut self.pending_animation_events),
            FlowPlayer::Static => Vec::new(),
        };
        let event_count = pending_events
            .len()
            .checked_add(produced_events.len())
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "runtime event count overflow",
                )
            })?;
        if event_count > MAX_BATCH_ITEMS {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                format!("runtime emitted more than {MAX_BATCH_ITEMS} events"),
            ));
        }
        pending_events.extend(produced_events);

        let mut reported_payloads = Vec::with_capacity(event_count);
        for event in pending_events {
            if !event.seconds_delay().is_finite() || event.seconds_delay() < 0.0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "runtime emitted an event with an invalid delay",
                ));
            }
            reported_payloads.push(self.event_payload(event)?);
        }

        let mut result = FlowResult::idle(false);
        result.dirty = changed;
        for payload in reported_payloads {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::ReportedEvents,
                payload,
            )?;
        }
        self.push_output(
            &mut result.outputs,
            cycle,
            FlowOutputPhase::RuntimeAdvance,
            FlowOutputPayload::RuntimeAdvanced { delta_seconds },
        )?;
        self.refresh_values()?;
        let value_changes = diff_value_arenas(
            &before_values,
            &self.bootstrap.values,
            &self.bootstrap.catalog,
        )?;
        result.dirty |= !value_changes.is_empty();
        for (instance_id, path, value) in value_changes {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::ViewModelChanges,
                FlowOutputPayload::StateChanged {
                    instance_id: Some(instance_id),
                    path,
                    value,
                    origin_mutation_id: None,
                },
            )?;
        }
        if render {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::Render,
                FlowOutputPayload::RenderRequested {
                    artboard_index: self.bootstrap.artboard_index,
                },
            )?;
        }
        let has_pending_reports = self.has_pending_reports();
        result.wake_after_seconds = has_pending_reports.then_some(0.0);
        result.settled = self.is_settled();
        Ok(result)
    }

    #[cfg(feature = "scripting")]
    fn flush_flow_listener_bindings(&mut self) -> Result<(), FlowSessionError> {
        let Some(previous_root) = self.listener_binding_baseline.clone() else {
            return Ok(());
        };
        let FlowPlayer::StateMachine(machine) = &mut self.player else {
            return Ok(());
        };
        let current_root = self
            .root_instance_id
            .and_then(|id| self.instances.get(&id))
            .cloned();
        // Snapshot before applying hydration: a bound trigger callback may
        // mutate the live Context.viewModel. Advancing to this pre-callback
        // source only after success preserves those writes for the next cycle.
        let next_baseline = detached_view_model_snapshot(current_root.as_ref());
        self.artboard
            .rehydrate_flow_listener_actions(machine, current_root.as_ref(), Some(&previous_root))
            .map_err(|error| {
                flow_script_error(error.with_context("scripted listener binding flush failed"))
            })?;
        self.listener_binding_baseline = next_baseline;
        Ok(())
    }

    fn finish_nonadvance_pointer_cycle(
        &mut self,
        before_values: FlowValueArena,
        changed: bool,
    ) -> Result<FlowResult, FlowSessionError> {
        let cycle = self.next_cycle;
        self.next_cycle = self.next_cycle.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "cycle counter overflow",
            )
        })?;
        self.refresh_values()?;
        let changes = diff_value_arenas(
            &before_values,
            &self.bootstrap.values,
            &self.bootstrap.catalog,
        )?;
        let mut result = FlowResult::idle(self.is_settled());
        result.dirty = changed || !changes.is_empty();
        for (instance_id, path, value) in changes {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::ViewModelChanges,
                FlowOutputPayload::StateChanged {
                    instance_id: Some(instance_id),
                    path,
                    value,
                    origin_mutation_id: None,
                },
            )?;
        }
        result.wake_after_seconds = self.has_pending_reports().then_some(0.0);
        result.settled = self.is_settled();
        Ok(result)
    }

    fn event_payload(
        &self,
        event: StateMachineReportedEvent,
    ) -> Result<FlowOutputPayload, FlowSessionError> {
        let runtime_properties = self
            .artboard
            .raw()
            .event_properties(event.event_local_index());
        if runtime_properties.len() > MAX_EVENT_PROPERTIES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "event property limit exceeded",
            ));
        }
        let properties: Vec<FlowEventProperty> = runtime_properties
            .into_iter()
            .map(|property| FlowEventProperty {
                name: property.name,
                value: match property.value {
                    RuntimeEventPropertyValue::Number(value) => FlowScalarValue::Number(value),
                    RuntimeEventPropertyValue::Bool(value) => FlowScalarValue::Bool(value),
                    RuntimeEventPropertyValue::String(value) => {
                        FlowScalarValue::String(String::from_utf8_lossy(&value).into_owned())
                    }
                    RuntimeEventPropertyValue::Color(value) => FlowScalarValue::Color(value),
                    RuntimeEventPropertyValue::Enum(value) => FlowScalarValue::Enum(value),
                    RuntimeEventPropertyValue::Trigger(value) => FlowScalarValue::Trigger(value),
                },
            })
            .collect();
        let name = event.name().map(ToOwned::to_owned);
        let event_type = event.event_core_type();
        let url = event.url().map(ToOwned::to_owned);
        let target = event.target().map(ToOwned::to_owned);
        let delay_seconds = event.seconds_delay();
        let payload = FlowOutputPayload::ReportedEvent {
            name,
            event_type,
            url,
            target,
            delay_seconds,
            properties,
        };
        validate_output_payload(&payload)?;
        Ok(payload)
    }

    fn push_output(
        &mut self,
        outputs: &mut Vec<FlowOutput>,
        cycle: u64,
        phase: FlowOutputPhase,
        payload: FlowOutputPayload,
    ) -> Result<(), FlowSessionError> {
        append_output(outputs, &mut self.next_sequence, cycle, phase, payload)
    }

    #[cfg(feature = "scripting")]
    fn append_lua_host_commands(
        &mut self,
        outputs: &mut Vec<FlowOutput>,
        cycle: u64,
        commands: Vec<LuaHostCommand>,
    ) -> Result<(), FlowSessionError> {
        for command in commands {
            let (name, payload) = match command {
                LuaHostCommand::Trigger { name, properties } => (
                    name,
                    FlowHostValue::Object(
                        properties
                            .into_iter()
                            .map(|(key, value)| (key, flow_host_value(value)))
                            .collect(),
                    ),
                ),
                LuaHostCommand::ResponseSet { field, value } => (
                    "$response_set".to_owned(),
                    FlowHostValue::Object(BTreeMap::from([
                        ("field".to_owned(), FlowHostValue::String(field)),
                        ("value".to_owned(), flow_host_value(value)),
                    ])),
                ),
            };
            self.push_output(
                outputs,
                cycle,
                FlowOutputPhase::HostWork,
                FlowOutputPayload::HostCommand { name, payload },
            )?;
        }
        Ok(())
    }

    #[cfg(feature = "scripting")]
    fn integrate_lua_host_commands(
        &mut self,
        result: &mut FlowResult,
        sequence_before: u64,
        commands: Vec<LuaHostCommand>,
    ) -> Result<(), FlowSessionError> {
        if commands.is_empty() {
            return Ok(());
        }
        let cycle = result
            .outputs
            .last()
            .map(|output| output.cycle)
            .unwrap_or_else(|| self.next_cycle.saturating_sub(1).max(1));
        let mut host_outputs = Vec::with_capacity(commands.len());
        self.append_lua_host_commands(&mut host_outputs, cycle, commands)?;
        let insertion = result
            .outputs
            .iter()
            .position(|output| {
                output.cycle > cycle
                    || (output.cycle == cycle && output.phase > FlowOutputPhase::HostWork)
            })
            .unwrap_or(result.outputs.len());
        result.outputs.splice(insertion..insertion, host_outputs);
        validate_output_batch(&result.outputs)?;
        let next_sequence = sequence_before
            .checked_add(u64::try_from(result.outputs.len()).map_err(|_| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "output sequence count overflow",
                )
            })?)
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "output sequence overflow",
                )
            })?;
        for (offset, output) in result.outputs.iter_mut().enumerate() {
            output.sequence = sequence_before
                .checked_add(u64::try_from(offset).map_err(|_| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "output sequence offset overflow",
                    )
                })?)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "output sequence overflow",
                    )
                })?;
        }
        self.next_sequence = next_sequence;
        Ok(())
    }

    fn refresh_values(&mut self) -> Result<(), FlowSessionError> {
        self.bootstrap.values = prepare_value_snapshot(
            self.artboard.file(),
            &mut self.instances,
            &mut self.bootstrap.catalog,
            &mut self.next_instance_id,
        )?;
        validate_bootstrap_payload(&self.bootstrap)?;
        Ok(())
    }
}

#[cfg(feature = "scripting")]
fn flow_host_value(value: LuaHostValue) -> FlowHostValue {
    match value {
        LuaHostValue::Bool(value) => FlowHostValue::Bool(value),
        LuaHostValue::Number(value) => FlowHostValue::Number(value),
        LuaHostValue::String(value) => FlowHostValue::String(value),
        LuaHostValue::Array(values) => {
            FlowHostValue::List(values.into_iter().map(flow_host_value).collect())
        }
        LuaHostValue::Object(values) => FlowHostValue::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, flow_host_value(value)))
                .collect(),
        ),
    }
}

fn prepare_value_snapshot(
    file: &File,
    instances: &mut BTreeMap<FlowInstanceId, ViewModelInstance>,
    catalog: &mut FlowCatalog,
    next_instance_id: &mut u64,
) -> Result<FlowValueArena, FlowSessionError> {
    let mut discovered = Vec::new();
    let mut traversed_edges = 0_usize;
    for instance in instances.values() {
        collect_reachable_instances(
            file,
            instance.handle(),
            "",
            0,
            &mut discovered,
            &mut traversed_edges,
        )?;
    }
    for handle in discovered {
        if instances
            .values()
            .any(|instance| instance.handle().ptr_eq(&handle))
        {
            continue;
        }
        if instances.len() >= MAX_INSTANCES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "instance limit exceeded",
            ));
        }
        let id = FlowInstanceId(*next_instance_id);
        *next_instance_id = next_instance_id.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "instance identity overflow",
            )
        })?;
        let view_model_index = handle.borrow().view_model_index();
        let schema_name = file
            .graph()
            .view_models
            .get(view_model_index)
            .and_then(|schema| schema.name.clone())
            .unwrap_or_else(|| format!("viewModel{view_model_index}"));
        catalog.instances.push(FlowInstanceMetadata {
            id,
            schema_name,
            authored_name: None,
            is_root: false,
        });
        instances.insert(id, ViewModelInstance { raw: handle });
    }
    validate_catalog(catalog)?;
    let mut builder = ValueArenaBuilder::new(file);
    for (id, instance) in instances {
        let root = builder.snapshot_handle(instance.handle(), "", 0)?;
        builder.arena.roots.push((*id, root));
    }
    Ok(builder.arena)
}

fn collect_reachable_instances(
    file: &File,
    handle: &RuntimeOwnedViewModelHandle,
    prefix: &str,
    depth: usize,
    discovered: &mut Vec<RuntimeOwnedViewModelHandle>,
    traversed_edges: &mut usize,
) -> Result<(), FlowSessionError> {
    if depth > MAX_VALUE_DEPTH {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "view-model graph depth limit exceeded",
        ));
    }
    let view_model_index = if prefix.is_empty() {
        handle.borrow().view_model_index()
    } else {
        let root_index = handle.borrow().view_model_index();
        resolve_nested_schema_index(file, root_index, prefix)?
    };
    let schema = file.runtime().view_model(view_model_index).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            "retained view-model schema disappeared",
        )
    })?;
    for property in schema.properties {
        let Some(name) = property.string_property("name") else {
            continue;
        };
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        match property.type_name {
            "ViewModelPropertyList" => {
                let item_count = handle
                    .list_item_count_by_property_name_path(&path)
                    .unwrap_or(0);
                if item_count > MAX_LIST_ITEMS {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable list item limit exceeded",
                    ));
                }
                *traversed_edges = traversed_edges.checked_add(item_count).ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge counter overflow",
                    )
                })?;
                if *traversed_edges > MAX_VALUE_EDGES {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge limit exceeded",
                    ));
                }
                for item in handle
                    .list_items_by_property_name_path(&path)
                    .unwrap_or_default()
                    .into_iter()
                    .take(item_count)
                {
                    if !discovered.iter().any(|existing| existing.ptr_eq(&item)) {
                        if discovered.len() >= MAX_INSTANCES {
                            return Err(FlowSessionError::new(
                                FlowSessionErrorKind::LimitExceeded,
                                "reachable instance limit exceeded",
                            ));
                        }
                        discovered.push(item.clone());
                        collect_reachable_instances(
                            file,
                            &item,
                            "",
                            depth.saturating_add(1),
                            discovered,
                            traversed_edges,
                        )?;
                    }
                }
            }
            "ViewModelPropertyViewModel" => {
                *traversed_edges = traversed_edges.checked_add(1).ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge counter overflow",
                    )
                })?;
                if *traversed_edges > MAX_VALUE_EDGES {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge limit exceeded",
                    ));
                }
                if let Some(linked) = handle.linked_view_model_by_property_name_path(&path) {
                    if !discovered.iter().any(|existing| existing.ptr_eq(&linked)) {
                        if discovered.len() >= MAX_INSTANCES {
                            return Err(FlowSessionError::new(
                                FlowSessionErrorKind::LimitExceeded,
                                "reachable instance limit exceeded",
                            ));
                        }
                        discovered.push(linked.clone());
                        collect_reachable_instances(
                            file,
                            &linked,
                            "",
                            depth.saturating_add(1),
                            discovered,
                            traversed_edges,
                        )?;
                    }
                } else {
                    collect_reachable_instances(
                        file,
                        handle,
                        &path,
                        depth.saturating_add(1),
                        discovered,
                        traversed_edges,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn resolve_nested_schema_index(
    file: &File,
    root_schema_index: usize,
    prefix: &str,
) -> Result<usize, FlowSessionError> {
    let mut schema_index = root_schema_index;
    for segment in prefix.split('/') {
        let schema = file.runtime().view_model(schema_index).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "nested view-model schema disappeared",
            )
        })?;
        let property = schema
            .properties
            .into_iter()
            .find(|property| property.string_property("name") == Some(segment))
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "nested view-model path disappeared",
                )
            })?;
        schema_index = usize::try_from(property.uint_property("viewModelReferenceId").ok_or_else(
            || {
                FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "nested view-model property has no schema reference id",
                )
            },
        )?)
        .map_err(|_| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "nested schema id is out of range",
            )
        })?;
    }
    Ok(schema_index)
}

struct ValueArenaBuilder<'a> {
    file: &'a File,
    arena: FlowValueArena,
    edge_count: usize,
    payload_bytes: usize,
    instance_nodes: Vec<(RuntimeOwnedViewModelHandle, FlowValueId)>,
}

impl<'a> ValueArenaBuilder<'a> {
    fn new(file: &'a File) -> Self {
        Self {
            file,
            arena: FlowValueArena::default(),
            edge_count: 0,
            payload_bytes: 0,
            instance_nodes: Vec::new(),
        }
    }

    fn push(&mut self, value: FlowValue) -> Result<FlowValueId, FlowSessionError> {
        if self.arena.nodes.len() >= MAX_VALUE_NODES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena node limit exceeded",
            ));
        }
        match &value {
            FlowValue::String(value) if value.len() > MAX_STRING_BYTES => {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "string value exceeds 1 MiB",
                ));
            }
            FlowValue::Number(value) if !value.is_finite() => {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "runtime produced a non-finite number value",
                ));
            }
            _ => {}
        }
        self.add_payload(16)?;
        let index = u32::try_from(self.arena.nodes.len()).map_err(|_| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena identity overflow",
            )
        })?;
        let id = FlowValueId(index.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena identity overflow",
            )
        })?);
        self.arena.nodes.push(FlowValueNode { id, value });
        Ok(id)
    }

    fn add_payload(&mut self, count: usize) -> Result<(), FlowSessionError> {
        self.payload_bytes = self.payload_bytes.checked_add(count).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded value arena size overflow",
            )
        })?;
        if self.payload_bytes > MAX_ENCODED_PAYLOAD_BYTES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded value arena exceeds 4 MiB",
            ));
        }
        Ok(())
    }

    fn add_edges(&mut self, count: usize) -> Result<(), FlowSessionError> {
        self.edge_count = self.edge_count.checked_add(count).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value edge counter overflow",
            )
        })?;
        if self.edge_count > MAX_VALUE_EDGES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena edge limit exceeded",
            ));
        }
        Ok(())
    }

    fn snapshot_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelHandle,
        prefix: &str,
        depth: usize,
    ) -> Result<FlowValueId, FlowSessionError> {
        if depth > MAX_VALUE_DEPTH {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena depth limit exceeded",
            ));
        }
        if prefix.is_empty()
            && let Some((_, id)) = self
                .instance_nodes
                .iter()
                .find(|(existing, _)| existing.ptr_eq(handle))
        {
            return Ok(*id);
        }
        let reserved = if prefix.is_empty() {
            let id = self.push(FlowValue::Null)?;
            self.instance_nodes.push((handle.clone(), id));
            Some(id)
        } else {
            None
        };
        let view_model_index = handle.borrow().view_model_index();
        let view_model = self
            .file
            .runtime()
            .view_model(view_model_index)
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "retained view-model schema disappeared",
                )
            })?;
        let mut edges = Vec::new();
        for property in view_model.properties {
            let Some(name) = property.string_property("name") else {
                continue;
            };
            let path = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix}/{name}")
            };
            let child = self.snapshot_property(handle, property, &path, depth.saturating_add(1))?;
            edges.push((name.to_owned(), child));
            self.add_payload(name.len())?;
        }
        self.add_edges(edges.len())?;
        if let Some(id) = reserved {
            let node = self
                .arena
                .nodes
                .iter_mut()
                .find(|node| node.id == id)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "reserved value node disappeared",
                    )
                })?;
            node.value = FlowValue::ViewModel(edges);
            Ok(id)
        } else {
            self.push(FlowValue::ViewModel(edges))
        }
    }

    fn snapshot_property(
        &mut self,
        handle: &RuntimeOwnedViewModelHandle,
        property: &nuxie_binary::RuntimeObject,
        path: &str,
        depth: usize,
    ) -> Result<FlowValueId, FlowSessionError> {
        validate_required_id_path(path, "value path")?;
        if depth > MAX_VALUE_DEPTH {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena depth limit exceeded",
            ));
        }
        let raw = handle.borrow();
        let scalar = match property.type_name {
            "ViewModelPropertyString" => raw
                .string_value_by_property_name_path(path)
                .map(|bytes| {
                    if bytes.len() > MAX_STRING_BYTES {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::LimitExceeded,
                            "string value exceeds 1 MiB",
                        ));
                    }
                    String::from_utf8(bytes.to_vec())
                        .map(FlowValue::String)
                        .map_err(|_| {
                            FlowSessionError::new(
                                FlowSessionErrorKind::Runtime,
                                format!("string property '{path}' is not UTF-8"),
                            )
                        })
                })
                .transpose()?,
            "ViewModelPropertyNumber" | "ViewModelPropertyInteger" => raw
                .number_value_by_property_name_path(path)
                .map(FlowValue::Number),
            "ViewModelPropertyBoolean" => raw
                .boolean_value_by_property_name_path(path)
                .map(FlowValue::Bool),
            "ViewModelPropertyColor" => raw
                .color_value_by_property_name_path(path)
                .map(FlowValue::Color),
            "ViewModelPropertyEnum"
            | "ViewModelPropertyEnumCustom"
            | "ViewModelPropertyEnumSystem" => raw
                .enum_value_by_property_name_path(path)
                .map(FlowValue::Enum),
            "ViewModelPropertySymbolListIndex" => raw
                .symbol_list_index_value_by_property_name_path(path)
                .map(FlowValue::ListIndex),
            "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => raw
                .asset_value_by_property_name_path(path)
                .map(FlowValue::Image),
            "ViewModelPropertyArtboard" => raw
                .artboard_value_by_property_name_path(path)
                .map(FlowValue::Enum),
            "ViewModelPropertyTrigger" => raw
                .trigger_value_by_property_name_path(path)
                .map(FlowValue::Enum),
            _ => None,
        };
        drop(raw);
        if let Some(value) = scalar {
            if let FlowValue::String(value) = &value {
                self.add_payload(value.len())?;
            }
            return self.push(value);
        }

        match property.type_name {
            "ViewModelPropertyList" => {
                let count = handle
                    .list_item_count_by_property_name_path(path)
                    .ok_or_else(|| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            format!("list property '{path}' could not be snapshotted"),
                        )
                    })?;
                if count > MAX_LIST_ITEMS {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "list item limit exceeded",
                    ));
                }
                let items = handle
                    .list_items_by_property_name_path(path)
                    .unwrap_or_default();
                let mut item_nodes = Vec::with_capacity(count);
                for item in items.iter().take(count) {
                    item_nodes.push(self.snapshot_handle(item, "", depth.saturating_add(1))?);
                }
                while item_nodes.len() < count {
                    item_nodes.push(self.push(FlowValue::Null)?);
                }
                self.add_edges(item_nodes.len())?;
                self.push(FlowValue::List(item_nodes))
            }
            "ViewModelPropertyViewModel" => {
                if let Some(linked) = handle.linked_view_model_by_property_name_path(path) {
                    return self.snapshot_handle(&linked, "", depth.saturating_add(1));
                }
                let referenced_index =
                    usize::try_from(property.uint_property("viewModelReferenceId").ok_or_else(
                        || {
                            FlowSessionError::new(
                                FlowSessionErrorKind::Runtime,
                                "nested view-model property has no schema reference id",
                            )
                        },
                    )?)
                    .map_err(|_| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            "nested view-model schema id is out of range",
                        )
                    })?;
                let nested = self
                    .file
                    .runtime()
                    .view_model(referenced_index)
                    .ok_or_else(|| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            "nested view-model schema was not found",
                        )
                    })?;
                let mut edges = Vec::new();
                for child_property in nested.properties {
                    let Some(name) = child_property.string_property("name") else {
                        continue;
                    };
                    let child_path = format!("{path}/{name}");
                    let child = self.snapshot_property(
                        handle,
                        child_property,
                        &child_path,
                        depth.saturating_add(1),
                    )?;
                    edges.push((name.to_owned(), child));
                    self.add_payload(name.len())?;
                }
                self.add_edges(edges.len())?;
                self.push(FlowValue::ViewModel(edges))
            }
            _ => self.push(FlowValue::Null),
        }
    }
}

#[derive(Default)]
struct ResultValueArenaProjection {
    nodes: usize,
    edges: usize,
    content_bytes: usize,
}

impl ResultValueArenaProjection {
    fn add_nodes(&mut self, count: usize) -> Result<(), FlowSessionError> {
        self.nodes = self.nodes.checked_add(count).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::ResultLimitExceeded,
                "result value arena node count overflowed",
            )
        })?;
        if self.nodes > MAX_VALUE_NODES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::ResultLimitExceeded,
                "result value arena exceeds 4096 nodes",
            ));
        }
        Ok(())
    }

    fn add_edges(&mut self, count: usize) -> Result<(), FlowSessionError> {
        self.edges = self.edges.checked_add(count).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::ResultLimitExceeded,
                "result value arena edge count overflowed",
            )
        })?;
        if self.edges > MAX_VALUE_EDGES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::ResultLimitExceeded,
                "result value arena exceeds 16384 edges",
            ));
        }
        Ok(())
    }

    fn add_content_bytes(&mut self, count: usize) -> Result<(), FlowSessionError> {
        self.content_bytes = self.content_bytes.checked_add(count).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::ResultLimitExceeded,
                "result ABI content size overflowed",
            )
        })?;
        if self.content_bytes > MAX_ENCODED_PAYLOAD_BYTES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::ResultLimitExceeded,
                "result ABI content exceeds 4 MiB",
            ));
        }
        Ok(())
    }

    fn add_catalog(&mut self, catalog: &FlowCatalog) -> Result<(), FlowSessionError> {
        for schema in &catalog.schemas {
            self.add_content_bytes(schema.name.len().saturating_mul(2))?;
            for property in &schema.properties {
                self.add_content_bytes(schema.name.len())?;
                self.add_content_bytes(property.name.len().saturating_mul(2))?;
                self.add_content_bytes(
                    property
                        .referenced_schema_name
                        .as_deref()
                        .map(str::len)
                        .unwrap_or(0),
                )?;
                for label in &property.enum_labels {
                    self.add_content_bytes(label.len())?;
                }
            }
        }
        for template in &catalog.templates {
            self.add_content_bytes(template.schema_name.len())?;
            self.add_content_bytes(template.authored_name.as_deref().map(str::len).unwrap_or(0))?;
        }
        for instance in &catalog.instances {
            self.add_content_bytes(instance.schema_name.len())?;
            self.add_content_bytes(instance.authored_name.as_deref().map(str::len).unwrap_or(0))?;
        }
        Ok(())
    }

    fn add_flow_arena(
        &mut self,
        arena: &FlowValueArena,
        catalog: Option<&FlowCatalog>,
    ) -> Result<(), FlowSessionError> {
        self.add_nodes(arena.nodes.len())?;
        for node in &arena.nodes {
            match &node.value {
                FlowValue::String(value) => self.add_content_bytes(value.len())?,
                FlowValue::Object(children) | FlowValue::ViewModel(children) => {
                    self.add_edges(children.len())?;
                    for (key, _) in children {
                        self.add_content_bytes(key.len())?;
                    }
                }
                FlowValue::List(children) => self.add_edges(children.len())?,
                FlowValue::Null
                | FlowValue::Number(_)
                | FlowValue::Bool(_)
                | FlowValue::Enum(_)
                | FlowValue::ListIndex(_)
                | FlowValue::Color(_)
                | FlowValue::Image(_) => {}
            }
        }
        if let Some(catalog) = catalog {
            let view_model_nodes = arena
                .nodes
                .iter()
                .filter_map(|node| {
                    matches!(&node.value, FlowValue::ViewModel(_)).then_some(node.id.get())
                })
                .collect::<BTreeSet<_>>();
            let mut root_schema_lengths = BTreeMap::new();
            for (instance_id, root_id) in &arena.roots {
                if !view_model_nodes.contains(&root_id.get()) {
                    continue;
                }
                let schema_length = catalog
                    .instances
                    .iter()
                    .find(|instance| instance.id == *instance_id)
                    .map(|instance| instance.schema_name.len())
                    .unwrap_or(0);
                root_schema_lengths.insert(root_id.get(), schema_length);
            }
            for schema_length in root_schema_lengths.into_values() {
                self.add_content_bytes(schema_length)?;
            }
        }
        Ok(())
    }

    fn add_host_value(
        &mut self,
        value: &FlowHostValue,
        depth: usize,
    ) -> Result<(), FlowSessionError> {
        if depth > MAX_VALUE_DEPTH {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::ResultLimitExceeded,
                "result host value depth exceeds 32 levels",
            ));
        }
        let child_depth = depth.saturating_add(1);
        self.add_nodes(1)?;
        match value {
            FlowHostValue::List(values) => {
                self.add_edges(values.len())?;
                for value in values {
                    self.add_host_value(value, child_depth)?;
                }
            }
            FlowHostValue::Object(values) => {
                self.add_edges(values.len())?;
                for (key, value) in values {
                    self.add_content_bytes(key.len())?;
                    self.add_host_value(value, child_depth)?;
                }
            }
            FlowHostValue::String(value) => self.add_content_bytes(value.len())?,
            FlowHostValue::Bool(_) | FlowHostValue::Number(_) => {}
        }
        Ok(())
    }

    fn add_outputs(&mut self, outputs: &[FlowOutput]) -> Result<(), FlowSessionError> {
        for output in outputs {
            validate_output_payload(&output.payload)?;
            match &output.payload {
                FlowOutputPayload::ReportedEvent {
                    name,
                    url,
                    target,
                    properties,
                    ..
                } => {
                    self.add_content_bytes(name.as_deref().map(str::len).unwrap_or(0))?;
                    self.add_content_bytes(url.as_deref().map(str::len).unwrap_or(0))?;
                    self.add_content_bytes(target.as_deref().map(str::len).unwrap_or(0))?;
                    self.add_nodes(
                        properties
                            .iter()
                            .filter(|property| {
                                !matches!(&property.value, FlowScalarValue::Trigger(_))
                            })
                            .count(),
                    )?;
                    for property in properties {
                        self.add_content_bytes(
                            property.name.as_deref().map(str::len).unwrap_or(0),
                        )?;
                        if let FlowScalarValue::String(value) = &property.value {
                            self.add_content_bytes(value.len())?;
                        }
                    }
                }
                FlowOutputPayload::StateChanged {
                    path,
                    value: Some(value),
                    ..
                } => {
                    self.add_content_bytes(path.len())?;
                    self.add_nodes(1)?;
                    match value {
                        FlowStateChangeValue::Scalar(FlowScalarValue::String(value)) => {
                            self.add_content_bytes(value.len())?;
                        }
                        FlowStateChangeValue::ViewModelReference { schema_name, .. } => {
                            self.add_content_bytes(schema_name.len())?;
                        }
                        FlowStateChangeValue::Scalar(_) => {}
                    }
                }
                FlowOutputPayload::StateChanged {
                    path, value: None, ..
                } => self.add_content_bytes(path.len())?,
                FlowOutputPayload::HostCommand { name, payload } => {
                    self.add_content_bytes(name.len())?;
                    self.add_host_value(payload, 1)?;
                }
                FlowOutputPayload::RenderRequested { .. }
                | FlowOutputPayload::RuntimeAdvanced { .. } => {}
            }
        }
        Ok(())
    }
}

fn validate_creation_value_arena_bounds(
    creation: &FlowSessionCreation,
) -> Result<(), FlowSessionError> {
    validate_bootstrap_payload(&creation.bootstrap)?;
    validate_output_batch(&creation.outputs)?;
    let mut projection = ResultValueArenaProjection::default();
    projection.add_content_bytes(
        creation
            .bootstrap
            .artboard_name
            .as_deref()
            .map(str::len)
            .unwrap_or(0),
    )?;
    projection.add_content_bytes(
        creation
            .bootstrap
            .player
            .name
            .as_deref()
            .map(str::len)
            .unwrap_or(0),
    )?;
    projection.add_catalog(&creation.bootstrap.catalog)?;
    projection.add_flow_arena(
        &creation.bootstrap.values,
        Some(&creation.bootstrap.catalog),
    )?;
    projection.add_outputs(&creation.outputs)
}

fn validate_result_value_arena_bounds(result: &FlowResult) -> Result<(), FlowSessionError> {
    let mut projection = ResultValueArenaProjection::default();
    if let Some(snapshot) = result.snapshot.as_ref() {
        projection
            .add_content_bytes(snapshot.artboard_name.as_deref().map(str::len).unwrap_or(0))?;
        projection.add_content_bytes(snapshot.player.name.as_deref().map(str::len).unwrap_or(0))?;
    }
    let effective_catalog = result
        .catalog
        .as_ref()
        .or_else(|| result.snapshot.as_ref().map(|snapshot| &snapshot.catalog));
    if let Some(catalog) = effective_catalog {
        projection.add_catalog(catalog)?;
    }
    if let Some(values) = result.values.as_ref() {
        projection.add_flow_arena(values, effective_catalog)?;
    } else if let Some(snapshot) = result.snapshot.as_ref() {
        projection.add_flow_arena(&snapshot.values, effective_catalog)?;
    }
    if let Some(inputs) = result.player_inputs.as_ref() {
        projection.add_nodes(inputs.len())?;
        for input in inputs {
            projection.add_content_bytes(input.name.as_deref().map(str::len).unwrap_or(0))?;
            if let FlowScalarValue::String(value) = &input.value {
                projection.add_content_bytes(value.len())?;
            }
        }
    }
    projection.add_outputs(&result.outputs)
}

fn merge_results(target: &mut FlowResult, mut source: FlowResult) -> Result<(), FlowSessionError> {
    if target.outputs.len().saturating_add(source.outputs.len()) > MAX_BATCH_ITEMS {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "output limit exceeded",
        ));
    }
    let encoded_size = target
        .outputs
        .iter()
        .chain(source.outputs.iter())
        .try_fold(0_usize, |total, output| {
            total.checked_add(flow_output_payload_bytes(&output.payload))
        })
        .ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded output size overflow",
            )
        })?;
    if encoded_size > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded output exceeds 4 MiB",
        ));
    }
    target.outputs.append(&mut source.outputs);
    target.dirty |= source.dirty;
    target.settled = source.settled;
    target.wake_after_seconds = source.wake_after_seconds;
    target
        .created_instances
        .append(&mut source.created_instances);
    Ok(())
}

/// Structural and identity-only changes cannot be reconciled from their
/// compact output payload alone. Include the operation's final authoritative
/// arena in the same result, while keeping scalar-only results allocation-free.
fn include_reconciliation_values(result: &mut FlowResult, values: &FlowValueArena) {
    let needs_values = result.outputs.iter().any(|output| {
        matches!(
            &output.payload,
            FlowOutputPayload::StateChanged {
                value: None | Some(FlowStateChangeValue::ViewModelReference { .. }),
                ..
            }
        )
    });
    if needs_values {
        result.values = Some(values.clone());
    }
}

fn validate_player_input_snapshot(inputs: &[FlowInputSnapshot]) -> Result<(), FlowSessionError> {
    if inputs.len() > MAX_BATCH_ITEMS {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "player input query item limit exceeded",
        ));
    }
    let mut total = inputs.len().saturating_mul(16);
    for input in inputs {
        if let Some(name) = input.name.as_deref() {
            validate_required_id_path(name, "input name")?;
            checked_payload_add(&mut total, name.len())?;
        }
        validate_scalar_value(&input.value, "player input")?;
        if let FlowScalarValue::String(value) = &input.value {
            checked_payload_add(&mut total, value.len())?;
        }
    }
    if total > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded player input snapshot exceeds 4 MiB",
        ));
    }
    Ok(())
}

fn checked_payload_add(total: &mut usize, value: usize) -> Result<(), FlowSessionError> {
    *total = total.checked_add(value).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded payload size overflow",
        )
    })?;
    Ok(())
}

fn append_output(
    outputs: &mut Vec<FlowOutput>,
    next_sequence: &mut u64,
    cycle: u64,
    phase: FlowOutputPhase,
    payload: FlowOutputPayload,
) -> Result<(), FlowSessionError> {
    validate_output_payload(&payload)?;
    if outputs.len() >= MAX_BATCH_ITEMS {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "output limit exceeded",
        ));
    }
    let encoded_size = outputs
        .iter()
        .try_fold(flow_output_payload_bytes(&payload), |size, output| {
            size.checked_add(flow_output_payload_bytes(&output.payload))
        })
        .ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded output size overflow",
            )
        })?;
    if encoded_size > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded output exceeds 4 MiB",
        ));
    }
    let sequence = *next_sequence;
    *next_sequence = next_sequence.checked_add(1).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "output sequence overflow",
        )
    })?;
    outputs.push(FlowOutput {
        sequence,
        cycle,
        phase,
        payload,
    });
    Ok(())
}

fn validate_output_batch(outputs: &[FlowOutput]) -> Result<(), FlowSessionError> {
    if outputs.len() > MAX_BATCH_ITEMS {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "output limit exceeded",
        ));
    }
    let encoded_size = outputs.iter().try_fold(0_usize, |size, output| {
        validate_output_payload(&output.payload)?;
        size.checked_add(flow_output_payload_bytes(&output.payload))
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "encoded output size overflow",
                )
            })
    })?;
    if encoded_size > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded output exceeds 4 MiB",
        ));
    }
    Ok(())
}

fn validate_scalar_value(value: &FlowScalarValue, label: &str) -> Result<(), FlowSessionError> {
    match value {
        FlowScalarValue::String(value) if value.len() > MAX_STRING_BYTES => {
            Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                format!("{label} string exceeds 1 MiB"),
            ))
        }
        FlowScalarValue::Number(value) if !value.is_finite() => Err(FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            format!("{label} number is non-finite"),
        )),
        _ => Ok(()),
    }
}

fn validate_output_payload(payload: &FlowOutputPayload) -> Result<(), FlowSessionError> {
    match payload {
        FlowOutputPayload::ReportedEvent {
            name,
            url,
            target,
            delay_seconds,
            properties,
            ..
        } => {
            if !delay_seconds.is_finite() || *delay_seconds < 0.0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "reported event delay is invalid",
                ));
            }
            validate_optional_selector(name.as_deref(), "event name")?;
            if url.is_some() != target.is_some() {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "OpenURL event URL and target presence must match",
                ));
            }
            if url
                .as_ref()
                .is_some_and(|value| value.len() > MAX_STRING_BYTES)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "OpenURL event URL exceeds 1 MiB",
                ));
            }
            if target
                .as_ref()
                .is_some_and(|value| value.len() > MAX_ID_PATH_BYTES)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "OpenURL event target exceeds identifier limit",
                ));
            }
            if target
                .as_deref()
                .is_some_and(|value| !matches!(value, "" | "_blank" | "_parent" | "_self" | "_top"))
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "OpenURL event target is not canonical",
                ));
            }
            if properties.len() > MAX_EVENT_PROPERTIES {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "event property limit exceeded",
                ));
            }
            for property in properties {
                validate_optional_selector(property.name.as_deref(), "event property name")?;
                validate_scalar_value(&property.value, "event property")?;
            }
        }
        FlowOutputPayload::StateChanged { path, value, .. } => {
            validate_required_id_path(path, "state-change path")?;
            if let Some(value) = value {
                match value {
                    FlowStateChangeValue::Scalar(value) => {
                        validate_scalar_value(value, "state change")?
                    }
                    FlowStateChangeValue::ViewModelReference { schema_name, .. } => {
                        validate_required_id_path(schema_name, "state-change schema name")?
                    }
                }
            }
        }
        FlowOutputPayload::HostCommand { name, payload } => {
            validate_required_id_path(name, "host command name")?;
            if !matches!(payload, FlowHostValue::Object(_)) {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "host command payload root must be an object",
                ));
            }
            let mut nodes = 0_usize;
            let mut edges = 0_usize;
            validate_host_value(payload, 1, &mut nodes, &mut edges)?;
        }
        FlowOutputPayload::RuntimeAdvanced { delta_seconds } => {
            if !delta_seconds.is_finite() || *delta_seconds < 0.0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "runtime advance delta is invalid",
                ));
            }
        }
        FlowOutputPayload::RenderRequested { .. } => {}
    }
    Ok(())
}

fn state_batch_payload_bytes(batch: &FlowStateBatch) -> Result<usize, FlowSessionError> {
    let mut total = 0_usize;
    for item in &batch.new_instances {
        checked_payload_add(&mut total, item.schema_name.len())?;
        checked_payload_add(
            &mut total,
            item.authored_instance_name
                .as_deref()
                .map(str::len)
                .unwrap_or(0),
        )?;
    }
    for mutation in &batch.mutations {
        let (path, value) = match mutation {
            FlowStateMutation::SetInputBool { name, .. }
            | FlowStateMutation::SetInputNumber { name, .. }
            | FlowStateMutation::FireInputTrigger { name } => (name.as_str(), None),
            FlowStateMutation::SetValue { path, value, .. } => (path.as_str(), Some(value)),
            FlowStateMutation::SetViewModel { path, .. }
            | FlowStateMutation::FireTrigger { path, .. }
            | FlowStateMutation::ListInsert { path, .. }
            | FlowStateMutation::ListRemove { path, .. }
            | FlowStateMutation::ListSwap { path, .. }
            | FlowStateMutation::ListMove { path, .. }
            | FlowStateMutation::ListSet { path, .. }
            | FlowStateMutation::ListClear { path, .. } => (path.as_str(), None),
        };
        checked_payload_add(&mut total, path.len())?;
        if let Some(FlowScalarValue::String(value)) = value {
            checked_payload_add(&mut total, value.len())?;
        }
    }
    Ok(total)
}

fn flow_output_payload_bytes(payload: &FlowOutputPayload) -> usize {
    match payload {
        FlowOutputPayload::ReportedEvent {
            name,
            url,
            target,
            properties,
            ..
        } => name
            .as_deref()
            .map(str::len)
            .unwrap_or(0)
            .saturating_add(url.as_deref().map(str::len).unwrap_or(0))
            .saturating_add(target.as_deref().map(str::len).unwrap_or(0))
            .saturating_add(
                properties
                    .iter()
                    .map(|property| {
                        property
                            .name
                            .as_deref()
                            .map(str::len)
                            .unwrap_or(0)
                            .saturating_add(match &property.value {
                                FlowScalarValue::String(value) => value.len(),
                                _ => 16,
                            })
                    })
                    .sum::<usize>(),
            ),
        FlowOutputPayload::StateChanged { path, value, .. } => {
            path.len().saturating_add(match value {
                Some(FlowStateChangeValue::Scalar(FlowScalarValue::String(value))) => value.len(),
                Some(FlowStateChangeValue::ViewModelReference { schema_name, .. }) => {
                    schema_name.len().saturating_add(16)
                }
                Some(FlowStateChangeValue::Scalar(_)) => 16,
                None => 0,
            })
        }
        FlowOutputPayload::HostCommand { name, payload } => {
            name.len().saturating_add(host_value_payload_bytes(payload))
        }
        FlowOutputPayload::RenderRequested { .. } | FlowOutputPayload::RuntimeAdvanced { .. } => 16,
    }
}

fn validate_host_value(
    value: &FlowHostValue,
    depth: usize,
    nodes: &mut usize,
    edges: &mut usize,
) -> Result<(), FlowSessionError> {
    if depth > MAX_VALUE_DEPTH {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "host command value depth limit exceeded",
        ));
    }
    let child_depth = depth.saturating_add(1);
    *nodes = nodes.checked_add(1).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "host command value node count overflowed",
        )
    })?;
    if *nodes > MAX_VALUE_NODES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "host command value node limit exceeded",
        ));
    }
    match value {
        FlowHostValue::Number(value) if !value.is_finite() => Err(FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            "host command number is non-finite",
        )),
        FlowHostValue::String(value) if value.len() > MAX_STRING_BYTES => {
            Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "host command string exceeds 1 MiB",
            ))
        }
        FlowHostValue::List(values) => {
            if values.len() > MAX_LIST_ITEMS {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "host command list item limit exceeded",
                ));
            }
            charge_host_value_edges(edges, values.len())?;
            for value in values {
                validate_host_value(value, child_depth, nodes, edges)?;
            }
            Ok(())
        }
        FlowHostValue::Object(values) => {
            if values.len() > MAX_LIST_ITEMS {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "host command object field limit exceeded",
                ));
            }
            charge_host_value_edges(edges, values.len())?;
            for (key, value) in values {
                validate_required_id_path(key, "host command object key")?;
                validate_host_value(value, child_depth, nodes, edges)?;
            }
            Ok(())
        }
        FlowHostValue::Bool(_) | FlowHostValue::Number(_) | FlowHostValue::String(_) => Ok(()),
    }
}

fn charge_host_value_edges(edges: &mut usize, count: usize) -> Result<(), FlowSessionError> {
    *edges = edges.checked_add(count).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "host command value edge count overflowed",
        )
    })?;
    if *edges > MAX_VALUE_EDGES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "host command value edge limit exceeded",
        ));
    }
    Ok(())
}

fn host_value_payload_bytes(value: &FlowHostValue) -> usize {
    match value {
        FlowHostValue::Bool(_) | FlowHostValue::Number(_) => 16,
        FlowHostValue::String(value) => value.len(),
        FlowHostValue::List(values) => values
            .iter()
            .map(host_value_payload_bytes)
            .fold(0_usize, usize::saturating_add),
        FlowHostValue::Object(values) => values.iter().fold(0_usize, |size, (key, value)| {
            size.saturating_add(key.len())
                .saturating_add(host_value_payload_bytes(value))
        }),
    }
}

#[derive(Debug, Clone)]
enum ResolvedMutation {
    SetInputBool {
        name: String,
        value: bool,
    },
    SetInputNumber {
        name: String,
        value: f32,
    },
    FireInputTrigger {
        name: String,
    },
    SetValue {
        instance: FlowInstanceId,
        path: String,
        value: FlowScalarValue,
    },
    SetViewModel {
        instance: FlowInstanceId,
        path: String,
        value: FlowInstanceId,
    },
    FireTrigger {
        instance: FlowInstanceId,
        path: String,
    },
    ListInsert {
        instance: FlowInstanceId,
        path: String,
        index: usize,
        item: FlowInstanceId,
    },
    ListRemove {
        instance: FlowInstanceId,
        path: String,
        index: usize,
    },
    ListSwap {
        instance: FlowInstanceId,
        path: String,
        first: usize,
        second: usize,
    },
    ListMove {
        instance: FlowInstanceId,
        path: String,
        from: usize,
        to: usize,
    },
    ListSet {
        instance: FlowInstanceId,
        path: String,
        index: usize,
        item: FlowInstanceId,
    },
    ListClear {
        instance: FlowInstanceId,
        path: String,
    },
}

fn resolve_instance_ref(
    reference: FlowInstanceRef,
    new_ids: &BTreeMap<u32, FlowInstanceId>,
) -> Result<FlowInstanceId, FlowSessionError> {
    match reference {
        FlowInstanceRef::Existing(id) => Ok(id),
        FlowInstanceRef::New(local_id) => new_ids.get(&local_id).copied().ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::NotFound,
                format!("transaction-local instance {local_id} was not declared"),
            )
        }),
    }
}

fn resolve_mutation(
    mutation: &FlowStateMutation,
    new_ids: &BTreeMap<u32, FlowInstanceId>,
) -> Result<ResolvedMutation, FlowSessionError> {
    Ok(match mutation {
        FlowStateMutation::SetInputBool { name, value } => ResolvedMutation::SetInputBool {
            name: name.clone(),
            value: *value,
        },
        FlowStateMutation::SetInputNumber { name, value } => ResolvedMutation::SetInputNumber {
            name: name.clone(),
            value: *value,
        },
        FlowStateMutation::FireInputTrigger { name } => {
            ResolvedMutation::FireInputTrigger { name: name.clone() }
        }
        FlowStateMutation::SetValue {
            instance,
            path,
            value,
        } => ResolvedMutation::SetValue {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            value: value.clone(),
        },
        FlowStateMutation::SetViewModel {
            instance,
            path,
            value,
        } => ResolvedMutation::SetViewModel {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            value: resolve_instance_ref(*value, new_ids)?,
        },
        FlowStateMutation::FireTrigger { instance, path } => ResolvedMutation::FireTrigger {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
        },
        FlowStateMutation::ListInsert {
            instance,
            path,
            index,
            item,
        } => ResolvedMutation::ListInsert {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            index: *index,
            item: resolve_instance_ref(*item, new_ids)?,
        },
        FlowStateMutation::ListRemove {
            instance,
            path,
            index,
        } => ResolvedMutation::ListRemove {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            index: *index,
        },
        FlowStateMutation::ListSwap {
            instance,
            path,
            first,
            second,
        } => ResolvedMutation::ListSwap {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            first: *first,
            second: *second,
        },
        FlowStateMutation::ListMove {
            instance,
            path,
            from,
            to,
        } => ResolvedMutation::ListMove {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            from: *from,
            to: *to,
        },
        FlowStateMutation::ListSet {
            instance,
            path,
            index,
            item,
        } => ResolvedMutation::ListSet {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            index: *index,
            item: resolve_instance_ref(*item, new_ids)?,
        },
        FlowStateMutation::ListClear { instance, path } => ResolvedMutation::ListClear {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
        },
    })
}

fn mutation_instance_ids(mutation: &ResolvedMutation) -> Vec<FlowInstanceId> {
    match mutation {
        ResolvedMutation::SetValue { instance, .. }
        | ResolvedMutation::FireTrigger { instance, .. }
        | ResolvedMutation::ListRemove { instance, .. }
        | ResolvedMutation::ListSwap { instance, .. }
        | ResolvedMutation::ListMove { instance, .. }
        | ResolvedMutation::ListClear { instance, .. } => vec![*instance],
        ResolvedMutation::ListInsert { instance, item, .. }
        | ResolvedMutation::ListSet { instance, item, .. } => vec![*instance, *item],
        ResolvedMutation::SetViewModel {
            instance, value, ..
        } => vec![*instance, *value],
        ResolvedMutation::SetInputBool { .. }
        | ResolvedMutation::SetInputNumber { .. }
        | ResolvedMutation::FireInputTrigger { .. } => Vec::new(),
    }
}

fn instantiate_named_view_model(
    file: &Arc<File>,
    schema_name: &str,
    authored_name: Option<&str>,
) -> Result<ViewModelInstance, FlowSessionError> {
    let view_model_index = file
        .graph()
        .view_models
        .iter()
        .position(|schema| schema.name.as_deref() == Some(schema_name))
        .ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::NotFound,
                format!("view-model schema '{schema_name}' was not found"),
            )
        })?;
    let raw = if let Some(name) = authored_name {
        if file
            .runtime()
            .view_model_instance_named(view_model_index, name)
            .is_none()
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::NotFound,
                format!("authored instance '{name}' was not found in schema '{schema_name}'"),
            ));
        }
        RuntimeOwnedViewModelInstance::from_instance_name(file.runtime(), view_model_index, name)
    } else {
        RuntimeOwnedViewModelInstance::new(file.runtime(), view_model_index)
    }
    .ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            "view-model instance could not be created",
        )
    })?;
    Ok(ViewModelInstance {
        raw: RuntimeOwnedViewModelHandle::new(raw),
    })
}

fn validate_required_id_path(value: &str, label: &str) -> Result<(), FlowSessionError> {
    if value.is_empty() {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::InvalidArgument,
            format!("{label} must not be empty"),
        ));
    }
    if value.len() > MAX_ID_PATH_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            format!("{label} exceeds {MAX_ID_PATH_BYTES} UTF-8 bytes"),
        ));
    }
    if value.split('/').any(str::is_empty) {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::InvalidArgument,
            format!("{label} contains an empty path segment"),
        ));
    }
    Ok(())
}

fn validate_required_text_run_name(value: &str) -> Result<(), FlowSessionError> {
    if value.is_empty() {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::InvalidArgument,
            "text-run name must not be empty",
        ));
    }
    if value.len() > MAX_ID_PATH_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            format!("text-run name exceeds {MAX_ID_PATH_BYTES} UTF-8 bytes"),
        ));
    }
    Ok(())
}

fn prevalidate_and_apply_mutation(
    machine: Option<&mut StateMachineInstance>,
    instances: &BTreeMap<FlowInstanceId, ViewModelInstance>,
    mutation: &ResolvedMutation,
) -> Result<(), FlowSessionError> {
    match mutation {
        ResolvedMutation::SetInputBool { name, value } => {
            validate_required_id_path(name, "input name")?;
            let machine = machine.ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    "session has no state-machine player",
                )
            })?;
            let index = machine.input_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("input '{name}' was not found"),
                )
            })?;
            if machine.input(index).map(|input| input.kind())
                != Some(crate::StateMachineInputKind::Bool)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    format!("input '{name}' is not boolean"),
                ));
            }
            let _ = machine.set_bool(index, *value);
        }
        ResolvedMutation::SetInputNumber { name, value } => {
            validate_required_id_path(name, "input name")?;
            if !value.is_finite() {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "numeric state values must be finite",
                ));
            }
            let machine = machine.ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    "session has no state-machine player",
                )
            })?;
            let index = machine.input_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("input '{name}' was not found"),
                )
            })?;
            if machine.input(index).map(|input| input.kind())
                != Some(crate::StateMachineInputKind::Number)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    format!("input '{name}' is not numeric"),
                ));
            }
            let _ = machine.set_number(index, *value);
        }
        ResolvedMutation::FireInputTrigger { name } => {
            validate_required_id_path(name, "input name")?;
            let machine = machine.ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    "session has no state-machine player",
                )
            })?;
            let index = machine.input_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("input '{name}' was not found"),
                )
            })?;
            if machine.input(index).map(|input| input.kind())
                != Some(crate::StateMachineInputKind::Trigger)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    format!("input '{name}' is not a trigger"),
                ));
            }
            let _ = machine.fire_trigger(index);
        }
        _ => apply_view_model_mutation(instances, mutation)?,
    }
    Ok(())
}

fn instance(
    instances: &BTreeMap<FlowInstanceId, ViewModelInstance>,
    id: FlowInstanceId,
) -> Result<&ViewModelInstance, FlowSessionError> {
    instances.get(&id).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::NotFound,
            format!("instance {} was not found", id.get()),
        )
    })
}

fn apply_view_model_mutation(
    instances: &BTreeMap<FlowInstanceId, ViewModelInstance>,
    mutation: &ResolvedMutation,
) -> Result<(), FlowSessionError> {
    match mutation {
        ResolvedMutation::SetValue {
            instance: id,
            path,
            value,
        } => {
            validate_required_id_path(path, "property path")?;
            let view_model = instance(instances, *id)?;
            let mut raw = view_model.raw_mut();
            match value {
                FlowScalarValue::Null => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::InvalidArgument,
                        "null cannot be assigned to a scalar property",
                    ));
                }
                FlowScalarValue::String(value) => {
                    if value.len() > MAX_STRING_BYTES {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::LimitExceeded,
                            "string value exceeds 1 MiB",
                        ));
                    }
                    if raw
                        .string_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("string property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_string_by_property_name_path(path, value.as_bytes());
                }
                FlowScalarValue::Number(value) => {
                    if !value.is_finite() {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::InvalidArgument,
                            "numeric state values must be finite",
                        ));
                    }
                    if raw
                        .number_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("number property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_number_by_property_name_path(path, *value);
                }
                FlowScalarValue::Bool(value) => {
                    if raw
                        .boolean_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("boolean property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_boolean_by_property_name_path(path, *value);
                }
                FlowScalarValue::Enum(value) => {
                    if raw.enum_source_handle_by_property_name_path(path).is_none() {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("enum property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_enum_by_property_name_path(path, *value);
                }
                FlowScalarValue::ListIndex(value) => {
                    if raw
                        .symbol_list_index_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("list-index property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_symbol_list_index_by_property_name_path(path, *value);
                }
                FlowScalarValue::Color(value) => {
                    if raw
                        .color_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("color property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_color_by_property_name_path(path, *value);
                }
                FlowScalarValue::Image(value) => {
                    if raw
                        .asset_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("image property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_asset_by_property_name_path(path, *value);
                }
                FlowScalarValue::Trigger(_) => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::InvalidArgument,
                        "trigger counters are advanced with FireTrigger",
                    ));
                }
            }
        }
        ResolvedMutation::SetViewModel {
            instance: id,
            path,
            value,
        } => {
            validate_required_id_path(path, "view-model property path")?;
            if path.contains('/') {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "nested view-model replacement currently supports outer properties only",
                ));
            }
            let owner = instance(instances, *id)?;
            let value = instance(instances, *value)?;
            match owner
                .handle()
                .link_view_model_by_property_name_path(path, value.handle())
            {
                Ok(_) => {}
                Err(RuntimeViewModelLinkError::PropertyNotFound) => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("view-model property '{path}' was not found"),
                    ));
                }
                Err(RuntimeViewModelLinkError::NestedPathUnsupported) => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::InvalidArgument,
                        "nested view-model replacement currently supports outer properties only",
                    ));
                }
                Err(RuntimeViewModelLinkError::SchemaMismatch) => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::Conflict,
                        "view-model replacement schema does not match the property",
                    ));
                }
                Err(RuntimeViewModelLinkError::Cycle) => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::Conflict,
                        "view-model replacement would create a cycle",
                    ));
                }
                Err(RuntimeViewModelLinkError::BorrowConflict) => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "view-model replacement graph is already borrowed",
                    ));
                }
            }
        }
        ResolvedMutation::FireTrigger { instance: id, path } => {
            validate_required_id_path(path, "property path")?;
            let view_model = instance(instances, *id)?;
            let mut raw = view_model.raw_mut();
            if raw
                .trigger_source_handle_by_property_name_path(path)
                .is_none()
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("trigger property '{path}' was not found"),
                ));
            }
            let current = raw.trigger_value_by_property_name_path(path).unwrap_or(0);
            let next = current.checked_add(1).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "trigger counter overflow",
                )
            })?;
            let _ = raw.set_trigger_by_property_name_path(path, next);
        }
        ResolvedMutation::ListInsert {
            instance: id,
            path,
            index,
            item,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let item = instance(instances, *item)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if count >= MAX_LIST_ITEMS || *index > count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "list insert index or capacity is out of range",
                ));
            }
            if !owner
                .handle()
                .insert_list_item_by_property_name_path(path, *index, item.handle())
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Conflict,
                    "list insert would create a cycle",
                ));
            }
        }
        ResolvedMutation::ListRemove {
            instance: id,
            path,
            index,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *index >= count
                || !owner
                    .handle()
                    .remove_list_item_by_property_name_path(path, *index)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list remove index is out of range",
                ));
            }
        }
        ResolvedMutation::ListSwap {
            instance: id,
            path,
            first,
            second,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *first >= count || *second >= count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list swap index is out of range",
                ));
            }
            if first != second {
                let _ = owner
                    .handle()
                    .swap_list_items_by_property_name_path(path, *first, *second);
            }
        }
        ResolvedMutation::ListMove {
            instance: id,
            path,
            from,
            to,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *from >= count || *to >= count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list move index is out of range",
                ));
            }
            if from != to {
                let _ = owner
                    .handle()
                    .move_list_item_by_property_name_path(path, *from, *to);
            }
        }
        ResolvedMutation::ListSet {
            instance: id,
            path,
            index,
            item,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let item = instance(instances, *item)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *index >= count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list set index is out of range",
                ));
            }
            if !owner
                .handle()
                .set_list_item_by_property_name_path(path, *index, item.handle())
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Conflict,
                    "list set would create a cycle or no change",
                ));
            }
        }
        ResolvedMutation::ListClear { instance: id, path } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            if owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .is_none()
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("list property '{path}' was not found"),
                ));
            }
            let _ = owner.handle().clear_list_items_by_property_name_path(path);
        }
        ResolvedMutation::SetInputBool { .. }
        | ResolvedMutation::SetInputNumber { .. }
        | ResolvedMutation::FireInputTrigger { .. } => {}
    }
    Ok(())
}

fn mutation_echo(
    mutation: &ResolvedMutation,
    catalog: &FlowCatalog,
) -> Result<Option<(Option<FlowInstanceId>, String, Option<FlowStateChangeValue>)>, FlowSessionError>
{
    match mutation {
        ResolvedMutation::SetInputBool { name, value } => Ok(Some((
            None,
            name.clone(),
            Some(FlowStateChangeValue::Scalar(FlowScalarValue::Bool(*value))),
        ))),
        ResolvedMutation::SetInputNumber { name, value } => Ok(Some((
            None,
            name.clone(),
            Some(FlowStateChangeValue::Scalar(FlowScalarValue::Number(
                *value,
            ))),
        ))),
        ResolvedMutation::FireInputTrigger { name } => Ok(Some((None, name.clone(), None))),
        ResolvedMutation::SetValue {
            instance,
            path,
            value,
        } => Ok(Some((
            Some(*instance),
            path.clone(),
            Some(FlowStateChangeValue::Scalar(value.clone())),
        ))),
        ResolvedMutation::SetViewModel {
            instance,
            path,
            value,
        } => {
            let schema_name = catalog
                .instances
                .iter()
                .find(|candidate| candidate.id == *value)
                .map(|candidate| candidate.schema_name.clone())
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "replacement instance metadata disappeared",
                    )
                })?;
            Ok(Some((
                Some(*instance),
                path.clone(),
                Some(FlowStateChangeValue::ViewModelReference {
                    instance_id: *value,
                    schema_name,
                }),
            )))
        }
        ResolvedMutation::FireTrigger { instance, path } => {
            Ok(Some((Some(*instance), path.clone(), None)))
        }
        ResolvedMutation::ListInsert { instance, path, .. }
        | ResolvedMutation::ListRemove { instance, path, .. }
        | ResolvedMutation::ListSwap { instance, path, .. }
        | ResolvedMutation::ListMove { instance, path, .. }
        | ResolvedMutation::ListSet { instance, path, .. }
        | ResolvedMutation::ListClear { instance, path } => {
            Ok(Some((Some(*instance), path.clone(), None)))
        }
    }
}

fn diff_value_arenas(
    before: &FlowValueArena,
    after: &FlowValueArena,
    catalog: &FlowCatalog,
) -> Result<Vec<(FlowInstanceId, String, Option<FlowStateChangeValue>)>, FlowSessionError> {
    let mut changes = Vec::new();
    for (instance_id, after_root) in &after.roots {
        let before_root = before
            .roots
            .iter()
            .find(|(id, _)| id == instance_id)
            .map(|(_, root)| *root);
        diff_value_node(
            before,
            before_root,
            after,
            Some(*after_root),
            *instance_id,
            "",
            catalog,
            &mut changes,
        )?;
    }
    let (mut structural, scalar): (Vec<_>, Vec<_>) = changes.into_iter().partition(|change| {
        matches!(
            &change.2,
            Some(FlowStateChangeValue::ViewModelReference { .. })
        )
    });
    structural.extend(scalar);
    Ok(structural)
}

fn diff_value_node(
    before: &FlowValueArena,
    before_id: Option<FlowValueId>,
    after: &FlowValueArena,
    after_id: Option<FlowValueId>,
    instance_id: FlowInstanceId,
    path: &str,
    catalog: &FlowCatalog,
    changes: &mut Vec<(FlowInstanceId, String, Option<FlowStateChangeValue>)>,
) -> Result<(), FlowSessionError> {
    let before_value = before_id
        .and_then(|id| before.nodes.iter().find(|node| node.id == id))
        .map(|node| &node.value);
    let after_value = after_id
        .and_then(|id| after.nodes.iter().find(|node| node.id == id))
        .map(|node| &node.value);
    match (before_value, after_value) {
        (Some(FlowValue::ViewModel(before_edges)), Some(FlowValue::ViewModel(after_edges))) => {
            if !path.is_empty() {
                let before_identity =
                    before_id.and_then(|id| value_instance_id_for_node(before, id));
                let after_identity = after_id.and_then(|id| value_instance_id_for_node(after, id));
                if let Some(after_identity) = after_identity {
                    if !path.contains('/') && before_identity != Some(after_identity) {
                        let schema_name = catalog
                            .instances
                            .iter()
                            .find(|candidate| candidate.id == after_identity)
                            .map(|candidate| candidate.schema_name.clone())
                            .ok_or_else(|| {
                                FlowSessionError::new(
                                    FlowSessionErrorKind::Runtime,
                                    "replacement instance metadata disappeared",
                                )
                            })?;
                        validate_required_id_path(path, "state-change path")?;
                        changes.push((
                            instance_id,
                            path.to_owned(),
                            Some(FlowStateChangeValue::ViewModelReference {
                                instance_id: after_identity,
                                schema_name,
                            }),
                        ));
                    }
                    // Linked nested view models own stable arena roots. Treat
                    // the nested node as an identity boundary so its root is
                    // the sole source of descendant scalar changes.
                    return Ok(());
                }
            }
            diff_named_value_edges(
                before,
                before_edges,
                after,
                after_edges,
                instance_id,
                path,
                catalog,
                changes,
            )?;
        }
        (Some(FlowValue::Object(before_edges)), Some(FlowValue::Object(after_edges))) => {
            diff_named_value_edges(
                before,
                before_edges,
                after,
                after_edges,
                instance_id,
                path,
                catalog,
                changes,
            )?;
        }
        (Some(FlowValue::List(before_items)), Some(FlowValue::List(after_items))) => {
            let before_identity = before_items
                .iter()
                .map(|node| value_instance_id_for_node(before, *node))
                .collect::<Vec<_>>();
            let after_identity = after_items
                .iter()
                .map(|node| value_instance_id_for_node(after, *node))
                .collect::<Vec<_>>();
            if before_items.len() != after_items.len() || before_identity != after_identity {
                validate_required_id_path(path, "state-change path")?;
                changes.push((instance_id, path.to_owned(), None));
            }
            for (index, after_child) in after_items.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                validate_required_id_path(&child_path, "state-change path")?;
                let before_child = before_items.get(index).copied();
                diff_value_node(
                    before,
                    before_child,
                    after,
                    Some(*after_child),
                    instance_id,
                    &child_path,
                    catalog,
                    changes,
                )?;
            }
        }
        (_, Some(value)) if before_value != Some(value) => {
            validate_required_id_path(path, "state-change path")?;
            changes.push((
                instance_id,
                path.to_owned(),
                flow_scalar_from_arena_value(value).map(FlowStateChangeValue::Scalar),
            ));
        }
        _ => {}
    }
    Ok(())
}

fn diff_named_value_edges(
    before: &FlowValueArena,
    before_edges: &[(String, FlowValueId)],
    after: &FlowValueArena,
    after_edges: &[(String, FlowValueId)],
    instance_id: FlowInstanceId,
    path: &str,
    catalog: &FlowCatalog,
    changes: &mut Vec<(FlowInstanceId, String, Option<FlowStateChangeValue>)>,
) -> Result<(), FlowSessionError> {
    for (name, after_child) in after_edges {
        let child_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{path}/{name}")
        };
        validate_required_id_path(&child_path, "state-change path")?;
        let before_child = before_edges
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, id)| *id);
        diff_value_node(
            before,
            before_child,
            after,
            Some(*after_child),
            instance_id,
            &child_path,
            catalog,
            changes,
        )?;
    }
    Ok(())
}

fn value_instance_id_for_node(arena: &FlowValueArena, node: FlowValueId) -> Option<FlowInstanceId> {
    arena
        .roots
        .iter()
        .find_map(|(instance_id, root)| (*root == node).then_some(*instance_id))
}

fn flow_scalar_from_arena_value(value: &FlowValue) -> Option<FlowScalarValue> {
    match value {
        FlowValue::Null => Some(FlowScalarValue::Null),
        FlowValue::String(value) => Some(FlowScalarValue::String(value.clone())),
        FlowValue::Number(value) => Some(FlowScalarValue::Number(*value)),
        FlowValue::Bool(value) => Some(FlowScalarValue::Bool(*value)),
        FlowValue::Enum(value) => Some(FlowScalarValue::Enum(*value)),
        FlowValue::ListIndex(value) => Some(FlowScalarValue::ListIndex(*value)),
        FlowValue::Color(value) => Some(FlowScalarValue::Color(*value)),
        FlowValue::Image(value) => Some(FlowScalarValue::Image(*value)),
        FlowValue::Object(_) | FlowValue::ViewModel(_) | FlowValue::List(_) => None,
    }
}

fn validate_optional_selector(value: Option<&str>, label: &str) -> Result<(), FlowSessionError> {
    if value.is_some_and(str::is_empty) {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::InvalidArgument,
            format!("{label} must not be empty"),
        ));
    }
    if value.is_some_and(|value| value.len() > MAX_ID_PATH_BYTES) {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            format!("{label} exceeds {MAX_ID_PATH_BYTES} UTF-8 bytes"),
        ));
    }
    Ok(())
}

fn select_player(
    artboard: crate::Artboard<'_>,
    instance: &OwnedArtboardInstance,
    explicit: Option<&FlowPlayerSelector>,
) -> Result<(FlowPlayerMetadata, FlowPlayer), FlowSessionError> {
    match explicit {
        Some(FlowPlayerSelector::StateMachine(name)) => {
            let index = artboard.state_machine_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("state machine '{name}' was not found"),
                )
            })?;
            let machine = instance.state_machine_instance_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "selected state machine could not be instantiated",
                )
            })?;
            return Ok((
                FlowPlayerMetadata {
                    kind: FlowPlayerKind::StateMachine,
                    selection: FlowPlayerSelection::ExplicitStateMachine,
                    index: Some(index),
                    name: Some(name.clone()),
                },
                FlowPlayer::StateMachine(Box::new(machine)),
            ));
        }
        Some(FlowPlayerSelector::LinearAnimation(name)) => {
            let index = artboard.animation_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("linear animation '{name}' was not found"),
                )
            })?;
            let animation = instance
                .linear_animation_instance_named(name)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "selected linear animation could not be instantiated",
                    )
                })?;
            return Ok((
                FlowPlayerMetadata {
                    kind: FlowPlayerKind::LinearAnimation,
                    selection: FlowPlayerSelection::ExplicitLinearAnimation,
                    index: Some(index),
                    name: Some(name.clone()),
                },
                FlowPlayer::Animation(animation),
            ));
        }
        None => {}
    }

    if let Some(index) = artboard.default_state_machine_index()
        && let Some(machine) = instance.state_machine_instance(index)
    {
        return Ok((
            FlowPlayerMetadata {
                kind: FlowPlayerKind::StateMachine,
                selection: FlowPlayerSelection::AuthoredDefaultStateMachine,
                index: Some(index),
                name: artboard.state_machine_name(index).map(ToOwned::to_owned),
            },
            FlowPlayer::StateMachine(Box::new(machine)),
        ));
    }
    if artboard.state_machine_count() > 0
        && let Some(machine) = instance.state_machine_instance(0)
    {
        return Ok((
            FlowPlayerMetadata {
                kind: FlowPlayerKind::StateMachine,
                selection: FlowPlayerSelection::FirstStateMachine,
                index: Some(0),
                name: artboard.state_machine_name(0).map(ToOwned::to_owned),
            },
            FlowPlayer::StateMachine(Box::new(machine)),
        ));
    }
    if let Some(animation) = artboard.graph().animations.first()
        && let Some(animation_instance) = instance.raw().linear_animation_instance(0)
    {
        return Ok((
            FlowPlayerMetadata {
                kind: FlowPlayerKind::LinearAnimation,
                selection: FlowPlayerSelection::FirstAnimation,
                index: Some(0),
                name: animation.name.clone(),
            },
            FlowPlayer::Animation(animation_instance),
        ));
    }
    Ok((
        FlowPlayerMetadata {
            kind: FlowPlayerKind::Static,
            selection: FlowPlayerSelection::Static,
            index: None,
            name: None,
        },
        FlowPlayer::Static,
    ))
}

fn build_catalog(
    file: &File,
    root_selection: Option<(usize, Option<usize>)>,
) -> Result<FlowCatalog, FlowSessionError> {
    let root_instance_id = root_selection.map(|_| FlowInstanceId(1));
    let mut templates = Vec::new();
    let mut schemas = Vec::with_capacity(file.graph().view_models.len());
    for (view_model_index, schema) in file.graph().view_models.iter().enumerate() {
        let schema_name = schema
            .name
            .clone()
            .unwrap_or_else(|| format!("viewModel{view_model_index}"));
        for (authored_index, instance) in schema.instances.iter().enumerate() {
            templates.push(FlowInstanceTemplate {
                schema_name: schema_name.clone(),
                authored_name: instance.name.clone(),
                authored_index,
            });
        }
        let runtime_schema = file.runtime().view_model(view_model_index).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "catalog view-model schema disappeared",
            )
        })?;
        let mut properties = Vec::with_capacity(schema.properties.len());
        for (property_index, property) in schema.properties.iter().enumerate() {
            let Some(name) = property.name.clone() else {
                continue;
            };
            let runtime_property =
                runtime_schema
                    .properties
                    .get(property_index)
                    .ok_or_else(|| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            "catalog view-model property disappeared",
                        )
                    })?;
            let value_type = flow_value_type_for_property(property.type_name);
            let mut enum_labels = Vec::new();
            if value_type == FlowValueType::Enum {
                while let Some(label) = file
                    .runtime()
                    .view_model_property_enum_value_for_index_object(
                        runtime_property,
                        enum_labels.len(),
                    )
                {
                    if enum_labels.len() >= MAX_BATCH_ITEMS {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::LimitExceeded,
                            "catalog enum label limit exceeded",
                        ));
                    }
                    enum_labels.push(String::from_utf8(label.to_vec()).map_err(|_| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            "catalog enum label is not UTF-8",
                        )
                    })?);
                }
            }
            let referenced_schema_name = if value_type == FlowValueType::ViewModel {
                let referenced_index = usize::try_from(
                    runtime_property
                        .uint_property("viewModelReferenceId")
                        .ok_or_else(|| {
                            FlowSessionError::new(
                                FlowSessionErrorKind::Runtime,
                                "catalog view-model property has no schema reference id",
                            )
                        })?,
                )
                .map_err(|_| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "catalog referenced schema id is out of range",
                    )
                })?;
                Some(
                    file.graph()
                        .view_models
                        .get(referenced_index)
                        .ok_or_else(|| {
                            FlowSessionError::new(
                                FlowSessionErrorKind::Runtime,
                                "catalog referenced schema disappeared",
                            )
                        })?
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("viewModel{referenced_index}")),
                )
            } else {
                None
            };
            properties.push(FlowPropertySchema {
                name,
                value_type,
                enum_labels,
                referenced_schema_name,
            });
        }
        schemas.push(FlowSchema {
            name: schema_name,
            properties,
        });
    }

    let instances = root_selection
        .map(|(view_model_index, authored_index)| {
            let schema_name = file
                .graph()
                .view_models
                .get(view_model_index)
                .and_then(|schema| schema.name.clone())
                .unwrap_or_else(|| format!("viewModel{view_model_index}"));
            let authored_name = authored_index.and_then(|index| {
                file.graph()
                    .view_models
                    .get(view_model_index)
                    .and_then(|schema| schema.instances.get(index))
                    .and_then(|instance| instance.name.clone())
            });
            vec![FlowInstanceMetadata {
                id: FlowInstanceId(1),
                schema_name,
                authored_name,
                is_root: true,
            }]
        })
        .unwrap_or_default();
    let catalog = FlowCatalog {
        schemas,
        templates,
        instances,
        root_instance_id,
    };
    validate_catalog(&catalog)?;
    Ok(catalog)
}

fn validate_catalog(catalog: &FlowCatalog) -> Result<(), FlowSessionError> {
    if catalog.schemas.len() > MAX_BATCH_ITEMS
        || catalog.templates.len() > MAX_BATCH_ITEMS
        || catalog.instances.len() > MAX_INSTANCES
    {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "catalog collection limit exceeded",
        ));
    }
    let mut property_count = 0_usize;
    let mut enum_label_count = 0_usize;
    for schema in &catalog.schemas {
        validate_required_id_path(&schema.name, "schema name")?;
        property_count = property_count
            .checked_add(schema.properties.len())
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "catalog property count overflow",
                )
            })?;
        if schema.properties.len() > MAX_BATCH_ITEMS || property_count > MAX_BATCH_ITEMS {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "catalog property limit exceeded",
            ));
        }
        for property in &schema.properties {
            validate_required_id_path(&property.name, "property name")?;
            enum_label_count = enum_label_count
                .checked_add(property.enum_labels.len())
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "catalog enum label count overflow",
                    )
                })?;
            if enum_label_count > MAX_BATCH_ITEMS {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "catalog enum label limit exceeded",
                ));
            }
            if property.value_type != FlowValueType::Enum && !property.enum_labels.is_empty() {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "non-enum catalog property contains enum labels",
                ));
            }
            for label in &property.enum_labels {
                if label.len() > MAX_STRING_BYTES {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "catalog enum label exceeds string limit",
                    ));
                }
            }
            if property.value_type != FlowValueType::ViewModel
                && property.referenced_schema_name.is_some()
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "non-view-model catalog property contains a schema reference",
                ));
            }
            if let Some(referenced_schema_name) = &property.referenced_schema_name {
                validate_required_id_path(referenced_schema_name, "referenced schema name")?;
                if !catalog
                    .schemas
                    .iter()
                    .any(|candidate| candidate.name == *referenced_schema_name)
                {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "catalog property references an unknown schema",
                    ));
                }
            }
        }
    }
    for template in &catalog.templates {
        validate_required_id_path(&template.schema_name, "schema name")?;
        if template
            .authored_name
            .as_ref()
            .is_some_and(|name| name.len() > MAX_ID_PATH_BYTES)
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "authored instance name exceeds identifier limit",
            ));
        }
    }
    for instance in &catalog.instances {
        validate_required_id_path(&instance.schema_name, "schema name")?;
        if instance
            .authored_name
            .as_ref()
            .is_some_and(|name| name.len() > MAX_ID_PATH_BYTES)
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "authored instance name exceeds identifier limit",
            ));
        }
    }
    if catalog_payload_bytes(catalog)? > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded catalog exceeds 4 MiB",
        ));
    }
    Ok(())
}

fn catalog_payload_bytes(catalog: &FlowCatalog) -> Result<usize, FlowSessionError> {
    let mut total = 32_usize;
    for schema in &catalog.schemas {
        checked_payload_add(&mut total, 16)?;
        checked_payload_add(&mut total, schema.name.len())?;
        for property in &schema.properties {
            checked_payload_add(&mut total, 32)?;
            checked_payload_add(&mut total, property.name.len())?;
            checked_payload_add(
                &mut total,
                property
                    .referenced_schema_name
                    .as_deref()
                    .map(str::len)
                    .unwrap_or(0),
            )?;
            for label in &property.enum_labels {
                checked_payload_add(&mut total, 8)?;
                checked_payload_add(&mut total, label.len())?;
            }
        }
    }
    for template in &catalog.templates {
        checked_payload_add(&mut total, 24)?;
        checked_payload_add(&mut total, template.schema_name.len())?;
        checked_payload_add(
            &mut total,
            template.authored_name.as_deref().map(str::len).unwrap_or(0),
        )?;
    }
    for instance in &catalog.instances {
        checked_payload_add(&mut total, 32)?;
        checked_payload_add(&mut total, instance.schema_name.len())?;
        checked_payload_add(
            &mut total,
            instance.authored_name.as_deref().map(str::len).unwrap_or(0),
        )?;
    }
    Ok(total)
}

fn value_arena_payload_bytes(arena: &FlowValueArena) -> Result<usize, FlowSessionError> {
    let mut total = 0_usize;
    checked_payload_add(&mut total, arena.roots.len().saturating_mul(16))?;
    checked_payload_add(&mut total, arena.nodes.len().saturating_mul(16))?;
    for node in &arena.nodes {
        match &node.value {
            FlowValue::String(value) => checked_payload_add(&mut total, value.len())?,
            FlowValue::Object(edges) | FlowValue::ViewModel(edges) => {
                for (name, _) in edges {
                    checked_payload_add(&mut total, 4)?;
                    checked_payload_add(&mut total, name.len())?;
                }
            }
            FlowValue::List(items) => {
                checked_payload_add(&mut total, items.len().saturating_mul(4))?;
            }
            FlowValue::Null
            | FlowValue::Number(_)
            | FlowValue::Bool(_)
            | FlowValue::Enum(_)
            | FlowValue::ListIndex(_)
            | FlowValue::Color(_)
            | FlowValue::Image(_) => {}
        }
    }
    Ok(total)
}

fn validate_bootstrap_payload(bootstrap: &FlowBootstrap) -> Result<(), FlowSessionError> {
    validate_catalog(&bootstrap.catalog)?;
    validate_optional_selector(bootstrap.artboard_name.as_deref(), "artboard name")?;
    validate_optional_selector(bootstrap.player.name.as_deref(), "player name")?;
    let bounds = bootstrap.bounds;
    if !bounds.x.is_finite()
        || !bounds.y.is_finite()
        || !bounds.width.is_finite()
        || !bounds.height.is_finite()
        || bounds.width <= 0.0
        || bounds.height <= 0.0
        || !(bounds.x + bounds.width).is_finite()
        || !(bounds.y + bounds.height).is_finite()
    {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            "artboard bounds are invalid",
        ));
    }
    if bootstrap.values.nodes.len() > MAX_VALUE_NODES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "value arena node limit exceeded",
        ));
    }
    let mut total = catalog_payload_bytes(&bootstrap.catalog)?;
    checked_payload_add(&mut total, 64)?;
    checked_payload_add(
        &mut total,
        bootstrap
            .artboard_name
            .as_deref()
            .map(str::len)
            .unwrap_or(0),
    )?;
    checked_payload_add(
        &mut total,
        bootstrap.player.name.as_deref().map(str::len).unwrap_or(0),
    )?;
    checked_payload_add(&mut total, value_arena_payload_bytes(&bootstrap.values)?)?;
    if total > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded bootstrap exceeds 4 MiB",
        ));
    }
    Ok(())
}

fn flow_value_type_for_property(type_name: &str) -> FlowValueType {
    match type_name {
        "ViewModelPropertyString" => FlowValueType::String,
        "ViewModelPropertyNumber" | "ViewModelPropertyInteger" => FlowValueType::Number,
        "ViewModelPropertyBoolean" => FlowValueType::Bool,
        "ViewModelPropertyEnum"
        | "ViewModelPropertyEnumCustom"
        | "ViewModelPropertyEnumSystem"
        | "ViewModelPropertyArtboard" => FlowValueType::Enum,
        "ViewModelPropertySymbolListIndex" => FlowValueType::ListIndex,
        "ViewModelPropertyColor" => FlowValueType::Color,
        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => FlowValueType::Image,
        "ViewModelPropertyList" => FlowValueType::List,
        "ViewModelPropertyViewModel" => FlowValueType::ViewModel,
        "ViewModelPropertyTrigger" => FlowValueType::Trigger,
        _ => FlowValueType::Null,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::File;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};

    const FIXTURE: &[u8] = include_bytes!("../../../fixtures/graph/dependency_test.riv");
    const SMI_FIXTURE: &[u8] = include_bytes!("../../../fixtures/animation/smi_test.riv");
    const TWO_ARTBOARDS_FIXTURE: &[u8] =
        include_bytes!("../../../fixtures/minimal/two_artboards.riv");

    fn smi_session() -> FlowSession {
        let file = Arc::new(File::import(SMI_FIXTURE).expect("import SMI fixture"));
        FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some("artboard to nest".to_owned()),
                player: Some(FlowPlayerSelector::StateMachine(
                    "State Machine 1".to_owned(),
                )),
            },
        )
        .expect("create SMI session")
        .0
    }

    fn authoring_record(
        type_name: &str,
        properties: Vec<(&str, AuthoringValue)>,
    ) -> AuthoringRecord {
        let definition =
            nuxie_schema::definition_by_name(type_name).expect("text-run fixture record type");
        let properties = properties
            .into_iter()
            .map(|(property_name, value)| {
                let property = std::iter::once(definition.name)
                    .chain(definition.ancestors.iter().copied())
                    .filter_map(nuxie_schema::definition_by_name)
                    .flat_map(|owner| owner.properties)
                    .find(|property| property.name == property_name)
                    .expect("text-run fixture property");
                AuthoringProperty {
                    key: property.key.int,
                    value,
                }
            })
            .collect();
        AuthoringRecord {
            type_key: definition.type_key.int,
            properties,
        }
    }

    fn text_run_session() -> FlowSession {
        let runtime = RuntimeFile::from_authoring_records(vec![
            authoring_record("Backboard", Vec::new()),
            authoring_record(
                "Artboard",
                vec![
                    ("name", AuthoringValue::String("Root".to_owned())),
                    ("width", AuthoringValue::Double(100.0)),
                    ("height", AuthoringValue::Double(100.0)),
                ],
            ),
            authoring_record(
                "Text",
                vec![("name", AuthoringValue::String("Text".to_owned()))],
            ),
            authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("name", AuthoringValue::String("headline".to_owned())),
                    ("text", AuthoringValue::String("initial".to_owned())),
                ],
            ),
            authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("name", AuthoringValue::String("headline".to_owned())),
                    ("text", AuthoringValue::String("duplicate".to_owned())),
                ],
            ),
            authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("name", AuthoringValue::String("group//headline".to_owned())),
                    ("text", AuthoringValue::String("literal".to_owned())),
                ],
            ),
        ])
        .expect("build text-run fixture");
        let file = Arc::new(File::from_runtime(runtime).expect("import text-run fixture"));
        FlowSession::create(file, FlowSessionConfig::default())
            .expect("create text-run session")
            .0
    }

    fn authored_nested_view_model_session() -> (FlowSession, FlowBootstrap) {
        let runtime = RuntimeFile::from_authoring_records(vec![
            authoring_record("Backboard", Vec::new()),
            authoring_record(
                "ViewModel",
                vec![("name", AuthoringValue::String("Root".to_owned()))],
            ),
            authoring_record(
                "ViewModelInstance",
                vec![
                    ("name", AuthoringValue::String("Root defaults".to_owned())),
                    ("viewModelId", AuthoringValue::Uint(0)),
                ],
            ),
            authoring_record(
                "ViewModelInstanceViewModel",
                vec![
                    ("viewModelPropertyId", AuthoringValue::Uint(0)),
                    ("propertyValue", AuthoringValue::Uint(0)),
                ],
            ),
            authoring_record(
                "ViewModelPropertyViewModel",
                vec![
                    ("name", AuthoringValue::String("paywall".to_owned())),
                    ("viewModelReferenceId", AuthoringValue::Uint(1)),
                ],
            ),
            authoring_record(
                "ViewModel",
                vec![("name", AuthoringValue::String("Paywall".to_owned()))],
            ),
            authoring_record(
                "ViewModelInstance",
                vec![
                    (
                        "name",
                        AuthoringValue::String("Paywall defaults".to_owned()),
                    ),
                    ("viewModelId", AuthoringValue::Uint(1)),
                ],
            ),
            authoring_record(
                "ViewModelInstanceString",
                vec![
                    ("viewModelPropertyId", AuthoringValue::Uint(0)),
                    ("propertyValue", AuthoringValue::String("pro".to_owned())),
                ],
            ),
            authoring_record(
                "ViewModelPropertyString",
                vec![(
                    "name",
                    AuthoringValue::String("selectedProductId".to_owned()),
                )],
            ),
            authoring_record(
                "Artboard",
                vec![
                    ("name", AuthoringValue::String("Projection".to_owned())),
                    ("width", AuthoringValue::Double(100.0)),
                    ("height", AuthoringValue::Double(100.0)),
                    ("viewModelId", AuthoringValue::Uint(0)),
                ],
            ),
        ])
        .expect("build authored nested view-model fixture");
        let file = Arc::new(
            File::from_runtime(runtime).expect("import authored nested view-model fixture"),
        );
        FlowSession::create(file, FlowSessionConfig::default())
            .expect("create authored nested view-model session")
    }

    fn external_fixture(relative: &str) -> Vec<u8> {
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        std::fs::read(
            std::path::PathBuf::from(root)
                .join("tests/unit_tests/assets")
                .join(relative),
        )
        .expect("read external fixture")
    }

    fn replace_fixture_string(mut bytes: Vec<u8>, from: &str, to: &str) -> Vec<u8> {
        assert!(from.len() < 0x80);
        assert!(to.len() < 0x80);
        let mut encoded_from = vec![from.len() as u8];
        encoded_from.extend_from_slice(from.as_bytes());
        let mut encoded_to = vec![to.len() as u8];
        encoded_to.extend_from_slice(to.as_bytes());
        let mut replacements = 0;
        let mut cursor = 0;
        while let Some(offset) = bytes[cursor..]
            .windows(encoded_from.len())
            .position(|candidate| candidate == encoded_from)
        {
            let start = cursor + offset;
            let end = start + encoded_from.len();
            bytes.splice(start..end, encoded_to.iter().copied());
            cursor = start + encoded_to.len();
            replacements += 1;
        }
        assert_eq!(replacements, 2, "fixture string occurrence count changed");
        bytes
    }

    fn same_name_player_fixture() -> (Arc<File>, Vec<u8>) {
        let bytes = replace_fixture_string(
            replace_fixture_string(SMI_FIXTURE.to_vec(), "Timeline 1", "Shared Player"),
            "State Machine 1",
            "Shared Player",
        );
        let file = Arc::new(File::import(&bytes).expect("import same-name player fixture"));
        (file, bytes)
    }

    fn arena_number(
        arena: &FlowValueArena,
        instance_id: FlowInstanceId,
        name: &str,
    ) -> Option<f32> {
        let root = arena.roots.iter().find(|(id, _)| *id == instance_id)?.1;
        let root_node = arena.nodes.iter().find(|node| node.id == root)?;
        let FlowValue::ViewModel(edges) = &root_node.value else {
            return None;
        };
        let value_id = edges.iter().find(|(property, _)| property == name)?.1;
        let value = arena.nodes.iter().find(|node| node.id == value_id)?;
        let FlowValue::Number(value) = value.value else {
            return None;
        };
        Some(value)
    }

    fn arena_string_path<'a>(
        arena: &'a FlowValueArena,
        instance_id: FlowInstanceId,
        path: &str,
    ) -> Option<&'a str> {
        let mut value_id = arena.roots.iter().find(|(id, _)| *id == instance_id)?.1;
        for segment in path.split('/') {
            let node = arena.nodes.iter().find(|node| node.id == value_id)?;
            let FlowValue::ViewModel(edges) = &node.value else {
                return None;
            };
            value_id = edges.iter().find(|(property, _)| property == segment)?.1;
        }
        let node = arena.nodes.iter().find(|node| node.id == value_id)?;
        let FlowValue::String(value) = &node.value else {
            return None;
        };
        Some(value)
    }

    fn arena_list_len(
        arena: &FlowValueArena,
        instance_id: FlowInstanceId,
        name: &str,
    ) -> Option<usize> {
        let root = arena.roots.iter().find(|(id, _)| *id == instance_id)?.1;
        let root_node = arena.nodes.iter().find(|node| node.id == root)?;
        let FlowValue::ViewModel(edges) = &root_node.value else {
            return None;
        };
        let value_id = edges.iter().find(|(property, _)| property == name)?.1;
        let value = arena.nodes.iter().find(|node| node.id == value_id)?;
        let FlowValue::List(items) = &value.value else {
            return None;
        };
        Some(items.len())
    }

    fn arena_list_items(
        arena: &FlowValueArena,
        instance_id: FlowInstanceId,
        name: &str,
    ) -> Option<Vec<FlowValueId>> {
        let root = arena.roots.iter().find(|(id, _)| *id == instance_id)?.1;
        let root_node = arena.nodes.iter().find(|node| node.id == root)?;
        let FlowValue::ViewModel(edges) = &root_node.value else {
            return None;
        };
        let value_id = edges.iter().find(|(property, _)| property == name)?.1;
        let value = arena.nodes.iter().find(|node| node.id == value_id)?;
        let FlowValue::List(items) = &value.value else {
            return None;
        };
        Some(items.clone())
    }

    fn outer_reference_diff_fixture(
        child_b_label: &str,
    ) -> (FlowValueArena, FlowValueArena, FlowCatalog) {
        let owner = FlowInstanceId::new(1).expect("owner id");
        let child_a = FlowInstanceId::new(2).expect("child A id");
        let child_b = FlowInstanceId::new(3).expect("child B id");
        let arena = |owner_child, child_b_label: &str| FlowValueArena {
            roots: vec![
                (owner, FlowValueId(1)),
                (child_a, FlowValueId(2)),
                (child_b, FlowValueId(4)),
            ],
            nodes: vec![
                FlowValueNode {
                    id: FlowValueId(1),
                    value: FlowValue::ViewModel(vec![("child".to_owned(), owner_child)]),
                },
                FlowValueNode {
                    id: FlowValueId(2),
                    value: FlowValue::ViewModel(vec![("label".to_owned(), FlowValueId(3))]),
                },
                FlowValueNode {
                    id: FlowValueId(3),
                    value: FlowValue::String("same".to_owned()),
                },
                FlowValueNode {
                    id: FlowValueId(4),
                    value: FlowValue::ViewModel(vec![("label".to_owned(), FlowValueId(5))]),
                },
                FlowValueNode {
                    id: FlowValueId(5),
                    value: FlowValue::String(child_b_label.to_owned()),
                },
            ],
        };
        let before = arena(FlowValueId(2), "same");
        let after = arena(FlowValueId(4), child_b_label);
        let catalog = FlowCatalog {
            instances: vec![
                FlowInstanceMetadata {
                    id: owner,
                    schema_name: "Main".to_owned(),
                    authored_name: None,
                    is_root: true,
                },
                FlowInstanceMetadata {
                    id: child_a,
                    schema_name: "Child".to_owned(),
                    authored_name: None,
                    is_root: false,
                },
                FlowInstanceMetadata {
                    id: child_b,
                    schema_name: "Child".to_owned(),
                    authored_name: None,
                    is_root: false,
                },
            ],
            root_instance_id: Some(owner),
            ..FlowCatalog::default()
        };
        (before, after, catalog)
    }

    #[test]
    fn same_valued_outer_view_model_replacement_emits_its_new_identity() {
        let (before, after, catalog) = outer_reference_diff_fixture("same");
        let changes = diff_value_arenas(&before, &after, &catalog).expect("diff references");
        assert_eq!(
            changes,
            vec![(
                FlowInstanceId::new(1).expect("owner id"),
                "child".to_owned(),
                Some(FlowStateChangeValue::ViewModelReference {
                    instance_id: FlowInstanceId::new(3).expect("child B id"),
                    schema_name: "Child".to_owned(),
                }),
            )]
        );
    }

    #[test]
    fn authored_outer_replacement_result_carries_values_while_scalar_only_result_does_not() {
        let (before, after, catalog) = outer_reference_diff_fixture("same");
        let changes = diff_value_arenas(&before, &after, &catalog).expect("diff references");
        let mut result = FlowResult::idle(false);
        let mut next_sequence = 1;
        for (instance_id, path, value) in changes {
            append_output(
                &mut result.outputs,
                &mut next_sequence,
                1,
                FlowOutputPhase::ViewModelChanges,
                FlowOutputPayload::StateChanged {
                    instance_id: Some(instance_id),
                    path,
                    value,
                    origin_mutation_id: None,
                },
            )
            .expect("append authored replacement");
        }
        include_reconciliation_values(&mut result, &after);
        assert_eq!(result.values.as_ref(), Some(&after));

        let mut scalar_result = FlowResult::idle(false);
        append_output(
            &mut scalar_result.outputs,
            &mut next_sequence,
            2,
            FlowOutputPhase::ViewModelChanges,
            FlowOutputPayload::StateChanged {
                instance_id: FlowInstanceId::new(1),
                path: "title".to_owned(),
                value: Some(FlowStateChangeValue::Scalar(FlowScalarValue::String(
                    "updated".to_owned(),
                ))),
                origin_mutation_id: None,
            },
        )
        .expect("append scalar change");
        include_reconciliation_values(&mut scalar_result, &after);
        assert!(scalar_result.values.is_none());
    }

    #[test]
    fn differing_outer_view_model_replacement_emits_identity_before_child_values_only_once() {
        let (before, mut after, catalog) = outer_reference_diff_fixture("different");
        after.roots.reverse();
        let changes = diff_value_arenas(&before, &after, &catalog).expect("diff references");
        assert_eq!(
            changes,
            vec![
                (
                    FlowInstanceId::new(1).expect("owner id"),
                    "child".to_owned(),
                    Some(FlowStateChangeValue::ViewModelReference {
                        instance_id: FlowInstanceId::new(3).expect("child B id"),
                        schema_name: "Child".to_owned(),
                    }),
                ),
                (
                    FlowInstanceId::new(3).expect("child B id"),
                    "label".to_owned(),
                    Some(FlowStateChangeValue::Scalar(FlowScalarValue::String(
                        "different".to_owned(),
                    ))),
                ),
            ],
            "the linked child root owns descendant changes without an owner-path duplicate",
        );
    }

    fn input_value(session: &mut FlowSession, name: &str) -> FlowScalarValue {
        session
            .perform(FlowOperation::Query(FlowQuery::PlayerInputs))
            .expect("query inputs")
            .player_inputs
            .expect("input snapshot")
            .into_iter()
            .find(|input| input.name.as_deref() == Some(name))
            .expect("named input")
            .value
    }

    #[test]
    fn explicit_missing_artboard_is_a_typed_not_found_without_fallback() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));

        let error = FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some("definitely-missing".to_owned()),
                player: None,
            },
        )
        .expect_err("an explicit missing artboard must not fall back");

        assert_eq!(error.kind(), FlowSessionErrorKind::NotFound);
    }

    #[test]
    fn named_player_selectors_match_cpp_lookup_namespaces_and_collisions() {
        let (file, bytes) = same_name_player_fixture();
        let create = |selector| {
            FlowSession::create(
                Arc::clone(&file),
                FlowSessionConfig {
                    artboard_name: Some("artboard to nest".to_owned()),
                    player: Some(selector),
                },
            )
        };

        let (_, state_machine) =
            create(FlowPlayerSelector::StateMachine("Shared Player".to_owned()))
                .expect("C++ stateMachineNamed finds its same-name state machine");
        assert_eq!(state_machine.player.kind, FlowPlayerKind::StateMachine);
        assert_eq!(
            state_machine.player.selection,
            FlowPlayerSelection::ExplicitStateMachine
        );
        assert_eq!(state_machine.player.name.as_deref(), Some("Shared Player"));

        let (_, animation) = create(FlowPlayerSelector::LinearAnimation(
            "Shared Player".to_owned(),
        ))
        .expect("C++ animationNamed finds its same-name linear animation");
        assert_eq!(animation.player.kind, FlowPlayerKind::LinearAnimation);
        assert_eq!(
            animation.player.selection,
            FlowPlayerSelection::ExplicitLinearAnimation
        );
        assert_eq!(animation.player.name.as_deref(), Some("Shared Player"));

        for (selector, expected) in [
            (
                FlowPlayerSelector::StateMachine("missing".to_owned()),
                "state machine 'missing' was not found",
            ),
            (
                FlowPlayerSelector::LinearAnimation("missing".to_owned()),
                "linear animation 'missing' was not found",
            ),
        ] {
            let error = create(selector).expect_err("an explicit missing name must not fall back");
            assert_eq!(error.kind(), FlowSessionErrorKind::NotFound);
            assert_eq!(error.to_string(), expected);
        }

        let Some(cpp_runner) = std::env::var_os("RIVE_GOLDEN_RUNNER") else {
            eprintln!("skipping C++ named-player lookup differential; set RIVE_GOLDEN_RUNNER");
            return;
        };
        let fixture_path = std::env::temp_dir().join(format!(
            "nuxie-named-player-collision-{}.riv",
            std::process::id()
        ));
        std::fs::write(&fixture_path, bytes).expect("write same-name C++ fixture");
        for selector_args in [
            ["--state-machine", "Shared Player"],
            ["--animation", "Shared Player"],
        ] {
            let output = std::process::Command::new(&cpp_runner)
                .args(["--file"])
                .arg(&fixture_path)
                .args(["--artboard", "artboard to nest", "--samples", "0"])
                .args(selector_args)
                .output()
                .expect("run pinned C++ named-player selector");
            assert!(
                output.status.success(),
                "pinned C++ rejected {selector_args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        for selector_args in [["--state-machine", "missing"], ["--animation", "missing"]] {
            let output = std::process::Command::new(&cpp_runner)
                .args(["--file"])
                .arg(&fixture_path)
                .args(["--artboard", "artboard to nest", "--samples", "0"])
                .args(selector_args)
                .output()
                .expect("run pinned C++ missing-player selector");
            assert!(
                !output.status.success(),
                "pinned C++ unexpectedly accepted {selector_args:?}"
            );
        }
        let _ = std::fs::remove_file(fixture_path);
    }

    #[test]
    fn fallback_skips_an_uninstantiable_authored_state_machine() {
        let file = Arc::new(File::import(TWO_ARTBOARDS_FIXTURE).expect("import fixture"));

        let (_, bootstrap) = FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some("Two".to_owned()),
                player: None,
            },
        )
        .expect("fall back to a static artboard");

        assert_eq!(bootstrap.player.kind, FlowPlayerKind::Static);
        assert_eq!(bootstrap.player.selection, FlowPlayerSelection::Static);
        assert_eq!(bootstrap.player.index, None);
        assert_eq!(bootstrap.player.name, None);
    }

    #[test]
    fn explicit_uninstantiable_state_machine_does_not_fall_back() {
        let file = Arc::new(File::import(TWO_ARTBOARDS_FIXTURE).expect("import fixture"));

        let error = FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some("Two".to_owned()),
                player: Some(FlowPlayerSelector::StateMachine(
                    "Auto Generated State Machine".to_owned(),
                )),
            },
        )
        .expect_err("an explicit uninstantiable state machine must fail");

        assert_eq!(error.kind(), FlowSessionErrorKind::Runtime);
        assert_eq!(
            error.to_string(),
            "selected state machine could not be instantiated"
        );
    }

    #[test]
    fn bootstrap_query_returns_creation_snapshot_without_ordered_outputs() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (mut session, bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");

        let result = session
            .perform(FlowOperation::Query(FlowQuery::Bootstrap))
            .expect("query bootstrap");

        assert!(result.outputs.is_empty());
        assert_eq!(result.snapshot, Some(bootstrap));
        assert!(!result.dirty);
    }

    #[test]
    fn text_run_batch_marks_only_actual_text_changes_dirty_and_immediately_wakeable() {
        let mut session = text_run_session();
        let operation = |text: &str| {
            FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![FlowTextRunMutation {
                    name: "headline".to_owned(),
                    text: text.to_owned(),
                }],
            })
        };

        let changed = session
            .perform(operation("updated"))
            .expect("change named text run");
        assert!(changed.dirty);
        assert_eq!(changed.wake_after_seconds, Some(0.0));
        assert!(changed.outputs.is_empty());

        let unchanged = session
            .perform(operation("updated"))
            .expect("repeat named text run value");
        assert!(!unchanged.dirty);
        assert_eq!(unchanged.wake_after_seconds, None);
        assert!(unchanged.outputs.is_empty());

        let empty = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch::default()))
            .expect("an empty text-run batch is a clean no-op");
        assert!(!empty.dirty);
        assert_eq!(empty.wake_after_seconds, None);
    }

    #[test]
    fn text_run_batch_resolves_every_root_name_before_any_write() {
        let mut session = text_run_session();
        let error = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![
                    FlowTextRunMutation {
                        name: "headline".to_owned(),
                        text: "updated".to_owned(),
                    },
                    FlowTextRunMutation {
                        name: "missing".to_owned(),
                        text: "ignored".to_owned(),
                    },
                ],
            }))
            .expect_err("a missing root run rejects the complete batch");

        assert_eq!(error.kind(), FlowSessionErrorKind::NotFound);
        assert_eq!(error.message(), "root TextValueRun 'missing' was not found");
        let unchanged = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![FlowTextRunMutation {
                    name: "headline".to_owned(),
                    text: "initial".to_owned(),
                }],
            }))
            .expect("failed batch leaves the first run unchanged");
        assert!(!unchanged.dirty);
    }

    #[test]
    fn text_run_name_treats_slashes_as_literal_authored_characters() {
        let mut session = text_run_session();
        let result = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![FlowTextRunMutation {
                    name: "group//headline".to_owned(),
                    text: "updated".to_owned(),
                }],
            }))
            .expect("an exact authored name is not interpreted as a path");

        assert!(result.dirty);
        assert_eq!(result.wake_after_seconds, Some(0.0));
    }

    #[test]
    fn text_run_batch_enforces_item_name_text_and_aggregate_payload_bounds() {
        let mutation = |name: String, text: String| FlowTextRunMutation { name, text };

        let mut session = text_run_session();
        let error = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![
                    mutation("headline".to_owned(), String::new());
                    MAX_BATCH_ITEMS + 1
                ],
            }))
            .expect_err("batch item count is bounded");
        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);

        let error = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![mutation(String::new(), "text".to_owned())],
            }))
            .expect_err("text-run names are required");
        assert_eq!(error.kind(), FlowSessionErrorKind::InvalidArgument);
        assert_eq!(error.message(), "text-run name must not be empty");

        let error = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![mutation(
                    "n".repeat(MAX_ID_PATH_BYTES + 1),
                    "text".to_owned(),
                )],
            }))
            .expect_err("text-run names are bounded");
        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);

        let error = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![mutation(
                    "headline".to_owned(),
                    "t".repeat(MAX_STRING_BYTES + 1),
                )],
            }))
            .expect_err("individual text values are bounded");
        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(error.message(), "text-run value exceeds 1 MiB");

        let one_mib = "t".repeat(MAX_STRING_BYTES);
        let error = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![mutation("headline".to_owned(), one_mib.clone()); 5],
            }))
            .expect_err("aggregate text is bounded");
        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(error.message(), "text-run batch text exceeds 4 MiB");

        let error = session
            .perform(FlowOperation::TextRunBatch(FlowTextRunBatch {
                mutations: vec![mutation("headline".to_owned(), one_mib); 4],
            }))
            .expect_err("names also count toward the operation payload envelope");
        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(error.message(), "encoded text-run batch exceeds 4 MiB");
    }

    #[test]
    fn bootstrap_query_remains_the_immutable_creation_snapshot() {
        let file = Arc::new(
            File::import(&external_fixture("data_binding_test_2.riv"))
                .expect("import data-bind fixture"),
        );
        let (mut session, bootstrap) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create data-bind session");
        let root = bootstrap.catalog.root_instance_id.expect("root id");

        session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(root),
                    path: "num".to_owned(),
                    value: FlowScalarValue::Number(999.0),
                }],
                new_instances: Vec::new(),
            }))
            .expect("mutate live state");

        let queried = session
            .perform(FlowOperation::Query(FlowQuery::Bootstrap))
            .expect("query bootstrap")
            .snapshot
            .expect("bootstrap snapshot");
        assert_eq!(queried, bootstrap);
        assert_ne!(
            session
                .perform(FlowOperation::Query(FlowQuery::Values))
                .expect("query values")
                .values
                .expect("live values"),
            bootstrap.values,
        );
    }

    #[test]
    fn combined_state_batch_limit_is_atomic() {
        let mut session = smi_session();
        let before_input = input_value(&mut session, "bool");
        let before_catalog = session.bootstrap.catalog.clone();
        let before_values = session.bootstrap.values.clone();
        let before_cycle = session.next_cycle;
        let before_sequence = session.next_sequence;
        let mutation = FlowStateMutation::SetInputBool {
            name: "bool".to_owned(),
            value: true,
        };
        let new_instance = FlowNewInstance {
            local_id: 1,
            schema_name: "unused".to_owned(),
            authored_instance_name: None,
        };

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![mutation; MAX_BATCH_ITEMS / 2 + 1],
                new_instances: vec![new_instance; MAX_BATCH_ITEMS / 2],
            }))
            .expect_err("combined batch count must be bounded");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(input_value(&mut session, "bool"), before_input);
        assert_eq!(session.bootstrap.catalog, before_catalog);
        assert_eq!(session.bootstrap.values, before_values);
        assert_eq!(session.next_cycle, before_cycle);
        assert_eq!(session.next_sequence, before_sequence);
    }

    #[test]
    fn counter_preflight_failure_leaves_state_and_counters_unchanged() {
        for exhaust_sequence in [false, true] {
            let mut session = smi_session();
            if exhaust_sequence {
                session.next_sequence = u64::MAX;
            } else {
                session.next_cycle = u64::MAX;
            }
            let before_input = input_value(&mut session, "bool");
            let before_catalog = session.bootstrap.catalog.clone();
            let before_values = session.bootstrap.values.clone();
            let before_cycle = session.next_cycle;
            let before_sequence = session.next_sequence;

            let error = session
                .perform(FlowOperation::StateBatch(FlowStateBatch {
                    host_mutation_id: Some(1),
                    mutations: vec![FlowStateMutation::SetInputBool {
                        name: "bool".to_owned(),
                        value: true,
                    }],
                    new_instances: Vec::new(),
                }))
                .expect_err("counter exhaustion must fail before commit");

            assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
            assert_eq!(input_value(&mut session, "bool"), before_input);
            assert_eq!(session.bootstrap.catalog, before_catalog);
            assert_eq!(session.bootstrap.values, before_values);
            assert_eq!(session.next_cycle, before_cycle);
            assert_eq!(session.next_sequence, before_sequence);
        }
    }

    #[test]
    fn value_arena_preflight_failure_leaves_catalog_ids_and_counters_unchanged() {
        let file = Arc::new(
            File::import(&external_fixture("data_binding_test.riv"))
                .expect("import data-bind fixture"),
        );
        let (mut session, bootstrap) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create data-bind session");
        let schema = bootstrap
            .catalog
            .schemas
            .iter()
            .filter(|schema| !schema.properties.is_empty())
            .max_by_key(|schema| schema.properties.len())
            .expect("fixture has a nonempty schema");
        let count = MAX_VALUE_EDGES
            .checked_div(schema.properties.len())
            .and_then(|value| value.checked_add(1))
            .expect("bounded instance count");
        assert!(count <= MAX_BATCH_ITEMS);
        assert!(session.instances.len().saturating_add(count) <= MAX_INSTANCES);
        let before_catalog = session.bootstrap.catalog.clone();
        let before_values = session.bootstrap.values.clone();
        let before_next_id = session.next_instance_id;
        let before_cycle = session.next_cycle;
        let before_sequence = session.next_sequence;
        let new_instances = (0..count)
            .map(|local_id| FlowNewInstance {
                local_id: u32::try_from(local_id).expect("local id"),
                schema_name: schema.name.clone(),
                authored_instance_name: None,
            })
            .collect();

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: Vec::new(),
                new_instances,
            }))
            .expect_err("candidate arena must exceed the edge envelope");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(session.bootstrap.catalog, before_catalog);
        assert_eq!(session.bootstrap.values, before_values);
        assert_eq!(session.next_instance_id, before_next_id);
        assert_eq!(session.next_cycle, before_cycle);
        assert_eq!(session.next_sequence, before_sequence);
    }

    #[test]
    fn advance_uses_required_caller_delta_and_allows_equal_timestamps() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");

        for delta_seconds in [0.25, 0.0] {
            let result = session
                .perform(FlowOperation::Advance(FlowAdvance {
                    timestamp_seconds: 10.0,
                    delta_seconds,
                    render: false,
                }))
                .expect("nondecreasing app clock");
            assert!(result.outputs.iter().any(|output| {
                output.phase == FlowOutputPhase::RuntimeAdvance
                    && output.payload == FlowOutputPayload::RuntimeAdvanced { delta_seconds }
            }));
        }
    }

    #[test]
    fn pointer_batch_limit_is_prevalidated() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");
        let event = FlowPointerEvent {
            kind: FlowPointerKind::Move,
            pointer_id: 1,
            x: 0.0,
            y: 0.0,
            timestamp_seconds: 0.0,
        };

        let error = session
            .perform(FlowOperation::PointerBatch(FlowPointerBatch {
                events: vec![event; MAX_POINTERS_PER_BATCH + 1],
            }))
            .expect_err("oversized pointer batch");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
    }

    #[test]
    fn pointer_timestamps_are_prevalidated_before_any_event_mutates_the_session() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");
        let before_cycle = session.next_cycle;
        let before_sequence = session.next_sequence;
        let before_active_pointers = session.active_pointer_ids.clone();

        let error = session
            .perform(FlowOperation::PointerBatch(FlowPointerBatch {
                events: vec![
                    FlowPointerEvent {
                        kind: FlowPointerKind::Down,
                        pointer_id: 1,
                        x: 0.0,
                        y: 0.0,
                        timestamp_seconds: 1.0,
                    },
                    FlowPointerEvent {
                        kind: FlowPointerKind::Move,
                        pointer_id: 1,
                        x: 1.0,
                        y: 1.0,
                        timestamp_seconds: f32::INFINITY,
                    },
                ],
            }))
            .expect_err("invalid timestamp must reject the complete batch");

        assert_eq!(error.kind(), FlowSessionErrorKind::InvalidArgument);
        assert_eq!(session.next_cycle, before_cycle);
        assert_eq!(session.next_sequence, before_sequence);
        assert_eq!(session.active_pointer_ids, before_active_pointers);
        assert!(session.terminal_failure.is_none());
        session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("prevalidation failure leaves the session usable");
    }

    #[test]
    fn state_batch_prevalidates_all_input_writes_before_commit() {
        let mut session = smi_session();
        assert_eq!(
            input_value(&mut session, "bool"),
            FlowScalarValue::Bool(false)
        );

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(7),
                mutations: vec![
                    FlowStateMutation::SetInputBool {
                        name: "bool".to_owned(),
                        value: true,
                    },
                    FlowStateMutation::SetInputNumber {
                        name: "bool".to_owned(),
                        value: 1.0,
                    },
                ],
                new_instances: Vec::new(),
            }))
            .expect_err("wrong-kind second write rejects whole batch");

        assert_eq!(error.kind(), FlowSessionErrorKind::InvalidArgument);
        assert_eq!(
            input_value(&mut session, "bool"),
            FlowScalarValue::Bool(false)
        );
    }

    #[test]
    fn state_batch_commits_inputs_with_direct_origin_echo_then_advances() {
        let mut session = smi_session();
        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(42),
                mutations: vec![
                    FlowStateMutation::SetInputBool {
                        name: "bool".to_owned(),
                        value: true,
                    },
                    FlowStateMutation::SetInputNumber {
                        name: "num".to_owned(),
                        value: 12.5,
                    },
                    FlowStateMutation::FireInputTrigger {
                        name: "trig".to_owned(),
                    },
                ],
                new_instances: Vec::new(),
            }))
            .expect("valid state batch");

        assert_eq!(
            input_value(&mut session, "bool"),
            FlowScalarValue::Bool(true)
        );
        assert_eq!(
            input_value(&mut session, "num"),
            FlowScalarValue::Number(12.5)
        );
        assert_eq!(result.outputs.len(), 3);
        assert!(result.outputs.iter().all(|output| {
            output.phase == FlowOutputPhase::ViewModelChanges
                && matches!(
                    &output.payload,
                    FlowOutputPayload::StateChanged {
                        origin_mutation_id: Some(42),
                        ..
                    }
                )
        }));
        assert!(
            result
                .outputs
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );

        session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.016,
                render: true,
            }))
            .expect("advance committed state");
    }

    #[test]
    fn view_model_batch_is_atomic_and_refreshes_the_value_arena() {
        let file = Arc::new(
            File::import(&external_fixture("data_binding_test_2.riv"))
                .expect("import data-bind fixture"),
        );
        let (mut session, bootstrap) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create data-bind session");
        let root = bootstrap
            .catalog
            .root_instance_id
            .expect("retained root id");
        let initial = arena_number(&bootstrap.values, root, "num").expect("initial number");

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(90),
                mutations: vec![
                    FlowStateMutation::SetValue {
                        instance: FlowInstanceRef::Existing(root),
                        path: "num".to_owned(),
                        value: FlowScalarValue::Number(initial + 10.0),
                    },
                    FlowStateMutation::SetValue {
                        instance: FlowInstanceRef::Existing(root),
                        path: "num".to_owned(),
                        value: FlowScalarValue::Bool(true),
                    },
                ],
                new_instances: Vec::new(),
            }))
            .expect_err("wrong-kind second mutation rejects whole batch");
        assert_eq!(error.kind(), FlowSessionErrorKind::NotFound);
        let unchanged = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_number(&unchanged, root, "num"), Some(initial));

        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(91),
                mutations: vec![FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(root),
                    path: "num".to_owned(),
                    value: FlowScalarValue::Number(137.0),
                }],
                new_instances: Vec::new(),
            }))
            .expect("valid VM mutation");
        assert!(matches!(
            result.outputs.as_slice(),
            [FlowOutput {
                payload: FlowOutputPayload::StateChanged {
                    origin_mutation_id: Some(91),
                    ..
                },
                ..
            }]
        ));
        let values = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_number(&values, root, "num"), Some(137.0));
    }

    #[test]
    fn list_mutations_are_sequentially_simulated_before_any_commit() {
        let file = Arc::new(
            File::import(&external_fixture("component_list_1.riv")).expect("import list fixture"),
        );
        let (mut session, bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create list session");
        let root = bootstrap
            .catalog
            .root_instance_id
            .expect("retained root id");
        let initial_instance_count = bootstrap.catalog.instances.len();
        let initial_len = arena_list_len(&bootstrap.values, root, "Buttons").expect("initial list");
        assert!(initial_len > 0);
        let new_instance = FlowNewInstance {
            local_id: 10,
            schema_name: "ItemVM".to_owned(),
            authored_instance_name: Some("Instance 3".to_owned()),
        };

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    FlowStateMutation::ListClear {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(10),
                    },
                    FlowStateMutation::ListRemove {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 1,
                    },
                ],
                new_instances: vec![new_instance.clone()],
            }))
            .expect_err("post-insert out-of-range remove rejects whole batch");
        assert_eq!(error.kind(), FlowSessionErrorKind::InvalidArgument);
        let unchanged = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(
            arena_list_len(&unchanged, root, "Buttons"),
            Some(initial_len)
        );
        assert_eq!(
            session
                .perform(FlowOperation::Query(FlowQuery::Catalog))
                .expect("query catalog")
                .catalog
                .expect("catalog")
                .instances
                .len(),
            initial_instance_count
        );

        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(101),
                mutations: vec![
                    FlowStateMutation::ListClear {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(10),
                    },
                ],
                new_instances: vec![new_instance],
            }))
            .expect("valid sequential list batch");
        assert_eq!(result.created_instances.len(), 1);
        let values = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_list_len(&values, root, "Buttons"), Some(1));
        let catalog = session
            .perform(FlowOperation::Query(FlowQuery::Catalog))
            .expect("query catalog")
            .catalog
            .expect("catalog");
        assert_eq!(catalog.instances.len(), initial_instance_count + 1);
        assert_eq!(
            catalog.instances.last().map(|instance| instance.id),
            Some(result.created_instances[0].id)
        );
    }

    #[test]
    fn list_order_exposes_stable_instance_identity_for_equal_instances() {
        let file = Arc::new(
            File::import(&external_fixture("component_list_1.riv")).expect("import list fixture"),
        );
        let (mut session, bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create list session");
        let root = bootstrap.catalog.root_instance_id.expect("root id");
        let created = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    FlowStateMutation::ListClear {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(1),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 1,
                        item: FlowInstanceRef::New(2),
                    },
                ],
                new_instances: vec![
                    FlowNewInstance {
                        local_id: 1,
                        schema_name: "ItemVM".to_owned(),
                        authored_instance_name: None,
                    },
                    FlowNewInstance {
                        local_id: 2,
                        schema_name: "ItemVM".to_owned(),
                        authored_instance_name: None,
                    },
                ],
            }))
            .expect("insert equal default instances")
            .created_instances;
        let first_id = created[0].id;
        let second_id = created[1].id;
        let before = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("values");
        let first_node = before
            .roots
            .iter()
            .find(|(id, _)| *id == first_id)
            .expect("first root")
            .1;
        let second_node = before
            .roots
            .iter()
            .find(|(id, _)| *id == second_id)
            .expect("second root")
            .1;
        assert_eq!(
            arena_list_items(&before, root, "Buttons"),
            Some(vec![first_node, second_node])
        );

        session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![FlowStateMutation::ListSwap {
                    instance: FlowInstanceRef::Existing(root),
                    path: "Buttons".to_owned(),
                    first: 0,
                    second: 1,
                }],
                new_instances: Vec::new(),
            }))
            .expect("swap equal instances");
        let after = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("values");
        let first_node = after
            .roots
            .iter()
            .find(|(id, _)| *id == first_id)
            .expect("first root")
            .1;
        let second_node = after
            .roots
            .iter()
            .find(|(id, _)| *id == second_id)
            .expect("second root")
            .1;
        assert_eq!(
            arena_list_items(&after, root, "Buttons"),
            Some(vec![second_node, first_node])
        );
        assert_eq!(
            diff_value_arenas(&before, &after, &FlowCatalog::default())
                .expect("diff equal-valued list identities"),
            vec![(root, "Buttons".to_owned(), None)]
        );
    }

    #[test]
    fn bootstrap_preserves_nonzero_authored_artboard_origin() {
        let file = Arc::new(
            File::import(&external_fixture("db_health_tracker.riv"))
                .expect("import origin fixture"),
        );
        let (name, expected) = file
            .artboards()
            .find_map(|artboard| {
                let instance =
                    OwnedArtboardInstance::instantiate(Arc::clone(&file), artboard.index()).ok()?;
                let bounds = instance.artboard_bounds();
                ((bounds.0 != 0.0 || bounds.1 != 0.0) && artboard.name().is_some())
                    .then(|| (artboard.name().unwrap_or_default().to_owned(), bounds))
            })
            .expect("fixture has nonzero-origin artboard");

        let (_, bootstrap) = FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some(name),
                player: None,
            },
        )
        .expect("create nonzero-origin session");

        assert_eq!(
            bootstrap.bounds,
            FlowArtboardBounds {
                x: expected.0,
                y: expected.1,
                width: expected.2,
                height: expected.3
            },
        );
        assert!(bootstrap.bounds.x != 0.0 || bootstrap.bounds.y != 0.0);
    }

    #[test]
    fn atomic_validation_preserves_cross_root_list_aliases_and_does_not_consume_ids() {
        let file = Arc::new(
            File::import(&external_fixture("component_list_1.riv")).expect("import list fixture"),
        );
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create list session");
        let initial_instance_count = session.bootstrap.catalog.instances.len();
        let created = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![FlowStateMutation::ListInsert {
                    instance: FlowInstanceRef::New(1),
                    path: "Buttons".to_owned(),
                    index: 0,
                    item: FlowInstanceRef::New(2),
                }],
                new_instances: vec![
                    FlowNewInstance {
                        local_id: 1,
                        schema_name: "MainVM".to_owned(),
                        authored_instance_name: None,
                    },
                    FlowNewInstance {
                        local_id: 2,
                        schema_name: "MainVM".to_owned(),
                        authored_instance_name: None,
                    },
                ],
            }))
            .expect("create aliased roots")
            .created_instances;
        let a = created[0].id;
        let b = created[1].id;

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(b),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(3),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(b),
                        path: "Buttons".to_owned(),
                        index: 1,
                        item: FlowInstanceRef::Existing(a),
                    },
                ],
                new_instances: vec![FlowNewInstance {
                    local_id: 3,
                    schema_name: "MainVM".to_owned(),
                    authored_instance_name: None,
                }],
            }))
            .expect_err("B to A closes the existing A to B cycle");
        assert_eq!(error.kind(), FlowSessionErrorKind::Conflict);
        let values = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_list_len(&values, a, "Buttons"), Some(1));
        assert_eq!(arena_list_len(&values, b, "Buttons"), Some(0));
        assert_eq!(
            session
                .perform(FlowOperation::Query(FlowQuery::Catalog))
                .expect("query catalog")
                .catalog
                .expect("catalog")
                .instances
                .len(),
            initial_instance_count + 2
        );

        let next = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: Vec::new(),
                new_instances: vec![FlowNewInstance {
                    local_id: 4,
                    schema_name: "MainVM".to_owned(),
                    authored_instance_name: None,
                }],
            }))
            .expect("next id remains available")
            .created_instances[0]
            .id;
        assert_eq!(next.get(), b.get() + 1);
    }

    #[test]
    fn authored_default_nested_values_are_mutable_like_cpp_create_instance_from_index() {
        let (mut session, bootstrap) = authored_nested_view_model_session();
        let root = bootstrap
            .catalog
            .root_instance_id
            .expect("root instance identity");
        assert_eq!(
            arena_string_path(&bootstrap.values, root, "paywall/selectedProductId",),
            Some("pro"),
        );

        // C++ `ViewModelRuntime::createInstanceFromIndex` clones the authored
        // root and calls `File::completeViewModelInstance` before exposing it
        // (`src/viewmodel/runtime/viewmodel_runtime.cpp:111-120`,
        // `src/file.cpp:864-941`). The sibling C++ oracle test pins that this
        // makes nested authored scalars writable; FlowSession must create its
        // default through the equivalent completed path.
        session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(root),
                    path: "paywall/selectedProductId".to_owned(),
                    value: FlowScalarValue::String("basic".to_owned()),
                }],
                new_instances: Vec::new(),
            }))
            .expect("write nested authored value");

        let values = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query nested authored value")
            .values
            .expect("value arena");
        assert_eq!(
            arena_string_path(&values, root, "paywall/selectedProductId"),
            Some("basic"),
        );
    }

    #[test]
    fn events_produced_by_advance_are_returned_in_that_advance_and_do_not_replay() {
        let file = Arc::new(
            File::import(&external_fixture("events_on_states.riv")).expect("import event fixture"),
        );
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create event session");

        let first = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }))
            .expect("first advance");
        // C++ `StateMachineInstance::advance` applies the prior listener queue
        // first, then reports transitions produced during this call and
        // returns with those reports visible (`state_machine_instance.cpp:
        // 2519-2557`). The sibling C++ oracle pins exact 0,1,0 report counts
        // around trigger + advance(0); the FlowSession cycle must preserve
        // that same boundary.
        let event_position = first
            .outputs
            .iter()
            .position(|output| output.phase == FlowOutputPhase::ReportedEvents)
            .expect("event produced by first advance");
        let advance_position = first
            .outputs
            .iter()
            .position(|output| output.phase == FlowOutputPhase::RuntimeAdvance)
            .expect("first runtime advance");
        assert!(event_position < advance_position);
        assert_eq!(
            first.outputs[event_position].cycle,
            first.outputs[advance_position].cycle
        );
        assert!(
            !first
                .outputs
                .iter()
                .any(|output| { output.phase == FlowOutputPhase::DelayedEventCallbacks })
        );
        let mut reported = first
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                FlowOutputPayload::ReportedEvent { url, target, .. } => {
                    Some((url.clone(), target.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let second = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }))
            .expect("second advance");
        assert!(
            !second
                .outputs
                .iter()
                .any(|output| output.phase == FlowOutputPhase::ReportedEvents),
            "the next advance must not replay the prior cycle's reports",
        );
        for (timestamp_seconds, delta_seconds) in [(1.0, 1.0), (2.0, 1.0), (2.0, 0.0)] {
            let result = session
                .perform(FlowOperation::Advance(FlowAdvance {
                    timestamp_seconds,
                    delta_seconds,
                    render: false,
                }))
                .expect("advance through authored state events");
            reported.extend(
                result
                    .outputs
                    .iter()
                    .filter_map(|output| match &output.payload {
                        FlowOutputPayload::ReportedEvent { url, target, .. } => {
                            Some((url.clone(), target.clone()))
                        }
                        _ => None,
                    }),
            );
        }
        assert!(
            reported.contains(&(Some(String::new()), Some("_blank".to_owned()),)),
            "reported event metadata: {reported:?}",
        );
        assert!(
            reported.contains(&(None, None)),
            "ordinary events must preserve OpenURL metadata absence",
        );
    }

    #[test]
    fn synchronous_pointer_events_survive_the_followup_advance_once_in_authored_order() {
        let file = Arc::new(
            File::import(&external_fixture("event_on_listener.riv"))
                .expect("import listener event fixture"),
        );
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create event session");
        session
            .perform(FlowOperation::Advance(FlowAdvance::default()))
            .expect("prime listener hit targets");

        // The C++ reference fixture test reports exactly Footstep then Event 3
        // from pointer callbacks, before its next `advance(0)`, and that
        // advance consumes the listener queue (`tests/unit_tests/runtime/
        // state_machine_event_test.cpp`). FlowSession follows every Down/Up
        // callback with an advance, so it must retain the host-visible prefix
        // while appending any reports produced by that advance.
        let click = session
            .perform(FlowOperation::PointerBatch(FlowPointerBatch {
                events: vec![
                    FlowPointerEvent {
                        kind: FlowPointerKind::Down,
                        pointer_id: 1,
                        x: 343.0,
                        y: 116.0,
                        timestamp_seconds: 0.0,
                    },
                    FlowPointerEvent {
                        kind: FlowPointerKind::Up,
                        pointer_id: 1,
                        x: 343.0,
                        y: 116.0,
                        timestamp_seconds: 0.0,
                    },
                ],
            }))
            .expect("click listener target");
        let names = click
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                FlowOutputPayload::ReportedEvent { name, .. } => name.as_deref(),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(names, ["Footstep", "Event 3"]);

        let next = session
            .perform(FlowOperation::Advance(FlowAdvance::default()))
            .expect("advance after click reports");
        assert!(
            !next
                .outputs
                .iter()
                .any(|output| output.phase == FlowOutputPhase::ReportedEvents),
            "synchronous pointer reports must not replay",
        );
    }

    #[test]
    fn synchronous_and_advance_events_share_one_cycle_in_cpp_order_without_replay() {
        let file = Arc::new(
            File::import(&external_fixture("events_on_states.riv"))
                .expect("import combined listener/state event fixture"),
        );
        let (mut session, _) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create combined event session");
        session
            .perform(FlowOperation::Advance(FlowAdvance::default()))
            .expect("prime state and listener hit targets");

        // This fixture's listener synchronously reports Third, First. Its
        // two-second state advance then reports Second, Third. The reference
        // sequence is pinned by the sibling C++ probe against rive-runtime
        // `d788e8ec6e8b598526607d6a1e8818e8b637b60c`. C++ consumes the former
        // queue in `applyEvents` before advancing layers and producing the
        // latter (`state_machine_instance.cpp:2320-2335,2546-2584`).
        session
            .apply_pointer_event(FlowPointerEvent {
                kind: FlowPointerKind::Down,
                pointer_id: 1,
                x: 343.0,
                y: 116.0,
                timestamp_seconds: 0.0,
            })
            .expect("pointer down");
        session
            .apply_pointer_event(FlowPointerEvent {
                kind: FlowPointerKind::Up,
                pointer_id: 1,
                x: 343.0,
                y: 116.0,
                timestamp_seconds: 0.0,
            })
            .expect("pointer up");

        let combined = session
            .run_player_cycle(2.0, false, None)
            .expect("advance with synchronous reports pending");
        let names = combined
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                FlowOutputPayload::ReportedEvent { name, .. } => name.as_deref(),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(names, ["Third", "First", "Second", "Third"]);

        let next = session
            .run_player_cycle(0.0, false, None)
            .expect("advance after combined reports");
        assert!(
            !next
                .outputs
                .iter()
                .any(|output| output.phase == FlowOutputPhase::ReportedEvents),
            "neither the synchronous prefix nor post-advance suffix may replay",
        );
    }

    #[test]
    fn event_seconds_delay_is_immediate_overshoot_metadata_not_a_deadline() {
        let file = Arc::new(
            File::import(&external_fixture("timeline_event_test.riv"))
                .expect("import timeline event fixture"),
        );
        let (mut session, _) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create timeline event session");
        for (timestamp_seconds, delta_seconds) in [(0.0, 0.0), (0.4, 0.4)] {
            session
                .perform(FlowOperation::Advance(FlowAdvance {
                    timestamp_seconds,
                    delta_seconds,
                    render: false,
                }))
                .expect("prime timeline");
        }
        let produced = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.6,
                delta_seconds: 0.2,
                render: false,
            }))
            .expect("cross event keyframe");
        let reported = produced
            .outputs
            .iter()
            .find_map(|output| match &output.payload {
                FlowOutputPayload::ReportedEvent {
                    name,
                    delay_seconds,
                    ..
                } if name.as_deref() == Some("Half") => Some((*delay_seconds, output.phase)),
                _ => None,
            })
            .expect("half event report from the crossing advance");
        assert!((reported.0 - 0.1).abs() < 0.0001);
        assert_eq!(reported.1, FlowOutputPhase::ReportedEvents);
        assert!(
            !produced
                .outputs
                .iter()
                .any(|output| { output.phase == FlowOutputPhase::DelayedEventCallbacks })
        );
        assert_ne!(produced.wake_after_seconds, Some(reported.0));

        let next = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.6,
                delta_seconds: 0.0,
                render: false,
            }))
            .expect("advance after report");
        assert!(
            !next.outputs.iter().any(|output| matches!(
                &output.payload,
                FlowOutputPayload::ReportedEvent { name, .. }
                    if name.as_deref() == Some("Half")
            )),
            "the next advance must not replay Half",
        );
    }

    #[test]
    fn zero_host_mutation_id_is_preserved_as_present_origin_evidence() {
        let mut session = smi_session();

        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(0),
                mutations: vec![FlowStateMutation::SetInputBool {
                    name: "bool".to_owned(),
                    value: true,
                }],
                new_instances: Vec::new(),
            }))
            .expect("zero is a valid present mutation identity");

        assert!(matches!(
            result.outputs.as_slice(),
            [FlowOutput {
                payload: FlowOutputPayload::StateChanged {
                    origin_mutation_id: Some(0),
                    ..
                },
                ..
            }]
        ));
    }

    #[test]
    fn value_arena_accepts_exact_node_limit_and_rejects_the_next_node() {
        let file = File::import(FIXTURE).expect("import fixture");
        let mut builder = ValueArenaBuilder::new(&file);

        for _ in 0..MAX_VALUE_NODES {
            builder.push(FlowValue::Null).expect("node within cap");
        }
        let error = builder
            .push(FlowValue::Null)
            .expect_err("node beyond cap must be rejected");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(builder.arena.nodes.len(), MAX_VALUE_NODES);
    }

    #[test]
    fn output_validation_rejects_invalid_event_and_scalar_payloads_before_sequence_use() {
        let mut outputs = Vec::new();
        let mut next_sequence = 1;
        let non_finite = FlowOutputPayload::ReportedEvent {
            name: Some("event".to_owned()),
            event_type: 0,
            url: None,
            target: None,
            delay_seconds: 0.0,
            properties: vec![FlowEventProperty {
                name: Some("number".to_owned()),
                value: FlowScalarValue::Number(f32::NAN),
            }],
        };

        let non_finite_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ReportedEvents,
            non_finite,
        )
        .expect_err("non-finite event values must be rejected");
        assert_eq!(non_finite_error.kind(), FlowSessionErrorKind::Runtime);

        let oversized = FlowOutputPayload::StateChanged {
            instance_id: None,
            path: "value".to_owned(),
            value: Some(FlowStateChangeValue::Scalar(FlowScalarValue::String(
                "x".repeat(MAX_STRING_BYTES + 1),
            ))),
            origin_mutation_id: None,
        };
        let oversized_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ViewModelChanges,
            oversized,
        )
        .expect_err("oversized scalar strings must be rejected");

        assert_eq!(oversized_error.kind(), FlowSessionErrorKind::LimitExceeded);

        let open_url =
            |url: Option<String>, target: Option<String>| FlowOutputPayload::ReportedEvent {
                name: Some("open".to_owned()),
                event_type: 131,
                url,
                target,
                delay_seconds: 0.0,
                properties: Vec::new(),
            };
        let unpaired_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ReportedEvents,
            open_url(Some("https://nuxie.example".to_owned()), None),
        )
        .expect_err("OpenURL fields must be paired");
        assert_eq!(unpaired_error.kind(), FlowSessionErrorKind::Runtime);

        let oversized_url_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ReportedEvents,
            open_url(
                Some("x".repeat(MAX_STRING_BYTES + 1)),
                Some("_blank".to_owned()),
            ),
        )
        .expect_err("oversized OpenURL URLs must be rejected");
        assert_eq!(
            oversized_url_error.kind(),
            FlowSessionErrorKind::LimitExceeded
        );

        let oversized_target_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ReportedEvents,
            open_url(
                Some("https://nuxie.example".to_owned()),
                Some("x".repeat(MAX_ID_PATH_BYTES + 1)),
            ),
        )
        .expect_err("oversized OpenURL targets must be rejected");
        assert_eq!(
            oversized_target_error.kind(),
            FlowSessionErrorKind::LimitExceeded
        );

        let malformed_target_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ReportedEvents,
            open_url(
                Some("https://nuxie.example".to_owned()),
                Some("new-window".to_owned()),
            ),
        )
        .expect_err("noncanonical OpenURL targets must be rejected");
        assert_eq!(malformed_target_error.kind(), FlowSessionErrorKind::Runtime);

        assert!(outputs.is_empty());
        assert_eq!(next_sequence, 1);
    }

    #[test]
    fn creation_projection_reserves_one_shared_arena_for_bootstrap_and_host_work() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (_, bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");
        let creation_with_nodes = |node_count: usize| {
            let mut bootstrap = bootstrap.clone();
            bootstrap.values = FlowValueArena {
                roots: Vec::new(),
                nodes: (1..=node_count)
                    .map(|id| FlowValueNode {
                        id: FlowValueId(u32::try_from(id).expect("bounded value id")),
                        value: FlowValue::Null,
                    })
                    .collect(),
            };
            FlowSessionCreation {
                bootstrap,
                outputs: vec![FlowOutput {
                    sequence: 1,
                    cycle: 0,
                    phase: FlowOutputPhase::HostWork,
                    payload: FlowOutputPayload::HostCommand {
                        name: "created".to_owned(),
                        payload: FlowHostValue::Object(BTreeMap::new()),
                    },
                }],
                dirty: false,
                settled: true,
                wake_after_seconds: None,
            }
        };

        assert!(
            validate_creation_value_arena_bounds(&creation_with_nodes(MAX_VALUE_NODES - 1)).is_ok(),
            "bootstrap plus creation HostWork may exactly fill the shared arena"
        );
        let error = validate_creation_value_arena_bounds(&creation_with_nodes(MAX_VALUE_NODES))
            .expect_err("creation HostWork must not overflow the bootstrap arena");
        assert_eq!(error.kind(), FlowSessionErrorKind::ResultLimitExceeded);
    }

    #[test]
    fn result_projection_aggregates_many_individually_valid_host_trees() {
        let payload = FlowHostValue::Object(BTreeMap::from([(
            "value".to_owned(),
            FlowHostValue::List(vec![FlowHostValue::Bool(true); 16]),
        )]));
        let result_with_commands = |command_count: usize| FlowResult {
            outputs: (0..command_count)
                .map(|index| FlowOutput {
                    sequence: u64::try_from(index + 1).expect("bounded sequence"),
                    cycle: 1,
                    phase: FlowOutputPhase::HostWork,
                    payload: FlowOutputPayload::HostCommand {
                        name: "event".to_owned(),
                        payload: payload.clone(),
                    },
                })
                .collect(),
            ..FlowResult::idle(true)
        };

        let exact = result_with_commands(227);
        assert!(validate_output_batch(&exact.outputs).is_ok());
        assert!(validate_result_value_arena_bounds(&exact).is_ok());

        let overflow = result_with_commands(256);
        assert!(
            validate_output_batch(&overflow.outputs).is_ok(),
            "each command and the encoded byte aggregate remain independently valid"
        );
        let error = validate_result_value_arena_bounds(&overflow)
            .expect_err("all command trees share one ABI result arena");
        assert_eq!(error.kind(), FlowSessionErrorKind::ResultLimitExceeded);
    }

    #[test]
    fn result_projection_combines_snapshot_and_host_content_into_the_abi_budget() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (_, mut bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");
        let snapshot_chunk = "s".repeat(MAX_STRING_BYTES);
        bootstrap.values = FlowValueArena {
            roots: Vec::new(),
            nodes: (1..=3)
                .map(|id| FlowValueNode {
                    id: FlowValueId(id),
                    value: FlowValue::String(snapshot_chunk.clone()),
                })
                .collect(),
        };
        let host_chunk = "h".repeat(600 * 1024);
        let output = FlowOutput {
            sequence: 1,
            cycle: 1,
            phase: FlowOutputPhase::HostWork,
            payload: FlowOutputPayload::HostCommand {
                name: "response".to_owned(),
                payload: FlowHostValue::Object(BTreeMap::from([
                    ("a".to_owned(), FlowHostValue::String(host_chunk.clone())),
                    ("b".to_owned(), FlowHostValue::String(host_chunk)),
                ])),
            },
        };

        assert!(validate_bootstrap_payload(&bootstrap).is_ok());
        assert!(validate_output_batch(std::slice::from_ref(&output)).is_ok());

        let creation = FlowSessionCreation {
            bootstrap: bootstrap.clone(),
            outputs: vec![output.clone()],
            dirty: false,
            settled: true,
            wake_after_seconds: None,
        };
        let creation_error = validate_creation_value_arena_bounds(&creation)
            .expect_err("creation shares one aggregate ABI content budget");
        assert_eq!(
            creation_error.kind(),
            FlowSessionErrorKind::ResultLimitExceeded
        );

        let result = FlowResult {
            values: Some(bootstrap.values),
            outputs: vec![output],
            ..FlowResult::idle(true)
        };
        let result_error = validate_result_value_arena_bounds(&result)
            .expect_err("operation values and HostWork share one ABI content budget");
        assert_eq!(
            result_error.kind(),
            FlowSessionErrorKind::ResultLimitExceeded
        );
    }

    #[test]
    fn post_mutation_advance_failure_terminally_poisoned_session_rejects_every_later_operation() {
        let mut session = smi_session();
        session.next_sequence = u64::MAX;

        let first_error = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.016,
                render: false,
            }))
            .expect_err("sequence exhaustion fails after entering the mutation cycle");
        assert_eq!(first_error.kind(), FlowSessionErrorKind::LimitExceeded);

        let terminal_error = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect_err("a terminal session must reject even read operations");
        assert_eq!(terminal_error.kind(), FlowSessionErrorKind::Runtime);
        assert!(
            terminal_error
                .message()
                .contains("flow session is terminal")
        );

        let retry_error = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.016,
                render: false,
            }))
            .expect_err("a terminal session must never replay the failed frame");
        assert_eq!(retry_error, terminal_error);
    }

    #[test]
    fn active_pointer_limit_spans_batches_and_exit_releases_capacity() {
        let mut session = smi_session();
        let pointer = |kind, pointer_id| {
            FlowOperation::PointerBatch(FlowPointerBatch {
                events: vec![FlowPointerEvent {
                    kind,
                    pointer_id,
                    x: -10_000.0,
                    y: -10_000.0,
                    timestamp_seconds: 0.0,
                }],
            })
        };

        for pointer_id in 1..=MAX_POINTERS_PER_BATCH as i32 {
            session
                .perform(pointer(FlowPointerKind::Move, pointer_id))
                .expect("pointer within the session-wide cap");
        }
        let error = session
            .perform(pointer(
                FlowPointerKind::Move,
                MAX_POINTERS_PER_BATCH as i32 + 1,
            ))
            .expect_err("the active-pointer cap must span separate batches");
        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);

        session
            .perform(pointer(FlowPointerKind::Exit, 1))
            .expect("exit releases one active pointer");
        session
            .perform(pointer(
                FlowPointerKind::Move,
                MAX_POINTERS_PER_BATCH as i32 + 1,
            ))
            .expect("released capacity can be reused by another pointer");
    }

    #[test]
    fn schema_mapping_uses_canonical_identity_and_trigger_types() {
        assert_eq!(
            flow_value_type_for_property("ViewModelPropertySymbolListIndex"),
            FlowValueType::ListIndex
        );
        assert_eq!(
            flow_value_type_for_property("ViewModelPropertyArtboard"),
            FlowValueType::Enum
        );
        assert_eq!(
            flow_value_type_for_property("ViewModelPropertyTrigger"),
            FlowValueType::Trigger
        );
    }
}
